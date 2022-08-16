use std::time::{SystemTime, UNIX_EPOCH};

use bimap::BiMap;
use uuid::Uuid;

use crate::data_manipulation::anonymizable::{
    Anonymizable, QuasiIdentifierType, QuasiIdentifierTypes, SensitiveAttribute,
};

lazy_static! {
    static ref EDU_BIMAP: BiMap<&'static str, i32> = {
        BiMap::from_iter(vec![
            (" Preschool", 0),
            (" Bachelors", 1),
            (" Some-college", 2),
            (" 11th", 3),
            (" HS-grad", 4),
            (" Prof-school", 5),
            (" Assoc-acdm", 6),
            (" Assoc-voc", 7),
            (" 9th", 8),
            (" 7th-8th", 9),
            (" 12th", 10),
            (" Masters", 11),
            (" 1st-4th", 12),
            (" 10th", 13),
            (" Doctorate", 14),
            (" 5th-6th", 15),
        ])
    };
    static ref MAR_BIMAP: BiMap<&'static str, i32> = {
        BiMap::from_iter(vec![
            (" Married-AF-spouse", 0),
            (" Married-civ-spouse", 1),
            (" Divorced", 2),
            (" Never-married", 3),
            (" Separated", 4),
            (" Widowed", 5),
            (" Married-spouse-absent", 6),
        ])
    };
    static ref WORK_BIMAP: BiMap<&'static str, i32> = {
        BiMap::from_iter(vec![
            (" Never-worked", 0),
            (" Private", 1),
            (" Self-emp-not-inc", 2),
            (" Self-emp-inc", 3),
            (" Federal-gov", 4),
            (" Local-gov", 5),
            (" State-gov", 6),
            (" Without-pay", 7),
        ])
    };
    static ref NAT_BIMAP: BiMap<&'static str, i32> = {
        BiMap::from_iter(vec![
            (" Holand-Netherlands", 0),
            (" United-States", 1),
            (" Cambodia", 2),
            (" England", 3),
            (" Puerto-Rico", 4),
            (" Canada", 5),
            (" Germany", 6),
            (" Outlying-US(Guam-USVI-etc)", 7),
            (" India", 8),
            (" Japan", 9),
            (" Greece", 10),
            (" South", 11),
            (" China", 12),
            (" Cuba", 13),
            (" Iran", 14),
            (" Honduras", 15),
            (" Philippines", 16),
            (" Italy", 17),
            (" Poland", 18),
            (" Jamaica", 19),
            (" Vietnam", 20),
            (" Mexico", 21),
            (" Portugal", 22),
            (" Ireland", 23),
            (" France", 24),
            (" Dominican-Republic", 25),
            (" Laos", 26),
            (" Ecuador", 27),
            (" Taiwan", 28),
            (" Haiti", 29),
            (" Columbia", 30),
            (" Hungary", 31),
            (" Guatemala", 32),
            (" Nicaragua", 33),
            (" Scotland", 34),
            (" Thailand", 35),
            (" Yugoslavia", 36),
            (" El-Salvador", 37),
            (" Trinadad&Tobago", 38),
            (" Peru", 39),
            (" Hong", 40),
        ])
    };
    static ref OCC_BIMAP: BiMap<&'static str, i32> = {
        BiMap::from_iter(vec![
            (" Armed-Forces", 0),
            (" Tech-support", 1),
            (" Craft-repair", 2),
            (" Other-service", 3),
            (" Sales", 4),
            (" Exec-managerial", 5),
            (" Prof-specialty", 6),
            (" Handlers-cleaners", 7),
            (" Machine-op-inspct", 8),
            (" Adm-clerical", 9),
            (" Farming-fishing", 10),
            (" Transport-moving", 11),
            (" Priv-house-serv", 12),
            (" Protective-serv", 13),
        ])
    };
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct AdultLarge {
    timestamp: i32,
    age: i32,
    fnlwgt: i32,
    education_num: i32,
    capital_gain: i32,
    capital_loss: i32,
    hours_per_week: i32,
    education: String,
    marital_status: String,
    workclass: String,
    native_country: String,
    occupation: String,
    class: String,
    #[serde(skip_deserializing, default = "default_time")]
    time_generated: SystemTime,
}

fn default_time() -> SystemTime {
    SystemTime::now()
}

