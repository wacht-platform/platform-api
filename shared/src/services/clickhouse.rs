use crate::error::AppError;
use chrono::{DateTime, Utc};
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ClickHouseService {
    client: Client,
}

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct UserEvent {
    pub deployment_id: i64,
    pub user_id: Option<i64>,
    pub event_type: String, // 'signup', 'signin', 'organization_created', 'workspace_created'
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub auth_method: Option<String>, // 'email', 'google', 'github', etc.
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Row)]
struct CountResult {
    count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentSignup {
    pub name: Option<String>,
    pub email: Option<String>,
    pub method: Option<String>,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Row)]
struct RecentSignupRow {
    user_name: Option<String>,
    user_email: Option<String>,
    auth_method: Option<String>,
    timestamp: DateTime<Utc>,
}

impl ClickHouseService {
    pub fn new(url: &str, password: &str) -> Result<Self, AppError> {
        let client = Client::default()
            .with_url(url)
            .with_user("default")
            .with_password(password)
            .with_database("wacht_analytics");

        Ok(Self { client })
    }

    pub async fn init_tables(&self) -> Result<(), AppError> {
        self.create_user_events_table().await?;
        Ok(())
    }

    async fn create_user_events_table(&self) -> Result<(), AppError> {
        let query = r#"
            CREATE TABLE IF NOT EXISTS user_events (
                deployment_id Int64,
                user_id Nullable(Int64),
                event_type String,
                user_name Nullable(String),
                user_email Nullable(String),
                auth_method Nullable(String),
                timestamp DateTime64(3, 'UTC'),
                ip_address Nullable(String),
                INDEX idx_event_type event_type TYPE bloom_filter GRANULARITY 1,
                INDEX idx_user_id user_id TYPE bloom_filter GRANULARITY 1
            ) ENGINE = MergeTree()
            ORDER BY (deployment_id, event_type, timestamp)
            PARTITION BY toYYYYMM(timestamp)
        "#;

        self.client.query(query).execute().await?;
        Ok(())
    }

    pub async fn insert_user_event(&self, event: &UserEvent) -> Result<(), AppError> {
        let mut insert = self.client.insert("user_events")?;
        insert.write(event).await?;
        insert.end().await?;
        Ok(())
    }

    pub async fn get_total_signups(&self, deployment_id: i64) -> Result<i64, AppError> {
        let query = "SELECT count(DISTINCT user_id) as count FROM user_events WHERE deployment_id = ? AND event_type = 'signup' AND user_id IS NOT NULL";

        let result = self
            .client
            .query(query)
            .bind(deployment_id)
            .fetch_one::<CountResult>()
            .await?;

        Ok(result.count)
    }

    pub async fn get_unique_signins(
        &self,
        deployment_id: i64,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<i64, AppError> {
        let query = "SELECT count(DISTINCT user_id) as count FROM user_events WHERE deployment_id = ? AND event_type = 'signin' AND timestamp >= ? AND timestamp <= ? AND user_id IS NOT NULL";

        let result = self
            .client
            .query(query)
            .bind(deployment_id)
            .bind(from.format("%Y-%m-%d %H:%M:%S").to_string())
            .bind(to.format("%Y-%m-%d %H:%M:%S").to_string())
            .fetch_one::<CountResult>()
            .await?;

        Ok(result.count)
    }

    pub async fn get_signups(
        &self,
        deployment_id: i64,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<i64, AppError> {
        let query = "SELECT count(*) as count FROM user_events WHERE deployment_id = ? AND event_type = 'signup' AND timestamp >= ? AND timestamp <= ?";

        let result = self
            .client
            .query(query)
            .bind(deployment_id)
            .bind(from.format("%Y-%m-%d %H:%M:%S").to_string())
            .bind(to.format("%Y-%m-%d %H:%M:%S").to_string())
            .fetch_one::<CountResult>()
            .await?;

        Ok(result.count)
    }

    pub async fn get_organizations_created(
        &self,
        deployment_id: i64,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<i64, AppError> {
        let query = "SELECT count(*) as count FROM user_events WHERE deployment_id = ? AND event_type = 'organization_created' AND timestamp >= ? AND timestamp <= ?";

        let result = self
            .client
            .query(query)
            .bind(deployment_id)
            .bind(from.format("%Y-%m-%d %H:%M:%S").to_string())
            .bind(to.format("%Y-%m-%d %H:%M:%S").to_string())
            .fetch_one::<CountResult>()
            .await?;

        Ok(result.count)
    }

    pub async fn get_workspaces_created(
        &self,
        deployment_id: i64,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<i64, AppError> {
        let query = "SELECT count(*) as count FROM user_events WHERE deployment_id = ? AND event_type = 'workspace_created' AND timestamp >= ? AND timestamp <= ?";

        let result = self
            .client
            .query(query)
            .bind(deployment_id)
            .bind(from.format("%Y-%m-%d %H:%M:%S").to_string())
            .bind(to.format("%Y-%m-%d %H:%M:%S").to_string())
            .fetch_one::<CountResult>()
            .await?;

        Ok(result.count)
    }

    pub async fn get_recent_signups(
        &self,
        deployment_id: i64,
        limit: i32,
    ) -> Result<Vec<RecentSignup>, AppError> {
        let query = "SELECT user_name, user_email, auth_method, timestamp FROM user_events WHERE deployment_id = ? AND event_type = 'signup' ORDER BY timestamp DESC LIMIT ?";

        let rows = self
            .client
            .query(query)
            .bind(deployment_id)
            .bind(limit)
            .fetch_all::<RecentSignupRow>()
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| RecentSignup {
                name: row.user_name,
                email: row.user_email,
                method: row.auth_method,
                date: row.timestamp,
            })
            .collect())
    }


}
