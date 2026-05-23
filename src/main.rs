#[macro_use]
extern crate rocket;

use firewall_vis::models::{FilterForm, Log, Stats};
use rocket::{State, response::content, serde::json::Json};
use sea_orm::DatabaseConnection;

#[get("/")]
fn index() -> content::RawHtml<&'static str> {
    content::RawHtml(include_str!("static/index.html"))
}

#[get("/api/stats?<filter..>")]
async fn api_stats(mut filter: FilterForm, db: &State<DatabaseConnection>) -> Json<Stats> {
    filter.time_range = 2u32.pow(32);
    Json(filter.stats(db.inner()).await.unwrap_or_default())
}

#[get("/api/logs?<filter..>")]
async fn api_logs(mut filter: FilterForm, db: &State<DatabaseConnection>) -> Json<Vec<Log>> {
    filter.time_range = 2u32.pow(32);
    Json(filter.query(db.inner()).await.unwrap_or_default())
}

#[launch]
async fn rocket() -> _ {
    let db = firewall_vis::models::init_db().await;

    rocket::build()
        .mount("/", routes![index, api_logs, api_stats])
        .manage(db)
}
