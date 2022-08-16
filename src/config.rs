use hocon::HoconLoader;

#[derive(Deserialize)]
pub struct Config {
    pub topic_in: String,
    pub topic_out: String,
    pub kafka_bootstrap: String,
    pub k: Vec<usize>,
    pub l: Vec<usize>,
    pub c: Vec<i32>,
    pub eps: Vec<f64>,
    pub diff_thres: Vec<f64>,
    pub delta: Vec<u64>,
    pub noise_thr: Vec<f64>,
    pub publish_remaining_tuples: bool,
}

impl Config {
    pub fn new(conf_file: &String) -> Self {
        let config: Config = HoconLoader::new()
            .load_file(conf_file)
            .expect("No application.conf found")
            .resolve()
            .expect("couldn't convert to Config");

        config
    }
}
