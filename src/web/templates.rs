use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Home {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ReposCreateDefaults<T: Into<String>> {
    pub owner: T,
}

#[derive(Debug, Serialize)]
pub struct ReposCreate<T: Into<String>> {
    pub defaults: ReposCreateDefaults<T>,
    pub user_is_logged_in: bool,
}

#[derive(Debug, Serialize)]
pub struct Base<T> {
    pub signups_allowed: bool,
    pub nested: T,
}
