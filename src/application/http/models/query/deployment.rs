use std::{
    fmt::{self, Display},
    str::FromStr,
};

use serde::Deserialize;

use super::SortOrder;

#[derive(Debug, PartialEq, Deserialize)]
pub enum UserListSortKey {
    CreatedAt,
    Username,
    Email,
    PhoneNumber,
}

impl Default for UserListSortKey {
    fn default() -> Self {
        Self::CreatedAt
    }
}

impl FromStr for UserListSortKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "created_at" => Ok(UserListSortKey::CreatedAt),
            "username" => Ok(UserListSortKey::Username),
            "email" => Ok(UserListSortKey::Email),
            "phone_number" => Ok(UserListSortKey::PhoneNumber),
            _ => Err("Invalid sort key".to_string()),
        }
    }
}

impl Display for UserListSortKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreatedAt => write!(f, "created_at"),
            Self::Username => write!(f, "username"),
            Self::Email => write!(f, "email"),
            Self::PhoneNumber => write!(f, "phone_number"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UserListQueryParams {
    pub offset: Option<i64>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde::enum_from_str::from_str_option"
    )]
    pub sort_key: Option<UserListSortKey>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde::enum_from_str::from_str_option"
    )]
    pub sort_order: Option<SortOrder>,
    pub limit: Option<usize>,
    pub disabled: Option<bool>,
    pub invited: Option<bool>,
}

impl Default for UserListQueryParams {
    fn default() -> Self {
        Self {
            offset: Some(0),
            sort_key: Some(UserListSortKey::CreatedAt),
            sort_order: Some(SortOrder::Desc),
            limit: Some(10),
            disabled: Some(false),
            invited: Some(false),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OrganizationListQueryParams {
    pub offset: Option<i64>,
    pub sort_key: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<i32>,
    pub disabled: Option<bool>,
    pub invited: Option<bool>,
}

impl Default for OrganizationListQueryParams {
    fn default() -> Self {
        Self {
            offset: Some(0),
            sort_key: Some("created_at".to_string()),
            sort_order: Some("desc".to_string()),
            limit: Some(10),
            disabled: Some(false),
            invited: Some(false),
        }
    }
}
