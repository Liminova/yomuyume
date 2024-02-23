use axum::async_trait;
use sea_orm_migration::prelude::*;

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
        let table = Table::create()
            .table(Progresses::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Progresses::Id)
                    .integer()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Progresses::UserId).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-progress-user_id")
                    .from(Progresses::Table, Progresses::UserId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new(Progresses::TitleId).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-progress-title_id")
                    .from(Progresses::Table, Progresses::TitleId)
                    .to(Titles::Table, Titles::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(
                ColumnDef::new(Progresses::LastReadAt)
                    .date_time()
                    .not_null(),
            )
            .col(ColumnDef::new(Progresses::Page).integer().not_null())
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(Progresses::Table).to_owned();
        manager.drop_table(table).await
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
