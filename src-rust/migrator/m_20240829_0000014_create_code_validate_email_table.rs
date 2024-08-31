use axum::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::m_20231113_000001_create_users_table::Users;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240829_0000014_create_code_validate_email_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CodeValidateEmail::Table)
                    .col(string_uniq(CodeValidateEmail::UserId).primary_key())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-code-validate-email-_user-id")
                            .from(CodeValidateEmail::Table, CodeValidateEmail::UserId)
                            .to(Users::Table, Users::Id)
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(date_time(CodeValidateEmail::CreatedAt))
                    .col(string_uniq(CodeValidateEmail::Code))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CodeValidateEmail::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum CodeValidateEmail {
    Table,
    UserId,
    CreatedAt,
    Code,
}
