use crate::data_manipulation::anonymizable::{
    IntervalType, NominalType, OrdinalType, QuasiIdentifierType, QuasiIdentifierTypes,
};
use itertools::Itertools;

/// TODO: check adding MEDIAN
pub enum AggregateType {
    Mean(Vec<QuasiIdentifierTypes>),
    Mode(Vec<QuasiIdentifierTypes>),
}

impl AggregateType {
    /// the vector should contain only the same type of QI type
    pub fn aggregate(self) -> QuasiIdentifierTypes {
        match self {
            AggregateType::Mean(mut list) => match list.pop().unwrap() {
                QuasiIdentifierTypes::Interval(interval) => {
                    Self::aggregate_interval(interval, list)
                }
                _ => panic!("Wrong QI type found during aggregation for Mean"),
            },
            AggregateType::Mode(mut list) => {
                // we need to pop the first element one to
                // get the shared attributes between different elements in the QI list
                match list.pop().unwrap() {
                    QuasiIdentifierTypes::Ordinal(ordinal) => {
                        Self::aggregate_ordinal(ordinal, list)
                    }
                    QuasiIdentifierTypes::Nominal(nominal) => {
                        Self::aggregate_nominal(nominal, list)
                    }
                    _ => panic!("Wrong QI type for calculating mode"),
                }
            }
        }
    }

    /// aggregate interval QI type
    fn aggregate_interval(
        interval: IntervalType,
        list: Vec<QuasiIdentifierTypes>,
    ) -> QuasiIdentifierTypes {
        // we need to increment is by one as the first interval has already been popped
        let size = list.len() + 1;
        match interval {
            (
                QuasiIdentifierType::Float(value),
                QuasiIdentifierType::Float(min),
                QuasiIdentifierType::Float(max),
                weight,
            ) => {
                let sum: f64 = list
                    .into_iter()
                    .map(|x| match x.extract_value() {
                        QuasiIdentifierType::Float(temp) => temp,
                        _ => panic!("Wrong type found for Mean aggregation"),
                    })
                    .sum();

                QuasiIdentifierTypes::Interval((
                    QuasiIdentifierType::Float((value + sum) / size as f64),
                    QuasiIdentifierType::Float(min),
                    QuasiIdentifierType::Float(max),
                    weight,
                ))
            }
            (
                QuasiIdentifierType::Integer(value),
                QuasiIdentifierType::Integer(min),
                QuasiIdentifierType::Integer(max),
                weight,
            ) => {
                let sum: i32 = list
                    .into_iter()
                    .map(|x| match x.extract_value() {
                        QuasiIdentifierType::Integer(temp) => temp,
                        _ => panic!("Wrong type found for Mean aggregation"),
                    })
                    .sum();

                QuasiIdentifierTypes::Interval((
                    QuasiIdentifierType::Integer((value + sum) / size as i32),
                    QuasiIdentifierType::Integer(min),
                    QuasiIdentifierType::Integer(max),
                    weight,
                ))
            }
            _ => panic!("Wrong interval type set found during aggregation"),
        }
    }

    /// aggregate ordinal QI type
    fn aggregate_ordinal(
        ordinal: OrdinalType,
        list: Vec<QuasiIdentifierTypes>,
    ) -> QuasiIdentifierTypes {
        let (rank, max_rank, weight) = ordinal;
        let mut mode_list = Vec::new();
        list.into_iter().for_each(|x| match x {
            QuasiIdentifierTypes::Ordinal((temp, _, _)) => mode_list.push(temp),
            _ => panic!("Wrong QI type"),
        });

        mode_list.push(rank);

        let mode = Self::get_mode(mode_list);

        QuasiIdentifierTypes::Ordinal((mode, max_rank, weight))
    }

    /// aggregate nominal QI type
    fn aggregate_nominal(
        nominal: NominalType,
        list: Vec<QuasiIdentifierTypes>,
    ) -> QuasiIdentifierTypes {
        let (value, max_value, weight) = nominal;
        let mut mode_list = Vec::new();
        list.into_iter().for_each(|x| match x {
            QuasiIdentifierTypes::Nominal((temp, _, _)) => mode_list.push(temp),
            _ => panic!("Wrong QI type"),
        });

        mode_list.push(value);

        let mode = Self::get_mode(mode_list);

        QuasiIdentifierTypes::Nominal((mode, max_value, weight))
    }

    /// retrieve mode from list of i32
    fn get_mode(mode_list: Vec<i32>) -> i32 {
        let mut mode_grouped: Vec<(i32, Vec<i32>)> = Vec::new();
        for (key, group) in &mode_list.into_iter().sorted().group_by(|&x| x) {
            mode_grouped.push((key, group.collect()))
        }

        let (mode, _) = mode_grouped
            .into_iter()
            .map(|(key, group)| (key, group.len()))
            .max_by_key(|(_, group)| *group)
            .unwrap();

        mode
    }
}

/// check if the value that was randomly generated is contained within its domain
pub fn truncate_to_domain<T: PartialOrd>(value: T, min: T, max: T) -> T {
    match value {
        x if x <= min => min,
        x if x >= max => max,
        _ => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncating_to_domain_integer() {
        let min = 0;
        let value = 5;
        let max = 10;

        assert_eq!(truncate_to_domain(value, min, max), 5)
    }

    #[test]
    fn truncate_to_domain_integer_max() {
        let min = 0;
        let value = 11;
        let max = 10;

        assert_eq!(truncate_to_domain(value, min, max), 10)
    }

    #[test]
    fn truncate_to_domain_float() {
        let min = 0.0;
        let value = 5.0;
        let max = 10.0;

        assert!(truncate_to_domain(value, min, max) - 5.0 <= f64::EPSILON)
    }

    #[test]
    fn truncate_to_domain_float_max() {
        let min = 0.0;
        let value = 10.0;
        let max = 5.0;

        assert!(truncate_to_domain(value, min, max) - 5.0 <= f64::EPSILON)
    }
}
