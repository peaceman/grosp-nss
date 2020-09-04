use std::path::PathBuf;
use std::sync::{Arc, RwLock, Weak};
use std::time::{Duration, Instant};

use log::{info, trace};
use tokio::sync::watch;
use tokio::time;

use super::{Bandwidth, BandwidthProvider};
use crate::stats::{NodeStatsDataSource, NodeStatsUpdateNotifier};

pub trait CounterSource: Send + Sync {
    fn get_rx(&self) -> u64;
    fn get_tx(&self) -> u64;
}

pub struct CounterRateBandwidthProvider {
    bandwidth: Arc<RwLock<Arc<Bandwidth>>>,
    update_receiver: watch::Receiver<()>,
}

#[derive(Default, Debug)]
struct CounterValues {
    rx: u64,
    tx: u64,
}

impl CounterRateBandwidthProvider {
    pub fn new<T: CounterSource + 'static>(source: T, update_interval: Duration) -> Self {
        let shared_bandwidth = Arc::new(RwLock::new(Arc::new(Default::default())));

        let (tx, rx) = watch::channel(());

        let provider = Self {
            bandwidth: Arc::clone(&shared_bandwidth),
            update_receiver: rx,
        };

        start_update_loop(
            Arc::downgrade(&shared_bandwidth),
            tx,
            source,
            update_interval,
        );

        provider
    }
}

impl BandwidthProvider for CounterRateBandwidthProvider {
    fn current_bandwidth(&self) -> Arc<Bandwidth> {
        Arc::clone(&self.bandwidth.read().unwrap())
    }
}

impl NodeStatsUpdateNotifier for CounterRateBandwidthProvider {
    fn get_update_channel_receiver(&self) -> watch::Receiver<()> {
        self.update_receiver.clone()
    }
}

impl NodeStatsDataSource for CounterRateBandwidthProvider {
    fn get_name(&self) -> &'static str {
        "CounterRateBandwidthProvider"
    }
}

fn start_update_loop<T: CounterSource + 'static>(
    bandwidth: Weak<RwLock<Arc<Bandwidth>>>,
    update_sender: watch::Sender<()>,
    counter_source: T,
    update_interval: Duration,
) {
    info!("Start CounterRateBandwidthProvider update loop");

    tokio::spawn(async move {
        update_loop(bandwidth, update_sender, counter_source, update_interval).await
    });
}

async fn update_loop<T: CounterSource>(
    bandwidth: Weak<RwLock<Arc<Bandwidth>>>,
    update_sender: watch::Sender<()>,
    counter_source: T,
    update_interval: Duration,
) {
    let mut interval = time::interval(update_interval);
    let mut last_time: Option<Instant> = None;
    let mut last_counter_values: CounterValues = Default::default();

    interval.tick().await; // the first tick will complete immediately

    loop {
        let bandwidth = match bandwidth.upgrade() {
            Some(bandwidth) => bandwidth,
            None => {
                info!("Couldn't get a reference to the bandwidth storage, ending update loop");
                break;
            }
        };

        let current_counter_values = CounterValues {
            rx: counter_source.get_rx(),
            tx: counter_source.get_tx(),
        };
        let current_time = Instant::now();

        if let Some(new_bandwidth) = calc_bandwidth(
            &current_counter_values,
            &last_counter_values,
            &last_time,
            &current_time,
        ) {
            *bandwidth.write().unwrap() = Arc::new(new_bandwidth);
            update_sender.broadcast(()).unwrap();
        }

        last_time = Some(current_time);
        last_counter_values = current_counter_values;

        interval.tick().await;
    }
}

