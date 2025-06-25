use lambda_runtime::{Error, run, service_fn, tracing};

mod event_handler;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(event_handler::function_handler)).await
}
