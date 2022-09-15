use crate::metrics::Metrics;

pub(crate) enum Generator<'a, T> {
    Command(Box<dyn Fn(T) -> T + Send + Sync + 'a>),
    Snapshot(Box<T>),
}

impl<'a, T: Clone> Generator<'a, T> {
    pub(crate) fn from_command<F>(command: F) -> Self
    where
        F: Fn(T) -> T + Send + Sync + 'a,
    {
        Generator::Command(Box::new(command))
    }
    pub(crate) fn from_state(state: &T) -> Self {
        Generator::Snapshot(Box::new(state.clone()))
    }

    pub(crate) fn generate_if_snapshot(&self) -> Option<T> {
        match self {
            Self::Snapshot(s) => Some(*s.clone()),
            _ => None,
        }
    }
    pub(crate) fn generate_if_command(&self, prev: T) -> Option<T> {
        match self {
            Self::Command(f) => Some(f(prev)),
            _ => None,
        }
    }

    pub(crate) fn generate(&self, prev: T) -> T {
        match self {
            Self::Command(f) => f(prev),
            Self::Snapshot(s) => *s.clone(),
        }
    }
}

pub(crate) struct Node<'a, T> {
    generator: Generator<'a, T>,
    metrics: Metrics,
}

impl<'a, T: Clone> Node<'a, T> {
    pub(crate) fn from_command<F>(command: F, metrics: Metrics) -> Self
    where
        F: Fn(T) -> T + Send + Sync + 'a,
    {
        Self {
            generator: Generator::from_command(command),
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
