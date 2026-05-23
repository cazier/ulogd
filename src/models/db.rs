use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

pub async fn init_db() -> DatabaseConnection {
    let db = Database::connect("sqlite:./firewall.db").await.unwrap();
    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA query_only = ON".to_owned(),
    ))
    .await
    .unwrap();
    db
}
