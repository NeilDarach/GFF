use crate::Config;
use crate::FestivalEvent;
use chrono::{DateTime, Duration};
use google_calendar3::api::{Channel, Event, EventDateTime, Scope};
use google_calendar3::{CalendarHub, Error};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use std::collections::HashMap;
use std::fs;
use yup_oauth2::{parse_service_account_key, ServiceAccountAuthenticator};

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

trait Gff {
    fn get_screening_id(&self) -> u32;
    fn set_screening_id(&mut self, id: u32) -> ();
    fn get_description(&self) -> String;
    fn set_description(&mut self, desc: &str) -> ();
}

impl Gff for Event {
    fn get_description(&self) -> String {
        if let Some(ext) = self.extended_properties.as_ref() {
            if let Some(shared) = ext.shared.as_ref() {
                if let Some(id) = shared.get("description") {
                    return id.clone();
                }
            }
        }
        "".to_string()
    }
    fn get_screening_id(&self) -> u32 {
        if let Some(ext) = self.extended_properties.as_ref() {
            if let Some(shared) = ext.shared.as_ref() {
                if let Some(id) = shared.get("screening_id") {
                    return id.parse().unwrap_or(0);
                }
            }
        }
        0
    }
    fn set_description(&mut self, desc: &str) -> () {
        if self.extended_properties.is_none() {
            self.extended_properties = Some(Default::default());
        }
        if self.extended_properties.as_ref().unwrap().shared.is_none() {
            self.extended_properties.as_mut().unwrap().shared = Some(Default::default());
        }
        let props = self
            .extended_properties
            .as_mut()
            .unwrap()
            .shared
            .as_mut()
            .unwrap();
        props.insert("description".to_owned(), desc.to_owned());
    }
    fn set_screening_id(&mut self, id: u32) -> () {
        if self.extended_properties.is_none() {
            self.extended_properties = Some(Default::default());
        }
        if self.extended_properties.as_ref().unwrap().shared.is_none() {
            self.extended_properties.as_mut().unwrap().shared = Some(Default::default());
        }
        let props = self
            .extended_properties
            .as_mut()
            .unwrap()
            .shared
            .as_mut()
            .unwrap();
        props.insert("screening_id".to_owned(), format!("{}", id));
    }
}

pub async fn sync_events(cfg: &Config, _festival_events: &[FestivalEvent]) -> (u32, u32) {
    let hub = hub(cfg).await;
    let mut to_delete = vec![];
    let mut to_modify = vec![];

    let main_events = events(&hub, &cfg.calendar_main_id)
        .await
        .into_iter()
        .filter(|e| e.get_screening_id() != 0)
        .filter(|e| {
            !e.description
                .as_ref()
                .cloned()
                .unwrap_or("".to_owned())
                .is_empty()
        })
        .map(|e| (e.get_screening_id(), e))
        .collect::<HashMap<u32, Event>>();
    let filter_events = events(&hub, &cfg.calendar_filter_id)
        .await
        .into_iter()
        .filter(|e| e.get_screening_id() != 0)
        .map(|e| (e.get_screening_id(), e))
        .collect::<HashMap<u32, Event>>();
    for (id, remote) in &main_events {
        if !filter_events.contains_key(id) {
            to_modify.push(filter_event_from(remote));
        }
    }
    for (id, evt) in filter_events {
        if !main_events.contains_key(&id) {
            to_delete.push(evt.clone())
        } else {
            let mut new_evt = filter_event_from(main_events.get(&id).as_ref().cloned().unwrap());
            new_evt.id = evt.id.clone();
            if !filter_events_match(&evt, &new_evt) {
                println!("mismatch\nlocal: {:?}\nremote: {:?}", &evt, &new_evt);
                to_modify.push(new_evt);
            }
        }
    }

    for evt in &to_modify[..] {
        println!("Processing {}", &evt.summary.as_ref().cloned().unwrap());
        if evt.id.is_none() {
            println!("In/erting {:?}", &evt.clone());
            hub.events()
                .insert(evt.clone(), &cfg.calendar_filter_id)
                .doit()
                .await
                .unwrap();
        } else {
            hub.events()
                .update(
                    evt.clone(),
                    &cfg.calendar_filter_id,
                    &evt.id.as_ref().cloned().unwrap(),
                )
                .doit()
                .await
                .unwrap();
        }
    }

    for evt in &to_delete[..] {
        hub.events()
            .delete(&cfg.calendar_filter_id, &evt.id.as_ref().cloned().unwrap())
            .doit()
            .await
            .unwrap();
    }
    println!("Update: {:?}", &to_modify);
    println!("Delete: {:?}", &to_delete);
    (to_modify.len() as u32, to_delete.len() as u32)
}

