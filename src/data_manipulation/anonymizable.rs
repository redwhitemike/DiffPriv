use crate::data_manipulation::aggregation::truncate_to_domain;
use num::abs;
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use rand_distr::Normal;
use serde::Serialize;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Hash, Eq, PartialEq)]
pub enum SensitiveAttribute {
    String(String),
    Integer(i32),
}

/// value, min_value, max_value, weight of attribute
pub type IntervalType = (
    QuasiIdentifierType,
    QuasiIdentifierType,
    QuasiIdentifierType,
    usize,
);

/// rank, max_rank, weight of attribute
pub type OrdinalType = (i32, i32, usize);

/// value, max value, weight of attribute
pub type NominalType = (i32, i32, usize);

#[derive(Debug, Copy, Clone)]
pub enum QuasiIdentifierType {
    Float(f64),
    Integer(i32),
}

/// Possible quasi identifier data category types
#[derive(Debug)]
pub enum QuasiIdentifierTypes {
    /// value, min_value, max_value, weight of attribute
    Interval(IntervalType),
    /// rank, max_rank, weight of attribute
    Ordinal(OrdinalType),
    /// value, weight of attribute
    Nominal(NominalType),
}

impl QuasiIdentifierTypes {
    /// consume itself and extract the value of the quasi identifier
    pub fn extract_value(self) -> QuasiIdentifierType {
        match self {
            QuasiIdentifierTypes::Interval((value, _, _, _)) => value,
            QuasiIdentifierTypes::Ordinal((value, _, _)) => QuasiIdentifierType::Integer(value),
            QuasiIdentifierTypes::Nominal((value, _, _)) => QuasiIdentifierType::Integer(value),
        }
    }

    pub fn randomize(self) -> QuasiIdentifierTypes {
        let mut rng = thread_rng();
        match self {
            QuasiIdentifierTypes::Interval((value, min, max, weight)) => match (value, min, max) {
                (
                    QuasiIdentifierType::Float(val_fl),
                    QuasiIdentifierType::Float(min_val),
                    QuasiIdentifierType::Float(max_val),
                ) => {
                    let normal: Normal<f64> = Normal::new(val_fl, 1.0).unwrap();
                    let e = normal.sample(&mut rng);
                    QuasiIdentifierTypes::Interval((
                        QuasiIdentifierType::Float(truncate_to_domain(e, min_val, max_val)),
                        QuasiIdentifierType::Float(min_val),
                        QuasiIdentifierType::Float(max_val),
                        weight,
                    ))
                }
                (
                    QuasiIdentifierType::Integer(val_int),
                    QuasiIdentifierType::Integer(min_val),
                    QuasiIdentifierType::Integer(max_val),
                ) => {
                    let normal: Normal<f64> = Normal::new(val_int as f64, 1.0).unwrap();
                    let e = normal.sample(&mut rng);
                    QuasiIdentifierTypes::Interval((
                        QuasiIdentifierType::Integer(truncate_to_domain(
                            e as i32, min_val, max_val,
                        )),
                        QuasiIdentifierType::Integer(min_val),
                        QuasiIdentifierType::Integer(max_val),
                        weight,
                    ))
                }
                _ => panic!("Wrong combination of type found in randomization of interval"),
            },
            QuasiIdentifierTypes::Ordinal((_, max_rank, weight)) => {
                let between = Uniform::<i32>::from(0..max_rank + 1);
                let random_ordinal_qi = between.sample(&mut rng);
                QuasiIdentifierTypes::Ordinal((random_ordinal_qi as i32, max_rank, weight))
            }
            QuasiIdentifierTypes::Nominal((_, max_value, weight)) => {
                let between = Uniform::<i32>::from(0..max_value + 1);
                let random_nominal_qi = between.sample(&mut rng);
                QuasiIdentifierTypes::Nominal((random_nominal_qi, max_value, weight))
            }
        }
    }
}

/// The role of this trait is to create a generic way of making sure that the struct can be anonymized
/// using the Anonymizer
pub trait Anonymizable: Default + Clone + Serialize + Sync {
    /// compare 2 data points and return the euclidean difference between them
    fn calculate_difference(&self, other: &Self) -> f64 {
        let mut sum_weight: usize = 0;
        let diff: f64 = self
            .quasi_identifiers()
            .into_iter()
            .zip(other.quasi_identifiers().into_iter())
            .map(|(x, y)| match (x, y) {
                (
                    QuasiIdentifierTypes::Interval(interval_x),
                    QuasiIdentifierTypes::Interval(interval_y),
                ) => {
                    let (_, _, _, weight) = interval_x;
                    sum_weight += weight;
                    Self::calculate_interval_distance(interval_x, interval_y)
                }
                (
                    QuasiIdentifierTypes::Ordinal(ordinal_x),
                    QuasiIdentifierTypes::Ordinal(ordinal_y),
                ) => {
                    let (_, _, weight) = ordinal_x;
                    sum_weight += weight;
                    Self::calculate_ordinal_distance(ordinal_x, ordinal_y)
                }
                (
                    QuasiIdentifierTypes::Nominal(nominal_x),
                    QuasiIdentifierTypes::Nominal(nominal_y),
                ) => {
                    let (_, _, weight) = nominal_x;
                    sum_weight += weight;
                    Self::calculate_nominal_distance(nominal_x, nominal_y)
                }
                _ => {
                    panic!("wrong types provided")
                }
            })
            .sum();

        diff / sum_weight as f64
    }

