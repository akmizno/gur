//! Thread-safe [Ur\<T\>](crate::ur::Ur)
use crate::agur::{Agur, AgurBuilder};
use crate::interface::*;
use crate::metrics::Metrics;
use crate::snapshot::{Snapshot, TraitSnapshot};

/// A builder to create an [Aur].
///
/// # Interface
/// [AurBuilder] implements following interfaces.
///
/// - [IBuilder](crate::interface::IBuilder)
/// - [ITriggerA](crate::interface::ITriggerA)
///
/// See the pages to view method list.
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

impl<'a, T, S> ITriggerA<'a> for AurBuilder<'a, T, S>
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
///
/// See also [AurBuilder].
///
/// # Sample code
/// ```
/// use gur::prelude::*;
/// use gur::aur::{Aur, AurBuilder};
/// use gur::snapshot::Snapshot;
///
/// // Appication state
/// #[derive(Clone)]
/// struct MyState {
///     data: String
/// }
///
/// // Implementing Snapshot trait
/// impl Snapshot for MyState {
///     type Snapshot = String;
///     fn to_snapshot(&self) -> Self::Snapshot {
///         self.data.clone()
///     }
///     fn from_snapshot(snapshot: &Self::Snapshot) -> Self {
///         MyState{ data: snapshot.clone() }
///     }
/// }
///
/// fn main() {
///     // Initialize
///     let mut state = AurBuilder::new().build(MyState{ data: "My".to_string() });
///     assert_eq!("My", state.data);
///
///     // Change state
///     state.edit(|mut state| { state.data += "State"; state });
///     assert_eq!("MyState", state.data);
///
///     // Undo
///     state.undo();
///     assert_eq!("My", state.data);
///
///     // Redo
///     state.redo();
///     assert_eq!("MyState", state.data);
/// }
/// ```
///
/// # Information
/// ## Snapshot trait
/// [Aur] requires a type `T` implementing [Snapshot](crate::snapshot::Snapshot).
/// The trait specifies conversion between `T` and its snapshot object.
///
/// [Acur](crate::acur::Acur) may be more suitable for simple types.
/// It requires [Clone] instead of [Snapshot](crate::snapshot::Snapshot).
/// See [Acur](crate::acur::Acur) for more detail.
///
/// ## Provided methods
/// [Aur] implements following undo-redo interfaces.
///
/// - [IUndoRedo](crate::interface::IUndoRedo)
/// - [IEditA](crate::interface::IEditA)
///
/// See the pages to view method list.
///
/// ## Thread-safety
/// [Aur] implements [Send] and [Sync].
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
