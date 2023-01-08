//! Triggers documentation
//!
//! # Snapshot trigger
//! The generative approach described [here](crate#generative-approach) may cause performance problem.
//! For example, undoing may take too long time if the commands are heavy computational tasks.
//!
//! The problem can be mitigated by taking snapshots frequently.
//! [UrBuilder](crate::ur::UrBuilder) provides a way to control the frequency.
//! That is "snapshot triggers."
//!
//! The snapshot trigger is used for deciding whether a snapshot should be created for each change.
//! This trigger will be called after updating the internal state in the [edit](crate::interface::IEdit::edit).
//! When the trigger returns true,
//! a snapshot is created from the state and is stored in the history instead of the command.
//!
//! Here is a simple example; a trigger based on elapsed time of changes.
//! ```rust
//! use gur::prelude::*;
//! use gur::cur::{Cur, CurBuilder};
//! use gur::metrics::Metrics;
//! use std::time::Duration;
//!
//! fn snapshot_500ms(metrics: &Metrics) -> bool {
//!     Duration::from_millis(500) < metrics.elapsed_from_snapshot()
//! }
//!
//! fn main() {
//!     let mut ur = CurBuilder::new().snapshot_trigger(snapshot_500ms).build(0);
//! }
//! ```
//! In this example, the function "snapshot_500ms" returns true when total computation time
//! of commands from last snapshot exceeds 500 ms.
//! So the internal history chain will become like following,
//! ```txt
//! c: command
//! s: snapshot
//!
//!       <----- (500ms<) ---->          <------- (500ms<) ------>
//! /--\  +--+  +--+       +--+  /----\  +----+  +----+       +--+  /----\
//! |s0|--|c1|--|c2|--...--|cm|--|sm+1|--|cm+2|--|cm+3|--...--|cn|--|sn+1|--...
//! \--/  +--+  +--+       +--+  \----/  +----+  +----+       +--+  \----/
//! ```
//! The total computation time between two snapshots (e.g. c1 + c2 + ... + cm) is limited within
//! 500 ms.
//! Therefore, it is expected that any undoing processes are completed within 500 ms.
//!
//! Some predefined triggers are found in [snapshot_trigger].
pub mod snapshot_trigger;
