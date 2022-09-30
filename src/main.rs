use std::net::SocketAddr;

use axum::{extract::Path, http::StatusCode, routing::get, Router};

#[tokio::main]
async fn main() {
    openssl_probe::init_ssl_cert_env_vars();

    let app = Router::new()
        .route("/calendar/:token", get(filtered_calendar))
        .route("/heatlh", get(health));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Starting...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health() -> &'static str {
    "up and running"
}

async fn filtered_calendar(Path(token): Path<String>) -> (StatusCode, String) {
    let cal_feed = match get_calendar(token).await {
        Ok(feed) => feed,
        Err(e) => {
            println!("error: {}", e);
            return (
                StatusCode::NOT_FOUND,
                "could not extract calendar".to_owned(),
            );
        }
    };

    let filtered = filter_calendar(cal_feed);

    (StatusCode::OK, filtered)
}

async fn get_calendar(token: String) -> Result<String, reqwest::Error> {
    let url = format!(
        "https://www.recurse.com/calendar/events.ics?token={}&scope=me",
        token
    );
    let body = reqwest::get(url).await?.text().await?;

    Ok(body)
}

fn filter_calendar(feed: String) -> String {
    feed
        .split_inclusive("BEGIN:")
        .filter(|s| !s.contains("STATUS:CANCELLED"))
        .collect()
}
