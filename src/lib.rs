//! Wrapper types to provide undo-redo functionality.
//!
//! [Ur](crate::ur::Ur) is a most basic type in this crate.
//! Some variants are provided in this crate.
//!
//! - [Ur](crate::ur::Ur): A basic wrapper for types implementing [Snapshot](crate::snapshot::Snapshot).
//! - [Cur](crate::cur::Cur): Another simple wrapper for types implementing [Clone].
//! - [Aur](crate::aur::Aur): [Ur](crate::ur::Ur) + [Send] + [Sync].
//! - [Acur](crate::acur::Acur): [Cur](crate::cur::Cur) + [Send] + [Sync].
//!
//! # Generative approach
//! A policy of this crate is that "Undo is regenerating (recomputing) the old state."
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
//! In this sample history, only one snapshot `s0` exists.
//!
//! The application state have changed in order of `t0`, `t1`, `t2`, and `t3`.
//! Now, the current state is `t3`.
//!
//! When undoing the current state `t3` to its previous state `t2`,
//! the system gets back to the initial state `t0` by restoring it from the snapshot `s0`,
//! and redo the commands (`c1` and `c2`) in order.
//! Then, the target state `t2` should be obtained.
//!
//! # Data structure
//! A history is managed as chain of commands and snapshots to perform the
//! above procedure.
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
mod gur;
mod history;

pub mod acur;
pub mod aur;
pub mod cur;
pub mod metrics;
pub mod snapshot;
pub mod triggers;
pub mod ur;
