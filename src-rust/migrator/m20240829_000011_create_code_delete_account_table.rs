use axum::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::m20231113_000001_create_users_table::Users;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240829_000011_create_code_delete_account_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CodeDeleteAccount::Table)
                    .if_not_exists()
                    .col(string_uniq(CodeDeleteAccount::UserId).primary_key())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-code-delete-account-_user-id")
                            .from(CodeDeleteAccount::Table, CodeDeleteAccount::UserId)
                            .to(Users::Table, Users::Id)
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(date_time(CodeDeleteAccount::CreatedAt))
                    .col(string_uniq(CodeDeleteAccount::Code))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CodeDeleteAccount::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum CodeDeleteAccount {
    Table,
    UserId,
    CreatedAt,
    Code,
}
