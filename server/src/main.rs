use warp::Filter;
use bytes::Bytes;
use core::data::text_data::{TextData, WritingMode};
use core::typesetting::compute_render_command;
use core::open_type_like::bbox::BBoxes;
use core::open_type_like::command::{tran_commands_stream, CommandsList};
use font::ttf::FontCache;
use font::woff::decompress_woff;
use font::check::check_type;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::time::SystemTime;
use std::thread;
use std::fs::File;
use std::io::Read;
use std::fmt::Formatter;
use server::svg_util::url::fetch_async;
use notify::{Watcher, RecursiveMode, watcher};
use serde_json::Value as JsonValue;

#[macro_use]
pub mod svg_util;
pub mod draw;

static FONT_UPDATE_DATA: &'static str = "/opt/chuangkit.font.cache/data.json";

type AF = Arc<Mutex<FontCache<Vec<u8>>>>;


struct ProcessError(String);

impl warp::reject::Reject for ProcessError {}

impl std::fmt::Debug for ProcessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[tokio::main]
async fn main() {
    let result = include_bytes!("./SourceHanSansSC-Regular.ttf") as &[u8];
    let font_cache = Arc::new(Mutex::new(FontCache::<Vec<u8>>::new()));
    font_cache.lock().unwrap().load_font_bytes("default".to_string(), Cow::Borrowed(&result).to_vec());

    let font_cache = warp::any().map(move || font_cache.clone());
    let font_update_map = Arc::new(Mutex::new(FontUpdateMap::new()));
    font_update_map.lock().unwrap().source = update_font_update_map();
    let font_update_map1 = font_update_map.clone();
    let font_update_map_in_warp = warp::any().map(move || font_update_map1.clone());

    thread::spawn(move || {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs_f32(0.1)).unwrap();
        let w = watcher.watch(FONT_UPDATE_DATA, RecursiveMode::Recursive);
        if w.is_err() { return println!("watch error: {:?}", w); }
        loop {
            match rx.recv() {
                Ok(event) => {
                    let mut font_update_map = &mut *font_update_map.lock().unwrap();
                    font_update_map.source = update_font_update_map();
                    font_update_map.update_times += 1;
                    println!("{:?}", event)
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });


    let convert_command = warp::path("convertCommand")
        .and(warp::body::bytes())
        .and(font_cache.clone())
        .and(font_update_map_in_warp.clone())
        .map(|json: Bytes, font_cache, font_update_map_in_warp| {
            let start = SystemTime::now();
            let json = String::from_utf8(json.to_vec());
            if json.is_err() { return warp::http::Response::builder().status(500).body(String::from("解析字符串失败")).unwrap(); }
            let json = json.unwrap();
            let result = cc(&json, &font_cache, &font_update_map_in_warp);
            if result.is_none() { return warp::http::Response::builder().status(500).body(String::from("解析文字数据失败")).unwrap(); }
            let (min_width, b_boxes, commands, _, _) = result.unwrap();
            let b_boxes: Vec<f32> = (&b_boxes).into();
            let commands: Vec<f32> = (&commands).into();
            let typed_array: Vec<f32> = [vec![min_width], b_boxes, commands].concat();
            let now = SystemTime::now();
            let diff = now.duration_since(start).unwrap_or(Duration::new(0, 0));
            let font_cache: &FontCache<Vec<u8>> = &*font_cache.lock().unwrap();
            let glyph_cache_count = font_cache.get_glyph_cache_count();
            let font_cache_count = font_cache.get_font_cache_count();
            println!("convertCommand: {:?}, graph_cache_count: {}, font_cache_count: {}", diff, glyph_cache_count, font_cache_count);
            warp::http::Response::builder().status(200).body(format!("{:?}", typed_array)).unwrap()
        });
    let convert_svg = warp::path("convertSvg")
        .and(warp::body::bytes().and_then(|json: Bytes| async move {
            let start = SystemTime::now();
            let json = String::from_utf8(json.to_vec());
            if json.is_err() { return Err(warp::reject::custom(ProcessError("解析字符串失败".to_string()))); }
            let json = json.unwrap();
            let text_data = TextData::parse(&json);
            if text_data.is_none() { return Err(warp::reject::custom(ProcessError("解析文字数据失败".to_string()))); }
            let text_data = text_data.unwrap();
            let texture_raw = if text_data.paragraph.art_text.is_some() {
                let art_text = text_data.paragraph.art_text.unwrap();
                if art_text.texture.is_some() {
                    let texture = art_text.texture.unwrap();
                    let result = fetch_async(&texture).await;
                    if result.is_none() { return Err(warp::reject::custom(ProcessError(format!("下载艺术字纹理失败, {:?}", &texture)))); }
                    result
                } else { None }
            } else { None };
            Ok((json, texture_raw, start))
        }))
        .and(font_cache.clone())
        .and(font_update_map_in_warp.clone())
        .map(|result: (String, Option<Bytes>, SystemTime), font_cache: AF, font_update_map_in_warp| {
            let (json, texture_raw, start) = result;
            let result = cc(&json, &font_cache, &font_update_map_in_warp);
            if result.is_none() { return warp::http::Response::builder().status(500).body(String::from("解析文字数据失败")).unwrap(); }
            let (_min_width, _b_boxes, commands, text_data, (width, height)) = result.unwrap();
            let ref_size = {
                let mut size = 16f32;
                if text_data.paragraph.paragraph_content.get(0).is_some() {
                    let blocks = &text_data.paragraph.paragraph_content.get(0).unwrap().blocks;
                    if blocks.get(0).is_some() {
                        size = blocks.get(0).unwrap().font_size.clone();
                    }
                }
                size
            };
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

            let svg = if text_data.paragraph.art_text.is_some() {
                let art_text = text_data.paragraph.art_text.unwrap();
                draw::exec_art_text(&commands, width, height, ref_size, art_text, texture_raw)
            } else {
                draw::exec_text(&commands, width, height, 1.0)
            };
            let now = SystemTime::now();
            let diff = now.duration_since(start).unwrap_or(Duration::new(0, 0));
            let font_cache: &FontCache<Vec<u8>> = &*font_cache.lock().unwrap();
            let glyph_cache_count = font_cache.get_glyph_cache_count();
            let font_cache_count = font_cache.get_font_cache_count();
            println!("convertSvg: {:?}, graph_cache_count: {}, font_cache_count: {}", diff, glyph_cache_count, font_cache_count);
            warp::http::Response::builder().status(200).header("content-type", "image/svg+xml").body(svg).unwrap()
        });
    let info = warp::path("info")
        .and(font_cache.clone())
        .map(|font_cache: AF| {
            let font_cache: &FontCache<Vec<u8>> = &*font_cache.lock().unwrap();
            let glyph_cache_count = font_cache.get_glyph_cache_count();
            let font_cache_count = font_cache.get_font_cache_count();
            format!("graph_cache_count: {}, font_cache_count: {}", glyph_cache_count, font_cache_count)
        });

    let routes = warp::post().and(convert_command.or(convert_svg).or(info));

    println!("text service on 8210");
    warp::serve(routes).run(([0, 0, 0, 0], 8210)).await;
}

fn cc(json: &String, font_cache: &AF, font_update_map: &Arc<Mutex<FontUpdateMap>>) -> Option<(f32, BBoxes, CommandsList, TextData, (f32, f32))> {
    let mut font_cache = &mut *font_cache.lock().unwrap();
    let text_data = TextData::parse(&json);
    if text_data.is_none() { return None; }
    let text_data = text_data.unwrap();

    for content in text_data.paragraph.paragraph_content.iter() {
        let blocks = &content.blocks;
        for block in blocks.iter() {
            let text = block.text.clone();
            let font_family = &block.font_family;
            load_font(font_family, &mut font_cache, font_update_map);
            let mut text_chars = text.chars();
            while let Some(text) = text_chars.next() {
                font_cache.check_glyph(font_family.to_string(), text as u32);
            }
        }
    }

    let (b_boxes, result, min_width, rect) = compute_render_command(&text_data, font_cache).unwrap_or((BBoxes::new(), (HashMap::new(), Vec::new()), -1.0, (20.0, 20.0)));
    let commands = tran_commands_stream(&result);

    Some((min_width, b_boxes, commands, text_data, rect))
}

fn load_font(font_name: &String, font_cache: &mut FontCache<Vec<u8>>, font_update_map: &Arc<Mutex<FontUpdateMap>>) {
    let font_update_map = &mut *font_update_map.lock().unwrap();
    if !font_update_map.is_latest(font_name) {
        let file = File::open(format!("/opt/chuangkit.font.cache/{}", font_name));
        if file.is_err() { return println!("打开字体文件失败 {:?}", &font_name); }
        let mut read = file.unwrap();
        let mut font_buffer = vec![];
        let result = read.read_to_end(&mut font_buffer);
        if result.is_err() { return println!("读字体文件失败 {:?}", &font_name); }
        if let Some((typ, p)) = check_type(&font_buffer) {
            if "ttf".to_string() == typ.clone() {
                if p {
                    println!("解压开始 {:?}", &font_name);
                    if let Some(data1) = decompress_woff(&font_buffer) {
                        font_buffer = data1;
                        println!("解压成功 {:?}", &font_name);
                    } else {
                        println!("解压失败 {:?}", &font_name);
                    }
                }
                let result = font_cache.load_font_bytes(font_name.clone(), font_buffer);
                if result.is_none() {
                    println!("加载失败 {:?}", &font_name);
                } else {
                    println!("加载完成 {:?}", &font_name);
                }
            } else {
                println!("加载失败 {:?} {:?}", font_name, typ);
            }
        }
        font_update_map.update(font_name);
    }
}

fn update_font_update_map() -> JsonValue {
    let read = File::open(FONT_UPDATE_DATA);
    if read.is_err() { return JsonValue::Null; }
    let mut read = read.unwrap();
    let mut font_update_data = String::from("");
    let r = read.read_to_string(&mut font_update_data);
    if r.is_err() { return JsonValue::Null; }
    let result = serde_json::from_str(&font_update_data);
    if result.is_ok() {
        result.unwrap()
    } else {
        JsonValue::Null
    }
}

///  控制字体更新版本
struct FontUpdateMap {
    source: JsonValue,
    update_times: usize,
    map: HashMap<String, (usize, u32)>,
}

impl FontUpdateMap {
    fn new() -> Self {
        FontUpdateMap {
            source: JsonValue::Null,
            update_times: 0,
            map: HashMap::<String, (usize, u32)>::new(),
        }
    }
    fn is_latest(&self, font_family: &str) -> bool {
        let value = self.map.get(font_family);
        if value.is_none() { return false; }
        let (times, tag) = value.unwrap();
        if *times == self.update_times { return true; }
        let tag_s = self.source.get(font_family).and_then(|v| v.as_i64()).and_then(|v| Some(v as u32));
        if tag_s.is_none() { return false; }
        let tag_s = tag_s.unwrap();
        if tag_s == *tag {
            return true;
        }
        false
    }

    fn update(&mut self, font_family: &str) {
        if self.is_latest(font_family) { return; }
        let tag_s = self.source.get(font_family).and_then(|v| v.as_i64()).and_then(|v| Some(v as u32));
        if tag_s.is_some() {
            let tag_s = tag_s.unwrap();
            self.map.insert(font_family.to_string(), (self.update_times, tag_s));
        }
    }
}


