use json::JsonValue::{Object, Array};
use json::JsonValue;


#[derive(Debug, Clone)]
pub struct Shadow {
    pub blur: f32,
    pub offset: (f32, f32),
    pub color: String,
}

#[derive(Debug, Clone)]
pub enum ShadowOption {
    Some(Shadow),
    None,
}

#[derive(Debug, Clone)]
pub struct ParagraphContent {
    pub line_height: f32,
    pub paragraph_indentation: f32,
    pub blocks: Vec<TextBlock>,
}

#[derive(Debug, Clone)]
pub struct ParagraphData {
    pub text_align: String,
    pub resizing: String,
    pub align: String,
    pub paragraph_spacing: f32,
    pub paragraph_content: Vec<ParagraphContent>,
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

#[derive(Debug, Clone)]
pub struct TextData {
    pub width: f32,
    pub height: f32,
    pub paragraph: ParagraphData,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct TextBlockDetail<'a> {
    pub glyph: &'a super::super::open_type_like::glyph::Glyph,
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

impl TextData {
    pub fn parse(source: &str) -> Option<TextData> {
        let json_d = json::parse(source).unwrap();

        let result = get_object(&json_d)?;

        let width = result.get("width")?.as_f32()?;
        let height = result.get("height")?.as_f32()?;

        let paragraph_data_json = get_object(result.get("paragraph")?)?;

        let text_align = paragraph_data_json.get("textAlign")?.as_str()?.to_string();
        let resizing = paragraph_data_json.get("resizing")?.as_str()?.to_string();
        let align = paragraph_data_json.get("align")?.as_str()?.to_string();
        let paragraph_spacing = paragraph_data_json.get("paragraphSpacing")?.as_f32()?;

        let mut paragraph_content = Vec::<ParagraphContent>::new();
        let paragraph_content_json_arr = get_array(paragraph_data_json.get("contents")?)?;

        for item in paragraph_content_json_arr {
            let item_inner = get_object(item)?;
            let line_height = item_inner.get("lineHeight")?.as_f32()?;
            let paragraph_indentation = item_inner.get("paragraphIndentation")?.as_f32()?;

            let text_blocks = get_array(item_inner.get("blocks")?)?;
            let mut blocks = Vec::<TextBlock>::new();

            for block in text_blocks {
                let text_block = get_object(block)?;
                let text = text_block.get("text")?.as_str()?.to_string();
                let font_family = text_block.get("fontFamily")?.as_str()?.to_string();
                let font_size = text_block.get("fontSize")?.as_f32()?;
                let letter_spacing = text_block.get("letterSpacing")?.as_f32()?;
                let fill = text_block.get("fill")?.as_str()?.to_string();
                let italic = text_block.get("italic")?.as_bool()?;
                let stroke = text_block.get("stroke")?.as_str()?.to_string();
                let stroke_width = text_block.get("strokeWidth")?.as_f32()?;
                let decoration = text_block.get("decoration")?.as_str()?.to_string();
                blocks.push(TextBlock {
                    text,
                    font_family,
                    font_size,
                    letter_spacing,
                    fill,
                    italic,
                    stroke,
                    stroke_width,
                    decoration,
                })
            }
            paragraph_content.push(ParagraphContent {
                line_height,
                paragraph_indentation,
                blocks,
            });
        }
        let paragraph = ParagraphData {
            text_align,
            resizing,
            align,
            paragraph_spacing,
            paragraph_content,
        };

        let text_data = TextData {
            width,
            height,
            paragraph,
            source: source.to_string(),
        };

        Some(text_data)
    }
}

//impl From<&Vec<f32>> for TextData {
//    fn from(item: &Vec<f32>) -> Self {
//
//    }
//}