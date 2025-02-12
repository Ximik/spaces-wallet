use std::{env, path::PathBuf};
use std::{fs::File, io::Write};

fn compile_icons(out_dir: &PathBuf) {
    println!("cargo:rerun-if-changed=assets/icons/*");

    let dir_source = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons");
    let font_file_dest = out_dir.join("icons.ttf");
    let rust_file_dest = out_dir.join("icons.rs");

    verglas::make_font(dir_source, &font_file_dest).expect("building icon font failed");
    let icons: std::collections::HashMap<String, char> = verglas::build_icon_map(&font_file_dest)
        .expect("building icon map failed")
        .iter()
        .filter_map(|(name, ch)| {
            let name = name
                .rsplit_once('/')
                .map_or(name.as_str(), |(_, n)| n)
                .split(|c: char| !c.is_alphanumeric())
                .map(|s| {
                    let mut word = s.to_lowercase();
                    if let Some(first) = word.get_mut(0..1) {
                        first.make_ascii_uppercase();
                    }
                    word
                })
                .collect();
            if name == "Notdef" {
                None
            } else {
                Some((name, *ch))
            }
        })
        .collect();

    let enum_def = format!(
        "#[derive(Debug, Clone, Copy)]\npub enum Icon {{\n{}\n}}\n\n",
        icons
            .iter()
            .map(|(name, _)| format!("    {}", name))
            .collect::<Vec<_>>()
            .join(",\n")
    );

    let impl_def = format!(
        "impl Icon {{\n    pub fn as_char(&self) -> char {{\n        match self {{\n{}\n        }}\n    }}\n}}\n",
        icons
            .iter()
            .map(|(name, ch)| format!("            Icon::{} => '{}'", name, ch))
            .collect::<Vec<_>>()
            .join(",\n")
    );

    let mut file = File::create(&rust_file_dest).expect("creation rust file failed");
    file.write_all((enum_def + &impl_def).as_bytes())
        .expect("writing rust file failed");
}

fn compile_logo(out_dir: &PathBuf) {
    println!("cargo:rerun-if-changed=assets/logo.png");

    let logo_source = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.png");
    let logo_dest = out_dir.join("logo.rgba");

    use image::GenericImageView;
    let img = image::open(logo_source).expect("opening logo failed");
    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();

    let mut file = File::create(&logo_dest).expect("creating logo binary failed");
    file.write_all(&width.to_le_bytes())
        .expect("writing logo binary failed");
    file.write_all(&height.to_le_bytes())
        .expect("writing logo binary failed");
    file.write_all(rgba.as_raw())
        .expect("writing logo binary failed");
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    compile_icons(&out_dir);
    compile_logo(&out_dir);
}
