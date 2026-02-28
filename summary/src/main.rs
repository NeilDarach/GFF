mod typst_map;
use serde::{Deserialize, Serialize};
use typst_bake::{IntoDict, IntoValue};
use typst_map::TypstMap;

#[derive(IntoValue, IntoDict, Serialize, Deserialize)]
struct Brochure {
    pub entries: Vec<BrochureEntry>,
    pub banner: Vec<u8>,
    pub version: String,
}

#[derive(IntoValue, Serialize, Deserialize)]
struct BrochureEntry {
    pub name: String,
    pub id: String,
    pub sortname: String,
    pub showings: Vec<BrochureShowing>,
    pub duration: u64,
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
    #[serde(skip)]
    pub poster_bytes: Option<Vec<u8>>,
}

#[derive(IntoValue, Serialize, Deserialize)]
struct BrochureShowing {
    pub screen: String,
    pub time: String,
    pub date: String,
    pub datetime: String,
}

#[derive(IntoValue, IntoDict)]
struct Summary {
    pub version: String,
    pub summary: TypstMap<TypstMap<Vec<Showing>>>,
    pub colours: TypstMap<String>,
    pub names: TypstMap<String>,
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
    /*
        let colours: TypstMap<String> = vec![
            ("GFT 1", "344798"),
            ("GFT 2", "6596d0"),
            ("GFT 3", "d9cdcc"),
            ("Odeon 10", "946c0c"),
            ("Odeon 11", "a30053"),
            ("Odeon 12", "f2c170"),
        ]
        .into();
        let names: TypstMap<String> = vec![
            ("M", "Marion"),
            ("N", "Neil"),
            ("Pt", "Patrick"),
            ("Pm", "Pam"),
            ("V", "Vanessa"),
        ]
        .into();
        let json = std::fs::read("summary.json").unwrap();
        let summary_data: TypstMap<TypstMap<Vec<Showing>>> =
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
    */
    let json = std::fs::read("brochure.json").unwrap();
    let mut entries: Vec<BrochureEntry> = serde_json::from_slice(&json[..]).unwrap();
    let banner = std::fs::read("banner.jpg").unwrap();
    load_pictures(&mut entries);
    let brochure = Brochure {
        entries,
        banner,
        version: "1".to_owned(),
    };
    let pdf = typst_bake::document!("brochure.typ")
        .with_inputs(brochure)
        .to_pdf()?;
    save_pdf(&pdf, "brochure.pdf")
}

fn load_pictures(entries: &mut Vec<BrochureEntry>) {
    for entry in entries {
        let data = std::fs::read(&entry.poster).unwrap();
        entry.poster_bytes = Some(data);
    }
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
    fn save_svg(data: &[String], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        std::fs::write(out_dir.join(filename), data.first().unwrap())?;
        println!("Generated {} ({} bytes)", filename, data.len());
        Ok(())
    }

    #[derive(IntoDict, IntoValue, Serialize, Deserialize)]
    struct Sample {
        hash: TypstMap<u32>,
    }
    #[test]
    fn test_serialize_myhash() {
        let mut hash = TypstMap::default();
        hash.insert("Key".to_string(), 4);
        let json = serde_json::to_string_pretty(&hash).unwrap();
        assert_eq!("{\n  \"Key\": 4\n}", json);
    }

    #[test]
    fn test_deserialize_myhash() {
        let hash: TypstMap<u32> = serde_json::from_str("{\n  \"Key\": 4\n}").unwrap();
        assert_eq!(4, *hash.get("Key").unwrap());
    }
    #[test]
    fn test_dump_sample() {
        let hash: TypstMap<u32> = serde_json::from_str("{\n  \"Key\": 4\n}").unwrap();
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
        let data: TypstMap<TypstMap<Vec<Showing>>> = serde_json::from_slice(&json[..]).unwrap();
        let pdf = typst_bake::document!("dump.typ")
            .with_inputs(data)
            .to_pdf()
            .unwrap();
        save_pdf(&pdf, "output.pdf").unwrap();
    }
}
