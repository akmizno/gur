use std::marker::PhantomData;
pub trait Snapshot {
    type Snapshot;
    fn to_snapshot(&self) -> Self::Snapshot;
    fn from_snapshot(snapshot: &Self::Snapshot) -> Self;
}

pub(crate) trait SnapshotHandler {
    type State;
    type Snapshot;
    fn to_snapshot(state: &Self::State) -> Self::Snapshot;
    fn from_snapshot(snapshot: &Self::Snapshot) -> Self::State;
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
