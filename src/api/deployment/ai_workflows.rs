use axum::{
    extract::{Json, Path, Query, State},
};

use crate::{
    application::{
        http::models::{
            query::deployment::GetWorkflowsQuery,
            json::deployment::{CreateWorkflowRequest, UpdateWorkflowRequest, ExecuteWorkflowRequest},
        },
        response::{ApiResult, PaginatedResponse},
        AppState
    },
    core::{
        models::{AiWorkflow, AiWorkflowWithDetails, WorkflowExecution},
        commands::{Command, CreateAiWorkflowCommand, UpdateAiWorkflowCommand, DeleteAiWorkflowCommand},
        queries::{GetAiWorkflowsQuery, GetAiWorkflowByIdQuery, Query as QueryTrait},
    },
};

pub async fn get_ai_workflows(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Query(query): Query<GetWorkflowsQuery>,
) -> ApiResult<PaginatedResponse<AiWorkflowWithDetails>> {
    let limit = query.limit.unwrap_or(50) as u32;

    let workflows = GetAiWorkflowsQuery::new(deployment_id)
        .with_limit(Some(limit + 1))
        .with_offset(query.offset.map(|o| o as u32))
        .with_search(query.search)
        .execute(&app_state)
        .await?;

    let has_more = workflows.len() > limit as usize;
    let workflows = if has_more {
        workflows[..limit as usize].to_vec()
    } else {
        workflows
    };

    Ok(PaginatedResponse {
        data: workflows,
        has_more,
    }.into())
}

pub async fn create_ai_workflow(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Json(request): Json<CreateWorkflowRequest>,
) -> ApiResult<AiWorkflow> {
    CreateAiWorkflowCommand::new(
        deployment_id,
        request.name,
        request.description,
        request.configuration.unwrap_or_default(),
        request.workflow_definition.unwrap_or_default(),
    )
    .execute(&app_state)
    .await
    .map(Into::into)
    .map_err(Into::into)
}

pub async fn get_ai_workflow_by_id(
    State(app_state): State<AppState>,
    Path((deployment_id, workflow_id)): Path<(i64, i64)>,
) -> ApiResult<AiWorkflowWithDetails> {
    GetAiWorkflowByIdQuery::new(deployment_id, workflow_id)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}

pub async fn update_ai_workflow(
    State(app_state): State<AppState>,
    Path((deployment_id, workflow_id)): Path<(i64, i64)>,
    Json(request): Json<UpdateWorkflowRequest>,
) -> ApiResult<AiWorkflow> {
    let mut command = UpdateAiWorkflowCommand::new(deployment_id, workflow_id);

    if let Some(name) = request.name {
        command = command.with_name(name);
    }
    if let Some(description) = request.description {
        command = command.with_description(Some(description));
    }
    if let Some(configuration) = request.configuration {
        command = command.with_configuration(configuration);
    }
    if let Some(workflow_definition) = request.workflow_definition {
        command = command.with_workflow_definition(workflow_definition);
    }

    command
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}

pub async fn delete_ai_workflow(
    State(app_state): State<AppState>,
    Path((deployment_id, workflow_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    DeleteAiWorkflowCommand::new(deployment_id, workflow_id)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}

pub async fn execute_ai_workflow(
    State(app_state): State<AppState>,
    Path((deployment_id, workflow_id)): Path<(i64, i64)>,
    Json(request): Json<ExecuteWorkflowRequest>,
) -> ApiResult<WorkflowExecution> {
    // TODO: Implement workflow execution logic
    // For now, create a basic execution record
    let execution_id = app_state.sf.next_id().map_err(|e| crate::application::AppError::Internal(e.to_string()))? as i64;
    let now = chrono::Utc::now();

    let execution_context = serde_json::json!({
        "variables": request.variables.unwrap_or_default(),
        "node_executions": [],
        "current_node": null
    });

    let execution = sqlx::query!(
        r#"
        INSERT INTO ai_workflow_executions (id, created_at, updated_at, workflow_id, status, trigger_data, execution_context, started_at)
        VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7)
        RETURNING id, created_at, updated_at, workflow_id, status, trigger_data, execution_context, output_data, error_message, started_at, completed_at
        "#,
        execution_id,
        now,
        now,
        workflow_id,
        serde_json::to_value(&request.trigger_data).unwrap_or(serde_json::Value::Null),
        execution_context,
        now,
    )
    .fetch_one(&app_state.db_pool)
    .await
    .map_err(|e| crate::application::AppError::Database(e))?;

    let execution_context: crate::core::models::ExecutionContext = serde_json::from_value(execution.execution_context)
        .unwrap_or_default();

    Ok(WorkflowExecution {
        id: execution.id,
        workflow_id: execution.workflow_id,
        created_at: execution.created_at,
        updated_at: execution.updated_at,
        status: crate::core::models::ExecutionStatus::from(execution.status),
        trigger_data: execution.trigger_data,
        execution_context,
        output_data: execution.output_data,
        started_at: execution.started_at,
        completed_at: execution.completed_at,
        error_message: execution.error_message,
    }.into())
}

pub async fn get_workflow_executions(
    State(app_state): State<AppState>,
    Path((deployment_id, workflow_id)): Path<(i64, i64)>,
    Query(query): Query<GetWorkflowsQuery>,
) -> ApiResult<PaginatedResponse<WorkflowExecution>> {
    let limit = query.limit.unwrap_or(50) as i64;

    let executions = sqlx::query!(
        r#"
        SELECT e.id, e.created_at, e.updated_at, e.workflow_id, e.status, e.trigger_data, e.execution_context, e.output_data, e.error_message, e.started_at, e.completed_at
        FROM ai_workflow_executions e
        JOIN ai_workflows w ON e.workflow_id = w.id
        WHERE w.deployment_id = $1 AND e.workflow_id = $2
        ORDER BY e.created_at DESC
        LIMIT $3 OFFSET $4
        "#,
        deployment_id,
        workflow_id,
        limit + 1,
        query.offset.unwrap_or(0) as i64,
    )
    .fetch_all(&app_state.db_pool)
    .await
    .map_err(|e| crate::application::AppError::Database(e))?;

    let result: Vec<WorkflowExecution> = executions
        .into_iter()
        .map(|e| {
            let execution_context: crate::core::models::ExecutionContext = serde_json::from_value(e.execution_context)
                .unwrap_or_default();

            WorkflowExecution {
                id: e.id,
                workflow_id: e.workflow_id,
                created_at: e.created_at,
                updated_at: e.updated_at,
                status: crate::core::models::ExecutionStatus::from(e.status),
                trigger_data: e.trigger_data,
                execution_context,
                output_data: e.output_data,
                started_at: e.started_at,
                completed_at: e.completed_at,
                error_message: e.error_message,
            }
        })
        .collect();

    let has_more = result.len() > limit as usize;
    let result = if has_more {
        result[..limit as usize].to_vec()
    } else {
        result
    };

    Ok(PaginatedResponse {
        data: result,
        has_more,
    }.into())
}