    /// calculate the info loss between 2 different Anonymizable
    /// structs
    fn calculate_info_loss(&self, other: &Self) -> f64 {
        let mut distance = 0.0;
        let self_qi = self.quasi_identifiers();
        let other_qi = other.quasi_identifiers();

        self_qi
            .into_iter()
            .zip(other_qi.into_iter())
            .for_each(|(x, y)| match (x.extract_value(), y.extract_value()) {
                (QuasiIdentifierType::Integer(value1), QuasiIdentifierType::Integer(value2)) => {
                    distance += (value1 as f64 - value2 as f64).powi(2)
                }
                (QuasiIdentifierType::Float(value1), QuasiIdentifierType::Float(value2)) => {
                    distance += (value1 - value2).powi(2)
                }
                _ => {
                    panic!("Incompatible values have been found")
                }
            });

        distance.sqrt()
    }

    /// return the values of the quasi identifiers in the data struct
    fn quasi_identifiers(&self) -> Vec<QuasiIdentifierTypes>;

    /// return a copy of the Anonymizable struct and replace its
    /// quasi identifier attributes with given QI's
    /// we return a copy because we want to keep the original intact for new aggregation
    fn update_quasi_identifiers(&self, qi: Vec<QuasiIdentifierTypes>) -> Self;

    /// return a copy of the sensitive attribute of the struct
    fn sensitive_value(&self) -> SensitiveAttribute;

    /// extract all the values in string format to be used for creating CSV
    fn extract_string_values(&self, uuid: Uuid, dr: f64) -> Vec<String>;

    // get the timestamp that the tuple has entered the algorithm
    fn get_timestamp(&self) -> SystemTime;

    /// suppress the qi's based on a buffer of Anonymizables
    fn suppress(&self) -> Self {
        let suppressed_qi = self
            .quasi_identifiers()
            .into_iter()
            .map(|x| x.randomize())
            .collect();

        self.update_quasi_identifiers(suppressed_qi)
    }

    /// calculate the euclidean distance between 2 ordinal data category types
    /// TODO: clarify that ranking starts at 1
    fn calculate_ordinal_distance(ordinal_x: OrdinalType, ordinal_y: OrdinalType) -> f64 {
        let (rank1, max_rank, weight) = ordinal_x;
        let (rank2, _, _) = ordinal_y;

        let x = (rank1 as f64 - 1.0) / (max_rank as f64 - 1.0);
        let y = (rank2 as f64 - 1.0) / (max_rank as f64 - 1.0);

        (weight as f64)
            * Self::calculate_interval_distance(
                (
                    QuasiIdentifierType::Float(x),
                    QuasiIdentifierType::Float(1.0),
                    QuasiIdentifierType::Float(max_rank as f64),
                    weight,
                ),
                (
                    QuasiIdentifierType::Float(y),
                    QuasiIdentifierType::Float(1.0),
                    QuasiIdentifierType::Float(max_rank as f64),
                    weight,
                ),
            )
    }

    /// calculate the euclidean distance between 2 interval data types
    fn calculate_interval_distance(interval_x: IntervalType, interval_y: IntervalType) -> f64 {
        let (num1, min, max, weight) = interval_x;
        let (num2, _, _, _) = interval_y;

        match (num1, min, max, num2) {
            (
                QuasiIdentifierType::Float(x),
                QuasiIdentifierType::Float(min),
                QuasiIdentifierType::Float(max),
                QuasiIdentifierType::Float(y),
            ) => weight as f64 * abs(x - y) / (max - min),
            (
                QuasiIdentifierType::Integer(x),
                QuasiIdentifierType::Integer(min),
                QuasiIdentifierType::Integer(max),
                QuasiIdentifierType::Integer(y),
            ) => weight as f64 * abs(x as f64 - y as f64) / (max as f64 - min as f64),
            _ => {
                panic!("wrong type conversion")
            }
        }
    }

