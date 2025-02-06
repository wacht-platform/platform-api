use super::Query;
use crate::{
    application::{AppError, AppState},
    core::models::UserWithIdentifiers,
};
use sqlx::{QueryBuilder, Row};

pub struct DeploymentUserListQuery {
    offset: i64,
    sort_key: Option<String>,
    sort_order: Option<String>,
    limit: i32,
    deployment_id: i64,
    disabled: bool,
    invited: bool,
}

impl DeploymentUserListQuery {
    pub fn new(id: i64) -> Self {
        Self {
            offset: 0,
            sort_key: None,
            sort_order: None,
            limit: 10,
            deployment_id: id,
            disabled: false,
            invited: false,
        }
    }

    pub fn offset(self, offset: i64) -> Self {
        Self { offset, ..self }
    }

    pub fn limit(self, limit: i32) -> Self {
        Self { limit, ..self }
    }

    pub fn sort_key(self, sort_key: Option<String>) -> Self {
        Self { sort_key, ..self }
    }

    pub fn sort_order(self, sort_order: Option<String>) -> Self {
        Self { sort_order, ..self }
    }

    pub fn disabled(self, disabled: bool) -> Self {
        Self { disabled, ..self }
    }

    pub fn invited(self, invited: bool) -> Self {
        Self { invited, ..self }
    }
}

impl Query<Vec<UserWithIdentifiers>> for DeploymentUserListQuery {
    async fn execute(&self, app_state: &AppState) -> Result<Vec<UserWithIdentifiers>, AppError> {
        println!("disbaled {:?}", self.disabled);
        println!("invited {:?}", self.invited);

        let mut query = QueryBuilder::new(
            r#"
            SELECT
                u.id, u.created_at, u.updated_at, u.deleted_at,
                u.first_name, u.last_name, u.username,
                e.email as primary_email_address,
                p.phone_number as primary_phone_number
            FROM users u
            LEFT JOIN user_email_addresses e ON u.primary_email_address_id = e.id AND e.deleted_at IS NULL
            LEFT JOIN user_phone_numbers p ON u.primary_phone_number_id = p.id AND p.deleted_at IS NULL
            WHERE u.deployment_id = "#,
        );
        query.push_bind(self.deployment_id);
        query.push(" AND u.deleted_at IS NULL");

        let sort_key = self.sort_key.as_deref().unwrap_or("created_at");
        let sort_order = self.sort_order.as_deref().unwrap_or("desc");

        query.push(" ORDER BY ");
        match sort_key {
            "created_at" => query.push("u.created_at"),
            "username" => query.push("u.username"),
            "email" => query.push("e.email"),
            "phone_number" => query.push("p.phone_number"),
            _ => query.push("u.created_at"),
        };

        query.push(match sort_order.to_lowercase().as_str() {
            "asc" => " ASC",
            _ => " DESC",
        });

        query.push(" OFFSET ");
        query.push_bind(self.offset);

        query.push(" LIMIT ");
        query.push_bind(self.limit);

        let rows = query.build().fetch_all(&app_state.pool).await?;

        let users = rows
            .into_iter()
            .map(|row| UserWithIdentifiers {
                id: row.get("id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                deleted_at: row.get("deleted_at"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                username: row.get("username"),
                primary_email_address: row.get("primary_email_address"),
                primary_phone_number: row.get("primary_phone_number"),
            })
            .collect();

        Ok(users)
    }
}
