use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

use crate::{models::prelude::UserID, types::custom_id::CustomID};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(14))")]
pub enum Purpose {
    #[sea_orm(string_value = "delete_account")]
    DeleteAccount,
    #[sea_orm(string_value = "reset_password")]
    ResetPassword,
    #[sea_orm(string_value = "validate_email")]
    ValidateEmail,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "temp_codes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub purpose: Purpose,
    pub user_id: UserID,
    pub code: CustomID,
    pub created_at: DateTime<Utc>,
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

impl ActiveModelBehavior for ActiveModel {}
