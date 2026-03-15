use std::fmt::Display;

use egui::load::SizedTexture;
use image::{ImageReader, RgbImage};
use serde::{Deserialize, Serialize};

pub struct ImagePair(pub String, pub [Image; 2]);
pub struct Image {
    pub id: ImageID,
    pub original_data: RgbImage,
    pub texture: Option<SizedTexture>,
    pub normalized_data: Option<RgbImage>,
    pub normalized_texture: Option<SizedTexture>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone)]
pub struct ImageID {
    directory: String,
    is_first: bool,
}

impl Display for ImageID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}-{}",
            self.directory,
            i32::from(!self.is_first)
        ))
    }
}

pub fn load_images(directory: String) -> (Vec<ImagePair>, Vec<String>) {
    let mut errors = vec![];
    let mut image_pairs = vec![];
    for e in std::fs::read_dir(directory).expect("could not read directory") {
        let entry = match e {
            Ok(entry) => entry,
            Err(e) => {
                errors.push(format!("could not read from directory entry {e}"));
                continue;
            }
        };
        match visit_dir(&entry) {
            Ok(value) => image_pairs.push(value),
            Err(e) => errors.push(format!(
                "error reading from {} because {e}",
                entry.path().display()
            )),
        }
    }
    (image_pairs, errors)
}

fn visit_dir(entry: &std::fs::DirEntry) -> Result<ImagePair, String> {
    let dir_name = entry
        .file_name()
        .into_string()
        .map_err(|e| format!("could not convert {} to string", e.display()))?;
    match entry.file_type() {
        Ok(t) if !t.is_dir() => return Err("expected a directory".to_owned()),
        Err(e) => {
            return Err(format!(
                "could not read file type of {} {e}",
                entry.path().display()
            ));
        }
        _ => {}
    }
    let mut first = None;
    let mut second = None;
    for sub_entry in std::fs::read_dir(entry.path()).map_err(|e| format!("could not read {e}"))? {
        let sub_entry = sub_entry.map_err(|e| format!("could not read {e}"))?;
        let p = sub_entry.path();
        if !sub_entry
            .file_type()
            .map_err(|e| format!("reading file_type for {} {e}", p.display()))?
            .is_file()
        {
            return Err(format!("expected an image file for {}", p.display()));
        }
        if p.to_string_lossy().contains("image0") {
            first = Some(
                ImageReader::open(sub_entry.path())
                    .map_err(|e| format!("opening {} {e}", sub_entry.path().display()))?
                    .decode()
                    .map_err(|e| format!("decoding {} {e}", sub_entry.path().display()))?,
            );
        }
        if sub_entry.path().to_string_lossy().contains("image1") {
            second = Some(
                ImageReader::open(sub_entry.path())
                    .map_err(|e| format!("opening {} {e}", sub_entry.path().display()))?
                    .decode()
                    .map_err(|e| format!("decoding {} {e}", sub_entry.path().display()))?,
            );
        }
    }
    let first =
        first.ok_or_else(|| format!("missing first image for {}", entry.path().display()))?;
    let second =
        second.ok_or_else(|| format!("missing second image for {}", entry.path().display()))?;
    Ok(ImagePair(
        dir_name.clone(),
        [
            Image {
                original_data: first.into(),
                id: ImageID {
                    directory: dir_name.clone(),
                    is_first: true,
                },
                texture: None,
                normalized_data: None,
                normalized_texture: None,
            },
            Image {
                original_data: second.into(),
                id: ImageID {
                    directory: dir_name,
                    is_first: false,
                },
                texture: None,
                normalized_data: None,
                normalized_texture: None,
            },
        ],
    ))
}
