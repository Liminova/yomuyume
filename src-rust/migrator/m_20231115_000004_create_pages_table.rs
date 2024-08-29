use axum::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

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
        manager
            .create_table(
                Table::create()
                    .table(Pages::Table)
                    .if_not_exists()
                    .col(string(Pages::Id).primary_key())
                    .col(string(Pages::TitleId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-page-title_id")
                            .from(Pages::Table, Pages::TitleId)
                            .to(Titles::Table, Titles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string(Pages::Path))
                    .col(string(Pages::Description))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Pages::Table).to_owned())
            .await
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
