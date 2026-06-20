#[macro_use]
extern crate rocket;
use std::sync::atomic::AtomicI64;

use clap::Parser;
use rocket::{Config, State, response::content, serde::json::Json};
use sea_orm::DatabaseConnection;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

#[derive(Parser)]
#[command(about = "Firewall traffic visualizer")]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0")]
    address: String,
    #[arg(short, long, default_value_t = 8000)]
    port: u16,
    #[arg(short, long, default_value = "./firewall.db")]
    db: String,
}

use firewalleye::models::{
    Bucket, FilterForm, Interface, Log, Options, OptionsForm, Stats, Summary, Top, Totals,
};

#[derive(OpenApi)]
#[openapi(
    paths(stats, summary, live, options),
    components(schemas(Stats, Summary, Options, Log, Bucket, Interface, Top, Totals))
)]
struct ApiDoc;

#[get("/")]
fn index() -> content::RawHtml<&'static str> {
    content::RawHtml(include_str!("static/index.html"))
}

#[get("/openapi.json")]
fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[utoipa::path(get, path = "/api/stats", params(FilterForm), responses((status = 200, body = Stats)))]
#[get("/stats?<filter..>")]
async fn stats(filter: FilterForm, db: &State<DatabaseConnection>) -> Json<Stats> {
    return Json(filter.stats(db.inner()).await.unwrap());
}

#[utoipa::path(get, path = "/api/summary", params(FilterForm), responses((status = 200, body = Summary)))]
#[get("/summary?<filter..>")]
async fn summary(filter: FilterForm, db: &State<DatabaseConnection>) -> Json<Summary> {
    return Json(filter.summary(db.inner()).await.unwrap());
}

#[utoipa::path(get, path = "/api/live", params(FilterForm), responses((status = 200, body = Vec<Log>)))]
#[get("/live?<filter..>")]
async fn live(
    filter: FilterForm,
    db: &State<DatabaseConnection>,
    row: &State<AtomicI64>,
) -> Json<Vec<Log>> {
    return Json(filter.live(db.inner(), row.inner()).await.unwrap());
}

#[utoipa::path(get, path = "/api/options", params(OptionsForm), responses((status = 200, body = Options)))]
#[get("/options?<options..>")]
async fn options(options: OptionsForm, db: &State<DatabaseConnection>) -> Json<Options> {
    return Json(options.query(db.inner()).await.unwrap());
}

#[launch]
async fn rocket() -> _ {
    let args = Args::parse();

    let db = firewalleye::models::init_db(&args.db)
        .await
        .expect("failed to connect to database");

    let row = AtomicI64::new(0);

    let config = Config {
        address: args.address.parse().expect("invalid bind address"),
        port: args.port,
        ..Config::default()
    };

    rocket::custom(config)
        .mount("/", routes![index])
        .mount("/api", routes![openapi_json, live, stats, summary, options])
        .mount("/docs", Scalar::with_url("/scalar", ApiDoc::openapi()))
        .manage(db)
        .manage(row)
}
