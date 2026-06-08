use std::path::Path;
use std::fs;
use std::thread;
use std::sync::{Arc, Mutex, mpsc::{self, Receiver}};
use std::collections::HashMap;
use crate::global::utils;

#[derive(Clone, Copy, Debug, Default)]
pub struct ImgVec2 { pub x: f32, pub y: f32 }

#[derive(Clone, Copy, Debug, Default)]
pub struct ImgRect { pub min: ImgVec2, pub max: ImgVec2 }

#[derive(Clone, Debug)]
pub struct SpriteCut {
    pub uv_coordinates: ImgRect,
    pub original_size: ImgVec2,
    #[allow(dead_code)] pub name: String,
}

impl Clone for SpriteSheet {
    fn clone(&self) -> Self {
        Self {
            image_data: self.image_data.clone(),
            cuts_map: self.cuts_map.clone(),
            is_loading_active: self.is_loading_active,
            data_receiver: None,
            sheet_name: self.sheet_name.clone(),
        }
    }
}

#[derive(Default)]
pub struct SpriteSheet {
    pub image_data: Option<Arc<image::RgbaImage>>,
    pub cuts_map: HashMap<usize, SpriteCut>,
    pub is_loading_active: bool,
    pub data_receiver: Option<Mutex<Receiver<(String, image::RgbaImage, HashMap<usize, SpriteCut>)>>>,
    pub sheet_name: String,
}


impl SpriteSheet {
    #[allow(dead_code)]
    pub fn is_ready(&self) -> bool {
        self.image_data.is_some()
    }

    pub fn load(&mut self, png_path: &Path, imgcut_path: &Path, id_str: String) {
        if self.is_loading_active { return; }

        self.is_loading_active = true;
        let png_path_buf = png_path.to_path_buf();
        let cut_path_buf = imgcut_path.to_path_buf();

        let (sender, receiver) = mpsc::channel();
        self.data_receiver = Some(Mutex::new(receiver));

        thread::spawn(move || {
            if let Some((image, cuts)) = Self::load_internal(&png_path_buf, &cut_path_buf) {
                let _ = sender.send((id_str, image, cuts));
            }
        });
    }

    pub fn update(&mut self) {
        if let Some(mutex) = &self.data_receiver
            && let Ok(receiver) = mutex.try_lock()
                && let Ok((name, image, cuts)) = receiver.try_recv() {
                    self.sheet_name = name.clone();
                    self.image_data = Some(Arc::new(image));
                    self.cuts_map = cuts;
                    self.is_loading_active = false;
                }

        if !self.is_loading_active && self.data_receiver.is_some() {
            self.data_receiver = None;
        }
    }

    fn load_internal(png_path: &Path, cut_path: &Path) -> Option<(image::RgbaImage, HashMap<usize, SpriteCut>)> {
        let image_data = fs::read(png_path).ok()?;
        let image = image::load_from_memory(&image_data).ok()?.to_rgba8();

        let w = image.width() as f32;
        let h = image.height() as f32;

        let content = fs::read_to_string(cut_path).ok()?;
        let delimiter = utils::detect_csv_separator(&content);
        let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();

        let mut sprite_count = 0;
        let mut data_start_index = 0;
        let mut found_header = false;

        for (index, line) in lines.iter().enumerate() {
            if !line.contains(',') {
                if let Ok(count_val) = line.trim().parse::<usize>()
                    && count_val > 0 && count_val < 5000 {
                        sprite_count = count_val;
                        data_start_index = index + 1;
                        found_header = true;
                    }
            } else if found_header {
                break;
            }
        }

        if !found_header || sprite_count == 0 {
            data_start_index = 0;
            sprite_count = lines.len();
        }

        let mut parsed_cuts = HashMap::new();

        for i in 0..sprite_count {
            let line_index = data_start_index + i;
            if line_index >= lines.len() { break; }

            let line = lines[line_index];
            let parts: Vec<&str> = line.split(delimiter).collect();

            if parts.len() >= 4
                && let (Ok(x), Ok(y), Ok(cut_width), Ok(cut_height)) = (
                    parts[0].trim().parse::<f32>(),
                    parts[1].trim().parse::<f32>(),
                    parts[2].trim().parse::<f32>(),
                    parts[3].trim().parse::<f32>(),
                ) {
                    let uv_min = ImgVec2 { x: x / w, y: y / h };
                    let uv_max = ImgVec2 { x: (x + cut_width) / w, y: (y + cut_height) / h };

                    let cut_name = if parts.len() > 4 {
                        parts[4].trim().to_string()
                    } else {
                        String::new()
                    };

                    parsed_cuts.insert(i, SpriteCut {
                        uv_coordinates: ImgRect { min: uv_min, max: uv_max },
                        original_size: ImgVec2 { x: cut_width, y: cut_height },
                        name: cut_name,
                    });
                }
        }

        Some((image, parsed_cuts))
    }
}