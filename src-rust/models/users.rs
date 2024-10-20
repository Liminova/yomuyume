use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

use crate::models::prelude::CustomID;

/// Just serve as an alias for [`CustomID`], nothing more.
pub type UserID = CustomID;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: UserID,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub profile_picture: Option<String>,
    pub ip_address: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "super::bookmarks::Entity",
        from = "Column::Id",
        to = "super::bookmarks::Column::UserId"
    )]
    Bookmarks,
    #[sea_orm(
        has_many = "super::progresses::Entity",
        from = "Column::Id",
        to = "super::progresses::Column::UserId"
    )]
    Progresses,
    #[sea_orm(
        has_many = "super::favorites::Entity",
        from = "Column::Id",
        to = "super::favorites::Column::UserId"
    )]
    Favorites,
}

impl Related<super::bookmarks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bookmarks.def()
    }
}

impl Related<super::progresses::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Progresses.def()
    }
}

impl Related<super::favorites::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Favorites.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
