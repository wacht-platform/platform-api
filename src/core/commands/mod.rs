use crate::application::{AppError, AppState};

pub trait Command {
    type Output;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError>;
}

pub mod create_organization;
pub mod create_workspace;
mod delete_organization;
pub mod deployment;
pub mod deployment_email_template;
mod organization_member;
mod organization_role;
pub mod project;
pub mod s3;
mod update_organization;
pub mod user;
pub mod user_identifiers;

pub use create_organization::*;
pub use create_workspace::*;
pub use delete_organization::*;
pub use deployment::*;
pub use deployment_email_template::*;
pub use organization_member::*;
pub use organization_role::*;
pub use project::*;
pub use s3::*;
pub use update_organization::*;
pub use user::*;
pub use user_identifiers::*;
