use crate::analysis::analyser::Analyser;
use crate::analysis::cluster_analyser::ClusterAnalyser;
use crate::analysis::disclosure_risk_analyser::DisclosureRiskAnalyser;
use crate::analysis::mse_analyser::MseAnalyser;
use crate::analysis::publishing_delay_analyser::PublishingDelayAnalyser;
use crate::analysis::sse_analyser::SseAnalyser;
use crate::anonymization::cluster::Cluster;
use crate::data_manipulation::anonymizable::{Anonymizable, QuasiIdentifierType};
use crate::noise::noiser::Noiser;
use crate::publishing::publisher::Publisher;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// the micro aggregation differential privacy anonymization
pub struct MicroaggAnonymizer<N, A, P>
where
    N: Noiser,
    A: Anonymizable,
    P: Publisher,
{
    k: usize,           // k-anonymity level
    k_max: usize,       // maximum k-anonymity level before cluster is removed
    l: usize,           // l-diversity level
    c: i32,             // recursive (l,c)-diversity
    delta: u128,        // life time delta in seconds
    diff_thres: f64,    // difference threshold between data points
    buffer_size: usize, // batch of data used to detect concept drift
    pub publisher: P,
    pub cluster_set: BTreeMap<u128, Cluster<A, N>>,
    pub noiser: N,
    pub analysers: Vec<Analyser<A>>,
}

