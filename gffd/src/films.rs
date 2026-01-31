use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::LazyLock;
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
    #[serde(deserialize_with = "deserialize_str_as_u32")]
    id: u32,
    #[serde(default)]
    movie_id: u32,
    #[serde(deserialize_with = "deserialize_datetime")]
    time: chrono::NaiveDateTime,
    #[serde(rename = "screenId")]
    #[serde(deserialize_with = "deserialize_str_as_u32")]
    screen_id: u32,
    #[serde(rename = "showingBadgeIds")]
    #[serde(deserialize_with = "deserialize_v_of_str_as_u32")]
    showing_badge_ids: Vec<u32>,
}

#[derive(Deserialize, Debug)]
struct Movie {
    #[serde(deserialize_with = "deserialize_str_as_u32")]
    id: u32,
    name: String,
    #[serde(rename = "posterImage")]
    poster_image: String,
    #[serde(deserialize_with = "deserialize_markup")]
    synopsis: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_csv")]
    staring: Vec<String>,
    #[serde(default, rename = "directedBy")]
    directed_by: String,
    duration: u32,
    #[serde(deserialize_with = "deserialize_csv")]
    #[serde(default, rename = "allGenres")]
    all_genres: Vec<String>,
    #[serde(default)]
    rating: String,
    #[serde(default, rename = "ratingReason")]
    #[serde(deserialize_with = "deserialize_none_as_vec")]
    rating_reason: Vec<String>,
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
    deserialize_screenings(id, &fetch_from_gft(&graphql)?)
}
pub fn deserialize_screenings(id: u32, json: &str) -> Result<Vec<Screening>, FilmError> {
    let value: Value = serde_json::from_str(json)
        .map_err(|_e| FilmError::BadValueType("Decoding screening string".to_string()))?;
    let data = value
        .pointer("/data/showingsForDate/data")
        .and_then(|v| v.as_array())
        .ok_or(FilmError::BadValueType("no internal data".to_string()))?;
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
    let graphql = r#"{"variables":{"titleClassIds":[196,211,229]},"query":"query ($titleClassIds: [ID]) { movies( limit: 255 titleClassIds: $titleClassIds ) { data { id name } } } "}"#;
    fetch_from_gft(&graphql)
}

pub fn fetch_film(id: u32) -> Result<Movie, FilmError> {
    let graphql = format!(
        r#"{{ "variables": {{ "movieId": "{}" }}, "query": "query ($movieId: ID) {{ showingsForDate( movieId: $movieId ) {{ data {{ movie {{ id name posterImage synopsis starring directedBy duration allGenres rating ratingReason  }}  }}  }} }} " }} "#,
        id,
    );
    deserialize_film(&fetch_from_gft(&graphql)?)
}
pub fn deserialize_film(json: &str) -> Result<Movie, FilmError> {
    let value: Value = serde_json::from_str(json)
        .map_err(|_e| FilmError::BadValueType("Decoding movie string".to_string()))?;
    let movie_val = value
        .pointer("/data/showingsForDate/data/0/movie")
        .ok_or(FilmError::BadValueType("no internal data".to_string()))?;
    let movie: Movie = serde_json::from_value(movie_val.clone())
        .map_err(|e| FilmError::BadValueType(format!("not a valid movie - {:?}", e)))?;
    Ok(movie)
}

pub fn load_ids(data: &str) -> Result<FilmMap, FilmError> {
    let full: Value = serde_json::from_str(data).unwrap();
    let list = full
        .pointer("/data/movies/data")
        .and_then(|v| v.as_array())
        .ok_or(FilmError::BadValueType("no data array".to_string()))?;
    let map = list
        .iter()
        .filter_map(|e| e.as_object())
        .fold(FilmMap::default(), |mut m, e| {
            let id = e
                .get("id")
                .and_then(|v| v.as_str())
                .and_then(|v| v.parse().ok());
            let name = e.get("name").and_then(|v| v.as_str());
            m.add(name.unwrap(), id.unwrap());
            m
        });
    Ok(map)
}
fn deserialize_markup<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let source = String::deserialize(deserializer)?;
    replace_regex(&source).map_err(serde::de::Error::custom)
}

pub fn replace_regex(source: &str) -> Result<String, FilmError> {
    use regex::Regex;
    static PATTERNS: LazyLock<Vec<(Regex, &str)>> = LazyLock::new(|| {
        [
            (r"------*", "----"),
            (r"<a [^>]+>(?<match>[^<]+)</a>", "($match)"),
            (r"</?style>", ""),
            (r"</?font[^>]*>", ""),
            (r"</?[iI]/?>", "_"),
            (r"<[bB]>(?<match>[^<]*)</[bB]>", "#strong[$match]"),
            //(r"<[iI]>(?<match>[^<]*)</[iI]>", "#emph[$match]"),
            (r"</?[^>]+/?>", ""),
            (r"\$", r"\$"),
            (r"\*", r"\*"),
        ]
        .iter()
        .map(|(r, m)| (Regex::new(r).unwrap(), *m))
        .collect()
    });
    Ok(PATTERNS.iter().fold(source.to_string(), |st, re| {
        re.0.replace_all(&st, re.1).to_string()
    }))
}

