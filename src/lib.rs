//! Wrapper types to provide undo-redo functionality.
//!
//! # Sample code
//! ```
//! use gur::prelude::*;
//! use gur::cur::{Cur, CurBuilder};
//!
//! // Appication state
//! #[derive(Clone)]
//! struct MyState {
//!     data: String
//! }
//!
//! fn main() {
//!     // Initialize
//!     let mut state: Cur<MyState> = CurBuilder::new().build(MyState{ data: "My".to_string() });
//!     assert_eq!("My", state.data);
//!
//!     // Change state
//!     state.edit(|mut state: MyState| { state.data += "State"; state });
//!     assert_eq!("MyState", state.data);
//!
//!     // Undo
//!     state.undo();
//!     assert_eq!("My", state.data);
//!
//!     // Redo
//!     state.redo();
//!     assert_eq!("MyState", state.data);
//! }
//! ```
//! The `MyState` is a type of user's application state.
//! `MyState` implements the [Clone] trait to use it with [Cur](crate::cur::Cur).
//! Then the variable `state : Cur<MyState>` is created to get the ability to undo-redo.
//!
//! The [edit](crate::interface::IEdit::edit) takes a closure to change the variable.
//! The closure is a function that consumes a current state and returns a new state.
//! A previous state can be restored by calling the [undo](crate::interface::IUndoRedo::undo).
//! The [redo](crate::interface::IUndoRedo::redo) is reverse operation of the [undo](crate::interface::IUndoRedo::undo).
//!
//! The [Cur](crate::cur::Cur) implements [Deref](std::ops::Deref).
//! So its internal state can be accessed like `*state`.
//!
//! # `Ur` family
//! [Ur](crate::ur::Ur) is a most basic type in this crate.
//! Some variants are provided in this crate.
//! The variants and their features are listed below.
//!
//! | Type                           | Trait bounds                               | Thread safety | Description                                                                   |
//! | :----------------------------- | :----------------------------------------- | :-----------: | :---------------------------------------------------------------------------- |
//! | [Ur\<T\>](crate::ur::Ur)       | `T`: [Snapshot](crate::snapshot::Snapshot) | No            | A basic wrapper for types implementing [Snapshot](crate::snapshot::Snapshot). |
//! | [Cur\<T\>](crate::cur::Cur)    | `T`: [Clone]                               | No            | Another simple wrapper for types implementing [Clone].                        |
//! | [Aur\<T\>](crate::aur::Aur)    | `T`: [Snapshot](crate::snapshot::Snapshot) | Yes           | [Ur\<T\>](crate::ur::Ur) + [Send] + [Sync]                                    |
//! | [Acur\<T\>](crate::acur::Acur) | `T`: [Clone]                               | Yes           | [Cur\<T\>](crate::cur::Cur) + [Send] + [Sync]                                 |
//!
//! ## Trait bounds
//! For example, [Ur\<T\>](crate::ur::Ur) requires `T` implementing [Snapshot](crate::snapshot::Snapshot).
//! On the other hand, [Cur\<T\>](crate::cur::Cur) requires [Clone] instead of [Snapshot](crate::snapshot::Snapshot) for simplicity.
//!
//! ## Thread safety
//! Some types of them can be [Send] and [Sync] and some not.
//! See [interface](crate::interface) for more details about thread safety.
//!
//! # Generative approach
//! This section describes a basic concept of this crate.
//! The concept is that "Undoing is regenerating (recomputing) the old state."
//!
//! For explanation, a sample history of changes is shown as follows,
//! ```txt
//! t: state
//! c: command
//! s: snapshot
//!
//! old <----------------------> new
//!      c1        c2        c3
//! t0 -----> t1 -----> t2 -----> t3
//! |  +-------------->           |
//! |  |                          |
//! s0 +--------------------------+
//!            undo t3 -> t2
//! ```
//! Where `tx` is an application state at time point `x`,
//! `cx` is a command to change a state `tx-1` to next `tx`,
//! and `sx` is a snapshot of a state `tx`.
//!
//! The application state have changed in order of `t0`, `t1`, `t2`, and `t3`.
//! Now, the current state is `t3`.
//! In the sample history, a snapshot `s0`, a snapshot of initial state `t0`, have been created.
//!
//! Let us consider undoing the current state `t3` to its previous state `t2`.
//! First, the system restores a old state from its snapshot at any point in the history.
//! In this case, We would have to restore the state `t0` from `s0` because there is only one snapshot `s0`.
//! Then the system reruns the commands (`c1` and `c2`) in order.
//! Finally, the target state `t2` will be obtained.
//!
//! # Data structure
//! A history is managed as chain of commands and snapshots to perform the
//! process described above.
//! No intermediate states are stored.
//! ```txt
//! c: command
//! s: snapshot
//!
//! old <----------------------------------------------> new
//! /--\  +--+  +--+       +--+  /----\  +----+  +----+
//! |s0|--|c1|--|c2|--...--|cn|--|sn+1|--|cn+2|--|cn+3|--...
//! \--/  +--+  +--+       +--+  \----/  +----+  +----+
//! ```
//! The frequency of snapshots can be customized by "trigger functions."
//! See [crate::triggers] for more details.
//!
//! # Pros and Cons
//! ## Pros
//! The advantages of this approach are usability and robustness.
//! There are no backward opeartions in the undoing process.
//! So users almost never have to write additional codes for the process.
//! If there are tests for the application state object, the correctness of undo-redo process is
//! also guaranteed.
//!
//! ## Cons
//! Users should pay attention to side effects of commands.
//!
//! See also ["`edit` with side effects"](crate::interface#edit-with-side-effects) for more details.
mod agur;
mod gur;
mod history;

pub mod acur;
pub mod aur;
pub mod cur;
pub mod interface;
pub mod metrics;
pub mod snapshot;
pub mod triggers;
pub mod ur;

pub mod prelude {
    //! Re-export common members
    pub use crate::interface::*;
}
