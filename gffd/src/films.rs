use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FilmError {
    // Bad date format
    #[error("Unable to parse {0} as a date")]
    BadDate(String),
    // Bad time format
    #[error("Unable to parse {0} as a time")]
    BadTime(String),
    // Missing field decoding unstructured json
    #[error("Expected to find the field {0} in the input")]
    MissingField(String),
    // Type error decoding unstructured json
    #[error("Expected to convert a value to {0} in the input")]
    BadValueType(String),
}

#[derive(Serialize, Deserialize, Default)]
pub struct FilmMap {
    id_to_film: HashMap<u32, String>,
    film_to_id: HashMap<String, u32>,
}

impl FilmMap {
    pub fn add(&mut self, film: &str, id: u32) {
        self.film_to_id.insert(film.to_string(), id);
        self.id_to_film.insert(id, film.to_string());
    }
    pub fn len(&self) -> usize {
        self.film_to_id.len()
    }
}
impl Debug for FilmMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct FestivalEvent {
    #[serde(
        deserialize_with = "deserialize_date",
        serialize_with = "serialize_date"
    )]
    pub date: NaiveDate,
    #[serde(
        deserialize_with = "deserialize_time",
        serialize_with = "serialize_time"
    )]
    pub start: NaiveTime,
    #[serde(
        deserialize_with = "deserialize_time",
        serialize_with = "serialize_time"
    )]
    pub end: NaiveTime,
    pub id: u32,
    pub title: String,
    pub strand: String,
    pub screen: String,
    pub attendees: Vec<String>,
    pub synopsis: String,
    pub staring: Vec<String>,
    pub genres: Vec<String>,
    pub director: String,
    pub rating: String,
    pub rating_reason: String,
    pub poster: String,
}

fn serialize_date<S>(date: &NaiveDate, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&date.format("%Y-%m-%d").to_string())
}

fn deserialize_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&buf, "%Y-%m-%d").map_err(serde::de::Error::custom)
}

fn serialize_time<S>(time: &NaiveTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&time.format("%H:%M").to_string())
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    NaiveTime::parse_from_str(&buf, "%H:%M").map_err(serde::de::Error::custom)
}

pub fn fetch_ids() -> String {
    let client = reqwest::blocking::Client::new();
    client.post("https://www.glasgowfilm.org/graphql")
    .body(r#"{"operationName":null,"variables":{"searchString":"","type":"now-playing-and-coming-soon","subtype":"watched","orderBy":"name", "descending":false,"limit":255,"titleClassId":null,"titleClassIds":[196,211,229]},"query":"query ($limit: Int, $orderBy: String, $descending: Boolean, $searchString: String, $titleClassId: ID, $titleClassIds: [ID], $type: String, $subtype: String) { movies( limit: $limit orderBy: $orderBy descending: $descending searchString: $searchString titleClassId: $titleClassId titleClassIds: $titleClassIds type: $type subtype: $subtype ) { data { id name } } } "}"#)
    .header(reqwest::header::USER_AGENT,"User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:122.0) Gecko/20100101 Firefox/122.0")
    .header(reqwest::header::ACCEPT,"*/*")
    .header(reqwest::header::CONTENT_TYPE,"application/json")
    .header("site-id","103")
    .header("client-type","consumer")
        .send().unwrap().text().unwrap()
}

pub fn load_ids(data: &str) -> Result<FilmMap, FilmError> {
    let full: Value = serde_json::from_str(data).unwrap();
    let list = full
        .get("data")
        .ok_or(FilmError::MissingField("data".to_string()))?
        .get("movies")
        .ok_or(FilmError::MissingField("movies".to_string()))?
        .get("data")
        .ok_or(FilmError::MissingField("data".to_string()))?;
    let map = list
        .as_array()
        .ok_or(FilmError::BadValueType("array".to_string()))?
        .iter()
        .filter_map(|e| e.as_object())
        .fold(FilmMap::default(), |mut m, e| {
            let id = e.get("id").unwrap();
            let name = e.get("name").unwrap();
            let id = id.as_str().unwrap();
            let name = name.as_str().unwrap();
            let id = id.parse().unwrap();
            m.add(name, id);
            m
        });
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_load_ids() {
        let map = load_ids(
            r#"{"data":{"movies":{"data":[{"id":"33606","name":"A Fox Under a Pink Moon"},{"id":"33607","name":"A Place For Her"}],"resultVersion":"3538720778"}}}"#,
        ).unwrap();
        assert_eq!(2, map.len());
        println!("{:?}", &map);
        assert!(false);
    }
    #[test]
    fn test_fetch_ids() {
        println!("{:?}", load_ids(&fetch_ids()));
        assert!(false);
    }
}
