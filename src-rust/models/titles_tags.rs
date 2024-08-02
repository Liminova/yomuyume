use sea_orm::entity::prelude::*;
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, ToSchema)]
#[schema(as = TitleTag)]
#[sea_orm(table_name = "titles_tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub title_id: String,
    pub tag_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tags::Entity",
        from = "Column::TitleId",
        to = "super::tags::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Tags,
    #[sea_orm(
        belongs_to = "super::titles::Entity",
        from = "Column::TitleId",
        to = "super::titles::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Titles,
}

impl Related<super::tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tags.def()
    }
}

impl Related<super::titles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Titles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