pub async fn upload_events(cfg: &Config, festival_events: &[FestivalEvent]) -> (u32, u32) {
    let hub = hub(cfg).await;
    let mut to_delete = vec![];
    let mut to_update = vec![];
    let remote_events = events(&hub, &cfg.calendar_main_id)
        .await
        .into_iter()
        .filter(|e| e.get_screening_id() != 0)
        .map(|e| (e.get_screening_id(), e))
        .collect::<HashMap<u32, Event>>();
    let local_events = festival_events
        .iter()
        .map(|e| (e.screening_id, e.clone()))
        .collect::<HashMap<_, _>>();
    println!("Got {} local events", local_events.len());
    for (id, remote) in &remote_events {
        if !local_events.contains_key(id) {
            to_delete.push(remote.clone());
        } else {
            let mut evt = main_event_from(local_events.get(id).unwrap());
            evt.description = remote.description.as_ref().cloned();
            evt.id = remote.id.clone();
            if !main_events_match(&evt, remote) {
                println!("mismatch\nlocal: {:?}\nremote: {:?}", &evt, &remote);
                to_update.push(evt);
            }
        }
    }
    for (id, evt) in local_events {
        if !remote_events.contains_key(&id) {
            to_update.push(main_event_from(&evt))
        }
    }

    for evt in &to_update[..] {
        println!("Processing {}", &evt.summary.as_ref().cloned().unwrap());
        if evt.id.is_none() {
            hub.events()
                .insert(evt.clone(), &cfg.calendar_main_id)
                .doit()
                .await
                .unwrap();
        } else {
            hub.events()
                .update(
                    evt.clone(),
                    &cfg.calendar_main_id,
                    &evt.id.as_ref().cloned().unwrap(),
                )
                .doit()
                .await
                .unwrap();
        }
    }

    for evt in &to_delete[..] {
        hub.events()
            .delete(&cfg.calendar_main_id, &evt.id.as_ref().cloned().unwrap())
            .doit()
            .await
            .unwrap();
    }
    println!("Update: {:?}", &to_update);
    println!("Delete: {:?}", &to_delete);
    (to_update.len() as u32, to_delete.len() as u32)
}

fn filter_events_match(a: &Event, b: &Event) -> bool {
    if a.summary != b.summary {
        return false;
    }
    if a.color_id != b.color_id {
        return false;
    }
    if a.start.as_ref().unwrap().date_time != b.start.as_ref().unwrap().date_time {
        return false;
    }
    if a.end.as_ref().unwrap().date_time != b.end.as_ref().unwrap().date_time {
        return false;
    }

    true
}
fn main_events_match(a: &Event, b: &Event) -> bool {
    if a.summary != b.summary {
        return false;
    }
    if a.color_id != b.color_id {
        return false;
    }
    if a.location != b.location {
        return false;
    }
    if a.start.as_ref().unwrap().date_time != b.start.as_ref().unwrap().date_time {
        return false;
    }
    if a.end.as_ref().unwrap().date_time != b.end.as_ref().unwrap().date_time {
        return false;
    }

    true
}

fn initials(desc: &str) -> String {
    let mut names = vec![];
    let mut name = "".to_string();
    for c in desc.chars() {
        if c.is_alphabetic() {
            name.push(c);
        } else if !name.is_empty() {
            names.push(name);
            name = "".to_string();
        }
    }

    if !name.is_empty() {
        names.push(name);
    }
    names
        .iter()
        .map(|e| match &e[..] {
            "Patrick" => "Pt".to_string(),
            "Pam" => "Pm".to_string(),
            _ => e.chars().next().unwrap().to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn filter_event_from(evt: &Event) -> Event {
    let mut cal: Event = Default::default();
    cal.summary = Some(format!(
        "{} - {} - {}",
        &evt.summary.as_ref().cloned().unwrap_or("".to_string()),
        evt.location.as_ref().cloned().unwrap_or("".to_string()),
        initials(&evt.description.as_ref().cloned().unwrap_or("".to_string()))
    ));
    cal.set_screening_id(evt.get_screening_id());
    cal.set_description(&evt.description.as_ref().cloned().unwrap_or("".to_string()));
    cal.color_id = evt.color_id.clone();
    cal.start = evt.start.clone();
    cal.end = evt.end.clone();
    cal
}

fn main_event_from(evt: &FestivalEvent) -> Event {
    let mut cal: Event = Default::default();
    let time_zone = chrono::Utc;
    let start_date_time = evt
        .date
        .and_time(evt.start)
        .and_local_timezone(chrono::Utc)
        .unwrap();
    let mut end_date_time = evt
        .date
        .and_time(evt.end)
        .and_local_timezone(chrono::Utc)
        .unwrap();
    if evt.end < evt.start {
        end_date_time += std::time::Duration::from_hours(24);
    }
    cal.summary = Some(evt.title.clone());
    cal.location = if evt.screen.is_empty() {
        None
    } else {
        Some(evt.screen.clone())
    };
    cal.start = Some(EventDateTime {
        date: None,
        date_time: Some(start_date_time),
        time_zone: Some(time_zone.to_string()),
    });
    cal.end = Some(EventDateTime {
        date: None,
        date_time: Some(end_date_time),
        time_zone: Some(time_zone.to_string()),
    });
    cal.color_id = Some(format!("{}", evt.screen_colour));
    cal.set_screening_id(evt.screening_id);
    cal
}

pub async fn events(hub: &CalendarHub<HttpsConnector<HttpConnector>>, id: &str) -> Vec<Event> {
    let query = hub
        .events()
        .list(id)
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
                .list(id)
                .add_scope(Scope::EventReadonly)
                .page_token(&token)
                .doit()
                .await
                .unwrap()
        } else {
            break;
        }
    }
    result
}

pub async fn filter_summary(cfg: &Config) -> Vec<FestivalEvent> {
    let hub = hub(cfg).await;
    events(&hub, &cfg.calendar_filter_id)
        .await
        .into_iter()
        .map(|e| e.into())
        .collect()
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
        date: Some(evt.date),
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
