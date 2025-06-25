use generator_core::{MapStyle, create_map};

use std::env;

use anyhow::Error;

use aws_config::{BehaviorVersion, Region, defaults as aws_config_defaults};
use aws_lambda_events::event::eventbridge::EventBridgeEvent;
use aws_sdk_s3::{
    Client as S3Client, operation::put_object::PutObjectOutput, primitives::ByteStream,
};
use lambda_runtime::{Diagnostic, LambdaEvent, tracing};

const AWS_REGION: &str = "AWS_REGION";
const BUCKET_NAME: &str = "BUCKET_NAME";

pub(crate) async fn function_handler(
    event: LambdaEvent<EventBridgeEvent>,
) -> Result<(), Diagnostic> {
    // Extract some useful information from the request
    let payload = event.payload;
    tracing::info!("Payload: {:?}", payload);

    let region =
        env::var(AWS_REGION).map_err(|_| Error::msg("AWS_REGION environment variable not set"))?;

    let config = aws_config_defaults(BehaviorVersion::latest())
        .region(Region::new(region))
        .load()
        .await;

    let s3 = S3Client::new(&config);

    let map_bytes = {
        let svg = create_map(48, 27, MapStyle::CastlevaniaSOTN);
        svg.to_string().into_bytes()
    };

    let bucket = env::var(BUCKET_NAME)
        .map_err(|_| Error::msg("BUCKET_NAME environment variable not set"))?;

    let key = format!("{}.svg", uuid::Uuid::new_v4());

    tracing::info!("Uploading map to S3 bucket: {}, key: {}", bucket, key);

    upload_map(&s3, &bucket, &key, map_bytes).await?;

    Ok(())
}

async fn upload_map(
    client: &aws_sdk_s3::Client,
    bucket_name: &str,
    key: &str,
    map_bytes: Vec<u8>,
) -> Result<PutObjectOutput, Error> {
    let body = ByteStream::from(map_bytes);

    client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body)
        .send()
        .await
        .map_err(Error::msg)
}
