use crate::analysis::cluster_analyser::ClusterAnalyser;
use crate::analysis::disclosure_risk_analyser::DisclosureRiskAnalyser;
use crate::analysis::mse_analyser::MseAnalyser;
use crate::analysis::publishing_delay_analyser::PublishingDelayAnalyser;
use crate::analysis::sse_analyser::SseAnalyser;
use crate::data_manipulation::anonymizable::Anonymizable;

/// This enum encompasses all the different analysers used in the framework
pub enum Analyser<A: Anonymizable> {
    Mse(MseAnalyser),
    Sse(SseAnalyser),
    PublishingDelay(PublishingDelayAnalyser),
    ClusterAnalyser(ClusterAnalyser),
    DisclosureRiskAnalyser(DisclosureRiskAnalyser<A>),
}
