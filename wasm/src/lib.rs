extern crate js_sys;
extern crate web_sys;
extern crate wasm_bindgen;
extern crate wasm_bindgen_test;
extern crate wasm_bindgen_futures;

pub mod chars;

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

use std::collections::HashMap;

use crate::js_sys::Promise;
use crate::wasm_bindgen::JsValue;
use crate::wasm_bindgen::prelude::wasm_bindgen;
use crate::wasm_bindgen_futures::{future_to_promise, JsFuture, spawn_local};

use core::open_type_like::command::{tran_commands_stream, CommandSegment};
use core::open_type_like::bbox::BBox;
use core::typesetting::{compute_render_command, MergedFont};
use core::data::font_data::FontData;
use core::data::text_data::TextData;
use wasm_bindgen_test::__rt::js_console_log;
use core::open_type_like::glyph::Glyph;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "getGlyphData")]
    fn get_glyph_data(ff: JsValue, char: JsValue) -> Box<[JsValue]>;

    #[wasm_bindgen(js_name = "getDate")]
    fn performance() -> f64;
}


#[wasm_bindgen]
pub struct Executor {
    //    font_data: HashMap<String, Box<FontData>>,
//    default_font: Option<Box<FontData>>,
    glyph_indexes: HashMap<(String, u16), usize>,
    glyph_caches: Vec<Box<Glyph>>,
}

fn now() -> f64 {
    performance()
}


#[wasm_bindgen]
impl Executor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Executor {
            glyph_indexes: HashMap::new(),
            glyph_caches: Vec::new(),
        }
    }


    #[wasm_bindgen(js_name = exec)]
    pub fn exec(&mut self, text_data: &str) -> Box<[f32]> {
//        let start = now();
//        js_console_log(&format!("start {:?}", start));
        let text_data = text_data.to_string();
        let text_data = &TextData::parse(&text_data).expect(&format!("文字解析失败{}", text_data));

        for content in text_data.paragraph.paragraph_content.iter() {
            let blocks = &content.blocks;
            for block in blocks.iter() {
                let text = block.text.clone();
                let font_family = &block.font_family;
                let mut text_chars = text.chars();
                while let Some(text) = text_chars.next() {
                    self.check_glyph(font_family.to_string(), text as u16);
                }
            }
        }

        let (b_boxes, result) = compute_render_command(text_data, self).unwrap();
//        js_console_log(&format!("排版耗时 {:?}", now() - start));
//        let start = now();
        let b_box = b_boxes.get_total_box();
        let mut width = b_box.get_width().ceil() as f32;
        let height = b_box.get_height().ceil() as f32;
        if text_data.width > width {
            width = text_data.width;
        }
        let result = tran_commands_stream(&result);

        let b_boxes: Vec<f32> = (&b_boxes).into();
        let commands: Vec<f32> = (&result).into();
        let typed_array: Vec<f32> = [b_boxes, commands].concat();
        let boxed_array = typed_array.into_boxed_slice();

//        js_console_log(&format!("拼接指令耗时 {:?}", now() - start));
        boxed_array
    }

    #[wasm_bindgen(js_name = loadFont)]
    pub fn load_font(&mut self, font_name: String) {
        for c in chars::get_char_code().iter() {
            self.check_glyph(font_name.clone(), *c)
        }
    }

    fn check_glyph(&mut self, font_name: String, c: u16) {
        let result = self.glyph_indexes.get(&(font_name.clone(), c.clone()));
        if result.is_none() {
            let result = get_glyph_data(JsValue::from(&font_name), JsValue::from(c));
            let advance_width = result[0].as_f64().unwrap() as i32;
            let units_per_em = result[1].as_f64().unwrap() as i32;
            let ascender = result[2].as_f64().unwrap() as i32;
            let descender = result[3].as_f64().unwrap() as i32;
            let path_str = result[4].as_string().unwrap();
            let glyph = Glyph::parse(&path_str, advance_width, units_per_em, ascender, descender).unwrap();
            self.glyph_caches.push(Box::new(glyph));
            self.glyph_indexes.insert((font_name.clone(), c.clone()), self.glyph_caches.len() - 1);
        }
    }
}

impl MergedFont for Executor {
    fn char_to_glyph(&self, font_name: String, c: char) -> &Box<Glyph> {
        let c = c as u16;
        let result = self.glyph_indexes.get(&(font_name.clone(), c));
        &self.glyph_caches.get(*result.unwrap()).unwrap()
    }
}
