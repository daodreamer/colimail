use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountConfig {
    pub id: Option<i32>,
    pub email: String,
    pub password: String,
    pub imap_server: String,
    pub imap_port: u16,
    pub smtp_server: String,
    pub smtp_port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailHeader {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub date: String,
}
