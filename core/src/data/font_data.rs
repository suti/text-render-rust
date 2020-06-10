use std::collections::HashMap;
use json::JsonValue::{Object, Array};
use json::JsonValue;
use super::super::open_type_like::glyph::Glyph;

#[derive(Debug, Clone)]
pub struct FontData {
    pub name: String,
    pub units_per_em: i32,
    pub ascender: i32,
    pub descender: i32,
    pub glyph_index_map: HashMap<String, usize>,
    pub glyphs_pack: Vec<(i32, Box<Glyph>)>,
    pub fall_back: Option<Box<FontData>>,
}

fn get_object(value: &JsonValue) -> Option<&json::object::Object> {
    if let Object(result) = value {
        Some(result)
    } else {
        None
    }
}

fn get_array(value: &JsonValue) -> Option<&json::Array> {
    if let Array(result) = value {
        Some(result)
    } else {
        None
    }
}

impl FontData {
    pub fn parse(source: &str) -> Option<FontData> {
        let json_d = json::parse(source).unwrap();
        let pack_data = get_array(&json_d)?;

        if pack_data.len() != 6 { return None; }

        let name = pack_data.get(0)?.as_str()?.to_string();
        let units_per_em = pack_data.get(1)?.as_i32()?;
        let ascender = pack_data.get(2)?.as_i32()?;
        let descender = pack_data.get(3)?.as_i32()?;

        let glyph_index_map_obj = get_object(pack_data.get(4)?)?;

        let mut glyph_index_map = HashMap::<String, usize>::new();
        for (str, value) in glyph_index_map_obj.iter() {
            glyph_index_map.insert(str.to_string(), value.as_usize()?);
        }

        let mut glyphs_pack = Vec::<(i32, Box<Glyph>)>::new();

        let glyphs_pack_arr = get_array(pack_data.get(5)?)?;

        for item in glyphs_pack_arr {
            let detail = get_array(item)?;
            let advance_width = detail.get(0)?.as_i32()?;
            let glyph_str = detail.get(1)?.as_str()?.to_string();
            let glyph: Box<Glyph> = Box::new(Glyph::parse(&glyph_str, advance_width, units_per_em, ascender, descender, 0)?);
            glyphs_pack.push((advance_width, glyph));
        }

        Some(FontData {
            name,
            units_per_em,
            ascender,
            descender,
            glyph_index_map,
            glyphs_pack,
            fall_back: None,
        })
    }
}

