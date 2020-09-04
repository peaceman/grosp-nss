pub mod proto {
    tonic::include_proto!("nodestats");
}

use std::sync::Arc;
use std::time::Duration;

use log::info;

use tokio::sync::mpsc;
use tokio::time;
use tonic::{Request, Response, Status};

use super::stats;

pub use proto::node_stats_service_server::NodeStatsServiceServer;

pub struct NodeStatsService {
    pub node_stats_provider: Arc<stats::NodeStatsProvider>,
}

#[tonic::async_trait]
impl proto::node_stats_service_server::NodeStatsService for NodeStatsService {
    type GetLiveStatsStream = mpsc::Receiver<Result<proto::NodeStats, Status>>;

    async fn get_live_stats(
        &self,
        request: Request<proto::LiveNodeStatsRequest>,
    ) -> Result<Response<Self::GetLiveStatsStream>, Status> {
        let node_stats_provider = Arc::clone(&self.node_stats_provider);
        let (mut tx, rx) = mpsc::channel(1);

        tokio::spawn(async move {
            info!(
                "Starting live stats streaming for client: {:?}",
                request.remote_addr()
            );
            let mut interval = time::interval(Duration::from_secs(1));
            interval.tick().await; // the first tick completes immediately

            loop {
                let node_stats = node_stats_provider.current_node_stats();
                let send_result = tx
                    .send(Ok(proto::NodeStats::from(node_stats.as_ref())))
                    .await;

                if let Err(e) = send_result {
                    info!("Failed to send live stats, stopping live stats streaming for client {:?}, SendError: {:?}", request.remote_addr(), e);
                    break;
                } else {
                    info!("Sent live stats to client {:?}", request.remote_addr());
                }

                interval.tick().await;
            }
        });

        Ok(Response::new(rx))
    }
}

impl From<&stats::NodeStats> for proto::NodeStats {
    fn from(node_stats: &stats::NodeStats) -> Self {
        proto::NodeStats {
            used_bandwidth: Some(proto::Bandwidth {
                tx_bps: node_stats.bandwidth.tx_bps,
                rx_bps: node_stats.bandwidth.rx_bps,
            }),
        }
    }
}
