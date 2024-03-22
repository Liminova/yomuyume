use axum::async_trait;
use sea_orm_migration::prelude::*;

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
            .col(ColumnDef::new(Titles::Id).string().not_null().primary_key())
            .col(ColumnDef::new(Titles::Title).string().not_null())
            .col(ColumnDef::new(Titles::CategoryId).string())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-title-category_id")
                    .from(Titles::Table, Titles::CategoryId)
                    .to(Categories::Table, Categories::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new(Titles::Author).string())
            .col(ColumnDef::new(Titles::Description).string())
            .col(ColumnDef::new(Titles::Release).date_time())
            .col(ColumnDef::new(Titles::Hash).string())
            .col(ColumnDef::new(Titles::Path).string())
            .col(ColumnDef::new(Titles::DateAdded).date_time())
            .col(ColumnDef::new(Titles::DateUpdated).date_time())
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
    Hash,
    Path,
    DateAdded,
    DateUpdated,
}
