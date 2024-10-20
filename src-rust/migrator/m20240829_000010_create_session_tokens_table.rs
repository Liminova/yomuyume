use axum::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::m20231113_000001_create_users_table::Users;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240829_000010_create_session_tokens_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SessionTokens::Table)
                    .if_not_exists()
                    .col(string(SessionTokens::SessionSecret).primary_key())
                    .col(string(SessionTokens::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-session-token_user-id")
                            .from(SessionTokens::Table, SessionTokens::UserId)
                            .to(Users::Table, Users::Id)
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(date_time(SessionTokens::CreatedAt))
                    .col(date_time(SessionTokens::LastUsedAt))
                    .col(string_null(SessionTokens::UserAgent))
                    .col(string(SessionTokens::Code))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SessionTokens::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum SessionTokens {
    Table,
    SessionSecret,
    UserId,
    CreatedAt,
    LastUsedAt,
    UserAgent,
    Code,
}
