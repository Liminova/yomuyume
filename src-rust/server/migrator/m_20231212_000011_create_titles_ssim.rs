use axum::async_trait;
use sea_orm_migration::prelude::*;

use super::m_20231115_000003_create_titles_table::Titles;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231212_000011_create_titles_ssim"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(TitlesSsim::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(TitlesSsim::Id)
                    .integer()
                    .auto_increment()
                    .primary_key()
                    .not_null(),
            )
            .col(ColumnDef::new(TitlesSsim::TitleIdA).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-title-ssim-title_id_a")
                    .from(TitlesSsim::Table, TitlesSsim::TitleIdA)
                    .to(Titles::Table, Titles::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new(TitlesSsim::TitleIdB).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-title-ssim-title_id_b")
                    .from(TitlesSsim::Table, TitlesSsim::TitleIdB)
                    .to(Titles::Table, Titles::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new(TitlesSsim::Ssim).integer().not_null())
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(TitlesSsim::Table).to_owned();
        manager.drop_table(table).await
    }
}

#[derive(Iden)]
pub enum TitlesSsim {
    Table,
    Id,
    TitleIdA,
    TitleIdB,
    Ssim,
}
