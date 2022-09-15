use crate::metrics::Metrics;
use crate::snapshot::Snapshot;

pub(crate) enum Generator<'a, T: Snapshot> {
    Editor(Box<dyn Fn(T) -> T + Send + Sync + 'a>),
    Snapshot(Box<T::Target>),
}

impl<'a, T: Snapshot> Generator<'a, T> {
    pub(crate) fn from_editor<F>(editor: F) -> Self
    where
        F: Fn(T) -> T + Send + Sync + 'a,
    {
        Generator::Editor(Box::new(editor))
    }
    pub(crate) fn from_state(state: &T) -> Self {
        Generator::Snapshot(Box::new(state.to_snapshot()))
    }

    pub(crate) fn generate_if_snapshot(&self) -> Option<T> {
        match self {
            Self::Snapshot(m) => Some(T::from_snapshot(m)),
            _ => None,
        }
    }
    pub(crate) fn generate_if_editor(&self, prev: T) -> Option<T> {
        match self {
            Self::Editor(ed) => Some(ed(prev)),
            _ => None,
        }
    }

    pub(crate) fn generate(&self, prev: T) -> T {
        match self {
            Self::Editor(ed) => ed(prev),
            Self::Snapshot(m) => T::from_snapshot(m),
        }
    }
}

pub(crate) struct Node<'a, T: Snapshot> {
    generator: Generator<'a, T>,
    metrics: Metrics,
}

impl<'a, T: Snapshot> Node<'a, T> {
    pub(crate) fn from_editor<F>(action: F, metrics: Metrics) -> Self
    where
        F: Fn(T) -> T + Send + Sync + 'a,
    {
        Self {
            generator: Generator::from_editor(action),
            metrics: metrics,
        }
    }
    pub(crate) fn from_state(state: &T) -> Self {
        Self {
            generator: Generator::from_state(state),
            metrics: Metrics::zero(),
        }
    }

    pub(crate) fn generator(&self) -> &Generator<'a, T> {
        &self.generator
    }
    pub(crate) fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}
