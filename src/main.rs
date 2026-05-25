#[macro_use]
extern crate rocket;

use firewall_vis::models::{FilterForm, Log, Options, OptionsForm, Stats};
use rocket::{State, response::content, serde::json::Json};
use sea_orm::DatabaseConnection;

#[get("/")]
fn index() -> content::RawHtml<&'static str> {
    content::RawHtml(include_str!("static/index.html"))
}

#[get("/api/stats?<filter..>")]
async fn api_stats(filter: FilterForm, db: &State<DatabaseConnection>) -> Json<Stats> {
    return Json(filter.stats(db.inner()).await.unwrap());
}

#[get("/api/logs?<filter..>")]
async fn api_logs(filter: FilterForm, db: &State<DatabaseConnection>) -> Json<Vec<Log>> {
    return Json(filter.query(db.inner()).await.unwrap());
}

#[get("/api/options?<options..>")]
async fn options(options: OptionsForm, db: &State<DatabaseConnection>) -> Json<Options> {
    return Json(options.query(db.inner()).await.unwrap());
}

#[launch]
async fn rocket() -> _ {
    let db = firewall_vis::models::init_db()
        .await
        .expect("failed to connect to database");

    rocket::build()
        .mount("/", routes![index, api_logs, api_stats, options])
        .manage(db)
}
