use crate::history::{History, Node};
use crate::interface::*;
use crate::metrics::Metrics;
use crate::snapshot::SnapshotHandler;
use crate::triggers::snapshot_trigger::snapshot_never;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

pub(crate) struct GurBuilder<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    capacity: usize,
    snapshot_trigger: Option<Box<dyn FnMut(&Metrics) -> bool + 'a>>,
    _snapshot_handler: PhantomData<H>,
}

impl<'a, T, S, H> GurBuilder<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    pub(crate) fn new() -> Self {
        Self {
            capacity: 0,
            snapshot_trigger: None,
            _snapshot_handler: PhantomData,
        }
    }
}

impl<'a, T, S, H> IBuilder for GurBuilder<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    type State = T;
    type Target = Gur<'a, T, S, H>;

    fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    fn build(self, initial_state: T) -> Gur<'a, T, S, H> {
        Gur::new(
            initial_state,
            self.capacity,
            self.snapshot_trigger.unwrap_or(Box::new(snapshot_never())),
        )
    }
}

impl<'a, T, S, H> ITrigger<'a> for GurBuilder<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    fn snapshot_trigger<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + 'a,
    {
        self.snapshot_trigger = Some(Box::new(f));
        self
    }
}

impl<'a, T, S, H> Default for GurBuilder<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct Gur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    state: Option<T>,

    history: History<Box<dyn Fn(T) -> T + 'a>, Box<S>>,

    snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + 'a>,
    _snapshot_handler: PhantomData<H>,
}

impl<'a, T, S, H> Gur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    fn get(&self) -> &T {
        debug_assert!(self.state.is_some());
        unsafe { self.state.as_ref().unwrap_unchecked() }
    }

    fn take(&mut self) -> T {
        debug_assert!(self.state.is_some());
        unsafe { self.state.take().unwrap_unchecked() }
    }

    fn new(
        initial_state: T,
        capacity: usize,
        snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + 'a>,
    ) -> Self {
        let first_node = Node::from_snapshot(Box::new(H::to_snapshot(&initial_state)));

        let history = if capacity == 0 {
            History::new_unlimited(first_node)
        } else {
            History::new(first_node, capacity)
        };

        Self {
            state: Some(initial_state),
            history,
            snapshot_trigger,
            _snapshot_handler: PhantomData,
        }
    }

    fn current_index(&self) -> usize {
        self.history.current_index()
    }

    // Regenerate a target state from history WITHOUT reusing the current state.
    fn regenerate(&mut self, target_idx: usize) -> T {
        let mut it = self.history.iter_from_last_snapshot(target_idx);

        let g = it.next().unwrap().generator();
        debug_assert!(g.is_snapshot());
        let mut state = H::from_snapshot(g.snapshot().unwrap());

        for node in it {
            debug_assert!(!node.generator().is_snapshot());
            let f = node.generator().command().unwrap();
            state = f(state);
        }

        state
    }

    fn redo_impl(&mut self, count: usize) {
        debug_assert!(0 < count);
        debug_assert!(count <= self.redoable_count());
        let current_idx = self.current_index();
        let target_idx = current_idx + count;
        let last_snapshot_idx = self.history.find_last_snapshot_index(target_idx);

        let state = if last_snapshot_idx.is_none() || current_idx < last_snapshot_idx.unwrap() {
            let _ = self.take(); // drop the current state before regeneration.
            self.regenerate(target_idx)
        } else {
            let mut state = self.take();

            let it = self.history.iter_from(current_idx).take(count + 1).skip(1);
            for node in it {
                debug_assert!(!node.generator().is_snapshot());
                let f = node.generator().command().unwrap();
                state = f(state);
            }

            state
        };

        self.state = Some(state);
        self.history.set_current(target_idx);
    }

    fn undo_impl(&mut self, count: usize) {
        debug_assert!(0 < count);
        debug_assert!(count <= self.undoable_count());
        let target_idx = self.current_index() - count;

        let _ = self.take(); // drop the current state before regeneration.

        self.state = Some(self.regenerate(target_idx));
        self.history.set_current(target_idx);
    }

    // Regenerate a target state from history WITHOUT reusing the current state.
    fn reset_state(&mut self, target_idx: usize) {
        // Drop the old state before running.
        self.state = None;

        self.state = Some(self.regenerate(target_idx));
        self.history.set_current(target_idx);
    }

    fn edit_if_impl<F>(command: &F, old_state: T) -> Option<(T, Duration)>
    where
        F: Fn(T) -> Option<T> + 'a,
    {
        let now = Instant::now();
        let new_state = command(old_state);
        let elapsed = now.elapsed();

        if let Some(new_state) = new_state {
            Some((new_state, elapsed))
        } else {
            None
        }
    }
}

