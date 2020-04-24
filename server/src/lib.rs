#[macro_use]
pub mod svg_util;
pub mod draw;

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::fs::File;
    use std::io::{Read, Write};
    use font::ttf::FontCache;
    use font::woff::decompress_woff;
    use core::data::text_data::TextData;
    use core::typesetting::compute_render_command;
    use core::open_type_like::command::tran_commands_stream;

    #[test]
    fn it_works() {
        let file = include_bytes!("/Users/suti/start/text-render-rust/font/src/c_764") as &[u8];
        let result = decompress_woff(file).unwrap();
        let mut font_cache = FontCache::<Vec<u8>>::new();
        font_cache.load_font_bytes("default".to_string(), Cow::Borrowed(&result).to_vec());
        let mut read = File::open("/Users/suti/start/text-render-rust/skia/src/t.json").unwrap();
        let mut test_text_json = String::from("");
        read.read_to_string(&mut test_text_json);
        let test_text_data = TextData::parse(&test_text_json).unwrap();

        for content in test_text_data.paragraph.paragraph_content.iter() {
            let blocks = &content.blocks;
            for block in blocks.iter() {
                let text = block.text.clone();
                let font_family = &block.font_family;
                let mut text_chars = text.chars();
                while let Some(text) = text_chars.next() {
                    font_cache.check_glyph(font_family.to_string(), text as u32);
                }
            }
        }
        let ref_size = {
            let mut size = 16f32;
            if test_text_data.paragraph.paragraph_content.get(0).is_some() {
                let blocks = &test_text_data.paragraph.paragraph_content.get(0).unwrap().blocks;
                if blocks.get(0).is_some() {
                    size = blocks.get(0).unwrap().font_size.clone();
                }
            }
            size
        };

        let (b_box, result, _) = compute_render_command(&test_text_data, &font_cache).unwrap();
        let b_box = b_box.get_total_box();
        let mut width = b_box.get_width().ceil() as f32;
        let height = b_box.get_height().ceil() as f32;
        if test_text_data.width > width {
            width = test_text_data.width
        }
        let commands = &tran_commands_stream(&result);
        let svg = if test_text_data.paragraph.art_text.is_some() {
            super::draw::exec_art_text(commands, width, height, ref_size, test_text_data.paragraph.art_text.unwrap(), None)
        } else {
            super::draw::exec_text(commands, width, height, 1.0)
        };
        let mut file = File::create("test.svg").unwrap();
        file.write_all(&svg.as_bytes()).unwrap();
    }
}