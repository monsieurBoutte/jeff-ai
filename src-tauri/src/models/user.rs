use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub family_name: String,
    pub given_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefinedMessage {
    pub suggested_message_rewrite: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub done: bool,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
} 