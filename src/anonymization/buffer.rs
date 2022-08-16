use crate::data_manipulation::aggregation::AggregateType;
use crate::data_manipulation::aggregation::AggregateType::{Mean, Mode};
use crate::data_manipulation::anonymizable::{Anonymizable, QuasiIdentifierTypes};
use std::collections::VecDeque;

/// used to keep note which data tuple has already been published or not
pub type DataContainer<M> = (bool, M);

/// Buffer used for containing individual data tuples and generalized centroid.
/// Also used for checking concept drift in a cluster by comparing 2 buffers with
/// each other.
#[derive(Clone)]
pub struct Buffer<M>
where
    M: Anonymizable,
{
    pub max_buffer_size: usize,
    pub centroid: M,
    pub buffer: VecDeque<DataContainer<M>>,
}

impl<M> Buffer<M>
where
    M: Anonymizable,
{
    /// add data tuple, updated centroid and return copy of
    /// centroid
    pub fn add_tuple(&mut self, value: M) -> M {
        self.buffer.push_back((false, value));

        self.update_centroid();

        self.centroid.clone()
    }

    /// aggregate all the QI's in the buffer
    /// into a centroid
    /// TODO: maybe use parallel iterator at some point?
    pub fn update_centroid(&mut self) {
        let mut qi_list: Vec<AggregateType> = Vec::new();
        self.buffer.iter().for_each(|(_, x)| {
            x.quasi_identifiers()
                .into_iter()
                .enumerate()
                .for_each(|(index, qi)| match qi_list.get_mut(index) {
                    None => {
                        let aggregate_type = match qi {
                            QuasiIdentifierTypes::Interval(qi) => {
                                Mean(vec![QuasiIdentifierTypes::Interval(qi)])
                            }
                            QuasiIdentifierTypes::Ordinal(qi) => {
                                Mode(vec![QuasiIdentifierTypes::Ordinal(qi)])
                            }
                            QuasiIdentifierTypes::Nominal(qi) => {
                                Mode(vec![QuasiIdentifierTypes::Nominal(qi)])
                            }
                        };
                        qi_list.insert(index, aggregate_type)
                    }
                    Some(aggregate_type) => match aggregate_type {
                        Mean(list) => list.push(qi),
                        Mode(list) => list.push(qi),
                    },
                })
        });

        let new_qi: Vec<QuasiIdentifierTypes> =
            qi_list.into_iter().map(|x| x.aggregate()).collect();

        self.centroid = self.centroid.update_quasi_identifiers(new_qi);
    }

    /// empty buffer after null hypothesis
    pub fn reset(&mut self) {
        self.buffer = VecDeque::new();
    }

    /// used for checking if the buffer needs to be emptied
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= self.max_buffer_size
    }
}

impl<M> Default for Buffer<M>
where
    M: Anonymizable,
{
    fn default() -> Self {
        Self {
            max_buffer_size: 0,
            centroid: Default::default(),
            buffer: VecDeque::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::anonymization::buffer::Buffer;
    use crate::data_manipulation::mueller::MuellerStream;

    #[test]
    fn update_centroid() {
        let mueller1 = MuellerStream {
            age: Some(30),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        let mueller2 = MuellerStream {
            age: Some(40),
            gender: Some("female".to_string()),
            ..MuellerStream::default()
        };

        let mueller3 = MuellerStream {
            age: Some(50),
            gender: Some("female".to_string()),
            ..MuellerStream::default()
        };

        let centroid = MuellerStream {
            age: Some(40),
            gender: Some("female".to_string()),
            ..MuellerStream::default()
        };

        let mut buffer: Buffer<MuellerStream> = Buffer::default();
        buffer.add_tuple(mueller1);
        buffer.add_tuple(mueller2);
        buffer.add_tuple(mueller3);

        assert_eq!(buffer.centroid, centroid)
    }

    #[test]
    fn is_full() {
        let mut buffer: Buffer<MuellerStream> = Buffer {
            max_buffer_size: 3,
            ..Buffer::default()
        };

        let mueller1 = MuellerStream {
            age: Some(30),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        let mueller2 = MuellerStream {
            age: Some(40),
            gender: Some("female".to_string()),
            ..MuellerStream::default()
        };

        let mueller3 = MuellerStream {
            age: Some(50),
            gender: Some("female".to_string()),
            ..MuellerStream::default()
        };

        buffer.add_tuple(mueller1);
        buffer.add_tuple(mueller2);
        buffer.add_tuple(mueller3);

        assert!(buffer.is_full())
    }
}
