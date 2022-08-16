#[derive(Default, Serialize, Debug)]
pub struct Metrics {
    pub mse: f64,
    pub sse: f64,
    pub publishing_delay: u128,
    pub execution_time: u128,
    pub clusters_created: i32,
    pub clusters_deleted: i32,
    pub disclosure_risk: f64,
}
