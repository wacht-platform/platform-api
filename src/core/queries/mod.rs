use crate::application::{AppError, AppState};

pub trait Query<T> {
    async fn execute(&self, app_state: &AppState) -> Result<T, AppError>;
}

pub mod deployment;
pub mod organization;
pub mod project;
pub mod user;

pub use organization::*;
pub use project::*;
pub use user::*;
