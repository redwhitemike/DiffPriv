use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use itertools::Itertools;
use uuid::Uuid;

use crate::analysis::analyser::Analyser;
use crate::anonymization::buffer::Buffer;
use crate::data_manipulation::anonymizable::{
    Anonymizable, QuasiIdentifierType, QuasiIdentifierTypes, SensitiveAttribute,
};
use crate::noise::noiser::Noiser;
use crate::publishing::publisher::Publisher;

/// Critical Values of Two-Sample Kolmogorov-Smirnorv Test Statistic
/// source: http://oak.ucc.nau.edu/rh83/Statistics/ks2/
const KS_CONFIDENDE_TABLE: &[f64] = &[
    1.22, // confidence 0.10
    1.36, // confidence 0.05
    1.48, // confidence 0.025
    1.63, // confidence 0.01
    1.73, // confidence 0.005
    1.95, // confidence 0.001
];
const KS_CRITICAL_VALUE: f64 = KS_CONFIDENDE_TABLE[0];

pub struct Cluster<A, N>
where
    A: Anonymizable,
    N: Noiser,
{
    pub uuid: Uuid,
    pub k: usize,             // desired k level of cluster
    pub l: usize,             // desired l diversity level of cluster
    pub c: i32,               // desired recursive (l,c)-diversity
    pub centroid: A,          // centroid of the cluster
    pub w_current: Buffer<A>, // current buffer of cluster
    pub w_prev: Buffer<A>,    // previous current buffer of cluster
    pub exit_time: f64,       // used to check what the cluster activity is
    pub sse: f64,             // sum of squared error of the cluster
    pub categorical_freq: HashMap<usize, HashMap<i32, i32>>, // used for checking categorical frequency for l-diversity
    pub sensitive_freq: HashMap<SensitiveAttribute, i32>, // used for checking sensitive attribute frequency for l-diversity
    pub complete_buffer_amount: usize, // the count of all added tuples to the cluster, used for max_k calculations
    pub last_arrival: u128,            // last arrival of tuple into the cluster
    pub noiser: N,
}