fn calc_bandwidth(
    current_counter_values: &CounterValues,
    last_counter_values: &CounterValues,
    last_time: &Option<Instant>,
    current_time: &Instant,
) -> Option<Bandwidth> {
    if last_time.is_none() {
        return None;
    }

    let last_time = last_time.unwrap();

    trace!(
        "Calculating bandwidth last_time: {:?} current_time: {:?}",
        last_time,
        current_time
    );

    if current_counter_values.rx < last_counter_values.rx
        || current_counter_values.tx < last_counter_values.tx
    {
        trace!(
            "Current counter values are smaller than the last counter values {:?} {:?}",
            current_counter_values,
            last_counter_values
        );
        return None;
    }

    let duration_in_s = current_time.duration_since(last_time).as_secs();

    if duration_in_s == 0 {
        trace!("Duration since last calculation is less than a second");
        return None;
    }

    let rx_diff = current_counter_values.rx - last_counter_values.rx;
    let tx_diff = current_counter_values.tx - last_counter_values.tx;

    let rx_bps = (rx_diff / duration_in_s) * 8;
    let tx_bps = (tx_diff / duration_in_s) * 8;

    let bandwidth = Bandwidth { rx_bps, tx_bps };
    trace!("Calculated bandwidth: {:?}", bandwidth);

    Some(bandwidth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use tokio::time;

    #[test]
    fn test_calc_bandwidth_first_run() {
        let current_counter_values = CounterValues { rx: 0, tx: 0 };
        let last_counter_values = CounterValues { rx: 0, tx: 0 };
        let last_time = None;
        let current_time = Instant::now();

        assert_eq!(
            None,
            calc_bandwidth(
                &current_counter_values,
                &last_counter_values,
                &last_time,
                &current_time,
            )
        );
    }

    #[test]
    fn test_calc_bandwidth_last_greater_than_current() {
        let current_counter_values = CounterValues { rx: 0, tx: 0 };
        let last_counter_values = CounterValues { rx: 1000, tx: 1000 };
        let last_time = Instant::now();
        let current_time = last_time + Duration::from_secs(10);

        assert_eq!(
            None,
            calc_bandwidth(
                &current_counter_values,
                &last_counter_values,
                &Some(last_time),
                &current_time,
            )
        );
    }

    #[test]
    fn test_calc_bandwidth_duration_since_last_calculation_less_than_a_second() {
        let current_counter_values = CounterValues { rx: 1000, tx: 1000 };
        let last_counter_values = CounterValues { rx: 0, tx: 0 };
        let last_time = Instant::now();
        let current_time = last_time;

        assert_eq!(
            None,
            calc_bandwidth(
                &current_counter_values,
                &last_counter_values,
                &Some(last_time),
                &current_time,
            )
        );
    }

    #[test]
    fn test_calc_bandwidth_regular() {
        let current_counter_values = CounterValues { rx: 2000, tx: 2000 };
        let last_counter_values = CounterValues { rx: 0, tx: 0 };
        let last_time = Instant::now();
        let current_time = last_time + Duration::from_secs(2);

        assert_eq!(
            Some(Bandwidth {
                rx_bps: 8000,
                tx_bps: 8000
            }),
            calc_bandwidth(
                &current_counter_values,
                &last_counter_values,
                &Some(last_time),
                &current_time,
            )
        );
    }

    #[tokio::test]
    async fn test_counter_rate_bandwidth_provider() {
        let counter_source = MockCounterSource {
            rx: Mutex::new(vec![0, 1000].into_iter().cycle()),
            tx: Mutex::new(vec![0, 1000].into_iter().cycle()),
        };

        let bandwidth_provider =
            CounterRateBandwidthProvider::new(counter_source, Duration::from_secs(1));

        let loop_start = Instant::now();
        let expected_bandwidth = Bandwidth {
            rx_bps: 8000,
            tx_bps: 8000,
        };
        loop {
            time::delay_for(Duration::from_millis(50)).await;
            let bandwidth = bandwidth_provider.current_bandwidth();

            if loop_start.elapsed() >= Duration::from_secs(2) {
                panic!("Failed to retrieve the expected bandwidth in time");
            }

            if *bandwidth == expected_bandwidth {
                break;
            }
        }

        assert_eq!(true, true);
    }

    struct MockCounterSource<T: Iterator> {
        rx: Mutex<T>,
        tx: Mutex<T>,
    }

    impl<T: Iterator + Send + Sync> CounterSource for MockCounterSource<T>
    where
        T: std::iter::Iterator<Item = u64>,
    {
        fn get_rx(&self) -> u64 {
            self.rx.lock().unwrap().next().unwrap()
        }

        fn get_tx(&self) -> u64 {
            self.tx.lock().unwrap().next().unwrap()
        }
    }
}
