use async_trait::async_trait;
use common::error::AppError;
use common::state::AppState;
use common::{search_full_text, search_hybrid, search_vector};
use models::ai_knowledge_base::DocumentChunkSearchResult;
use models::hybrid_search::{FullTextSearchResult, HybridSearchKbResult};
use models::MemoryRecord;
use std::sync::Arc;

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn search_kb_full_text(
        &self,
        kb_ids: &[i64],
        query: &str,
        limit: usize,
    ) -> Result<Vec<FullTextSearchResult>, AppError>;

    async fn search_kb_vector(
        &self,
        kb_ids: &[i64],
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<DocumentChunkSearchResult>, AppError>;

    async fn search_kb_hybrid(
        &self,
        kb_ids: &[i64],
        query: &str,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<HybridSearchKbResult>, AppError>;

    async fn get_startup_memories(
        &self,
        thread_id: i64,
        actor_id: i64,
        limit: usize,
    ) -> Result<Vec<MemoryRecord>, AppError>;
}

pub trait VectorStoreFactory: Send + Sync {
    fn create(&self, deployment_id: i64, embedding_dimension: i32) -> Arc<dyn VectorStore>;
}

pub struct PostgresVectorStore {
    app_state: AppState,
    deployment_id: i64,
    embedding_dimension: i32,
}

impl PostgresVectorStore {
    pub fn new(app_state: AppState, deployment_id: i64, embedding_dimension: i32) -> Self {
        Self {
            app_state,
            deployment_id,
            embedding_dimension,
        }
    }
}

#[async_trait]
impl VectorStore for PostgresVectorStore {
    async fn search_kb_full_text(
        &self,
        kb_ids: &[i64],
        query: &str,
        limit: usize,
    ) -> Result<Vec<FullTextSearchResult>, AppError> {
        search_full_text(&self.app_state, self.deployment_id, kb_ids, query, limit).await
    }

    async fn search_kb_vector(
        &self,
        kb_ids: &[i64],
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<DocumentChunkSearchResult>, AppError> {
        search_vector(
            &self.app_state,
            self.deployment_id,
            kb_ids,
            query_embedding,
            limit,
            self.embedding_dimension,
        )
        .await
    }

    async fn search_kb_hybrid(
        &self,
        kb_ids: &[i64],
        query: &str,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<HybridSearchKbResult>, AppError> {
        search_hybrid(
            &self.app_state,
            self.deployment_id,
            kb_ids,
            query,
            query_embedding,
            limit,
            self.embedding_dimension,
        )
        .await
    }

    async fn get_startup_memories(
        &self,
        thread_id: i64,
        actor_id: i64,
        limit: usize,
    ) -> Result<Vec<MemoryRecord>, AppError> {
        commands::get_startup_memories(
            &self.app_state,
            self.deployment_id,
            thread_id,
            actor_id,
            limit,
        )
        .await
    }
}

pub struct PostgresVectorStoreFactory {
    app_state: AppState,
}

impl PostgresVectorStoreFactory {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

impl VectorStoreFactory for PostgresVectorStoreFactory {
    fn create(&self, deployment_id: i64, embedding_dimension: i32) -> Arc<dyn VectorStore> {
        Arc::new(PostgresVectorStore::new(
            self.app_state.clone(),
            deployment_id,
            embedding_dimension,
        ))
    }
}
