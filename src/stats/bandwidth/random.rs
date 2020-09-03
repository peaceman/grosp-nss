use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;

use log::info;
use rand::thread_rng;
use rand::Rng;
use tokio::time;

use super::{Bandwidth, BandwidthProvider};

pub struct RandomBandwidthProvider {
    bandwidth: Arc<RwLock<Arc<Bandwidth>>>,
}

impl BandwidthProvider for RandomBandwidthProvider {
    fn current_bandwidth(&self) -> Arc<Bandwidth> {
        Arc::clone(&self.bandwidth.read().unwrap())
    }
}

impl RandomBandwidthProvider {
    pub fn new() -> Self {
        let shared_bandwidth = Arc::new(RwLock::new(Arc::new(Default::default())));

        let provider = Self {
            bandwidth: Arc::clone(&shared_bandwidth),
        };

        start_update_loop(Arc::downgrade(&shared_bandwidth));

        provider
    }
}

fn start_update_loop(bandwidth: Weak<RwLock<Arc<Bandwidth>>>) {
    info!("Start RandomBandwidthProvider update loop");

    tokio::spawn(async move { update_loop(bandwidth).await });
}

async fn update_loop(bandwidth: Weak<RwLock<Arc<Bandwidth>>>) {
    let mut interval = time::interval(Duration::from_secs(5));

    loop {
        let bandwidth = match bandwidth.upgrade() {
            Some(bandwidth) => bandwidth,
            None => {
                info!("Couldn't get reference to the bandwidth storage, ending update loop");
                break;
            }
        };

        let rx_bps = thread_rng().gen::<u64>();
        let tx_bps = thread_rng().gen::<u64>();

        *bandwidth.write().unwrap() = Arc::new(Bandwidth { rx_bps, tx_bps });

        interval.tick().await;
    }
}
