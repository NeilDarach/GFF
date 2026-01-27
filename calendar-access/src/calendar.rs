use google_calendar3::api::{Channel, Event};
use google_calendar3::{CalendarHub, Error};
use hyper::client::HttpConnector;
use hyper::Client;
use hyper_rustls::HttpsConnector;
use serde::Serialize;
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;
use yup_oauth2::authenticator::Authenticator;
use std::env::var;
use std::sync::LazyLock;

static MAIN_CALENDAR: LazyLock<String> = LazyLock::new(|| { var("GFF_FULL_ID").expect("GFF_FULL_ID should be set to a google calendar id")});
static FILTERED_CALENDAR: LazyLock<String> = LazyLock::new(|| { var("GFF_FILTER_ID").expect("GFF_FILTER_ID should be set to a google calendar id")});
static CALLBACK_URL: LazyLock<String> = LazyLock::new(|| { var("GFF_CALLBACK").expect("GFF_CALLBACK should be set to an external url for google to notify changes to")});

/*
* Main calendar with all events.  Each event has an id (main_id).
* Filter calendar with some events.  Each event has an id (filter_id), and an extended property
* containing a main_id.
* Delete all filter events where a) there is no corresponding main event, or b) there are no people
* going to the main event
* For each main event where someone is going a) if there is no corresponding filter event, create
* one, or b) if the filter event is out of date, update it.
*
*
* main_events - HashMap<main_id,Event>
* filter_events - HashMap<filter_id, Event>
* main_to_filter - HashMap<main_id,filter_id>
*/

#[derive(Default,Serialize)]
pub struct Summary {
    pub start: String,
    pub title: String,
    pub strand: Option<String>,
    pub duration: u16,
    pub color: Option<String>,
    pub id: Option<String>,
    pub day: Option<String>,
    pub attendees: Vec<String>,
}

#[derive(Default, Serialize)]
pub struct Events {
    #[serde(skip_serializing)]
    hub: Option<CalendarHub<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>>,
    pub uuid: String,
    watch_ids: Option<(String, String)>,
    main_sync_token: Option<String>,
    filtered_sync_token: Option<String>,
    main_events: HashMap<String, Event>,
    filter_events: HashMap<String, Event>,
    main_to_filter: HashMap<String, String>,
}

