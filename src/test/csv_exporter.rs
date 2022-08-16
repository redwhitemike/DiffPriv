use crate::test::environment::Datasets;
use csv::Writer;

const MUELLER_HEADERS: [&str; 16] = [
    "uuid",
    "disclosure_risk",
    "id",
    "gender",
    "age",
    "run_id",
    "running",
    "speed",
    "duration",
    "distance",
    "side",
    "bout",
    "freq",
    "walk_ratio",
    "start",
    "end",
];

const ADULT_LARGE_HEADERS: [&str; 15] = [
    "uuid",
    "disclosure_risk",
    "timestamp",
    "age",
    "fnlwgt",
    "education_num",
    "capital_gain",
    "capital_loss",
    "hours_per_week",
    "education",
    "marital_status",
    "workclass",
    "native_country",
    "occupation",
    "class",
];

const ADULT_HEADERS: [&str; 10] = [
    "uuid",
    "disclosure_risk",
    "timestamp",
    "age",
    "fnlwgt",
    "education_num",
    "capital_gain",
    "capital_loss",
    "hours_per_week",
    "class",
];

pub struct CsvExporter {
    data: Vec<Vec<String>>,
    pub path: String,
    dataset: Datasets,
}

impl CsvExporter {
    pub fn new(path: String, dataset: Datasets) -> Self {
        Self {
            path,
            data: vec![],
            dataset,
        }
    }

    pub fn export(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Exporting to {}", self.path);
        let mut writer = Writer::from_path(&self.path)?;
        // write header
        match self.dataset {
            Datasets::Adult(_) => writer.write_record(&ADULT_HEADERS)?,
            Datasets::AdultLarge(_) => writer.write_record(&ADULT_LARGE_HEADERS)?,
            Datasets::Mueller(_) => writer.write_record(&MUELLER_HEADERS)?,
        }

        for record in &self.data {
            writer.write_record(record)?
        }

        writer.flush()?;
        Ok(())
    }

    pub fn add(&mut self, value: Vec<String>) {
        self.data.push(value)
    }
}
