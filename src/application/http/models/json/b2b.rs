use serde::Deserialize;

// Organization models
#[derive(Debug, Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub public_metadata: Option<serde_json::Value>,
    pub private_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub public_metadata: Option<serde_json::Value>,
    pub private_metadata: Option<serde_json::Value>,
}

// Workspace models
#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub public_metadata: Option<serde_json::Value>,
    pub private_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub public_metadata: Option<serde_json::Value>,
    pub private_metadata: Option<serde_json::Value>,
}

// Organization member models
#[derive(Debug, Deserialize)]
pub struct AddOrganizationMemberRequest {
    pub user_id: i64,
    pub role_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrganizationMemberRequest {
    pub role_ids: Vec<i64>,
}

// Organization role models
#[derive(Debug, Deserialize)]
pub struct CreateOrganizationRoleRequest {
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrganizationRoleRequest {
    pub name: Option<String>,
    pub permissions: Option<Vec<String>>,
}
