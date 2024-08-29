use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

use crate::models::prelude::CustomID;

/// Just serve as an alias for [`CustomID`], nothing more.
pub type CategoryID = CustomID;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, ToSchema)]
#[schema(as = Category)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: CategoryID,
    pub name: String,
    pub description: Option<String>,

    // no we don't need a separated table for this
    pub cover_path: Option<String>,
    pub cover_blurhash: Option<String>,
    pub cover_width: Option<u32>,
    pub cover_height: Option<u32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::titles::Entity")]
    Titles,
}

impl Related<super::titles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Titles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
