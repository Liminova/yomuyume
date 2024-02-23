use axum::async_trait;
use sea_orm_migration::prelude::*;

use super::{
    m_20231115_000003_create_titles_table::Titles, m_20231116_000005_create_tags_table::Tags,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231116_000006_create_titles_tags_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(TitlesTags::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(TitlesTags::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(TitlesTags::TitleId).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-titletag-title_id")
                    .from(TitlesTags::Table, TitlesTags::TitleId)
                    .to(Titles::Table, Titles::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new(TitlesTags::TagId).integer().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-titletag-tag_id")
                    .from(TitlesTags::Table, TitlesTags::TagId)
                    .to(Tags::Table, Tags::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(TitlesTags::Table).to_owned();
        manager.drop_table(table).await
    }
}

#[derive(Iden)]
pub enum TitlesTags {
    Table,
    Id,
    TitleId,
    TagId,
}
