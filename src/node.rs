use crate::metrics::Metrics;

pub(crate) enum Generator<'a, T, S> {
    Command(Box<dyn Fn(T) -> T + 'a>),
    Snapshot(Box<S>),
}

impl<'a, T, S> Generator<'a, T, S> {
    pub(crate) fn from_command<F>(command: F) -> Self
    where
        F: Fn(T) -> T + 'a,
    {
        Generator::Command(Box::new(command))
    }
    pub(crate) fn from_snapshot(snapshot: S) -> Self {
        Generator::Snapshot(Box::new(snapshot))
    }

    pub(crate) fn generate_if_snapshot(&self) -> Option<&S> {
        match self {
            Self::Snapshot(s) => Some(s),
            _ => None,
        }
    }
    pub(crate) fn generate_if_command(&self, prev: T) -> Option<T> {
        match self {
            Self::Command(f) => Some(f(prev)),
            _ => None,
        }
    }

    pub(crate) fn is_snapshot(&self) -> bool {
        match self {
            Self::Snapshot(_) => true,
            _ => false,
        }
    }
}

pub(crate) struct Node<'a, T, S> {
    generator: Generator<'a, T, S>,
    metrics: Metrics,
}

impl<'a, T, S> Node<'a, T, S> {
    pub(crate) fn from_command<F>(command: F, metrics: Metrics) -> Self
    where
        F: Fn(T) -> T + 'a,
    {
        Self {
            generator: Generator::from_command(command),
            metrics: metrics,
        }
    }
    pub(crate) fn from_snapshot(snapshot: S) -> Self {
        Self {
            generator: Generator::from_snapshot(snapshot),
            metrics: Metrics::zero(),
        }
    }

    pub(crate) fn generator(&self) -> &Generator<'a, T, S> {
        &self.generator
    }
    pub(crate) fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}
