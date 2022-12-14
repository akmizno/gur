//! Simplified [Ur\<T\>](crate::ur::Ur)
use crate::gur::{Gur, GurBuilder};
use crate::interface::*;
use crate::metrics::Metrics;
use crate::snapshot::CloneSnapshot;

/// A builder to create an [Cur].
///
/// # Interface
/// [CurBuilder] implements following interfaces.
///
/// - [IBuilder](crate::interface::IBuilder)
/// - [ITrigger](crate::interface::ITrigger)
///
/// See the pages to view method list.
#[derive(Default)]
pub struct CurBuilder<'a, T: Clone>(GurBuilder<'a, T, T, CloneSnapshot<T>>);

impl<'a, T: Clone> CurBuilder<'a, T> {
    /// Create a new builder instance.
    pub fn new() -> Self {
        Self(GurBuilder::new())
    }
}

impl<'a, T: Clone> IBuilder for CurBuilder<'a, T> {
    type State = T;
    type Target = Cur<'a, T>;

    fn capacity(mut self, capacity: usize) -> Self {
        self.0 = self.0.capacity(capacity);
        self
    }

    fn build(self, initial_state: T) -> Cur<'a, T> {
        Cur::new(self.0.build(initial_state))
    }
}

impl<'a, T: Clone> ITrigger<'a> for CurBuilder<'a, T> {
    fn snapshot_trigger<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + 'a,
    {
        self.0 = self.0.snapshot_trigger(f);
        self
    }
}

/// A wrapper type providing basic undo-redo functionality for [Clone] implementors.
///
/// See also [CurBuilder].
///
/// # Sample code
/// ```
/// use gur::prelude::*;
/// use gur::cur::{Cur, CurBuilder};
///
/// // Appication state
/// #[derive(Clone)]
/// struct MyState {
///     data: String
/// }
///
/// fn main() {
///     // Initialize
///     let mut state = CurBuilder::new().build(MyState{ data: "My".to_string() });
///     assert_eq!("My", state.data);
///
///     // Change state
///     state.edit(|mut state| { state.data += "State"; state });
///     assert_eq!("MyState", state.data);
///
///     // Undo
///     state.undo();
///     assert_eq!("My", state.data);
///
///     // Redo
///     state.redo();
///     assert_eq!("MyState", state.data);
/// }
/// ```
///
/// # Information
/// ## Snapshot by Clone
/// Unlike [Ur](crate::ur::Ur), [Cur] takes snapshots by [Clone].
/// So [Cur] requires a type `T` implementing [Clone] instead of
/// [Snapshot](crate::snapshot::Snapshot).
///
/// ## Provided methods
/// [Cur] implements following undo-redo interfaces.
///
/// - [IUndoRedo](crate::interface::IUndoRedo)
/// - [IEdit](crate::interface::IEdit)
///
/// See the pages to view method list.
///
/// ## Thread-safety
/// [Cur] does not implement [Send] and [Sync].
/// If you want a type [Cur] + [Send] + [Sync],
/// use [Acur](crate::acur::Acur).
#[derive(Debug)]
pub struct Cur<'a, T: Clone>(Gur<'a, T, T, CloneSnapshot<T>>);

impl<'a, T: Clone> Cur<'a, T> {
    fn new(inner: Gur<'a, T, T, CloneSnapshot<T>>) -> Self {
        Self(inner)
    }
}

impl<'a, T: Clone> IUndoRedo for Cur<'a, T> {
    type State = T;

    fn into_inner(self) -> T {
        self.0.into_inner()
    }

    fn capacity(&self) -> Option<usize> {
        self.0.capacity()
    }

    fn undoable_count(&self) -> usize {
        self.0.undoable_count()
    }

    fn redoable_count(&self) -> usize {
        self.0.redoable_count()
    }

    fn undo_multi(&mut self, count: usize) -> Option<&T> {
        self.0.undo_multi(count)
    }

    fn redo_multi(&mut self, count: usize) -> Option<&T> {
        self.0.redo_multi(count)
    }

    fn try_edit<F>(&mut self, command: F) -> Result<&T, Box<dyn std::error::Error>>
    where
        F: FnOnce(T) -> Result<T, Box<dyn std::error::Error>>,
    {
        self.0.try_edit(command)
    }
}

impl<'a, T: Clone> IEdit<'a> for Cur<'a, T> {
    type State = T;

    fn edit_if<F>(&mut self, command: F) -> Option<&T>
    where
        F: Fn(T) -> Option<T> + 'a,
    {
        self.0.edit_if(command)
    }
}

