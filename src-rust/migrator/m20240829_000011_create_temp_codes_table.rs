use axum::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use crate::{migrator::m20231113_000001_create_users_table::Users, models::temp_codes};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240829_000013_create_temp_codes_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TempCodes::Table)
                    .if_not_exists()
                    .col(pk_auto(TempCodes::Id))
                    .col(
                        ColumnDef::new(temp_codes::Column::Purpose)
                            .string_len(14)
                            .not_null(),
                    )
                    .col(string(TempCodes::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-temp-code_user-id")
                            .from(TempCodes::Table, TempCodes::UserId)
                            .to(Users::Table, Users::Id)
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string_uniq(TempCodes::Code))
                    .col(date_time(TempCodes::CreatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-temp-code-purpose-user_id")
                    .table(TempCodes::Table)
                    .col(TempCodes::Purpose)
                    .col(TempCodes::UserId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TempCodes::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum TempCodes {
    Table,
    Id,
    Purpose,
    UserId,
    Code,
    CreatedAt,
}
