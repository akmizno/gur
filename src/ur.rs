use crate::metrics::Metrics;
use crate::node::Node;
use std::time::{Duration, Instant};

/// A builder to create an [Ur].
///
pub struct UrBuilder<'a> {
    snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + 'a>,
}

impl<'a> UrBuilder<'a> {
    /// Create a new builder instance.
    pub fn new() -> Self {
        Self {
            snapshot_trigger: Box::new(|_m| false),
        }
    }

    /// Takes a closure to decide whether to save a snapshot of internal state.
    ///
    /// See [Snapshot trigger](crate::triggers#Snapshot&#32;trigger) for more details.
    pub fn snapshot_trigger<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + 'a,
    {
        self.snapshot_trigger = Box::new(f);
        self
    }

    /// Create a new [Ur] object by the initial state of T.
    pub fn build<T: Clone>(self, initial_state: T) -> Ur<'a, T> {
        Ur::new(initial_state, self.snapshot_trigger)
    }
}

impl<'a> Default for UrBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// A wrapper type to provide basic undo-redo functionality.
///
/// # Generative approach
/// A key idea of [Ur] is that "Undo is regenerating old state."
///
/// For explanation, there is a sample history of changes (t0, t1, t2, and t3) as
/// shown in the following figure.
/// When undoing the latest state (t3) to the previous state (t2),
/// [Ur] will back to the snapshot (s0) and restore the initial state (t0),
/// then redo the actions (a1 and a2) in order until the target state (t2) is obtained.
/// ```txt
/// t: state
/// a: action
/// s: snapshot
///
/// +---+ a1 +---+ a2 +---+ a3 +---+
/// |t0 |--->|t1 |--->|t2 |--->|t3 |
/// +---+    +---+    +---+    +---+
///   |    +--------->           |
/// +---+  |                     |
/// |s0 |  +---------------------+
/// ----+       undo t3 -> t2
/// ```
///
/// [Ur] manages object state and its history of changes.
/// To implement the above undoing procedure,
/// the history is stored as chain of actions instead of chain of states.
///
/// # Performance customization
/// There is a performance problem with this generative approach.
/// For example, undoing may take too long time if the actions are heavy computational tasks.
///
/// The problem can be mitigated by taking snapshots at appropriate intervals.
/// [Ur] provides a way to control the behavior by trigger functions.
/// See [Triggers](crate::triggers) about it.
///
/// # Deref
/// [Ur] implements [Deref](std::ops::Deref).
///
/// # Thread-safety
/// [Ur] does not implement [Send] and [Sync].
/// If you want a type [Ur] + [Send] + [Sync],
/// [Aur](crate::aur::Aur) can be used.
pub struct Ur<'a, T> {
    state: Option<T>,

    history: Vec<Node<'a, T>>,
    current: usize,

    snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + 'a>,
}

impl<'a, T> Ur<'a, T> {
    fn get(&self) -> &T {
        debug_assert!(self.state.is_some());
        unsafe { self.state.as_ref().unwrap_unchecked() }
    }
}
impl<'a, T: Clone> Ur<'a, T> {
    pub(crate) fn new(
        initial_state: T,
        snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + 'a>,
    ) -> Self {
        let first_node = Node::from_state(&initial_state);
        Self {
            state: Some(initial_state),
            history: vec![first_node],
            current: 0,
            snapshot_trigger,
        }
    }

    /// Restore the previous state.
    ///
    /// Same as `self.undo_multi(1)`.
    ///
    /// # Return
    /// [None] is returned if there is no older state in the history,
    /// otherwise immutable reference to the updated internal state.
    pub fn undo(&mut self) -> Option<&T> {
        self.undo_multi(1)
    }

    /// Undo multiple steps.
    ///
    /// This method is more efficient than running `self.undo()` multiple times.
    ///
    /// # Return
    /// [None] is returned if the target state is out of the history,
    /// otherwise immutable reference to the updated internal state.
    /// If `count=0`, this method does nothing and returns reference to the current state.
    pub fn undo_multi(&mut self, count: usize) -> Option<&T> {
        debug_assert!(count < isize::MAX as usize);
        self.jumpdo(-(count as isize))
    }

