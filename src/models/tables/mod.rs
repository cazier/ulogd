pub mod filters;
pub mod log;

use sea_orm::{ColumnTrait, EntityTrait};

pub trait HasTimestamp: EntityTrait {
    type TimestampColumn: ColumnTrait;
    fn timestamp_column() -> Self::TimestampColumn;
}
