use axum::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::m_20231113_000001_create_users_table::Users;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240829_0000013_create_code_reset_password_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CodeResetPassword::Table)
                    .col(string_uniq(CodeResetPassword::UserId).primary_key())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pending-code-reset-password-_user-id")
                            .from(CodeResetPassword::Table, CodeResetPassword::UserId)
                            .to(Users::Table, Users::Id)
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(date_time(CodeResetPassword::CreatedAt))
                    .col(string_uniq(CodeResetPassword::Code))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CodeResetPassword::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum CodeResetPassword {
    Table,
    UserId,
    CreatedAt,
    Code,
}
