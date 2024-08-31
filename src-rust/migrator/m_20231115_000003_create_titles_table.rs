use axum::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::m_20231115_000002_create_categories_table::Categories;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231115_000003_create_titles_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Titles::Table)
            .if_not_exists()
            .col(string(Titles::Id).primary_key())
            .col(string(Titles::Title))
            .col(string_null(Titles::CategoryId))
            .foreign_key(
                ForeignKey::create()
                    .name("fk-title-category_id")
                    .from(Titles::Table, Titles::CategoryId)
                    .to(Categories::Table, Categories::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .col(string_null(Titles::Author))
            .col(string_null(Titles::Description))
            .col(date_time_null(Titles::Release))
            .col(string(Titles::Path))
            .col(string(Titles::ContentFileHash))
            .col(string(Titles::CoverAndPageDescHash))
            .col(string_null(Titles::CoverPath))
            .col(string_null(Titles::CoverBlurhash))
            .col(integer_null(Titles::BlurhashWidth))
            .col(integer_null(Titles::BlurhashHeight))
            .col(date_time(Titles::DateAdded))
            .col(date_time_null(Titles::DateUpdated))
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(Titles::Table).to_owned();
        manager.drop_table(table).await
    }
}

#[derive(Iden)]
pub enum Titles {
    Table,
    Id,
    Title,
    CategoryId,
    Author,
    Description,
    Release,
    Path,

    ContentFileHash,
    CoverAndPageDescHash,

    CoverPath,
    CoverBlurhash,
    BlurhashWidth,
    BlurhashHeight,

    DateAdded,
    DateUpdated,
}
