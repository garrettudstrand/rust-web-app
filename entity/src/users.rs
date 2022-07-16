use rocket::{serde::{Deserialize, Serialize}, FromForm};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, FromForm)]
#[serde(crate = "rocket::serde")]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[field(default = 0)]
    pub id: i32,
    pub username: String,
    pub password: String
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Tasks
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Tasks => Entity::has_many(super::tasks::Entity).into(),
        }
    }
}

impl Related<super::tasks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tasks.def()
    }
}

pub const USER_PASSWORD_SALT: &[u8] = b"some_random_salt";

impl ActiveModelBehavior for ActiveModel {}