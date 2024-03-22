use super::m_20231115_000003_create_titles_table::Titles;
use axum::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231212_000008_create_covers_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Covers::Table)
            .if_not_exists()
            .col(ColumnDef::new(Covers::Id).string().primary_key())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-cover-title_id")
                    .from(Covers::Table, Covers::Id)
                    .to(Titles::Table, Titles::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new(Covers::Path).string().not_null())
            .col(ColumnDef::new(Covers::Blurhash).string().not_null())
            .col(ColumnDef::new(Covers::Ratio).integer().not_null())
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(Covers::Table).to_owned();
        manager.drop_table(table).await
    }
}

#[derive(Iden)]
pub enum Covers {
    Table,
    Id,
    Path,
    Blurhash,
    Ratio,
}
