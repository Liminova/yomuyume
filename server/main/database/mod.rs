pub mod content;
pub mod user;

use std::path::Path;

#[derive(Debug)]
pub struct Database {
    pub user: redb::Database,
    pub content: redb::Database,
}

impl Database {
    pub fn new(
        user_db_path: impl AsRef<Path>,
        content_db_path: impl AsRef<Path>,
    ) -> Result<Self, redb::DatabaseError> {
        Ok(Self {
            user: redb::Database::create(user_db_path)?,
            content: redb::Database::create(content_db_path)?,
        })
    }
}
