use std::time::Duration;

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

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
    pub fn elapsed_from_snapshot(&self) -> Duration {
        self.elapsed_from_snapshot
    }
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
