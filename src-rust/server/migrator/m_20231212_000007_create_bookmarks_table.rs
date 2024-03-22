use axum::async_trait;
use sea_orm_migration::prelude::*;

use super::{
    m_20231113_000001_create_users_table::Users, m_20231115_000003_create_titles_table::Titles,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231212_000006_create_bookmarks_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Bookmarks::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Bookmarks::Id)
                    .integer()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Bookmarks::UserId).string().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-bookmark-user_id")
                    .from(Bookmarks::Table, Bookmarks::UserId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new(Bookmarks::TitleId).string().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-bookmark-title_id")
                    .from(Bookmarks::Table, Bookmarks::TitleId)
                    .to(Titles::Table, Titles::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(Bookmarks::Table).to_owned();
        manager.drop_table(table).await
    }
}

#[derive(Iden)]
pub enum Bookmarks {
    Table,
    Id,
    UserId,
    TitleId,
}
