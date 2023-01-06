use crate::gur::{Gur, GurBuilder};
use crate::interface::*;
use crate::metrics::Metrics;
use crate::snapshot::SnapshotHandler;

#[derive(Default)]
pub(crate) struct AgurBuilder<'a, T, S, H>(GurBuilder<'a, T, S, H>)
where
    H: SnapshotHandler<State = T, Snapshot = S>;

impl<'a, T, S, H> AgurBuilder<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    pub(crate) fn new() -> Self {
        Self(GurBuilder::new())
    }
}

impl<'a, T, S, H> IBuilder for AgurBuilder<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    type State = T;
    type Target = Agur<'a, T, S, H>;

    fn capacity(mut self, capacity: usize) -> Self {
        self.0 = self.0.capacity(capacity);
        self
    }

    fn build(self, initial_state: T) -> Agur<'a, T, S, H> {
        Agur::new(self.0.build(initial_state))
    }
}

impl<'a, T, S, H> IBuilderTriggerA<'a> for AgurBuilder<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    fn snapshot_trigger<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + Send + Sync + 'a,
    {
        self.0 = self.0.snapshot_trigger(f);
        self
    }
}

pub(crate) struct Agur<'a, T, S, H>(Gur<'a, T, S, H>)
where
    H: SnapshotHandler<State = T, Snapshot = S>;

impl<'a, T, S, H> Agur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    fn new(inner: Gur<'a, T, S, H>) -> Self {
        Self(inner)
    }
}

impl<'a, T, S, H> IUndoRedo for Agur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    type State = T;

    fn into_inner(self) -> T {
        self.0.into_inner()
    }

    fn capacity(&self) -> Option<usize> {
        self.0.capacity()
    }

    fn undoable_count(&self) -> usize {
        self.0.undoable_count()
    }

    fn redoable_count(&self) -> usize {
        self.0.redoable_count()
    }

    fn undo_multi(&mut self, count: usize) -> Option<&T> {
        self.0.undo_multi(count)
    }

    fn redo_multi(&mut self, count: usize) -> Option<&T> {
        self.0.redo_multi(count)
    }

    fn try_edit<F>(&mut self, command: F) -> Result<&T, Box<dyn std::error::Error>>
    where
        F: FnOnce(T) -> Result<T, Box<dyn std::error::Error>>,
    {
        self.0.try_edit(command)
    }
}

impl<'a, T, S, H> IEditA<'a> for Agur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    type State = T;

    fn edit_if<F>(&mut self, command: F) -> Option<&T>
    where
        F: Fn(T) -> Option<T> + Send + Sync + 'a,
    {
        self.0.edit_if(command)
    }
}

impl<'a, T: std::fmt::Debug, S, H> std::fmt::Debug for Agur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl<'a, T: std::fmt::Display, S, H> std::fmt::Display for Agur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl<'a, T, S, H> std::ops::Deref for Agur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

// NOTE
// Implementing the Send and Sync for Agur is safe,
// since Agur guarantees that all of stored commands and triggers implement the traits.
unsafe impl<'a, T: Send, S: Send, H> Send for Agur<'a, T, S, H> where
    H: SnapshotHandler<State = T, Snapshot = S>
{
}
unsafe impl<'a, T: Sync, S: Sync, H> Sync for Agur<'a, T, S, H> where
    H: SnapshotHandler<State = T, Snapshot = S>
{
}
