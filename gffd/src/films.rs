use crate::config::Config;
use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fs;
use std::sync::LazyLock;
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FilmError {
    // Bad date format
    #[error("Unable to parse {0} as a date")]
    BadDate(String),
    // Bad time format
    #[error("Unable to parse {0} as a time")]
    BadTime(String),
    // Type error decoding unstructured json
    #[error("Expected to convert a value to {0} in the input")]
    BadValueType(String),
    // Error connecting to the GFT web server
    #[error("Error connecting to the web server - {0}")]
    WebError(String),
    // Error reading from the disk
    #[error("Error reading {0} from disk")]
    ReadError(String),
    // Error writing to the disk
    #[error("Error writing {0} to disk")]
    WriteError(String),
}

#[derive(Serialize, Deserialize, Default)]
pub struct FilmMap {
    pub id_to_film: HashMap<u32, String>,
    pub film_to_id: HashMap<String, u32>,
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

#[derive(Serialize, Deserialize, Debug)]
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
    pub movie_id: u32,
    pub screening_id: u32,
    pub title: String,
    pub strand: String,
    pub strand_id: u32,
    pub strand_priority: u32,
    pub strand_colour: String,
    pub screen: String,
    pub screen_id: u32,
    pub screen_colour: u32,
    pub attendees: Vec<String>,
    pub synopsis: String,
    pub starring: Vec<String>,
    pub genres: Vec<String>,
    pub director: String,
    pub rating: String,
    pub rating_reasons: Vec<String>,
    pub poster: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BrochureEntry {
    pub name: String,
    pub id: String,
    pub sortname: String,
    pub showings: Vec<Showing>,
    pub duration: u32,
    pub synopsis: String,
    pub starring: String,
    pub genres: String,
    #[serde(rename = "directedBy")]
    pub directed_by: String,
    pub rating: String,
    #[serde(rename = "ratingReason")]
    pub rating_reason: String,
    pub strand: String,
    pub poster: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Showing {
    screen: String,
    time: String,
    date: String,
    datetime: String,
}

impl BrochureEntry {
    fn sortname(title: &str) -> String {
        if let Some(t) = title.strip_prefix("The ") {
            return format!("{}{}", t, ", The");
        }
        if let Some(t) = title.strip_prefix("A ") {
            return format!("{}{}", t, ", A");
        }
        if let Some(t) = title.strip_prefix("An ") {
            return format!("{}{}", t, ", An");
        }
        if let Some(t) = title.strip_prefix("Take 2: ") {
            return format!("{}{}", Self::sortname(t), ", Take 2");
        }
        if let Some(t) = title.strip_prefix("Frightfest ") {
            return format!("{}{}", Self::sortname(t), ", Frightfest");
        }
        if let Some(t) = title.strip_prefix("Closing Gala: ") {
            return format!("{}{}", Self::sortname(t), ", Closing Gala");
        }
        if let Some(t) = title.strip_prefix("Opening Gala: ") {
            return format!("{}{}", Self::sortname(t), ", Opening Gala");
        }
        ucfirst::ucfirst(title)
    }

    pub fn from_event(events: &[FestivalEvent]) -> Self {
        let showings = events
            .iter()
            .map(|e| {
                let time = e.start.format("%H:%M").to_string();
                let date = e.date.format("%a, %B %-d").to_string();
                let datetime_date = e.date.format("%Y-%m-%d").to_string();
                let datetime = format!("{}T{}:00Z", datetime_date, &time);
                Showing {
                    screen: e.screen.clone(),
                    time,
                    date,
                    datetime,
                }
            })
            .collect::<Vec<_>>();
        let movie = events.first().unwrap();
        let sortname = Self::sortname(&movie.title);
        let mut duration = (movie.end - movie.start).num_minutes();
        if duration < 0 {
            duration += 24 * 60;
        }
        Self {
            name: movie.title.clone(),
            id: format!("{}", movie.movie_id),
            sortname,
            showings,
            duration: duration.try_into().unwrap(),
            synopsis: movie.synopsis.clone(),
            starring: movie.starring.join(", "),
            genres: movie.genres.join(", "),
            directed_by: movie.director.clone(),
            rating: movie.rating.clone(),
            rating_reason: movie.rating_reasons.join(", "),
            strand: movie.strand.clone(),
            poster: format!("posters/{}.jpg", &movie.poster),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Screening {
    id: String,
    movie_id: Option<u32>,
    time: String,
    #[serde(rename = "screenId")]
    screen_id: String,
    #[serde(rename = "showingBadgeIds")]
    showing_badge_ids: Vec<String>,
    movie: Movie,
}

#[derive(Deserialize, Debug)]
struct Movie {
    id: String,
    name: String,
    #[serde(rename = "posterImage")]
    poster_image: Option<String>,
    synopsis: String,
    #[serde(default)]
    starring: Option<String>,
    #[serde(default, rename = "directedBy")]
    directed_by: Option<String>,
    duration: u32,
    #[serde(default, rename = "allGenres")]
    all_genres: Option<String>,
    #[serde(default)]
    rating: Option<String>,
    #[serde(default, rename = "ratingReason")]
    rating_reason: Option<String>,
}

impl FestivalEvent {
    fn csv(input: &str) -> Vec<String> {
        input.split(",").map(|e| e.trim().to_string()).collect()
    }
    fn markup(source: &str) -> Result<String, FilmError> {
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
    pub fn fetch_from_gft(cfg: &Config, movie_id: u32) -> Result<Vec<Self>, FilmError> {
        let cache_file = format!("{}/screenings/{}.json", &cfg.state_directory, movie_id);
        let event = match (cfg.is_live(), fs::exists(&cache_file)) {
            (false, Ok(true)) => {
                let bytes = fs::read(&cache_file)
                    .map_err(|_| FilmError::ReadError(cache_file.to_string()))?;

                if cfg.is_debug() {
                    println!("Returning cached - {}", movie_id);
                }
                let evt: Vec<FestivalEvent> =
                    serde_json::from_slice::<Vec<FestivalEvent>>(&bytes[..])
                        .map_err(|_e| FilmError::ReadError(cache_file.clone()))?;
                evt
            }
            _ => {
                if cfg.is_debug() {
                    println!("Reading from web - {}", movie_id);
                }
                let screenings = fetch_screenings(cfg, movie_id)?;
                let mut result = vec![];
                for screening in screenings {
                    let date = NaiveDate::parse_from_str(&screening.time[0..10], "%Y-%m-%d")
                        .map_err(|_| FilmError::BadDate(screening.time.clone()))?;
                    let start = NaiveTime::parse_from_str(&screening.time[11..16], "%H:%M")
                        .map_err(|_| FilmError::BadTime(screening.time.clone()))?;
                    let badge_ids: Vec<u32> = screening
                        .showing_badge_ids
                        .iter()
                        .map(|e| e.parse().unwrap_or(0))
                        .collect();
                    let starring = Self::csv(&screening.movie.starring.unwrap_or("".to_string()));
                    let genres = Self::csv(&screening.movie.all_genres.unwrap_or("".to_string()));
                    let rating_reasons =
                        Self::csv(&screening.movie.rating_reason.unwrap_or("".to_string()));
                    let screen_id = screening.screen_id.clone().parse().unwrap_or(0);
                    let (strand_name, strand) = cfg.strand_from_badges(badge_ids);
                    let (screen_name, screen) = cfg.screen_from_id(screen_id);
                    result.push(Self {
                        date,
                        start,
                        end: start + chrono::TimeDelta::minutes(screening.movie.duration.into()),
                        screening_id: screening.id.parse().unwrap_or(0),
                        movie_id,
                        title: screening.movie.name.clone(),
                        strand: strand_name,
                        strand_id: strand.id,
                        strand_colour: strand.colour,
                        strand_priority: strand.priority,
                        screen: screen_name,
                        screen_id: screen.id,
                        screen_colour: screen.colour,
                        attendees: vec![],
                        synopsis: Self::markup(&screening.movie.synopsis)?,
                        starring,
                        genres,
                        director: screening.movie.directed_by.unwrap_or("".to_string()),
                        rating: screening.movie.rating.unwrap_or("".to_string()),
                        rating_reasons,
                        poster: screening.movie.poster_image.unwrap_or("".to_string()),
                    });
                }
                fs::write(
                    &cache_file,
                    serde_json::to_string_pretty(&result)
                        .map_err(|_| FilmError::WriteError(cache_file.clone()))?,
                )
                .map_err(|_| FilmError::WriteError(cache_file.clone()))?;
                sleep(Duration::from_millis(250));
                result
            }
        };
        if !event.is_empty() {
            fetch_image(cfg, &event[0].poster)?
        }
        Ok(event)
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

pub fn fetch_screenings(cfg: &Config, id: u32) -> Result<Vec<Screening>, FilmError> {
    let graphql = r#"{"query": "query { showingsForDate(movieId: &) { data { movie { id name posterImage synopsis starring directedBy duration allGenres rating ratingReason } id time screenId showingBadgeIds }}}"}"#.replace("&",&format!("{}",id));
    let from_gft = fetch_from_gft(&graphql)?;
    if cfg.is_debug() {
        println!("{}", &from_gft);
    }
    deserialize_screenings(id, &from_gft)
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
        let mut screening: Screening = serde_json::from_value(each.clone()).map_err(|e| {
            FilmError::BadValueType(format!("not a valid screening - {:?} - {:?}", e, json))
        })?;
        screening.movie_id = Some(id);
        result.push(screening);
    }
    Ok(result)
}

pub fn fetch_ids() -> Result<String, FilmError> {
    let graphql = r#"{"variables":{"titleClassIds":[196,211,229],"type":"now-playing-and-coming-soon"},"query":"query ($titleClassIds: [ID], $type: String) { movies( limit: 500 type: $type titleClassIds: $titleClassIds ) { data { id name } } } "}"#;
    fetch_from_gft(graphql)
}

pub fn id_map(cfg: &Config) -> Result<FilmMap, FilmError> {
    let cache_file = format!("{}/ids.json", &cfg.state_directory);
    if let Ok(true) = fs::exists(&cache_file) {
        let bytes =
            fs::read(&cache_file).map_err(|_| FilmError::ReadError(cache_file.to_string()))?;

        //println!("Returning cached id map");
        return serde_json::from_slice(&bytes[..])
            .map_err(|_e| FilmError::ReadError(cache_file.clone()));
    };

    let map = load_ids(&fetch_ids()?)?;
    fs::write(
        &cache_file,
        serde_json::to_string_pretty(&map)
            .map_err(|_| FilmError::WriteError(cache_file.clone()))?,
    )
    .map_err(|_| FilmError::WriteError(cache_file.clone()))?;
    sleep(Duration::from_millis(250));
    Ok(map)
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

fn deserialize_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&buf, "%Y-%m-%d").map_err(serde::de::Error::custom)
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    NaiveTime::parse_from_str(&buf, "%H:%M").map_err(serde::de::Error::custom)
}

pub fn fetch_image(cfg: &Config, key: &str) -> Result<(), FilmError> {
    let cache_file = format!("{}/posters/{}.jpg", &cfg.state_directory, key);
    if let Ok(true) = fs::exists(&cache_file) {
        return Ok(());
    };
    //println!("fetching a poster for {}", &key);
    let client = reqwest::blocking::Client::new();
    let rsp = client.get(format!("https://indy-systems.imgix.net/{}?fit=crop&w=400&h=600&fm=jpeg&auto=format,compress&cs=origin",key))
        .send().map_err(|e| {FilmError::WebError(format!("{}",e))})?
        .bytes().map_err(|e| {FilmError::WebError(format!("{}",e))})?;
    fs::write(&cache_file, rsp).map_err(|_| FilmError::WriteError(cache_file.clone()))?;
    sleep(Duration::from_millis(250));
    Ok(())
}

pub fn fetch_from_gft(graphql: &str) -> Result<String, FilmError> {
    let client = reqwest::blocking::Client::new();
    let rsp = client.post("https://www.glasgowfilm.org/graphql")
    .body(graphql.to_string())
    .header(reqwest::header::USER_AGENT,"User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:122.0) Gecko/20100101 Firefox/122.0")
    .header(reqwest::header::ACCEPT,"*/*")
    .header(reqwest::header::CONTENT_TYPE,"application/json")
    .header("site-id","103")
    .header("client-type","consumer")
        .send().map_err(|e| {FilmError::WebError(format!("{}",e))})?
        .text().map_err(|e| {FilmError::WebError(format!("{}",e))});
    //println!("{:?}", &rsp);
    rsp
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
    fn test_markup() {
        assert_eq!(
            "with _italics_ test".to_string(),
            FestivalEvent::markup("with <i>italics</i> test").unwrap()
        );
        assert_eq!(
            "with #strong[bold] test \\* and \\$".to_string(),
            FestivalEvent::markup("with <b>bold</B> test * and $").unwrap()
        );
    }

    #[test]
    fn test_fetch_from_gft() {
        let mut cfg = Config::default();
        cfg.state_directory = "/tmp".to_string();
        cfg.screens.insert(
            "GFT 1".to_string(),
            crate::config::ScreenConfig { id: 175, colour: 1 },
        );
        cfg.strands.insert(
            "Official Selection".to_string(),
            crate::config::StrandConfig {
                id: 852,
                colour: "bc0e77".to_string(),
                priority: 2,
            },
        );
        let _event = FestivalEvent::fetch_from_gft(&cfg, 33606).unwrap();
    }

    fn test_fetch_ids() {
        if let Ok(ids) = fetch_ids() {
            println!("{:?}", load_ids(&ids));
        }
        assert!(false);
    }
}
