use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement};

pub async fn init_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite:./firewall.db").await?;
    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA query_only = ON".to_owned(),
    ))
    .await?;
    Ok(db)
}
