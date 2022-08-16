use crate::data_manipulation::anonymizable::Anonymizable;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Analyses the average delay a tuple experiences between being put
/// inside the algorithm and being published
#[derive(Default)]
pub struct PublishingDelayAnalyser {
    count: i32,
    sum_delays: Duration,
}

impl PublishingDelayAnalyser {
    pub fn add_delay<A: Anonymizable>(&mut self, value: &A) {
        self.count += 1;
        self.sum_delays += SystemTime::now()
            .duration_since(value.get_timestamp())
            .unwrap()
    }

    pub fn calculate_average_delay(&self) -> Duration {
        self.sum_delays.div_f64(self.count as f64)
    }
}
