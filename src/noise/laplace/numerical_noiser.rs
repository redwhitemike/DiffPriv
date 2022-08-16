use crate::data_manipulation::anonymizable::{IntervalType, QuasiIdentifierType};
use crate::noise::laplace::laplace_noiser::LOC;
use float_next_after::NextAfter;
use num::abs;
use num::integer::Roots;
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use std::collections::VecDeque;

/// Noiser for numerical QI types
#[derive(Clone)]
pub struct NumericalNoiser {
    eps: f64,                      // differential privacy parameter
    k: usize,                      // k anonymity level
    history_window: VecDeque<f64>, // vector containing all the previous laplace noises with a maximum of MAX_HISTORY
    max: f64,                      // maximal value observed in the noiser
    min: f64,                      // minimal value observed in the noiser
    qi_amount: f64,                // count of qi's in stream
    window: usize,                 // window of historic values
}

impl NumericalNoiser {
    /// create a new numerical noiser and initialize the first values
    pub fn initialize(eps: f64, k: usize, qi_amount: f64, interval: &IntervalType) -> Self {
        let (qi_type, _, _, _) = interval;
        let value = Self::extract_convert_value(qi_type);

        let window = match k.sqrt() as usize {
            val if val <= 2 => 2,
            val if val > 2 => val,
            _ => panic!("value couldn't be calculated"),
        };

        Self {
            eps,
            k,
            max: value,
            min: value,
            qi_amount,
            window,
            ..Default::default()
        }
    }

    /// extract the value from a `QuasiIdentifierType` and return the
    /// (converted to f64) value
    fn extract_convert_value(interval: &QuasiIdentifierType) -> f64 {
        match *interval {
            QuasiIdentifierType::Integer(value) => value as f64,
            QuasiIdentifierType::Float(value) => value,
        }
    }

    /// calculate the noise with an estimate of a scale
    pub fn generate_noise(&mut self, interval: &IntervalType) -> f64 {
        let (value, _, _, _) = interval;
        let value = Self::extract_convert_value(value);
        let scale = self.estimate_scale(value);

        let between = Uniform::<f64>::from(-0.5..0.5);
        let mut rng = thread_rng();
        let mut sign = 1.0;
        let unif = between.sample(&mut rng);
        let diff = 0_f64.next_after(1_f64).max(1.0 - 2.0 * abs(unif));

        if unif < 0.0 {
            sign = -1.0;
        }

        LOC - (scale * sign * diff.ln())
    }

    /// return the estimated scale based on the history of previous
    /// laplace noises
    fn estimate_scale(&mut self, value: f64) -> f64 {
        if value < self.min {
            self.min = value
        }
        if value > self.max {
            self.max = value
        }

        if self.history_window.len() > self.window {
            self.history_window.pop_front();
        }
        self.history_window.push_back(value);
        let predicted_sensitivity = self
            .history_window
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
            - self
                .history_window
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

        0.5 * self.qi_amount * predicted_sensitivity / (self.k as f64 * self.eps)
    }
}

impl Default for NumericalNoiser {
    fn default() -> Self {
        Self {
            eps: 0.0,
            k: 0,
            history_window: VecDeque::new(),
            max: 0.0,
            min: 0.0,
            qi_amount: 0.0,
            window: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::noise::laplace::laplace_noiser::LOC;
    use float_next_after::NextAfter;
    use num::abs;
    use rand::distributions::Uniform;
    use rand::prelude::Distribution;
    use rand::thread_rng;

    const BUCKET_COUNT: usize = 100;
    const SAMPLE_SIZE: usize = 50000;

    fn truncate(index: i32) -> usize {
        if index >= BUCKET_COUNT as i32 {
            return BUCKET_COUNT - 1;
        }

        if index < 0 {
            return 0_usize;
        }

        index as usize
    }

    fn x_to_index(x: f64, x0: f64, x1: f64) -> usize {
        let index = ((x - x0) / (x1 - x0) * BUCKET_COUNT as f64).floor() as i32;
        truncate(index)
    }

    #[test]
    fn generate_noise() {
        let scale = 1.0;
        let mut buckets = [0; BUCKET_COUNT];

        let x0 = -8.0;
        let x1 = 8.0;

        for _ in 0..SAMPLE_SIZE {
            let between = Uniform::<f64>::from(-0.5..0.5);
            let mut rng = thread_rng();
            let mut sign = 1.0;
            let unif = between.sample(&mut rng);
            let diff = 0_f64.next_after(1_f64).max(1.0 - 2.0 * abs(unif));

            if unif < 0.0 {
                sign = -1.0;
            }

            let x = LOC - (scale * sign * diff.ln());
            let index = x_to_index(x, x0, x1);
            buckets[index] += 1;
        }

        let x_test = -1.0;
        let index_test = x_to_index(x_test, x0, x1);
        let fraction_below_x_test =
            buckets[0..index_test].iter().sum::<i32>() as f64 / SAMPLE_SIZE as f64;

        assert!((0.16..=0.17).contains(&fraction_below_x_test));
    }
}
