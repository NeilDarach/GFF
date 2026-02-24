use crate::Config;
use crate::FestivalEvent;
use chrono::DateTime;
use google_calendar3::api::{Channel, Event, Scope};
use google_calendar3::{CalendarHub, Error};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use std::fs;
use yup_oauth2::{ServiceAccountAuthenticator, parse_service_account_key};

enum EventType {
    Main,
    Filter,
}

async fn hub(cfg: &Config) -> CalendarHub<HttpsConnector<HttpConnector>> {
    let service_credentials =
        fs::read_to_string(&cfg.calendar_auth_file).expect("Unable to read auth file");

    let service_key =
        parse_service_account_key(service_credentials).expect("Bad gmail credentials");
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .unwrap()
        .https_only()
        .enable_http2()
        .build();

    let executor = hyper_util::rt::TokioExecutor::new();
    let client = Client::builder(executor).build(https);
    let auth = ServiceAccountAuthenticator::builder(service_key)
        .build()
        .await
        .expect("Failed to create an authenticator");
    let hub = CalendarHub::new(client, auth);
    hub
}

pub async fn upload_events(cfg: &Config, events: &[FestivalEvent]) -> (u32, u32) {
    let hub = hub(cfg).await;
}

pub async fn filter_summary(cfg: &Config) -> Vec<FestivalEvent> {
    let hub = hub(cfg).await;
    let query = hub
        .events()
        .list(&cfg.calendar_filter_id)
        .time_min(chrono::Utc::now() - chrono::Duration::days(21))
        .add_scope(Scope::EventReadonly);
    let mut result = vec![];
    let mut response = query.doit().await.unwrap();
    loop {
        if let Some(mut items) = response.1.items {
            result.append(&mut items);
        };
        if let Some(token) = response.1.next_page_token {
            response = hub
                .events()
                .list(&cfg.calendar_filter_id)
                .add_scope(Scope::EventReadonly)
                .page_token(&token)
                .doit()
                .await
                .unwrap()
        } else {
            break;
        }
    }
    result.into_iter().map(|e| e.into()).collect()
}

fn main_event(evt: &FestivalEvent) -> Event {
    let start = google_calendar3::api::EventDateTime {
        date: Some(evt.date.clone()),
        date_time: Some(
            evt.date
                .and_time(evt.start)
                .and_local_timezone(chrono::Utc)
                .unwrap(),
        ),
        time_zone: Some(chrono::Utc.to_string()),
    };
    let end = google_calendar3::api::EventDateTime {
        date: Some(evt.date.clone()),
        date_time: Some(
            evt.date
                .and_time(evt.end)
                .and_local_timezone(chrono::Utc)
                .unwrap(),
        ),
        time_zone: Some(chrono::Utc.to_string()),
    };
    let mut event = Event::default();
    event.color_id = Some(evt.screen_colour.to_string());
    event.description = Some(evt.screen.clone());
    event.end = Some(end);
    event.extended_properties = None;
    event.start = Some(start);
    event
}

impl From<Event> for FestivalEvent {
    fn from(value: Event) -> Self {
        let date = value.start.clone().unwrap().date_time.unwrap().date_naive();
        let start = value.start.clone().unwrap().date_time.unwrap().time();
        let end = value.end.clone().unwrap().date_time.unwrap().time();
        let movie_id = 0;
        let screening_id = 0;
        let title = value.summary.clone().unwrap();
        let strand = "".to_owned();
        let strand_id = 0;
        let strand_priority = 0;
        let strand_colour = "FFFFFF".to_owned();
        let screen = value.summary.clone().unwrap();
        let screen_id = 0;
        let screen_colour = 0;
        let attendees = vec![];
        let synopsis = "".to_owned();
        let starring = vec![];
        let genres = vec![];
        let director = "".to_owned();
        let rating = "".to_owned();
        let rating_reasons = vec![];
        let poster = "".to_owned();
        Self {
            date,
            start,
            end,
            movie_id,
            screening_id,
            title,
            strand,
            strand_id,
            strand_priority,
            strand_colour,
            screen,
            screen_id,
            screen_colour,
            attendees,
            synopsis,
            starring,
            genres,
            director,
            rating,
            rating_reasons,
            poster,
        }
    }
}
