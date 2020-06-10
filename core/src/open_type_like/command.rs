use super::path::PathData;
use super::super::data::text_data::{TextBlock, TextBlockDetail};

use super::transform::Transform;

use std::f64::consts::PI;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use crate::data::text_data::WritingMode;

macro_rules! join_str {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = String::from("");
            $(
                let s = $x;
                temp_vec.push_str(&s.to_string());
            )*
            temp_vec
        }
    };
}

#[derive(Debug, Clone)]
pub enum CommandSegment {
    Use(String, u32, f64),
    Path(PathData),
    Transform(Transform, bool),
    Fill(String),
    Stroke(String, f64),
    Clip,
}

fn get_transform_str(t: &Transform) -> String {
    let Transform { a, b, c, d, e, f } = t;
    join_str!["{\"a\": ",a,", \"b\": ",b,", \"c\": ",c,", \"d\": ",d,", \"e\": ",e,", \"f\": ",f,"}"]
}

impl From<&CommandSegment> for String {
    fn from(com: &CommandSegment) -> String {
        match com {
            CommandSegment::Use(ref a, ref b, ref c) => {
                join_str!["{\"type\": \"use\", \"source\": ", a, ", \"b\": ", b, ", \"c\": ", c, "}"]
            }
            CommandSegment::Clip => {
                join_str!["{\"type\": \"clip\"}"]
            }
            CommandSegment::Fill(ref a) => {
                format!(r#"{{"type": "fill", "value": {:?}}}"#, a)
            }
            CommandSegment::Path(ref a) => {
                join_str!["{\"type\": \"path\", \"value\": ", &String::from(a), "}"]
            }
            CommandSegment::Stroke(ref a, ref b) => {
                format!(r#"{{"type": "stroke", "value": {:?}, "width": {:?}}}"#, a, b)
            }
            CommandSegment::Transform(ref a, ref _b) => {
                join_str!("{\"type\": \"transform\", \"value\": ", get_transform_str(a), "}")
            }
        }
    }
}

impl From<&CommandSegment> for Vec<f32> {
    fn from(item: &CommandSegment) -> Self {
        match item {
            CommandSegment::Transform(ref a, ref _b) => {
                let Transform { a, b, c, d, e, f } = a;
                vec![0f32, *a as f32, *b as f32, *c as f32, *d as f32, *e as f32, *f as f32]
            }
            CommandSegment::Path(ref a) => {
                let mut packed = vec![1f32];
                let path_data: Vec<f32> = a.into();
                for item in path_data.iter() {
                    packed.push(*item);
                }
                packed
            }
            CommandSegment::Stroke(ref a, ref b) => {
                let without_prefix = a.trim_start_matches("#");
                let color = i64::from_str_radix(without_prefix, 16u32).unwrap_or(0i64) as i64;
                vec![2f32, *b as f32, color as f32]
            }
            CommandSegment::Fill(ref a) => {
                let without_prefix = a.trim_start_matches("#");
                let color = i64::from_str_radix(without_prefix, 16u32).unwrap_or(0i64) as i64;
                vec![3f32, color as f32]
            }
            _ => vec![]
        }
    }
}

#[derive(Debug, Clone)]
pub struct Command(CommandSegment);

impl Command {}


#[derive(Debug, Clone)]
pub struct CommandList<'a>(&'a Vec<(TextBlock, TextBlockDetail<'a>)>);

impl<'a> CommandList<'a> {
    pub fn new(d: &'a Vec<(TextBlock, TextBlockDetail)>) -> Self {
        CommandList(d)
    }

    fn get_transform(block: &TextBlock, detail: &TextBlockDetail) -> CommandSegment {
        let a = 1f32;
        let b = 0f32;
        let d = 1f32;

        let x = detail.position.0 as f32;
        let y = detail.position.1 as f32;
        let line_height = detail.line_height as f32;

        let c = if block.italic { (-(PI * 15f64 / 180f64).sin()) as f32 } else { 0f32 };
        let e = x - line_height * c;
        let f = y;
        CommandSegment::Transform(Transform { a, b, c, d, e, f }, false)
    }

    fn get_path_commands(&self) -> HashMap<(String, u32), PathData> {
        let mut paths = HashMap::<(String, u32), PathData>::new();
        for item in self.iter() {
            let (b, d) = item;
            let path = d.glyph.get_path(0f32, 0f32, 100f32, &d.writing_mode);
            let font_family = &b.font_family;
            let mut chars = b.text.chars();
            while let Some(text) = chars.next() {
                let unicode = text as u32;
                &paths.insert((font_family.to_string(), unicode), path.clone());
            }
        }
        paths
    }

    fn get_stroke(block: &TextBlock) -> CommandSegment {
        let font_size = block.font_size;
        let fill = block.fill.clone();
        let mut stroke_width = block.stroke_width as f64 * font_size as f64 / 20f64;
        if stroke_width > 0f64 && stroke_width < 0.42f64 { stroke_width = 0.42f64 }

        CommandSegment::Stroke(fill, stroke_width)
    }

    fn get_decoration(block: &TextBlock, detail: &TextBlockDetail) -> Vec<CommandSegment> {
        let decoration = &block.decoration;
        let font_size = block.font_size;
        let (x, mut y) = detail.position;
        let b_width = detail.b_width;
        let writing_mode = &detail.writing_mode;
        let line_width = 0.04f32 * font_size as f32;
        y += line_width;
        let transform = CommandSegment::Transform(Default::default(), true);
        let mut path_data = PathData::new();

        match writing_mode {
            &WritingMode::HorizontalTB => {
                match decoration.as_ref() {
                    "line-through" => {}
                    "overline" => {}
                    "underline" => {
                        path_data.move_to(x, y);
                        path_data.line_to(x + b_width, y);
                        path_data.line_to(x + b_width, y + line_width);
                        path_data.line_to(x, y + line_width);
                        path_data.line_to(x, y);
                        path_data.close();
                    }
                    _ => {}
                }
            }
            _ => {
                match decoration.as_ref() {
                    "line-through" => {}
                    "overline" => {}
                    "underline" => {
                        path_data.move_to(x, y);
                        path_data.line_to(x, y + b_width);
                        path_data.line_to(x + line_width, y + b_width);
                        path_data.line_to(x + line_width, y);
                        path_data.line_to(x, y);
                        path_data.close();
                    }
                    _ => {}
                }
            }
        }

        let mut result = Vec::<CommandSegment>::new();
        if !path_data.is_empty() {
            result.push(transform);
            result.push(CommandSegment::Path(path_data));
            result.push(CommandSegment::Fill(block.fill.clone()));
        }
        result
    }

    pub fn get_commands(&self) -> (HashMap<(String, u32), PathData>, Vec<CommandSegment>) {
        let paths = self.get_path_commands();
        let mut commands = Vec::<CommandSegment>::new();
        for (b, d) in self.iter() {
            let mut chars = b.text.chars();
            while let Some(text) = chars.next() {
                let transform = Self::get_transform(b, d);
//                transform
                let path = CommandSegment::Use(b.font_family.to_string(), text as u32, b.font_size as f64);
                let fill = CommandSegment::Fill(b.fill.to_string());
                let stroke = Self::get_stroke(b);
                let decoration = Self::get_decoration(b, d);
                commands.push(transform);
                commands.push(path);
                commands.push(fill);
                commands.push(stroke);
                for command in decoration {
                    commands.push(command);
                }
            }
        }
        (paths, commands)
    }
}

impl<'a> Deref for CommandList<'a> {
    type Target = &'a Vec<(TextBlock, TextBlockDetail<'a>)>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct CommandsList(Vec<CommandSegment>);

impl CommandsList {
    pub fn new() -> Self {
        CommandsList(Vec::<CommandSegment>::new())
    }
}

impl Deref for CommandsList {
    type Target = Vec<CommandSegment>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CommandsList {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<&CommandsList> for String {
    fn from(l: &CommandsList) -> Self {
        let mut result = String::from("[");
        for segment in l.iter() {
            result.push_str(&String::from(segment));
            result.push_str(",");
        }
        if l.len() > 0 { result.pop(); }
        result.push_str("]");
        result
    }
}

impl From<&CommandsList> for Vec<f32> {
    fn from(item: &CommandsList) -> Self {
        let mut result = Vec::<f32>::new();
        result.push(item.len() as f32);
        for command in item.iter() {
            let packed: Vec<f32> = command.into();
            for item in packed.iter() {
                result.push(*item);
            }
        }
        result
    }
}


pub fn tran_commands_stream(content: &(HashMap<(String, u32), PathData>, Vec<CommandSegment>)) -> CommandsList {
    let (g_path, source_commands) = content;
    let mut commands = CommandsList::new();
    for command in source_commands {
        match command {
            CommandSegment::Use(font_family, unicode, font_size) => {
                let default_path_data = PathData::new();
                let mut result = g_path.get(&(font_family.clone(), *unicode)).unwrap_or(&default_path_data).clone();
                result.transform(Transform::new(*font_size as f32 / 100f32, 0f32, 0f32, *font_size as f32 / 100f32, 0f32, 0f32));
                (commands.0).push(CommandSegment::Path(result));
            }
            _ => {
                (commands.0).push(command.clone());
            }
        }
    }
    commands
}