#[allow(clippy::too_many_arguments)]
impl<N, A, P> MicroaggAnonymizer<N, A, P>
where
    N: Noiser,
    A: Anonymizable,
    P: Publisher,
{
    pub fn new(
        k: usize,
        k_max: usize,
        l: usize,
        c: i32,
        diff_thres: f64,
        delta: u128,
        buffer_size: usize,
        publisher: P,
        noiser: N,
    ) -> Self {
        let analysers = vec![
            Analyser::Mse(MseAnalyser::default()),
            Analyser::Sse(SseAnalyser::default()),
            Analyser::PublishingDelay(PublishingDelayAnalyser::default()),
            Analyser::ClusterAnalyser(ClusterAnalyser::default()),
            Analyser::DisclosureRiskAnalyser(DisclosureRiskAnalyser::initialize(100)),
        ];
        Self {
            k,
            k_max,
            l,
            c,
            diff_thres,
            delta: delta * 1000000000,
            buffer_size,
            publisher,
            cluster_set: Default::default(),
            noiser,
            analysers,
        }
    }

    /// feed the data tuple through the differential privacy algorithm
    pub fn anonymize(&mut self, value: A) {
        // Borrowing the right cluster caused multiple ownership problems as we borrow
        // self mutable and immutable.
        debug!("cluster count: {}", self.cluster_set.len());
        match self.find_best_cluster(&value) {
            // create new cluster
            None => {
                info!("new cluster created");
                let mut cluster = self.create_new_cluster();

                cluster.add_tuple(value);
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                self.cluster_set.insert(now.as_nanos(), cluster);
                self.analysers.iter_mut().for_each(|analyser| {
                    if let Analyser::ClusterAnalyser(cluster_analyser) = analyser {
                        cluster_analyser.add_count()
                    }
                })
            }
            Some(mut cluster) => {
                info!("cluster found");
                // check life time and change cluster
                if cluster.check_cluster_life_time() >= self.delta {
                    cluster.publish_all(&mut self.publisher, &mut self.analysers);
                    cluster = self.create_new_cluster();
                    info!("cluster life time delta exceeded")
                }

                cluster.add_tuple(value);

                // publishing value when k-anon level is met
                if cluster.w_current.buffer.len() == self.k {
                    cluster.publish_all(&mut self.publisher, &mut self.analysers);
                    info!("k-level is met published all")
                } else if self.k < cluster.w_current.buffer.len()
                    && cluster.w_current.buffer.len() <= self.k_max + 1
                {
                    // if the cluster contains at least k records
                    cluster.publish(&mut self.publisher, &mut self.analysers);
                    info!("publishing")
                }

                // check if the w_current is full (max size of buffer)
                // and reuse the buffer if concept drift is not detected
                if cluster.is_full() {
                    info!("cluster is full, checking concept drift");
                    cluster.detect_concept_drift()
                }

                // we removed the cluster in the find best cluster method
                // we need to insert it again
                match cluster.complete_buffer_amount > self.k_max {
                    true => {
                        cluster.publish_all(&mut self.publisher, &mut self.analysers);
                        info!("cluster is full removing..");
                        cluster.print_domain_qis().into_iter().enumerate().for_each(
                            |(index, domain)| match domain {
                                (
                                    QuasiIdentifierType::Integer(min),
                                    QuasiIdentifierType::Integer(max),
                                ) => {
                                    debug!("QI {}| min: {:?}| max: {:?}", index + 1, min, max)
                                }
                                (
                                    QuasiIdentifierType::Float(min),
                                    QuasiIdentifierType::Float(max),
                                ) => {
                                    debug!("QI {}| min: {:?}| max: {:?}", index + 1, min, max)
                                }
                                _ => panic!("wrong QI"),
                            },
                        );
                        // cluster has already been removed from cluster_set
                        // and it does not need to be added again
                        self.analysers.iter_mut().for_each(|analyser| {
                            if let Analyser::ClusterAnalyser(cluster_analyser) = analyser {
                                cluster_analyser.remove_count()
                            }
                        })
                    }
                    false => {
                        // add the cluster again at its arrival time into the cluster,
                        // if there already is another cluster there change the arrival time
                        // and try again
                        while self.cluster_set.contains_key(&cluster.last_arrival) {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_nanos();
                            cluster.last_arrival = now
                        }
                        self.cluster_set.insert(cluster.last_arrival, cluster);
                    }
                };
            }
        }
    }

    /// finding best cluster looking at the threshold
    /// Some -> use cluster for further building
    /// None -> create new cluster
    /// TODO: create a check here for old cluster so that the looping through the cluster_set
    /// is only done once, here we can improve massively on speed to use async to publishing the cluster set concurrently while
    /// looping further maybe?
    fn find_best_cluster(&mut self, value: &A) -> Option<Cluster<A, N>> {
        // remove the cluster from self and return it
        let mut best_cluster: Option<Cluster<A, N>> = None;
        let best_cluster_key: Mutex<Option<u128>> = Mutex::new(None);
        let least_info_loss: Mutex<Option<f64>> = Mutex::new(None);

        self.cluster_set.par_iter().for_each(|(key, a)| {
            if a.centroid.calculate_difference(value) <= self.diff_thres {
                let info_loss = value.calculate_info_loss(&a.centroid);
                let mut least_info = least_info_loss.lock().unwrap();
                match *least_info {
                    None => {
                        *least_info = Some(info_loss);
                        *best_cluster_key.lock().unwrap() = Some(*key)
                    }
                    Some(current_info_loss) => {
                        if info_loss < current_info_loss {
                            *least_info = Some(info_loss);
                            *best_cluster_key.lock().unwrap() = Some(*key)
                        }
                    }
                }
            }
        });

        // remove the key from the hashmap and set the best_cluster
        // variable
        if let Some(key) = *best_cluster_key.lock().unwrap() {
            best_cluster = self.cluster_set.remove(&key);
        }

        best_cluster
    }

    /// create new cluster
    fn create_new_cluster(&self) -> Cluster<A, N> {
        Cluster::new(
            self.k,
            self.l,
            self.c,
            self.buffer_size,
            self.noiser.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::MicroaggAnonymizer;
    use crate::data_manipulation::mueller::MuellerStream;
    use crate::noise::laplace::laplace_noiser::LaplaceNoiser;
    use crate::test::dummy_publisher::DummyPublisher;

    #[test]
    fn find_best_cluster() {
        let noiser = LaplaceNoiser::new(0.1, 3, 0.1);
        let publisher = DummyPublisher::default();
        let mut anonymizer: MicroaggAnonymizer<LaplaceNoiser, MuellerStream, DummyPublisher> =
            MicroaggAnonymizer::new(2, 10, 2, 2, 0.65, 10, 5, publisher, noiser);

        let mueller1 = MuellerStream {
            age: Some(30),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        assert!(anonymizer.find_best_cluster(&mueller1).is_none());

        anonymizer.anonymize(mueller1);

        assert_eq!(anonymizer.cluster_set.len(), 1);

        let mueller2 = MuellerStream {
            age: Some(30),
            gender: Some("male".to_string()),
            ..MuellerStream::default()
        };

        assert!(anonymizer.find_best_cluster(&mueller2).is_some());

        anonymizer.anonymize(mueller2);

        let mueller3 = MuellerStream {
            age: Some(50),
            gender: Some("female".to_string()),
            ..MuellerStream::default()
        };

        assert_eq!(anonymizer.cluster_set.len(), 1);

        anonymizer.anonymize(mueller3);

        assert_eq!(anonymizer.cluster_set.len(), 2)
    }
}
