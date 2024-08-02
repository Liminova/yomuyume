use axum::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231116_000005_create_tags_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Tags::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Tags::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Tags::Name).string().not_null().unique_key())
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(Tags::Table).to_owned();
        manager.drop_table(table).await
    }
}

#[derive(Iden)]
pub enum Tags {
    Table,
    Id,
    Name,
}
