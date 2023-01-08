use crate::metrics::Metrics;
use std::collections::VecDeque;

pub(crate) enum Generator<F, S> {
    Command(F),
    Snapshot(S),
}

impl<F, S> Generator<F, S> {
    pub(crate) fn is_snapshot(&self) -> bool {
        matches!(self, Generator::Snapshot(_))
    }

    pub(crate) fn snapshot(&self) -> Option<&S> {
        match self {
            Generator::Snapshot(s) => Some(s),
            Generator::Command(_) => None,
        }
    }
    pub(crate) fn command(&self) -> Option<&F> {
        match self {
            Generator::Snapshot(_) => None,
            Generator::Command(f) => Some(f),
        }
    }
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

pub(crate) struct History<F, S> {
    inner: VecDeque<Node<F, S>>,
    current: usize,
    capacity: usize, // 0: no limit
    logical_first: usize,
}

impl<F, S> History<F, S> {
    pub(crate) fn new_unlimited(init: Node<F, S>) -> History<F, S> {
        let mut v = VecDeque::new();
        v.push_back(init);
        History {
            inner: v,
            current: 0,
            capacity: 0,
            logical_first: 0,
        }
    }
    pub(crate) fn new(init: Node<F, S>, capacity: usize) -> History<F, S> {
        let mut history = History::new_unlimited(init);
        history.capacity = capacity;
        history
    }

    // Trim front nodes if logical_first points a snapshot node because
    // the nodes are not needed for undoing.
    fn trim_before_snapshot(&mut self) {
        if self
            .get_node_inner(self.logical_first)
            .generator()
            .is_snapshot()
        {
            self.inner.drain(..self.logical_first);
            self.current -= self.logical_first;
            self.logical_first = 0;
        }
    }

    pub(crate) fn push_node(&mut self, node: Node<F, S>) {
        self.inner.truncate(self.current + 1);

        self.current += 1;
        let new_len = self.len() + 1;

        if self.capacity() != 0 && self.capacity() < new_len {
            debug_assert_eq!(self.capacity() + 1, new_len);
            self.logical_first += 1;
            self.trim_before_snapshot();
        }

        self.inner.push_back(node);
    }

    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }

    fn get_node_inner(&self, index: usize) -> &Node<F, S> {
        &self.inner[index]
    }
    pub(crate) fn current(&self) -> &Node<F, S> {
        self.get_node_inner(self.current)
    }
    pub(crate) fn current_index(&self) -> usize {
        debug_assert!(self.logical_first <= self.current);
        self.current - self.logical_first
    }

    pub(crate) fn set_current(&mut self, index: usize) -> bool {
        if self.len() <= index {
            false
        } else {
            self.current = self.inner_index(index);
            true
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.inner.len() - self.logical_first
    }

    pub(crate) fn len_before_current(&self) -> usize {
        self.current - self.logical_first
    }
    pub(crate) fn len_after_current(&self) -> usize {
        self.inner.len() - self.current - 1
    }

    fn find_last_snapshot_index_inner(&self, inner_index: usize) -> usize {
        let node = self.get_node_inner(inner_index);
        match node.generator() {
            Generator::Snapshot(_) => inner_index,
            Generator::Command(_) => {
                let m = node.metrics();
                let ret = inner_index - m.distance_from_snapshot();
                debug_assert!(self.get_node_inner(ret).generator().is_snapshot());
                ret
            }
        }
    }
    // Returns None if the index is out of bound.
    pub(crate) fn find_last_snapshot_index(&self, index: usize) -> Option<usize> {
        let inner = self.find_last_snapshot_index_inner(self.inner_index(index));
        self.outer_index(inner)
    }

    // Returns None if the index is out of bound.
    fn outer_index(&self, inner_index: usize) -> Option<usize> {
        if inner_index < self.logical_first {
            None
        } else {
            Some(inner_index - self.logical_first)
        }
    }
    fn inner_index(&self, index: usize) -> usize {
        index + self.logical_first
    }
    pub(crate) fn iter_from_last_snapshot(
        &self,
        back_index: usize,
    ) -> impl Iterator<Item = &Node<F, S>> {
        let inner_back = self.inner_index(back_index);
        let inner_snapshot = self.find_last_snapshot_index_inner(inner_back);
        self.inner.range(inner_snapshot..inner_back + 1)
    }

    pub(crate) fn iter_from(&self, first_index: usize) -> impl Iterator<Item = &Node<F, S>> {
        let inner_first = self.inner_index(first_index);
        self.inner.range(inner_first..)
    }
}

#[cfg(test)]
mod test {}
