use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

use crate::{models::prelude::CategoryID, types::custom_id::CustomID};

/// Just serve as an alias for [`CustomID`], nothing more.
pub type TitleID = CustomID;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "titles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: TitleID,
    pub title: String,
    pub category_id: Option<CategoryID>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub release: Option<DateTime<Utc>>,
    pub path: String,

    pub cover_path: Option<String>,
    pub cover_blurhash: Option<String>,
    pub cover_width: Option<u32>,
    pub cover_height: Option<u32>,

    pub file_hash: u32,
    pub date_added: DateTime<Utc>,
    pub date_updated: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::categories::Entity",
        from = "Column::CategoryId",
        to = "super::categories::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Categories,
    #[sea_orm(has_many = "super::pages::Entity")]
    Pages,
    #[sea_orm(has_many = "super::titles_tags::Entity")]
    TitlesTags,
    #[sea_orm(has_many = "super::bookmarks::Entity")]
    Bookmarks,
    #[sea_orm(has_many = "super::favorites::Entity")]
    Favorites,
    #[sea_orm(has_many = "super::progresses::Entity")]
    Progresses,
}

impl Related<super::categories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Categories.def()
    }
}

impl Related<super::pages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Pages.def()
    }
}

impl Related<super::titles_tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TitlesTags.def()
    }
}

impl Related<super::bookmarks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bookmarks.def()
    }
}

impl Related<super::favorites::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Favorites.def()
    }
}

impl Related<super::progresses::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Progresses.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
