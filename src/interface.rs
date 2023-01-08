//! Interfaces of [Ur](crate::ur::Ur) and related types.
//!
//! # Thread safety
//! In this crate, there are some similar types providing undo-redo functionality, e.g.
//! [Ur](crate::ur::Ur) and [Aur](crate::aur::Aur).
//! They have almost the same methods.
//! Unfortunately, however, the trait bounds in some methods are a bit different.
//!
//! The reason why is thread safety.
//! The [Ur](crate::ur::Ur) does not require closures implementing [Send] and [Sync] in some methods (e.g. [edit](crate::ur::Ur::edit)).
//! So [Ur](crate::ur::Ur)'s internal history data can not be thread-safe because the closures are stored as records in it.
//!
//! Unlike [Ur](crate::ur::Ur), [Aur](crate::aur::Aur) requires closures implementing [Send] and [Sync].
//! Therefore, [Aur](crate::aur::Aur) can implement [Send] and [Sync] because all of stored closures are guaranteed
//! to be thread-safe objects.
//!
//! To clarify the differences,
//! their interfaces are defined as traits in this module.
//!
//! All types in this crate implements some of the interface traits as appropriate.
//! Mappings between types and the traits are listed below.
//!
//! # Interface for undo-redo types
//! | Type                      | Trait                  |
//! | :------------------------ | :--------------------- |
//! | [Ur](crate::ur::Ur)       | [IUndoRedo] + [IEdit]  |
//! | [Cur](crate::cur::Cur)    | [IUndoRedo] + [IEdit]  |
//! | [Aur](crate::aur::Aur)    | [IUndoRedo] + [IEditA] |
//! | [Acur](crate::acur::Acur) | [IUndoRedo] + [IEditA] |
//!
//! # Interface for builder types
//! | Type                                    | Trait                    |
//! | :-------------------------------------- | :----------------------- |
//! | [UrBuilder](crate::ur::UrBuilder)       | [IBuilder] + [ITrigger]  |
//! | [CurBuilder](crate::cur::CurBuilder)    | [IBuilder] + [ITrigger]  |
//! | [AurBuilder](crate::aur::AurBuilder)    | [IBuilder] + [ITriggerA] |
//! | [AcurBuilder](crate::acur::AcurBuilder) | [IBuilder] + [ITriggerA] |
//!
//! # `edit` with side effects
//! [Ur](crate::ur::Ur) family have methods to change their state; [edit](IEdit::edit), [edit_if](IEdit::edit_if), and [try_edit](IUndoRedo::try_edit).
//! Users should choose which method to use depending on a closure has side effects.
//!
//! ## `edit` and `edit_if`
//! [edit](IEdit::edit) and [edit_if](IEdit::edit_if) MUST take closures that produces a same result for a same input.
//! The reason why is that the closures are stored in the history and used reproducing old states
//! for undoing.
//!
//! ## `try_edit`
//! [try_edit](IUndoRedo::try_edit) accepts closures that can never reproduce same output again.
//!
//! Generally, closures including following functions should use [try_edit](IUndoRedo::try_edit):
//!
//! - I/O
//! - IPC
//! - random
//!
//! etc.
use crate::metrics::Metrics;

/// A common interface for [Ur](crate::ur::Ur) and othres.
///
/// Following traits provides additional interfaces.
/// - [IEdit]
/// - [IEditA]
///
/// See [module-level documentation](crate::interface) for more details.
pub trait IUndoRedo {
    /// A type of state managed in Self.
    type State;

    /// Returns the current state object, consuming the self.
    fn into_inner(self) -> Self::State;

    /// Returns the maximum number of changes stored in the history.
    fn capacity(&self) -> Option<usize>;

    /// Returns the number of versions older than current state in the history.
    fn undoable_count(&self) -> usize;

    /// Returns the number of versions newer than current state in the history.
    fn redoable_count(&self) -> usize;

    /// Restores the previous state.
    /// Same as `self.undo_multi(1)`.
    ///
    /// # Return
    /// [None] is returned if no older version exists in the history,
    /// otherwise immutable reference to the updated internal state.
    fn undo(&mut self) -> Option<&Self::State> {
        self.undo_multi(1)
    }

