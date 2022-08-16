pub struct Environment {
    pub k: usize,
    pub k_max: usize,
    pub l: usize,
    pub c: i32,
    pub diff_thres: f64,
    pub eps: f64,
    pub delta: u128,
    pub buff_size: usize,
    pub noise_thr: f64,
    pub dataset: Datasets,
    pub publish_remaining_tuples: bool,
}

#[derive(Copy, Clone)]
pub enum Datasets {
    Adult(Dataset),
    AdultLarge(Dataset),
    Mueller(Dataset),
}

impl Datasets {
    pub fn extract(&self) -> Dataset {
        match *self {
            Datasets::Adult(dataset) => dataset,
            Datasets::AdultLarge(dataset) => dataset,
            Datasets::Mueller(dataset) => dataset,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct Dataset {
    pub path: &'static str,
    pub export: &'static str,
}

impl Environment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        k: usize,
        k_max: usize,
        l: usize,
        c: i32,
        diff_thres: f64,
        eps: f64,
        delta: u128,
        noise_thr: f64,
        dataset: Datasets,
        publish_remaining_tuples: bool,
    ) -> Self {
        Self {
            k,
            k_max,
            l,
            c,
            diff_thres,
            eps,
            delta,
            buff_size: k * 3,
            noise_thr,
            dataset,
            publish_remaining_tuples,
        }
    }
}
