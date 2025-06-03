use crate::{
    application::{AppState, AppError},
    core::{
        commands::{Command, UploadToCdnCommand},
        models::{AiKnowledgeBase, AiKnowledgeBaseDocument},
    },
};
use chrono::Utc;
use sqlx::Row;

pub struct CreateAiKnowledgeBaseCommand {
    pub deployment_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub configuration: serde_json::Value,
}

impl CreateAiKnowledgeBaseCommand {
    pub fn new(
        deployment_id: i64,
        name: String,
        description: Option<String>,
        configuration: serde_json::Value,
    ) -> Self {
        Self {
            deployment_id,
            name,
            description,
            configuration,
        }
    }
}

impl Command for CreateAiKnowledgeBaseCommand {
    type Output = AiKnowledgeBase;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let knowledge_base_id = app_state.sf.next_id()? as i64;
        let now = Utc::now();

        let knowledge_base = sqlx::query!(
            r#"
            INSERT INTO ai_knowledge_bases (id, created_at, updated_at, name, description, deployment_id, configuration)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, created_at, updated_at, name, description, deployment_id, configuration
            "#,
            knowledge_base_id,
            now,
            now,
            self.name,
            self.description,
            self.deployment_id,
            self.configuration,
        )
        .fetch_one(&app_state.db_pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(AiKnowledgeBase {
            id: knowledge_base.id,
            created_at: knowledge_base.created_at,
            updated_at: knowledge_base.updated_at,
            name: knowledge_base.name,
            description: knowledge_base.description,
            deployment_id: knowledge_base.deployment_id,
            configuration: knowledge_base.configuration,
        })
    }
}

pub struct UpdateAiKnowledgeBaseCommand {
    pub deployment_id: i64,
    pub knowledge_base_id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub configuration: Option<serde_json::Value>,
}

impl UpdateAiKnowledgeBaseCommand {
    pub fn new(deployment_id: i64, knowledge_base_id: i64) -> Self {
        Self {
            deployment_id,
            knowledge_base_id,
            name: None,
            description: None,
            configuration: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }



    pub fn with_configuration(mut self, configuration: serde_json::Value) -> Self {
        self.configuration = Some(configuration);
        self
    }
}

impl Command for UpdateAiKnowledgeBaseCommand {
    type Output = AiKnowledgeBase;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let now = Utc::now();

        // Build dynamic query based on provided fields
        let mut query_parts = vec!["updated_at = $1".to_string()];
        let mut param_count = 2;

        if self.name.is_some() {
            query_parts.push(format!("name = ${}", param_count));
            param_count += 1;
        }
        if self.description.is_some() {
            query_parts.push(format!("description = ${}", param_count));
            param_count += 1;
        }

        if self.configuration.is_some() {
            query_parts.push(format!("configuration = ${}", param_count));
            param_count += 1;
        }

        let query = format!(
            r#"
            UPDATE ai_knowledge_bases 
            SET {}
            WHERE id = ${} AND deployment_id = ${}
            RETURNING id, created_at, updated_at, name, description, deployment_id, configuration
            "#,
            query_parts.join(", "),
            param_count,
            param_count + 1
        );

        let mut query_builder = sqlx::query(&query);
        query_builder = query_builder.bind(now);

        if let Some(name) = self.name {
            query_builder = query_builder.bind(name);
        }
        if let Some(description) = self.description {
            query_builder = query_builder.bind(description);
        }

        if let Some(configuration) = self.configuration {
            query_builder = query_builder.bind(configuration);
        }

        query_builder = query_builder.bind(self.knowledge_base_id).bind(self.deployment_id);

        let knowledge_base = query_builder
            .fetch_one(&app_state.db_pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(AiKnowledgeBase {
            id: knowledge_base.get("id"),
            created_at: knowledge_base.get("created_at"),
            updated_at: knowledge_base.get("updated_at"),
            name: knowledge_base.get("name"),
            description: knowledge_base.get("description"),
            deployment_id: knowledge_base.get("deployment_id"),
            configuration: knowledge_base.get("configuration"),
        })
    }
}

pub struct DeleteAiKnowledgeBaseCommand {
    pub deployment_id: i64,
    pub knowledge_base_id: i64,
}

impl DeleteAiKnowledgeBaseCommand {
    pub fn new(deployment_id: i64, knowledge_base_id: i64) -> Self {
        Self {
            deployment_id,
            knowledge_base_id,
        }
    }
}

impl Command for DeleteAiKnowledgeBaseCommand {
    type Output = ();

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let mut tx = app_state.db_pool.begin().await
            .map_err(|e| AppError::Database(e))?;

        // Delete all documents first
        sqlx::query!(
            "DELETE FROM ai_knowledge_base_documents WHERE knowledge_base_id = $1",
            self.knowledge_base_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e))?;

        // Delete the knowledge base
        sqlx::query!(
            "DELETE FROM ai_knowledge_bases WHERE id = $1 AND deployment_id = $2",
            self.knowledge_base_id,
            self.deployment_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e))?;

        tx.commit().await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }
}

pub struct UploadKnowledgeBaseDocumentCommand {
    pub knowledge_base_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub file_name: String,
    pub file_content: Vec<u8>,
    pub file_type: String,
}

impl UploadKnowledgeBaseDocumentCommand {
    pub fn new(
        knowledge_base_id: i64,
        title: String,
        description: Option<String>,
        file_name: String,
        file_content: Vec<u8>,
        file_type: String,
    ) -> Self {
        Self {
            knowledge_base_id,
            title,
            description,
            file_name,
            file_content,
            file_type,
        }
    }
}

impl Command for UploadKnowledgeBaseDocumentCommand {
    type Output = AiKnowledgeBaseDocument;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let document_id = app_state.sf.next_id()? as i64;
        let now = Utc::now();
        let file_size = self.file_content.len() as i64;

        // Upload file to CDN
        let file_path = format!("knowledge-bases/{}/documents/{}", self.knowledge_base_id, self.file_name);
        let file_url = UploadToCdnCommand::new(file_path, self.file_content)
            .execute(app_state)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let document = sqlx::query!(
            r#"
            INSERT INTO ai_knowledge_base_documents
            (id, created_at, updated_at, title, description, file_name, file_size, file_type, file_url, knowledge_base_id, usage_count)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, created_at, updated_at, title, description, file_name, file_size, file_type, file_url, knowledge_base_id, processing_metadata, usage_count
            "#,
            document_id,
            now,
            now,
            self.title,
            self.description,
            self.file_name,
            file_size,
            self.file_type,
            file_url,
            self.knowledge_base_id,
            0i64,
        )
        .fetch_one(&app_state.db_pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(AiKnowledgeBaseDocument {
            id: document.id,
            created_at: document.created_at,
            updated_at: document.updated_at,
            title: document.title,
            description: document.description,
            file_name: document.file_name,
            file_size: document.file_size,
            file_type: document.file_type,
            file_url: document.file_url,
            knowledge_base_id: document.knowledge_base_id,
            processing_metadata: document.processing_metadata,
            usage_count: document.usage_count,
        })
    }
}