impl<A, N> Cluster<A, N>
where
    A: Anonymizable,
    N: Noiser,
{
    pub fn new(k: usize, l: usize, c: i32, max_buffer_size: usize, noiser: N) -> Self {
        Self {
            k,
            l,
            c,
            w_current: Buffer {
                max_buffer_size,
                ..Default::default()
            },
            w_prev: Buffer {
                max_buffer_size,
                ..Default::default()
            },
            noiser,
            ..Self::default()
        }
    }

    // add tuple to cluster
    // 4 possible outcomes
    // 1. update inner state with tuple
    // 2. update inner state and publishing tuple(s)
    // 3. concept drift detected create new cluster
    // 4. close cluster if |C| > KMax
    /// add tuple to cluster, replace w_prev with w_current
    /// and update centroid
    pub fn add_tuple(&mut self, value: A) {
        self.update_frequencies(&value);
        let temp_buffer = self.w_current.clone();
        let new_centroid = self.w_current.add_tuple(value);
        self.centroid = new_centroid;
        self.w_prev = temp_buffer;
        self.complete_buffer_amount += 1;
        self.last_arrival = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    /// update all the hashmaps containing frequencies of values
    fn update_frequencies(&mut self, value: &A) {
        self.update_categorical_frequency(value);
        self.update_sensitive_frequency(value);
    }

    /// check if the cluster satisfies (c,l)-diversity
    /// return false or true depending on if the most common value appears to often
    /// or if the least common value appears to infrequent
    fn check_l_recursive_diversity(&self) -> bool {
        match self.k <= self.w_current.buffer.len() {
            true => {
                let sorted_sensitive_frequency: Vec<i32> =
                    self.sensitive_freq.values().copied().sorted().collect();
                let r1 = sorted_sensitive_frequency[0];

                let rl_to_rm: i32 = sorted_sensitive_frequency[1..].iter().sum();

                r1 < self.c * rl_to_rm
            }
            false => false,
        }
    }

    /// publish last added data tuple to the publisher and update published status
    pub fn publish<P: Publisher>(&mut self, publisher: &mut P, analysers: &mut [Analyser<A>]) {
        // we can use unwrap here because there is always a value in the cluster when
        // data in a cluster is published
        let (_, original) = self.w_current.buffer.back().cloned().unwrap();

        self.publish_data(&original, publisher, analysers);
        let (published, _) = self.w_current.buffer.back_mut().unwrap();
        *published = true
    }

    /// publishing all the tuples in the buffer that have still not been published
    pub fn publish_all<P: Publisher>(&mut self, publisher: &mut P, analysers: &mut [Analyser<A>]) {
        let publish: Vec<A> = self
            .w_current
            .buffer
            .iter_mut()
            .filter_map(|(published, original)| match published {
                true => None,
                false => {
                    *published = true;
                    Some(original.clone())
                }
            })
            .collect();

        publish
            .into_iter()
            .for_each(|original| self.publish_data(&original, publisher, analysers))
    }

    /// publish a given data tuple looking at (c,l)-diversity
    fn publish_data<P: Publisher>(
        &mut self,
        value: &A,
        publisher: &mut P,
        analysers: &mut [Analyser<A>],
    ) {
        let publish = match self.check_l_recursive_diversity() {
            true => {
                debug!("l-diversity met");
                let centroid_qi = self.noiser.add_noise(&self.centroid);
                value.update_quasi_identifiers(centroid_qi)
            }
            false => {
                debug!("l-diversity not met, suppressing data");
                value.suppress()
            }
        };

        let mut dr = 0.0;
        analysers.iter_mut().for_each(|analyser| match analyser {
            Analyser::Mse(mse) => {
                let error = value.calculate_difference(&publish);
                mse.add_error(error)
            }
            Analyser::Sse(sse) => {
                let error = value.calculate_info_loss(&publish);
                sse.add_info_loss(error)
            }
            Analyser::PublishingDelay(publishing_delay) => publishing_delay.add_delay(value),
            Analyser::DisclosureRiskAnalyser(disclosure_risk) => {
                disclosure_risk.add_data(value.clone(), &publish);
                dr = disclosure_risk.current_linkage_probability;
            }
            _ => {}
        });
        publisher.publish(publish, self.uuid, dr)
    }

    /// calculate the duration since last arrival
    pub fn check_cluster_life_time(&self) -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            - self.last_arrival
    }

    /// update the frequency of sensitive values that have been added to the cluster
    fn update_sensitive_frequency(&mut self, value: &A) {
        let sensitive_attribute = value.sensitive_value();

        match self.sensitive_freq.get_mut(&sensitive_attribute) {
            None => {
                self.sensitive_freq.insert(sensitive_attribute, 1);
            }
            Some(sensitive_counter) => *sensitive_counter += 1,
        }
    }

    /// keep up to date which and how many times categorical data has been present
    /// iniside the cluster
    fn update_categorical_frequency(&mut self, value: &A) {
        let qi_list = value.quasi_identifiers();
        qi_list
            .into_iter()
            .enumerate()
            .for_each(|(index, qi)| match qi {
                QuasiIdentifierTypes::Interval(_) => {}
                QuasiIdentifierTypes::Ordinal((value, _, _)) => {
                    self.update_categorical_map_frequency(index, value)
                }
                QuasiIdentifierTypes::Nominal((value, _, _)) => {
                    self.update_categorical_map_frequency(index, value)
                }
            });
    }

    fn update_categorical_map_frequency(&mut self, index: usize, value: i32) {
        match self.categorical_freq.get_mut(&index) {
            None => {
                let mut cat_freq: HashMap<i32, i32> = HashMap::new();
                cat_freq.insert(value, 1);
                self.categorical_freq.insert(index, cat_freq);
            }
            Some(cat_freq) => match cat_freq.get_mut(&value) {
                None => {
                    cat_freq.insert(value, 1);
                }
                Some(occurences) => *occurences += 1,
            },
        }
    }

    /// detect if the cluster is experiencing concept drift after
    /// the max buffer size has been achieved
    pub fn detect_concept_drift(&mut self) {
        // if the categorical frequency hashmap is empty only numerical QI's are present
        // use the k-s test if that is the case, otherwise calculate the difference between
        // centroids of w_curr and w_prev
        let rejected = match self.categorical_freq.is_empty() {
            true => {
                let w_curr_qi: Vec<f64> = Self::flatten_buffer_qi(&self.w_current);
                let w_prev_qi: Vec<f64> = Self::flatten_buffer_qi(&self.w_prev);

                let result = kolmogorov_smirnov::test_f64(
                    &w_prev_qi,
                    &w_curr_qi,
                    self.calculate_threshold(),
                );
                result.is_rejected
            }
            false => {
                // check if the difference is > then 1.0 - confidence threshold
                self.w_current
                    .centroid
                    .calculate_difference(&self.w_prev.centroid)
                    > self.calculate_threshold()
            }
        };

        // if concept drift is detected reset the current centroid to the previous buffer
        if rejected {
            self.w_current.centroid = self.w_prev.centroid.clone();
            self.centroid = self.w_current.centroid.clone();
        }

        self.w_current.reset()
    }

    /// calculate the needed threshold for KS test for mixed types data tuples
    /// From: D. Reis et. al., "Fast Unsupervised Online Drift Detection Using Incremental Kolmogorov-Smirnov Test", 2016
    fn calculate_threshold(&self) -> f64 {
        let curr_buff_size = self.w_current.buffer.len() as f64;
        let prev_buff_size = self.w_prev.buffer.len() as f64;
        KS_CRITICAL_VALUE
            * ((curr_buff_size + prev_buff_size) / (curr_buff_size * prev_buff_size)).sqrt()
    }

    /// return the qi values that are present inside the buffer
    /// to be used in the Kolmogorov Smirnov test
    fn flatten_buffer_qi(buffer: &Buffer<A>) -> Vec<f64> {
        buffer
            .buffer
            .iter()
            .flat_map(|(_, buffer)| {
                buffer
                    .quasi_identifiers()
                    .into_iter()
                    .map(|qi| match qi.extract_value() {
                        QuasiIdentifierType::Float(value) => value,
                        QuasiIdentifierType::Integer(value) => value as f64,
                    })
            })
            .collect()
    }

    pub fn is_full(&self) -> bool {
        self.w_current.is_full()
    }

    pub fn print_domain_qis(&self) -> Vec<(QuasiIdentifierType, QuasiIdentifierType)> {
        let mut qi_list: Vec<Vec<QuasiIdentifierTypes>> = Vec::new();
        self.w_current.buffer.iter().for_each(|(_, x)| {
            x.quasi_identifiers()
                .into_iter()
                .enumerate()
                .for_each(|(index, qi)| match qi_list.get_mut(index) {
                    None => {
                        qi_list.insert(index, vec![qi]);
                    }
                    Some(list) => list.push(qi),
                })
        });

        qi_list
            .into_iter()
            .map(|x| {
                let domain = x
                    .into_iter()
                    .map(|x| x.extract_value())
                    .collect::<Vec<QuasiIdentifierType>>();

                let max = domain
                    .iter()
                    .copied()
                    .max_by(|unit1, unit2| match (unit1, unit2) {
                        (QuasiIdentifierType::Integer(a), QuasiIdentifierType::Integer(b)) => {
                            a.cmp(b)
                        }
                        (QuasiIdentifierType::Float(a), QuasiIdentifierType::Float(b)) => {
                            a.partial_cmp(b).unwrap()
                        }
                        _ => panic!("wrong type"),
                    })
                    .unwrap();

                let min = domain
                    .into_iter()
                    .min_by(|unit1, unit2| match (unit1, unit2) {
                        (QuasiIdentifierType::Integer(a), QuasiIdentifierType::Integer(b)) => {
                            a.cmp(b)
                        }
                        (QuasiIdentifierType::Float(a), QuasiIdentifierType::Float(b)) => {
                            a.partial_cmp(b).unwrap()
                        }
                        _ => panic!("wrong type"),
                    })
                    .unwrap();
                (min, max)
            })
            .collect()
    }
}

