# DiffPriv
DiffPriv is a differential privacy framework for real time data streaming written in Rust. Supporting k-anonymity,
(c,l)-diversity and ε-differential privacy. The framework is based on the [Preserving Differential Privacy and Utility of Non-stationary Data Streams](https://ieeexplore.ieee.org/document/8637412) paper, with various improvements implemented.

This project is the result of my master thesis: Differential privacy in large scale data streaming.
It has been developer during an intership at [STRM Privacy](https://strmprivacy.io/)
## How to run
it's recommended to first build the application using as it will significantly speed up the algorithm
> cargo build --release

An application.conf needs to be present in the root folder.
this will build a binary that can be run with the following command

> RUST_LOG="debug" ./target/release/diff-priv

This will use a dataset from the `datasets` folder, the supported datasets can be seen in `test/tests.rs`
`RUST_LOG` part can be removed to the users liking. This removes debugging logging when the algorithm will run.

## Where is the data exported
When `main.rs` is run, the processed datasets can be seen in the `exports` directory.

## Application parameters
Inside the `application.conf` all the different privacy parameters can be edited to the users liking.
At this moment for `buffer_size` we use `3*k` and for `k_max` we use `4*k`. This can be edited in the `environment.rs` and `tests.rs` file.
Additional parameters can be easily added through the `config.rs` file by adding it as a struct attribute and then adding it to `application.conf`.

## Implementing `Anonymizable` trait to anonymize new data
By implementing the `Anonymizable` trait on any type of datastructure, DiffPriv will know how to anonymize it.
The following QIs types are implemented
```
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
```
An example implementation can be seen below

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use bimap::BiMap;
use lazy_static::lazy_static;
use uuid::Uuid;

use diff_priv::data_manipulation::anonymizable::{
    Anonymizable, QuasiIdentifierType, QuasiIdentifierTypes, SensitiveAttribute,
};

lazy_static! {
    static ref CLASS_BIMAP: BiMap<&'static str, i32> =
        BiMap::from_iter(vec![("<=50K", 0), (">50K", 1),]);
}

// This is the datastructure that we are going to anonymize
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Adult {
    timestamp: i32,
    age: i32,
    capital_gain: i32,
    capital_loss: i32,
    class: String,
    #[serde(skip_deserializing, default = "default_time")]
    time_generated: SystemTime,
}

fn default_time() -> SystemTime {
    SystemTime::now()
}

impl Default for Adult {
    fn default() -> Self {
        Self {
            timestamp: 0,
            age: 0,
            capital_gain: 0,
            capital_loss: 0,
            class: "".to_string(),
            time_generated: SystemTime::now(),
        }
    }
}

impl Adult {
    // here we extract an Interval QI from the `age` attribute
    fn get_age_qi(&self) -> QuasiIdentifierTypes {
        QuasiIdentifierTypes::Interval((
            QuasiIdentifierType::Integer(self.age),
            QuasiIdentifierType::Integer(1),
            QuasiIdentifierType::Integer(100),
            1,
        ))
    }

    // here we extract an Interval QI from the `capital_gain` attribute
    fn get_capital_gain_qi(&self) -> QuasiIdentifierTypes {
        QuasiIdentifierTypes::Interval((
            QuasiIdentifierType::Integer(self.capital_gain),
            QuasiIdentifierType::Integer(0),
            QuasiIdentifierType::Integer(100000),
            1,
        ))
    }

    // here we extract an Interval QI from the `capital_loss` attribute
    fn get_capital_loss_qi(&self) -> QuasiIdentifierTypes {
        QuasiIdentifierTypes::Interval((
            QuasiIdentifierType::Integer(self.capital_loss),
            QuasiIdentifierType::Integer(0),
            QuasiIdentifierType::Integer(5000),
            1,
        ))
    }

}

// Here we implement the `Anonymizable` trait
impl Anonymizable for Adult {
    // We extract the QIs from the datastructure and return a `vec` of QIs
    fn quasi_identifiers(&self) -> Vec<QuasiIdentifierTypes> {
        let age = self.get_age_qi();
        let capital_gain = self.get_capital_gain_qi();
        let capital_loss = self.get_capital_loss_qi();

        vec![
            age,
            capital_gain,
            capital_loss,
        ]
    }

    // We update the datastructures QIs with a `vec` of QIs. The `vec` needs to be
    // popped in the same order that the QIs are extracted with the `quasi_identifiers`
    // function
    fn update_quasi_identifiers(&self, mut qi: Vec<QuasiIdentifierTypes>) -> Self {
        if let (
            QuasiIdentifierType::Integer(capital_loss),
            QuasiIdentifierType::Integer(capital_gain),
            QuasiIdentifierType::Integer(age),
        ) = (
            qi.pop().unwrap().extract_value(),
            qi.pop().unwrap().extract_value(),
            qi.pop().unwrap().extract_value(),
        ) {
            Self {
                timestamp: self.timestamp,
                age,
                capital_gain,
                capital_loss,
                class: self.class.to_owned(),
                time_generated: self.time_generated,
            }
        } else {
            panic!("Couldn't Adult with QI's")
        }
    }

    // We extract the sensative attribute from the datastructure
    fn sensitive_value(&self) -> SensitiveAttribute {
        SensitiveAttribute::String(self.class.to_owned())
    }

    // We return a vector of strings containing the String version of the QIs
    // Used for printing to CSVs
    fn extract_string_values(&self, uuid: Uuid, dr: f64) -> Vec<String> {
        vec![
            uuid.to_string(),
            dr.to_string(),
            self.timestamp.to_string(),
            self.age.to_string(),
            self.capital_gain.to_string(),
            self.capital_loss.to_string(),
            self.class.to_owned(),
        ]
    }

    fn get_timestamp(&self) -> SystemTime {
        self.time_generated
    }
}
```


## Architecture
The architecture of the DiffPriv framework can be seen below
![Alt text](midipsa_1.png?raw=true "Title")

License:

MIT License

Copyright (c) 2022 Maciek Mika

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
