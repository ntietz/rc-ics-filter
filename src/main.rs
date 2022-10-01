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

    let filtered = filter_calendar(&cal_feed);

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

fn filter_calendar(feed: &str) -> String {
    let filtered = feed
        .split("BEGIN:")
        .filter(|s| !s.contains("STATUS:CANCELLED"))
        .collect::<Vec<_>>()
        .join("BEGIN:");
    ensure_ending_tag(&filtered)
}

const END_TAG: &'static str = "END:VCALENDAR";

fn ensure_ending_tag(feed: &str) -> String {
    let feed = feed.trim().to_owned();
    if feed.ends_with(END_TAG) {
        feed.to_string()
    } else {
        format!("{}\n{}", feed, END_TAG)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn ending_with_cancelled_works() {
        let feed = "
BEGIN:VCALENDAR
VERSION:2.0
PRODID:icalendar-ruby
CALSCALE:GREGORIAN
METHOD:PUBLISH
NAME:RC Personal Calendar
X-WR-CALNAME:RC Personal Calendar
REFRESH-INTERVAL;VALUE=DURATION:PT1M
X-PUBLISHED-TTL:PT1M
BEGIN:VTIMEZONE
TZID:America/New_York
BEGIN:DAYLIGHT
DTSTART:20220313T030000
TZOFFSETFROM:-0500
TZOFFSETTO:-0400
RRULE:FREQ=YEARLY;BYDAY=2SU;BYMONTH=3
TZNAME:EDT
END:DAYLIGHT
BEGIN:STANDARD
DTSTART:20221106T010000
TZOFFSETFROM:-0400
TZOFFSETTO:-0500
RRULE:FREQ=YEARLY;BYDAY=1SU;BYMONTH=11
TZNAME:EST
END:STANDARD
END:VTIMEZONE
BEGIN:VEVENT
DTSTAMP:20220929T105407Z
UID:calendar-event-18894@recurse.com
DTSTART;TZID=America/New_York:20221031T110000
DTEND;TZID=America/New_York:20221031T113000
STATUS:CANCELLED
END:VEVENT
END:VCALENDAR
".trim();
    let expected = "
BEGIN:VCALENDAR
VERSION:2.0
PRODID:icalendar-ruby
CALSCALE:GREGORIAN
METHOD:PUBLISH
NAME:RC Personal Calendar
X-WR-CALNAME:RC Personal Calendar
REFRESH-INTERVAL;VALUE=DURATION:PT1M
X-PUBLISHED-TTL:PT1M
BEGIN:VTIMEZONE
TZID:America/New_York
BEGIN:DAYLIGHT
DTSTART:20220313T030000
TZOFFSETFROM:-0500
TZOFFSETTO:-0400
RRULE:FREQ=YEARLY;BYDAY=2SU;BYMONTH=3
TZNAME:EDT
END:DAYLIGHT
BEGIN:STANDARD
DTSTART:20221106T010000
TZOFFSETFROM:-0400
TZOFFSETTO:-0500
RRULE:FREQ=YEARLY;BYDAY=1SU;BYMONTH=11
TZNAME:EST
END:STANDARD
END:VTIMEZONE
END:VCALENDAR
".trim();

    assert_eq!(expected, filter_calendar(feed));
    }
}
