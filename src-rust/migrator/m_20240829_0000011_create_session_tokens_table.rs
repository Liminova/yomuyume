use axum::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::m_20231113_000001_create_users_table::Users;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240829_0000011_create_session_tokens_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SessionTokens::Table)
                    .col(pk_auto(SessionTokens::SessionId))
                    .col(string(SessionTokens::SessionSecret))
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
    SessionId,
    SessionSecret,
    UserId,
    CreatedAt,
    UserAgent,
    Code,
}
