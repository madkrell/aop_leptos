use std::sync::Arc;

use crate::db::Db;
use crate::services::email::Email;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub email: Arc<Email>,
}
