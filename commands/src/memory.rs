use chrono::{DateTime, Utc};
use common::{
    HasDbRouter, HasEmbeddingProvider, HasEncryptionProvider, HasIdProvider, HasNatsProvider,
    ReadConsistency, error::AppError,
};
use dto::json::agent_executor::{MemorySearchApproach, MemorySource, SearchDepth};
use dto::json::memory::MemoryCategory;
use models::MemoryRecord;
use pgvector::Vector;

use crate::GenerateEmbeddingsCommand;

/// Weight applied per hour of staleness in the recency-adjusted composite
/// score. At 0.0002, a record ages ~0.14 per 30 days — enough to give fresh
/// results a slight edge over equally-similar old ones without overpowering a
/// strong semantic match (the dedup cutoff of 0.35 is well above this).
const DECAY_PER_HOUR: f32 = 0.0002;
/// The base weight comes from `MemoryCategory::retrieval_weight()`, and is
/// applied as a rank adjustment so higher-weighted categories surface above
/// lower-weighted ones at similar relevance, without overwhelming top matches.
const CATEGORY_WEIGHT_STRENGTH: f64 = 0.3;
const RRF_K: f64 = 60.0;

#[derive(Debug)]
struct MemoryRecordRow {
    id: i64,
    deployment_id: i64,
    actor_id: Option<i64>,
    project_id: Option<i64>,
    thread_id: Option<i64>,
    execution_run_id: Option<i64>,
    owner_agent_id: Option<i64>,
    recorded_by_agent_id: Option<i64>,
    memory_scope: String,
    content: String,
    embedding: Vector,
    memory_category: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    distance: Option<f64>,
}

impl From<MemoryRecordRow> for MemoryRecord {
    fn from(row: MemoryRecordRow) -> Self {
        Self {
            id: row.id,
            deployment_id: row.deployment_id,
            actor_id: row.actor_id,
            project_id: row.project_id,
            thread_id: row.thread_id,
            execution_run_id: row.execution_run_id,
            owner_agent_id: row.owner_agent_id,
            recorded_by_agent_id: row.recorded_by_agent_id,
            memory_scope: row.memory_scope,
            content: row.content,
            embedding: Some(row.embedding.to_vec()),
            memory_category: row.memory_category,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
            distance: row.distance.map(|distance| distance as f32),
        }
    }
}

pub struct StoreMemoryCommand {
    pub id: i64,
    pub deployment_id: i64,
    pub actor_id: Option<i64>,
    pub project_id: Option<i64>,
    pub thread_id: Option<i64>,
    pub execution_run_id: Option<i64>,
    pub owner_agent_id: Option<i64>,
    pub recorded_by_agent_id: Option<i64>,
    pub memory_scope: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub memory_category: MemoryCategory,
    pub metadata: serde_json::Value,
}

pub struct SaveAgentMemoryCommand {
    pub deployment_id: i64,
    pub agent_id: i64,
    pub thread_id: i64,
    pub execution_run_id: i64,
    pub actor_id: i64,
    pub project_id: i64,
    pub content: String,
    pub category: Option<String>,
    pub scope: Option<String>,
    pub observation: Option<String>,
    pub signals: Vec<String>,
    pub related: Vec<String>,
}

pub struct LoadAgentMemoryCommand {
    pub deployment_id: i64,
    pub agent_id: i64,
    pub thread_id: i64,
    pub actor_id: i64,
    pub project_id: i64,
    pub query: String,
    pub categories: Vec<MemoryCategory>,
    pub sources: Vec<MemorySource>,
    pub depth: Option<SearchDepth>,
    pub search_approach: MemorySearchApproach,
}

