use std::str::FromStr;

use aws_config::Region;
use aws_sdk_s3::Client as S3Client;
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub s3_client: S3Client,
    pub sf: sonyflake::Sonyflake,
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
                .region(Region::new(
                    std::env::var("AWS_DEFAULT_REGION").expect("AWS_DEFAULT_REGION must be set"),
                ))
                .load()
                .await,
        );

        //2014-09-01 00:00:00 +0000 UTC
        let sf = sonyflake::Sonyflake::builder()
            .start_time(
                chrono::DateTime::<chrono::Utc>::from_str("2025-01-01 00:00:00+0000").unwrap(),
            )
            .finalize()
            .expect("Failed to create Sonyflake");

        Self {
            db_pool: pool,
            s3_client,
            sf,
        }
    }
}
