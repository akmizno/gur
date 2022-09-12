use std::time::Duration;

pub struct Metrics {
    exec_time: Duration,
    time_from_snapshot: Duration,
    distance: usize,
}

impl Metrics {
    pub fn zero() -> Self {
        Self {
            exec_time: Duration::ZERO,
            time_from_snapshot: Duration::ZERO,
            distance: 0,
        }
    }

    pub fn exec_time(&self) -> Duration {
        self.exec_time
    }
    pub fn time_from_snapshot(&self) -> Duration {
        self.time_from_snapshot
    }
    pub fn distance_from_snapshot(&self) -> usize {
        self.distance
    }

    pub(crate) fn make_next(&self, next_duration: Duration) -> Self {
        let accumulated = next_duration + self.time_from_snapshot();
        Self {
            exec_time: next_duration,
            time_from_snapshot: accumulated,
            distance: 1 + self.distance_from_snapshot(),
        }
    }
}
