use serde_json::Value;
use typst_bake::{IntoDict, IntoValue};

#[derive(IntoValue, IntoDict)]
struct Summary {
    pub version: u32,
    pub dates: Vec<String>,
    pub screens: Vec<Screen>,
}

#[derive(IntoValue)]
struct Screen {
    pub names: Vec<String>,
    pub showings: Vec<Vec<Showing>>,
}

#[derive(IntoValue)]
struct Showing {
    pub start: String,
    pub title: String,
    pub strand: String,
    pub duration: u32,
    pub color: String,
    pub id: Option<String>,
    pub day: String,
    pub attendees: Vec<String>,
}

impl Into<Showing> for Value {
    fn into(self) -> Showing {
        let map = self.as_object().unwrap();
        let start = map.get("start").unwrap().as_str().unwrap().to_owned();
        let title = map.get("title").unwrap().as_str().unwrap().to_owned();
        let strand = map.get("strand").unwrap().as_str().unwrap().to_owned();
        let color = map.get("color").unwrap().as_str().unwrap().to_owned();
        let day = map.get("day").unwrap().as_str().unwrap().to_owned();
        let duration = map
            .get("duration")
            .unwrap()
            .as_number()
            .unwrap()
            .as_u64()
            .unwrap() as u32;
        let attendees = map
            .get("attendees")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|e: &Value| e.as_str().unwrap().to_owned())
            .collect::<Vec<String>>();
        Showing {
            id: None,
            start,
            title,
            strand,
            color,
            day,
            duration,
            attendees,
        }
    }
}
impl From<Value> for Screen {
    fn from(val: Value) -> Self {
        let map = val.as_object().unwrap();
        let names = map.keys().cloned().collect();
        let showings = map
            .values()
            .map(|e: &Value| {
                e.as_array()
                    .unwrap()
                    .iter()
                    .map(|ea| Into::<Showing>::into(ea.clone()))
                    .collect()
            })
            .collect::<Vec<Vec<Showing>>>();
        Screen { names, showings }
    }
}
impl From<Value> for Summary {
    fn from(val: Value) -> Self {
        let map = val.as_object().unwrap();
        let dates = map.keys().cloned().collect();
        let screens = map
            .values()
            .map(|e: &Value| Into::<Screen>::into(e.clone()))
            .collect::<Vec<Screen>>();
        Summary {
            version: 0,
            dates,
            screens,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = std::fs::read("./summary.json").unwrap();
    let data: Value = serde_json::from_slice(&json[..]).unwrap();
    let mut summary: Summary = data.into();
    summary.version = 1;
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
