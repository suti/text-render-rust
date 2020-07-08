use crate::core::open_type_like::glyph::Glyph;
use crate::core::open_type_like::path::PathData;
use crate::core::typesetting::MergedFont;
use std::collections::HashMap;
use stb_truetype as stt;

pub struct Font<Data: std::ops::Deref<Target=[u8]>>(stt::FontInfo<Data>);

impl<Data: std::ops::Deref<Target=[u8]>> Font<Data> {
    pub fn new(d: Data) -> Option<Font<Data>> {
        let font_info = stt::FontInfo::new(d, 0);
        if let Some(font_info) = font_info {
            font_info.get_font_name_strings();
            Some(Font(font_info))
        } else {
            None
        }
    }

    pub fn get_glyph(&self, char_code: u32) -> Glyph {
        let index = self.find_glyph_index(char_code);
        if index == 0 {
            return Glyph::get_none();
        }
        let stt::VMetrics { ascent: ascender, descent: descender, line_gap: _ } = self.get_v_metrics();
        let path_vertex = self.get_glyph_shape(index).unwrap_or(vec![]);
        let stt::HMetrics { advance_width, left_side_bearing } = self.get_glyph_h_metrics(index);
        let units_per_em = self.units_per_em() as i32;
        Glyph {
            path: vertex_to_path_data(&path_vertex),
            advance_width,
            units_per_em,
            ascender,
            descender,
            left_side_bearing,
            char_code: Some(char_code),
        }
    }
}

impl<Data: std::ops::Deref<Target=[u8]>> std::ops::Deref for Font<Data> {
    type Target = stt::FontInfo<Data>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn vertex_to_path_data(vvs: &Vec<stt::Vertex>) -> PathData {
    let mut path_data = PathData::new();
    for item in vvs {
        match item.vertex_type() {
            stt::VertexType::MoveTo => {
                path_data.move_to(item.x as f32, item.y as f32);
            }
            stt::VertexType::LineTo => {
                path_data.line_to(item.x as f32, item.y as f32);
            }
            stt::VertexType::CurveTo => {
                path_data.quad_to(item.x as f32, item.y as f32, item.cx as f32, item.cy as f32);
            }
        }
    }
    if path_data.len() > 0 { path_data.close(); }
    path_data
}

pub struct FontMap<Data: std::ops::Deref<Target=[u8]>>(HashMap<String, Box<Font<Data>>>);

pub struct FontCache<Data: std::ops::Deref<Target=[u8]>> {
    font_map: FontMap<Data>,
    glyph_indexes: HashMap<(String, u32), usize>,
    glyph_caches: Vec<Box<Glyph>>,
}

impl<Data: std::ops::Deref<Target=[u8]>> FontCache<Data> {
    pub fn new() -> Self {
        FontCache {
            font_map: FontMap::new(),
            glyph_indexes: HashMap::new(),
            glyph_caches: vec![],
        }
    }
    pub fn load_font_bytes(&mut self, font_name: String, data: Data) -> Option<()> {
        let font = Font::new(data)?;
        self.font_map.insert(font_name, Box::new(font));
        Some(())
    }
    pub fn has_glyph(&self, font_name: String, c: u32) -> bool {
        let result = self.glyph_indexes.get(&(font_name.clone(), c.clone()));
        result.is_some()
    }
    pub fn check_glyph(&mut self, font_name: String, c: u32) {
        let result = self.glyph_indexes.get(&(font_name.clone(), c.clone()));
        if result.is_none() {
            let glyph = self.font_map.char_to_glyph(font_name.clone(), c.clone() as u32);
            self.glyph_caches.push(glyph);
            self.glyph_indexes.insert((font_name, c), self.glyph_caches.len() - 1);
        }
    }
    pub fn get_glyph_cache_count(&self) -> usize {
        self.glyph_caches.len()
    }
    pub fn get_font_cache_count(&self) -> usize {
        self.font_map.0.len()
    }
}

impl<Data: std::ops::Deref<Target=[u8]>> FontMap<Data> {
    pub fn new() -> Self {
        FontMap(HashMap::new())
    }
}

impl<Data: std::ops::Deref<Target=[u8]>> std::ops::Deref for FontMap<Data> {
    type Target = HashMap<String, Box<Font<Data>>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Data: std::ops::Deref<Target=[u8]>> std::ops::DerefMut for FontMap<Data> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Data: std::ops::Deref<Target=[u8]>> FontMap<Data> {
    fn char_to_glyph(&self, font_name: String, c: u32) -> Box<Glyph> {
        if c == 8203u32 {
            return Box::new(Glyph::get_none());
        }
        let mut result = self.get(&font_name);
        let mut is_default = false;
        if result.is_none() {
            result = self.get("default");
            is_default = true;
        }
        let mut font = result.unwrap();
        let find_index = font.find_glyph_index(c.clone());
        if find_index == 0 && !is_default {
            font = self.get("default").unwrap();
        }
        let boxed = Box::new(font.get_glyph(c));
        boxed
    }
}

impl<Data: std::ops::Deref<Target=[u8]>> MergedFont for FontCache<Data> {
    fn char_to_glyph<'a>(&'a self, font_name: String, c: char) -> &'a Box<Glyph> {
        let c = c as u32;
        let result = self.glyph_indexes.get(&(font_name.clone(), c));
        &self.glyph_caches.get(*result.unwrap()).unwrap()
    }
}



