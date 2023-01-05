use std::marker::PhantomData;

/// A trait for ability to convert between an object and its snapshot.
///
/// [Ur](crate::ur::Ur) and [Aur](crate::aur::Aur) requires types implementing this trait.
pub trait Snapshot {
    /// Type of snapshot.
    type Snapshot;
    /// Create a snapshot object.
    fn to_snapshot(&self) -> Self::Snapshot;
    /// Restore from a snapshot.
    fn from_snapshot(snapshot: &Self::Snapshot) -> Self;
}

/// Internal snapshot handler.
///
///```txt
///          to_snapshot
///       ---------------->
/// State  SnapshotHandler  Snapshot
///       <----------------
///         from_snapshot
///```
pub(crate) trait SnapshotHandler {
    type State;
    type Snapshot;
    fn to_snapshot(state: &Self::State) -> Self::Snapshot;
    fn from_snapshot(snapshot: &Self::Snapshot) -> Self::State;
}

/// Snapshot handler for [Clone] implementors.
#[derive(Clone, Debug, Default)]
pub(crate) struct CloneSnapshot<T>(PhantomData<T>);

impl<T: Clone> SnapshotHandler for CloneSnapshot<T> {
    type State = T;
    type Snapshot = T;
    fn to_snapshot(state: &Self::State) -> Self::Snapshot {
        state.clone()
    }
    fn from_snapshot(snapshot: &Self::Snapshot) -> Self::State {
        snapshot.clone()
    }
}

/// Snapshot handler for [Snapshot] implementors.
#[derive(Clone, Debug, Default)]
pub(crate) struct TraitSnapshot<T: Snapshot<Snapshot = S>, S>(PhantomData<T>);

impl<T: Snapshot<Snapshot = S>, S> SnapshotHandler for TraitSnapshot<T, S> {
    type State = T;
    type Snapshot = S;
    fn to_snapshot(state: &Self::State) -> Self::Snapshot {
        state.to_snapshot()
    }
    fn from_snapshot(snapshot: &Self::Snapshot) -> Self::State {
        Self::State::from_snapshot(snapshot)
    }
}
