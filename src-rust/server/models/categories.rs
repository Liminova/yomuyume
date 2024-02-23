use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, ToSchema)]
#[schema(as = Category)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::titles::Entity")]
    Titles,
    #[sea_orm(has_one = "super::thumbnails::Entity")]
    Thumbnails,
}

impl Related<super::titles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Titles.def()
    }
}

impl Related<super::thumbnails::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Thumbnails.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
