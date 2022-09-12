use crate::action::{Action, TryAction};
use crate::memento::Memento;
use crate::metrics::Metrics;
use crate::node::{Creator, Node};
use std::iter::Iterator;
use std::time::{Duration, Instant};

pub struct GurBuilder<'a> {
    snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + 'a>,
}

impl<'a> GurBuilder<'a> {
    pub fn new() -> Self {
        Self {
            snapshot_trigger: Box::new(|_m| false),
        }
    }

    pub fn snapshot_trigger<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + 'a,
    {
        self.snapshot_trigger = Box::new(f);
        self
    }

    pub fn build<T: Memento + 'a>(self, initial_state: T) -> Gur<'a, T> {
        Gur::new(initial_state, self.snapshot_trigger)
    }
}

impl<'a> Default for GurBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Gur<'a, T: Memento> {
    state: Option<T>,

    actions: Vec<Node<'a, T>>,
    current: usize,

    snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + 'a>,
}

impl<'a, T: Default + Memento + 'a> Default for Gur<'a, T> {
    fn default() -> Self {
        GurBuilder::new().build(T::default())
    }
}
impl<'a, T: std::fmt::Display + Memento> std::fmt::Display for Gur<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.get().fmt(f)
    }
}

impl<'a, T: Memento> std::ops::Deref for Gur<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'a, T: Memento + 'a> Gur<'a, T> {
    fn new(initial_state: T, snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + 'a>) -> Self {
        let first_node = Node::from_memento(&initial_state);
        Self {
            state: Some(initial_state),
            actions: vec![first_node],
            current: 0,
            snapshot_trigger,
        }
    }
    pub fn get(&self) -> &T {
        debug_assert!(self.state.is_some());
        unsafe { self.state.as_ref().unwrap_unchecked() }
    }
    pub fn undo(&mut self) -> Option<&T> {
        debug_assert!(self.current < self.actions.len());
        if self.current == 0 {
            None
        } else {
            self.undo_impl();
            self.current -= 1;
            Some(self.get())
        }
    }
    pub fn redo(&mut self) -> Option<&T> {
        debug_assert!(self.current < self.actions.len());
        if self.current + 1 == self.actions.len() {
            None
        } else {
            self.redo_impl();
            self.current += 1;
            Some(self.get())
        }
    }

    fn find_last_snapshot(&self, end: usize) -> (T, usize) {
        if 0 < end {
            for (node, i) in self.actions[1..end].iter().rev().zip(1..) {
                if let Some(m) = node.creator().get_if_memento() {
                    let s = T::from_memento(m);
                    return (s, end - i);
                }
            }
        }
        let m = self.actions[0].creator().get_if_memento();
        debug_assert!(m.is_some());
        let s = T::from_memento(unsafe { m.unwrap_unchecked() });
        (s, 0)
    }

    fn undo_impl(&mut self) {
        let (first_state, first_idx) = self.find_last_snapshot(self.current);
        self.state = Some(first_state);

        for i in first_idx + 1..self.current {
            let prev = self.state.take().unwrap();

            debug_assert!(i < self.actions.len());
            let action = self.actions[i].creator().get_if_action().unwrap();

            let next = action.execute(prev);
            self.state = Some(next);
        }
    }

    fn redo_impl(&mut self) {
        let node = &self.actions[self.current + 1];
        let new_state = match node.creator() {
            Creator::Snapshot(m) => T::from_memento(m),
            Creator::Action(a) => a.execute(self.state.take().unwrap()),
        };
        self.state = Some(new_state);
    }

    fn redo_from_last_snapshot(&mut self) {
        let (first_state, first_idx) = self.find_last_snapshot(self.current + 1);
        self.state = Some(first_state);

        for i in first_idx + 1..self.current + 1 {
            let prev = self.state.take().unwrap();

            debug_assert!(i < self.actions.len());
            let action = self.actions[i].creator().get_if_action().unwrap();

            let next = action.execute(prev);
            self.state = Some(next);
        }
    }

    fn act_impl<A>(action: &A, old_state: T) -> (T, Duration)
    where
        A: Action<State = T> + 'a,
    {
        let now = Instant::now();
        let new_state = action.execute(old_state);
        let elapsed = now.elapsed();

        (new_state, elapsed)
    }
    pub fn act<A>(&mut self, action: A) -> &T
    where
        A: Action<State = T> + 'a,
    {
        debug_assert!(self.state.is_some());

        let old_state = unsafe { self.state.take().unwrap_unchecked() };

        let (new_state, elapsed) = Self::act_impl(&action, old_state);

        self.actions.truncate(self.current + 1);

        let last_metrics = self.actions.last().unwrap().metrics();
        let action_metrics = last_metrics.make_next(elapsed);

        if (self.snapshot_trigger)(&action_metrics) {
            self.actions.push(Node::from_memento(&new_state));
        } else {
            self.actions.push(Node::from_action(action, action_metrics));
        }

        self.current += 1;

        self.state.replace(new_state);
        self.get()
    }
}

