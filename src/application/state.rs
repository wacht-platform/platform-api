use std::str::FromStr;

use aws_config::Region;
use aws_sdk_cloudfront::Client as CloudFrontClient;
use aws_sdk_s3::Client as S3Client;
use redis::Client as RedisClient;
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub s3_client: S3Client,
    pub sf: sonyflake::Sonyflake,
    pub redis_client: RedisClient,
}

impl AppState {
    pub async fn new_from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let s3_client = S3Client::new(
            &aws_config::from_env()
                .endpoint_url("https://65983e7602b77f53fde8372f51933eb0.r2.cloudflarestorage.com")
                .region(Region::new(
                    std::env::var("AWS_DEFAULT_REGION").expect("AWS_DEFAULT_REGION must be set"),
                ))
                .load()
                .await,
        );

        let sf = sonyflake::Sonyflake::builder()
            .start_time(
                chrono::DateTime::<chrono::Utc>::from_str("2025-01-01 00:00:00+0000").unwrap(),
            )
            .finalize()
            .expect("Failed to create Sonyflake");

        let redis_client =
            RedisClient::open(std::env::var("REDIS_URL").expect("REDIS_URL must be set"))
                .expect("Failed to create Redis client");

        Self {
            db_pool: pool,
            s3_client,
            sf,
            redis_client,
        }
    }
}
