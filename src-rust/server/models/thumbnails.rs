use sea_orm::entity::prelude::*;
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, ToSchema)]
#[schema(as = Thumbnail)]
#[sea_orm(table_name = "thumbnails")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub path: String,
    pub blurhash: String,
    pub ratio: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::categories::Entity",
        from = "Column::Id",
        to = "super::categories::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Categories,
    #[sea_orm(
        belongs_to = "super::titles::Entity",
        from = "Column::Id",
        to = "super::titles::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Titles,
}

impl Related<super::categories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Categories.def()
    }
}

impl Related<super::titles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Titles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
