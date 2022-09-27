use std::marker::PhantomData;

pub trait SnapshotMaker {
    type State;
    type Snapshot;
    fn to_snapshot(&self, state: &Self::State) -> Self::Snapshot;
    fn from_snapshot(&self, snapshot: &Self::Snapshot) -> Self::State;
}

#[derive(Clone, Debug)]
pub struct CloneMaker<T>(PhantomData<T>);

impl<T> CloneMaker<T> {
    pub const fn new() -> Self {
        Self(PhantomData::<T>)
    }
}

impl<T: Clone> SnapshotMaker for CloneMaker<T> {
    type State = T;
    type Snapshot = T;
    fn to_snapshot(&self, state: &Self::State) -> Self::Snapshot {
        state.clone()
    }
    fn from_snapshot(&self, snapshot: &Self::Snapshot) -> Self::State {
        snapshot.clone()
    }
}
