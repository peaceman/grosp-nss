mod counter_rate;
mod random;

use std::sync::Arc;

use super::{NodeStats, NodeStatsUpdater};

pub use random::RandomBandwidthProvider;

#[derive(Debug, PartialEq)]
pub struct Bandwidth {
    pub tx_bps: u64,
    pub rx_bps: u64,
}

impl Default for Bandwidth {
    fn default() -> Self {
        Self {
            tx_bps: 0,
            rx_bps: 0,
        }
    }
}

pub trait BandwidthProvider: Send + Sync {
    fn current_bandwidth(&self) -> Arc<Bandwidth>;
}

impl<T: BandwidthProvider> NodeStatsUpdater for T {
    fn update_node_stats(&self, mut node_stats: NodeStats) -> NodeStats {
        node_stats.bandwidth = self.current_bandwidth();

        node_stats
    }
}
