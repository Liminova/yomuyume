use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

use crate::models::prelude::CustomID;

pub type CategoryID = CustomID;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, ToSchema)]
#[schema(as = Category)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: CategoryID,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::titles::Entity")]
    Titles,
    #[sea_orm(has_one = "super::covers::Entity")]
    Covers,
}

impl Related<super::titles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Titles.def()
    }
}

impl Related<super::covers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Covers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
