use crate::application::{AppError, AppState};

pub trait Command<T> {
    async fn execute(&self, app_state: &AppState) -> Result<T, AppError>;
}

pub mod deployment;
pub use deployment::*;
