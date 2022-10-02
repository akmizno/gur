use crate::metrics::Metrics;
use crate::node::Node;
use crate::snapshot::SnapshotHandler;
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
    /// Create a new builder instance.
    pub(crate) fn new() -> Self {
        Self {
            snapshot_trigger: None,
            _snapshot_handler: PhantomData,
        }
    }

    /// Takes a closure to decide whether to save a snapshot of internal state.
    ///
    /// See [Snapshot trigger](crate::triggers#Snapshot&#32;trigger) for more details.
    pub(crate) fn snapshot_trigger<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + 'a,
    {
        self.snapshot_trigger = Some(Box::new(f));
        self
    }

    /// Create a new [Gur] object by the initial state of T.
    pub(crate) fn build(self, initial_state: T) -> Gur<'a, T, S, H> {
        Gur::new(
            initial_state,
            self.snapshot_trigger.unwrap_or(Box::new(|_m| false)),
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
    // Some means that the current state is owned by itself.
    // None means that the current state is not owned by this variable.
    // In this case, the self.history[self.current] should be accessed as a snapshot node and
    // it is used as a current state.
    state: Option<T>,

    history: Vec<Node<'a, T, S>>,
    current: usize,

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

    unsafe fn restore_from_snapshot(&self, target: usize) -> T {
        debug_assert!(target < self.history.len());
        debug_assert!(self.history[target].generator().is_snapshot());

        let snapshot = self
            .history
            .get_unchecked(target)
            .generator()
            .generate_if_snapshot()
            .unwrap_unchecked();

        H::from_snapshot(snapshot)
    }

    fn take(&mut self) -> T {
        debug_assert!(self.state.is_some());
        unsafe { self.state.take().unwrap_unchecked() }
    }

    pub(crate) fn new(
        initial_state: T,
        snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + 'a>,
    ) -> Self {
        let first_node = Node::from_snapshot(H::to_snapshot(&initial_state));
        Self {
            state: Some(initial_state),
            history: vec![first_node],
            current: 0,
            snapshot_trigger,
            _snapshot_handler: PhantomData,
        }
    }

    /// Restore the previous state.
    ///
    /// Same as `self.undo_multi(1)`.
    ///
    /// # Return
    /// [None] is returned if there is no older version in the history,
    /// otherwise immutable reference to the updated internal state.
    pub(crate) fn undo(&mut self) -> Option<&T> {
        self.undo_multi(1)
    }

    /// Undo multiple steps.
    ///
    /// This method is more efficient than running `self.undo()` multiple times.
    ///
    /// # Return
    /// [None] is returned if the target version is out of the history,
    /// otherwise immutable reference to the updated internal state.
    /// If `count=0`, this method does nothing and returns reference to the current state.
    pub(crate) fn undo_multi(&mut self, count: usize) -> Option<&T> {
        debug_assert!(count < isize::MAX as usize);
        self.jumpdo(-(count as isize))
    }

    /// Restore the next state.
    ///
    /// Same as `self.redo_multi(1)`.
    ///
    /// # Return
    /// [None] is returned if there is no newer version in the history,
    /// otherwise immutable reference to the updated internal state.
    pub(crate) fn redo(&mut self) -> Option<&T> {
        self.redo_multi(1)
    }

    /// Redo multiple steps.
    ///
    /// This method is more efficient than running `self.redo()` multiple times.
    ///
    /// # Return
    /// [None] is returned if the target version is out of the history,
    /// otherwise immutable reference to the updated internal state.
    /// If `count=0`, this method does nothing and returns reference to the current state.
    pub(crate) fn redo_multi(&mut self, count: usize) -> Option<&T> {
        debug_assert!(count < isize::MAX as usize);
        self.jumpdo(count as isize)
    }

    /// Undo-redo bidirectionally.
    ///
    /// This is integrated method of [undo_multi](Gur::undo_multi) and [redo_multi](Gur::redo_multi).
    ///
    /// - `count < 0` => `self.undo_multi(-count)`.
    /// - `0 < count` => `self.redo_multi(count)`.
    pub(crate) fn jumpdo(&mut self, count: isize) -> Option<&T> {
        if 0 == count {
            // Nothing to do
            return Some(self.get());
        }

        // Check the argment
        if count < 0 {
            // Undo
            if self.current < count.abs() as usize {
                return None;
            }
        } else {
            // Redo
            if self.history.len() <= self.current + count.abs() as usize {
                return None;
            }
        }

        self.jumpdo_impl(count);
        Some(self.get())
    }

    fn get_regeneration_range(&self, target: usize) -> (usize, usize) {
        debug_assert!(target < self.history.len());
        let last_node = unsafe { self.history.get_unchecked(target) };
        let dist = last_node.metrics().distance_from_snapshot();
        debug_assert!(dist <= target);
        let first = target - dist;
        debug_assert!(self.history[first].generator().is_snapshot());
        (first, target + 1)
    }

    fn regenerate(first_state: T, history: &[Node<'a, T, S>]) -> T {
        let mut state = first_state;
        for node in history {
            let next = node.generator().generate_if_command(state);
            debug_assert!(next.is_some());
            state = unsafe { next.unwrap_unchecked() };
        }

        state
    }

    fn jumpdo_impl(&mut self, count: isize) {
        debug_assert!(count != 0);

        let target = if count < 0 {
            debug_assert!(count.abs() as usize <= self.current);
            self.current - count.abs() as usize
        } else {
            self.current + count as usize
        };

        debug_assert!(target < self.history.len());

        let (begin, end) = self.get_regeneration_range(target);

        let (first_state, begin) = if begin <= self.current && self.current < end {
            // Reuse current state
            debug_assert!(self.state.is_some());
            (self.take(), self.current)
        } else {
            // The current state is not resusable.

            // Drop the old state before runnning.
            self.state = None;

            let restored = unsafe { self.restore_from_snapshot(begin) };
            (restored, begin)
        };

        self.state = Some(Self::regenerate(first_state, &self.history[begin + 1..end]));
        self.current = target;
    }

    // Regenerate a target state from history WITHOUT reusing the current state.
    fn reset_state(&mut self, target: usize) {
        // Drop the old state before running.
        self.state = None;

        let (begin, end) = self.get_regeneration_range(target);

        let first_state = unsafe { self.restore_from_snapshot(begin) };
        self.state = Some(Self::regenerate(first_state, &self.history[begin + 1..end]));
        self.current = target;
    }

    /// Takes a closure and update the internal state.
    ///
    /// The closure consumes the current state and produces a new state.
    ///
    /// # Return
    /// Immutable reference to the new state.
    ///
    /// # Remarks
    /// The closure MUST produce a same result for a same input.
    /// If it is impossible, use [try_edit](Gur::try_edit).
    pub(crate) fn edit<F>(&mut self, command: F) -> &T
    where
        F: Fn(T) -> T + 'a,
    {
        // NOTE
        // This call is guaranteed to succeed.
        unsafe { self.edit_if(move |s| Some(command(s))).unwrap_unchecked() }
    }

    /// Takes a closure and update the internal state.
    ///
    /// The closure consumes the current state and produces a new state or [None].
    /// If the closure returns [None], the internal state is not changed.
    ///
    /// # Return
    /// Immutable reference to the new state or [None].
    ///
    /// # Remarks
    /// The closure MUST produce a same result for a same input.
    /// If it is impossible, use [try_edit](Gur::try_edit).
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
                self.reset_state(self.current);
                return None;
            };

        self.history.truncate(self.current + 1);

        let last_metrics = self.history.last().unwrap().metrics();
        let new_metrics = last_metrics.make_next(elapsed);

        if (self.snapshot_trigger)(&new_metrics) {
            self.history
                .push(Node::from_snapshot(H::to_snapshot(&new_state)));
        } else {
            self.history.push(Node::from_command(
                // This must succeed.
                move |s| unsafe { command(s).unwrap_unchecked() },
                new_metrics,
            ));
        }

        self.state.replace(new_state);
        self.current += 1;
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

    /// Takes a closure and update the internal state.
    ///
    /// The closure consumes the current state and produces a new state or an error.
    /// If the closure returns an error, the internal state is not changed.
    ///
    /// # Return
    /// Immutable reference to the new state or an error produced by the closure.
    ///
    /// # Remark
    /// In this method, the produced state from the closure is stored as a snapshot always;
    /// because the type of closure is [FnOnce], same output can not be reproducible never again.
    pub(crate) fn try_edit<F>(&mut self, command: F) -> Result<&T, Box<dyn std::error::Error>>
    where
        F: FnOnce(T) -> Result<T, Box<dyn std::error::Error>>,
    {
        let old_state = self.take();
        match command(old_state) {
            Ok(new_state) => {
                self.history.truncate(self.current + 1);
                self.history
                    .push(Node::from_snapshot(H::to_snapshot(&new_state)));

                self.state.replace(new_state);
                self.current += 1;

                Ok(self.get())
            }
            Err(e) => {
                self.reset_state(self.current);
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
