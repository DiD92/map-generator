mod config;
mod domain;
mod inbound;
mod outbound;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::Config::from_env()?;

    // A minimal tracing middleware for request logging.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let repository = outbound::repositories::NullRepository;
    let metrics = outbound::metrics::NullMetrics;
    let notifier = outbound::notifiers::NullNotifier;
    let map_service = domain::service::Service::new(repository, metrics, notifier);

    let server_config = inbound::HttpServerConfig {
        port: &config.server_port,
    };
    let http_server = inbound::HttpServer::new(map_service, server_config).await?;

    http_server.run().await
}
