use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use utoipa::ToSchema;

use crate::models::prelude::{CustomID, UserID};

/// Just serve as an alias for [`CustomID`], nothing more.
pub type SessionSecret = CustomID;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, ToSchema)]
#[schema(as = Progress)]
#[sea_orm(table_name = "session_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub session_secret: SessionSecret,
    pub user_id: UserID,
    pub created_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub ip_address: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
