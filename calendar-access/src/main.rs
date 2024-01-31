use google_calendar3::api::Event;
use google_calendar3::{CalendarHub, Error};
use hyper::Client;
use std::collections::HashMap;
use yup_oauth2 as oauth2;

static MAIN_CALENDAR: &str =
    "c12717e59b8cbf4e58b2eb5b0fe0e8aa823cf71943cab642507715cd86db80f8@group.calendar.google.com";
static FILTERED_CALENDAR: &str =
    "70ff6ebc2f94e898b99fa265e71b4d8cd7f2087728d78e9e75f537813b678974@group.calendar.google.com";

#[derive(Default)]
struct Events {
    main_by_id: HashMap<String, Event>,
    filter_by_id: HashMap<String, Event>,
    filter_to_orig: HashMap<String, String>,
    orig_to_filter: HashMap<String, String>,
}

impl Events {
    pub fn load_main(&mut self, events: Vec<Event>) {
        for evt in events {
            self.main_by_id
                .insert(evt.id.as_ref().unwrap().clone(), evt);
        }
    }

    pub fn load_filtered(&mut self, events: Vec<Event>) {
        for evt in events {
            self.filter_by_id
                .insert(evt.id.as_ref().unwrap().clone(), evt);
        }
    }

    pub fn get_orig_id(evt: &Event) -> Option<String> {
        let ext = evt.extended_properties.as_ref()?;
        let shared = ext.shared.as_ref()?;
        let id = shared.get("sourceid")?;
        Some(id.clone())
    }
    pub fn create_references(&mut self) {
        for id in self.filter_by_id.keys() {
            if let Some(evt) = self.filter_by_id.get(id) {
                if let Some(orig) = Self::get_orig_id(evt) {
                    self.filter_to_orig.insert(id.clone(), orig.clone());
                    self.orig_to_filter.insert(orig.clone(), id.clone());
                }
            }
        }
    }

    pub fn filtered_event_for(&self, id: &str) -> Option<String> {
        self.orig_to_filter.get(id).cloned()
    }

    // If there's a filter event that doesn't have an original back-ref
    // or the back-ref doesn't exist in the main calendar
    pub fn ids_to_delete(&self) -> Vec<String> {
        let mut result = Vec::new();
        for id in self.filter_by_id.keys() {
            if let Some(orig) = self.filter_to_orig.get(id) {
                if !self.main_by_id.contains_key(orig) {
                    result.push(id.clone());
                }
            } else {
                result.push(id.clone())
            }
        }
        result
    }

