use sea_orm::entity::prelude::*;

use crate::types::custom_id::CustomID;

/// Just serve as an alias for [`CustomID`], nothing more.
pub type CategoryID = CustomID;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: CategoryID,
    pub name: String,
    pub description: Option<String>,

    pub cover_path: Option<String>,
    pub cover_blurhash: Option<String>,
    pub blurhash_resolution_x: Option<u8>,
    pub blurhash_resolution_y: Option<u8>,
    pub cover_file_hash: Option<u32>,
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
