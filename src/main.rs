use std::sync::Arc;

use tonic::transport::Server;

use node_stats_service::{
    grpc, settings::Settings, stats::NodeStatsProvider, stats::RandomBandwidthProvider,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let settings = Settings::from_file("config.yml").expect("Failed to load config");

    let node_stats_service = grpc::NodeStatsService {
        node_stats_provider: Arc::new(NodeStatsProvider::new(vec![Box::new(
            RandomBandwidthProvider::new(),
        )])),
    };

    let svc = grpc::NodeStatsServiceServer::new(node_stats_service);

    // let addr = "[::1]:10000".parse()?;
    // let route_guide = RouteGuideService {
    //     features: Arc::new(data::load()),
    // };

    // let svc = RouteGuideServer::new(route_guide);

    Server::builder()
        .add_service(svc)
        .serve(settings.http.socket)
        .await?;

    Ok(())
}
