use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub database_uri: String,
    pub anonymous_repos_user: i64,
}

pub const CONFIG: Lazy<Config> = Lazy::new(|| {
    toml::from_str::<Config>(&std::fs::read_to_string("./Config.toml").unwrap()).unwrap()
});
