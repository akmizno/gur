use crate::action::Action;
use crate::memento::Memento;

pub(crate) enum Node<'a, T: Memento> {
    Action(Box<dyn Action<State = T> + 'a>),
    Memento(Box<T::Target>),
}

impl<'a, T: Memento> Node<'a, T> {
    pub(crate) fn from_action<A: Action<State = T> + 'a>(action: A) -> Self {
        Node::Action(Box::new(action))
    }
    pub(crate) fn from_memento(state: &T) -> Self {
        Node::Memento(Box::new(state.to_memento()))
    }

    pub(crate) fn get_if_memento(&self) -> Option<&T::Target> {
        match self {
            Self::Memento(m) => Some(m),
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
