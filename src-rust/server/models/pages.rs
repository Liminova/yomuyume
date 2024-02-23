use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, ToSchema)]
#[schema(as = Page)]
#[sea_orm(table_name = "pages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub title_id: String,
    pub path: String,
    pub description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::titles::Entity",
        from = "Column::TitleId",
        to = "super::titles::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Titles,
}

impl Related<super::titles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Titles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
