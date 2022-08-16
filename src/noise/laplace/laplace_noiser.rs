use crate::data_manipulation::aggregation::truncate_to_domain;
use crate::data_manipulation::anonymizable::{
    Anonymizable, IntervalType, NominalType, OrdinalType, QuasiIdentifierType, QuasiIdentifierTypes,
};
use crate::noise::laplace::categorical_noiser::CategoricalNoiser;
use crate::noise::laplace::numerical_noiser::NumericalNoiser;
use crate::noise::noiser::Noiser;

/// location of laplace distribution (mu)
pub const LOC: f64 = 0.0;

/// possible noiser categories for the laplace noiser
#[derive(Clone)]
enum NoiserCategories {
    NumericalNoiser(NumericalNoiser),
    CategoricalNoiser(CategoricalNoiser),
}

/// QI types that support categorical noise
#[derive(Clone)]
pub enum CategoricalTypes {
    Nominal(NominalType),
    Ordinal(OrdinalType),
}

/// The laplace noice noiser used for introducing random noise to make the QI's
/// differentially private
#[derive(Default, Clone)]
pub struct LaplaceNoiser {
    eps: f64,                          // differential privacy parameter
    k: usize,                          // k anonymity level
    noise_thr: f64,                    // categorical noise threshold
    qi_noisers: Vec<NoiserCategories>, // vector containing all the different noisers for the QI's
}

impl LaplaceNoiser {
    pub fn new(eps: f64, k: usize, noise_thr: f64) -> Self {
        Self {
            eps,
            k,
            noise_thr,
            ..Default::default()
        }
    }

    /// generate and add noise to an interval QI type
    fn generate_noise_interval(
        &mut self,
        interval: IntervalType,
        qi_len: usize,
        index: usize,
    ) -> QuasiIdentifierTypes {
        match self.qi_noisers.get_mut(index) {
            None => {
                let mut noiser =
                    NumericalNoiser::initialize(self.eps, self.k, qi_len as f64, &interval);
                let noise = noiser.generate_noise(&interval);
                self.qi_noisers
                    .push(NoiserCategories::NumericalNoiser(noiser));
                QuasiIdentifierTypes::Interval(self.add_noise_interval(noise, interval))
            }
            Some(category) => match category {
                NoiserCategories::NumericalNoiser(noiser) => {
                    let noise = noiser.generate_noise(&interval);
                    QuasiIdentifierTypes::Interval(self.add_noise_interval(noise, interval))
                }
                _ => panic!("wrong noiser type detected"),
            },
        }
    }

    /// generate and add noise to an ordinal QI type
    fn generate_noise_ordinal(
        &mut self,
        ordinal: OrdinalType,
        stream_weight: usize,
        index: usize,
    ) -> QuasiIdentifierTypes {
        match self.qi_noisers.get_mut(index) {
            None => {
                let mut noiser = CategoricalNoiser::initialize(self.noise_thr, stream_weight);
                let noise = noiser.generate_noise(CategoricalTypes::Ordinal(ordinal));
                self.qi_noisers
                    .push(NoiserCategories::CategoricalNoiser(noiser));
                QuasiIdentifierTypes::Ordinal(self.add_noise_ordinal(noise, ordinal))
            }
            Some(category) => match category {
                NoiserCategories::CategoricalNoiser(noiser) => {
                    let noise = noiser.generate_noise(CategoricalTypes::Ordinal(ordinal));
                    QuasiIdentifierTypes::Ordinal(self.add_noise_ordinal(noise, ordinal))
                }
                _ => panic!("wrong noiser type detected"),
            },
        }
    }

    /// generate and add noise to an ordinal QI type
    fn generate_noise_nominal(
        &mut self,
        nominal: NominalType,
        stream_weight: usize,
        index: usize,
    ) -> QuasiIdentifierTypes {
        match self.qi_noisers.get_mut(index) {
            None => {
                let mut noiser = CategoricalNoiser::initialize(self.noise_thr, stream_weight);
                let noise = noiser.generate_noise(CategoricalTypes::Nominal(nominal));
                self.qi_noisers
                    .push(NoiserCategories::CategoricalNoiser(noiser));
                QuasiIdentifierTypes::Nominal(self.add_noise_nominal(noise, nominal))
            }
            Some(categorical) => match categorical {
                NoiserCategories::CategoricalNoiser(noiser) => {
                    let noise = noiser.generate_noise(CategoricalTypes::Nominal(nominal));
                    QuasiIdentifierTypes::Nominal(self.add_noise_nominal(noise, nominal))
                }
                _ => panic!("wrong noiser type detected"),
            },
        }
    }

    /// add noise to a interval QI type value
    pub fn add_noise_interval(&self, noise: f64, interval: IntervalType) -> IntervalType {
        match interval {
            (
                QuasiIdentifierType::Float(value),
                QuasiIdentifierType::Float(min_value),
                QuasiIdentifierType::Float(max_value),
                weight,
            ) => (
                QuasiIdentifierType::Float(truncate_to_domain(value + noise, min_value, max_value)),
                QuasiIdentifierType::Float(min_value),
                QuasiIdentifierType::Float(max_value),
                weight,
            ),
            (
                QuasiIdentifierType::Integer(value),
                QuasiIdentifierType::Integer(min_value),
                QuasiIdentifierType::Integer(max_value),
                weight,
            ) => (
                QuasiIdentifierType::Integer(truncate_to_domain(
                    (value as f64 + noise) as i32,
                    min_value,
                    max_value,
                )),
                QuasiIdentifierType::Integer(min_value),
                QuasiIdentifierType::Integer(max_value),
                weight,
            ),
            _ => {
                panic!("Wrong typing combination when adding noise to interval value")
            }
        }
    }

    /// add generated noise to a nominal QI value
    fn add_noise_nominal(&self, noise: i32, nominal: NominalType) -> NominalType {
        let (_, max_value, weight) = nominal;
        (noise, max_value, weight)
    }

    /// add generated noise to a ordinal QI value
    fn add_noise_ordinal(&self, noise: i32, ordinal: OrdinalType) -> OrdinalType {
        let (_, max_rank, weight) = ordinal;
        (noise, max_rank, weight)
    }

    /// calculate the full weight of all the QI's
    fn calculate_stream_weight(&self, qi: &[QuasiIdentifierTypes]) -> usize {
        qi.iter()
            .map(|x| match x {
                QuasiIdentifierTypes::Interval((_, _, _, weight)) => weight,
                QuasiIdentifierTypes::Ordinal((_, _, weight)) => weight,
                QuasiIdentifierTypes::Nominal((_, _, weight)) => weight,
            })
            .sum()
    }
}

impl Noiser for LaplaceNoiser {
    fn add_noise<M: Anonymizable>(&mut self, value: &M) -> Vec<QuasiIdentifierTypes> {
        let qi = value.quasi_identifiers();
        let qi_len = qi.len();
        let stream_weight = match self.qi_noisers.is_empty() {
            true => self.calculate_stream_weight(&qi),
            false => 0,
        };

        qi.into_iter()
            .enumerate()
            .map(|(index, x)| match x {
                QuasiIdentifierTypes::Interval(interval) => {
                    self.generate_noise_interval(interval, qi_len, index)
                }
                QuasiIdentifierTypes::Ordinal(ordinal) => {
                    self.generate_noise_ordinal(ordinal, stream_weight, index)
                }
                QuasiIdentifierTypes::Nominal(nominal) => {
                    self.generate_noise_nominal(nominal, stream_weight, index)
                }
            })
            .collect::<Vec<QuasiIdentifierTypes>>()
    }
}
