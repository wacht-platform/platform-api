use crate::application::{AppError, AppState};

pub trait Command {
    type Output;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError>;
}

pub mod deployment;
pub mod project;
pub mod s3;
pub use deployment::*;
pub use project::*;
pub use s3::*;
