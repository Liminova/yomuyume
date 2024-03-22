use axum::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231115_000002_create_categories_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Categories::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Categories::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(Categories::Name).string().not_null())
            .col(ColumnDef::new(Categories::Description).string())
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(Categories::Table).to_owned();
        manager.drop_table(table).await
    }
}

#[derive(Iden)]
pub enum Categories {
    Table,
    Id,
    Name,
    Description,
}
