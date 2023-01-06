use crate::gur::{Gur, GurBuilder};
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

impl<'a, T, S, H> AgurBuilder<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    pub(crate) fn capacity(mut self, capacity: usize) -> Self {
        self.0 = self.0.capacity(capacity);
        self
    }

    pub(crate) fn snapshot_trigger<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + Send + Sync + 'a,
    {
        self.0 = self.0.snapshot_trigger(f);
        self
    }

    pub(crate) fn build(self, initial_state: T) -> Agur<'a, T, S, H> {
        Agur::new(self.0.build(initial_state))
    }
}

pub(crate) struct Agur<'a, T, S, H>(Gur<'a, T, S, H>)
where
    H: SnapshotHandler<State = T, Snapshot = S>;

impl<'a, T, S, H> Agur<'a, T, S, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
{
    pub(crate) fn new(inner: Gur<'a, T, S, H>) -> Self {
        Self(inner)
    }

    pub(crate) fn into_inner(self) -> T {
        self.0.into_inner()
    }

    pub(crate) fn capacity(&self) -> Option<usize> {
        self.0.capacity()
    }

    pub(crate) fn undoable_count(&self) -> usize {
        self.0.undoable_count()
    }

    pub(crate) fn redoable_count(&self) -> usize {
        self.0.redoable_count()
    }

    pub(crate) fn undo(&mut self) -> Option<&T> {
        self.0.undo()
    }

    pub(crate) fn undo_multi(&mut self, count: usize) -> Option<&T> {
        self.0.undo_multi(count)
    }

    pub(crate) fn redo(&mut self) -> Option<&T> {
        self.0.redo()
    }

    pub(crate) fn redo_multi(&mut self, count: usize) -> Option<&T> {
        self.0.redo_multi(count)
    }

    pub(crate) fn jump(&mut self, count: isize) -> Option<&T> {
        self.0.jump(count)
    }

    pub(crate) fn edit<F>(&mut self, command: F) -> &T
    where
        F: Fn(T) -> T + Send + Sync + 'a,
    {
        self.0.edit(command)
    }

    pub(crate) fn edit_if<F>(&mut self, command: F) -> Option<&T>
    where
        F: Fn(T) -> Option<T> + Send + Sync + 'a,
    {
        self.0.edit_if(command)
    }

    pub(crate) fn try_edit<F>(&mut self, command: F) -> Result<&T, Box<dyn std::error::Error>>
    where
        F: FnOnce(T) -> Result<T, Box<dyn std::error::Error>>,
    {
        self.0.try_edit(command)
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