    /// Undo multiple steps.
    /// This method is more efficient than running `self.undo()` multiple times.
    ///
    /// # Return
    /// [None] is returned if the target version is out of the history,
    /// otherwise immutable reference to the updated internal state.
    /// If `count=0`, this method does nothing and returns reference to the current state.
    fn undo_multi(&mut self, count: usize) -> Option<&Self::State>;

    /// Restores the next state.
    /// Same as `self.redo_multi(1)`.
    ///
    /// # Return
    /// [None] is returned if no newer version exists in the history,
    /// otherwise immutable reference to the updated internal state.
    fn redo(&mut self) -> Option<&Self::State> {
        self.redo_multi(1)
    }

    /// Redo multiple steps.
    /// This method is more efficient than running `self.redo()` multiple times.
    ///
    /// # Return
    /// [None] is returned if the target version is out of the history,
    /// otherwise immutable reference to the updated internal state.
    /// If `count=0`, this method does nothing and returns reference to the current state.
    fn redo_multi(&mut self, count: usize) -> Option<&Self::State>;

    /// Undo-redo bidirectionally.
    /// This is integrated method of [undo_multi](Self::undo_multi) and [redo_multi](Self::redo_multi).
    ///
    /// - `count < 0` => `self.undo_multi(-count)`.
    /// - `0 < count` => `self.redo_multi(count)`.
    fn jump(&mut self, count: isize) -> Option<&Self::State> {
        if count < 0 {
            self.undo_multi(count.abs() as usize)
        } else {
            self.redo_multi(count as usize)
        }
    }

    /// Takes a closure and update the internal state.
    /// The closure consumes the current state and produces a new state or an error.
    /// If the closure returns an error, the internal state is not changed.
    ///
    /// # Return
    /// Immutable reference to the new state or an error produced by the closure.
    ///
    /// # Remark
    /// Unlike [edit](IEdit::edit) and [edit_if](IEdit::edit_if), [try_edit](IUndoRedo::try_edit) accepts closures that can never reproduce same output again.
    /// After changing the internal state by the closure, a snapshot is taken for undoability.
    ///
    /// Generally, closures including following functions should use this method:
    ///
    /// - I/O
    /// - IPC
    /// - random
    ///
    /// etc.
    ///
    /// See also ["`edit` with side effects"](crate::interface#edit-with-side-effects) for more details.
    fn try_edit<F>(&mut self, command: F) -> Result<&Self::State, Box<dyn std::error::Error>>
    where
        F: FnOnce(Self::State) -> Result<Self::State, Box<dyn std::error::Error>>;
}

/// A interface for non-thread safe types.
///
/// See [module-level documentation](crate::interface) for more details.
pub trait IEdit<'a> {
    /// A type of state managed in Self.
    type State;

    /// Takes a closure and update the internal state.
    /// The closure consumes the current state and produces a new state.
    ///
    /// # Return
    /// Immutable reference to the new state.
    ///
    /// # Remarks
    /// The closure MUST produce a same result for a same input.
    /// If it is impossible, use [try_edit](IUndoRedo::try_edit).
    ///
    /// See also ["`edit` with side effects"](crate::interface#edit-with-side-effects) for more details.
    fn edit<F>(&mut self, command: F) -> &Self::State
    where
        F: Fn(Self::State) -> Self::State + 'a,
    {
        // NOTE
        // This call is guaranteed to succeed.
        unsafe { self.edit_if(move |s| Some(command(s))).unwrap_unchecked() }
    }

    /// Takes a closure and update the internal state.
    /// The closure consumes the current state and produces a new state or [None].
    /// If the closure returns [None], the internal state is not changed.
    ///
    /// # Return
    /// Immutable reference to the new state or [None].
    ///
    /// # Remarks
    /// The closure MUST produce a same result for a same input.
    /// If it is impossible, use [try_edit](IUndoRedo::try_edit).
    ///
    /// See also ["`edit` with side effects"](crate::interface#edit-with-side-effects) for more details.
    fn edit_if<F>(&mut self, command: F) -> Option<&Self::State>
    where
        F: Fn(Self::State) -> Option<Self::State> + 'a;
}

