//! Triggers documentation
//!
//! # Snapshot trigger
//! The generative approach described [here](crate#Generative&#32;approach) may cause performance problem.
//! For example, undoing may take too long time if the commands are heavy computational tasks.
//!
//! The problem can be mitigated by taking snapshots more frequently.
//! [UrBuilder](crate::ur::UrBuilder) provides a way to control the frequency.
//! That is "snapshot triggers."
//!
//! The snapshot trigger is used for deciding whether a snapshot should be created for each change.
//! This trigger will be called after updating the internal state in the [Ur::edit](crate::ur::Ur::edit).
//! When the trigger returns true,
//! a snapshot is created from the state and is stored in the history instead of the command.
//!
//! A simple solution is based on elapsed time of changes.
//! ```rust
//! use gur::cur::{Cur, CurBuilder};
//! use gur::metrics::Metrics;
//! use std::time::Duration;
//!
//! fn Snapshot500ms(metrics: &Metrics) -> bool {
//!     Duration::from_millis(500) < metrics.elapsed_from_snapshot()
//! }
//!
//! fn main() {
//!     let mut ur = CurBuilder::new().snapshot_trigger(Snapshot500ms).build(0);
//! }
//! ```
//! In this example, the function "Snapshot500ms" returns true when total computation time
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
//! Therefore, any undo processes is expected that it is completed within 500 ms.
