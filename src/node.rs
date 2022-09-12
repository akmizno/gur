use crate::action::Action;
use crate::memento::Memento;
use crate::metrics::Metrics;

pub(crate) enum Creator<'a, T: Memento> {
    Action(Box<dyn Action<State = T> + 'a>),
    Snapshot(Box<T::Target>),
}

impl<'a, T: Memento> Creator<'a, T> {
    pub(crate) fn from_action<A: Action<State = T> + 'a>(action: A) -> Self {
        Creator::Action(Box::new(action))
    }
    pub(crate) fn from_memento(state: &T) -> Self {
        Creator::Snapshot(Box::new(state.to_memento()))
    }

    pub(crate) fn get_if_memento(&self) -> Option<&T::Target> {
        match self {
            Self::Snapshot(m) => Some(m),
            _ => None,
        }
    }
    pub(crate) fn get_if_action(&self) -> Option<&(dyn Action<State = T> + 'a)> {
        match self {
            Self::Action(a) => Some(&**a),
            _ => None,
        }
    }
}

pub(crate) struct Node<'a, T: Memento> {
    creator: Creator<'a, T>,
    metrics: Metrics,
}

impl<'a, T: Memento> Node<'a, T> {
    pub(crate) fn from_action<A: Action<State = T> + 'a>(action: A, metrics: Metrics) -> Self {
        Self {
            creator: Creator::from_action(action),
            metrics: metrics,
        }
    }
    pub(crate) fn from_memento(state: &T) -> Self {
        Self {
            creator: Creator::from_memento(state),
            metrics: Metrics::zero(),
        }
    }

    pub(crate) fn creator(&self) -> &Creator<'a, T> {
        &self.creator
    }
    pub(crate) fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}
