use crate::application::{AppError, AppState};

pub trait Query {
    type Output;

    async fn execute(&self, app_state: &AppState) -> Result<Self::Output, AppError>;
}

pub mod b2b;
pub mod deployment;
pub mod project;
pub mod user;

pub use b2b::*;
pub use deployment::*;
pub use project::*;
pub use user::*;
