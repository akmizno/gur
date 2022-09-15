use crate::metrics::Metrics;
use crate::node::Node;
use crate::snapshot::Snapshot;
use std::iter::Iterator;
use std::time::{Duration, Instant};

pub struct GurBuilder<'a> {
    snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + Send + Sync + 'a>,
}

impl<'a> GurBuilder<'a> {
    pub fn new() -> Self {
        Self {
            snapshot_trigger: Box::new(|_m| false),
        }
    }

    pub fn snapshot_trigger<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + Send + Sync + 'a,
    {
        self.snapshot_trigger = Box::new(f);
        self
    }

    pub fn build<T: Snapshot + 'a>(self, initial_state: T) -> Gur<'a, T> {
        Gur::new(initial_state, self.snapshot_trigger)
    }
}

impl<'a> Default for GurBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Gur<'a, T: Snapshot> {
    state: Option<T>,

    history: Vec<Node<'a, T>>,
    current: usize,

    snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + Send + Sync + 'a>,
}

impl<'a, T: Default + Snapshot + 'a> Default for Gur<'a, T> {
    fn default() -> Self {
        GurBuilder::new().build(T::default())
    }
}
impl<'a, T: std::fmt::Display + Snapshot> std::fmt::Display for Gur<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.get().fmt(f)
    }
}

impl<'a, T: Snapshot> std::ops::Deref for Gur<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'a, T: Snapshot + 'a> Gur<'a, T> {
    fn new(
        initial_state: T,
        snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + Send + Sync + 'a>,
    ) -> Self {
        let first_node = Node::from_state(&initial_state);
        Self {
            state: Some(initial_state),
            history: vec![first_node],
            current: 0,
            snapshot_trigger,
        }
    }
    pub fn get(&self) -> &T {
        debug_assert!(self.state.is_some());
        unsafe { self.state.as_ref().unwrap_unchecked() }
    }
    pub fn undo(&mut self) -> Option<&T> {
        debug_assert!(self.current < self.history.len());
        if self.current == 0 {
            None
        } else {
            self.undo_impl();
            self.current -= 1;
            Some(self.get())
        }
    }
    pub fn redo(&mut self) -> Option<&T> {
        debug_assert!(self.current < self.history.len());
        if self.current + 1 == self.history.len() {
            None
        } else {
            self.redo_impl();
            self.current += 1;
            Some(self.get())
        }
    }

    fn find_last_snapshot(&self, end: usize) -> (T, usize) {
        if 0 < end {
            for (node, i) in self.history[1..end].iter().rev().zip(1..) {
                if let Some(s) = node.generator().generate_if_snapshot() {
                    return (s, end - i);
                }
            }
        }
        let s = self.history[0].generator().generate_if_snapshot().unwrap();
        (s, 0)
    }

    fn undo_impl(&mut self) {
        let (first_state, first_idx) = self.find_last_snapshot(self.current);
        self.state = Some(first_state);

        for i in first_idx + 1..self.current {
            let prev = self.state.take().unwrap();

            debug_assert!(i < self.history.len());
            let next = self.history[i]
                .generator()
                .generate_if_editor(prev)
                .unwrap();
            self.state = Some(next);
        }
    }

    fn redo_impl(&mut self) {
        let node = &self.history[self.current + 1];
        let new_state = node.generator().generate(self.state.take().unwrap());
        self.state = Some(new_state);
    }

    fn redo_from_last_snapshot(&mut self) {
        let (first_state, first_idx) = self.find_last_snapshot(self.current + 1);
        self.state = Some(first_state);

        for i in first_idx + 1..self.current + 1 {
            let prev = self.state.take().unwrap();

            debug_assert!(i < self.history.len());
            let next = self.history[i]
                .generator()
                .generate_if_editor(prev)
                .unwrap();
            self.state = Some(next);
        }
    }

