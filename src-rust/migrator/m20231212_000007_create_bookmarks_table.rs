use axum::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::{
    m20231113_000001_create_users_table::Users, m20231115_000003_create_titles_table::Titles,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20231212_000007_create_bookmarks_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Bookmarks::Table)
                    .if_not_exists()
                    .col(pk_auto(Bookmarks::Id))
                    .col(string(Bookmarks::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-bookmark-user_id")
                            .from(Bookmarks::Table, Bookmarks::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string(Bookmarks::TitleId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-bookmark-title_id")
                            .from(Bookmarks::Table, Bookmarks::TitleId)
                            .to(Titles::Table, Titles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(date_time(Bookmarks::CreatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-bookmarks-user_id-title_id")
                    .table(Bookmarks::Table)
                    .col(Bookmarks::UserId)
                    .col(Bookmarks::TitleId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Bookmarks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Bookmarks {
    Table,
    Id,
    UserId,
    TitleId,
    CreatedAt,
}
