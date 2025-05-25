use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
    pub phone_number: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteUserRequest {
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
    pub expiry_days: Option<i64>, // Optional, defaults to 7 days
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddToWaitlistRequest {
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub public_metadata: Option<Value>,
    pub private_metadata: Option<Value>,
}

// Email management requests
#[derive(Debug, Serialize, Deserialize)]
pub struct AddEmailRequest {
    pub email: String,
    pub verified: Option<bool>,
    pub is_primary: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEmailRequest {
    pub email: Option<String>,
    pub verified: Option<bool>,
    pub is_primary: Option<bool>,
}

// Phone number management requests
#[derive(Debug, Serialize, Deserialize)]
pub struct AddPhoneRequest {
    pub phone_number: String,
    pub verified: Option<bool>,
    pub is_primary: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePhoneRequest {
    pub phone_number: Option<String>,
    pub verified: Option<bool>,
    pub is_primary: Option<bool>,
}
