use super::svg_util::render::*;
use super::svg_util::svg_methods::*;
use core::open_type_like::command::{CommandSegment, CommandsList};
use core::open_type_like::path::{PathSegment, PathData, BoundingBox};
use core::open_type_like::transform::Transform;
use core::data::text_data::{ArtTextOption, Gradient};
use uuid::Uuid;
use imagesize::blob_size;
use svg::Node;
use bytes::Bytes;

pub struct Color(u8, u8, u8, f32);

impl From<(u8, u8, u8, f32)> for Color {
    fn from((r, g, b, a): (u8, u8, u8, f32)) -> Self {
        Color(r, g, b, a)
    }
}

trait ToColorString {
    fn to_rgb(&self) -> String;
    fn to_rgba(&self) -> String;
}

impl ToColorString for Color {
    fn to_rgb(&self) -> String {
        let Color(r, g, b, _) = self;
        format!("rgb({},{},{})", *r, *g, *b)
    }

    fn to_rgba(&self) -> String {
        let Color(r, g, b, a) = self;
        format!("rgba({},{},{},{})", *r, *g, *b, *a)
    }
}

pub fn exec_text(commands: &CommandsList, width: f32, height: f32, scale: f32) -> String {
    let mut ctx = Context::new(width, height);

    ctx.line_cap = "round".to_string();
    ctx.line_join = "round".to_string();

    for command in commands.iter() {
        match command {
            CommandSegment::Path(ref data) => {
                let mut data = data.clone();
                data.transform(Transform::new(scale, 0.0, 0.0, scale, 0.0, 0.0));
                ctx.begin_path();
                for path_segment in data.iter() {
                    match path_segment {
                        PathSegment::MoveTo { ref x, ref y } => {
                            ctx.move_to(*x, *y)
                        }
                        PathSegment::LineTo { ref x, ref y } => {
                            ctx.line_to(*x, *y);
                        }
                        PathSegment::CurveTo { ref x, ref y, ref x1, ref y1, ref x2, ref y2, } => {
                            ctx.curve_to(*x1, *y1, *x2, *y2, *x, *y);
                        }
                        PathSegment::ClosePath => {
                            ctx.close();
                        }
                    }
                }
            }
            CommandSegment::Fill(ref color) => {
                let (r, g, b, a) = split_color_string(color).unwrap_or((0, 0, 0, 1.0));
                ctx.fill_style = Style::Color(r, g, b, a);
                ctx.stroke_style = Style::Color(r, g, b, a);
                ctx.fill(None);
            }
            CommandSegment::Stroke(ref color, ref width) => {
                if *width == 0f64 { continue; }
                let (r, g, b, a) = split_color_string(color).unwrap_or((0, 0, 0, 1.0));
                ctx.stroke_style = Style::Color(r, g, b, a);
                ctx.line_width = *width as f32;
                ctx.stroke(None);
            }
            CommandSegment::Transform(ref t, ref f) => {
                let transform = t.clone();
                if *f {
                    ctx.reset_transform()
                }
                ctx.set_transform(transform)
            }
            CommandSegment::Clip => {}
            _ => {}
        }
    }
    ctx.as_svg()
}

pub fn simply_command(commands: &CommandsList) -> PathData {
    let mut transform: Transform = Default::default();
    let mut path = PathData::new();
    for command in commands.iter() {
        match command {
            CommandSegment::Transform(ref t, ref _f) => {
                transform = t.clone()
            }
            CommandSegment::Path(ref data) => {
                for path_segment in data.iter() {
                    match path_segment {
                        PathSegment::MoveTo { ref x, ref y } => {
                            let (x, y) = transform.apply(*x, *y);
                            path.move_to(x, y)
                        }
                        PathSegment::LineTo { ref x, ref y } => {
                            let (x, y) = transform.apply(*x, *y);
                            path.line_to(x, y);
                        }
                        PathSegment::CurveTo { ref x, ref y, ref x1, ref y1, ref x2, ref y2, } => {
                            let (x, y) = transform.apply(*x, *y);
                            let (x1, y1) = transform.apply(*x1, *y1);
                            let (x2, y2) = transform.apply(*x2, *y2);
                            path.curve_to(x, y, x1, y1, x2, y2);
                        }
                        PathSegment::ClosePath => {
                            path.close();
                        }
                    }
                }
            }
            _ => {}
        }
    }
    path
}

