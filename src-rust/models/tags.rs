use sea_orm::entity::prelude::*;
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, ToSchema)]
#[schema(as = Tag)]
#[sea_orm(table_name = "tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::titles_tags::Entity")]
    TitlesTags,
}

impl Related<super::titles_tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TitlesTags.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
