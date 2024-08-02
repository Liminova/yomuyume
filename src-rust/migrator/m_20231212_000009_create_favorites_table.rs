use axum::async_trait;
use sea_orm_migration::prelude::*;

use super::{
    m_20231113_000001_create_users_table::Users, m_20231115_000003_create_titles_table::Titles,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231212_000006_create_favorites_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Favorites::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Favorites::Id)
                    .integer()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Favorites::UserId).string().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-favorite-user_id")
                    .from(Favorites::Table, Favorites::UserId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new(Favorites::TitleId).string().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-favorite-title_id")
                    .from(Favorites::Table, Favorites::TitleId)
                    .to(Titles::Table, Titles::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(Favorites::Table).to_owned();
        manager.drop_table(table).await
    }
}

#[derive(Iden)]
pub enum Favorites {
    Table,
    Id,
    UserId,
    TitleId,
}
