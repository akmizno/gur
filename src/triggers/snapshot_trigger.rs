//! Predefined snapshot triggers
use crate::metrics::Metrics;
use std::time::Duration;

/// Creates a snapshot trigger by total elapsed time.
///
/// This function returns a closure object comparing the specified duration and
/// [Metrics::elapsed_from_snapshot](crate::metrics::Metrics::elapsed_from_snapshot).
/// The closure activates when the elapsed time exceeds the specified duration.
///
/// # Usage
/// ```rust
/// use gur::prelude::*;
/// use gur::cur::{Cur, CurBuilder};
/// use gur::metrics::Metrics;
/// use gur::triggers::snapshot_trigger::snapshot_by_total_elapsed;
/// use std::time::Duration;
///
/// fn main() {
///     let mut ur = CurBuilder::new()
///                 .snapshot_trigger(snapshot_by_total_elapsed(Duration::from_millis(500)))
///                 .build(0);
/// }
/// ```
pub fn snapshot_by_total_elapsed(duration: Duration) -> impl Fn(&Metrics) -> bool {
    move |metrics| duration < metrics.elapsed_from_snapshot()
}

/// Creates a snapshot trigger by distance from last snapshot.
///
/// This function returns a closure object comparing the specified distance and
/// [Metrics::distance_from_snapshot](crate::metrics::Metrics::distance_from_snapshot).
/// The closure activates when the distance exceeds the specified value.
///
/// # Usage
/// ```rust
/// use gur::prelude::*;
/// use gur::cur::{Cur, CurBuilder};
/// use gur::metrics::Metrics;
/// use gur::triggers::snapshot_trigger::snapshot_by_distance;
/// use std::time::Duration;
///
/// fn main() {
///     let mut ur = CurBuilder::new()
///                 .snapshot_trigger(snapshot_by_distance(10))
///                 .build(0);
/// }
/// ```
pub fn snapshot_by_distance(distance: usize) -> impl Fn(&Metrics) -> bool {
    move |metrics| distance < metrics.distance_from_snapshot()
}

/// Creates a "always-on" snapshot trigger.
///
/// The created closure returns true everytime.
/// By using this trigger, snapshots will always been taken for each [IEdit::edit](crate::interface::IEdit::edit).
///
/// # Usage
/// ```rust
/// use gur::prelude::*;
/// use gur::cur::{Cur, CurBuilder};
/// use gur::metrics::Metrics;
/// use gur::triggers::snapshot_trigger::snapshot_always;
/// use std::time::Duration;
///
/// fn main() {
///     let mut ur = CurBuilder::new()
///                 .snapshot_trigger(snapshot_always())
///                 .build(0);
/// }
/// ```
pub fn snapshot_always() -> impl Fn(&Metrics) -> bool {
    |_metrics| true
}

/// Creates a "always-off" snapshot trigger.
///
/// The created closure returns false everytime.
/// By using this trigger, snapshots will never been taken for each [IEdit::edit](crate::interface::IEdit::edit).
///
/// # Remarks
/// Even if you use this trigger, [IUndoRedo::try_edit](crate::interface::IUndoRedo::try_edit) takes snapshots.
///
/// # Usage
/// ```rust
/// use gur::prelude::*;
/// use gur::cur::{Cur, CurBuilder};
/// use gur::metrics::Metrics;
/// use gur::triggers::snapshot_trigger::snapshot_never;
/// use std::time::Duration;
///
/// fn main() {
///     let mut ur = CurBuilder::new()
///                 .snapshot_trigger(snapshot_never())
///                 .build(0);
/// }
/// ```
pub fn snapshot_never() -> impl Fn(&Metrics) -> bool {
    |_metrics| false
}
