use axum::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231113_000001_create_users_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Users::Table)
            .if_not_exists()
            .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
            .col(ColumnDef::new(Users::Username).string().not_null())
            .col(ColumnDef::new(Users::Email).string().not_null())
            .col(ColumnDef::new(Users::ProfilePicture).string())
            .col(ColumnDef::new(Users::CreatedAt).date_time().not_null())
            .col(ColumnDef::new(Users::UpdatedAt).date_time().not_null())
            .col(ColumnDef::new(Users::Password).string().not_null())
            .col(ColumnDef::new(Users::IsVerified).boolean().not_null())
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(Users::Table).to_owned();
        manager.drop_table(table).await
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