fn deserialize_csv<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec = String::deserialize(deserializer)?
        .split(",")
        .map(|e| e.trim().to_owned())
        .collect();
    Ok(vec)
}
fn deserialize_none_as_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    if opt.is_none() {
        return Ok(vec![]);
    }

    let result = opt
        .unwrap()
        .split(",")
        .map(|e| e.trim().to_owned())
        .collect();
    Ok(result)
}
fn deserialize_none_as_str<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::deserialize(deserializer)? {
        Some(s) => Ok(s),
        None => Ok("".to_string()),
    }
}
fn deserialize_str_as_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let string: String = String::deserialize(deserializer)?;
    string.parse().map_err(serde::de::Error::custom)
}
fn deserialize_v_of_str_as_u32<'de, D>(deserializer: D) -> Result<Vec<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec: Vec<String> = Vec::deserialize(deserializer)?;
    let mut result = vec![];
    for each in vec {
        if let Ok(elem) = each.parse() {
            result.push(elem)
        }
    }
    Ok(result)
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
    fn test_replace_regex() {
        assert_eq!(
            "with _italics_ test".to_string(),
            replace_regex("with <i>italics</i> test").unwrap()
        );
        assert_eq!(
            "with #strong[bold] test \\* and \\$".to_string(),
            replace_regex("with <b>bold</B> test * and $").unwrap()
        );
    }

    #[test]
    fn test_fetch_film() {
        let film = r#"{"data":{"showingsForDate":{"data":[{"movie":{"id":"33606","name":"A Fox Under a Pink Moon","posterImage":"bwotd0lk7ox5bsiyvyr96yw6ub9h","synopsis":"An intimate self-portrait of teenage artist \u003cb\u003eSoraya\u003c/b\u003e, an Afghan refugee, as she tries to travel from Iran to reach her mother in Austria. A raw and intelligent collaborative work between \u003cb\u003eSoraya\u003c/b\u003e – who shoots footage on her phone – and documentarian \u003cb\u003eMehrdad Oskouei\u003c/b\u003e (\u003ci\u003eStarless Dreams\u003c/i\u003e), it was shot over five years as she struggled to make it to Europe. In addition to video diaries charting \u003cb\u003eSoraya’s\u003c/b\u003e progress, the film features animation based on her own artwork and muses on themes including exile and domestic violence, while retaining a sense of her optimistic defiance in the face of life’s injustices.","starring":"","directedBy":"Mehrdad Oskouei","duration":76,"allGenres":"Documentary,Drama,Animation","rating":"N/C 15+","ratingReason":null}},{"movie":{"id":"33606","name":"A Fox Under a Pink Moon","posterImage":"bwotd0lk7ox5bsiyvyr96yw6ub9h","synopsis":"An intimate self-portrait of teenage artist \u003cb\u003eSoraya\u003c/b\u003e, an Afghan refugee, as she tries to travel from Iran to reach her mother in Austria. A raw and intelligent collaborative work between \u003cb\u003eSoraya\u003c/b\u003e – who shoots footage on her phone – and documentarian \u003cb\u003eMehrdad Oskouei\u003c/b\u003e (\u003ci\u003eStarless Dreams\u003c/i\u003e), it was shot over five years as she struggled to make it to Europe. In addition to video diaries charting \u003cb\u003eSoraya’s\u003c/b\u003e progress, the film features animation based on her own artwork and muses on themes including exile and domestic violence, while retaining a sense of her optimistic defiance in the face of life’s injustices.","starring":"","directedBy":"Mehrdad Oskouei","duration":76,"allGenres":"Documentary,Drama,Animation","rating":"N/C 15+","ratingReason":null}}],"resultVersion":"2787629567"}}}"#;
        let screenings = r#"{"data":{"showingsForDate":{"data":[{"id":"388329","time":"2026-02-26T21:00:00Z","screenId":"175","showingBadgeIds":["827","864","852","549","560","562"]},{"id":"388644","time":"2026-02-27T15:00:00Z","screenId":"175","showingBadgeIds":["827","864","852","549","560","562"]}],"resultVersion":"2222388313"}}}"#;

        println!("{:?}", &deserialize_film(film));
        println!("{:?}", &deserialize_screenings(33606, screenings));
        assert!(false);
    }

    fn test_fetch_ids() {
        if let Ok(ids) = fetch_ids() {
            println!("{:?}", load_ids(&ids));
        }
        assert!(false);
    }
}
