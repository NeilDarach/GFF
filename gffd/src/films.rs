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
    // Error connecting to the GFT web server
    #[error("Error connecting to the web server - {0}")]
    WebError(String),
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

#[derive(Deserialize, Debug)]
struct Screening {
    id: String,
    #[serde(default)]
    movie_id: u32,
    #[serde(deserialize_with = "deserialize_datetime")]
    time: chrono::NaiveDateTime,
    #[serde(rename = "screenId")]
    screen_id: String,
    #[serde(rename = "showingBadgeIds")]
    showing_badge_ids: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Movie {
    id: String,
    name: String,
    #[serde(rename = "posterImage")]
    poster_image: String,
    synopsis: String,
    #[serde(default)]
    staring: String,
    #[serde(default, rename = "directedBy")]
    directed_by: String,
    duration: u32,
    #[serde(default, rename = "all_genres")]
    all_genres: String,
    #[serde(default)]
    rating: String,
    #[serde(default, rename = "ratingReason")]
    rating_reason: Option<String>,
}

impl FestivalEvent {
    pub fn fetch_from_gft(movie_id: u32) -> Result<Vec<Self>, FilmError> {
        let movie = fetch_film(movie_id)?;
        let screenings = fetch_screenings(movie_id)?;
        let result = vec![];
        for screening in screenings {}
        Ok(result)
    }
}

fn serialize_date<S>(date: &NaiveDate, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&date.format("%Y-%m-%d").to_string())
}

fn serialize_time<S>(time: &NaiveTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&time.format("%H:%M").to_string())
}

pub fn fetch_screenings(id: u32) -> Result<Vec<Screening>, FilmError> {
    let graphql = format!(
        r#"{{ "variables": {{ "movieId": "{}" }}, "query": "query ($movieId: ID) {{ showingsForDate( movieId: $movieId ) {{ data {{ id time screenId showingBadgeIds }}  }} }}" }}"#,
        id,
    );
    let string = fetch_from_gft(&graphql)?;
    let value: Value = serde_json::from_str(&string)
        .map_err(|_e| FilmError::BadValueType("Decoding screening string".to_string()))?;
    let data = value
        .get("data")
        .ok_or(FilmError::BadValueType(
            "screening does not contain data".to_string(),
        ))?
        .get("showingsForDate")
        .ok_or(FilmError::BadValueType(
            "screening does not contain showingForDate".to_string(),
        ))?
        .get("data")
        .ok_or(FilmError::BadValueType(
            "screening does not contain second data".to_string(),
        ))?
        .as_array()
        .ok_or(FilmError::BadValueType("data is not an array".to_string()))?;
    let mut result = vec![];
    for each in data {
        let mut screening: Screening = serde_json::from_value(each.clone())
            .map_err(|e| FilmError::BadValueType(format!("not a valid screening - {:?}", e)))?;
        screening.movie_id = id;
        result.push(screening);
    }
    Ok(result)
}

pub fn fetch_ids() -> Result<String, FilmError> {
    let graphql = r#"{"variables":{"type":"now-playing-and-coming-soon","orderBy":"name", "descending":false,"limit":255,"titleClassIds":[196,211,229]},"query":"query ($limit: Int, $orderBy: String, $descending: Boolean, $titleClassIds: [ID]) { movies( limit: $limit orderBy: $orderBy descending: $descending titleClassIds: $titleClassIds ) { data { id name } } } "}"#;
    fetch_from_gft(&graphql)
}

pub fn fetch_film(id: u32) -> Result<Movie, FilmError> {
    let graphql = format!(
        r#"{{ "variables": {{ "movieId": "{}" }}, "query": "query ($movieId: ID) {{ showingsForDate( movieId: $movieId ) {{ data {{ movie {{ id name posterImage synopsis starring directedBy duration allGenres rating ratingReason  }}  }}  }} }} " }} "#,
        id,
    );
    let string = fetch_from_gft(&graphql)?;
    let value: Value = serde_json::from_str(&string)
        .map_err(|_e| FilmError::BadValueType("Decoding movie string".to_string()))?;
    let data = value
        .get("data")
        .ok_or(FilmError::BadValueType(
            "movie does not contain data".to_string(),
        ))?
        .get("showingsForDate")
        .ok_or(FilmError::BadValueType(
            "movie does not contain showingForDate".to_string(),
        ))?
        .get("data")
        .ok_or(FilmError::BadValueType(
            "movie does not contain second data".to_string(),
        ))?
        .as_array()
        .ok_or(FilmError::BadValueType("data is not an array".to_string()))?;
    let movie_val: &Value = data
        .first()
        .ok_or(FilmError::BadValueType("not a valid movie (i)".to_string()))?
        .get("movie")
        .ok_or(FilmError::BadValueType("no embedded movie".to_string()))?;
    let movie: Movie = serde_json::from_value(movie_val.clone())
        .map_err(|e| FilmError::BadValueType(format!("not a valid movie - {:?}", e)))?;
    Ok(movie)
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

fn deserialize_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&buf, "%Y-%m-%d").map_err(serde::de::Error::custom)
}

fn deserialize_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    let date =
        NaiveDate::parse_from_str(&buf[..10], "%Y-%m-%d").map_err(serde::de::Error::custom)?;
    let time =
        NaiveTime::parse_from_str(&buf[11..16], "%H:%M").map_err(serde::de::Error::custom)?;
    Ok(NaiveDateTime::new(date, time))
    //Ok(NaiveDateTime::from_timestamp(0, 0))
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    NaiveTime::parse_from_str(&buf, "%H:%M").map_err(serde::de::Error::custom)
}

pub fn fetch_from_gft(graphql: &str) -> Result<String, FilmError> {
    let client = reqwest::blocking::Client::new();
    client.post("https://www.glasgowfilm.org/graphql")
    .body(graphql.to_string())
    .header(reqwest::header::USER_AGENT,"User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:122.0) Gecko/20100101 Firefox/122.0")
    .header(reqwest::header::ACCEPT,"*/*")
    .header(reqwest::header::CONTENT_TYPE,"application/json")
    .header("site-id","103")
    .header("client-type","consumer")
        .send().map_err(|e| {FilmError::WebError(format!("{}",e))})?
        .text().map_err(|e| {FilmError::WebError(format!("{}",e))})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_load_ids() {
        let map = load_ids(
            r#"{"data":{"movies":{"data":[{"id":"33606","name":"A Fox Under a Pink Moon"},{"id":"33607","name":"A Place For Her"}],"resultVersion":"3538720778"}}}"#,
        ).unwrap();
        assert_eq!(2, map.len());
        println!("{:?}", &map);
        assert!(false);
    }
    #[test]
    fn test_fetch_film() {
        println!("{:?}", &fetch_film(33606));
        println!("{:?}", &fetch_screenings(33606));
        assert!(false);
    }

    fn test_fetch_ids() {
        if let Ok(ids) = fetch_ids() {
            println!("{:?}", load_ids(&ids));
        }
        assert!(false);
    }
}
