use crate::oauth2::authenticator::Authenticator;
use google_calendar3::api::Event;
use google_calendar3::{CalendarHub, Error};
use hyper::Client;
use std::collections::HashMap;

static MAIN_CALENDAR: &str =
    "c12717e59b8cbf4e58b2eb5b0fe0e8aa823cf71943cab642507715cd86db80f8@group.calendar.google.com";
static FILTERED_CALENDAR: &str =
    "70ff6ebc2f94e898b99fa265e71b4d8cd7f2087728d78e9e75f537813b678974@group.calendar.google.com";

#[derive(Default)]
pub struct Events {
    hub: Option<CalendarHub<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>>,
    main_sync_token: Option<String>,
    filtered_sync_token: Option<String>,
    main_by_id: HashMap<String, Event>,
    filter_by_id: HashMap<String, Event>,
    filter_to_orig: HashMap<String, String>,
    orig_to_filter: HashMap<String, String>,
}

impl Events {
    pub fn new(
        auth: Authenticator<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    ) -> Self {
        let client = Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .build(),
        );

        let hub = Some(CalendarHub::new(client, auth));
        Self {
            hub,
            ..Default::default()
        }
    }

    pub async fn all_events(
        &self,
        cal_id: &str,
        sync_token: Option<String>,
    ) -> Result<(Option<String>, Vec<Event>), Error> {
        let mut result = Vec::new();
        let mut response;
        let mut new_sync_token = None;
        if let Some(hub) = &self.hub {
            let mut builder = hub.events().list(cal_id);
            if let Some(token) = sync_token {
                builder = builder.sync_token(&token);
            }
            response = builder.doit().await?;
            loop {
                if let Some(mut items) = response.1.items {
                    result.append(&mut items);
                };
                if let Some(token) = response.1.next_page_token {
                    response = hub.events().list(cal_id).page_token(&token).doit().await?;
                } else {
                    new_sync_token = response.1.next_sync_token;
                    break;
                }
            }
        }
        Ok((new_sync_token, result))
    }

    pub async fn load_main(&mut self) -> Result<(), Error> {
        let token = self.main_sync_token.clone();
        let (sync, events) = self.all_events(MAIN_CALENDAR, token).await?;
        self.main_sync_token = sync;
        for evt in events {
            if Some("cancelled") == evt.status.as_deref() {
                self.main_by_id.remove(&evt.id.unwrap());
            } else {
                self.main_by_id
                    .insert(evt.id.as_ref().unwrap().clone(), evt);
            }
        }
        Ok(())
    }

    pub async fn load_filtered(&mut self) -> Result<(), Error> {
        let (sync, events) = self.all_events(FILTERED_CALENDAR, None).await?;
        self.filtered_sync_token = sync;
        for evt in events {
            self.filter_by_id
                .insert(evt.id.as_ref().unwrap().clone(), evt);
        }
        Ok(())
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

    pub fn events_with_description(&self) -> Vec<Event> {
        self.main_by_id
            .values()
            .filter(|e| e.description.is_some())
            .cloned()
            .collect::<Vec<_>>()
    }
    pub async fn delete_filtered_events(&mut self) -> Result<(), Error> {
        if let Some(hub) = &self.hub {
            for id in self.ids_to_delete() {
                hub.events().delete(FILTERED_CALENDAR, &id).doit().await?;
                println!("Deleted id {}", id);
            }
        }
        Ok(())
    }
    pub async fn update_filtered_event(
        &mut self,
        orig: &Event,
        filter_id: &str,
    ) -> Result<(), Error> {
        if let Some(hub) = &self.hub {
            let filter = self.filter_by_id.get(filter_id).unwrap();
            let new_evt = populate_event(orig, Default::default());
            if is_different(&new_evt, filter) {
                println!("Updating {}", &new_evt.summary.as_ref().unwrap());
                hub.events()
                    .update(
                        new_evt.clone(),
                        FILTERED_CALENDAR,
                        filter.id.as_ref().unwrap(),
                    )
                    .doit()
                    .await?;
                self.filter_by_id.insert(filter_id.to_owned(), new_evt);
            } else {
                println!("Skipped update for {}", &new_evt.summary.unwrap());
            }
        }
        Ok(())
    }

    pub async fn add_filtered_event(&mut self, evt: &Event) -> Result<(), Error> {
        if let Some(hub) = &self.hub {
            let fevt = populate_event(evt, Default::default());
            hub.events().insert(fevt, FILTERED_CALENDAR).doit().await?;
        }
        Ok(())
    }
}

pub fn is_different(evt1: &Event, evt2: &Event) -> bool {
    if get_source_id(evt1) != get_source_id(evt2) {
        return true;
    }
    if evt1.summary != evt2.summary {
        return true;
    }
    if evt1.start.as_ref().unwrap().date_time != evt2.start.as_ref().unwrap().date_time {
        return true;
    }
    if evt1.end.as_ref().unwrap().date_time != evt2.end.as_ref().unwrap().date_time {
        return true;
    }
    if evt1.color_id != evt2.color_id {
        return true;
    }
    false
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
    filter.color_id = Some(color_id(orig).to_owned());
    filter.end = orig.end.clone();
    filter.start = orig.start.clone();
    filter
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
            "Barras Art & Design (BAoD)" => "Barras",
            "CCA Cinema" => "CCA",
            _ => "?",
        })
        .to_owned()
    } else {
        "?".to_owned()
    }
}

pub fn color_id(evt: &Event) -> &str {
    if let Some(loc) = &evt.location {
        match &loc[..] {
            "GFT 1" => "1",
            "GFT 2" => "2",
            "GFT 3" => "3",
            "Cineworld Screen 1" => "4",
            "Cineworld Screen 2" => "5",
            "Cottiers" => "6",
            "Barras Art & Design (BAoD)" => "7",
            "CCA Cinema" => "8",
            _ => "10",
        }
    } else {
        "10"
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
