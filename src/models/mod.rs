use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement};

mod models;
mod queries;
pub mod tables;

pub use models::{Bucket, Interface, Options, Stats, Summary, Top, Totals};
pub use queries::{FilterForm, OptionsForm};
pub use tables::{filters::Model as Filter, log::Model as Log};

pub async fn init_db(path: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(format!("sqlite:{path}")).await?;
    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA query_only = ON".to_owned(),
    ))
    .await?;
    Ok(db)
}