impl<'a, T, S, H> IUndoRedo for Gur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    type State = T;

    fn into_inner(mut self) -> T {
        self.take()
    }

    fn capacity(&self) -> Option<usize> {
        let cap = self.history.capacity();
        if 0 < cap {
            Some(cap)
        } else {
            None
        }
    }

    fn undoable_count(&self) -> usize {
        self.history.len_before_current()
    }
    fn redoable_count(&self) -> usize {
        self.history.len_after_current()
    }

    fn undo_multi(&mut self, count: usize) -> Option<&T> {
        if 0 == count {
            // Nothing to do
            return Some(self.get());
        }

        if self.undoable_count() < count as usize {
            return None;
        }

        self.undo_impl(count);
        Some(self.get())
    }

    fn redo_multi(&mut self, count: usize) -> Option<&T> {
        if 0 == count {
            // Nothing to do
            return Some(self.get());
        }

        if self.redoable_count() < count {
            return None;
        }

        self.redo_impl(count);
        Some(self.get())
    }

    fn try_edit<F>(&mut self, command: F) -> Result<&T, Box<dyn std::error::Error>>
    where
        F: FnOnce(T) -> Result<T, Box<dyn std::error::Error>>,
    {
        let old_state = self.take();
        match command(old_state) {
            Ok(new_state) => {
                self.history
                    .push_node(Node::from_snapshot(Box::new(H::to_snapshot(&new_state))));

                self.state.replace(new_state);

                Ok(self.get())
            }
            Err(e) => {
                self.reset_state(self.history.current_index());
                Err(e)
            }
        }
    }
}

impl<'a, T, S, H> IEdit<'a> for Gur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    type State = T;

    fn edit_if<F>(&mut self, command: F) -> Option<&T>
    where
        F: Fn(T) -> Option<T> + 'a,
    {
        let old_state = self.take();

        let (new_state, elapsed) =
            if let Some((new_state, elapsed)) = Self::edit_if_impl(&command, old_state) {
                (new_state, elapsed)
            } else {
                // Reset the current state
                self.reset_state(self.history.current_index());
                return None;
            };

        let last_metrics = self.history.current().metrics();
        let new_metrics = last_metrics.make_next(elapsed);

        if (self.snapshot_trigger)(&new_metrics) {
            self.history
                .push_node(Node::from_snapshot(Box::new(H::to_snapshot(&new_state))));
        } else {
            self.history.push_node(Node::from_command(
                // This must succeed.
                Box::new(move |s| unsafe { command(s).unwrap_unchecked() }),
                new_metrics,
            ));
        }

        self.state.replace(new_state);
        Some(self.get())
    }
}

impl<'a, T: std::fmt::Debug, S, H> std::fmt::Debug for Gur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.get().fmt(f)
    }
}

impl<'a, T: std::fmt::Display, S, H> std::fmt::Display for Gur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.get().fmt(f)
    }
}

impl<'a, T, S, H> std::ops::Deref for Gur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'a, T, S, H> std::panic::UnwindSafe for Gur<'a, T, S, H>
where
    T: std::panic::UnwindSafe,
    H: SnapshotHandler<State = T, Snapshot = S>,
{
}

impl<'a, T, S, H> std::panic::RefUnwindSafe for Gur<'a, T, S, H>
where
    T: std::panic::RefUnwindSafe,
    H: SnapshotHandler<State = T, Snapshot = S>,
{
}
