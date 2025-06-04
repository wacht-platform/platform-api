use sqlx::Row;

use crate::{
    application::{AppState, AppError},
    core::{
        models::{AiTool, AiToolWithDetails, AiToolType, AiToolConfiguration},
        queries::Query,
    },
};

pub struct GetAiToolsQuery {
    pub deployment_id: i64,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub search: Option<String>,
}

impl GetAiToolsQuery {
    pub fn new(deployment_id: i64) -> Self {
        Self {
            deployment_id,
            limit: None,
            offset: None,
            search: None,
        }
    }

    pub fn with_limit(mut self, limit: Option<u32>) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_offset(mut self, offset: Option<u32>) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_search(mut self, search: Option<String>) -> Self {
        self.search = search;
        self
    }
}

impl Query for GetAiToolsQuery {
    type Output = Vec<AiToolWithDetails>;

    async fn execute(&self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let mut query = r#"
            SELECT
                t.id, t.created_at, t.updated_at, t.name, t.description,
                t.tool_type, t.deployment_id, t.configuration,
                COALESCE(a.agents_count, 0) as agents_count
            FROM ai_tools t
            LEFT JOIN (
                SELECT tool_id, COUNT(*) as agents_count
                FROM ai_agent_tools
                GROUP BY tool_id
            ) a ON t.id = a.tool_id
            WHERE t.deployment_id = $1
        "#.to_string();

        let mut param_count = 2;
        if self.search.is_some() {
            query.push_str(&format!(" AND (t.name ILIKE ${} OR t.description ILIKE ${})", param_count, param_count + 1));
            param_count += 2;
        }

        query.push_str(" ORDER BY t.created_at DESC");
        query.push_str(&format!(" LIMIT ${} OFFSET ${}", param_count, param_count + 1));

        let mut query_builder = sqlx::query(&query);
        query_builder = query_builder.bind(self.deployment_id);

        if let Some(search) = &self.search {
            let search_pattern = format!("%{}%", search);
            query_builder = query_builder.bind(search_pattern.clone()).bind(search_pattern);
        }

        query_builder = query_builder
            .bind(self.limit.unwrap_or(50) as i64)
            .bind(self.offset.unwrap_or(0) as i64);

        let tools = query_builder
            .fetch_all(&app_state.db_pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(tools
            .into_iter()
            .map(|row| {
                let tool_type = AiToolType::from(row.get::<String, _>("tool_type"));
                let configuration: AiToolConfiguration = serde_json::from_value(row.get("configuration"))
                    .unwrap_or_default();

                AiToolWithDetails {
                    id: row.get("id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    name: row.get("name"),
                    description: row.get("description"),
                    tool_type,
                    deployment_id: row.get("deployment_id"),
                    configuration,

                }
            })
            .collect())
    }
}

pub struct GetAiToolByIdQuery {
    pub deployment_id: i64,
    pub tool_id: i64,
}

impl GetAiToolByIdQuery {
    pub fn new(deployment_id: i64, tool_id: i64) -> Self {
        Self {
            deployment_id,
            tool_id,
        }
    }
}

impl Query for GetAiToolByIdQuery {
    type Output = AiToolWithDetails;

    async fn execute(&self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let tool = sqlx::query!(
            r#"
            SELECT
                t.id, t.created_at, t.updated_at, t.name, t.description,
                t.tool_type, t.deployment_id, t.configuration,
                COALESCE(a.agents_count, 0) as agents_count
            FROM ai_tools t
            LEFT JOIN (
                SELECT tool_id, COUNT(*) as agents_count
                FROM ai_agent_tools
                GROUP BY tool_id
            ) a ON t.id = a.tool_id
            WHERE t.id = $1 AND t.deployment_id = $2
            "#,
            self.tool_id,
            self.deployment_id
        )
        .fetch_one(&app_state.db_pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        let tool_type = AiToolType::from(tool.tool_type);
        let configuration: AiToolConfiguration = serde_json::from_value(tool.configuration)
            .unwrap_or_default();

        Ok(AiToolWithDetails {
            id: tool.id,
            created_at: tool.created_at,
            updated_at: tool.updated_at,
            name: tool.name,
            description: tool.description,
            tool_type,
            deployment_id: tool.deployment_id,
            configuration,

        })
    }
}
