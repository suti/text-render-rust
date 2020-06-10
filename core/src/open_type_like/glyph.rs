use svgtypes::{PathParser, PathSegment};

use super::path::PathData;
use super::transform::Transform;
use crate::data::text_data::WritingMode;

#[derive(Debug, Clone)]
pub struct Glyph {
    pub path: PathData,
    pub advance_width: i32,
    pub units_per_em: i32,
    pub ascender: i32,
    pub descender: i32,
    pub char_code: Option<u32>,
    pub left_side_bearing: i32,
}

fn is_orientation(char_code: u32) -> bool {
    char_code > 32 && char_code < 126 || char_code == 32
}

impl Glyph {
    pub fn parse(source: &str, advance_width: i32, units_per_em: i32, ascender: i32, descender: i32, left_side_bearing: i32) -> Option<Glyph> {
        let result = PathParser::from(source);
        let mut path_data = PathData::new();
        for d_r in result {
            let d = d_r.unwrap();
            add_path_segment(&mut path_data, d);
        }

        Some(Glyph {
            path: path_data,
            advance_width,
            units_per_em,
            ascender,
            descender,
            left_side_bearing,
            char_code: None,
        })
    }

    pub fn get_none() -> Self {
        Glyph {
            path: PathData(vec![]),
            advance_width: 0i32,
            units_per_em: 1000i32,
            ascender: 900i32,
            descender: -100i32,
            left_side_bearing: 0,
            char_code: None,
        }
    }

    pub fn to_path(&self) -> String {
        String::from(&self.path)
    }

    pub fn get_path(&self, x: f32, y: f32, font_size: f32, writing_mode: &WritingMode) -> PathData {
        let mut path_data = self.path.clone();
        let scale = 1.0f32 / (self.units_per_em as f32) * font_size;
        let mut transform = Transform {
            a: scale,
            b: 0.0f32,
            c: 0.0f32,
            d: -scale,
            e: x,
            f: y,
        };
        match writing_mode {
            WritingMode::HorizontalTB => (),
            _ => if self.char_code
                .and_then(|c| Some(is_orientation(c)))
                .unwrap_or(false)
            {
                transform.rotate(-90.0);
            } else {
                let dx = -0.05 * font_size / scale;
                let dy = -self.ascender as f32 / (self.ascender as f32 - self.descender as f32) * font_size / scale;
                transform.translate(dx, dy)
            }
        }
        path_data.transform(transform);
        path_data
    }

    pub fn get_advance_width(&self, font_size: f32) -> f32 {
        (self.advance_width as f32) / (self.units_per_em as f32) * font_size
    }

    pub fn get_advance_height(&self, font_size: f32) -> f32 {
        if self.advance_width == 0 {
            0.0
        } else {
            ((self.ascender - self.descender) / self.units_per_em) as f32 * font_size
        }
    }

    pub fn get_spacing(&self, font_size: f32, writing_mode: &WritingMode) -> f32 {
        match writing_mode {
            WritingMode::HorizontalTB => self.get_advance_width(font_size),
            _ => if self.char_code
                .and_then(|c| Some(is_orientation(c)))
                .unwrap_or(false)
            {
                self.get_advance_width(font_size)
            } else {
                self.get_advance_height(font_size)
            }
        }
    }
}

impl Default for Glyph {
    fn default() -> Self {
        Glyph {
            path: PathData(Vec::new()),
            advance_width: 0,
            units_per_em: 0,
            ascender: 0,
            descender: 0,
            left_side_bearing: 0,
            char_code: None,
        }
    }
}

//impl std::ops::Deref for Glyph {
//    type Target = String;
//
//    #[inline]
//    fn deref(&self) -> &Self::Target {
//        &self.path_str
//    }
//}


fn add_path_segment(p: &mut PathData, d: PathSegment) {
    match d {
        PathSegment::MoveTo { x, y, abs: _ } => {
            p.move_to(x as f32, y as f32)
        }
        PathSegment::LineTo { x, y, abs: _ } => {
            p.line_to(x as f32, y as f32)
        }
        PathSegment::CurveTo { x, y, x1, y1, x2, y2, abs: _ } => {
            p.curve_to(x as f32, y as f32, x1 as f32, y1 as f32, x2 as f32, y2 as f32)
        }
        PathSegment::Quadratic { x, y, x1, y1, abs: _ } => {
            p.quad_to(x as f32, y as f32, x1 as f32, y1 as f32)
        }
        PathSegment::ClosePath { abs: _ } => {
            p.close()
        }
        _ => {
            println!("Do not support {:?}", d)
        }
    }
}




