use crate::agur::{Agur, AgurBuilder};
use crate::interface::*;
use crate::metrics::Metrics;
use crate::snapshot::CloneSnapshot;

#[derive(Default)]
pub struct AcurBuilder<'a, T: Clone>(AgurBuilder<'a, T, T, CloneSnapshot<T>>);

impl<'a, T: Clone> AcurBuilder<'a, T> {
    pub fn new() -> Self {
        Self(AgurBuilder::new())
    }
}

impl<'a, T: Clone> IBuilder for AcurBuilder<'a, T> {
    type State = T;
    type Target = Acur<'a, T>;

    fn capacity(mut self, capacity: usize) -> Self {
        self.0 = self.0.capacity(capacity);
        self
    }

    fn build(self, initial_state: T) -> Acur<'a, T> {
        Acur::new(self.0.build(initial_state))
    }
}

impl<'a, T: Clone> IBuilderTriggerA<'a> for AcurBuilder<'a, T> {
    fn snapshot_trigger<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + Send + Sync + 'a,
    {
        self.0 = self.0.snapshot_trigger(f);
        self
    }
}

/// [Cur](crate::cur::Cur) + [Send] + [Sync]
#[derive(Debug)]
pub struct Acur<'a, T: Clone>(Agur<'a, T, T, CloneSnapshot<T>>);

impl<'a, T: Clone> Acur<'a, T> {
    fn new(ur: Agur<'a, T, T, CloneSnapshot<T>>) -> Self {
        Self(ur)
    }
}

impl<'a, T: Clone> IUndoRedo for Acur<'a, T> {
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

impl<'a, T: Clone> IEditA<'a> for Acur<'a, T> {
    type State = T;

    fn edit_if<F>(&mut self, command: F) -> Option<&T>
    where
        F: Fn(T) -> Option<T> + Send + Sync + 'a,
    {
        self.0.edit_if(command)
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
