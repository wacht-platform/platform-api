use chrono::Utc;
use serde_json::json;

use crate::{
    application::{AppError, AppState},
    core::models::{
        AiAgent, AiAgentStatus, AiAgentWithDetails, CreateAiAgentRequest, UpdateAiAgentRequest,
    },
};

use super::Command;

pub struct CreateAiAgentCommand {
    pub deployment_id: i64,
    pub request: CreateAiAgentRequest,
}

impl CreateAiAgentCommand {
    pub fn new(deployment_id: i64, request: CreateAiAgentRequest) -> Self {
        Self {
            deployment_id,
            request,
        }
    }
}

impl Command for CreateAiAgentCommand {
    type Output = AiAgentWithDetails;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let now = Utc::now();
        let agent_id = app_state.sf.next_id()? as i64;

        // Insert the AI agent
        let agent_row = sqlx::query!(
            r#"
            INSERT INTO ai_agents (
                id, created_at, updated_at, name, description, status,
                configuration, deployment_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, created_at, updated_at, name, description, status,
                     configuration, deployment_id
            "#,
            agent_id,
            now,
            now,
            self.request.name,
            self.request.description,
            "draft",
            self.request.configuration,
            self.deployment_id
        )
        .fetch_one(&app_state.db_pool)
        .await?;

        // TODO: Insert relationships with tools, workflows, and knowledge bases
        // This would involve junction tables like:
        // - ai_agent_tools
        // - ai_agent_workflows
        // - ai_agent_knowledge_bases

        Ok(AiAgentWithDetails {
            id: agent_row.id,
            created_at: agent_row.created_at,
            updated_at: agent_row.updated_at,
            name: agent_row.name,
            description: agent_row.description,
            status: AiAgentStatus::from(agent_row.status),
            configuration: agent_row.configuration,
            deployment_id: agent_row.deployment_id,
            tools_count: 0,
            workflows_count: 0,
            knowledge_bases_count: 0,
        })
    }
}

pub struct UpdateAiAgentCommand {
    pub deployment_id: i64,
    pub agent_id: i64,
    pub request: UpdateAiAgentRequest,
}

impl UpdateAiAgentCommand {
    pub fn new(deployment_id: i64, agent_id: i64, request: UpdateAiAgentRequest) -> Self {
        Self {
            deployment_id,
            agent_id,
            request,
        }
    }
}

#[async_trait::async_trait]
impl Command<AiAgentWithDetails> for UpdateAiAgentCommand {
    async fn execute(&self, app_state: &AppState) -> Result<AiAgentWithDetails, CommandError> {
        let now = Utc::now();

        // Build dynamic update query based on provided fields
        let mut query = "UPDATE ai_agents SET updated_at = $1".to_string();
        let mut param_count = 2;
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> =
            vec![Box::new(now)];

        if let Some(name) = &self.request.name {
            query.push_str(&format!(", name = ${}", param_count));
            params.push(Box::new(name.clone()));
            param_count += 1;
        }

        if let Some(description) = &self.request.description {
            query.push_str(&format!(", description = ${}", param_count));
            params.push(Box::new(description.clone()));
            param_count += 1;
        }

        if let Some(status) = &self.request.status {
            query.push_str(&format!(", status = ${}", param_count));
            params.push(Box::new(format!("{:?}", status).to_lowercase()));
            param_count += 1;
        }

        if let Some(configuration) = &self.request.configuration {
            query.push_str(&format!(", configuration = ${}", param_count));
            params.push(Box::new(configuration.clone()));
            param_count += 1;
        }

        query.push_str(&format!(
            " WHERE id = ${} AND deployment_id = ${} RETURNING id, created_at, updated_at, name, description, status, configuration, deployment_id",
            param_count, param_count + 1
        ));
        params.push(Box::new(self.agent_id));
        params.push(Box::new(self.deployment_id));

        // For now, return a placeholder - in a real implementation, you'd execute the dynamic query
        // This is a simplified version that assumes name update
        let agent_row = sqlx::query!(
            r#"
            UPDATE ai_agents
            SET updated_at = $1, name = COALESCE($2, name), description = COALESCE($3, description)
            WHERE id = $4 AND deployment_id = $5
            RETURNING id, created_at, updated_at, name, description, status,
                     configuration, deployment_id
            "#,
            now,
            self.request.name,
            self.request.description,
            self.agent_id,
            self.deployment_id
        )
        .fetch_one(&app_state.db_pool)
        .await
        .map_err(|e| CommandError::DatabaseError(e.to_string()))?;

        Ok(AiAgentWithDetails {
            id: agent_row.id,
            created_at: agent_row.created_at,
            updated_at: agent_row.updated_at,
            name: agent_row.name,
            description: agent_row.description,
            status: AiAgentStatus::from(agent_row.status),
            configuration: agent_row.configuration,
            deployment_id: agent_row.deployment_id,
            tools_count: 0, // TODO: Calculate from junction tables
            workflows_count: 0,
            knowledge_bases_count: 0,
        })
    }
}

pub struct DeleteAiAgentCommand {
    pub deployment_id: i64,
    pub agent_id: i64,
}

impl DeleteAiAgentCommand {
    pub fn new(deployment_id: i64, agent_id: i64) -> Self {
        Self {
            deployment_id,
            agent_id,
        }
    }
}

#[async_trait::async_trait]
impl Command<()> for DeleteAiAgentCommand {
    async fn execute(&self, app_state: &AppState) -> Result<(), CommandError> {
        sqlx::query!(
            "DELETE FROM ai_agents WHERE id = $1 AND deployment_id = $2",
            self.agent_id,
            self.deployment_id
        )
        .execute(&app_state.db_pool)
        .await
        .map_err(|e| CommandError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
