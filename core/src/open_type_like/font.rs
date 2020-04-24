use super::super::data::font_data::FontData;
use super::glyph::Glyph;

pub trait Font {
    fn char_to_glyph(&self, c: &str, default_font: Option<&FontData>) -> Option<Vec<(String, Box<Glyph>)>>;
    fn has_char(&self, c: &str) -> bool;
}

impl Font for FontData {
    fn char_to_glyph(&self, c: &str, default_font: Option<&FontData>) -> Option<Vec<(String, Box<Glyph>)>> {
        let mut chars = c.chars();
        let mut char_glyphs = Vec::<(String, Box<Glyph>)>::new();

        while let Some(c) = chars.next() {
            let index = c as u32;
            let pack_index = self.glyph_index_map.get(&index.to_string());
//            let mut aw = &200i32;
//            let mut gs = &"".to_string();
            match pack_index {
                Some(i) => {
                    let (_a, g) = &self.glyphs_pack[*i];
                    char_glyphs.push((c.to_string(), g.clone()));
                }
                None => {
                    if let Some(fall_back) = default_font {
                        let pack_index = fall_back.glyph_index_map.get(&index.to_string());
                        if let Some(i) = pack_index {
                            let (_a, g) = &fall_back.glyphs_pack[*i];
                            char_glyphs.push((c.to_string(), g.clone()));
                        }
                    }
                }
            }
        }
        Some(char_glyphs)
    }

    fn has_char(&self, c: &str) -> bool {
        let mut chars = c.chars();
        match chars.next() {
            Some(c) => {
                let index = c as u32;
                let pack_index = self.glyph_index_map.get(&index.to_string());
                match pack_index {
                    Some(_) => return true,
                    None => return false
                }
            }
            None => return false
        }
    }
}