/// A interface of `edit` and `edit_if` for thread-safe types.
///
/// See [module-level documentation](crate::interface) for more details.
pub trait IEditA<'a> {
    /// A type of state managed in Self.
    type State;

    /// Takes a closure and update the internal state.
    /// The closure consumes the current state and produces a new state.
    ///
    /// The difference with the [IEdit] is that
    /// this method requires [Send] and [Sync] for the closure.
    ///
    /// # Return
    /// Immutable reference to the new state.
    ///
    /// # Remarks
    /// The closure MUST produce a same result for a same input.
    /// If it is impossible, use [try_edit](IUndoRedo::try_edit).
    ///
    /// See also ["`edit` with side effects"](crate::interface#edit-with-side-effects) for more details.
    fn edit<F>(&mut self, command: F) -> &Self::State
    where
        F: Fn(Self::State) -> Self::State + Send + Sync + 'a,
    {
        // NOTE
        // This call is guaranteed to succeed.
        unsafe { self.edit_if(move |s| Some(command(s))).unwrap_unchecked() }
    }

    /// Takes a closure and update the internal state.
    /// The closure consumes the current state and produces a new state or [None].
    /// If the closure returns [None], the internal state is not changed.
    ///
    /// The difference with [IEdit] is that
    /// this method requires [Send] and [Sync] for the closure.
    ///
    /// # Return
    /// Immutable reference to the new state or [None].
    ///
    /// # Remarks
    /// The closure MUST produce a same result for a same input.
    /// If it is impossible, use [try_edit](IUndoRedo::try_edit).
    ///
    /// See also ["`edit` with side effects"](crate::interface#edit-with-side-effects) for more details.
    fn edit_if<F>(&mut self, command: F) -> Option<&Self::State>
    where
        F: Fn(Self::State) -> Option<Self::State> + Send + Sync + 'a;
}

/// A common interface for all builder types.
///
/// Following traits provides additional builder interfaces.
/// - [ITrigger]
/// - [ITriggerA]
///
/// See [module-level documentation](crate::interface) for more details.
pub trait IBuilder {
    /// A type of state managed in [Target](Self::Target)
    type State;
    /// A type created by this builder.
    type Target;

    /// Set the maximum number of changes stored in the history.
    ///
    /// When more changes are applied than the capacity, the oldest record in the history is removed.
    ///
    /// # Remarks
    /// `capacity=0` means no limit.
    fn capacity(self, capacity: usize) -> Self;

    /// Creates a new [Target](Self::Target) object with an initial state of [State](Self::State).
    fn build(self, initial_state: Self::State) -> Self::Target;
}

/// A interface of `snapshot_trigger` for builders of non-thread safe types.
///
/// See [module-level documentation](crate::interface) for more details.
pub trait ITrigger<'a> {
    /// Takes a closure to decide whether to take a snapshot of internal state.
    ///
    /// # Remarks
    /// [snapshot_never](crate::triggers::snapshot_trigger::snapshot_never) are used as a default
    /// trigger.
    /// Note that it may cause performance problem.
    /// See ["Snapshot trigger"](crate::triggers#snapshot-trigger) for more details.
    fn snapshot_trigger<F>(self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + 'a;
}

/// A interface of `snapshot_trigger` for builders of thread-safe types.
///
/// See [module-level documentation](crate::interface) for more details.
pub trait ITriggerA<'a> {
    /// Takes a closure to decide whether to take a snapshot of internal state.
    ///
    /// The difference with the [ITrigger] is that
    /// this method requires [Send] and [Sync] for the closure.
    ///
    /// # Remarks
    /// [snapshot_never](crate::triggers::snapshot_trigger::snapshot_never) are used as a default
    /// trigger.
    /// Note that it may cause performance problem.
    /// See ["Snapshot trigger"](crate::triggers#snapshot-trigger) for more details.
    fn snapshot_trigger<F>(self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + Send + Sync + 'a;
}
