use std::sync::Arc;

use tonic::transport::Server;

use node_stats_service::{
    grpc,
    settings::Settings,
    stats::bandwidth::{CounterRateBandwidthProvider, FileCounterSource},
    stats::NodeStatsProvider,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let settings = Settings::from_file("config.yml").expect("Failed to load config");

    let node_stats_service = grpc::NodeStatsService {
        node_stats_provider: Arc::new(NodeStatsProvider::new(vec![Box::new(
            CounterRateBandwidthProvider::new(
                FileCounterSource::new(
                    settings.node_stats.bandwidth.rx_file,
                    settings.node_stats.bandwidth.tx_file,
                ),
                settings.node_stats.bandwidth.update_interval,
            ),
        )])),
    };

    let svc = grpc::NodeStatsServiceServer::new(node_stats_service);

    Server::builder()
        .add_service(svc)
        .serve(settings.http.socket)
        .await?;

    Ok(())
}
