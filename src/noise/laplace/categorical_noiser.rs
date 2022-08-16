use crate::noise::laplace::laplace_noiser::CategoricalTypes;
use crate::vec_set::VecSet;
use rand::distributions::Distribution;
use rand::{thread_rng, Rng};
use rand_distr::Normal;

/// Noiser for categorical QI types
#[derive(Clone)]
pub struct CategoricalNoiser {
    observed_values: VecSet<i32>, // hashset containing all observed values of the QI
    noise_thr: f64,               // categorical noise threshold
    stream_weight: f64,           // sum weight of all the QI's
}

impl CategoricalNoiser {
    pub fn initialize(noise_thr: f64, stream_weight: usize) -> Self {
        Self {
            noise_thr,
            stream_weight: stream_weight as f64,
            ..Default::default()
        }
    }

    /// extract value from the possible categorical QI types
    fn extract_value(categorical: &CategoricalTypes) -> i32 {
        match *categorical {
            CategoricalTypes::Ordinal((rank, _, _)) => rank,
            CategoricalTypes::Nominal((value, _, _)) => value,
        }
    }

    /// generate noise for categorical QI types. Return the noise to be used
    /// instead of the original
    pub fn generate_noise(&mut self, categorical: CategoricalTypes) -> i32 {
        let value = Self::extract_value(&categorical);
        self.observed_values.insert(value);

        let mut random = thread_rng();
        let normal: Normal<f64> = Normal::new(0.0, 1.0).unwrap();
        let e = normal.sample(&mut random);

        match self.observed_values.len() > 1 && e < self.noise_thr * self.stream_weight as f64 {
            true => {
                let mut index = random.gen_range(0..self.observed_values.len());
                while *self.observed_values.get(index).unwrap() == value {
                    index = random.gen_range(0..self.observed_values.len());
                }
                *self.observed_values.get(index).unwrap()
            }
            false => value,
        }
    }
}

impl Default for CategoricalNoiser {
    fn default() -> Self {
        Self {
            observed_values: Default::default(),
            noise_thr: 0.0,
            stream_weight: 0.0,
        }
    }
}
