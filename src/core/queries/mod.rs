use crate::application::{AppError, AppState};

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Not found")]
    NotFound,
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl From<QueryError> for AppError {
    fn from(error: QueryError) -> Self {
        match error {
            QueryError::DatabaseError(msg) => AppError::Internal(msg),
            QueryError::NotFound => AppError::NotFound("Resource not found".to_string()),
            QueryError::BadRequest(msg) => AppError::BadRequest(msg),
        }
    }
}

pub trait Query {
    type Output;

    async fn execute(&self, app_state: &AppState) -> Result<Self::Output, AppError>;
}

pub mod b2b;
pub mod deployment;
pub mod project;
pub mod user;

// AI-related queries
pub mod ai_agent;
pub mod ai_workflow;
pub mod ai_tool;
pub mod ai_knowledge_base;

pub use b2b::*;
pub use deployment::*;
pub use project::*;
pub use user::*;

// AI-related exports
pub use ai_agent::*;
pub use ai_workflow::*;
pub use ai_tool::*;
pub use ai_knowledge_base::*;
