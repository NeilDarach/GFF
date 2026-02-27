use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use typst::foundations as t;
use typst_bake::__internal::typst::foundations as tb;
use typst_bake::{IntoDict, IntoValue};
use typst_library::foundations as tl;

#[derive(Clone, Default, Serialize, Deserialize)]
struct TypstHash<T: tb::IntoValue + Clone>(HashMap<String, T>);

impl<T: tb::IntoValue + Clone> DerefMut for TypstHash<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T: tb::IntoValue + Clone> Deref for TypstHash<T> {
    type Target = HashMap<String, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: tb::IntoValue + Clone> From<HashMap<String, T>> for TypstHash<T> {
    fn from(value: HashMap<String, T>) -> Self {
        TypstHash(value)
    }
}

impl<T: tb::IntoValue + Clone> t::IntoValue for TypstHash<T> {
    fn into_value(self) -> tb::Value {
        let d = {
            #[allow(unused_mut)]
            let mut map = tl::IndexMap::default();
            for (k, v) in self.iter() {
                map.insert(
                    k.clone().into(),
                    tl::IntoValue::into_value(tb::IntoValue::into_value(v.clone())),
                );
            }
            tl::Dict::from(map)
        };
        tb::Value::Dict(d)
    }
}

impl<T: tb::IntoValue + Clone> TypstHash<T> {
    #[inline]
    #[must_use]
    pub fn into_dict(self) -> tb::Dict {
        {
            #[allow(unused_mut)]
            let mut map = tl::IndexMap::default();
            for (k, v) in self.iter() {
                map.insert(
                    k.clone().into(),
                    tl::IntoValue::into_value(tb::IntoValue::into_value(v.clone())),
                );
            }

            tl::Dict::from(map)
        }
    }
}

impl<T: t::IntoValue + Clone> From<TypstHash<T>> for t::Dict {
    fn from(value: TypstHash<T>) -> Self {
        value.into_dict()
    }
}

#[derive(IntoValue, IntoDict)]
struct Summary {
    pub version: u32,
    pub summary: TypstHash<TypstHash<Vec<Showing>>>,
    pub colours: TypstHash<String>,
    pub names: TypstHash<String>,
}

#[derive(Clone, IntoValue, Serialize, Deserialize)]
struct Showing {
    start: String,
    title: String,
    strand: String,
    duration: u64,
    color: String,
    id: Option<String>,
    day: String,
    attendees: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let colours = vec![
        ("GFT 1", "344798"),
        ("GFT 2", "6596d0"),
        ("GFT 3", "d9cdcc"),
        ("Odeon 10", "946c0c"),
        ("Odeon 11", "a30053"),
        ("Odeon 12", "f2c170"),
    ];
    let colours = colours
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<HashMap<String, String>>()
        .into();
    let names = vec![
        ("M", "Marion"),
        ("N", "Neil"),
        ("Pt", "Patrick"),
        ("Pm", "Pam"),
        ("V", "Vanessa"),
    ];
    let names = names
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<HashMap<String, String>>()
        .into();
    let json = std::fs::read("summary.json").unwrap();
    let summary_data: TypstHash<TypstHash<Vec<Showing>>> =
        serde_json::from_slice(&json[..]).unwrap();
    let summary = Summary {
        version: 1,
        summary: summary_data,
        colours,
        names,
    };
    let pdf = typst_bake::document!("summary.typ")
        .with_inputs(summary)
        .to_pdf()?;
    save_pdf(&pdf, "output.pdf")
}

fn save_pdf(data: &[u8], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    std::fs::write(out_dir.join(filename), data)?;
    println!("Generated {} ({} bytes)", filename, data.len());
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    fn save_svg(data: &Vec<String>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        std::fs::write(out_dir.join(filename), data.first().unwrap())?;
        println!("Generated {} ({} bytes)", filename, data.len());
        Ok(())
    }

    #[derive(IntoDict, IntoValue, Serialize, Deserialize)]
    struct Sample {
        hash: TypstHash<u32>,
    }
    #[test]
    fn test_serialize_myhash() {
        let mut hash = TypstHash::default();
        hash.insert("Key".to_string(), 4);
        let json = serde_json::to_string_pretty(&hash).unwrap();
        assert_eq!("{\n  \"Key\": 4\n}", json);
    }

    #[test]
    fn test_deserialize_myhash() {
        let hash: TypstHash<u32> = serde_json::from_str("{\n  \"Key\": 4\n}").unwrap();
        assert_eq!(4, *hash.get("Key").unwrap());
    }
    #[test]
    fn test_dump_sample() {
        let hash: TypstHash<u32> = serde_json::from_str("{\n  \"Key\": 4\n}").unwrap();
        let sample = Sample { hash };
        let svg = typst_bake::document!("dump.typ")
            .with_inputs(sample)
            .to_svg()
            .unwrap();
        save_svg(&svg, "output.svg").unwrap();
        assert!(false);
    }
    #[test]
    fn test_read_summary() {
        let json = std::fs::read("summary.json").unwrap();
        let data: TypstHash<TypstHash<Vec<Showing>>> = serde_json::from_slice(&json[..]).unwrap();
        let pdf = typst_bake::document!("dump.typ")
            .with_inputs(data)
            .to_pdf()
            .unwrap();
        save_pdf(&pdf, "output.pdf").unwrap();
    }
}
