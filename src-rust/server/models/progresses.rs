use sea_orm::entity::prelude::*;
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, ToSchema)]
#[schema(as = Progress)]
#[sea_orm(table_name = "progresses")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: String,
    pub title_id: String,
    pub last_read_at: String,
    pub page: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::Id",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
    #[sea_orm(
        belongs_to = "super::titles::Entity",
        from = "Column::Id",
        to = "super::titles::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Titles,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::titles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Titles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