impl<'a, T: std::fmt::Display + Clone> std::fmt::Display for Cur<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl<'a, T: Clone> std::ops::Deref for Cur<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ok_add() {
        let mut s = CurBuilder::default().build(0);

        let t1 = s.try_edit(|n| Ok(n + 1)).unwrap();
        assert_eq!(1, *t1);
    }
    #[test]
    fn err_add() {
        let err_add = |n| "NaN".parse::<i32>().map(|p| p + n).map_err(|e| e.into());
        let add_one = |n| n + 1;

        let mut s = CurBuilder::default().build(0);

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
        let mut s = CurBuilder::default().build(0);

        s.edit(|n| n + 1);
        assert_eq!(1, *s);
        s.edit(|n| n * 3);
        assert_eq!(3, *s);
        s.edit(|n| n + 5);
        assert_eq!(8, *s);
        s.edit(|n| n * 7);
        assert_eq!(56, *s);
    }

    #[test]
    fn undo() {
        let mut s = CurBuilder::default().build(0);

        let t0 = *s;
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

        let mut s = CurBuilder::default()
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
        let mut s = CurBuilder::default().build(0);

        let t0 = *s;
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
        let mut s = CurBuilder::default().build(0);

        let t0 = *s;
        assert_eq!(0, t0);

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
        let mut s = CurBuilder::default().build(0);

        let t0 = *s;
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
    fn jump() {
        let mut s = CurBuilder::default().build(0);

        let t0 = *s; // 0
        let t1 = *s.edit(|n| n + 1); // 1
        let t2 = *s.edit(|n| n * 3); // 3
        let t3 = *s.edit(|n| n + 5); // 8
        let t4 = *s.edit(|n| n * 7); // 56
        let t5 = *s.edit(|n| n + 9); // 65

        // undo by jump()
        let j4 = s.jump(-1).unwrap();
        assert_eq!(t4, *j4);
        assert_eq!(t4, *s);
        let j2 = s.jump(-2).unwrap();
        assert_eq!(t2, *j2);
        assert_eq!(t2, *s);
        assert!(s.jump(-3).is_none());
        let j0 = s.jump(-2).unwrap();
        assert_eq!(t0, *j0);
        assert_eq!(t0, *s);

        // redo by jump()
        let j1 = s.jump(1).unwrap();
        assert_eq!(t1, *j1);
        assert_eq!(t1, *s);
        let j3 = s.jump(2).unwrap();
        assert_eq!(t3, *j3);
        assert_eq!(t3, *s);
        assert!(s.jump(3).is_none());
        let j5 = s.jump(2).unwrap();
        assert_eq!(t5, *j5);
        assert_eq!(t5, *s);
    }

    #[test]
    fn undo_multi() {
        let mut s = CurBuilder::default().build(0);

        let t0 = *s; // 0
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
        let mut s = CurBuilder::default().build(0);

        let t0 = *s; // 0
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
        let mut s = CurBuilder::default().build(0);

        let t0 = *s;
        assert_eq!(0, t0);

        let t_some = s.edit_if(|n| Some(n + 1));
        assert_eq!(1, *t_some.unwrap());
        assert_eq!(1, *s);
        let t_none = s.edit_if(|_| None);
        assert!(t_none.is_none());
        assert_eq!(1, *s);
    }

    #[test]
    fn into_inner() {
        let mut s = CurBuilder::default().build(0);

        let t0 = *s;
        assert_eq!(0, t0);

        s.edit_if(|n| Some(n + 1)).unwrap();
        assert_eq!(1, *s);

        assert_eq!(1, s.into_inner());
    }

    #[test]
    fn undoable_count() {
        let mut s = CurBuilder::default().build(0);

        let _t0 = *s; // 0
        assert_eq!(s.undoable_count(), 0);
        let _t1 = *s.edit(|n| n + 1); // 1
        assert_eq!(s.undoable_count(), 1);
        let _t2 = *s.edit(|n| n * 3); // 3
        assert_eq!(s.undoable_count(), 2);
        let _t3 = *s.edit(|n| n + 5); // 8
        assert_eq!(s.undoable_count(), 3);
        let _t4 = *s.edit(|n| n * 7); // 56
        assert_eq!(s.undoable_count(), 4);
        let _t5 = *s.edit(|n| n + 9); // 65
        assert_eq!(s.undoable_count(), 5);

        let _u4 = s.undo().unwrap();
        assert_eq!(s.undoable_count(), 4);
        let _u1 = s.undo_multi(3).unwrap();
        assert_eq!(s.undoable_count(), 1);
        let _u0 = s.undo().unwrap();
        assert_eq!(s.undoable_count(), 0);
        let _u0 = s.undo();
        assert_eq!(s.undoable_count(), 0);

        let _r1 = s.redo().unwrap();
        assert_eq!(s.undoable_count(), 1);
        let _r3 = s.redo_multi(2).unwrap();
        assert_eq!(s.undoable_count(), 3);
        let t6 = *s.edit(|n| n + 11); // 19
        assert_eq!(t6, 19);
        assert_eq!(s.undoable_count(), 4);
    }

    #[test]
    fn redoable_count() {
        let mut s = CurBuilder::default().build(0);

        let _t0 = *s; // 0
        assert_eq!(s.redoable_count(), 0);
        let _t1 = *s.edit(|n| n + 1); // 1
        assert_eq!(s.redoable_count(), 0);
        let _t2 = *s.edit(|n| n * 3); // 3
        assert_eq!(s.redoable_count(), 0);
        let _t3 = *s.edit(|n| n + 5); // 8
        assert_eq!(s.redoable_count(), 0);
        let _t4 = *s.edit(|n| n * 7); // 56
        assert_eq!(s.redoable_count(), 0);
        let _t5 = *s.edit(|n| n + 9); // 65
        assert_eq!(s.redoable_count(), 0);

        let _u4 = s.undo().unwrap();
        assert_eq!(s.redoable_count(), 1);
        let _u1 = s.undo_multi(3).unwrap();
        assert_eq!(s.redoable_count(), 4);
        let _u0 = s.undo().unwrap();
        assert_eq!(s.redoable_count(), 5);
        let _u0 = s.undo();
        assert_eq!(s.redoable_count(), 5);

        let _r1 = s.redo().unwrap();
        assert_eq!(s.redoable_count(), 4);
        let _r3 = s.redo_multi(2).unwrap();
        assert_eq!(s.redoable_count(), 2);
        let t6 = *s.edit(|n| n + 11); // 19
        assert_eq!(t6, 19);
        assert_eq!(s.redoable_count(), 0);
    }

    #[test]
    fn history_limit() {
        let mut s = CurBuilder::default().capacity(3).build(0);

        let _t0 = *s; // 0
        assert_eq!(s.undoable_count(), 0);
        assert_eq!(s.redoable_count(), 0);
        let _t1 = *s.edit(|n| n + 1); // 1
        assert_eq!(s.undoable_count(), 1);
        assert_eq!(s.redoable_count(), 0);
        let _t2 = *s.edit(|n| n * 3); // 3
        assert_eq!(s.undoable_count(), 2);
        assert_eq!(s.redoable_count(), 0);
        let _t3 = *s.edit(|n| n + 5); // 8
        assert_eq!(s.undoable_count(), 2);
        assert_eq!(s.redoable_count(), 0);
        let _t4 = *s.edit(|n| n * 7); // 56
        assert_eq!(s.undoable_count(), 2);
        assert_eq!(s.redoable_count(), 0);
        let _t5 = *s.edit(|n| n + 9); // 65
        assert_eq!(s.undoable_count(), 2);
        assert_eq!(s.redoable_count(), 0);

        let _u4 = s.undo().unwrap();
        assert_eq!(s.undoable_count(), 1);
        assert_eq!(s.redoable_count(), 1);
        let _u3 = s.undo().unwrap();
        assert_eq!(s.undoable_count(), 0);
        assert_eq!(s.redoable_count(), 2);

        assert!(s.undo().is_none());

        let _r4 = s.redo().unwrap();
        assert_eq!(s.undoable_count(), 1);
        assert_eq!(s.redoable_count(), 1);

        let t6 = *s.edit(|n| n + 11); // 67
        assert_eq!(t6, 67);
        assert_eq!(s.undoable_count(), 2);
        assert_eq!(s.redoable_count(), 0);

        let t7 = *s.edit(|n| n + 13); // 80
        assert_eq!(t7, 80);
        assert_eq!(s.undoable_count(), 2);
        assert_eq!(s.redoable_count(), 0);
    }

    #[test]
    fn capacity() {
        assert!(CurBuilder::default().build(0).capacity().is_none());
        assert!(CurBuilder::default()
            .capacity(0)
            .build(0)
            .capacity()
            .is_none());

        assert_eq!(
            CurBuilder::default()
                .capacity(3)
                .build(0)
                .capacity()
                .unwrap(),
            3
        );
    }
}
