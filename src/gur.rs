use crate::history::{History, Node};
use crate::metrics::Metrics;
use crate::snapshot::SnapshotHandler;
use crate::triggers::snapshot_trigger::snapshot_never;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

pub(crate) struct GurBuilder<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    snapshot_trigger: Option<Box<dyn FnMut(&Metrics) -> bool + 'a>>,
    _snapshot_handler: PhantomData<H>,
}

impl<'a, T, S, H> GurBuilder<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    pub(crate) fn new() -> Self {
        Self {
            snapshot_trigger: None,
            _snapshot_handler: PhantomData,
        }
    }

    pub(crate) fn snapshot_trigger<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + 'a,
    {
        self.snapshot_trigger = Some(Box::new(f));
        self
    }

    pub(crate) fn build(self, initial_state: T) -> Gur<'a, T, S, H> {
        Gur::new(
            initial_state,
            self.snapshot_trigger.unwrap_or(Box::new(snapshot_never())),
        )
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

    pub(crate) fn into_inner(mut self) -> T {
        self.take()
    }

    pub(crate) fn new(
        initial_state: T,
        snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + 'a>,
    ) -> Self {
        let first_node = Node::from_snapshot(Box::new(H::to_snapshot(&initial_state)));
        Self {
            state: Some(initial_state),
            history: History::new_unlimited(first_node),
            snapshot_trigger,
            _snapshot_handler: PhantomData,
        }
    }

    pub(crate) fn undo(&mut self) -> Option<&T> {
        self.undo_multi(1)
    }

    pub(crate) fn undo_multi(&mut self, count: usize) -> Option<&T> {
        if 0 == count {
            // Nothing to do
            return Some(self.get());
        }

        if self.history.len_before_current() < count as usize {
            return None;
        }

        self.undo_impl(count);
        Some(self.get())
    }

    pub(crate) fn redo(&mut self) -> Option<&T> {
        self.redo_multi(1)
    }

    pub(crate) fn redo_multi(&mut self, count: usize) -> Option<&T> {
        if 0 == count {
            // Nothing to do
            return Some(self.get());
        }

        if self.history.len_after_current() < count {
            return None;
        }

        self.redo_impl(count);
        Some(self.get())
    }

    pub(crate) fn jump(&mut self, count: isize) -> Option<&T> {
        if count < 0 {
            self.undo_multi(count.abs() as usize)
        } else {
            self.redo_multi(count as usize)
        }
    }

    fn current(&self) -> usize {
        self.history.current_index()
    }
    fn redo_impl(&mut self, count: usize) {
        debug_assert!(0 < count);
        let current_idx = self.current();
        let target_idx = current_idx + count;
        let last_snapshot_idx = self.history.find_last_snapshot_index(target_idx);

        let (mut state, begin, cmd_count) = if last_snapshot_idx <= current_idx {
            (self.take(), current_idx, count)
        } else {
            let _ = self.take(); // drop the current state before a first state restored.

            let g = self.history.get_node(last_snapshot_idx).generator();
            debug_assert!(g.is_snapshot());
            let first_state = H::from_snapshot(g.snapshot().unwrap());

            debug_assert!(last_snapshot_idx <= target_idx);
            let cmd_count = target_idx - last_snapshot_idx;

            (first_state, last_snapshot_idx, cmd_count)
        };

        if 0 < cmd_count {
            let iter = self.history.iter_from(begin + 1).take(cmd_count);
            for node in iter {
                debug_assert!(!node.generator().is_snapshot());
                let f = node.generator().command().unwrap();
                state = f(state);
            }
        }

        self.state = Some(state);
        self.history.set_current(begin + cmd_count);
    }

    fn undo_impl(&mut self, count: usize) {
        debug_assert!(0 < count);
        debug_assert!(count <= self.current());
        let target_idx = self.current() - count;
        let last_snapshot_idx = self.history.find_last_snapshot_index(target_idx);

        let (mut state, begin, cmd_count) = {
            let _ = self.take(); // drop the current state before a first state restored.

            let g = self.history.get_node(last_snapshot_idx).generator();
            debug_assert!(g.is_snapshot());
            let first_state = H::from_snapshot(g.snapshot().unwrap());

            let cmd_count = target_idx - last_snapshot_idx;

            (first_state, last_snapshot_idx, cmd_count)
        };

        if 0 < cmd_count {
            let iter = self.history.iter_from(begin + 1).take(cmd_count);

            for node in iter {
                debug_assert!(!node.generator().is_snapshot());
                let f = node.generator().command().unwrap();
                state = f(state);
            }
        }

        self.state = Some(state);
        self.history.set_current(target_idx);
    }

    // Regenerate a target state from history WITHOUT reusing the current state.
    fn reset_state(&mut self, target: usize) {
        // Drop the old state before running.
        self.state = None;

        let mut it = self.history.iter_from_last_snapshot(target);
        let snapshot = it.next().unwrap().generator().snapshot().unwrap();
        let mut state = H::from_snapshot(snapshot);
        for command_node in it {
            let command = command_node.generator().command().unwrap();
            state = command(state);
        }

        self.state = Some(state);
        self.history.set_current(target);
    }

    pub(crate) fn edit<F>(&mut self, command: F) -> &T
    where
        F: Fn(T) -> T + 'a,
    {
        // NOTE
        // This call is guaranteed to succeed.
        unsafe { self.edit_if(move |s| Some(command(s))).unwrap_unchecked() }
    }

    pub(crate) fn edit_if<F>(&mut self, command: F) -> Option<&T>
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

    pub(crate) fn try_edit<F>(&mut self, command: F) -> Result<&T, Box<dyn std::error::Error>>
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
