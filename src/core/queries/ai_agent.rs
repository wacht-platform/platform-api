use crate::{
    application::AppState,
    core::models::{AiAgent, AiAgentStatus, AiAgentWithDetails},
};

use super::{Query, QueryError};

pub struct GetAiAgentsQuery {
    pub deployment_id: i64,
    pub offset: u32,
    pub limit: u32,
    pub search: Option<String>,
}

impl GetAiAgentsQuery {
    pub fn new(deployment_id: i64, offset: u32, limit: u32, search: Option<String>) -> Self {
        Self {
            deployment_id,
            offset,
            limit,
            search,
        }
    }
}

#[async_trait::async_trait]
impl Query<Vec<AiAgentWithDetails>> for GetAiAgentsQuery {
    async fn execute(&self, app_state: &AppState) -> Result<Vec<AiAgentWithDetails>, QueryError> {
        let agents = if let Some(search) = &self.search {
            sqlx::query!(
                r#"
                SELECT 
                    a.id, a.created_at, a.updated_at, a.name, a.description, 
                    a.status, a.configuration, a.deployment_id,
                    COALESCE(t.tools_count, 0) as tools_count,
                    COALESCE(w.workflows_count, 0) as workflows_count,
                    COALESCE(k.knowledge_bases_count, 0) as knowledge_bases_count
                FROM ai_agents a
                LEFT JOIN (
                    SELECT agent_id, COUNT(*) as tools_count 
                    FROM ai_agent_tools 
                    GROUP BY agent_id
                ) t ON a.id = t.agent_id
                LEFT JOIN (
                    SELECT agent_id, COUNT(*) as workflows_count 
                    FROM ai_agent_workflows 
                    GROUP BY agent_id
                ) w ON a.id = w.agent_id
                LEFT JOIN (
                    SELECT agent_id, COUNT(*) as knowledge_bases_count 
                    FROM ai_agent_knowledge_bases 
                    GROUP BY agent_id
                ) k ON a.id = k.agent_id
                WHERE a.deployment_id = $1 
                AND (a.name ILIKE $2 OR a.description ILIKE $2)
                ORDER BY a.created_at DESC
                LIMIT $3 OFFSET $4
                "#,
                self.deployment_id,
                format!("%{}%", search),
                self.limit as i64,
                self.offset as i64
            )
            .fetch_all(&app_state.db_pool)
            .await
        } else {
            sqlx::query!(
                r#"
                SELECT 
                    a.id, a.created_at, a.updated_at, a.name, a.description, 
                    a.status, a.configuration, a.deployment_id,
                    COALESCE(t.tools_count, 0) as tools_count,
                    COALESCE(w.workflows_count, 0) as workflows_count,
                    COALESCE(k.knowledge_bases_count, 0) as knowledge_bases_count
                FROM ai_agents a
                LEFT JOIN (
                    SELECT agent_id, COUNT(*) as tools_count 
                    FROM ai_agent_tools 
                    GROUP BY agent_id
                ) t ON a.id = t.agent_id
                LEFT JOIN (
                    SELECT agent_id, COUNT(*) as workflows_count 
                    FROM ai_agent_workflows 
                    GROUP BY agent_id
                ) w ON a.id = w.agent_id
                LEFT JOIN (
                    SELECT agent_id, COUNT(*) as knowledge_bases_count 
                    FROM ai_agent_knowledge_bases 
                    GROUP BY agent_id
                ) k ON a.id = k.agent_id
                WHERE a.deployment_id = $1
                ORDER BY a.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                self.deployment_id,
                self.limit as i64,
                self.offset as i64
            )
            .fetch_all(&app_state.db_pool)
            .await
        }
        .map_err(|e| QueryError::DatabaseError(e.to_string()))?;

        Ok(agents
            .into_iter()
            .map(|row| AiAgentWithDetails {
                id: row.id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                name: row.name,
                description: row.description,
                status: AiAgentStatus::from(row.status),
                configuration: row.configuration,
                deployment_id: row.deployment_id,
                tools_count: row.tools_count.unwrap_or(0),
                workflows_count: row.workflows_count.unwrap_or(0),
                knowledge_bases_count: row.knowledge_bases_count.unwrap_or(0),
            })
            .collect())
    }
}

pub struct GetAiAgentByIdQuery {
    pub deployment_id: i64,
    pub agent_id: i64,
}

impl GetAiAgentByIdQuery {
    pub fn new(deployment_id: i64, agent_id: i64) -> Self {
        Self {
            deployment_id,
            agent_id,
        }
    }
}

#[async_trait::async_trait]
impl Query<AiAgentWithDetails> for GetAiAgentByIdQuery {
    async fn execute(&self, app_state: &AppState) -> Result<AiAgentWithDetails, QueryError> {
        let agent = sqlx::query!(
            r#"
            SELECT 
                a.id, a.created_at, a.updated_at, a.name, a.description, 
                a.status, a.configuration, a.deployment_id,
                COALESCE(t.tools_count, 0) as tools_count,
                COALESCE(w.workflows_count, 0) as workflows_count,
                COALESCE(k.knowledge_bases_count, 0) as knowledge_bases_count
            FROM ai_agents a
            LEFT JOIN (
                SELECT agent_id, COUNT(*) as tools_count 
                FROM ai_agent_tools 
                GROUP BY agent_id
            ) t ON a.id = t.agent_id
            LEFT JOIN (
                SELECT agent_id, COUNT(*) as workflows_count 
                FROM ai_agent_workflows 
                GROUP BY agent_id
            ) w ON a.id = w.agent_id
            LEFT JOIN (
                SELECT agent_id, COUNT(*) as knowledge_bases_count 
                FROM ai_agent_knowledge_bases 
                GROUP BY agent_id
            ) k ON a.id = k.agent_id
            WHERE a.id = $1 AND a.deployment_id = $2
            "#,
            self.agent_id,
            self.deployment_id
        )
        .fetch_one(&app_state.db_pool)
        .await
        .map_err(|e| QueryError::DatabaseError(e.to_string()))?;

        Ok(AiAgentWithDetails {
            id: agent.id,
            created_at: agent.created_at,
            updated_at: agent.updated_at,
            name: agent.name,
            description: agent.description,
            status: AiAgentStatus::from(agent.status),
            configuration: agent.configuration,
            deployment_id: agent.deployment_id,
            tools_count: agent.tools_count.unwrap_or(0),
            workflows_count: agent.workflows_count.unwrap_or(0),
            knowledge_bases_count: agent.knowledge_bases_count.unwrap_or(0),
        })
    }
}