    /// Restore the next state.
    ///
    /// Same as `self.redo_multi(1)`.
    ///
    /// # Return
    /// [None] is returned if there is no newer state in the history,
    /// otherwise immutable reference to the updated internal state.
    pub fn redo(&mut self) -> Option<&T> {
        self.redo_multi(1)
    }

    /// Redo multiple steps.
    ///
    /// This method is more efficient than running `self.redo()` multiple times.
    ///
    /// # Return
    /// [None] is returned if the target state is out of the history,
    /// otherwise immutable reference to the updated internal state.
    /// If `count=0`, this method does nothing and returns reference to the current state.
    pub fn redo_multi(&mut self, count: usize) -> Option<&T> {
        debug_assert!(count < isize::MAX as usize);
        self.jumpdo(count as isize)
    }

    /// Undo-redo bidirectionally.
    ///
    /// This is integrated method of [undo_multi](Ur::undo_multi) and [redo_multi](Ur::redo_multi).
    ///
    /// - `count < 0` => `self.undo_multi(-count)`.
    /// - `0 < count` => `self.redo_multi(count)`.
    pub fn jumpdo(&mut self, count: isize) -> Option<&T> {
        if 0 == count {
            // Nothing to do
            return self.state.as_ref();
        }

        // Check the argment
        if count < 0 {
            // Undo
            if self.current < count.abs() as usize {
                return None;
            }
        } else {
            // Redo
            if self.history.len() <= self.current + count.abs() as usize {
                return None;
            }
        }

        self.jumpdo_impl(count);
        self.state.as_ref()
    }

    fn get_regeneration_range(&self, target: usize) -> (usize, usize) {
        debug_assert!(target < self.history.len());
        let last_node = unsafe { self.history.get_unchecked(target) };
        let dist = last_node.metrics().distance_from_snapshot();
        debug_assert!(dist <= target);
        let first = target - dist;
        debug_assert!(self.history[first].generator().is_snapshot());
        (first, target + 1)
    }