impl Default for AdultLarge {
    fn default() -> Self {
        Self {
            timestamp: 0,
            age: 0,
            fnlwgt: 0,
            education_num: 0,
            capital_gain: 0,
            capital_loss: 0,
            hours_per_week: 0,
            education: "".to_string(),
            marital_status: "".to_string(),
            workclass: "".to_string(),
            native_country: "".to_string(),
            occupation: "".to_string(),
            class: "".to_string(),
            time_generated: SystemTime::now(),
        }
    }
}

impl AdultLarge {
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

    fn get_education_qi(&self) -> QuasiIdentifierTypes {
        let rank = match EDU_BIMAP.get_by_left(self.education.as_str()) {
            None => {
                panic!("Wrong education status found: {}", self.education)
            }
            Some(education) => *education,
        };

        QuasiIdentifierTypes::Nominal((rank, 15, 1))
    }

    fn get_marital_status_qi(&self) -> QuasiIdentifierTypes {
        let rank = match MAR_BIMAP.get_by_left(self.marital_status.as_str()) {
            None => panic!("Wrong marital status found: {}", self.marital_status),
            Some(marital) => *marital,
        };

        QuasiIdentifierTypes::Nominal((rank, 6, 1))
    }

    fn get_workclass_qi(&self) -> QuasiIdentifierTypes {
        let rank = match WORK_BIMAP.get_by_left(self.workclass.as_str()) {
            None => panic!("Wrong workclass status found: {}", self.workclass),
            Some(workclass) => *workclass,
        };

        QuasiIdentifierTypes::Nominal((rank, 7, 1))
    }

    fn get_native_country_qi(&self) -> QuasiIdentifierTypes {
        let rank = match NAT_BIMAP.get_by_left(self.native_country.as_str()) {
            None => panic!("Wrong native country found: {}", self.native_country),
            Some(native) => *native,
        };

        QuasiIdentifierTypes::Nominal((rank, 40, 1))
    }

    fn get_occupation_qi(&self) -> QuasiIdentifierTypes {
        let rank = match OCC_BIMAP.get_by_left(self.occupation.as_str()) {
            None => panic!("Wrong marital status found: {}", self.occupation),
            Some(occupation) => *occupation,
        };

        QuasiIdentifierTypes::Nominal((rank, 13, 1))
    }
}

impl Anonymizable for AdultLarge {
    fn quasi_identifiers(&self) -> Vec<QuasiIdentifierTypes> {
        let age = self.get_age_qi();
        let fnlwgt = self.get_fnlwgt_qi();
        let education_num = self.get_education_num_qi();
        let capital_gain = self.get_capital_gain_qi();
        let capital_loss = self.get_capital_loss_qi();
        let hours_per_week = self.get_hours_per_week_qi();
        let education = self.get_education_qi();
        let marital_status = self.get_marital_status_qi();
        let workclass = self.get_workclass_qi();
        let native_country = self.get_native_country_qi();
        let occupation = self.get_occupation_qi();

        vec![
            age,
            fnlwgt,
            education_num,
            capital_gain,
            capital_loss,
            hours_per_week,
            education,
            marital_status,
            workclass,
            native_country,
            occupation,
        ]
    }

    fn update_quasi_identifiers(&self, mut qi: Vec<QuasiIdentifierTypes>) -> Self {
        if let (
            QuasiIdentifierType::Integer(occupation),
            QuasiIdentifierType::Integer(native_country),
            QuasiIdentifierType::Integer(workclass),
            QuasiIdentifierType::Integer(marital_status),
            QuasiIdentifierType::Integer(education),
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
                education: EDU_BIMAP.get_by_right(&education).unwrap().to_string(),
                marital_status: MAR_BIMAP.get_by_right(&marital_status).unwrap().to_string(),
                workclass: WORK_BIMAP.get_by_right(&workclass).unwrap().to_string(),
                native_country: NAT_BIMAP.get_by_right(&native_country).unwrap().to_string(),
                occupation: OCC_BIMAP.get_by_right(&occupation).unwrap().to_string(),
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
            self.education.to_owned(),
            self.marital_status.to_owned(),
            self.workclass.to_owned(),
            self.native_country.to_owned(),
            self.occupation.to_owned(),
            self.class.to_owned(),
        ]
    }

    fn get_timestamp(&self) -> SystemTime {
        self.time_generated
    }
}
