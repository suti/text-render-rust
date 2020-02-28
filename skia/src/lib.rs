mod bind;

pub use bind::exec_skia_command;
use core::typesetting::MergedFont;
use core::data::font_data::FontData;
use core::open_type_like::glyph::Glyph;
use std::collections::HashMap;

pub struct FontMap {
    map: HashMap<String, Box<FontData>>
}

impl FontMap {
    pub fn new() -> Self {
        FontMap {
            map: HashMap::new()
        }
    }
}

impl std::ops::Deref for FontMap {
    type Target = HashMap<String, Box<FontData>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl std::ops::DerefMut for FontMap {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl MergedFont for FontMap {
    fn char_to_glyph<'a>(&'a self, font_name: String, c: char) -> &'a Box<Glyph> {
        let mut result = self.get(&font_name);
        if result.is_none() {
            result = self.get("default");
        }
        let font = result.unwrap();
        let index = c as u32;

        let pack_index = font.glyph_index_map.get(&index.to_string());
        let mut i = &0usize;
        if pack_index.is_some() {
            i = pack_index.unwrap();
        }
        let (a, g) = &font.glyphs_pack[*i];
        g
    }
}

#[cfg(test)]
mod tests {
    use core::open_type_like::command::tran_commands_stream;
    use core::typesetting::compute_render_command;
    use core::data::text_data::TextData;
    use core::data::font_data::FontData;

    use std::collections::HashMap;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::prelude::*;

    use super::bind::exec_skia_command;
    use crate::FontMap;

    #[test]
    fn it_works() {
        let mut read = File::open("/Users/suti/start/text-render-rust/skia/src/f1.json").unwrap();
        let mut test_font_json = String::from("");
        read.read_to_string(&mut test_font_json);

        let mut read = File::open("/Users/suti/start/text-render-rust/skia/src/t.json").unwrap();
        let mut test_text_json = String::from("");
        read.read_to_string(&mut test_text_json);
        let test_text_data = TextData::parse(&test_text_json).unwrap();
        let mut map = HashMap::<String, String>::new();
        map.insert("Noto Sans S Chinese Regular".to_string(), test_font_json.clone());
        map.insert("default".to_string(), test_font_json.clone());
        let mut font_data_parsed: HashMap<String, Box<FontData>> = HashMap::new();
        for (ff, font_data) in map.iter() {
            let font = Box::new(FontData::parse(font_data).unwrap());
            font_data_parsed.insert(ff.clone(), font);
        }
        let mut font_data_ref = FontMap::new();
        for (ff, font_data) in font_data_parsed.iter() {
            font_data_ref.insert(ff.clone(), font_data.clone());
        }
        let text_data = TextData::parse(&test_text_json).unwrap();
        let (b_box, result, _) = compute_render_command(&text_data, &font_data_ref).unwrap();
        let b_box = b_box.get_total_box();
        let mut width = b_box.get_width().ceil() as f32;
        let height = b_box.get_height().ceil() as f32;
        if test_text_data.width > width {
            width = test_text_data.width
        }
        let result1 = tran_commands_stream(&result);
        let result = exec_skia_command(&result1, width, height, 2.0).unwrap();
        let time = std::time::SystemTime::now();
        let st = String::from(&result1);
        if let Ok(d) = std::time::SystemTime::now().duration_since(time) {
            println!("{:?}", d);
        }
        let mut file = File::create("result.png").unwrap();
        file.write(result.as_bytes());
    }
}
