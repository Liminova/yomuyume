use axum::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::{
    m_20231113_000001_create_users_table::Users, m_20231115_000003_create_titles_table::Titles,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231212_000010_create_progresses_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Progresses::Table)
                    .if_not_exists()
                    .col(pk_auto(Progresses::Id))
                    .col(string(Progresses::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-progress-user_id")
                            .from(Progresses::Table, Progresses::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string(Progresses::TitleId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-progress-title_id")
                            .from(Progresses::Table, Progresses::TitleId)
                            .to(Titles::Table, Titles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(date_time(Progresses::LastReadAt))
                    .col(integer(Progresses::Page))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Progresses::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Progresses {
    Table,
    Id,
    UserId,
    TitleId,
    LastReadAt,
    Page,
}
