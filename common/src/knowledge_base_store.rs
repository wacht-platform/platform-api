use models::ai_knowledge_base::DocumentChunkSearchResult;
use models::error::AppError;
use models::hybrid_search::{FullTextSearchResult, HybridSearchKbResult};
use pgvector::Vector;

use crate::HasDbRouter;

const KNOWLEDGE_EMBEDDING_DIMENSION: i32 = 1536;
const KNOWLEDGE_EMBEDDING_DIMENSION_768: i32 = 768;
const RRF_K: f64 = 60.0;

#[derive(Debug, Clone)]
pub struct KnowledgeBaseChunkRecord {
    pub knowledge_base_id: i64,
    pub document_id: i64,
    pub chunk_index: i32,
    pub path: String,
    pub title: String,
    pub description: Option<String>,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
}

fn embedding_column(embedding_dimension: i32) -> Result<&'static str, AppError> {
    match embedding_dimension {
        KNOWLEDGE_EMBEDDING_DIMENSION => Ok("embedding"),
        KNOWLEDGE_EMBEDDING_DIMENSION_768 => Ok("embedding_768"),
        dimension => Err(AppError::Validation(format!(
            "Unsupported knowledge base embedding dimension: {}",
            dimension
        ))),
    }
}

pub async fn replace_document_chunks<D>(
    deps: &D,
    document_id: i64,
    chunks: &[KnowledgeBaseChunkRecord],
    embedding_dimension: i32,
) -> Result<(), AppError>
where
    D: HasDbRouter + ?Sized,
{
    let column = embedding_column(embedding_dimension)?;
    let mut tx = deps
        .writer_pool()
        .begin()
        .await
        .map_err(AppError::Database)?;

    sqlx::query("DELETE FROM knowledge_base_chunks WHERE document_id = $1")
        .bind(document_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

    let insert_sql = format!(
        "INSERT INTO knowledge_base_chunks (knowledge_base_id, document_id, chunk_index, path, title, description, content, {}, {}) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        column,
        if column == "embedding" {
            "embedding_768"
        } else {
            "embedding"
        }
    );

    for chunk in chunks {
        let embedding = chunk.embedding.clone().map(Vector::from);
        let (embedding_1536, embedding_768) = if column == "embedding" {
            (embedding, None)
        } else {
            (None, embedding)
        };

        sqlx::query(&insert_sql)
            .bind(chunk.knowledge_base_id)
            .bind(chunk.document_id)
            .bind(chunk.chunk_index)
            .bind(&chunk.path)
            .bind(&chunk.title)
            .bind(&chunk.description)
            .bind(&chunk.content)
            .bind(embedding_1536)
            .bind(embedding_768)
            .execute(&mut *tx)
            .await
            .map_err(AppError::Database)?;
    }

    tx.commit().await.map_err(AppError::Database)
}

pub async fn delete_document_chunks<D>(deps: &D, document_id: i64) -> Result<(), AppError>
where
    D: HasDbRouter + ?Sized,
{
    sqlx::query("DELETE FROM knowledge_base_chunks WHERE document_id = $1")
        .bind(document_id)
        .execute(deps.writer_pool())
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn delete_knowledge_base_chunks<D>(
    deps: &D,
    knowledge_base_id: i64,
) -> Result<(), AppError>
where
    D: HasDbRouter + ?Sized,
{
    sqlx::query("DELETE FROM knowledge_base_chunks WHERE knowledge_base_id = $1")
        .bind(knowledge_base_id)
        .execute(deps.writer_pool())
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn search_full_text<D>(
    deps: &D,
    deployment_id: i64,
    knowledge_base_ids: &[i64],
    query: &str,
    limit: usize,
) -> Result<Vec<FullTextSearchResult>, AppError>
where
    D: HasDbRouter + ?Sized,
{
    if knowledge_base_ids.is_empty() || query.trim().is_empty() {
        return Ok(Vec::new());
    }

    sqlx::query_as::<_, FullTextSearchResult>(
        r#"
        SELECT
            document_id,
            knowledge_base_id,
            chunk_index,
            content,
            ts_rank_cd(search_vector, websearch_to_tsquery('english', $3))::double precision AS text_rank,
            title AS document_title,
            description AS document_description
        FROM knowledge_base_chunks
            WHERE knowledge_base_id = ANY($2::bigint[])
          AND EXISTS (
              SELECT 1 FROM ai_knowledge_bases
              WHERE ai_knowledge_bases.id = knowledge_base_chunks.knowledge_base_id
                AND ai_knowledge_bases.deployment_id = $1
          )
          AND search_vector @@ websearch_to_tsquery('english', $3)
        ORDER BY text_rank DESC, document_id ASC, chunk_index ASC
        LIMIT $4
        "#,
    )
    .bind(deployment_id)
    .bind(knowledge_base_ids)
    .bind(query)
    .bind(limit as i64)
    .fetch_all(deps.reader_pool(crate::ReadConsistency::Eventual))
    .await
    .map_err(AppError::Database)
}

pub async fn search_vector<D>(
    deps: &D,
    deployment_id: i64,
    knowledge_base_ids: &[i64],
    query_embedding: &[f32],
    limit: usize,
    embedding_dimension: i32,
) -> Result<Vec<DocumentChunkSearchResult>, AppError>
where
    D: HasDbRouter + ?Sized,
{
    if knowledge_base_ids.is_empty() {
        return Ok(Vec::new());
    }
    let column = embedding_column(embedding_dimension)?;
    let sql = format!(
        r#"
        SELECT
            document_id,
            knowledge_base_id,
            content,
            (1.0 - ({} <=> $3))::double precision AS score,
            chunk_index,
            title AS document_title,
            description AS document_description
        FROM knowledge_base_chunks
        WHERE knowledge_base_id = ANY($2::bigint[])
          AND EXISTS (
              SELECT 1 FROM ai_knowledge_bases
              WHERE ai_knowledge_bases.id = knowledge_base_chunks.knowledge_base_id
                AND ai_knowledge_bases.deployment_id = $1
          )
          AND {} IS NOT NULL
        ORDER BY {} <=> $3, document_id ASC, chunk_index ASC
        LIMIT $4
        "#,
        column, column, column
    );

    sqlx::query_as::<_, DocumentChunkSearchResult>(&sql)
        .bind(deployment_id)
        .bind(knowledge_base_ids)
        .bind(Vector::from(query_embedding.to_vec()))
        .bind(limit as i64)
        .fetch_all(deps.reader_pool(crate::ReadConsistency::Eventual))
        .await
        .map_err(AppError::Database)
}

pub async fn search_hybrid<D>(
    deps: &D,
    deployment_id: i64,
    knowledge_base_ids: &[i64],
    query: &str,
    query_embedding: &[f32],
    limit: usize,
    embedding_dimension: i32,
) -> Result<Vec<HybridSearchKbResult>, AppError>
where
    D: HasDbRouter + ?Sized,
{
    if knowledge_base_ids.is_empty() || query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let column = embedding_column(embedding_dimension)?;
    let sql = format!(
        r#"
        WITH semantic_candidates AS (
            SELECT id, row_number() OVER (ORDER BY {0} <=> $4, id ASC) AS semantic_rank
            FROM knowledge_base_chunks
            WHERE knowledge_base_id = ANY($2::bigint[])
              AND EXISTS (
                  SELECT 1 FROM ai_knowledge_bases
                  WHERE ai_knowledge_bases.id = knowledge_base_chunks.knowledge_base_id
                    AND ai_knowledge_bases.deployment_id = $1
              )
              AND {0} IS NOT NULL
            ORDER BY {0} <=> $4, id ASC
            LIMIT $5
        ), text_candidates AS (
            SELECT
                id,
                row_number() OVER (
                    ORDER BY ts_rank_cd(search_vector, websearch_to_tsquery('english', $3)) DESC, id ASC
                ) AS text_rank,
                ts_rank_cd(search_vector, websearch_to_tsquery('english', $3))::double precision AS raw_text_rank
            FROM knowledge_base_chunks
            WHERE knowledge_base_id = ANY($2::bigint[])
              AND EXISTS (
                  SELECT 1 FROM ai_knowledge_bases
                  WHERE ai_knowledge_bases.id = knowledge_base_chunks.knowledge_base_id
                    AND ai_knowledge_bases.deployment_id = $1
              )
              AND search_vector @@ websearch_to_tsquery('english', $3)
            ORDER BY raw_text_rank DESC, id ASC
            LIMIT $5
        ), merged AS (
            SELECT
                ids.id,
                COALESCE(1.0 / ($6::double precision + semantic_candidates.semantic_rank), 0.0)
                    + COALESCE(1.0 / ($6::double precision + text_candidates.text_rank), 0.0) AS combined_score
            FROM (
                SELECT id FROM semantic_candidates
                UNION
                SELECT id FROM text_candidates
            ) ids
            LEFT JOIN semantic_candidates ON semantic_candidates.id = ids.id
            LEFT JOIN text_candidates ON text_candidates.id = ids.id
        )
        SELECT
            chunks.document_id,
            chunks.knowledge_base_id,
            chunks.chunk_index,
            chunks.content,
            chunks.title AS document_title,
            chunks.description AS document_description,
            (1.0 - (chunks.{0} <=> $4))::double precision AS vector_similarity,
            COALESCE(text_candidates.raw_text_rank, 0.0)::double precision AS text_rank,
            merged.combined_score::double precision AS combined_score
        FROM merged
        JOIN knowledge_base_chunks chunks ON chunks.id = merged.id
        LEFT JOIN text_candidates ON text_candidates.id = merged.id
        ORDER BY merged.combined_score DESC, chunks.id ASC
        LIMIT $5
        "#,
        column
    );

    sqlx::query_as::<_, HybridSearchKbResult>(&sql)
        .bind(deployment_id)
        .bind(knowledge_base_ids)
        .bind(query)
        .bind(Vector::from(query_embedding.to_vec()))
        .bind(limit as i64)
        .bind(RRF_K)
        .fetch_all(deps.reader_pool(crate::ReadConsistency::Eventual))
        .await
        .map_err(AppError::Database)
}
