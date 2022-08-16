use crate::data_manipulation::anonymizable::Anonymizable;
use std::cmp::Ordering;
use std::collections::VecDeque;

/// Analyses the risk of disclosure of the anonymized
/// tuple
pub struct DisclosureRiskAnalyser<A>
where
    A: Anonymizable,
{
    buffer_size: usize,
    pub current_linkage_probability: f64,
    sum_linkage_probability: f64,
    count: i32,
    buffer: VecDeque<A>,
}

impl<A> DisclosureRiskAnalyser<A>
where
    A: Anonymizable,
{
    pub fn initialize(buffer_size: usize) -> Self {
        Self {
            buffer_size,
            current_linkage_probability: 0.0,
            sum_linkage_probability: 0.0,
            count: 0,
            buffer: VecDeque::new(),
        }
    }

    pub fn add_data(&mut self, value: A, anonymized_value: &A) {
        if self.buffer.len() >= self.buffer_size {
            self.buffer.pop_back();
        }
        self.buffer.push_front(value);
        self.count += 1;
        self.update_estimation(anonymized_value)
    }

    fn update_estimation(&mut self, anonymized_value: &A) {
        let newest_tuple = self.buffer.front().unwrap();
        let mut minimum_distance = newest_tuple.calculate_difference(anonymized_value);
        let mut indices: Vec<usize> = vec![0];

        self.buffer
            .iter()
            .skip(1)
            .enumerate()
            .for_each(|(index, unit)| {
                let difference = unit.calculate_difference(anonymized_value);
                match difference.partial_cmp(&minimum_distance) {
                    Some(Ordering::Less) => {
                        minimum_distance = difference;
                        indices = vec![index]
                    }
                    Some(Ordering::Equal) => indices.push(index),
                    _ => {}
                }
            });

        match indices.contains(&0) {
            true => {
                self.sum_linkage_probability += 1.0 / indices.len() as f64;
                self.current_linkage_probability = self.sum_linkage_probability / self.count as f64
            }
            false => {}
        }
    }

    pub fn calculate_disclosure_risk(&self) -> f64 {
        self.sum_linkage_probability / self.count as f64
    }
}