impl StoreMemoryCommand {
    pub async fn execute_with_deps<D>(self, deps: &D) -> Result<MemoryRecord, AppError>
    where
        D: HasDbRouter
            + HasEmbeddingProvider
            + HasEncryptionProvider
            + HasNatsProvider
            + HasIdProvider
            + ?Sized,
    {
        let now = Utc::now();
        let embedding = Vector::from(self.embedding);
        let category = self.memory_category.to_string();

        let row = sqlx::query_as!(
            MemoryRecordRow,
            r#"
            INSERT INTO agent_memories (
                id,
                deployment_id,
                actor_id,
                project_id,
                thread_id,
                execution_run_id,
                owner_agent_id,
                recorded_by_agent_id,
                memory_scope,
                content,
                embedding,
                memory_category,
                metadata,
                created_at,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING
                id,
                deployment_id,
                actor_id,
                project_id,
                thread_id,
                execution_run_id,
                owner_agent_id,
                recorded_by_agent_id,
                memory_scope,
                content,
                embedding as "embedding: Vector",
                memory_category,
                metadata,
                created_at,
                updated_at,
                NULL::double precision as distance
            "#,
            self.id,
            self.deployment_id,
            self.actor_id,
            self.project_id,
            self.thread_id,
            self.execution_run_id,
            self.owner_agent_id,
            self.recorded_by_agent_id,
            self.memory_scope,
            self.content,
            embedding as Vector,
            category,
            self.metadata,
            now,
            now
        )
        .fetch_one(deps.writer_pool())
        .await
        .map_err(AppError::Database)?;

        Ok(row.into())
    }
}

impl SaveAgentMemoryCommand {
    pub async fn execute_with_deps<D>(self, deps: &D) -> Result<MemoryRecord, AppError>
    where
        D: HasDbRouter
            + HasEmbeddingProvider
            + HasEncryptionProvider
            + HasNatsProvider
            + HasIdProvider
            + ?Sized,
    {
        let category_str = self.category.as_deref().unwrap_or("semantic");
        let scope_str = self
            .scope
            .as_deref()
            .unwrap_or(models::memory::scope::PROJECT);

        let category = MemoryCategory::from_str(category_str).unwrap_or(MemoryCategory::Semantic);

        let embeddings = GenerateEmbeddingsCommand::new(vec![self.content.clone()])
            .for_retrieval_document()
            .for_deployment(self.deployment_id)
            .execute_with_deps(deps)
            .await?;

        let embedding = embeddings
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("Failed to generate embedding".to_string()))?;

        let (actor_id, project_id, thread_id, owner_agent_id, memory_scope) = match scope_str {
            models::memory::scope::ACTOR => (
                Some(self.actor_id),
                None,
                None,
                None,
                models::memory::scope::ACTOR.to_string(),
            ),
            models::memory::scope::PROJECT => (
                Some(self.actor_id),
                Some(self.project_id),
                None,
                None,
                models::memory::scope::PROJECT.to_string(),
            ),
            _ => (
                Some(self.actor_id),
                Some(self.project_id),
                Some(self.thread_id),
                Some(self.agent_id),
                models::memory::scope::THREAD.to_string(),
            ),
        };

