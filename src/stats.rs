mod bandwidth;

use std::fmt;
use std::sync::{Arc, RwLock, Weak};

use log::info;
use tokio::sync::watch::Receiver;

use crate::util::TraitDisplay;
use bandwidth::*;

pub use bandwidth::RandomBandwidthProvider;

#[derive(Debug, Default, Clone)]
pub struct NodeStats {
    pub bandwidth: Arc<Bandwidth>,
}

pub trait NodeStatsUpdater: Send + Sync {
    fn update_node_stats(&self, node_stats: NodeStats) -> NodeStats;
}

pub trait NodeStatsUpdateNotifier: Send + Sync {
    fn get_update_channel_receiver(&self) -> Receiver<()>;
}

pub trait NodeStatsDataSource: NodeStatsUpdater + NodeStatsUpdateNotifier {
    fn get_name(&self) -> &'static str;
}

impl<'a, T> fmt::Display for TraitDisplay<'a, T>
where
    T: NodeStatsDataSource + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.get_name())
    }
}

pub struct NodeStatsProvider {
    node_stats: Arc<RwLock<Arc<NodeStats>>>,
}

impl NodeStatsProvider {
    pub fn new(updaters: Vec<Box<dyn NodeStatsDataSource>>) -> Self {
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
    updaters: Vec<Box<dyn NodeStatsDataSource>>,
) {
    info!("Start NodeStatsProvider update loop");

    for updater in updaters {
        let mut update_notification_rx = updater.get_update_channel_receiver();
        let node_stats = node_stats.clone();

        tokio::spawn(async move {
            loop {
                if await_update_notification(&mut update_notification_rx, updater.as_ref())
                    .await
                    .is_none()
                {
                    break;
                }

                let node_stats = match node_stats.upgrade() {
                    Some(node_stats) => node_stats,
                    None => {
                        info!("Couldn't get a reference to the node stats storage, ending update loop, updater: {}", TraitDisplay(updater.as_ref()));
                        break;
                    }
                };

                let mut ns_lock_guard = node_stats.write().unwrap();
                *ns_lock_guard = Arc::new(updater.update_node_stats((**ns_lock_guard).clone()));
            }
        });
    }

    info!("Finished NodeStatsProvider update loop");
}

async fn await_update_notification<T: NodeStatsDataSource + ?Sized>(
    channel: &mut Receiver<()>,
    updater: &T,
) -> Option<()> {
    if channel.recv().await.is_none() {
        info!(
            "Received none over the update notification channel from {}, stop listening",
            TraitDisplay(updater)
        );
        None
    } else {
        info!(
            "Received updater notification from {}",
            TraitDisplay(updater)
        );
        Some(())
    }
}
