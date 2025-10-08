use std::{
    env,
    sync::{Arc, LazyLock, RwLock},
};

use axum::{
    Json, Router,
    extract::State,
    response::{ErrorResponse, Html},
    routing::get,
};
use chrono::{DateTime, Days, Timelike, Utc};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

pub const PUSHOVER_USER_ID: LazyLock<String> =
    LazyLock::new(|| env::var("PUSHOVER_USER_ID").unwrap());
pub const PUSHOVER_API_KEY: LazyLock<String> =
    LazyLock::new(|| env::var("PUSHOVER_API_KEY").unwrap());
pub const SOLAREDGE_SITE_ID: LazyLock<String> =
    LazyLock::new(|| env::var("SOLAREDGE_SITE_ID").unwrap());
pub const SOLAREDGE_API_KEY: LazyLock<String> =
    LazyLock::new(|| env::var("SOLAREDGE_API_KEY").unwrap());

type CachedAppState = Arc<RwLock<AppState>>;

#[derive(Debug, Clone)]
struct AppState {
    day_checked: bool,
    cache_reset: DateTime<Utc>,
    values: EnergyResponse,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let bind_addr = env::var("BIND_ADDR").expect("BIND_ADDR");

    let app_state: CachedAppState = Arc::new(RwLock::new(AppState {
        values: EnergyResponse {
            energy: Energy {
                timeUnit: "".to_string(),
                unit: "".to_string(),
                values: vec![],
            },
        },
        day_checked: false,
        cache_reset: Utc::now() - Days::new(1),
    }));

    let app = Router::new()
        .route("/api/energy", get(energy))
        .with_state(app_state)
        .route("/", get(index))
        .nest_service(
            "/static",
            ServiceBuilder::new().service(ServeDir::new("static")),
        );
    // .layer(LiveReloadLayer::new());

    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
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

async fn energy(
    State(state): State<CachedAppState>,
) -> axum::response::Result<Json<EnergyResponse>, ErrorResponse> {
    let energy_response = fetch_energy_response(state.clone()).await?;
    check_energy(state, &energy_response).await?;

    Ok(Json(energy_response))
}

async fn check_energy(
    state: CachedAppState,
    energy_response: &EnergyResponse,
) -> axum::response::Result<(), ErrorResponse> {
    let now = Utc::now();
    let hour = now.hour();
    let is_checked_today = state.read().unwrap().day_checked;
    if hour == 12 && !is_checked_today {
        let energy_at_1200 = energy_response
            .energy
            .values
            .iter()
            .find(|v| v.date.ends_with("12:00:00+02:00"))
            .map(|v| v.value)
            .flatten();
        if let Some(energy_at_1200) = energy_at_1200 {
            if energy_at_1200 == 0.0 {
                report().await?;
            }
        }
        state.write().unwrap().day_checked = true;
    }
    //reset at 00:00
    if hour == 0 && is_checked_today {
        state.write().unwrap().day_checked = false;
    }
    Ok(())
}
async fn report() -> axum::response::Result<(), ErrorResponse> {
    let url = "https://api.pushover.net/1/messages.json";
    let form = reqwest::multipart::Form::new()
        .text("token", PUSHOVER_API_KEY.to_string())
        .text("user", PUSHOVER_USER_ID.to_string())
        .text("message", "No energy measured on the solar panels");

    let client = reqwest::Client::new();
    let _ = client
        .post(url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| ErrorResponse::from(e.to_string()))?;
    Ok(())
}

async fn fetch_energy_response(state: CachedAppState) -> axum::response::Result<EnergyResponse> {
    let reset_ts = state.read().unwrap().cache_reset;
    let now = Utc::now();
    if now.signed_duration_since(reset_ts).as_seconds_f32() > 300.0 {
        state.write().unwrap().cache_reset = now;

        let url = format!(
            "https://monitoringapi.solaredge.com/site/{}/energy?timeUnit=QUARTER_OF_AN_HOUR&endDate={}&startDate={}&api_key={}",
            SOLAREDGE_SITE_ID.as_str(),
            now.date_naive(),
            now.date_naive(),
            SOLAREDGE_API_KEY.as_str(),
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
        state.write().unwrap().values = energy_response.clone();
        Ok(energy_response)
    } else {
        Ok(state.read().unwrap().values.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EnergyResponse {
    energy: Energy,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Energy {
    timeUnit: String,
    unit: String,
    values: Vec<EnergyValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EnergyValue {
    date: String,
    value: Option<f32>,
}
