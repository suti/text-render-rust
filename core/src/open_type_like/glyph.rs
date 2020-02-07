use svgtypes::{PathParser, PathSegment};

use super::path::PathData;
use super::transform::Transform;
#[derive(Debug, Clone)]
pub struct Glyph {
    pub path: PathData,
    pub advance_width: i32,
    pub units_per_em: i32,
    pub ascender: i32,
    pub descender: i32,
}

impl Glyph {
    pub fn parse(source: &str, advance_width: i32, units_per_em: i32, ascender: i32, descender: i32) -> Option<Glyph> {
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
        })
    }

    pub fn get_none() -> Self {
        Glyph {
            path: PathData(vec![]),
            advance_width: 0i32,
            units_per_em: 1000i32,
            ascender: 900i32,
            descender: -100i32,
        }
    }

    pub fn to_path(&self) -> String {
        String::from("")
    }

    pub fn get_path(&self, x: f32, y: f32, font_size: f32) -> PathData {
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
        path_data.transform(transform);
        path_data
    }

    pub fn get_advance_width(&self, font_size: f32) -> f32 {
        (self.advance_width as f32) / (self.units_per_em as f32) * font_size
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
        PathSegment::MoveTo { x, y, abs } => {
            p.move_to(x as f32, y as f32)
        }
        PathSegment::LineTo { x, y, abs } => {
            p.line_to(x as f32, y as f32)
        }
        PathSegment::CurveTo { x, y, x1, y1, x2, y2, abs } => {
            p.curve_to(x as f32, y as f32, x1 as f32, y1 as f32, x2 as f32, y2 as f32)
        }
        PathSegment::Quadratic { x, y, x1, y1, abs } => {
            p.quad_to(x as f32, y as f32, x1 as f32, y1 as f32)
        }
        PathSegment::ClosePath { abs } => {
            p.close()
        }
        _ => {
            println!("Do not support {:?}", d)
        }
    }
}




