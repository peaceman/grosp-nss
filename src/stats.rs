mod bandwidth;

use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;

use bandwidth::*;
use log::info;
use tokio::time;

pub use bandwidth::RandomBandwidthProvider;

#[derive(Debug, Default)]
pub struct NodeStats {
    pub bandwidth: Arc<Bandwidth>,
}

pub trait NodeStatsUpdater: Send + Sync {
    fn update_node_stats(&self, node_stats: NodeStats) -> NodeStats;
}

pub struct NodeStatsProvider {
    node_stats: Arc<RwLock<Arc<NodeStats>>>,
}

impl NodeStatsProvider {
    pub fn new(updaters: Vec<Box<dyn NodeStatsUpdater>>) -> Self {
        let shared_node_stats = Arc::new(RwLock::new(Arc::new(Default::default())));

        let provider = Self {
            node_stats: Arc::clone(&shared_node_stats),
        };

        start_update_loop(Arc::downgrade(&shared_node_stats), updaters);

        provider
    }

    pub fn current_node_stats(&self) -> Arc<NodeStats> {
        Arc::clone(&self.node_stats.read().unwrap())
    }
}

fn start_update_loop(
    node_stats: Weak<RwLock<Arc<NodeStats>>>,
    updaters: Vec<Box<dyn NodeStatsUpdater>>,
) {
    info!("Start NodeStatsProvider update loop");

    tokio::spawn(async move { update_loop(node_stats, updaters).await });
}

async fn update_loop(
    node_stats: Weak<RwLock<Arc<NodeStats>>>,
    updaters: Vec<Box<dyn NodeStatsUpdater>>,
) {
    let mut interval = time::interval(Duration::from_secs(1));

    loop {
        let node_stats = match node_stats.upgrade() {
            Some(node_stats) => node_stats,
            None => {
                info!("Couldn't get a reference to the node stats storage, ending update loop");
                break;
            }
        };

        let mut new_node_stats: NodeStats = Default::default();
        for updater in &updaters {
            new_node_stats = updater.update_node_stats(new_node_stats);
        }

        *node_stats.write().unwrap() = Arc::new(new_node_stats);

        interval.tick().await;
    }
}