        let mut metadata_obj = serde_json::Map::new();
        if let Some(observation) = self.observation.as_ref().map(|s| s.trim()) {
            if !observation.is_empty() {
                metadata_obj.insert(
                    "observation".to_string(),
                    serde_json::Value::String(observation.to_string()),
                );
            }
        }
        if !self.signals.is_empty() {
            metadata_obj.insert(
                "signals".to_string(),
                serde_json::Value::Array(
                    self.signals
                        .iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }
        if !self.related.is_empty() {
            metadata_obj.insert(
                "related".to_string(),
                serde_json::Value::Array(
                    self.related
                        .iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }

        StoreMemoryCommand {
            id: deps.id_provider().next_id()? as i64,
            deployment_id: self.deployment_id,
            actor_id,
            project_id,
            thread_id,
            execution_run_id: Some(self.execution_run_id),
            owner_agent_id,
            recorded_by_agent_id: Some(self.agent_id),
            memory_scope,
            content: self.content,
            embedding,
            memory_category: category,
            metadata: serde_json::Value::Object(metadata_obj),
        }
        .execute_with_deps(deps)
        .await
    }
}

pub struct UpdateAgentMemoryCommand {
    pub deployment_id: i64,
    pub memory_id: i64,
    pub actor_id: i64,
    pub project_id: i64,
    pub thread_id: i64,
    pub content: Option<String>,
    pub category: Option<String>,
    pub scope: Option<String>,
    pub observation: Option<String>,
    pub signals: Option<Vec<String>>,
    pub related: Option<Vec<String>>,
}

impl UpdateAgentMemoryCommand {
    pub async fn execute_with_deps<D>(self, deps: &D) -> Result<MemoryRecord, AppError>
    where
        D: HasDbRouter
            + HasEmbeddingProvider
            + HasEncryptionProvider
            + HasNatsProvider
            + HasIdProvider
            + ?Sized,
    {
        let mut tx = deps.writer_pool().begin().await.map_err(AppError::Database)?;

        let existing = sqlx::query_as!(
            MemoryRecordRow,
            r#"
            SELECT
                id,
                deployment_id,
                actor_id,
                project_id,
                thread_id,
                execution_run_id,
                owner_agent_id,
                recorded_by_agent_id,
                memory_scope,
                content,
                embedding as "embedding: Vector",
                memory_category,
                metadata,
                created_at,
                updated_at,
                NULL::double precision as distance
            FROM agent_memories
            WHERE deployment_id = $1 AND id = $2
            FOR UPDATE
            LIMIT 1
            "#,
            self.deployment_id,
            self.memory_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("Memory {} not found", self.memory_id)))?;

        let scope_changed = self
            .scope
            .as_deref()
            .map(|s| s != existing.memory_scope)
            .unwrap_or(false);
        if scope_changed {
            return Err(AppError::Validation(
                "Updating memory_scope is not supported; re-save the memory in the new scope instead."
                    .to_string(),
            ));
        }

        let new_content = self
            .content
            .clone()
            .unwrap_or_else(|| existing.content.clone());
        let content_changed = new_content != existing.content;
        let embedding = if content_changed {
            let embeddings = GenerateEmbeddingsCommand::new(vec![new_content.clone()])
                .for_retrieval_document()
                .for_deployment(self.deployment_id)
                .execute_with_deps(deps)
                .await?;
            embeddings.into_iter().next().ok_or_else(|| {
                AppError::Internal("Failed to generate embedding for updated memory".to_string())
            })?
        } else {
            existing.embedding.to_vec()
        };

        let category = self
            .category
            .as_deref()
            .map(|c| {
                MemoryCategory::from_str(c)
                    .ok_or_else(|| AppError::Validation(format!("Unknown memory category '{}'", c)))
            })
            .transpose()?
            .map(|c| c.to_string())
            .unwrap_or(existing.memory_category.clone());

        let mut metadata_obj = existing
            .metadata
            .as_object()
            .cloned()
            .unwrap_or_else(serde_json::Map::new);

        if let Some(observation) = self.observation.as_ref() {
            let trimmed = observation.trim();
            if trimmed.is_empty() {
                metadata_obj.remove("observation");
            } else {
                metadata_obj.insert(
                    "observation".to_string(),
                    serde_json::Value::String(trimmed.to_string()),
                );
            }
        }
        if let Some(signals) = self.signals.as_ref() {
            if signals.is_empty() {
                metadata_obj.remove("signals");
            } else {
                metadata_obj.insert(
                    "signals".to_string(),
                    serde_json::Value::Array(
                        signals
                            .iter()
                            .map(|s| serde_json::Value::String(s.clone()))
                            .collect(),
                    ),
                );
            }
        }
        if let Some(related) = self.related.as_ref() {
            if related.is_empty() {
                metadata_obj.remove("related");
            } else {
                metadata_obj.insert(
                    "related".to_string(),
                    serde_json::Value::Array(
                        related
                            .iter()
                            .map(|s| serde_json::Value::String(s.clone()))
                            .collect(),
                    ),
                );
            }
        }

        let embedding = Vector::from(embedding);
        let metadata = serde_json::Value::Object(metadata_obj);
        let now = Utc::now();

        let row = sqlx::query_as!(
            MemoryRecordRow,
            r#"
            UPDATE agent_memories
            SET
                content = $3,
                embedding = $4,
                memory_category = $5,
                metadata = $6,
                updated_at = $7
            WHERE deployment_id = $1 AND id = $2
            RETURNING
                id,
                deployment_id,
                actor_id,
                project_id,
                thread_id,
                execution_run_id,
                owner_agent_id,
                recorded_by_agent_id,
                memory_scope,
                content,
                embedding as "embedding: Vector",
                memory_category,
                metadata,
                created_at,
                updated_at,
                NULL::double precision as distance
            "#,
            self.deployment_id,
            self.memory_id,
            new_content,
            embedding as Vector,
            category,
            metadata,
            now
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("Memory {} not found", self.memory_id)))?;

        tx.commit().await.map_err(AppError::Database)?;
        Ok(row.into())
    }
}

impl LoadAgentMemoryCommand {
    pub async fn execute_with_deps<D>(self, deps: &D) -> Result<Vec<MemoryRecord>, AppError>
    where
        D: HasDbRouter
            + HasEmbeddingProvider
            + HasEncryptionProvider
            + HasNatsProvider
            + HasIdProvider
            + ?Sized,
    {
        let limit = match self.depth.unwrap_or(SearchDepth::Moderate) {
            SearchDepth::Shallow => 20,
            SearchDepth::Moderate => 50,
            SearchDepth::Deep => 100,
        };
        let query = self.query.trim().to_string();

        if query.is_empty() {
            return load_recent_memories_from_sources(
                deps,
                self.deployment_id,
                self.thread_id,
                self.actor_id,
                self.project_id,
                &self.sources,
                &self.categories,
                limit,
            )
            .await;
        }

        let filters = build_memory_query_filters(
            self.thread_id,
            self.actor_id,
            self.project_id,
            &self.sources,
            &self.categories,
        );

        match self.search_approach {
            MemorySearchApproach::Semantic => {
                let embedding = build_query_embedding(deps, self.deployment_id, &query).await?;
                let fetch_limit = limit * 3;
                search_memories_semantic(
                    deps,
                    self.deployment_id,
                    &embedding,
                    &filters,
                    fetch_limit,
                    limit,
                )
                .await
            }
            MemorySearchApproach::FullText => {
                let fetch_limit = limit * 3;
                search_memories_full_text(
                    deps,
                    self.deployment_id,
                    &query,
                    &filters,
                    fetch_limit,
                    limit,
                )
                .await
            }
            MemorySearchApproach::Hybrid => {
                let embedding = build_query_embedding(deps, self.deployment_id, &query).await?;
                let fetch_limit = limit * 2;
                search_memories_hybrid(
                    deps,
                    self.deployment_id,
                    &query,
                    &embedding,
                    &filters,
                    fetch_limit,
                    limit,
                )
                .await
            }
        }
    }
}

pub async fn find_similar_memories<D>(
    deps: &D,
    deployment_id: i64,
    thread_id: i64,
    actor_id: i64,
    project_id: i64,
    content: &str,
    limit: usize,
) -> Result<Vec<MemoryRecord>, AppError>
where
    D: HasDbRouter
        + HasEmbeddingProvider
        + HasEncryptionProvider
        + HasNatsProvider
        + HasIdProvider
        + ?Sized,
{
    let embedding = build_query_embedding(deps, deployment_id, content).await?;
    let filters = MemoryQueryFilters {
        actor_id: Some(actor_id),
        project_id: Some(project_id),
        thread_id: Some(thread_id),
        categories: None,
    };
    search_memories_semantic(deps, deployment_id, &embedding, &filters, limit, limit).await
}

pub async fn get_startup_memories<D>(
    deps: &D,
    deployment_id: i64,
    thread_id: i64,
    actor_id: i64,
    limit: usize,
) -> Result<Vec<MemoryRecord>, AppError>
where
    D: HasDbRouter + ?Sized,
{
    let rows = sqlx::query_as!(
        MemoryRecordRow,
        r#"
        SELECT
            id,
            deployment_id,
            actor_id,
            project_id,
            thread_id,
            execution_run_id,
            owner_agent_id,
            recorded_by_agent_id,
            memory_scope,
            content,
            embedding as "embedding: Vector",
            memory_category,
            metadata,
            created_at,
            updated_at,
            NULL::double precision as distance
        FROM agent_memories
        WHERE deployment_id = $1
            AND embedding IS NOT NULL
            AND (
                (thread_id = $2 AND memory_scope = 'thread')
                OR (actor_id = $3 AND memory_scope = 'actor')
            )
        ORDER BY created_at DESC
        LIMIT $4
        "#,
        deployment_id,
        thread_id,
        actor_id,
        limit as i64
    )
    .fetch_all(deps.reader_pool(ReadConsistency::Eventual))
    .await
    .map_err(AppError::Database)?;

    Ok(rows.into_iter().map(Into::into).collect())
}

async fn build_query_embedding<D>(
    deps: &D,
    deployment_id: i64,
    query: &str,
) -> Result<Vec<f32>, AppError>
where
    D: HasDbRouter
        + HasEmbeddingProvider
        + HasEncryptionProvider
        + HasNatsProvider
        + HasIdProvider
        + ?Sized,
{
    let embeddings = GenerateEmbeddingsCommand::new(vec![query.to_string()])
        .for_retrieval_query()
        .for_deployment(deployment_id)
        .execute_with_deps(deps)
        .await?;

    embeddings
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Internal("Failed to generate query embedding".to_string()))
}

struct MemoryQueryFilters {
    actor_id: Option<i64>,
    project_id: Option<i64>,
    thread_id: Option<i64>,
    categories: Option<Vec<String>>,
}

fn build_memory_query_filters(
    thread_id: i64,
    actor_id: i64,
    project_id: i64,
    sources: &[MemorySource],
    categories: &[MemoryCategory],
) -> MemoryQueryFilters {
    MemoryQueryFilters {
        actor_id: sources.contains(&MemorySource::Actor).then_some(actor_id),
        project_id: sources
            .contains(&MemorySource::Project)
            .then_some(project_id),
        thread_id: sources.contains(&MemorySource::Thread).then_some(thread_id),
        categories: (!categories.is_empty()).then(|| {
            categories
                .iter()
                .map(|category| category.to_string())
                .collect::<Vec<_>>()
        }),
    }
}

async fn search_memories_semantic<D>(
    deps: &D,
    deployment_id: i64,
    embedding: &[f32],
    filters: &MemoryQueryFilters,
    fetch_limit: usize,
    final_limit: usize,
) -> Result<Vec<MemoryRecord>, AppError>
where
    D: HasDbRouter + ?Sized,
{
    let query_embedding = Vector::from(embedding.to_vec());
    let categories = filters.categories.clone().unwrap_or_default();
    let category_filter_enabled = filters.categories.is_some();

    let rows = sqlx::query_as!(
        MemoryRecordRow,
        r#"
        WITH candidates AS (
            SELECT
                id,
                deployment_id,
                actor_id,
                project_id,
                thread_id,
                execution_run_id,
                owner_agent_id,
                recorded_by_agent_id,
                memory_scope,
                content,
                embedding,
                memory_category,
                metadata,
                created_at,
                updated_at,
                (embedding <=> $2) as distance
            FROM agent_memories
            WHERE deployment_id = $1
                AND embedding IS NOT NULL
                AND (
                    ($3::bigint IS NOT NULL AND actor_id = $3 AND memory_scope = 'actor')
                    OR ($4::bigint IS NOT NULL AND project_id = $4 AND memory_scope = 'project')
                    OR ($5::bigint IS NOT NULL AND thread_id = $5 AND memory_scope = 'thread')
                )
                AND (NOT $6::boolean OR memory_category = ANY($7::text[]))
            ORDER BY embedding <=> $2
            LIMIT $8
        ), recency_ranked AS (
            SELECT
                *,
                row_number() OVER (
                    ORDER BY distance + ($10::double precision * GREATEST(EXTRACT(EPOCH FROM (NOW() - updated_at)) / 3600.0, 0.0)) ASC
                ) as position,
                count(*) OVER () as total_count
            FROM candidates
        )
        SELECT
            id,
            deployment_id,
            actor_id,
            project_id,
            thread_id,
            execution_run_id,
            owner_agent_id,
            recorded_by_agent_id,
            memory_scope,
            content,
            embedding as "embedding: Vector",
            memory_category,
            metadata,
            created_at,
            updated_at,
            distance
        FROM recency_ranked
        ORDER BY
            (1.0 - ((position - 1)::double precision / GREATEST(total_count, 1)::double precision))
            + ((CASE memory_category
                WHEN 'fact' THEN 1.3
                WHEN 'preference' THEN 1.2
                WHEN 'observation' THEN 1.1
                WHEN 'conversation_summary' THEN 0.9
                ELSE 1.0
            END - 1.0) * $11::double precision) DESC,
            position ASC
        LIMIT $9
        "#,
        deployment_id,
        query_embedding as Vector,
        filters.actor_id,
        filters.project_id,
        filters.thread_id,
        category_filter_enabled,
        &categories,
        fetch_limit as i64,
        final_limit as i64,
        DECAY_PER_HOUR as f64,
        CATEGORY_WEIGHT_STRENGTH
    )
    .fetch_all(deps.reader_pool(ReadConsistency::Eventual))
    .await
    .map_err(AppError::Database)?;

    Ok(rows.into_iter().map(Into::into).collect())
}

async fn search_memories_full_text<D>(
    deps: &D,
    deployment_id: i64,
    query: &str,
    filters: &MemoryQueryFilters,
    fetch_limit: usize,
    final_limit: usize,
) -> Result<Vec<MemoryRecord>, AppError>
where
    D: HasDbRouter + ?Sized,
{
    let categories = filters.categories.clone().unwrap_or_default();
    let category_filter_enabled = filters.categories.is_some();

    let rows = sqlx::query_as!(
        MemoryRecordRow,
        r#"
        WITH candidates AS (
            SELECT
                id,
                deployment_id,
                actor_id,
                project_id,
                thread_id,
                execution_run_id,
                owner_agent_id,
                recorded_by_agent_id,
                memory_scope,
                content,
                embedding,
                memory_category,
                metadata,
                created_at,
                updated_at,
                row_number() OVER (ORDER BY ts_rank_cd(search_vector, websearch_to_tsquery('english', $2)) DESC, created_at DESC) as position,
                count(*) OVER () as total_count
            FROM agent_memories
            WHERE deployment_id = $1
                AND embedding IS NOT NULL
                AND (
                    ($3::bigint IS NOT NULL AND actor_id = $3 AND memory_scope = 'actor')
                    OR ($4::bigint IS NOT NULL AND project_id = $4 AND memory_scope = 'project')
                    OR ($5::bigint IS NOT NULL AND thread_id = $5 AND memory_scope = 'thread')
                )
                AND (NOT $6::boolean OR memory_category = ANY($7::text[]))
                AND search_vector @@ websearch_to_tsquery('english', $2)
            ORDER BY ts_rank_cd(search_vector, websearch_to_tsquery('english', $2)) DESC, created_at DESC
            LIMIT $8
        )
        SELECT
            id,
            deployment_id,
            actor_id,
            project_id,
            thread_id,
            execution_run_id,
            owner_agent_id,
            recorded_by_agent_id,
            memory_scope,
            content,
            embedding as "embedding: Vector",
            memory_category,
            metadata,
            created_at,
            updated_at,
            NULL::double precision as distance
        FROM candidates
        ORDER BY
            (1.0 - ((position - 1)::double precision / GREATEST(total_count, 1)::double precision))
            + ((CASE memory_category
                WHEN 'fact' THEN 1.3
                WHEN 'preference' THEN 1.2
                WHEN 'observation' THEN 1.1
                WHEN 'conversation_summary' THEN 0.9
                ELSE 1.0
            END - 1.0) * $10::double precision) DESC,
            position ASC
        LIMIT $9
        "#,
        deployment_id,
        query,
        filters.actor_id,
        filters.project_id,
        filters.thread_id,
        category_filter_enabled,
        &categories,
        fetch_limit as i64,
        final_limit as i64,
        CATEGORY_WEIGHT_STRENGTH
    )
    .fetch_all(deps.reader_pool(ReadConsistency::Eventual))
    .await
    .map_err(AppError::Database)?;

    Ok(rows.into_iter().map(Into::into).collect())
}

async fn search_memories_hybrid<D>(
    deps: &D,
    deployment_id: i64,
    query: &str,
    embedding: &[f32],
    filters: &MemoryQueryFilters,
    fetch_limit: usize,
    final_limit: usize,
) -> Result<Vec<MemoryRecord>, AppError>
where
    D: HasDbRouter + ?Sized,
{
    let query_embedding = Vector::from(embedding.to_vec());
    let categories = filters.categories.clone().unwrap_or_default();
    let category_filter_enabled = filters.categories.is_some();

    let rows = sqlx::query_as!(
        MemoryRecordRow,
        r#"
        WITH semantic_candidates AS (
            SELECT
                id,
                row_number() OVER (ORDER BY embedding <=> $3) as semantic_rank
            FROM agent_memories
            WHERE deployment_id = $1
                AND embedding IS NOT NULL
                AND (
                    ($4::bigint IS NOT NULL AND actor_id = $4 AND memory_scope = 'actor')
                    OR ($5::bigint IS NOT NULL AND project_id = $5 AND memory_scope = 'project')
                    OR ($6::bigint IS NOT NULL AND thread_id = $6 AND memory_scope = 'thread')
                )
                AND (NOT $7::boolean OR memory_category = ANY($8::text[]))
            ORDER BY embedding <=> $3
            LIMIT $9
        ), text_candidates AS (
            SELECT
                id,
                row_number() OVER (ORDER BY ts_rank_cd(search_vector, websearch_to_tsquery('english', $2)) DESC, created_at DESC) as text_rank
            FROM agent_memories
            WHERE deployment_id = $1
                AND embedding IS NOT NULL
                AND (
                    ($4::bigint IS NOT NULL AND actor_id = $4 AND memory_scope = 'actor')
                    OR ($5::bigint IS NOT NULL AND project_id = $5 AND memory_scope = 'project')
                    OR ($6::bigint IS NOT NULL AND thread_id = $6 AND memory_scope = 'thread')
                )
                AND (NOT $7::boolean OR memory_category = ANY($8::text[]))
                AND search_vector @@ websearch_to_tsquery('english', $2)
            ORDER BY ts_rank_cd(search_vector, websearch_to_tsquery('english', $2)) DESC, created_at DESC
            LIMIT $9
        ), merged AS (
            SELECT
                ids.id,
                COALESCE(1.0 / ($12::double precision + semantic_candidates.semantic_rank::double precision), 0.0)
                    + COALESCE(1.0 / ($12::double precision + text_candidates.text_rank::double precision), 0.0) as rrf_score
            FROM (
                SELECT id FROM semantic_candidates
                UNION
                SELECT id FROM text_candidates
            ) ids
            LEFT JOIN semantic_candidates ON semantic_candidates.id = ids.id
            LEFT JOIN text_candidates ON text_candidates.id = ids.id
        ), ranked AS (
            SELECT
                m.*,
                memories.deployment_id,
                memories.actor_id,
                memories.project_id,
                memories.thread_id,
                memories.execution_run_id,
                memories.owner_agent_id,
                memories.recorded_by_agent_id,
                memories.memory_scope,
                memories.content,
                memories.embedding,
                memories.memory_category,
                memories.metadata,
                memories.created_at,
                memories.updated_at,
                (memories.embedding <=> $3) as distance,
                row_number() OVER (
                    ORDER BY m.rrf_score DESC, memories.id ASC
                ) as rrf_position,
                count(*) OVER () as total_count
            FROM merged m
            JOIN agent_memories memories ON memories.id = m.id
            ORDER BY m.rrf_score DESC, memories.id ASC
            LIMIT $10
        ), recency_ranked AS (
            SELECT
                *,
                row_number() OVER (
                    ORDER BY rrf_position::double precision
                        + ($13::double precision * GREATEST(EXTRACT(EPOCH FROM (NOW() - updated_at)) / 3600.0, 0.0)) ASC
                ) as position
            FROM ranked
        )
        SELECT
            id as "id!",
            deployment_id,
            actor_id,
            project_id,
            thread_id,
            execution_run_id,
            owner_agent_id,
            recorded_by_agent_id,
            memory_scope,
            content,
            embedding as "embedding: Vector",
            memory_category,
            metadata,
            created_at,
            updated_at,
            distance
        FROM recency_ranked
        ORDER BY
            (1.0 - ((position - 1)::double precision / GREATEST(total_count, 1)::double precision))
            + ((CASE memory_category
                WHEN 'fact' THEN 1.3
                WHEN 'preference' THEN 1.2
                WHEN 'observation' THEN 1.1
                WHEN 'conversation_summary' THEN 0.9
                ELSE 1.0
            END - 1.0) * $14::double precision) DESC,
            position ASC
        LIMIT $11
        "#,
        deployment_id,
        query,
        query_embedding as Vector,
        filters.actor_id,
        filters.project_id,
        filters.thread_id,
        category_filter_enabled,
        &categories,
        fetch_limit as i64,
        (final_limit * 3) as i64,
        final_limit as i64,
        RRF_K,
        DECAY_PER_HOUR as f64,
        CATEGORY_WEIGHT_STRENGTH
    )
    .fetch_all(deps.reader_pool(ReadConsistency::Eventual))
    .await
    .map_err(AppError::Database)?;

    Ok(rows.into_iter().map(Into::into).collect())
}

async fn load_recent_memories_from_sources<D>(
    deps: &D,
    deployment_id: i64,
    thread_id: i64,
    actor_id: i64,
    project_id: i64,
    sources: &[MemorySource],
    categories: &[MemoryCategory],
    limit: usize,
) -> Result<Vec<MemoryRecord>, AppError>
where
    D: HasDbRouter + ?Sized,
{
    let deduped_sources = dedupe_sources(sources);
    if deduped_sources.is_empty() {
        return Ok(Vec::new());
    }

    let per_source_limit = std::cmp::max(1, limit.div_ceil(deduped_sources.len()));
    let mut groups = Vec::new();

    for source in deduped_sources {
        groups.push(
            load_recent_memories_for_source(
                deps,
                deployment_id,
                thread_id,
                actor_id,
                project_id,
                source,
                categories,
                per_source_limit,
            )
            .await?,
        );
    }

    Ok(merge_unique_memories(groups, limit))
}

async fn load_recent_memories_for_source<D>(
    deps: &D,
    deployment_id: i64,
    thread_id: i64,
    actor_id: i64,
    project_id: i64,
    source: MemorySource,
    categories: &[MemoryCategory],
    limit: usize,
) -> Result<Vec<MemoryRecord>, AppError>
where
    D: HasDbRouter + ?Sized,
{
    let (scope, source_actor_id, source_project_id, source_thread_id) = match source {
        MemorySource::Thread => (models::memory::scope::THREAD, None, None, Some(thread_id)),
        MemorySource::Project => (models::memory::scope::PROJECT, None, Some(project_id), None),
        MemorySource::Actor => (models::memory::scope::ACTOR, Some(actor_id), None, None),
    };
    let categories = categories
        .iter()
        .map(|category| category.to_string())
        .collect::<Vec<_>>();
    let category_filter_enabled = !categories.is_empty();

    let rows = sqlx::query_as!(
        MemoryRecordRow,
        r#"
        SELECT
            id,
            deployment_id,
            actor_id,
            project_id,
            thread_id,
            execution_run_id,
            owner_agent_id,
            recorded_by_agent_id,
            memory_scope,
            content,
            embedding as "embedding: Vector",
            memory_category,
            metadata,
            created_at,
            updated_at,
            NULL::double precision as distance
        FROM agent_memories
        WHERE deployment_id = $1
            AND embedding IS NOT NULL
            AND memory_scope = $2
            AND ($3::bigint IS NULL OR actor_id = $3)
            AND ($4::bigint IS NULL OR project_id = $4)
            AND ($5::bigint IS NULL OR thread_id = $5)
            AND (NOT $6::boolean OR memory_category = ANY($7::text[]))
        ORDER BY created_at DESC
        LIMIT $8
        "#,
        deployment_id,
        scope,
        source_actor_id,
        source_project_id,
        source_thread_id,
        category_filter_enabled,
        &categories,
        limit as i64
    )
    .fetch_all(deps.reader_pool(ReadConsistency::Eventual))
    .await
    .map_err(AppError::Database)?;

    Ok(rows.into_iter().map(Into::into).collect())
}

fn dedupe_sources(sources: &[MemorySource]) -> Vec<MemorySource> {
    let mut deduped = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for source in sources {
        if seen.insert(*source) {
            deduped.push(*source);
        }
    }

    deduped
}

fn merge_unique_memories(groups: Vec<Vec<MemoryRecord>>, limit: usize) -> Vec<MemoryRecord> {
    let mut merged = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for group in groups {
        for memory in group {
            if seen.insert(memory.id) {
                merged.push(memory);
            }
        }
    }

    merged.truncate(limit);
    merged
}
