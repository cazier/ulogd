use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement};

mod models;
mod queries;
pub mod tables;

pub use models::{Options, Stats, Summary};
pub use queries::{FilterForm, OptionsForm};
pub use tables::filters::Model as Filter;
pub use tables::log::Model as Log;

pub async fn init_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite:./firewall.db").await?;
    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA query_only = ON".to_owned(),
    ))
    .await?;
    Ok(db)
}