    fn regenerate(first_state: T, history: &[Node<'a, T>]) -> T {
        let mut state = first_state;
        for node in history {
            let next = node.generator().generate_if_command(state);
            debug_assert!(next.is_some());
            state = unsafe { next.unwrap_unchecked() };
        }

        state
    }

    fn jumpdo_impl(&mut self, count: isize) {
        let target = if count < 0 {
            debug_assert!(count.abs() as usize <= self.current);
            self.current - count.abs() as usize
        } else {
            self.current + count as usize
        };

        debug_assert!(target < self.history.len());

        let (begin, end) = self.get_regeneration_range(target);

        let (first_state, begin) = unsafe {
            if begin < self.current && self.current < end {
                // Reuse current state
                debug_assert!(self.state.is_some());
                (self.state.take().unwrap_unchecked(), self.current)
            } else {
                // The current state is not resusable.

                // Drop the old state before runnning.
                self.state = None;

                (
                    self.history
                        .get_unchecked(begin)
                        .generator()
                        .generate_if_snapshot()
                        .unwrap_unchecked(),
                    begin,
                )
            }
        };

        self.state = Some(Self::regenerate(first_state, &self.history[begin + 1..end]));
        self.current = target;
    }

    // Regenerate a target state from history WITHOUT reusing the current state.
    fn reset_state(&mut self, target: usize) {
        // Drop the old state before running.
        self.state = None;

        let (begin, end) = self.get_regeneration_range(target);

        let first_state = unsafe {
            self.history
                .get_unchecked(begin)
                .generator()
                .generate_if_snapshot()
                .unwrap_unchecked()
        };

        self.state = Some(Self::regenerate(first_state, &self.history[begin + 1..end]));
        self.current = target;
    }

    /// Takes a closure and update the internal state.
    ///
    /// The closure consumes the current state and produces a new state.
    ///
    /// # Return
    /// Immutable reference to the new state.
    ///
    /// # Remarks
    /// The closure MUST produce a same result for a same input.
    /// If it is impossible, use [try_edit](Ur::try_edit).
    pub fn edit<F>(&mut self, command: F) -> &T
    where
        F: Fn(T) -> T + 'a,
    {
        // NOTE
        // This call is guaranteed to succeed.
        unsafe { self.edit_if(move |s| Some(command(s))).unwrap_unchecked() }
    }

    /// Takes a closure and update the internal state.
    ///
    /// The closure consumes the current state and produces a new state or [None].
    /// If the closure returns [None], the internal state is not changed.
    ///
    /// # Return
    /// Immutable reference to the new state or [None]
    ///
    /// # Remarks
    /// The closure MUST produce a same result for a same input.
    /// If it is impossible, use [try_edit](Ur::try_edit).
    pub fn edit_if<F>(&mut self, command: F) -> Option<&T>
    where
        F: Fn(T) -> Option<T> + 'a,
    {
        debug_assert!(self.state.is_some());

        let old_state = unsafe { self.state.take().unwrap_unchecked() };

        let (new_state, elapsed) =
            if let Some((new_state, elapsed)) = Self::edit_if_impl(&command, old_state) {
                (new_state, elapsed)
            } else {
                // Reset the current state
                self.reset_state(self.current);
                return None;
            };

        self.history.truncate(self.current + 1);

        let last_metrics = self.history.last().unwrap().metrics();
        let new_metrics = last_metrics.make_next(elapsed);

        if (self.snapshot_trigger)(&new_metrics) {
            self.history.push(Node::from_state(&new_state));
        } else {
            self.history.push(Node::from_command(
                // This must succeed.
                move |s| unsafe { command(s).unwrap_unchecked() },
                new_metrics,
            ));
        }

        self.current += 1;

        self.state.replace(new_state);
        Some(self.get())
    }

    fn edit_if_impl<F>(command: &F, old_state: T) -> Option<(T, Duration)>
    where
        F: Fn(T) -> Option<T> + 'a,
    {
        let now = Instant::now();
        let new_state = command(old_state);
        let elapsed = now.elapsed();

        if let Some(new_state) = new_state {
            Some((new_state, elapsed))
        } else {
            None
        }
    }

    /// Takes a closure and update the internal state.
    ///
    /// The closure consumes the current state and produces a new state or an error.
    /// If the closure returns an error, the internal state is not changed.
    ///
    /// # Return
    /// Immutable reference to the new state or an error produced by the closure.
    ///
    /// # Remark
    /// In this method, the produced state from the closure is stored as a snapshot always;
    /// because of the type of closure is [FnOnce], same output can not be reproducible never again.
    pub fn try_edit<F>(&mut self, command: F) -> Result<&T, Box<dyn std::error::Error>>
    where
        F: FnOnce(T) -> Result<T, Box<dyn std::error::Error>>,
    {
        debug_assert!(self.state.is_some());

        let old_state = unsafe { self.state.take().unwrap_unchecked() };
        match command(old_state) {
            Ok(new_state) => {
                self.history.truncate(self.current + 1);
                self.history.push(Node::from_state(&new_state));
                self.current += 1;

                self.state.replace(new_state);
                Ok(self.get())
            }
            Err(e) => {
                self.reset_state(self.current);
                Err(e)
            }
        }
    }
}

impl<'a, T: std::fmt::Debug> std::fmt::Debug for Ur<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.get().fmt(f)
    }
}

impl<'a, T: std::fmt::Display> std::fmt::Display for Ur<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.get().fmt(f)
    }
}

