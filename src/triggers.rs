//! Triggers documentation
//!
//! # Snapshot trigger
//! The snapshot trigger is used for deciding whether a snapshot should be created.
//! This will be invoked after updating the internal state in the [Ur::edit](crate::ur::Ur::edit).
//! If the trigger returns true,
//! a snapshot is created from the state and is stored in the history instead of the action.
