extern crate js_sys;
extern crate web_sys;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;

pub mod chars;

use std::collections::HashMap;
use std::borrow::Cow;

use crate::js_sys::Promise;
use crate::wasm_bindgen::JsValue;
use crate::wasm_bindgen::prelude::wasm_bindgen;
use crate::wasm_bindgen_futures::{future_to_promise, JsFuture, spawn_local};

use core::open_type_like::command::{tran_commands_stream, CommandSegment};
use core::open_type_like::bbox::{BBox, BBoxes};
use core::typesetting::{compute_render_command, MergedFont};
use core::data::font_data::FontData;
use core::data::text_data::{TextData, WritingMode};
use core::open_type_like::glyph::Glyph;
use font::ttf::FontCache;
use font::check::check_type;
use font::woff::decompress_woff;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "getDate")]
    fn performance() -> f64;
    #[wasm_bindgen(js_namespace = console, js_name = info)]
    fn js_console_info(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn js_console_error(s: &str);
}


#[wasm_bindgen]
pub struct Executor(FontCache<Vec<u8>>);

fn now() -> f64 {
    performance()
}


#[wasm_bindgen]
impl Executor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Executor(FontCache::new())
    }

    #[wasm_bindgen(js_name = exec)]
    pub fn exec(&mut self, text_data: &str) -> Box<[f32]> {
//        let start = now();
        let text_data = text_data.to_string();
        let text_data = &TextData::parse(&text_data).expect(&format!("文字解析失败{}", text_data));

        for content in text_data.paragraph.paragraph_content.iter() {
            let blocks = &content.blocks;
            for block in blocks.iter() {
                let text = block.text.clone();
                let font_family = &block.font_family;
                let mut text_chars = text.chars();
                while let Some(text) = text_chars.next() {
                    self.check_glyph(font_family.to_string(), text as u32);
                }
            }
        }

        let (b_boxes, result, min_width, (width, height)) = compute_render_command(text_data, self).unwrap_or((BBoxes::new(), (HashMap::new(), Vec::new()), -1.0, (20.0,20.0)));
        let mut width = width;
        let mut height = height;

        match &text_data.paragraph.writing_mode {
            &WritingMode::HorizontalTB => if text_data.width > width {
                width = text_data.width
            },
            _ => if text_data.height > height {
                height = text_data.height
            },
        }
        let result = tran_commands_stream(&result);

        let b_boxes: Vec<f32> = (&b_boxes).into();
        let commands: Vec<f32> = (&result).into();
        let typed_array: Vec<f32> = [vec![-5.0, min_width, width, height], b_boxes, commands].concat();
        let boxed_array = typed_array.into_boxed_slice();

//        js_console_log(&format!("缓存数量 {:?} 耗时 {:?}", self.get_cache_count(), now() - start));
        boxed_array
    }

    #[wasm_bindgen(js_name = loadFontBuffer)]
    pub fn load_font_buffer(&mut self, font_name: String, data: &[u8]) {
        let data = Cow::Borrowed(data);
        let mut data = data.to_vec();
        if let Some((typ, p)) = check_type(&data) {
            if "ttf".to_string() == typ.clone() {
                if p {
                    let start = now();
                    js_console_info(&format!("解压开始 {:?}", &font_name));
                    if let Some(data1) = decompress_woff(&data) {
                        data = data1;
                    } else {
                        js_console_info(&format!("解压成功 {:?} 耗时 {:?}", &font_name, now() - start));
                    }
                }
                let result = self.load_font_bytes(font_name.clone(), data);
                if result.is_none() {
                    js_console_error(&format!("加载失败 {:?} 不标准的ttf", &font_name));
                }
            } else {
                js_console_error(&format!("加载失败 {:?} {:?}", font_name, typ));
            }
        }
    }
}

impl std::ops::Deref for Executor {
    type Target = FontCache<Vec<u8>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Executor {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MergedFont for Executor {
    fn char_to_glyph<'a>(&'a self, font_name: String, char: char) -> &'a Box<Glyph> {
        self.0.char_to_glyph(font_name, char)
    }
}
