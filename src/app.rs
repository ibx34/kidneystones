use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use snowflake::SnowflakeIdBucket;
use tokio::sync::Mutex;

use crate::{
    config::CONFIG,
    db::Database,
    models::{Accounts, Repos, Sessions},
};
use anyhow::Result;
use axum_template::engine::Engine;
use handlebars::Handlebars;

#[derive(Clone)]
pub struct App {
    pub db: Database,
    pub hbs: Engine<Handlebars<'static>>,
    pub ids: Arc<Mutex<SnowflakeIdBucket>>,
}

impl App {
    pub async fn init() -> Result<App> {
        let database = Database::new(&CONFIG.database_uri).await.unwrap();
        database.migrate().await.unwrap();

        let mut hbs = Handlebars::new();
        hbs.register_template_file("home", "/home/alfredo/kidney-stones/templates/home.hbs")?;
        hbs.register_template_file(
            "repos/create",
            "/home/alfredo/kidney-stones/templates/repos/create.hbs",
        )?;
        hbs.register_partial(
            "html_head",
            &std::fs::read_to_string("/home/alfredo/kidney-stones/templates/head.hbs")?,
        )?;
        hbs.register_partial(
            "html_tail",
            &std::fs::read_to_string("/home/alfredo/kidney-stones/templates/tail.hbs")?,
        )?;
        hbs.register_partial(
            "nav",
            &std::fs::read_to_string("/home/alfredo/kidney-stones/templates/nav.hbs")?,
        )?;

        Ok(Self {
            db: database,
            hbs: Engine::from(hbs),
            ids: Arc::new(Mutex::new(SnowflakeIdBucket::new(1, 1))),
        })
    }

    pub async fn get_session_by_key(&self, key: &str) -> Result<Sessions> {
        Ok(
            sqlx::query_as::<_, Sessions>(r#"SELECT * FROM sessions WHERE key = ($1)"#)
                .bind(key)
                .fetch_one(&self.db.0)
                .await?,
        )
    }

    pub async fn get_account_by_id(&self, id: i64) -> Result<Accounts> {
        Ok(
            sqlx::query_as::<_, Accounts>(r#"SELECT * FROM accounts WHERE id = ($1)"#)
                .bind(id)
                .fetch_one(&self.db.0)
                .await?,
        )
    }

    pub async fn get_session(&self, key: &str) -> Result<(Sessions, Accounts)> {
        let session = self.get_session_by_key(key).await?;
        let accociated_account = self.get_account_by_id(session.owner).await?;
        return Ok((session, accociated_account));
    }

    pub async fn create_repo(&self, name: &str, owner: i64, owner_name: &str) -> Result<Repos> {
        Ok(sqlx::query_as::<_, Repos>(
            r#"INSERT INTO repos(id,name,owner,owner_name) VALUES($1,$2,$3,$4) RETURNING *"#,
        )
        .bind(self.ids.clone().lock().await.get_id())
        .bind(name)
        .bind(owner)
        .bind(owner_name)
        .fetch_one(&self.db.0)
        .await?)
    }

    pub async fn create_account(&self, name: &str, password: &str) -> Result<Accounts> {
        let password_as_bytes = password.as_bytes();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hashed_password = argon2
            .hash_password(&password_as_bytes, &salt)
            .unwrap()
            .to_string();

        let parsed_hash = PasswordHash::new(&hashed_password).unwrap();
        assert!(Argon2::default()
            .verify_password(&password_as_bytes, &parsed_hash)
            .is_ok());

        Ok(sqlx::query_as::<_, Accounts>(
            r#"INSERT INTO accounts(id,name,password) VALUES($1,$2,$3) RETURNING *"#,
        )
        .bind(self.ids.clone().lock().await.get_id())
        .bind(name)
        .bind(hashed_password)
        .fetch_one(&self.db.0)
        .await?)
    }
}
