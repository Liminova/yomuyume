use axum::async_trait;
use sea_orm_migration::prelude::*;

use super::m_20231115_000003_create_titles_table::Titles;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231115_000004_create_pages_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Pages::Table)
            .if_not_exists()
            .col(ColumnDef::new(Pages::Id).string().not_null().primary_key())
            .col(ColumnDef::new(Pages::TitleId).string().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-page-title_id")
                    .from(Pages::Table, Pages::TitleId)
                    .to(Titles::Table, Titles::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new(Pages::Path).string().not_null())
            .col(ColumnDef::new(Pages::Description).string())
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(Pages::Table).to_owned();
        manager.drop_table(table).await
    }
}

#[derive(Iden)]
pub enum Pages {
    Table,
    Id,
    TitleId,
    Path,
    Description,
}
