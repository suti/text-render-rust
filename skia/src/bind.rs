extern crate skia_safe;

use core::open_type_like::command::tran_commands_stream;
use core::open_type_like::command::CommandSegment;
use core::open_type_like::path::PathSegment;
use core::open_type_like::transform::Transform;

use std::collections::HashMap;

use skia_safe::{Canvas, Path, Paint, Matrix, paint::Style, Color4f, Point, Color, Surface, EncodedImageFormat, Data};



fn p(x: &f32, y: &f32) -> Point {
    Point {
        x: *x as f32,
        y: *y as f32,
    }
}

pub fn exec_skia_command(source: &Vec<CommandSegment>, width: f32, height: f32, scale: f32) -> Option<Data> {
    let width_height = ((width * scale) as i32, (height * scale) as i32);
    let mut surface = Surface::new_raster_n32_premul(width_height).expect("no surface");
    let mut canvas = surface.canvas();
    let mut target_path = Path::default();
    canvas.reset_matrix();
    for command in source {
        match command {
            CommandSegment::Path(ref data) => {
                target_path = Path::default();
                let mut data = data.clone();
                data.transform(Transform::new(scale as f32, 0.0, 0.0, scale as f32, 0.0, 0.0));
                for path_segment in data.iter() {
                    match path_segment {
                        PathSegment::MoveTo { ref x, ref y } => {
                            target_path.move_to(p(x, y));
                        }
                        PathSegment::LineTo { ref x, ref y } => {
                            target_path.line_to(p(x, y));
                        }
                        PathSegment::CurveTo { ref x, ref y, ref x1, ref y1, ref x2, ref y2, } => {
                            target_path.cubic_to(p(x1, y1), p(x2, y2), p(x, y));
                        }
                        PathSegment::ClosePath => {
                            target_path.close();
                        }
                    }
                }
            }
            CommandSegment::Fill(ref color) => {
                let mut paint = Paint::default();
                let without_prefix = color.trim_start_matches("#");
                let color = i64::from_str_radix(without_prefix, 16u32).unwrap() as i64;
                paint.set_style(Style::Fill);
                paint.set_color(Color::from_argb(255u8, (color >> 16) as u8, (color >> 8) as u8, color as u8));
//                paint.set_anti_alias(true);
                canvas.draw_path(&target_path, &paint);
            }
            CommandSegment::Stroke(ref color, ref width) => {
                let mut paint = Paint::default();
                let without_prefix = color.trim_start_matches("#");
                let color = i64::from_str_radix(without_prefix, 16u32).unwrap() as i64;
                paint.set_style(Style::Stroke);
                paint.set_color(Color::from_argb(255u8, (color >> 16) as u8, (color >> 8) as u8, color as u8));
                paint.set_stroke_width(*width as f32 * scale);
//                paint.set_anti_alias(true);
                canvas.draw_path(&target_path, &paint);
            }
            CommandSegment::Transform(ref t, ref f) => {
                let transform = Matrix::new_all(t.a as f32, t.c as f32, t.e as f32 * scale, t.b as f32, t.d as f32, t.f as f32 * scale, 0f32, 0f32, 1f32);
                if *f {
                    canvas.reset_matrix();
                }
                canvas.set_matrix(&transform);
            }
            CommandSegment::Clip => {}
            _ => {}
        }
    }
    let image = surface.image_snapshot();
    image.encode_to_data(EncodedImageFormat::PNG)
}