impl<'a, T> std::ops::Deref for Ur<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ok_add() {
        let mut s = UrBuilder::new().build(0);

        let t1 = s.try_edit(|n| Ok(n + 1)).unwrap();
        assert_eq!(1, *t1);
    }
    #[test]
    fn err_add() {
        let err_add = |n| "NaN".parse::<i32>().map(|p| p + n).map_err(|e| e.into());
        let add_one = |n| n + 1;

        let mut s = UrBuilder::new().build(0);

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
        let mut s = UrBuilder::new().build(0);

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
        let mut s = UrBuilder::new().build(0);

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

        let mut s = UrBuilder::new()
            // This trigger sometimes inserts snapshots to speed up undo()/redo().
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
        let mut s = UrBuilder::new().build(0);

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
        let mut s = UrBuilder::new().build(0);

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
        let mut s = UrBuilder::new().build(0);

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

    #[test]
    fn jumpdo() {
        let mut s = UrBuilder::new().build(0);

        let t0 = *s.get(); // 0
        let t1 = *s.edit(|n| n + 1); // 1
        let t2 = *s.edit(|n| n * 3); // 3
        let t3 = *s.edit(|n| n + 5); // 8
        let t4 = *s.edit(|n| n * 7); // 56
        let t5 = *s.edit(|n| n + 9); // 65

        // undo by jumpdo()
        let j4 = s.jumpdo(-1).unwrap();
        assert_eq!(t4, *j4);
        assert_eq!(t4, *s);
        let j2 = s.jumpdo(-2).unwrap();
        assert_eq!(t2, *j2);
        assert_eq!(t2, *s);
        assert!(s.jumpdo(-3).is_none());
        let j0 = s.jumpdo(-2).unwrap();
        assert_eq!(t0, *j0);
        assert_eq!(t0, *s);

        // redo by jumpdo()
        let j1 = s.jumpdo(1).unwrap();
        assert_eq!(t1, *j1);
        assert_eq!(t1, *s);
        let j3 = s.jumpdo(2).unwrap();
        assert_eq!(t3, *j3);
        assert_eq!(t3, *s);
        assert!(s.jumpdo(3).is_none());
        let j5 = s.jumpdo(2).unwrap();
        assert_eq!(t5, *j5);
        assert_eq!(t5, *s);
    }

    #[test]
    fn undo_multi() {
        let mut s = UrBuilder::new().build(0);

        let t0 = *s.get(); // 0
        let _t1 = *s.edit(|n| n + 1); // 1
        let t2 = *s.edit(|n| n * 3); // 3
        let _t3 = *s.edit(|n| n + 5); // 8
        let t4 = *s.edit(|n| n * 7); // 56
        let _t5 = *s.edit(|n| n + 9); // 65

        let u4 = s.undo_multi(1).unwrap();
        assert_eq!(t4, *u4);
        assert_eq!(t4, *s);
        let u2 = s.undo_multi(2).unwrap();
        assert_eq!(t2, *u2);
        assert_eq!(t2, *s);
        assert!(s.undo_multi(3).is_none());
        let u0 = s.undo_multi(2).unwrap();
        assert_eq!(t0, *u0);
        assert_eq!(t0, *s);
    }

    #[test]
    fn redo_multi() {
        let mut s = UrBuilder::new().build(0);

        let t0 = *s.get(); // 0
        let t1 = *s.edit(|n| n + 1); // 1
        let _t2 = *s.edit(|n| n * 3); // 3
        let t3 = *s.edit(|n| n + 5); // 8
        let _t4 = *s.edit(|n| n * 7); // 56
        let t5 = *s.edit(|n| n + 9); // 65

        let u0 = s.undo_multi(5).unwrap();
        assert_eq!(t0, *u0);
        assert_eq!(t0, *s);

        let r1 = s.redo_multi(1).unwrap();
        assert_eq!(t1, *r1);
        assert_eq!(t1, *s);
        let r3 = s.redo_multi(2).unwrap();
        assert_eq!(t3, *r3);
        assert_eq!(t3, *s);
        assert!(s.redo_multi(3).is_none());
        let r5 = s.redo_multi(2).unwrap();
        assert_eq!(t5, *r5);
        assert_eq!(t5, *s);
    }

    #[test]
    fn edit_if() {
        let mut s = UrBuilder::new().build(0);

        let t0 = s.get();
        assert_eq!(0, *t0);

        let t_some = s.edit_if(|n| Some(n + 1));
        assert_eq!(1, *t_some.unwrap());
        assert_eq!(1, *s);
        let t_none = s.edit_if(|_| None);
        assert!(t_none.is_none());
        assert_eq!(1, *s);
    }
}
