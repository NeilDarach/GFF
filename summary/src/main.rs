use typst_bake::{IntoDict, IntoValue};

#[derive(IntoValue, IntoDict)]
struct Summary {
    pub version: u32,
    pub json: String,
    pub colours: Vec<Pair>,
    pub names: Vec<Pair>,
}

#[derive(IntoValue)]
struct Pair {
    key: String,
    value: String,
}

impl From<(&str, &str)> for Pair {
    fn from(value: (&str, &str)) -> Self {
        Self {
            key: value.0.to_owned(),
            value: value.1.to_owned(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = String::from_utf8(std::fs::read("./summary.json").unwrap()).unwrap();
    let colours = vec![
        ("GFT 1", "344798").into(),
        ("GFT 2", "6596d0").into(),
        ("GFT 3", "d9cdcc").into(),
        ("Odeon 10", "946c0c").into(),
        ("Odeon 11", "a30053").into(),
        ("Odeon 12", "f2c170").into(),
    ];
    let names = vec![
        ("M", "Marion").into(),
        ("N", "Neil").into(),
        ("Pt", "Patrick").into(),
        ("Pm", "Pam").into(),
        ("V", "Vanessa").into(),
    ];
    let summary = Summary {
        version: 1,
        json,
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