impl<A: Anonymizable, N: Noiser> Default for Cluster<A, N> {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            k: 0,
            l: 0,
            c: 0,
            centroid: Default::default(),
            w_current: Default::default(),
            w_prev: Default::default(),
            exit_time: 0.0,
            sse: 0.0,
            categorical_freq: Default::default(),
            sensitive_freq: Default::default(),
            complete_buffer_amount: 0,
            last_arrival: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            noiser: Default::default(),
        }
        // set exit_time to 0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::anonymization::buffer::{Buffer, DataContainer};
    use crate::anonymization::cluster::Cluster;
    use crate::data_manipulation::mueller::MuellerStream;
    use crate::noise::laplace::laplace_noiser::LaplaceNoiser;

    fn create_test_buffer(qi_list: Vec<(i32, &str)>) -> VecDeque<DataContainer<MuellerStream>> {
        qi_list
            .into_iter()
            .map(|(age, gender)| {
                (
                    false,
                    MuellerStream {
                        age: Some(age),
                        gender: Some(gender.to_string()),
                        ..Default::default()
                    },
                )
            })
            .collect()
    }

    fn setup_cluster(
        w_current_qis: Vec<(i32, &str)>,
        w_prev_qis: Vec<(i32, &str)>,
        max_buffer_size: usize,
        mueller: MuellerStream,
    ) -> Cluster<MuellerStream, LaplaceNoiser> {
        let w_current_buffer = create_test_buffer(w_current_qis);

        let w_prev_buffer = create_test_buffer(w_prev_qis);

        let mut w_current: Buffer<MuellerStream> = Buffer {
            buffer: w_current_buffer,
            max_buffer_size,
            ..Default::default()
        };
        w_current.update_centroid();

        let mut w_prev: Buffer<MuellerStream> = Buffer {
            buffer: w_prev_buffer,
            max_buffer_size,
            ..Default::default()
        };
        w_prev.update_centroid();

        let mut cluster: Cluster<MuellerStream, LaplaceNoiser> = Cluster {
            centroid: w_current.centroid.clone(),
            w_current,
            w_prev,
            ..Default::default()
        };

        cluster.add_tuple(mueller);

        cluster
    }

    #[test]
    fn no_concept_drift() {
        let w_current_qis = [
            (46, "male"),
            (44, "male"),
            (45, "female"),
            (80, "male"),
            (55, "male"),
            (36, "male"),
            (70, "female"),
            (43, "female"),
        ];

        let w_prev_qis = [
            (44, "male"),
            (45, "female"),
            (80, "male"),
            (55, "male"),
            (36, "male"),
            (70, "female"),
            (43, "female"),
        ];

        let mueller = MuellerStream {
            age: Some(45),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        let mut cluster = setup_cluster(w_current_qis.to_vec(), w_prev_qis.to_vec(), 8, mueller);

        cluster.detect_concept_drift();

        assert_eq!(cluster.w_current.buffer.len(), 0);
        assert_eq!(cluster.centroid, cluster.w_current.centroid);
        assert_ne!(cluster.w_current.centroid, cluster.w_prev.centroid)
    }

    #[test]
    fn detected_concept_drift() {
        let w_current_qis = [
            (33, "male"),
            (33, "male"),
            (33, "female"),
            (33, "female"),
            (33, "male"),
            (33, "female"),
            (33, "male"),
            (33, "female"),
        ];

        let w_prev_qis = [
            (33, "male"),
            (33, "female"),
            (33, "female"),
            (33, "male"),
            (33, "female"),
            (33, "male"),
            (33, "female"),
        ];

        let mueller = MuellerStream {
            age: Some(85),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        let mut cluster = setup_cluster(w_current_qis.to_vec(), w_prev_qis.to_vec(), 8, mueller);

        // with the current QI's it's difficult to get a certain threshold to uncover concept drift so we
        // create a centroid manually
        cluster.w_current.centroid = MuellerStream {
            age: Some(55),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        cluster.detect_concept_drift();

        assert_eq!(cluster.w_current.buffer.len(), 0);
        assert_eq!(cluster.centroid, cluster.w_current.centroid);
        assert_eq!(cluster.w_current.centroid, cluster.w_prev.centroid)
    }

    #[test]
    fn update_categorical_frequency() {
        let noiser = LaplaceNoiser::new(0.1, 3, 0.1);
        let mut cluster: Cluster<MuellerStream, LaplaceNoiser> = Cluster::new(3, 3, 3, 10, noiser);

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

        cluster.add_tuple(mueller1);
        cluster.add_tuple(mueller2);
        cluster.add_tuple(mueller3);

        assert_eq!(
            *cluster.categorical_freq.get(&1).unwrap().get(&0).unwrap(),
            1
        );
        assert_eq!(
            *cluster.categorical_freq.get(&1).unwrap().get(&1).unwrap(),
            2
        )
    }

    #[test]
    fn calculate_ks_threshold_mixed_data_types() {
        let w_current_qis = [
            (33, "male"),
            (33, "male"),
            (33, "female"),
            (33, "female"),
            (33, "male"),
            (33, "female"),
            (33, "male"),
            (33, "female"),
        ];

        let w_prev_qis = [
            (33, "male"),
            (33, "female"),
            (33, "female"),
            (33, "male"),
            (33, "female"),
            (33, "male"),
            (33, "female"),
        ];

        let mueller = MuellerStream {
            age: Some(83),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        let cluster = setup_cluster(w_current_qis.to_vec(), w_prev_qis.to_vec(), 8, mueller);

        let critical_value = cluster.calculate_threshold();
        assert!((critical_value - 0.592_813_442_642_605_5) <= f64::EPSILON)
    }
}
