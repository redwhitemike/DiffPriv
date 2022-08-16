/// Analyses the behaviour of creating and deleting
/// the clusters throughout algorithms lifetime
#[derive(Default)]
pub struct ClusterAnalyser {
    pub delete_counter: i32,
    pub create_counter: i32,
}

impl ClusterAnalyser {
    pub fn add_count(&mut self) {
        self.create_counter += 1;
    }

    pub fn remove_count(&mut self) {
        self.delete_counter += 1;
    }
}
