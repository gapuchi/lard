use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub content: String,
}

#[derive(Deserialize)]
pub struct User {
    pub username: String,
    pub discriminator: String,
    pub avatar: String,
    pub email: Option<String>,
    pub flags: Option<String>,
}