pub fn exec_art_text(commands: &CommandsList, width: f32, height: f32, ref_size: f32, config: ArtTextOption, texture_raw: Option<Bytes>) -> String {
    let ArtTextOption { fill, texture, stroke, shadow } = config;
    if fill.is_none() && stroke.len() == 0 && shadow.len() == 0 {
        return exec_text(commands, width, height, 1.0);
    }
    let path_data = simply_command(commands);
    let bbox = path_data.get_bounding_box().unwrap_or_else(|| BoundingBox::new(0.0, 0.0));
    let b_width = bbox.get_width();
    let b_height = bbox.get_height();

    let uuid = into_str!["u", Uuid::new_v4().to_string()];
    let ori = (width.powf(2.0) + height.powf(2.0)).sqrt();

    let max_stroke_width = if stroke.len() == 0 {
        0f32
    } else {
        let (_, width) = stroke.last().unwrap();
        *width
    };

    let mut stroke_box = bbox.clone();
    stroke_box.extends(max_stroke_width);

    let mut union_box = stroke_box.clone();

    if shadow.len() > 0 {
        for (_color, (x, y), blur) in shadow.iter() {
            let mut shadow_box = stroke_box.clone();
            shadow_box.extends(blur * ori * 2.0);
            shadow_box.move_t(x * ori * 2.0, y * ori * 2.0);
            union_box = union_box.merge(&shadow_box);
        }
    }

    let mut defs = create_defs_tag();
    let mut content = group(vec![]);
    let mut path = create_path_tag(String::from(&path_data));
    path.assign("stroke-linecap", into_str!["round"]);
    path.assign("stroke-linejoin", into_str!["round"]);
    path.assign("id", into_str![&uuid, "-path"]);
    defs.append(path);

    if shadow.len() > 0 {
        let line_width = max_stroke_width;
        let style = create_style_tag(&into_str![".", &uuid,"-shadow { stroke-width: ", line_width * ref_size, "; stroke: #000000; fill: #ffffff; }"]);
        defs.append(style);

        let mut len = shadow.len() - 1;
        let default_shadow = ((0u8, 0u8, 0u8, 1f32), (0f32, 0f32), 0f32);
        loop {
            let (color, (x, y), blur) = shadow.get(len).unwrap_or(&default_shadow).clone();
            let mut use_s = create_use_tag(into_str![&uuid, "-path"]);
            use_s.assign("class", into_str![&uuid, "-shadow"]);
            let mut g = group(vec![use_s]);
            apply_shadow(&mut defs, &mut g, color, (x * ori, y * ori), blur * ori);
            content.append(g);
            if len == 0 { break; }
            len -= 1;
        }
    }

    if stroke.len() > 0 {
        let mut len = stroke.len() - 1;
        let default_stroke = ((0u8, 0u8, 0u8, 1f32), 0f32);
        loop {
            let (color, width) = stroke.get(len).unwrap_or(&default_stroke).clone();
            let use_s = create_use_tag(into_str![&uuid, "-path"]);
            let mut g = group(vec![use_s]);
            let color: Color = color.into();
            g.assign("stroke-width", into_str![width * ref_size]);
            g.assign("stroke", color.to_rgb());
            g.assign("fill", color.to_rgb());
            content.append(g);
            if len == 0 { break; }
            len -= 1;
        }
    }

    let mut use_clip = create_use_tag(into_str![&uuid, "-path"]);
    use_clip.assign("fill", "#ffffff");
    let mask = create_mask_tag(vec![use_clip], into_str![&uuid, "-mask"]);
    defs.append(mask);

    if texture.is_some() {
        let texture = texture.unwrap();
        let byte_data = if texture_raw.is_some() {
            texture_raw
        } else {
            super::svg_util::url::fetch(&texture)
        };
        if byte_data.is_some() {
            let byte_data = byte_data.unwrap();
            let mut cw = b_width;
            let mut ch = b_height;
            let mut x = bbox.x1;
            let mut y = bbox.y1;
            let result = blob_size(&byte_data);
            if result.is_ok() {
                let result = result.unwrap();
                if b_width / b_height > result.width as f32 / result.height as f32 {
                    cw = b_width;
                    ch = b_width / result.width as f32 * result.height as f32;
                    y -= (ch - b_height) / 2.0;
                } else {
                    ch = b_height;
                    cw = b_height / result.height as f32 * result.width as f32;
                    x -= (cw - b_width) / 2.0;
                }
            }
            let mut image = create_image_tag(into_str!["data:image/png;base64,", base64::encode(byte_data)]);
            image.assign("x", x);
            image.assign("y", y);
            image.assign("width", cw);
            image.assign("height", ch);
            let mut g = group(vec![image]);
            g.assign("mask", format!("url(#{}-mask)", &uuid));
            content.append(g);
        }
    }
    if fill.is_some() {
        let Gradient { type_: _, vector, stop } = fill.unwrap();

        let stop = {
            let mut n = Vec::<(String, String)>::new();
            for (k, v) in stop.iter() {
                let color: Color = v.clone().into();
                n.push((k.to_string(), color.to_rgba()));
            }
            n
        };

        let lg = create_linear_gradient(vector, stop, into_str![&uuid, "-linear"]);
        defs.append(lg);
        let mut rect = create_rect_tag(width, height, 0f32, 0f32);
        rect.assign("fill", format!("url(#{}-linear)", &uuid));
        let mut g = group(vec![rect]);
        g.assign("mask", format!("url(#{}-mask)", &uuid));
        content.append(g);
    }

    let source_box = BoundingBox {
        x1: 0.0,
        y1: 0.0,
        x2: width,
        y2: height,
    };

    union_box = union_box.merge(&source_box);

    let w = union_box.get_width();
    let h = union_box.get_height();
    let dx = if union_box.x1 < 0.0 { union_box.x1 } else { 0.0 };
    let dy = if union_box.y1 < 0.0 { union_box.y1 } else { 0.0 };

    let mut svg = create_svg_tag(w, h);
    svg.assign("viewBox", format!("{} {} {} {}", dx, dy, w, h));
    svg.add(defs)
        .add(content)
        .to_string()
}