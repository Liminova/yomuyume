use axum::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231113_000001_create_users_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .col(string(Users::Id).primary_key())
                    .col(string_uniq(Users::Username))
                    .col(string_uniq(Users::Email))
                    .col(string_null(Users::ProfilePicture))
                    .col(date_time(Users::CreatedAt))
                    .col(date_time(Users::UpdatedAt))
                    .col(string(Users::Password))
                    .col(boolean(Users::IsVerified))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    Username,
    Email,
    ProfilePicture,
    CreatedAt,
    UpdatedAt,
    Password,
    IsVerified,
}
