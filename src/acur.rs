use crate::cur::{Cur, CurBuilder};
use crate::metrics::Metrics;

#[derive(Default)]
pub struct AcurBuilder<'a, T: Clone>(CurBuilder<'a, T>);

impl<'a, T: Clone> AcurBuilder<'a, T> {
    pub fn new() -> Self {
        Self(CurBuilder::new())
    }

    pub fn snapshot_trigger<F>(self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + Send + Sync + 'a,
    {
        Self(self.0.snapshot_trigger(f))
    }

    pub fn build(self, initial_state: T) -> Acur<'a, T> {
        Acur::new(self.0.build(initial_state))
    }
}

/// Cur<T> + Send + Sync
pub struct Acur<'a, T: Clone>(Cur<'a, T>);

impl<'a, T: Clone> Acur<'a, T> {
    fn new(ur: Cur<'a, T>) -> Self {
        Self(ur)
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
// Implementing the Send and Sync for Acur<T> is safe,
// since Acur<T> guarantees that all of stored commands and triggers implement the traits.
unsafe impl<'a, T: Clone + Send> Send for Acur<'a, T> {}
unsafe impl<'a, T: Clone + Sync> Sync for Acur<'a, T> {}

impl<'a, T: Clone + std::fmt::Debug> std::fmt::Debug for Acur<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl<'a, T: Clone + std::fmt::Display> std::fmt::Display for Acur<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl<'a, T: Clone> std::ops::Deref for Acur<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
