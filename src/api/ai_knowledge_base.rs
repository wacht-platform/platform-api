use axum::{
    extract::{Json, Multipart, Path, State},
    http::StatusCode,
};

use crate::{
    application::AppState,
    core::{
        commands::{
            Command, CreateAiKnowledgeBaseCommand, DeleteAiKnowledgeBaseCommand,
            UpdateAiKnowledgeBaseCommand, UploadKnowledgeBaseDocumentCommand,
        },
        models::{
            AiKnowledgeBaseDocument, AiKnowledgeBaseWithDocuments, CreateAiKnowledgeBaseRequest,
            UpdateAiKnowledgeBaseRequest,
        },
        queries::{GetAiKnowledgeBaseByIdQuery, GetAiKnowledgeBasesQuery, Query},
    },
};

use crate::application::response::{ApiResult, PaginatedResponse};

pub async fn get_ai_knowledge_bases(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
) -> ApiResult<PaginatedResponse<AiKnowledgeBaseWithDocuments>> {
    let knowledge_bases = GetAiKnowledgeBasesQuery::new(deployment_id, 0, 50, None)
        .execute(&app_state)
        .await?;

    Ok(PaginatedResponse {
        data: knowledge_bases,
        has_more: false,
    }
    .into())
}

pub async fn get_ai_knowledge_base_by_id(
    State(app_state): State<AppState>,
    Path((deployment_id, kb_id)): Path<(i64, i64)>,
) -> ApiResult<AiKnowledgeBaseWithDocuments> {
    let knowledge_base = GetAiKnowledgeBaseByIdQuery::new(deployment_id, kb_id)
        .execute(&app_state)
        .await?;

    Ok(knowledge_base.into())
}

pub async fn create_ai_knowledge_base(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Json(request): Json<CreateAiKnowledgeBaseRequest>,
) -> ApiResult<AiKnowledgeBaseWithDocuments> {
    let knowledge_base = CreateAiKnowledgeBaseCommand::new(deployment_id, request)
        .execute(&app_state)
        .await?;

    Ok(knowledge_base.into())
}

pub async fn update_ai_knowledge_base(
    State(app_state): State<AppState>,
    Path((deployment_id, kb_id)): Path<(i64, i64)>,
    Json(request): Json<UpdateAiKnowledgeBaseRequest>,
) -> ApiResult<AiKnowledgeBaseWithDocuments> {
    let knowledge_base = UpdateAiKnowledgeBaseCommand::new(deployment_id, kb_id, request)
        .execute(&app_state)
        .await?;

    Ok(knowledge_base.into())
}

pub async fn delete_ai_knowledge_base(
    State(app_state): State<AppState>,
    Path((deployment_id, kb_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    DeleteAiKnowledgeBaseCommand::new(deployment_id, kb_id)
        .execute(&app_state)
        .await?;

    Ok(().into())
}

pub async fn upload_knowledge_base_document(
    State(app_state): State<AppState>,
    Path((deployment_id, kb_id)): Path<(i64, i64)>,
    mut multipart: Multipart,
) -> ApiResult<AiKnowledgeBaseDocument> {
    let document = UploadKnowledgeBaseDocumentCommand::new(deployment_id, kb_id, multipart)
        .execute(&app_state)
        .await?;

    Ok(document.into())
}
