use crate::metrics::Metrics;
use crate::snapshot::Snapshot;
use crate::ur::{Ur, UrBuilder};

#[derive(Default)]
pub struct AurBuilder<'a, T, S>(UrBuilder<'a, T, S>)
where
    T: Snapshot<Snapshot = S>;

impl<'a, T, S> AurBuilder<'a, T, S>
where
    T: Snapshot<Snapshot = S>,
{
    pub fn new() -> Self {
        Self(UrBuilder::new())
    }

    pub fn snapshot_trigger<F>(self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + Send + Sync + 'a,
    {
        Self(self.0.snapshot_trigger(f))
    }

    pub fn build(self, initial_state: T) -> Aur<'a, T, S> {
        Aur::new(self.0.build(initial_state))
    }
}

/// Ur<T> + Send + Sync
pub struct Aur<'a, T, S>(Ur<'a, T, S>)
where
    T: Snapshot<Snapshot = S>;

impl<'a, T, S> Aur<'a, T, S>
where
    T: Snapshot<Snapshot = S>,
{
    fn new(ur: Ur<'a, T, S>) -> Self {
        Self(ur)
    }
    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }
    pub fn undo(&mut self) -> Option<&T> {
        self.0.undo()
    }
    pub fn undo_multi(&mut self, count: usize) -> Option<&T> {
        self.0.undo_multi(count)
    }
    pub fn redo(&mut self) -> Option<&T> {
        self.0.redo()
    }
    pub fn redo_multi(&mut self, count: usize) -> Option<&T> {
        self.0.redo_multi(count)
    }
    pub fn jumpdo(&mut self, count: isize) -> Option<&T> {
        self.0.jumpdo(count)
    }

    pub fn edit<F>(&mut self, command: F) -> &T
    where
        F: Fn(T) -> T + Send + Sync + 'a,
    {
        self.0.edit(command)
    }

    pub fn edit_if<F>(&mut self, command: F) -> Option<&T>
    where
        F: Fn(T) -> Option<T> + Send + Sync + 'a,
    {
        self.0.edit_if(command)
    }

    pub fn try_edit<F>(&mut self, command: F) -> Result<&T, Box<dyn std::error::Error>>
    where
        F: FnOnce(T) -> Result<T, Box<dyn std::error::Error>>,
    {
        self.0.try_edit(command)
    }
}

// NOTE
// Implementing the Send and Sync for Aur<T> is safe,
// since Aur<T> guarantees that all of stored commands and triggers implement the traits.
unsafe impl<'a, T: Send, S: Send> Send for Aur<'a, T, S> where T: Snapshot<Snapshot = S> {}
unsafe impl<'a, T: Sync, S: Sync> Sync for Aur<'a, T, S> where T: Snapshot<Snapshot = S> {}

impl<'a, T, S> std::fmt::Debug for Aur<'a, T, S>
where
    T: Snapshot<Snapshot = S> + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl<'a, T, S> std::fmt::Display for Aur<'a, T, S>
where
    T: Snapshot<Snapshot = S> + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl<'a, T, S> std::ops::Deref for Aur<'a, T, S>
where
    T: Snapshot<Snapshot = S>,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
