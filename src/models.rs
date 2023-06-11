use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Deserialize, Serialize)]
pub struct RepoFile {
    pub filename: String,
    pub size: usize,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Accounts {
    pub id: i64,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Repos {
    pub id: i64,
    pub name: String,
    pub owner: i64,
    pub owner_name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Sessions {
    pub id: i64,
    pub owner: i64,
    pub key: String,
}
