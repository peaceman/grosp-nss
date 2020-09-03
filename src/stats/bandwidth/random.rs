use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;

use log::info;
use rand::thread_rng;
use rand::Rng;
use tokio::time;
use tokio::sync::watch;

use super::{Bandwidth, BandwidthProvider};
use crate::stats::{NodeStatsUpdateNotifier, NodeStatsDataSource};

pub struct RandomBandwidthProvider {
    bandwidth: Arc<RwLock<Arc<Bandwidth>>>,
    update_receiver: watch::Receiver<()>,
}

impl RandomBandwidthProvider {
    pub fn new() -> Self {
        let shared_bandwidth = Arc::new(RwLock::new(Arc::new(Default::default())));

        let (tx, rx) = watch::channel(());

        let provider = Self {
            bandwidth: Arc::clone(&shared_bandwidth),
            update_receiver: rx,
        };

        start_update_loop(Arc::downgrade(&shared_bandwidth), tx);

        provider
    }
}

impl BandwidthProvider for RandomBandwidthProvider {
    fn current_bandwidth(&self) -> Arc<Bandwidth> {
        Arc::clone(&self.bandwidth.read().unwrap())
    }
}

impl NodeStatsUpdateNotifier for RandomBandwidthProvider {
    fn get_update_channel_receiver(&self) -> watch::Receiver<()> {
        self.update_receiver.clone()
    }
}

impl NodeStatsDataSource for RandomBandwidthProvider {
    fn get_name(&self) -> &'static str {
        "RandomBandwidthProvider"
    }
}

impl Default for RandomBandwidthProvider {
    fn default() -> Self {
        RandomBandwidthProvider::new()
    }
}

fn start_update_loop(bandwidth: Weak<RwLock<Arc<Bandwidth>>>, update_sender: watch::Sender<()>) {
    info!("Start RandomBandwidthProvider update loop");

    tokio::spawn(async move { update_loop(bandwidth, update_sender).await });
}

async fn update_loop(bandwidth: Weak<RwLock<Arc<Bandwidth>>>, update_sender: watch::Sender<()>) {
    let mut interval = time::interval(Duration::from_secs(5));

    loop {
        {
            let bandwidth = match bandwidth.upgrade() {
                Some(bandwidth) => bandwidth,
                None => {
                    info!("Couldn't get reference to the bandwidth storage, ending update loop");
                    break;
                }
            };

            let rx_bps = thread_rng().gen::<u64>();
            let tx_bps = thread_rng().gen::<u64>();

            info!("Updating");
            *bandwidth.write().unwrap() = Arc::new(Bandwidth { rx_bps, tx_bps });
            update_sender.broadcast(()).unwrap();
        }

        interval.tick().await;
    }
}