    /// calculate the euclidean distance between 2 nominal data types
    fn calculate_nominal_distance(nominal_x: NominalType, nominal_y: NominalType) -> f64 {
        let (x, _, weight) = nominal_x;
        let (y, _, _) = nominal_y;

        match x == y {
            true => 0.0,
            false => weight as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data_manipulation::aggregation::AggregateType;
    use crate::data_manipulation::anonymizable::Anonymizable;
    use crate::data_manipulation::anonymizable::QuasiIdentifierType::{Float, Integer};
    use crate::data_manipulation::anonymizable::QuasiIdentifierTypes::{
        Interval, Nominal, Ordinal,
    };
    use crate::data_manipulation::mueller::MuellerStream;

    #[test]
    fn get_quasi_identifiers() {
        let mueller = MuellerStream {
            age: Some(32),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        let mut quasi_identifiers = mueller.quasi_identifiers();

        match quasi_identifiers.remove(0) {
            Interval((Integer(32), Integer(33), Integer(85), 1)) => {}
            _ => {
                panic!()
            }
        }

        match quasi_identifiers.remove(0) {
            Nominal((0, 1, 1)) => {}
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn update_quasi_identifiers() {
        let mueller = MuellerStream {
            age: Some(32),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        let centroid = MuellerStream {
            age: Some(50),
            gender: Some("female".to_string()),
            ..MuellerStream::default()
        };

        let anonymized = mueller.update_quasi_identifiers(centroid.quasi_identifiers());

        assert_eq!(anonymized.age, Some(50));
        assert_eq!(anonymized.gender, Some("female".to_string()))
    }

    #[test]
    fn calculate_difference() {
        let mueller = MuellerStream {
            age: Some(37),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        let centroid = MuellerStream {
            age: Some(50),
            gender: Some("female".to_string()),
            ..MuellerStream::default()
        };

        let difference = mueller.calculate_difference(&centroid);

        assert_eq!(difference, 0.625)
    }

    #[test]
    fn calculate_difference_zero() {
        let mueller = MuellerStream {
            age: Some(37),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        let centroid = MuellerStream {
            age: Some(37),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        let difference = mueller.calculate_difference(&centroid);

        assert_eq!(difference, 0.0)
    }

    #[test]
    fn calculate_difference_one() {
        let mueller = MuellerStream {
            age: Some(33),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        let centroid = MuellerStream {
            age: Some(85),
            gender: Some("female".to_string()),
            ..MuellerStream::default()
        };

        let difference = mueller.calculate_difference(&centroid);

        assert_eq!(difference, 1.0)
    }

    #[test]
    fn calculate_info_loss() {
        let mueller = MuellerStream {
            age: Some(33),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        let centroid = MuellerStream {
            age: Some(50),
            gender: Some("female".to_string()),
            ..MuellerStream::default()
        };

        let info_loss = mueller.calculate_info_loss(&centroid);
        assert!((info_loss - 17.29) <= f64::EPSILON)
    }

    #[test]
    fn aggregation_interval_integer() {
        let agg1 = Interval((Integer(1), Integer(0), Integer(10), 1));
        let agg2 = Interval((Integer(4), Integer(0), Integer(10), 1));
        let agg3 = Interval((Integer(6), Integer(0), Integer(10), 1));
        let agg4 = Interval((Integer(10), Integer(0), Integer(10), 1));

        let aggregation = AggregateType::Mean(vec![agg1, agg2, agg3, agg4]).aggregate();

        if let Integer(value) = aggregation.extract_value() {
            assert_eq!(value, 5)
        } else {
            panic!()
        }
    }

    #[test]
    fn aggregation_interval_float() {
        let agg1 = Interval((Float(1.0), Float(0.0), Float(10.0), 1));
        let agg2 = Interval((Float(4.0), Float(0.0), Float(10.0), 1));
        let agg3 = Interval((Float(6.0), Float(0.0), Float(10.0), 1));
        let agg4 = Interval((Float(10.0), Float(0.0), Float(10.0), 1));

        let aggregation = AggregateType::Mean(vec![agg1, agg2, agg3, agg4]).aggregate();

        if let Float(value) = aggregation.extract_value() {
            assert_eq!(value, 5.25)
        } else {
            panic!()
        }
    }

    #[test]
    fn aggregation_ordinal() {
        let agg1 = Ordinal((1, 10, 1));
        let agg2 = Ordinal((1, 10, 1));
        let agg3 = Ordinal((2, 10, 1));
        let agg4 = Ordinal((4, 10, 1));

        let aggregation = AggregateType::Mode(vec![agg1, agg2, agg3, agg4]).aggregate();

        if let Integer(value) = aggregation.extract_value() {
            assert_eq!(value, 1)
        } else {
            panic!()
        }
    }

    #[test]
    fn aggregation_nominal() {
        let agg1 = Nominal((1, 4, 10));
        let agg2 = Nominal((1, 4, 10));
        let agg3 = Nominal((2, 4, 10));
        let agg4 = Nominal((4, 4, 10));

        let aggregation = AggregateType::Mode(vec![agg1, agg2, agg3, agg4]).aggregate();

        if let Integer(value) = aggregation.extract_value() {
            assert_eq!(value, 1)
        } else {
            panic!()
        }
    }
}
