use super::data::text_data::{TextData, TextBlock, ParagraphContent, ParagraphData, TextBlockDetail};
use super::data::font_data::FontData;
use super::open_type_like::command::{CommandSegment, CommandList};
use super::open_type_like::glyph::Glyph;
use super::open_type_like::font::Font;
use super::open_type_like::path::PathData;
use super::open_type_like::bbox::{BBox, BBoxes};
use super::open_type_like::word::Word;

use std::collections::HashMap;
use regex::Regex;

pub trait MergedFont {
    fn char_to_glyph<'a>(&'a self, font_name: String, char: char) -> &'a Box<Glyph>;
}


//pub struct Typesetting<'a> {
//    pub text_data: TextData,
//    font_data_list: HashMap<String, &'a Box<FontData>>,
//}
//
//impl<'a> Typesetting<'a> {
////    pub fn new(text_data_source: &str, font_data_source: HashMap<String, String>) -> Self {
////        let mut font_data_parsed: HashMap<String, &'a Box<FontData>> = HashMap::new();
////        for (ff, font_data) in font_data_source.clone().iter() {
////            let font: &'static Box<FontData> = &Box::new(FontData::parse(font_data).unwrap());
////            font_data_parsed.insert(ff.clone(), &font);
////            std::mem::forget(font);
////        }
////        Typesetting {
////            text_data: TextData::parse(text_data_source).unwrap(),
////            font_data_list: font_data_parsed,
////        }
////    }
//
//    pub fn from(text_data: TextData, font_data_list: HashMap<String, &'a Box<FontData>>) -> Self {
//        Typesetting {
//            text_data,
//            font_data_list,
//        }
//    }
//
//    fn parse_font_to_glyph(&self) -> Option<Box<HashMap<(String, String), Glyph>>> {
//        let text_data = self.text_data.clone();
//        let default_font = &self.font_data_list.get("default")?;
//        let mut font_glyph = HashMap::<(String, String), Glyph>::new();
//        for pc in text_data.paragraph.paragraph_content.iter() {
//            for b in pc.blocks.iter() {
//                let item = self.font_data_list.get(&b.font_family)?;
//                let g_list = item.char_to_glyph(&b.text, Some(default_font))?;
//                for (c, g) in g_list {
//                    font_glyph.insert((b.font_family.clone(), c.to_string()), g);
//                }
//            }
//        }
//        Some(Box::new(font_glyph))
//    }
//    /// Option<Vec<Command>>
//}


