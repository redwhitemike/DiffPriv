use std::time::{SystemTime, UNIX_EPOCH};

use bimap::BiMap;
use uuid::Uuid;

use crate::data_manipulation::anonymizable::{
    Anonymizable, QuasiIdentifierType, QuasiIdentifierTypes, SensitiveAttribute,
};

lazy_static! {
    static ref CLASS_BIMAP: BiMap<&'static str, i32> =
        BiMap::from_iter(vec![("<=50K", 0), (">50K", 1),]);
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Adult {
    timestamp: i32,
    age: i32,
    fnlwgt: i32,
    education_num: i32,
    capital_gain: i32,
    capital_loss: i32,
    hours_per_week: i32,
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
            fnlwgt: 0,
            education_num: 0,
            capital_gain: 0,
            capital_loss: 0,
            hours_per_week: 0,
            class: "".to_string(),
            time_generated: SystemTime::now(),
        }
    }
}

impl Adult {
    fn get_age_qi(&self) -> QuasiIdentifierTypes {
        QuasiIdentifierTypes::Interval((
            QuasiIdentifierType::Integer(self.age),
            QuasiIdentifierType::Integer(1),
            QuasiIdentifierType::Integer(100),
            1,
        ))
    }

    fn get_fnlwgt_qi(&self) -> QuasiIdentifierTypes {
        QuasiIdentifierTypes::Interval((
            QuasiIdentifierType::Integer(self.fnlwgt),
            QuasiIdentifierType::Integer(1),
            QuasiIdentifierType::Integer(1500000),
            1,
        ))
    }

    fn get_education_num_qi(&self) -> QuasiIdentifierTypes {
        QuasiIdentifierTypes::Interval((
            QuasiIdentifierType::Integer(self.education_num),
            QuasiIdentifierType::Integer(1),
            QuasiIdentifierType::Integer(20),
            1,
        ))
    }

    fn get_capital_gain_qi(&self) -> QuasiIdentifierTypes {
        QuasiIdentifierTypes::Interval((
            QuasiIdentifierType::Integer(self.capital_gain),
            QuasiIdentifierType::Integer(0),
            QuasiIdentifierType::Integer(100000),
            1,
        ))
    }

    fn get_capital_loss_qi(&self) -> QuasiIdentifierTypes {
        QuasiIdentifierTypes::Interval((
            QuasiIdentifierType::Integer(self.capital_loss),
            QuasiIdentifierType::Integer(0),
            QuasiIdentifierType::Integer(5000),
            1,
        ))
    }

    fn get_hours_per_week_qi(&self) -> QuasiIdentifierTypes {
        QuasiIdentifierTypes::Interval((
            QuasiIdentifierType::Integer(self.hours_per_week),
            QuasiIdentifierType::Integer(1),
            QuasiIdentifierType::Integer(100),
            1,
        ))
    }
}

impl Anonymizable for Adult {
    fn quasi_identifiers(&self) -> Vec<QuasiIdentifierTypes> {
        let age = self.get_age_qi();
        let fnlwgt = self.get_fnlwgt_qi();
        let education_num = self.get_education_num_qi();
        let capital_gain = self.get_capital_gain_qi();
        let capital_loss = self.get_capital_loss_qi();
        let hours_per_week = self.get_hours_per_week_qi();

        vec![
            age,
            fnlwgt,
            education_num,
            capital_gain,
            capital_loss,
            hours_per_week,
        ]
    }

    fn update_quasi_identifiers(&self, mut qi: Vec<QuasiIdentifierTypes>) -> Self {
        if let (
            QuasiIdentifierType::Integer(hours_per_week),
            QuasiIdentifierType::Integer(capital_loss),
            QuasiIdentifierType::Integer(capital_gain),
            QuasiIdentifierType::Integer(education_num),
            QuasiIdentifierType::Integer(fnlwgt),
            QuasiIdentifierType::Integer(age),
        ) = (
            qi.pop().unwrap().extract_value(),
            qi.pop().unwrap().extract_value(),
            qi.pop().unwrap().extract_value(),
            qi.pop().unwrap().extract_value(),
            qi.pop().unwrap().extract_value(),
            qi.pop().unwrap().extract_value(),
        ) {
            Self {
                timestamp: self.timestamp,
                age,
                fnlwgt,
                education_num,
                capital_gain,
                capital_loss,
                hours_per_week,
                class: self.class.to_owned(),
                time_generated: self.time_generated,
            }
        } else {
            panic!("Couldn't Adult with QI's")
        }
    }

    fn sensitive_value(&self) -> SensitiveAttribute {
        SensitiveAttribute::String(self.class.to_owned())
    }

    fn extract_string_values(&self, uuid: Uuid, dr: f64) -> Vec<String> {
        vec![
            uuid.to_string(),
            dr.to_string(),
            self.timestamp.to_string(),
            self.age.to_string(),
            self.fnlwgt.to_string(),
            self.education_num.to_string(),
            self.capital_gain.to_string(),
            self.capital_loss.to_string(),
            self.hours_per_week.to_string(),
            self.class.to_owned(),
        ]
    }

    fn get_timestamp(&self) -> SystemTime {
        self.time_generated
    }
}