impl<'a, T: Memento + 'a> Gur<'a, T> {
    pub fn try_act<A>(&mut self, action: A) -> Result<&T, Box<dyn std::error::Error>>
    where
        A: TryAction<State = T>,
    {
        debug_assert!(self.state.is_some());

        let old_state = unsafe { self.state.take().unwrap_unchecked() };
        match action.try_execute(old_state) {
            Ok(new_state) => {
                self.actions.truncate(self.current + 1);
                self.actions.push(Node::from_memento(&new_state));
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
    use crate::action::{Action, TryAction};
    use crate::memento::Memento;

    #[derive(Debug)]
    struct Add(i32);

    impl Action for Add {
        type State = i32;
        fn execute(&self, prev: Self::State) -> Self::State {
            prev + self.0
        }
    }

    impl Memento for i32 {
        type Target = Self;
        fn to_memento(&self) -> Self::Target {
            *self
        }
        fn from_memento(memento: &Self::Target) -> Self {
            *memento
        }
    }

    #[derive(Debug)]
    struct Mul(i32);

    impl Action for Mul {
        type State = i32;
        fn execute(&self, prev: Self::State) -> Self::State {
            prev * self.0
        }
    }

    #[derive(Debug)]
    struct OkAdd(i32);

    impl TryAction for OkAdd {
        type State = i32;
        fn try_execute(
            &self,
            prev: Self::State,
        ) -> Result<Self::State, Box<dyn std::error::Error>> {
            Ok(prev + self.0)
        }
    }

    #[derive(Debug)]
    struct ErrAdd(i32);

    impl TryAction for ErrAdd {
        type State = i32;
        fn try_execute(&self, _: Self::State) -> Result<Self::State, Box<dyn std::error::Error>> {
            "NaN".parse::<i32>().map_err(|e| e.into())
        }
    }

    #[test]
    fn ok_add() {
        let mut s = GurBuilder::new().build(0);

        let t1 = s.try_act(OkAdd(1)).unwrap();
        assert_eq!(1, *t1);
    }
    #[test]
    fn err_add() {
        let mut s = GurBuilder::new().build(0);

        assert_eq!(0, *s);

        let t1 = s.try_act(ErrAdd(1));
        assert!(t1.is_err());
        assert_eq!(0, *s);

        let t1 = s.act(Add(1));
        assert_eq!(1, *t1);
        let t2 = s.act(Add(1));
        assert_eq!(2, *t2);
        let t3 = s.try_act(ErrAdd(1));
        assert!(t3.is_err());
        assert_eq!(2, *s);
    }
    #[test]
    fn deref() {
        let mut s = GurBuilder::new().build(0);

        s.act(Add(1));
        assert_eq!(1, *s);
        assert_eq!(s.get(), &*s);
        s.act(Mul(3));
        assert_eq!(3, *s);
        assert_eq!(s.get(), &*s);
        s.act(Add(5));
        assert_eq!(8, *s);
        assert_eq!(s.get(), &*s);
        s.act(Mul(7));
        assert_eq!(56, *s);
        assert_eq!(s.get(), &*s);
    }

    #[test]
    fn undo() {
        let mut s = GurBuilder::new().build(0);

        let t0 = *s.get();
        assert_eq!(0, t0);
        assert!(s.undo().is_none());

        let t1 = *s.act(Add(1));
        assert_eq!(1, *s);
        let t2 = *s.act(Mul(3));
        assert_eq!(3, *s);
        let t3 = *s.act(Add(5));
        assert_eq!(8, *s);
        let t4 = *s.act(Mul(7));
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
            // To speed up undo()/redo(), a memento is sometimes created.
            .snapshot_trigger(|metrics| 10 < metrics.distance_from_snapshot())
            .build(0);

        for i in 0..n {
            s.act(Add(1));
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

        let t1 = *s.act(Add(1));
        assert_eq!(1, *s);
        let t2 = *s.act(Mul(3));
        assert_eq!(3, *s);
        let t3 = *s.act(Add(5));
        assert_eq!(8, *s);
        let t4 = *s.act(Mul(7));
        assert_eq!(56, *s);

        let _ = s.undo().unwrap();
        let _ = s.undo().unwrap();
        let _ = s.undo().unwrap();
        let _ = s.undo().unwrap();
        assert!(s.undo().is_none());

        let r1 = *s.act(Add(1));
        assert_eq!(1, *s);
        let r2 = *s.act(Mul(3));
        assert_eq!(3, *s);
        let r3 = *s.act(Add(5));
        assert_eq!(8, *s);
        let r4 = *s.act(Mul(7));
        assert_eq!(56, *s);
        assert!(s.redo().is_none());

        assert_eq!(t1, r1);
        assert_eq!(t2, r2);
        assert_eq!(t3, r3);
        assert_eq!(t4, r4);
    }

    #[test]
    fn act_undo_act() {
        let mut s = GurBuilder::new().build(0);

        let t0 = s.get();
        assert_eq!(0, *t0);

        let t1 = s.act(Add(1));
        assert_eq!(1, *t1);
        let t2 = s.act(Mul(3));
        assert_eq!(3, *t2);

        let u1 = s.undo().unwrap();
        assert_eq!(1, *u1);
        let t2d = s.act(Add(4));
        assert_eq!(5, *t2d);
    }

    #[test]
    fn act_undo_act_act_undo_redo() {
        let mut s = GurBuilder::new().build(0);

        let t0 = *s.get();
        assert_eq!(0, t0);

        let t1 = s.act(Add(1));
        assert_eq!(1, *t1);
        let t2 = s.act(Mul(3));
        assert_eq!(3, *t2);

        let u1 = s.undo().unwrap();
        assert_eq!(1, *u1);
        let t2d = s.act(Add(4));
        assert_eq!(5, *t2d);
        let t3d = s.act(Mul(5));
        assert_eq!(25, *t3d);

        let u2d = s.undo().unwrap();
        assert_eq!(5, *u2d);

        let r3d = s.redo().unwrap();
        assert_eq!(25, *r3d);
    }
}
