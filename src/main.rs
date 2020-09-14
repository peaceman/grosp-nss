use std::sync::Arc;

use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};

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
                    &settings.node_stats.bandwidth.rx_file,
                    &settings.node_stats.bandwidth.tx_file,
                ),
                settings.node_stats.bandwidth.update_interval,
            ),
        )])),
    };

    let svc = grpc::NodeStatsServiceServer::new(node_stats_service);

    Server::builder()
        .tls_config(build_tls_config(&settings).await)?
        .add_service(svc)
        .serve(settings.http.socket)
        .await?;

    Ok(())
}

async fn build_tls_config(settings: &Settings) -> ServerTlsConfig {
    let tls_settings = &settings.http.tls;

    let cert = tokio::fs::read(&tls_settings.server_cert_file)
        .await
        .expect("Failed to load server certificate");
    let key = tokio::fs::read(&tls_settings.server_key_file)
        .await
        .expect("Failed to load server certificate key");
    let identity = Identity::from_pem(cert, key);

    let client_ca_cert = tokio::fs::read(&tls_settings.ca_cert_file)
        .await
        .expect("Failed to load client ca cert");
    let client_ca_cert = Certificate::from_pem(client_ca_cert);

    ServerTlsConfig::new()
        .identity(identity)
        .client_ca_root(client_ca_cert)
}
