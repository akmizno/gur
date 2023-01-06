use crate::agur::{Agur, AgurBuilder};
use crate::interface::*;
use crate::metrics::Metrics;
use crate::snapshot::{Snapshot, TraitSnapshot};

#[derive(Default)]
pub struct AurBuilder<'a, T, S>(AgurBuilder<'a, T, S, TraitSnapshot<T, S>>)
where
    T: Snapshot<Snapshot = S>;

impl<'a, T, S> AurBuilder<'a, T, S>
where
    T: Snapshot<Snapshot = S>,
{
    pub fn new() -> Self {
        Self(AgurBuilder::new())
    }
}

impl<'a, T, S> IBuilder for AurBuilder<'a, T, S>
where
    T: Snapshot<Snapshot = S>,
{
    type State = T;
    type Target = Aur<'a, T, S>;

    fn capacity(mut self, capacity: usize) -> Self {
        self.0 = self.0.capacity(capacity);
        self
    }

    fn build(self, initial_state: T) -> Aur<'a, T, S> {
        Aur::new(self.0.build(initial_state))
    }
}

impl<'a, T, S> IBuilderTriggerA<'a> for AurBuilder<'a, T, S>
where
    T: Snapshot<Snapshot = S>,
{
    fn snapshot_trigger<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + Send + Sync + 'a,
    {
        self.0 = self.0.snapshot_trigger(f);
        self
    }
}

/// [Ur](crate::ur::Ur) + [Send] + [Sync]
pub struct Aur<'a, T, S>(Agur<'a, T, S, TraitSnapshot<T, S>>)
where
    T: Snapshot<Snapshot = S>;

impl<'a, T, S> Aur<'a, T, S>
where
    T: Snapshot<Snapshot = S>,
{
    fn new(ur: Agur<'a, T, S, TraitSnapshot<T, S>>) -> Self {
        Self(ur)
    }
}

impl<'a, T, S> IUndoRedo for Aur<'a, T, S>
where
    T: Snapshot<Snapshot = S>,
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

impl<'a, T, S> IEditA<'a> for Aur<'a, T, S>
where
    T: Snapshot<Snapshot = S>,
{
    type State = T;

    fn edit_if<F>(&mut self, command: F) -> Option<&T>
    where
        F: Fn(T) -> Option<T> + Send + Sync + 'a,
    {
        self.0.edit_if(command)
    }
}

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
