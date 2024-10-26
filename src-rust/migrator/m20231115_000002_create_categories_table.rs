use axum::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20231115_000002_create_categories_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .if_not_exists()
                    .col(string(Categories::Id).primary_key())
                    .col(string(Categories::Name))
                    .col(string_null(Categories::Description))
                    .col(string_null(Categories::CoverPath))
                    .col(string_null(Categories::CoverBlurhash))
                    .col(integer_null(Categories::BlurhashResolutionX))
                    .col(integer_null(Categories::BlurhashResolutionY))
                    .col(integer_null(Categories::CoverFileHash))
                    .to_owned(),
            )
            .await
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

    CoverPath,
    CoverBlurhash,
    BlurhashResolutionX,
    BlurhashResolutionY,
    CoverFileHash,
}