    fn edit_impl<F>(editor: &F, old_state: T) -> (T, Duration)
    where
        F: Fn(T) -> T + 'a,
    {
        let now = Instant::now();
        let new_state = editor(old_state);
        let elapsed = now.elapsed();

        (new_state, elapsed)
    }
    pub fn edit<F>(&mut self, editor: F) -> &T
    where
        F: Fn(T) -> T + Send + Sync + 'a,
    {
        debug_assert!(self.state.is_some());

        let old_state = unsafe { self.state.take().unwrap_unchecked() };

        let (new_state, elapsed) = Self::edit_impl(&editor, old_state);

        self.history.truncate(self.current + 1);

        let last_metrics = self.history.last().unwrap().metrics();
        let new_metrics = last_metrics.make_next(elapsed);

        if (self.snapshot_trigger)(&new_metrics) {
            self.history.push(Node::from_state(&new_state));
        } else {
            self.history.push(Node::from_editor(editor, new_metrics));
        }

        self.current += 1;

        self.state.replace(new_state);
        self.get()
    }

    pub fn try_edit<F>(&mut self, editor: F) -> Result<&T, Box<dyn std::error::Error>>
    where
        F: FnOnce(T) -> Result<T, Box<dyn std::error::Error>>,
    {
        debug_assert!(self.state.is_some());

        let old_state = unsafe { self.state.take().unwrap_unchecked() };
        match editor(old_state) {
            Ok(new_state) => {
                self.history.truncate(self.current + 1);
                self.history.push(Node::from_state(&new_state));
                self.current += 1;

                self.state.replace(new_state);
                Ok(self.get())
            }
            Err(e) => {
                self.redo_from_last_snapshot();
                Err(e)
            }
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::snapshot::Snapshot;

    impl Snapshot for i32 {
        type Target = Self;
        fn to_snapshot(&self) -> Self::Target {
            *self
        }
        fn from_snapshot(snapshot: &Self::Target) -> Self {
            *snapshot
        }
    }

    #[test]
    fn ok_add() {
        let mut s = GurBuilder::new().build(0);

        let t1 = s.try_edit(|n| Ok(n + 1)).unwrap();
        assert_eq!(1, *t1);
    }
    #[test]
    fn err_add() {
        let err_add = |n| "NaN".parse::<i32>().map(|p| p + n).map_err(|e| e.into());
        let add_one = |n| n + 1;

        let mut s = GurBuilder::new().build(0);

        assert_eq!(0, *s);

        let t1 = s.try_edit(err_add);
        assert!(t1.is_err());
        assert_eq!(0, *s);

        let t1 = s.edit(add_one);
        assert_eq!(1, *t1);
        let t2 = s.edit(add_one);
        assert_eq!(2, *t2);
        let t3 = s.try_edit(err_add);
        assert!(t3.is_err());
        assert_eq!(2, *s);
    }
    #[test]
    fn deref() {
        let mut s = GurBuilder::new().build(0);

        s.edit(|n| n + 1);
        assert_eq!(1, *s);
        assert_eq!(s.get(), &*s);
        s.edit(|n| n * 3);
        assert_eq!(3, *s);
        assert_eq!(s.get(), &*s);
        s.edit(|n| n + 5);
        assert_eq!(8, *s);
        assert_eq!(s.get(), &*s);
        s.edit(|n| n * 7);
        assert_eq!(56, *s);
        assert_eq!(s.get(), &*s);
    }

    #[test]
    fn undo() {
        let mut s = GurBuilder::new().build(0);

        let t0 = *s.get();
        assert_eq!(0, t0);
        assert!(s.undo().is_none());

        let t1 = *s.edit(|n| n + 1);
        assert_eq!(1, *s);
        let t2 = *s.edit(|n| n * 3);
        assert_eq!(3, *s);
        let t3 = *s.edit(|n| n + 5);
        assert_eq!(8, *s);
        let t4 = *s.edit(|n| n * 7);
        assert_eq!(56, *s);

        let u3 = *s.undo().unwrap();
        assert_eq!(8, *s);
        let u2 = *s.undo().unwrap();
        assert_eq!(3, *s);
        let u1 = *s.undo().unwrap();
        assert_eq!(1, *s);
        let u0 = *s.undo().unwrap();
        assert_eq!(0, *s);
        assert!(s.undo().is_none());

        assert_eq!(t0, u0);
        assert_eq!(t1, u1);
        assert_eq!(t2, u2);
        assert_eq!(t3, u3);
        assert_eq!(t4, 56);
    }

    #[test]
    fn undo_redo_many() {
        let n = 100000;

        let mut s = GurBuilder::new()
            // To speed up undo()/redo(), a snapshot is sometimes created.
            .snapshot_trigger(|metrics| 10 < metrics.distance_from_snapshot())
            .build(0);

        for i in 0..n {
            s.edit(|n| n + 1);
            assert_eq!(i + 1, *s);
        }

        for i in (0..n).rev() {
            assert_eq!(i, *s.undo().unwrap());
        }
        assert!(s.undo().is_none());

        for i in 0..n {
            assert_eq!(i + 1, *s.redo().unwrap());
        }
        assert!(s.redo().is_none());
    }

    #[test]
    fn redo() {
        let mut s = GurBuilder::new().build(0);

        let t0 = *s.get();
        assert_eq!(0, t0);
        assert!(s.undo().is_none());
        assert!(s.redo().is_none());

        let t1 = *s.edit(|n| n + 1);
        assert_eq!(1, *s);
        let t2 = *s.edit(|n| n * 3);
        assert_eq!(3, *s);
        let t3 = *s.edit(|n| n + 5);
        assert_eq!(8, *s);
        let t4 = *s.edit(|n| n * 7);
        assert_eq!(56, *s);

        let _ = s.undo().unwrap();
        let _ = s.undo().unwrap();
        let _ = s.undo().unwrap();
        let _ = s.undo().unwrap();
        assert!(s.undo().is_none());

        let r1 = *s.redo().unwrap();
        assert_eq!(1, *s);
        let r2 = *s.redo().unwrap();
        assert_eq!(3, *s);
        let r3 = *s.redo().unwrap();
        assert_eq!(8, *s);
        let r4 = *s.redo().unwrap();
        assert_eq!(56, *s);
        assert!(s.redo().is_none());

        assert_eq!(t1, r1);
        assert_eq!(t2, r2);
        assert_eq!(t3, r3);
        assert_eq!(t4, r4);
    }

    #[test]
    fn edit_undo_edit() {
        let mut s = GurBuilder::new().build(0);

        let t0 = s.get();
        assert_eq!(0, *t0);

        let t1 = s.edit(|n| n + 1);
        assert_eq!(1, *t1);
        let t2 = s.edit(|n| n * 3);
        assert_eq!(3, *t2);

        let u1 = s.undo().unwrap();
        assert_eq!(1, *u1);
        let t2d = s.edit(|n| n + 4);
        assert_eq!(5, *t2d);
    }

    #[test]
    fn edit_undo_edit_edit_undo_redo() {
        let mut s = GurBuilder::new().build(0);

        let t0 = *s.get();
        assert_eq!(0, t0);

        let t1 = s.edit(|n| n + 1);
        assert_eq!(1, *t1);
        let t2 = s.edit(|n| n * 3);
        assert_eq!(3, *t2);

        let u1 = s.undo().unwrap();
        assert_eq!(1, *u1);
        let t2d = s.edit(|n| n + 4);
        assert_eq!(5, *t2d);
        let t3d = s.edit(|n| n * 5);
        assert_eq!(25, *t3d);

        let u2d = s.undo().unwrap();
        assert_eq!(5, *u2d);

        let r3d = s.redo().unwrap();
        assert_eq!(25, *r3d);
    }
}
