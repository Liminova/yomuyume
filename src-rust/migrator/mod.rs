use axum::async_trait;
use sea_orm_migration::prelude::*;

mod m_20231113_000001_create_users_table;
mod m_20231115_000002_create_categories_table;
mod m_20231115_000003_create_titles_table;
mod m_20231115_000004_create_pages_table;
mod m_20231116_000005_create_tags_table;
mod m_20231116_000006_create_titles_tags_table;
mod m_20231212_000007_create_bookmarks_table;
mod m_20231212_000009_create_favorites_table;
mod m_20231212_000010_create_progresses_table;
mod m_20240829_0000011_create_session_tokens_table;
mod m_20240829_0000012_create_code_delete_account_table;
mod m_20240829_0000013_create_code_reset_password_table;
mod m_20240829_0000014_create_code_validate_email_table;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m_20231113_000001_create_users_table::Migration),
            Box::new(m_20231115_000002_create_categories_table::Migration),
            Box::new(m_20231115_000003_create_titles_table::Migration),
            Box::new(m_20231115_000004_create_pages_table::Migration),
            Box::new(m_20231116_000005_create_tags_table::Migration),
            Box::new(m_20231116_000006_create_titles_tags_table::Migration),
            Box::new(m_20231212_000007_create_bookmarks_table::Migration),
            Box::new(m_20231212_000009_create_favorites_table::Migration),
            Box::new(m_20231212_000010_create_progresses_table::Migration),
            Box::new(m_20240829_0000011_create_session_tokens_table::Migration),
            Box::new(m_20240829_0000012_create_code_delete_account_table::Migration),
            Box::new(m_20240829_0000013_create_code_reset_password_table::Migration),
            Box::new(m_20240829_0000014_create_code_validate_email_table::Migration),
        ]
    }
}
