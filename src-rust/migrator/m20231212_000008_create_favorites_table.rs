use axum::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::{
    m20231113_000001_create_users_table::Users, m20231115_000003_create_titles_table::Titles,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20231212_000006_create_favorites_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Favorites::Table)
                    .if_not_exists()
                    .col(pk_auto(Favorites::Id))
                    .col(string(Favorites::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-favorite-user_id")
                            .from(Favorites::Table, Favorites::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string(Favorites::TitleId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-favorite-title_id")
                            .from(Favorites::Table, Favorites::TitleId)
                            .to(Titles::Table, Titles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(date_time(Favorites::CreatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-favorites-user_id-title_id")
                    .table(Favorites::Table)
                    .col(Favorites::UserId)
                    .col(Favorites::TitleId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Favorites::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Favorites {
    Table,
    Id,
    UserId,
    TitleId,
    CreatedAt,
}
