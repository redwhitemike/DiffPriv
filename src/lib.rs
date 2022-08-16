//! ```
//! use std::time::{SystemTime, UNIX_EPOCH};
//! use serde::{Serialize, Deserialize};
//! use bimap::BiMap;
//! use lazy_static::lazy_static;
//! use uuid::Uuid;
//!
//! use diff_priv::data_manipulation::anonymizable::{
//!     Anonymizable, QuasiIdentifierType, QuasiIdentifierTypes, SensitiveAttribute,
//! };
//!
//! lazy_static! {
//!     static ref CLASS_BIMAP: BiMap<&'static str, i32> =
//!         BiMap::from_iter(vec![("<=50K", 0), (">50K", 1),]);
//! }
//!
//! // This is the datastructure that we are going to anonymize
//! #[derive(Debug, Serialize, Clone, Deserialize)]
//! pub struct Adult {
//!     timestamp: i32,
//!     age: i32,
//!     capital_gain: i32,
//!     capital_loss: i32,
//!     class: String,
//!     #[serde(skip_deserializing, default = "default_time")]
//!     time_generated: SystemTime,
//! }
//!
//! fn default_time() -> SystemTime {
//!     SystemTime::now()
//! }
//!
//! impl Default for Adult {
//!     fn default() -> Self {
//!         Self {
//!             timestamp: 0,
//!             age: 0,
//!             capital_gain: 0,
//!             capital_loss: 0,
//!             class: "".to_string(),
//!             time_generated: SystemTime::now(),
//!         }
//!     }
//! }
//!
//! impl Adult {
//!     // here we extract an Interval QI from the `age` attribute
//!     fn get_age_qi(&self) -> QuasiIdentifierTypes {
//!         QuasiIdentifierTypes::Interval((
//!             QuasiIdentifierType::Integer(self.age),
//!             QuasiIdentifierType::Integer(1),
//!             QuasiIdentifierType::Integer(100),
//!             1,
//!         ))
//!     }
//!
//!     // here we extract an Interval QI from the `capital_gain` attribute
//!     fn get_capital_gain_qi(&self) -> QuasiIdentifierTypes {
//!         QuasiIdentifierTypes::Interval((
//!             QuasiIdentifierType::Integer(self.capital_gain),
//!             QuasiIdentifierType::Integer(0),
//!             QuasiIdentifierType::Integer(100000),
//!             1,
//!         ))
//!     }
//!
//!     // here we extract an Interval QI from the `capital_loss` attribute
//!     fn get_capital_loss_qi(&self) -> QuasiIdentifierTypes {
//!         QuasiIdentifierTypes::Interval((
//!             QuasiIdentifierType::Integer(self.capital_loss),
//!             QuasiIdentifierType::Integer(0),
//!             QuasiIdentifierType::Integer(5000),
//!             1,
//!         ))
//!     }
//!
//! }
//!
//! // Here we implement the `Anonymizable` trait
//! impl Anonymizable for Adult {
//!     // We extract the QIs from the datastructure and return a `vec` of QIs
//!     fn quasi_identifiers(&self) -> Vec<QuasiIdentifierTypes> {
//!         let age = self.get_age_qi();
//!         let capital_gain = self.get_capital_gain_qi();
//!         let capital_loss = self.get_capital_loss_qi();
//!
//!         vec![
//!             age,
//!             capital_gain,
//!             capital_loss,
//!         ]
//!     }
//!     
//!     // We update the datastructures QIs with a `vec` of QIs. The `vec` needs to be
//!     // popped in the same order that the QIs are extracted with the `quasi_identifiers`
//!     // function
//!     fn update_quasi_identifiers(&self, mut qi: Vec<QuasiIdentifierTypes>) -> Self {
//!         if let (
//!             QuasiIdentifierType::Integer(capital_loss),
//!             QuasiIdentifierType::Integer(capital_gain),
//!             QuasiIdentifierType::Integer(age),
//!         ) = (
//!             qi.pop().unwrap().extract_value(),
//!             qi.pop().unwrap().extract_value(),
//!             qi.pop().unwrap().extract_value(),
//!         ) {
//!             Self {
//!                 timestamp: self.timestamp,
//!                 age,
//!                 capital_gain,
//!                 capital_loss,
//!                 class: self.class.to_owned(),
//!                 time_generated: self.time_generated,
//!             }
//!         } else {
//!             panic!("Couldn't Adult with QI's")
//!         }
//!     }
//!     
//!     // We extract the sensative attribute from the datastructure
//!     fn sensitive_value(&self) -> SensitiveAttribute {
//!         SensitiveAttribute::String(self.class.to_owned())
//!     }
//!
//!     // We return a vector of strings containing the String version of the QIs
//!     // Used for printing to CSVs
//!     fn extract_string_values(&self, uuid: Uuid, dr: f64) -> Vec<String> {
//!         vec![
//!             uuid.to_string(),
//!             dr.to_string(),
//!             self.timestamp.to_string(),
//!             self.age.to_string(),
//!             self.capital_gain.to_string(),
//!             self.capital_loss.to_string(),
//!             self.class.to_owned(),
//!         ]
//!     }
//!
//!     fn get_timestamp(&self) -> SystemTime {
//!         self.time_generated
//!     }
//! }
//! ```

#[macro_use]
extern crate serde;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

extern crate core;
extern crate pretty_env_logger;

pub mod analysis;
pub mod anonymization;
pub mod config;
pub mod data_manipulation;
pub mod kafka;
pub mod noise;
pub mod publishing;
pub mod test;
pub mod vec_set;
