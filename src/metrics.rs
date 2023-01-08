//! Metrics of commands
use std::time::Duration;

/// Metrics of commands.
///
/// A trigger function is called for each command with this type of object.
/// The trigger makes decisions whether a snapshot should be taken or not
/// using the information in this type.
/// See [Snapshot trigger](crate::triggers#snapshot-trigger) for more details.
#[derive(Clone, Debug)]
pub struct Metrics {
    elapsed: Duration,
    elapsed_from_snapshot: Duration,
    distance: usize,
}

impl Metrics {
    pub(crate) fn zero() -> Self {
        Self {
            elapsed: Duration::ZERO,
            elapsed_from_snapshot: Duration::ZERO,
            distance: 0,
        }
    }

    /// Elapsed time of one command.
    ///
    /// ```txt
    /// c: command
    /// s: snapshot
    ///
    /// c1    <-->
    /// c2          <-->
    /// c3                <-->
    /// c4                      <-->
    /// /--\  +--+  +--+  +--+  +--+
    /// |s0|--|c1|--|c2|--|c3|--|c4|
    /// \--/  +--+  +--+  +--+  +--+
    /// ```
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Total elapsed time of commands from last snapshot.
    ///
    /// ```txt
    /// c: command
    /// s: snapshot
    ///
    /// c1    <-->
    /// c2    <-------->
    /// c3    <-------------->
    /// c4    <-------------------->
    /// /--\  +--+  +--+  +--+  +--+
    /// |s0|--|c1|--|c2|--|c3|--|c4|
    /// \--/  +--+  +--+  +--+  +--+
    /// ```
    pub fn elapsed_from_snapshot(&self) -> Duration {
        self.elapsed_from_snapshot
    }

    /// Distance (number of commands) from last snapshot.
    ///
    /// ```txt
    /// c: command
    /// s: snapshot
    ///
    /// c1    1 command
    /// c2    2 commands
    /// c3    3 commands
    /// c4    4 commands
    /// /--\  +--+  +--+  +--+  +--+
    /// |s0|--|c1|--|c2|--|c3|--|c4|
    /// \--/  +--+  +--+  +--+  +--+
    /// ```
    pub fn distance_from_snapshot(&self) -> usize {
        self.distance
    }

    pub(crate) fn make_next(&self, next_duration: Duration) -> Self {
        let accumulated = next_duration + self.elapsed_from_snapshot();
        Self {
            elapsed: next_duration,
            elapsed_from_snapshot: accumulated,
            distance: 1 + self.distance_from_snapshot(),
        }
    }
}