pub fn compute_render_command(text_data: &TextData, font: &impl MergedFont) -> Option<(BBoxes, (HashMap<(String, u32), PathData>, Vec<CommandSegment>))> {
    let mut width = text_data.width;
    let mut height = text_data.height;
    /// get all text glyph width font_family
    let mut font_glyph = HashMap::<(String, String), &Box<Glyph>>::new();
//    let get_glyph = |font_family, c| {
//        font.char_to_glyph(font_family, c)
//    };
    for pc in text_data.paragraph.paragraph_content.iter() {
        for b in pc.blocks.iter() {
            let text = (&b.text).clone();
            let mut chars = text.chars();

            while let Some(c) = chars.next() {
                let font_family = b.font_family.clone();
                let g = font.char_to_glyph(font_family, c);
                font_glyph.insert((b.font_family.clone(), c.to_string()), g);
            }
        }
    }
//    let font_glyph: &Box<HashMap<(String, String), Glyph>> = &font_glyph;
    let glyph_none = Box::new(Glyph::get_none());
    let get_glyph = |ff: String, text: String| {
        let result = font_glyph.get(&(ff, text));
        if result.is_none() {
            Some(&glyph_none)
        } else {
            Some(*result.unwrap())
        }
    };

    /// make TextBlock & TextBlockDetail together
    let mut mix_text_data = Vec::<Vec<(TextBlock, TextBlockDetail)>>::new();
    let ParagraphData {
        paragraph_content,
        paragraph_spacing,
        align,
        resizing,
        text_align,
    } = &text_data.paragraph;
    for content in paragraph_content.iter() {
        mix_text_data.push(Vec::<(TextBlock, TextBlockDetail)>::new());
        let ParagraphContent {
            paragraph_indentation: mut paragraph_indentation,
            line_height: mut line_height,
            blocks
        } = content;

        for block in blocks.iter() {
            let TextBlock {
                font_family,
                text,
                font_size,
                letter_spacing,
                fill,
                italic,
                stroke,
                stroke_width,
                decoration
            } = block;
            let mut text_chars = text.chars();
            while let Some(text) = text_chars.next() {
                let glyph = get_glyph(font_family.clone(), text.to_string())?;
                let text_block_detail = TextBlockDetail {
                    glyph,
                    line_height,
                    paragraph_indentation,
                    align: align.clone(),
                    resizing: resizing.clone(),
                    text_align: text_align.clone(),
                    paragraph_spacing: *paragraph_spacing,
                    b_width: 0f32,
                    position: (0f32, 0f32),
                    base_line_to_top: 0f32,
                    base_line_to_bottom: 0f32,
                };
                let mut new_text_block = block.clone();
                new_text_block.text = text.to_string();
                mix_text_data.last_mut()?.push((new_text_block, text_block_detail));
                paragraph_indentation = 0.0;
            }
        }
    };

    ///
    let mut min_width = 0f32;

    for item in mix_text_data.concat().iter() {
        let (b, d) = item;
        let width = d.glyph.get_advance_width(b.font_size as f32) + b.font_size as f32 * b.letter_spacing as f32;
        if width > min_width {
            min_width = width;
        }
    }

    /// trans letter data to word data
    let mut mix_word_data = Vec::<Vec<Word>>::new();
    for mix_text_data_in_line in mix_text_data {
        let result = Word::pick_words(mix_text_data_in_line);
        mix_word_data.push(result);
    }

    if min_width > width as f32 {
        width = min_width.ceil();
    }

    let mut mix_word_data_wrapped = Vec::<Vec<Word>>::new();

    for x in &mix_word_data {
        let result = compute_auto_wrap(width, x);
        for line in result {
            mix_word_data_wrapped.push(line);
        }
    }

    std::mem::drop(mix_word_data);

    let mut l_index = 0usize;
    let mut mix_letter_data_width_position = Vec::<(TextBlock, TextBlockDetail)>::new();
    mix_word_data_wrapped.iter().fold((width as f32, height as f32, text_align.to_string(), 0f32), |p, c| {
        let (result, option) = compute_glyph_position(c, p, l_index);
        for item in result {
            mix_letter_data_width_position.push(item);
        }
        l_index += 1;
        option
    });

    std::mem::drop(mix_word_data_wrapped);

    let mut mat_data = BBoxes::new();
    mix_letter_data_width_position.iter().for_each(|letter| {
        let w = letter.1.b_width;
        let (x, y) = letter.1.position;
        let t = letter.1.base_line_to_top;
        let b = letter.1.base_line_to_bottom;
        mat_data.push(BBox::new(x.into(), (y - t).into(), (x + w).into(), (y + b).into()));
    });

    let command_list = CommandList::new(&mix_letter_data_width_position);
    let commands = command_list.get_commands();

    Some((mat_data, commands))
}

/// 计算换行
fn compute_auto_wrap<'a>(limit: f32, words: &Vec<Word<'a>>) -> Vec<Vec<Word<'a>>> {
    let mut wrapped_all = Vec::<Vec<Word>>::new();
    let wrapped_words = words.iter().map(|word| {
        let word_width = word.get_advance_width();
        if limit < word_width.ceil() as f32 {
            let mut split_words = Vec::<Word>::new();
            let mut split_letters = Vec::<(TextBlock, TextBlockDetail)>::new();
            word.iter().fold(0f32, |p, c| {
                let c_width = c.1.glyph.get_advance_width(c.0.font_size as f32) + c.0.font_size as f32 * c.0.letter_spacing as f32;
                if (p + c_width).ceil() as f32 > limit {
                    if split_letters.len() > 0 {
                        split_words.push(Word { letters: split_letters.splice((..), vec![]).collect() });
                    }
                    split_letters.push((c.0.clone(), c.1.clone()));
                    return c_width;
                }
                split_letters.push((c.0.clone(), c.1.clone()));
                p + c_width
            });
            if split_letters.len() > 0 {
                split_words.push(Word { letters: split_letters.splice((..), vec![]).collect() })
            }
            return split_words;
        }
        vec![word.clone()]
    });

    let mut flat_wrapped_words = Vec::<Word>::new();

    for item1 in wrapped_words {
        for item2 in item1 {
            flat_wrapped_words.push(item2)
        }
    }

    flat_wrapped_words.iter().fold(0f32, |p, c| {
        let word_width = c.get_advance_width();
        if (p + word_width).ceil() > limit as f32 {
            wrapped_all.push(vec![c.clone()]);
            return word_width;
        } else {
            if let Some(last) = wrapped_all.last_mut() {
                last.push(c.clone());
            } else {
                wrapped_all.push(vec![c.clone()]);
            }
            return p + word_width;
        }
    });
    wrapped_all
}

enum JustifyText {
    Word(f32),
    Space(f32),
    None,
}

