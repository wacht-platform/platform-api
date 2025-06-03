use crate::{
    application::{AppState, AppError},
    core::{
        models::{AiKnowledgeBase, AiKnowledgeBaseWithDetails, AiKnowledgeBaseDocument},
        queries::Query,
    },
};

pub struct GetAiKnowledgeBasesQuery {
    pub deployment_id: i64,
    pub limit: usize,
    pub offset: usize,
    pub search: Option<String>,
}

impl GetAiKnowledgeBasesQuery {
    pub fn new(deployment_id: i64, limit: usize, offset: usize) -> Self {
        Self {
            deployment_id,
            limit,
            offset,
            search: None,
        }
    }

    pub fn with_search(mut self, search: String) -> Self {
        self.search = Some(search);
        self
    }
}

impl Query for GetAiKnowledgeBasesQuery {
    type Output = Vec<AiKnowledgeBaseWithDetails>;

    async fn execute(&self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let knowledge_bases = if let Some(search) = &self.search {
            sqlx::query!(
                r#"
                SELECT
                    kb.id, kb.created_at, kb.updated_at, kb.name, kb.description,
                    kb.configuration, kb.deployment_id,
                    COALESCE(d.documents_count, 0) as documents_count,
                    COALESCE(d.total_size, 0) as total_size
                FROM ai_knowledge_bases kb
                LEFT JOIN (
                    SELECT knowledge_base_id, COUNT(*) as documents_count, SUM(file_size) as total_size
                    FROM ai_knowledge_base_documents 
                    GROUP BY knowledge_base_id
                ) d ON kb.id = d.knowledge_base_id
                WHERE kb.deployment_id = $1 
                AND (kb.name ILIKE $2 OR kb.description ILIKE $2)
                ORDER BY kb.created_at DESC
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
                    kb.id, kb.created_at, kb.updated_at, kb.name, kb.description,
                    kb.configuration, kb.deployment_id,
                    COALESCE(d.documents_count, 0) as documents_count,
                    COALESCE(d.total_size, 0) as total_size
                FROM ai_knowledge_bases kb
                LEFT JOIN (
                    SELECT knowledge_base_id, COUNT(*) as documents_count, SUM(file_size) as total_size
                    FROM ai_knowledge_base_documents 
                    GROUP BY knowledge_base_id
                ) d ON kb.id = d.knowledge_base_id
                WHERE kb.deployment_id = $1
                ORDER BY kb.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                self.deployment_id,
                self.limit as i64,
                self.offset as i64
            )
            .fetch_all(&app_state.db_pool)
            .await
        }
        .map_err(|e| AppError::Database(e))?;

        Ok(knowledge_bases
            .into_iter()
            .map(|row| AiKnowledgeBaseWithDetails {
                id: row.id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                name: row.name,
                description: row.description,
                configuration: row.configuration,
                deployment_id: row.deployment_id,
                documents_count: row.documents_count.unwrap_or(0),
                total_size: row.total_size.unwrap_or(0),
            })
            .collect())
    }
}

pub struct GetAiKnowledgeBaseByIdQuery {
    pub deployment_id: i64,
    pub knowledge_base_id: i64,
}

impl GetAiKnowledgeBaseByIdQuery {
    pub fn new(deployment_id: i64, knowledge_base_id: i64) -> Self {
        Self {
            deployment_id,
            knowledge_base_id,
        }
    }
}

impl Query for GetAiKnowledgeBaseByIdQuery {
    type Output = AiKnowledgeBaseWithDetails;

    async fn execute(&self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let knowledge_base = sqlx::query!(
            r#"
            SELECT
                kb.id, kb.created_at, kb.updated_at, kb.name, kb.description,
                kb.configuration, kb.deployment_id,
                COALESCE(d.documents_count, 0) as documents_count,
                COALESCE(d.total_size, 0) as total_size
            FROM ai_knowledge_bases kb
            LEFT JOIN (
                SELECT knowledge_base_id, COUNT(*) as documents_count, SUM(file_size) as total_size
                FROM ai_knowledge_base_documents 
                GROUP BY knowledge_base_id
            ) d ON kb.id = d.knowledge_base_id
            WHERE kb.id = $1 AND kb.deployment_id = $2
            "#,
            self.knowledge_base_id,
            self.deployment_id
        )
        .fetch_one(&app_state.db_pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(AiKnowledgeBaseWithDetails {
            id: knowledge_base.id,
            created_at: knowledge_base.created_at,
            updated_at: knowledge_base.updated_at,
            name: knowledge_base.name,
            description: knowledge_base.description,
            configuration: knowledge_base.configuration,
            deployment_id: knowledge_base.deployment_id,
            documents_count: knowledge_base.documents_count.unwrap_or(0),
            total_size: knowledge_base.total_size.unwrap_or(0),
        })
    }
}

pub struct GetKnowledgeBaseDocumentsQuery {
    pub knowledge_base_id: i64,
    pub limit: usize,
    pub offset: usize,
}

impl GetKnowledgeBaseDocumentsQuery {
    pub fn new(knowledge_base_id: i64, limit: usize, offset: usize) -> Self {
        Self {
            knowledge_base_id,
            limit,
            offset,
        }
    }
}

impl Query for GetKnowledgeBaseDocumentsQuery {
    type Output = Vec<AiKnowledgeBaseDocument>;

    async fn execute(&self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let documents = sqlx::query!(
            r#"
            SELECT
                id, created_at, updated_at, title, description, file_name,
                file_size, file_type, file_url, knowledge_base_id,
                processing_metadata, usage_count
            FROM ai_knowledge_base_documents
            WHERE knowledge_base_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            self.knowledge_base_id,
            self.limit as i64,
            self.offset as i64
        )
        .fetch_all(&app_state.db_pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(documents
            .into_iter()
            .map(|row| AiKnowledgeBaseDocument {
                id: row.id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                title: row.title,
                description: row.description,
                file_name: row.file_name,
                file_size: row.file_size,
                file_type: row.file_type,
                file_url: row.file_url,
                knowledge_base_id: row.knowledge_base_id,
                processing_metadata: row.processing_metadata,
                usage_count: row.usage_count,
            })
            .collect())
    }
}
