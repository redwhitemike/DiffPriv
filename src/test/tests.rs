use crate::data_manipulation::anonymizable::Anonymizable;
use crate::test::adult::Adult;
use crate::test::adult_large::AdultLarge;
use crate::test::csv_importer::{CsvImporter, EnrichedRow};
use crate::test::environment::{Dataset, Datasets, Environment};

use crate::config::Config;
use csv::Reader;
use serde::de::DeserializeOwned;

/**
    This file contains the main test architecture to run all the different possible
    sets of parameters defined in the `application.conf`
**/

pub fn start_tests(conf_file: &String) {
    let config = Config::new(conf_file);
    let environments = prepare_environments(&config);
    environments.into_iter().for_each(|env| match env.dataset {
        Datasets::Adult(dataset) => create_test::<Adult>(env, dataset),
        Datasets::AdultLarge(dataset) => create_test::<AdultLarge>(env, dataset),
        Datasets::Mueller(dataset) => create_test::<EnrichedRow>(env, dataset),
    });
}

fn create_test<A: Anonymizable + DeserializeOwned>(env: Environment, dataset: Dataset) {
    println!("Reading file {}", dataset.path);
    let file = Reader::from_path(dataset.path).unwrap();
    let mut csv_importer = CsvImporter::new(file);
    match csv_importer.convert::<A>(env) {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e)
        }
    }
}

const DATASETS: [Datasets; 3] = [
    Datasets::Adult(Dataset {
        path: "datasets/Adult_1_numeric_only_class_50K.csv",
        export: "exports/adult",
    }),
    Datasets::AdultLarge(Dataset {
        path: "datasets/Adult_2_numerical_categorical_class_50K_drift.csv",
        export: "exports/adultLarge",
    }),
    Datasets::Mueller(Dataset {
        path: "datasets/steps-converted.csv",
        export: "exports/steps",
    }),
];

fn prepare_environments(config: &Config) -> Vec<Environment> {
    let mut environments: Vec<Environment> = vec![];
    DATASETS.iter().copied().for_each(|dataset| {
        config.k.iter().copied().for_each(|k| {
            config.l.iter().copied().for_each(|l| {
                config.c.iter().copied().for_each(|c| {
                    config.diff_thres.iter().copied().for_each(|diff_thres| {
                        config.eps.iter().copied().for_each(|eps| {
                            config.delta.iter().copied().for_each(|delta| {
                                config.noise_thr.iter().copied().for_each(|noise_thr| {
                                    environments.push(Environment::new(
                                        k,
                                        k * 4,
                                        l,
                                        c,
                                        diff_thres,
                                        eps,
                                        delta as u128,
                                        noise_thr,
                                        dataset,
                                        config.publish_remaining_tuples,
                                    ))
                                })
                            })
                        })
                    })
                })
            })
        })
    });

    environments
}
