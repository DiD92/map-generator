use generator_core::{MapStyle, create_map};

use std::env;

use anyhow::Error;
use rand::Rng;
use serde::{Deserialize, Serialize};

use aws_config::{BehaviorVersion, Region, defaults as aws_config_defaults};
use aws_sdk_s3::{
    Client as S3Client, operation::put_object::PutObjectOutput, primitives::ByteStream,
};
use lambda_runtime::{Diagnostic, LambdaEvent, tracing};

const AWS_REGION: &str = "AWS_REGION";
const BUCKET_NAME: &str = "BUCKET_NAME";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LambdaEventPayload {
    columns: u32,
    rows: u32,
}

impl LambdaEventPayload {
    fn is_valid(&self) -> bool {
        self.columns > 0 && self.rows > 0
    }
}

fn get_random_style() -> MapStyle {
    let mut rng = rand::rng();

    match rng.random_range(0..4) {
        0 => MapStyle::CastlevaniaSOTN,
        1 => MapStyle::CastlevaniaAOS,
        2 => MapStyle::CastlevaniaHOD,
        _ => MapStyle::CastlevaniaCOTM,
    }
}

async fn instantiate_s3_client() -> S3Client {
    let region = env::var(AWS_REGION).expect("AWS_REGION environment variable not set");
    let config = aws_config_defaults(BehaviorVersion::latest())
        .region(Region::new(region))
        .load()
        .await;

    S3Client::new(&config)
}

async fn upload_map(
    client: &S3Client,
    bucket_name: &str,
    key: &str,
    map_bytes: Vec<u8>,
) -> Result<PutObjectOutput, Error> {
    client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(ByteStream::from(map_bytes))
        .send()
        .await
        .map_err(|err| {
            Error::msg(format!(
                "Failed to upload map to S3: {}",
                err.into_service_error()
            ))
        })
}

pub(crate) async fn function_handler(
    event: LambdaEvent<LambdaEventPayload>,
) -> Result<(), Diagnostic> {
    let payload = event.payload;
    tracing::debug!("Payload: {:?}", payload);

    if !payload.is_valid() {
        return Err(
            Error::msg("Invalid payload: columns and/or rows must be greater than 0").into(),
        );
    }

    let LambdaEventPayload { columns, rows } = payload;
    let style = get_random_style();

    tracing::info!(
        "Creating map with dimensions: {}x{}, style: {:?}",
        columns,
        rows,
        style
    );

    let map_bytes = {
        let svg = create_map(columns, rows, style);
        svg.to_string().into_bytes()
    };

    let s3_client = instantiate_s3_client().await;

    tracing::info!("S3 client instantiated successfully");

    let bucket = env::var(BUCKET_NAME)
        .map_err(|_| Error::msg("BUCKET_NAME environment variable not set"))?;

    let key = format!("{}.svg", uuid::Uuid::new_v4());

    tracing::info!("Uploading map to S3 bucket: {}, key: {}", bucket, key);

    upload_map(&s3_client, &bucket, &key, map_bytes).await?;

    tracing::info!("Map uploaded successfully to {}/{}", bucket, key);

    Ok(())
}
