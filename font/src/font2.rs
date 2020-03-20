extern crate ttf_parser as ttf;

use ttf::{Font as F, GlyphId, OutlineBuilder};
use std::collections::HashMap;
use crate::core::open_type_like::glyph::Glyph;
use crate::core::open_type_like::path::PathData as P;
use crate::core::typesetting::MergedFont;
use std::ops::Deref;

use std::sync::Mutex;
use std::rc::Rc;
use std::ptr::NonNull;
use std::pin::Pin;


struct PathData(P);

impl PathData {
    pub fn new() -> Self { PathData(P::new()) }
}

impl std::ops::Deref for PathData {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PathData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl OutlineBuilder for PathData {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y)
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y)
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x, y, x1, y1)
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.curve_to(x, y, x1, y1, x2, y2)
    }

    fn close(&mut self) {
        self.0.close()
    }
}

pub struct Font<'a>(F<'a>, Option<NonNull<Vec<u8>>>);

impl<'a> Font<'a> {
    pub fn new<Data: std::ops::Deref<Target=[u8]>>(data: &'a Data) -> Option<Self> {
        let font = F::from_data(&data, 0);
        if font.is_some() {
            Some(Font(font.unwrap(), None))
        } else {
            None
        }
    }

    pub fn get_glyph(&self, c: char) -> Glyph {
        let id = self.glyph_index(c).unwrap_or(GlyphId(0));
        let mut path = PathData::new();
        self.outline_glyph(id, &mut path);
        let advance_width = self.glyph_hor_advance(id).unwrap_or(0);
        let units_per_em = self.units_per_em().unwrap_or(1000);
        let ascender = self.ascender();
        let descender = self.descender();
        Glyph {
            path: path.0,
            advance_width: advance_width as i32,
            units_per_em: units_per_em as i32,
            ascender: ascender as i32,
            descender: descender as i32,
        }
    }
}

impl<'a> std::ops::Deref for Font<'a> {
    type Target = F<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct FontCache<'a> {
    fonts: HashMap<String, Font<'a>>,
    glyph_indexes: HashMap<(String, char), usize>,
    glyph_caches: Vec<Box<Glyph>>,
}

impl<'a> FontCache<'a> {
    pub fn new() -> Self {
        FontCache {
            fonts: HashMap::new(),
            glyph_caches: vec![],
            glyph_indexes: HashMap::new(),
        }
    }

    pub fn load_font_bytes(&mut self, font_name: String, data: &'a Vec<u8>) {
        // let mut hm = HASHMAP.lock().unwrap();
        // hm.insert(font_name.clone(), data);
        // let data: Vec<u8> = hm.get(&font_name).unwrap().to_vec();
        // let data: &'static Vec<u8> = &data;
        let font = Font::new(data).unwrap();
        self.fonts.insert(font_name, font);
    }

    pub fn get_glyph(&self, font_name: String, c: char) -> Box<Glyph> {
        let mut result = self.fonts.get(&font_name);
        let mut is_default = false;
        if result.is_none() {
            result = self.fonts.get("default");
            is_default = true;
        }
        let mut font = result.unwrap();
        let find_index = font.glyph_index(c);
        if find_index.is_none() && !is_default {
            font = self.fonts.get("default").unwrap();
        }
        let boxed = Box::new(font.get_glyph(c));
        boxed
    }

    pub fn check_glyph(&mut self, font_name: String, c: char) {
        let result = self.glyph_indexes.get(&(font_name.clone(), c.clone()));
        if result.is_none() {
            let glyph = self.get_glyph(font_name.clone(), c.clone());
            self.glyph_caches.push(glyph);
            self.glyph_indexes.insert((font_name, c), self.glyph_caches.len() - 1);
        }
    }
    pub fn get_cache_count(&self) -> usize {
        self.glyph_caches.len()
    }
}

impl<'b> MergedFont for FontCache<'b> {
    fn char_to_glyph<'a>(&'a self, font_name: String, c: char) -> &'a Box<Glyph> {
        let result = self.glyph_indexes.get(&(font_name.clone(), c));
        &self.glyph_caches.get(*result.unwrap()).unwrap()
    }
}

// pub struct Ttt {
//     data: HashMap<String, Vec<u8>>,
//     fonts: FontCache<'static>,
// }
//
// impl Ttt {
//     pub fn new() -> Self {
//         Ttt {
//             data: HashMap::new(),
//             fonts: FontCache::new(),
//         }
//     }
//     pub fn load(&mut self, name: String) {
//         let data = self.data.get(&name).unwrap();
//         self.fonts.load_font_bytes(name, &data.as_slice());
//     }
// }

// impl std::ops::Deref for Vec<u8> {
//     type Target = [u8];
//
//     fn deref(&self) -> &Self::Target {
//         self.as_slice()
//     }
// }

pub mod test {
    use std::borrow::Cow;
    use crate::font2::{Font, FontCache};
    use core::open_type_like::glyph::Glyph;
    use core::typesetting::MergedFont;

    use core::open_type_like::command::tran_commands_stream;
    use core::typesetting::compute_render_command;
    use core::data::text_data::TextData;

    use std::collections::HashMap;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::prelude::*;


    #[test]
    fn test() {
        let file = include_bytes!("./c_739.ttf") as &[u8];
        let file = Cow::Borrowed(file).to_vec();
        // let mut hm = HASHMAP.lock().unwrap();
        // hm.insert("default".to_string(), file.to_vec());
        // let file1: Vec<u8> = hm.get("default").unwrap().to_vec();
        let mut font_cache = FontCache::new();
        font_cache.load_font_bytes("default".to_string(), &file);
        let mut read = File::open("/Users/suti/start/text-render-rust/skia/src/t.json").unwrap();
        let mut test_text_json = String::from("");
        read.read_to_string(&mut test_text_json);
        let test_text_data = TextData::parse(&test_text_json).unwrap();

        for content in test_text_data.paragraph.paragraph_content.iter() {
            let blocks = &content.blocks;
            for block in blocks.iter() {
                let text = block.text.clone();
                let font_family = &block.font_family;
                let mut text_chars = text.chars();
                while let Some(text) = text_chars.next() {
                    font_cache.check_glyph(font_family.to_string(), text);
                }
            }
        }
        let text_data = TextData::parse(&test_text_json).unwrap();
        let (b_box, result, _) = compute_render_command(&text_data, &font_cache).unwrap();
        let b_box = b_box.get_total_box();
        let mut width = b_box.get_width().ceil() as f32;
        let height = b_box.get_height().ceil() as f32;
        if test_text_data.width > width {
            width = test_text_data.width
        }
        let result1 = tran_commands_stream(&result);
        println!("ok");
    }
}