impl Events {
    pub fn new(
        client: Client<HttpsConnector<HttpConnector>>,
        auth: Authenticator<HttpsConnector<HttpConnector>>,
    ) -> Self {
        let uuid = Uuid::new_v4().to_string();

        let hub = Some(CalendarHub::new(client, auth));
        Self {
            hub,
            uuid,
            ..Default::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.main_events.is_empty()
    }

    pub async fn renew_watch(&mut self, period: u128) -> Result<(), Error> {
        println!("Renew_watch called");
        if let Some(hub) = &self.hub {
            if let Some((id, resource_id)) = &self.watch_ids {
                let req = Channel {
                    id: Some(id.clone()),
                    token: Some("gff2024".to_owned()),
                    type_: Some("webhook".to_owned()),
                    resource_id: Some(resource_id.clone()),
                    ..Default::default()
                };
                println!("req: {:?}", req);
                let result = hub.channels().stop(req).doit().await;
                println!("result: {:?}", result);
                self.watch_ids = None;
            }

            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            println!("time is {}", now);
            let expiry = now + (period as u64 / 1000) + 10;
            let mut map = HashMap::default();
            map.insert("ttl".to_owned(), format!("{}", expiry).to_owned());
            let req = Channel {
                id: Some(self.uuid.clone()),
                token: Some("gff2024".to_owned()),
                address: Some(format!("{}/{}",&*CALLBACK_URL, self.uuid).to_owned()),
                params: Some(map),
                type_: Some("webhook".to_owned()),
                ..Default::default()
            };
            println!("Creating watch with {:?}", req);
            let result = hub.events().watch(req, &*MAIN_CALENDAR).doit().await;
            println!("Created watch with {:?}", result);
            if let Ok(res) = result {
                self.watch_ids = Some((res.1.id.unwrap(), res.1.resource_id.unwrap()));
            }
        }
        Ok(())
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
            let mut builder = hub
                .events()
                .list(cal_id)
                .time_min(chrono::Utc::now() - chrono::Duration::days(1));
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
        let (_, events) = self.all_events(&*MAIN_CALENDAR, None).await?;
        for evt in events {
            if Some("cancelled") != evt.status.as_deref() {
                self.main_events
                    .insert(evt.id.as_ref().unwrap().clone(), evt);
            }
        }
        Ok(())
    }

    pub async fn load_filtered(&mut self) -> Result<(), Error> {
        let (_, events) = self.all_events(&*FILTERED_CALENDAR, None).await?;
        for evt in events {
            if Some("cancelled") != evt.status.as_deref() {
                self.filter_events
                    .insert(evt.id.as_ref().unwrap().clone(), evt);
            }
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
        for id in self.filter_events.keys() {
            if let Some(evt) = self.filter_events.get(id) {
                if let Some(orig) = Self::get_orig_id(evt) {
                    self.main_to_filter.insert(orig.clone(), id.clone());
                }
            }
        }
    }

    pub fn filtered_event_for(&self, id: &str) -> Option<String> {
        self.main_to_filter.get(id).cloned()
    }

    // If there's a filter event that doesn't have an original back-ref
    // or the back-ref doesn't exist in the main calendar
    pub fn should_delete_filter(&self, filter_id: &str) -> bool {
        if let Some(filter_evt) = self.filter_events.get(filter_id) {
            println!(
                "should_delete_filter: {}",
                filter_evt.summary.as_ref().unwrap()
            );
            match Self::get_orig_id(filter_evt) {
                None => {
                    // Event doesn't have a backreference
                    println!("  No backreference, deleting");
                    return true;
                }
                Some(orig_id) => {
                    println!("  Orig id is {}", orig_id);
                    match self.main_events.get(&orig_id) {
                        Some(main_evt) => {
                            println!(
                                "  Main event is {}, people = {}",
                                main_evt.summary.as_ref().unwrap(),
                                people(main_evt)
                            );
                            if people(main_evt).is_empty() {
                                println!("  No people, deleting");
                                // Event exists, but no one is going
                                return true;
                            }
                        }
                        None => {
                            println!("  No original event with that id, deleting");
                            // Event has a back reference but no main event exists
                            return true;
                        }
                    }
                }
            }
        };
        println!("  Keeping");
        false
    }

    pub fn ids_to_delete(&self) -> Vec<String> {
        let mut result = Vec::new();
        for id in self.filter_events.keys() {
            if self.should_delete_filter(id) {
                result.push(id.clone())
            }
        }
        result
    }

    pub fn events_with_description(&self) -> Vec<Event> {
        self.main_events
            .values()
            .filter(|e| e.description.is_some())
            .cloned()
            .collect::<Vec<_>>()
    }
    pub async fn delete_filtered_events(&mut self) -> Result<(), Error> {
        if let Some(hub) = &self.hub {
            for id in self.ids_to_delete() {
                let filter_evt = self.filter_events.get(&id).unwrap();
                let orig_id = Self::get_orig_id(filter_evt).unwrap();
                self.main_to_filter.remove(&orig_id);
                hub.events().delete(&*FILTERED_CALENDAR, &id).doit().await?;
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
            let filter = self.filter_events.get(filter_id).unwrap();
            let new_evt = populate_event(orig, Default::default());
            if is_different(&new_evt, filter) {
                println!("Updating {}", &new_evt.summary.as_ref().unwrap());
                hub.events()
                    .update(
                        new_evt.clone(),
                        &*FILTERED_CALENDAR,
                        filter.id.as_ref().unwrap(),
                    )
                    .doit()
                    .await?;
            } else {
                println!("Skipped update for {}", &new_evt.summary.unwrap());
            }
        }
        Ok(())
    }

    pub async fn add_filtered_event(&mut self, evt: &Event) -> Result<(), Error> {
        if let Some(hub) = &self.hub {
            let fevt = populate_event(evt, Default::default());
            hub.events().insert(fevt, &*FILTERED_CALENDAR).doit().await?;
        }
        Ok(())
    }

    pub async fn scan_calendar(&mut self) -> Result<(), Error> {
        self.main_sync_token = None;
        self.filtered_sync_token = None;
        self.main_events = Default::default();
        self.filter_events = Default::default();
        self.main_to_filter = Default::default();
        self.load_main().await?;
        self.load_filtered().await?;
        self.create_references();
        self.delete_filtered_events().await?;
        println!("Finished scan");
        Ok(())
    }


    fn properties(&self,evt: &Event) -> HashMap<String,String> {
        let mut result = HashMap::new();
        if let Some(ext) = evt.extended_properties.as_ref() {
        if let Some(shared) = ext.shared.as_ref() {
        if let Some(id) = shared.get("sourceid") { result.insert("id".to_string(),id.to_string()); }
        if let Some(screen) = shared.get("screen") { result.insert("screen".to_string(),screen.to_string()); }
        if let Some(strand) = shared.get("strand") { result.insert("strand".to_string(),strand.to_string()); }
        if let Some(color) = shared.get("color") { result.insert("color".to_string(),color.to_string()); }
            }
        }
        return result;
    }

    pub async fn fetch_summary(&self) -> Result<HashMap<String,HashMap<String,Vec<Summary>>>,Error> {
        let mut by_date = HashMap::new();
        for (_key,evt) in &self.main_events {
            if let Some(_desc) = &evt.description {
            let start = &evt.start.as_ref().unwrap().date_time.unwrap();
            let end = &evt.end.as_ref().unwrap().date_time.unwrap();
                let duration = *end - start;

            let date = start.to_string()[..10].to_string();
                let time = start.to_string()[11..16].to_string();
                let screen = &evt.location.as_ref().unwrap().to_string();
                let properties = self.properties(evt);
                

            let by_screen = by_date.entry(date).or_insert(HashMap::new());
                let entries = by_screen.entry(screen.clone()).or_insert(Vec::new());
                let summary = Summary {
                    start: time,
                    title: evt.summary.as_ref().unwrap().to_string(),
                    strand: properties.get("strand").cloned(),
                    duration: duration.num_seconds() as u16,
                    color: properties.get("color").cloned(),
                id: None,
                day: Some(start.format("%A").to_string()),
                    attendees: people(&evt).split(" ").map(|e| e.to_string()).collect(),
                };
                entries.push(summary);
            }
        }
        return Ok(by_date);
    }

    pub async fn update_filtered_events(&mut self) -> Result<(), Error> {
        let events = self.events_with_description();
        for evt in events {
            match if let Some(filter_id) = self.filtered_event_for(evt.id.as_ref().unwrap()) {
                self.update_filtered_event(&evt, &filter_id).await
            } else {
                self.add_filtered_event(&evt).await
            } {
                Ok(_) => {
                    println!("Update ok for {}", &evt.summary.unwrap());
                }
                Err(e) => {
                    println!("Error in updating - {:?}", e);
                }
            }
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

pub fn set_extended_properties(evt: &mut Event, orig: &Event) {
    if evt.extended_properties.is_none() {
        evt.extended_properties = Some(Default::default());
    }
    if evt.extended_properties.as_ref().unwrap().shared.is_none() {
        evt.extended_properties.as_mut().unwrap().shared = Some(Default::default());
    }

    let id = orig.id.as_ref().unwrap();
    let screen = orig.extended_properties.as_ref().unwrap().shared.as_ref().unwrap().get("screen").unwrap().to_string();
    let color = orig.extended_properties.as_ref().unwrap().shared.as_ref().unwrap().get("color").unwrap().to_string();
    let strand = orig.extended_properties.as_ref().unwrap().shared.as_ref().unwrap().get("strand").unwrap().to_string();
    let props = evt.extended_properties
        .as_mut()
        .unwrap()
        .shared
        .as_mut()
        .unwrap();

        props.insert("sourceid".to_owned(), id.to_string());
        props.insert("screen".to_owned(), screen.to_string());
        props.insert("color".to_owned(), color.to_string());
        props.insert("strand".to_owned(), strand.to_string());
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
    set_extended_properties(&mut filter, &orig);
    

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
            "Odeon 10" => "O10",
            "Odeon 11" => "O11",
            "Odeon 12" => "O12",
            "Pyramid" => "Pyr",
            "PYWC" => "PYWC",
            "Special" => "Sp",
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
            "Odeon 10" => "4",
            "Odeon 11" => "5",
            "Odeon 12" => "6",
            "Pyramid" => "7",
            "PYWC" => "8",
            "Special" => "9",
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
        "".to_owned()
    }
}
pub fn people_from_string(description: &str) -> String {
    let mut vec = description
        .split(|c: char| !c.is_alphabetic())
        .map(|e| match e {
            "Neil" | "Marion" | "Vanessa" | "Fi" | "Fiona" | "Emmzi" => &e[0..1],
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