    pub fn events_with_description(&self) -> Vec<&Event> {
        self.main_by_id
            .values()
            .filter(|e| e.description.is_some())
            .collect::<Vec<_>>()
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load the secret
    let secret = oauth2::read_application_secret("credentials.json")
        .await
        .expect("Client secret not loaded from credentials.json");
    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk("tokencache.json")
    .build()
    .await
    .unwrap();
    let client = Client::builder().build(
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .build(),
    );

    let hub = CalendarHub::new(client, auth);
    let mut event_struct = Events::default();
    event_struct.load_main(all_events(&hub, MAIN_CALENDAR).await?);
    event_struct.load_filtered(all_events(&hub, FILTERED_CALENDAR).await?);
    event_struct.create_references();

    for id in event_struct.ids_to_delete() {
        delete_filtered_event(&hub, &id).await?;
    }

    for evt in event_struct.events_with_description() {
        if let Some(filter_id) = event_struct.filtered_event_for(evt.id.as_ref().unwrap()) {
            update_filtered_event(
                &hub,
                evt,
                event_struct.filter_by_id.get(&filter_id).unwrap(),
            )
            .await?;
            // Update event
        } else {
            // Create event
            add_filtered_event(&hub, evt).await?;
        }
    }
    Ok(())
}

pub async fn all_events(
    hub: &CalendarHub<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    cal_id: &str,
) -> Result<Vec<Event>, Error> {
    let mut result = Vec::new();
    let mut response;
    response = hub.events().list(cal_id).doit().await?;
    loop {
        if let Some(mut items) = response.1.items {
            result.append(&mut items);
        };
        if let Some(token) = response.1.next_page_token {
            response = hub.events().list(cal_id).page_token(&token).doit().await?;
        } else {
            break;
        }
    }

    Ok(result)
}

pub async fn delete_filtered_event(
    hub: &CalendarHub<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    id: &str,
) -> Result<(), Error> {
    hub.events().delete(FILTERED_CALENDAR, id).doit().await?;
    println!("Deleted id {}", id);
    Ok(())
}

pub async fn update_filtered_event(
    hub: &CalendarHub<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    orig: &Event,
    filter: &Event,
) -> Result<(), Error> {
    println!(
        "Considering update for {}",
        filter.summary.as_ref().unwrap()
    );
    let new_evt = populate_event(orig, Default::default());
    let mut needs_update = false;
    if get_source_id(&new_evt) != get_source_id(filter) {
        println!(
            "srcid {:?} vs {:?}",
            get_source_id(&new_evt),
            get_source_id(filter)
        );
        needs_update = true;
    }
    if new_evt.summary != filter.summary {
        println!("{:?} vs {:?}", new_evt.summary, filter.summary);
        needs_update = true;
    }
    if new_evt.start.as_ref().unwrap().date_time != filter.start.as_ref().unwrap().date_time {
        println!(
            "{:?} vs {:?}",
            new_evt.start.as_ref().unwrap().date_time,
            filter.start.as_ref().unwrap().date_time
        );
        needs_update = true;
    }
    if new_evt.end.as_ref().unwrap().date_time != filter.end.as_ref().unwrap().date_time {
        println!(
            "{:?} vs {:?}",
            new_evt.end.as_ref().unwrap().date_time,
            filter.end.as_ref().unwrap().date_time
        );
        needs_update = true;
    }
    if needs_update {
        println!("Updating {}", &new_evt.summary.as_ref().unwrap());
        hub.events()
            .update(new_evt, FILTERED_CALENDAR, filter.id.as_ref().unwrap())
            .doit()
            .await?;
    } else {
        println!("Skipped update for {}", &new_evt.summary.unwrap());
    }
    Ok(())
}

pub fn set_source_id(evt: &mut Event, id: &str) {
    if evt.extended_properties.is_none() {
        evt.extended_properties = Some(Default::default());
    }
    if evt.extended_properties.as_ref().unwrap().shared.is_none() {
        evt.extended_properties.as_mut().unwrap().shared = Some(Default::default());
    }

    evt.extended_properties
        .as_mut()
        .unwrap()
        .shared
        .as_mut()
        .unwrap()
        .insert("sourceid".to_owned(), id.to_string());
}

pub fn get_source_id(evt: &Event) -> Option<String> {
    if let Some(props) = &evt.extended_properties {
        if let Some(map) = &props.shared {
            map.get("sourceid").cloned()
        } else {
            None
        }
    } else {
        None
    }
}

pub fn populate_event(orig: &Event, mut filter: Event) -> Event {
    set_source_id(&mut filter, orig.id.as_ref().unwrap());

    filter.summary =
        Some(format!("{} - {} - {}", title(orig), screen(orig), people(orig)).to_string());
    filter.end = orig.end.clone();
    filter.start = orig.start.clone();
    println!("Building new evt {}", &filter.summary.clone().unwrap());
    filter
}

pub async fn add_filtered_event(
    hub: &CalendarHub<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    evt: &Event,
) -> Result<(), Error> {
    let fevt = populate_event(evt, Default::default());
    hub.events().insert(fevt, FILTERED_CALENDAR).doit().await?;
    Ok(())
}

pub fn title(evt: &Event) -> String {
    evt.summary.clone().unwrap()
}

pub fn screen(evt: &Event) -> String {
    if let Some(loc) = &evt.location {
        (match &loc[..] {
            "GFT 1" => "GFT1",
            "GFT 2" => "GFT2",
            "GFT 3" => "GFT3",
            "Cineworld Screen 1" => "C1",
            "Cineworld Screen 2" => "C2",
            "Cottiers" => "Cot",
            "CCA Cinema" => "CCA",
            _ => "?",
        })
        .to_owned()
    } else {
        "?".to_owned()
    }
}

pub fn people(evt: &Event) -> String {
    if let Some(description) = &evt.description {
        people_from_string(description)
    } else {
        "?".to_owned()
    }
}
pub fn people_from_string(description: &str) -> String {
    let mut vec = description
        .split(|c: char| !c.is_alphabetic())
        .map(|e| match e {
            "Neil" | "Marion" | "Vanessa" | "Fi" | "Emmzi" => &e[0..1],
            "Pam" => "Pm",
            "Patrick" => "Pt",
            _ => "",
        })
        .filter(|e| !e.is_empty())
        .collect::<Vec<_>>();
    vec.sort();
    vec.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn people_1() {
        assert_eq!(people_from_string("Neil, Vanessa"), "N V");
    }
}
