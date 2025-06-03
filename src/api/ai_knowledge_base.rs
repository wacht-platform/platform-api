use axum::{
    extract::{Json, Multipart, Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{
    application::{response::ApiResult, AppState, AppError},
    core::{
        commands::{
            Command, CreateAiKnowledgeBaseCommand, UpdateAiKnowledgeBaseCommand,
            DeleteAiKnowledgeBaseCommand, UploadKnowledgeBaseDocumentCommand,
        },
        models::{AiKnowledgeBase, AiKnowledgeBaseWithDetails, AiKnowledgeBaseDocument},
        queries::{
            GetAiKnowledgeBasesQuery, GetAiKnowledgeBaseByIdQuery, GetKnowledgeBaseDocumentsQuery,
            Query as QueryTrait,
        },
    },
};

#[derive(Deserialize)]
pub struct GetKnowledgeBasesQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub search: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateKnowledgeBaseRequest {
    pub name: String,
    pub description: Option<String>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct UpdateKnowledgeBaseRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct KnowledgeBaseResponse {
    pub data: Vec<AiKnowledgeBaseWithDetails>,
    pub has_more: bool,
}

pub async fn get_ai_knowledge_bases(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Query(query): Query<GetKnowledgeBasesQuery>,
) -> ApiResult<KnowledgeBaseResponse> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let mut query_builder = GetAiKnowledgeBasesQuery::new(deployment_id, limit + 1, offset);

    if let Some(search) = query.search {
        query_builder = query_builder.with_search(search);
    }

    let mut knowledge_bases = query_builder.execute(&app_state).await.map_err(|e| AppError::from(e))?;

    let has_more = knowledge_bases.len() > limit;
    if has_more {
        knowledge_bases.pop();
    }

    Ok(KnowledgeBaseResponse {
        data: knowledge_bases,
        has_more,
    }
    .into())
}

pub async fn create_ai_knowledge_base(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Json(request): Json<CreateKnowledgeBaseRequest>,
) -> ApiResult<AiKnowledgeBase> {
    let configuration = request.configuration.unwrap_or(serde_json::json!({}));

    CreateAiKnowledgeBaseCommand::new(
        deployment_id,
        request.name,
        request.description,
        configuration,
    )
    .execute(&app_state)
    .await
    .map(Into::into)
    .map_err(Into::into)
}

pub async fn get_ai_knowledge_base_by_id(
    State(app_state): State<AppState>,
    Path((deployment_id, kb_id)): Path<(i64, i64)>,
) -> ApiResult<AiKnowledgeBaseWithDetails> {
    GetAiKnowledgeBaseByIdQuery::new(deployment_id, kb_id)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(|e| AppError::from(e).into())
}

pub async fn update_ai_knowledge_base(
    State(app_state): State<AppState>,
    Path((deployment_id, kb_id)): Path<(i64, i64)>,
    Json(request): Json<UpdateKnowledgeBaseRequest>,
) -> ApiResult<AiKnowledgeBase> {
    let mut command = UpdateAiKnowledgeBaseCommand::new(deployment_id, kb_id);

    if let Some(name) = request.name {
        command = command.with_name(name);
    }

    if let Some(description) = request.description {
        command = command.with_description(Some(description));
    }



    if let Some(configuration) = request.configuration {
        command = command.with_configuration(configuration);
    }

    command
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}

pub async fn delete_ai_knowledge_base(
    State(app_state): State<AppState>,
    Path((deployment_id, kb_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    DeleteAiKnowledgeBaseCommand::new(deployment_id, kb_id)
        .execute(&app_state)
        .await
        .map(|_| ().into())
        .map_err(Into::into)
}

pub async fn upload_knowledge_base_document(
    State(app_state): State<AppState>,
    Path((deployment_id, kb_id)): Path<(i64, i64)>,
    mut multipart: Multipart,
) -> ApiResult<AiKnowledgeBaseDocument> {
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut file_content: Vec<u8> = Vec::new();
    let mut file_name: Option<String> = None;
    let mut file_type: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to read multipart field: {}", e),
        )
    })? {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "title" => {
                title = Some(field.text().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read title: {}", e),
                    )
                })?);
            }
            "description" => {
                description = Some(field.text().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read description: {}", e),
                    )
                })?);
            }
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                file_type = field.content_type().map(|s| s.to_string());
                file_content = field.bytes().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read file content: {}", e),
                    )
                })?.to_vec();
            }
            _ => {
                // Skip unknown fields
            }
        }
    }

    let title = title.ok_or((StatusCode::BAD_REQUEST, "Title is required".to_string()))?;
    let file_name = file_name.ok_or((StatusCode::BAD_REQUEST, "File is required".to_string()))?;
    let file_type = file_type.unwrap_or("application/octet-stream".to_string());

    if file_content.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "File content is empty".to_string()).into());
    }

    // Verify the knowledge base exists and belongs to the deployment
    GetAiKnowledgeBaseByIdQuery::new(deployment_id, kb_id)
        .execute(&app_state)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Knowledge base not found".to_string()))?;

    UploadKnowledgeBaseDocumentCommand::new(
        kb_id,
        title,
        description,
        file_name,
        file_content,
        file_type,
    )
    .execute(&app_state)
    .await
    .map(Into::into)
    .map_err(Into::into)
}

#[derive(Deserialize)]
pub struct GetDocumentsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn get_knowledge_base_documents(
    State(app_state): State<AppState>,
    Path((deployment_id, kb_id)): Path<(i64, i64)>,
    Query(query): Query<GetDocumentsQuery>,
) -> ApiResult<Vec<AiKnowledgeBaseDocument>> {
    // Verify the knowledge base exists and belongs to the deployment
    GetAiKnowledgeBaseByIdQuery::new(deployment_id, kb_id)
        .execute(&app_state)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Knowledge base not found".to_string()))?;

    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    GetKnowledgeBaseDocumentsQuery::new(kb_id, limit, offset)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(|e| AppError::from(e).into())
}
