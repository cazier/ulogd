pub mod filters;
pub mod log;

pub trait HasTimestamp: sea_orm::EntityTrait {
    type TimestampColumn: sea_orm::ColumnTrait;
    fn timestamp_column() -> Self::TimestampColumn;
}
