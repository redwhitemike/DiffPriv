/// Analyses the mean of square errors between the
/// original data tuple and its anonymized version throughout the
/// algorithms lifetime
pub struct MseAnalyser {
    count: i32,
    sum_square_erros: f64,
}

impl MseAnalyser {
    pub fn add_error(&mut self, square_errors: f64) {
        self.count += 1;
        self.sum_square_erros += square_errors
    }

    pub fn calculate_mse(&self) -> f64 {
        self.sum_square_erros / self.count as f64
    }
}

impl Default for MseAnalyser {
    fn default() -> Self {
        Self {
            count: 0,
            sum_square_erros: 0.0,
        }
    }
}