#[inline]
fn get_max_item<'a>(line_data: &'a Vec<Word<'a>>) -> &'a (TextBlock, TextBlockDetail<'a>) {
    let mut target_point = &line_data[0][0];
    let mut font_size = target_point.0.font_size;
    for word in line_data.iter() {
        for text in word.iter() {
            if text.0.font_size > font_size {
                font_size = text.0.font_size;
                target_point = text;
            }
        }
    }
    target_point
}

/// 计算每个字形的位置
fn compute_glyph_position<'a>(line_data: &Vec<Word<'a>>, option: (f32, f32, String, f32), index: usize) -> (Vec<(TextBlock, TextBlockDetail<'a>)>, (f32, f32, String, f32)) {
    let (width, mut height, text_align, mut offset) = option;
    let mut flat_data = Vec::<(TextBlock, TextBlockDetail)>::new();
    if line_data.len() == 0 { return (flat_data, (width, height, text_align, offset)); }
    let mut font_size = 0f32;
    let mut max_letter: &(TextBlock, TextBlockDetail) = get_max_item(line_data);
    for word in line_data.iter() {
        for letter in word.letters.iter() {
            if letter.0.font_size > font_size {
                font_size = letter.0.font_size;
                max_letter = letter;
            }
        }
    }

    let max_font_size = max_letter.0.font_size;
    let ascender = max_letter.1.glyph.ascender;
    let descender = if max_letter.1.glyph.descender < 0 { max_letter.1.glyph.descender } else { -max_letter.1.glyph.descender };
    let line_height = max_letter.1.line_height;
    let line_height_padding = (line_height - 1f32) * max_font_size as f32 / 2f32;
    let base_line_to_top = max_font_size as f32 * (ascender as f32 / (ascender as f32 - descender as f32)) + line_height_padding as f32;
    let base_line_to_bottom = max_font_size as f32 * (-descender as f32 / (ascender as f32 - descender as f32)) + line_height_padding as f32;

    let mut start_position = (0f32, base_line_to_top + offset + if index == 0usize { 0f32 } else { base_line_to_bottom });
    let line_width = {
        let mut width = 0f32;
        for item in line_data {
            width += item.get_advance_width();
        }
        width
    };
    let diff_width = width - line_width;
    let mut padding_left = 0f32;
    let mut text_align_result = JustifyText::None;

    match text_align.as_ref() {
        "right" => { padding_left = diff_width }
        "center" => { padding_left = diff_width / 2f32 }
        "justify" => {
            let space_list = line_data.iter().filter(|word| word.is_blank());
            let space_test_value = space_list.fold(0f32, |p, c| { p + c[0].0.font_size as f32 * 0.2f32 });
            let mut pin = 0usize;
            let word_test_value = line_data.iter().fold(0f32, |p, c| {
                p + {
                    let mut v = 0f32;
                    if line_data.len() != pin + 1 {
                        v = c.last().unwrap().0.font_size as f32 * 0.2f32;
                    }
                    pin += 1;
                    v
                }
            });
            if space_test_value > diff_width {
                let font_size_total = space_test_value * 5f32;
                text_align_result = JustifyText::Space(diff_width / font_size_total);
            } else if word_test_value > diff_width {
                let font_size_total = word_test_value * 5f32;
                text_align_result = JustifyText::Word(diff_width / font_size_total);
            }
        }
        _ => {}
    };

    let mut l_index = 0usize;

    start_position.0 += padding_left;

    line_data.iter().for_each(|word| {
        let mut w_index = 0usize;
        word.iter().for_each(|letter| {
            let font_size = letter.0.font_size;
            let letter_spacing = letter.0.letter_spacing;
            let advance_width = letter.1.glyph.get_advance_width(font_size as f32);
            let paragraph_indentation = letter.1.paragraph_indentation;
            let mut b_width = advance_width + font_size as f32 * letter_spacing as f32;

            match text_align_result {
                JustifyText::Word(v) => {
                    if l_index != line_data.len() - 1 && w_index == word.len() - 1 {
                        b_width = advance_width + v * font_size as f32;
                    }
                }
                JustifyText::Space(v) => {
                    if word.is_blank() {
                        b_width = advance_width + v * font_size as f32;
                    }
                }
                JustifyText::None => {}
            };
            start_position.0 += paragraph_indentation as f32;
            let mut text_block = letter.0.clone();
            let mut text_block_detail = letter.1.clone();
            text_block_detail.b_width = b_width.into();
            text_block_detail.position = (start_position.0.into(), start_position.1.into());
            text_block_detail.base_line_to_top = base_line_to_top;
            text_block_detail.base_line_to_bottom = base_line_to_bottom;
            flat_data.push((text_block, text_block_detail));
            start_position.0 += b_width as f32;
            w_index += 1;
        });
        l_index += 1;
    });
    offset += base_line_to_top + if index == 0 { 0f32 } else { base_line_to_bottom };
    (flat_data, (width, height, text_align, offset))
}