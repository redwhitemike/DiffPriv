/// Analyses the overall data loss experienced throughout the
/// algorithms lifetime
#[derive(Default)]
pub struct SseAnalyser {
    sum_info_loss: f64,
}

impl SseAnalyser {
    pub fn add_info_loss(&mut self, info_loss: f64) {
        self.sum_info_loss += info_loss
    }

    pub fn total_info_loss(&self) -> f64 {
        self.sum_info_loss
    }
}
