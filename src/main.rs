use axum::{
    Extension, Json, Router,
    response::{ErrorResponse, Html},
    routing::get,
};
use chrono::prelude::*;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let app = Router::new()
        .route("/api/energy", get(energy))
        .route("/", get(index))
        .nest_service(
            "/static",
            ServiceBuilder::new().service(ServeDir::new("static")),
        )
        .layer(LiveReloadLayer::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("server on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn index() -> Html<String> {
    let html = format!(
        r#"html<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <meta http-equiv="refresh" content="0; url=/static/index.html">
        </head>"#
    );

    Html(html)
}

async fn energy() -> axum::response::Result<Json<EnergyResponse>, ErrorResponse> {
    let site_id = std::env::var("SITE_ID").unwrap();
    let api_key = std::env::var("API_KEY").unwrap();
    let utc_now = Utc::now().date_naive();

    let url = format!(
        "https://monitoringapi.solaredge.com/site/{}/energy?timeUnit=QUARTER_OF_AN_HOUR&endDate={}&startDate={}&api_key={}",
        site_id, utc_now, utc_now, api_key,
    );

    let mut energy_response = reqwest::get(url)
        .await
        .map_err(|e| ErrorResponse::from(e.to_string()))?
        .json::<EnergyResponse>()
        .await
        .map_err(|e| ErrorResponse::from(e.to_string()))?;

    let values: Vec<EnergyValue> = energy_response
        .energy
        .values
        .iter()
        .map(|v| EnergyValue {
            date: format!("{}+02:00", v.date.replace(' ', "T")).to_string(),
            value: v.value,
        })
        .collect();

    energy_response.energy.values = values;
    Ok(Json(energy_response))
}

#[derive(Debug, Serialize, Deserialize)]
struct EnergyResponse {
    energy: Energy,
}

#[derive(Debug, Serialize, Deserialize)]
struct Energy {
    timeUnit: String,
    unit: String,
    values: Vec<EnergyValue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EnergyValue {
    date: String,
    value: Option<f32>,
}
