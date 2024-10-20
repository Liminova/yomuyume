use axum::async_trait;
use sea_orm_migration::prelude::*;

mod m20231113_000001_create_users_table;
mod m20231115_000002_create_categories_table;
mod m20231115_000003_create_titles_table;
mod m20231115_000004_create_pages_table;
mod m20231116_000005_create_tags_table;
mod m20231116_000006_create_titles_tags_table;
mod m20231212_000007_create_bookmarks_table;
mod m20231212_000008_create_favorites_table;
mod m20231212_000009_create_progresses_table;
mod m20240829_000010_create_session_tokens_table;
mod m20240829_000011_create_temp_codes_table;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20231113_000001_create_users_table::Migration),
            Box::new(m20231115_000002_create_categories_table::Migration),
            Box::new(m20231115_000003_create_titles_table::Migration),
            Box::new(m20231115_000004_create_pages_table::Migration),
            Box::new(m20231116_000005_create_tags_table::Migration),
            Box::new(m20231116_000006_create_titles_tags_table::Migration),
            Box::new(m20231212_000007_create_bookmarks_table::Migration),
            Box::new(m20231212_000008_create_favorites_table::Migration),
            Box::new(m20231212_000009_create_progresses_table::Migration),
            Box::new(m20240829_000010_create_session_tokens_table::Migration),
            Box::new(m20240829_000011_create_temp_codes_table::Migration),
        ]
    }
}
