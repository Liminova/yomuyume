use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

use crate::models::prelude::CustomID;

pub type UserID = CustomID;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, ToSchema)]
#[schema(as = User)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: UserID,
    pub username: String,
    pub email: String,
    pub profile_picture: Option<String>,
    pub created_at: String,
    pub ip_address: String,
    pub updated_at: String,
    pub password: String,
    pub is_verified: bool,
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
