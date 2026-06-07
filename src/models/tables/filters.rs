use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum Kind {
    #[sea_orm(string_value = "iiface")]
    InputInterface,
    #[sea_orm(string_value = "oiface")]
    OutputInterface,
    #[sea_orm(string_value = "protocol")]
    Protocol,
}

#[derive(Clone, Debug, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "filters")]
pub struct Model {
    pub oob_time_sec: u32,
    pub oob_time_usec: u32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub kind: Kind,
    #[sea_orm(primary_key, auto_increment = false)]
    pub value: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl super::HasTimestamp for Entity {
    type TimestampColumn = Column;
    fn timestamp_column() -> Column {
        Column::OobTimeSec
    }
}
