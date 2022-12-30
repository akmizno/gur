use crate::metrics::Metrics;
use crate::snapshot::SnapshotHandler;
use std::collections::VecDeque;
use std::marker::PhantomData;

pub(crate) enum Generator<F, S> {
    Command(F),
    Snapshot(S),
}

pub(crate) struct Node<F, S> {
    generator: Generator<F, S>,
    metrics: Metrics,
}

impl<F, S> Node<F, S> {
    pub(crate) fn from_command(command: F, metrics: Metrics) -> Self {
        Self {
            generator: Generator::Command(command),
            metrics: metrics,
        }
    }
    pub(crate) fn from_snapshot(snapshot: S) -> Self {
        Self {
            generator: Generator::Snapshot(snapshot),
            metrics: Metrics::zero(),
        }
    }

    pub(crate) fn generator(&self) -> &Generator<F, S> {
        &self.generator
    }

    pub(crate) fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}

pub(crate) struct History<T, S, F, H>
where
    H: SnapshotHandler<State = T, Snapshot = S>,
    F: Fn(T) -> T,
{
    inner: VecDeque<Node<F, S>>,
    current: usize,

    _phantom: PhantomData<(T, H)>,
}

// impl<H, F> History<H, F>
// where
//     H: SnapshotHandler,
//     F: Fn(<H as SnapshotHandler>::State) -> <H as SnapshotHandler>::State,
// {
//     type T = i32;
// }
