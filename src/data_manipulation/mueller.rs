use std::time::{Duration, SystemTime, UNIX_EPOCH};

use strm_privacy_driver::StrmPrivacyValue;
use uuid::Uuid;

use crate::data_manipulation::anonymizable::QuasiIdentifierType::Integer;
use crate::data_manipulation::anonymizable::QuasiIdentifierTypes::{Interval, Nominal};
use crate::data_manipulation::anonymizable::{
    Anonymizable, QuasiIdentifierTypes, SensitiveAttribute,
};

impl StrmPrivacyValue for MuellerStream {
    const STRM_SCHEMA_REF: &'static str = "diff_priv/mueller-steps/1.0.0";
    const STRM_SCHEMA: &'static str = r#"
{"type":"record","name":"MuellerStream","namespace":"diff_priv.MuellerSteps.v1_0_0","fields":[{"name":"strmMeta","type":{"type":"record","name":"strmMeta","namespace":"diff_priv.MuellerSteps.v1_0_0.strmmeta","fields":[{"name":"eventContractRef","type":"string"},{"name":"nonce","type":["null","int"],"default":null},{"name":"timestamp","type":["null","long"],"default":null},{"name":"keyLink","type":["null","string"],"default":null},{"name":"billingId","type":["null","string"],"default":null},{"name":"consentLevels","type":{"type":"array","items":"int"}}]}},{"name":"id","type":"string","doc":"id"},{"name":"gender","type":["null","string"],"doc":"gender of the user","default":null},{"name":"age","type":["null","int"],"doc":"the URL of the current page","default":null},{"name":"run_id","type":["null","int"],"doc":"the id of the run session","default":null},{"name":"running","type":["null","boolean"],"doc":"if the user is running","default":null},{"name":"speed","type":["null","float"],"doc":"the speed of the user","default":null},{"name":"duration","type":["null","float"],"doc":"duration of the step","default":null},{"name":"distance","type":["null","float"],"doc":"distance travaled","default":null},{"name":"side","type":["null","string"],"doc":"which foot","default":null},{"name":"bout","type":["null","int"],"doc":"doc","default":null},{"name":"freq","type":["null","float"],"doc":"which foot","default":null},{"name":"walk_ratio","type":["null","float"],"doc":"which foot","default":null},{"name":"start","type":["null","string"],"doc":"start of step","default":null},{"name":"end","type":["null","string"],"doc":"end of step","default":null}]}
    "#;
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct StrmMeta {
    #[serde(rename = "eventContractRef")]
    pub event_contract_ref: String,
    pub nonce: Option<i32>,
    pub timestamp: Option<i64>,
    #[serde(rename = "keyLink")]
    pub key_link: Option<String>,
    #[serde(rename = "billingId")]
    pub billing_id: Option<String>,
    #[serde(rename = "consentLevels")]
    pub consent_levels: Vec<i32>,
}

impl Default for StrmMeta {
    fn default() -> StrmMeta {
        StrmMeta {
            event_contract_ref: "diff_priv/mueller-steps/1.0.0".to_string(),
            nonce: None,
            timestamp: None,
            key_link: None,
            billing_id: None,
            consent_levels: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MuellerStream {
    #[serde(rename = "strmMeta")]
    pub strm_meta: StrmMeta,
    pub id: String,
    pub gender: Option<String>,
    pub age: Option<i32>,
    pub run_id: Option<i32>,
    pub running: Option<bool>,
    pub speed: Option<f32>,
    pub duration: Option<f32>,
    pub distance: Option<f32>,
    pub side: Option<String>,
    pub bout: Option<i32>,
    pub freq: Option<f32>,
    pub walk_ratio: Option<f32>,
    pub start: Option<String>,
    pub end: Option<String>,
    #[serde(skip_deserializing, default = "default_time")]
    pub time_generated: SystemTime,
}

fn default_time() -> SystemTime {
    SystemTime::now()
}

impl Default for MuellerStream {
    fn default() -> MuellerStream {
        MuellerStream {
            strm_meta: StrmMeta::default(),
            id: String::default(),
            gender: None,
            age: None,
            run_id: None,
            running: None,
            speed: None,
            duration: None,
            distance: None,
            side: None,
            bout: None,
            freq: None,
            walk_ratio: None,
            start: None,
            end: None,
            time_generated: SystemTime::now(),
        }
    }
}

impl Anonymizable for MuellerStream {
    fn quasi_identifiers(&self) -> Vec<QuasiIdentifierTypes> {
        let age = Interval((Integer(self.age.unwrap()), Integer(33), Integer(85), 1));
        let gender_string = self.gender.as_ref().unwrap();
        let gender = match gender_string.as_str() {
            "male" => (0, 1, 1),
            "female" => (1, 1, 1),
            _ => panic!("Not all categories covered"),
        };

        let gender = Nominal(gender);

        vec![age, gender]
    }

    fn update_quasi_identifiers(&self, mut qi: Vec<QuasiIdentifierTypes>) -> Self {
        let mut update = self.clone();
        let gender_qi = qi.pop().unwrap().extract_value();
        let age_qi = qi.pop().unwrap().extract_value();

        match gender_qi {
            Integer(0) => update.gender = Some(String::from("male")),
            Integer(1) => update.gender = Some(String::from("female")),
            _ => panic!("Not all categories covered"),
        }

        if let Integer(age) = age_qi {
            update.age = Some(age)
        }

        update
    }

    fn sensitive_value(&self) -> SensitiveAttribute {
        SensitiveAttribute::String(self.id.to_owned())
    }

    fn extract_string_values(&self, _uuid: Uuid, _dr: f64) -> Vec<String> {
        todo!()
    }

    fn get_timestamp(&self) -> SystemTime {
        self.time_generated
    }
}
