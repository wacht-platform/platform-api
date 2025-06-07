use axum::extract::{Path, Query, State};

use crate::{
    application::{AppError, HttpState, response::ApiResult},
    core::{
        dto::json::ai_knowledge_base::{
            KnowledgeBaseSearchResult, SearchKnowledgeBaseQuery, SearchKnowledgeBaseResponse,
        },
        queries::{Query as QueryTrait, ai_knowledge_base::GetAiKnowledgeBaseByIdQuery},
        services::qdrant::QdrantService,
    },
};

/// Search across knowledge bases for a deployment
pub async fn search_knowledge_base(
    Path(deployment_id): Path<i64>,
    Query(params): Query<SearchKnowledgeBaseQuery>,
    State(app_state): State<HttpState>,
) -> ApiResult<SearchKnowledgeBaseResponse> {
    let limit = params.limit.unwrap_or(10).min(100); // Cap at 100 results

    // Generate embedding for the search query
    let query_embedding = app_state
        .embedding_service
        .generate_embeddings(vec![params.query.clone()])
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Internal("Failed to generate query embedding".to_string()))?;

    let results = if let Some(kb_id) = params.knowledge_base_id {
        // Search specific knowledge base
        let _kb = GetAiKnowledgeBaseByIdQuery::new(deployment_id, kb_id)
            .execute(&app_state)
            .await
            .map_err(|_| AppError::NotFound("Knowledge base not found".to_string()))?;

        QdrantService::search_similar(query_embedding, limit, kb_id).await?
    } else {
        // For searching across all knowledge bases in a deployment, we need to get all KB IDs
        // and search each one, then combine results. For now, return an error.
        return Err(AppError::BadRequest(
            "Please specify a knowledge_base_id parameter for search".to_string(),
        )
        .into());
    };

    let search_results: Vec<KnowledgeBaseSearchResult> = results
        .into_iter()
        .map(KnowledgeBaseSearchResult::from)
        .collect();

    let total_results = search_results.len();

    Ok(SearchKnowledgeBaseResponse {
        results: search_results,
        total_results,
        query: params.query,
    }
    .into())
}

/// Search within a specific knowledge base
pub async fn search_specific_knowledge_base(
    Path((deployment_id, knowledge_base_id)): Path<(i64, i64)>,
    Query(params): Query<SearchKnowledgeBaseQuery>,
    State(app_state): State<HttpState>,
) -> ApiResult<SearchKnowledgeBaseResponse> {
    // Verify the knowledge base exists and belongs to the deployment
    let _kb = GetAiKnowledgeBaseByIdQuery::new(deployment_id, knowledge_base_id)
        .execute(&app_state)
        .await
        .map_err(|_| AppError::NotFound("Knowledge base not found".to_string()))?;

    let limit = params.limit.unwrap_or(10).min(100); // Cap at 100 results

    // Generate embedding for the search query
    let query_embedding = app_state
        .embedding_service
        .generate_embeddings(vec![params.query.clone()])
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Internal("Failed to generate query embedding".to_string()))?;

    let results = QdrantService::search_similar(query_embedding, limit, knowledge_base_id).await?;

    let search_results: Vec<KnowledgeBaseSearchResult> = results
        .into_iter()
        .map(KnowledgeBaseSearchResult::from)
        .collect();

    let total_results = search_results.len();

    Ok(SearchKnowledgeBaseResponse {
        results: search_results,
        total_results,
        query: params.query,
    }
    .into())
}
