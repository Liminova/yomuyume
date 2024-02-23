use sea_orm::entity::prelude::*;
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, ToSchema)]
#[schema(as = TitleTag)]
#[sea_orm(table_name = "titles_ssim")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64, // Auto increment, don't care
    pub title_id_a: String,
    pub title_id_b: String,
    pub ssim: u16, // Normalized to 0-1000
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::models::titles::Entity",
        from = "Column::TitleIdA",
        to = "crate::models::titles::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    TitleSsimA,

    #[sea_orm(
        belongs_to = "crate::models::titles::Entity",
        from = "Column::TitleIdB",
        to = "crate::models::titles::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    TitleSsimB,
}

impl Linked for Entity {
    type FromEntity = crate::models::titles_ssim::Entity;
    type ToEntity = crate::models::titles::Entity;
    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::TitleSsimA.def(), Relation::TitleSsimB.def()]
    }
}

impl ActiveModelBehavior for ActiveModel {}
