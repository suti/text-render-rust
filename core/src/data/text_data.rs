use serde_json::Value;
use crate::open_type_like::glyph::Glyph;

#[derive(Debug, Clone)]
pub struct Shadow {
    pub blur: f32,
    pub offset: (f32, f32),
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct Gradient {
    pub type_: String,
    pub vector: (f32, f32),
    pub stop: Vec<(String, (u8, u8, u8, f32))>,
}

#[derive(Debug, Clone)]
pub struct ArtTextOption {
    pub fill: Option<Gradient>,
    pub texture: Option<String>,
    pub stroke: Vec<((u8, u8, u8, f32), f32)>,
    pub shadow: Vec<((u8, u8, u8, f32), (f32, f32), f32)>,
    pub use_: bool,
}

#[derive(Debug, Clone)]
pub enum ShadowOption {
    Some(Shadow),
    None,
}

#[derive(Debug, Clone)]
pub enum WritingMode {
    HorizontalTB,
    VerticalRL,
    VerticalLR,
}

#[derive(Debug, Clone)]
pub struct ParagraphContent {
    pub line_height: f32,
    pub paragraph_indentation: f32,
    pub blocks: Vec<TextBlock>,
}

#[derive(Debug, Clone)]
pub struct ParagraphData {
    pub writing_mode: WritingMode,
    pub text_align: String,
    pub resizing: String,
    pub align: String,
    pub paragraph_spacing: f32,
    pub paragraph_content: Vec<ParagraphContent>,
    pub art_text: Option<ArtTextOption>,
}

#[derive(Debug, Clone)]
pub struct TextBlock {
    pub text: String,
    pub font_family: String,
    pub font_size: f32,
    pub letter_spacing: f32,
    pub fill: String,
    pub italic: bool,
    pub stroke: String,
    pub stroke_width: f32,
    pub decoration: String,
}

impl Default for TextBlock {
    fn default() -> Self {
        TextBlock {
            text: "".to_string(),
            font_family: "".to_string(),
            font_size: 0.0,
            letter_spacing: 0.0,
            fill: "".to_string(),
            italic: false,
            stroke: "".to_string(),
            stroke_width: 0.0,
            decoration: "".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextData {
    pub width: f32,
    pub height: f32,
    pub paragraph: ParagraphData,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct TextBlockDetail<'a> {
    pub glyph: &'a Glyph,
    pub writing_mode: WritingMode,
    pub paragraph_indentation: f32,
    pub line_height: f32,
    pub text_align: String,
    pub resizing: String,
    pub align: String,
    pub paragraph_spacing: f32,
    pub b_width: f32,
    pub position: (f32, f32),
    pub base_line_to_top: f32,
    pub base_line_to_bottom: f32,
}

impl<'a> TextBlockDetail<'a> {
    pub fn default(glyph: &'a Glyph) -> Self {
        TextBlockDetail {
            glyph,
            writing_mode: WritingMode::HorizontalTB,
            paragraph_indentation: 0.0,
            line_height: 0.0,
            text_align: "".to_string(),
            resizing: "".to_string(),
            align: "".to_string(),
            paragraph_spacing: 0.0,
            b_width: 0.0,
            position: (0.0, 0.0),
            base_line_to_top: 0.0,
            base_line_to_bottom: 0.0,
        }
    }
}

fn split_color_string(s: &str) -> Option<(u8, u8, u8, f32)> {
    let s = s.replace(" ", "");
    if !s.contains("rgb") {
        if !s.contains("#") {
            return None;
        }
        let without_prefix = s.trim_start_matches("#");
        let color = i64::from_str_radix(without_prefix, 16u32).unwrap() as i64;
        let r = (color >> 16) as u8;
        let g = (color >> 8) as u8;
        let b = color as u8;
        let a = 1.0;
        Some((r, g, b, a))
    } else {
        let mode = if s.contains("rgba(") {
            "rgba("
        } else {
            "rgb("
        };
        let s_v: Vec<&str> = s.split(mode).collect();
        let s_v: Vec<&str> = s_v.get(1)?.split(")").collect();
        let s_v: Vec<&str> = s_v.get(0)?.split(",").collect();
        let r = s_v.get(0).unwrap_or(&"0").parse::<u8>().unwrap_or(0);
        let g = s_v.get(1).unwrap_or(&"0").parse::<u8>().unwrap_or(0);
        let b = s_v.get(2).unwrap_or(&"0").parse::<u8>().unwrap_or(0);
        let a = s_v.get(3).unwrap_or(&"1.0").parse::<f32>().unwrap_or(0.0f32);
        Some((r, g, b, a))
    }
}

impl TextData {
    pub fn parse(source: &str) -> Option<TextData> {
        let result = serde_json::from_str(source);
        if result.is_err() { return None; }
        let json: Value = result.unwrap();
        let default_num = Value::Number(serde_json::Number::from_f64(200f64).unwrap());
        let width = *&json.get("width").unwrap_or(&default_num).as_f64().unwrap_or_else(||
            *&json.get("width").unwrap_or(&default_num).as_str().unwrap_or("200").parse::<f64>().unwrap_or(200f64)
        ) as f32;
        let height = *&json.get("height").unwrap_or(&default_num).as_f64().unwrap_or_else(||
            *&json.get("height").unwrap_or(&default_num).as_str().unwrap_or("200").parse::<f64>().unwrap_or(200f64)
        ) as f32;

        if *&json["paragraph"].as_object().is_none() { return None; }

        let paragraph_json = &json["paragraph"].as_object().unwrap();
        let default_text_align = Value::String("center".to_string());
        let text_align = paragraph_json.get("textAlign").unwrap_or(&default_text_align).as_str().unwrap_or("center").to_string();
        let default_resizing = Value::String("grow-vertically".to_string());
        let resizing = paragraph_json.get("resizing").unwrap_or(&default_resizing).as_str().unwrap_or("grow-vertically").to_string();
        let default_align = Value::String("middle".to_string());
        let align = paragraph_json.get("align").unwrap_or(&default_align).as_str().unwrap_or("middle").to_string();
        let default_paragraph_spacing = Value::Number(serde_json::Number::from_f64(0.0).unwrap());
        let paragraph_spacing = paragraph_json.get("paragraphSpacing").unwrap_or(&default_paragraph_spacing).as_f64().unwrap_or(0.0) as f32;
        let mut art_text: Option<ArtTextOption> = None;

        let writing_mode = paragraph_json.get("writingMode")
            .and_then(|value| value.as_str())
            .and_then(|s| match s {
                "horizontal-tb" => Some(WritingMode::HorizontalTB),
                "vertical-rl" => Some(WritingMode::VerticalRL),
                "vertical-lr" => Some(WritingMode::VerticalLR),
                _ => None
            }).unwrap_or(WritingMode::HorizontalTB);

        if paragraph_json.get("advancedData").unwrap_or_else(|| &Value::Null).as_object().is_some() {
            let art_text_json = paragraph_json.get("advancedData").unwrap().as_object().unwrap();
            let get_fill = || {
                let type_ = "linear".to_string();
                let default_stop = vec![("0".to_string(), (0u8, 0u8, 0u8, 1.0f32))];

                if art_text_json.get("fill").is_some() {
                    let default_fill = r#"
                    {
                        "stop": {
                            "0": "rgba(0,0,0,1)"
                        },
                        "vector": [0, 1]
                    }"#;
                    let default_fill = serde_json::from_str(default_fill).unwrap();
                    let fill_json = art_text_json.get("fill").unwrap().as_object().unwrap_or(&default_fill);
                    let stop = {
                        let mut stop = Vec::<(String, (u8, u8, u8, f32))>::new();
                        let stop_json = fill_json.get("stop").unwrap().as_object().unwrap();
                        for (key, value) in stop_json.iter() {
                            let value = value.as_str().unwrap_or("rgba(0,0,0,1)");
                            let value = split_color_string(value).unwrap_or((0u8, 0u8, 0u8, 1.0f32));
                            stop.push((key.to_string(), value));
                        }
                        stop
                    };
                    let default_vec = vec![];
                    let vector = fill_json.get("vector").unwrap().as_array().unwrap_or(&default_vec);
                    let v0 = vector.get(0).unwrap_or(&Value::Number(serde_json::Number::from_f64(0.0).unwrap())).as_f64().unwrap_or(0.0) as f32;
                    let v1 = vector.get(1).unwrap_or(&Value::Number(serde_json::Number::from_f64(0.0).unwrap())).as_f64().unwrap_or(0.0) as f32;
                    Gradient {
                        type_,
                        vector: (v0, v1),
                        stop,
                    }
                } else {
                    Gradient {
                        type_,
                        vector: (0.0, 1.0),
                        stop: default_stop,
                    }
                }
            };
            let get_stroke = || {
                let default_vec = Value::Array(vec![]);
                let default_vec1 = vec![];
                let stroke = art_text_json.get("stroke").unwrap_or(&default_vec).as_array().unwrap_or(&default_vec1);
                let stroke = {
                    let mut v: Vec<((u8, u8, u8, f32), f32)> = vec![];
                    for item in stroke.iter() {
                        if item.as_object().is_none() {
                            continue;
                        }
                        let item = item.as_object().unwrap();
                        if let Some(hidden) = item.get("hidden") {
                            if let Some(hidden) = hidden.as_bool() {
                                if hidden { continue; }
                            }
                        }
                        if item.get("width").is_none() {
                            continue;
                        }
                        let color: (u8, u8, u8, f32) = {
                            if item.get("color").is_none() {
                                (0u8, 0u8, 0u8, 1.0f32)
                            } else {
                                let color_str = item.get("color").unwrap().as_str().unwrap_or("rgba(0,0,0,1)");
                                split_color_string(color_str).unwrap_or((0u8, 0u8, 0u8, 1.0f32))
                            }
                        };
                        let width = item.get("width").unwrap().as_f64().unwrap_or(0.0) as f32;
                        v.push((color, width))
                    }
                    v
                };
                stroke
            };
            let get_shadow = || {
                let default_vec = Value::Array(vec![]);
                let default_vec1 = vec![];
                let shadow = art_text_json.get("shadow").unwrap_or(&default_vec).as_array().unwrap_or(&default_vec1);
                let shadow = {
                    let mut v: Vec<((u8, u8, u8, f32), (f32, f32), f32)> = vec![];
                    for item in shadow.iter() {
                        if item.as_object().is_none() {
                            continue;
                        }
                        let item = item.as_object().unwrap();
                        if let Some(hidden) = item.get("hidden") {
                            if let Some(hidden) = hidden.as_bool() {
                                if hidden { continue; }
                            }
                        }
                        let blur =
                            if item.get("blur").is_none() {
                                0f32
                            } else {
                                item.get("blur").unwrap().as_f64().unwrap_or(0f64) as f32
                            };
                        let color: (u8, u8, u8, f32) =
                            if item.get("color").is_none() {
                                (0u8, 0u8, 0u8, 1.0f32)
                            } else {
                                let color_str = item.get("color").unwrap().as_str().unwrap_or("rgba(0,0,0,1)");
                                split_color_string(color_str).unwrap_or((0u8, 0u8, 0u8, 1.0f32))
                            };
                        let offset =
                            if item.get("offset").is_none() {
                                (0f32, 0f32)
                            } else {
                                let default_num = Value::Number(serde_json::Number::from_f64(0.0).unwrap());
                                let default_vec = vec![];
                                let offset = item.get("offset").unwrap().as_array().unwrap_or(&default_vec);
                                let o0 = offset.get(0).unwrap_or(&default_num.clone()).as_f64().unwrap_or(0.0) as f32;
                                let o1 = offset.get(1).unwrap_or(&default_num.clone()).as_f64().unwrap_or(0.0) as f32;
                                (o0, o1)
                            };
                        v.push((color, offset, blur))
                    }
                    v
                };
                shadow
            };

            let fill = Some(get_fill());
            let texture =
                if art_text_json.get("texture").is_some() {
                    let text = art_text_json.get("texture").unwrap().as_str();
                    if text.is_some() {
                        Some(text.unwrap().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };
            let stroke = get_stroke();
            let shadow = get_shadow();
            let use_ = art_text_json.get("use").and_then(|v| v.as_bool()).unwrap_or(true);
            art_text = Some(ArtTextOption {
                fill,
                texture,
                stroke,
                shadow,
                use_,
            })
        }

        let paragraph_content = {
            let mut v = Vec::<ParagraphContent>::new();
            let content_json: &Vec<Value> = paragraph_json.get("contents")?.as_array()?;
            for item in content_json.iter() {
                let obj = item.as_object();
                if obj.is_none() { continue; }
                let obj = obj.unwrap();
                let line_height = {
                    let value = obj.get("lineHeight");
                    if value.is_none() {
                        1.2f32
                    } else {
                        value.unwrap().as_f64().unwrap_or(1.2f64) as f32
                    }
                };
                let paragraph_indentation = {
                    let value = obj.get("paragraphIndentation");
                    if value.is_none() {
                        0f32
                    } else {
                        value.unwrap().as_f64().unwrap_or(0f64) as f32
                    }
                };
                let blocks = {
                    let mut block_vec = Vec::<TextBlock>::new();
                    let block_vec_json = obj.get("blocks")?.as_array()?;
                    for item in block_vec_json.iter() {
                        let obj = item.as_object();
                        if obj.is_none() { continue; }
                        let obj = obj.unwrap();
                        let text = {
                            let value = obj.get("text");
                            if value.is_none() {
                                ""
                            } else {
                                value.unwrap().as_str().unwrap_or("")
                            }
                        }.to_string();
                        let font_family = {
                            let value = obj.get("fontFamily");
                            if value.is_none() {
                                "default"
                            } else {
                                value.unwrap().as_str().unwrap_or("default")
                            }
                        }.to_string();
                        let fill = {
                            let value = obj.get("fill");
                            if value.is_none() {
                                "#000000"
                            } else {
                                value.unwrap().as_str().unwrap_or("#000000")
                            }
                        }.to_string();
                        let stroke = {
                            let value = obj.get("stroke");
                            if value.is_none() {
                                "#000000"
                            } else {
                                value.unwrap().as_str().unwrap_or("#000000")
                            }
                        }.to_string();
                        let decoration = {
                            let value = obj.get("decoration");
                            if value.is_none() {
                                ""
                            } else {
                                value.unwrap().as_str().unwrap_or("")
                            }
                        }.to_string();
                        let font_size = {
                            let value = obj.get("fontSize");
                            if value.is_none() {
                                16f32
                            } else {
                                value.unwrap().as_f64().unwrap_or(16f64) as f32
                            }
                        };
                        let letter_spacing = {
                            let value = obj.get("letterSpacing");
                            if value.is_none() {
                                0f32
                            } else {
                                value.unwrap().as_f64().unwrap_or(0f64) as f32
                            }
                        };
                        let stroke_width = {
                            let value = obj.get("strokeWidth");
                            if value.is_none() {
                                0f32
                            } else {
                                value.unwrap().as_f64().unwrap_or(0f64) as f32
                            }
                        };
                        let italic = {
                            let value = obj.get("italic");
                            if value.is_none() {
                                false
                            } else {
                                value.unwrap().as_bool().unwrap_or(false)
                            }
                        };
                        let block = TextBlock {
                            text,
                            font_family,
                            font_size,
                            letter_spacing,
                            fill,
                            italic,
                            stroke,
                            stroke_width,
                            decoration,
                        };
                        block_vec.push(block);
                    }
                    block_vec
                };
                let content = ParagraphContent {
                    line_height,
                    paragraph_indentation,
                    blocks,
                };
                v.push(content);
            }
            v
        };

        let paragraph = ParagraphData {
            text_align,
            resizing,
            align,
            paragraph_spacing,
            paragraph_content,
            art_text,
            writing_mode,
        };

        Some(TextData {
            width,
            height,
            paragraph,
            source: source.to_string(),
        })
    }
}

//impl From<&Vec<f32>> for TextData {
//    fn from(item: &Vec<f32>) -> Self {
//
//    }
//}