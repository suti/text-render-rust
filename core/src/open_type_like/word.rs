use super::super::data::text_data::{TextBlock, TextBlockDetail};

use regex::Regex;


#[derive(Debug, Clone)]
pub struct Word<'a> {
    pub letters: Vec<(TextBlock, TextBlockDetail<'a>)>,
}

impl<'a> Word<'a> {
    pub fn pick_words(letters: Vec<(TextBlock, TextBlockDetail<'a>)>) -> Vec<Word<'a>> {
        let mut blocks = Vec::<(TextBlock, TextBlockDetail)>::new();
        let mut words = Vec::<Word>::new();
        let mut point = 0usize;
//        let mut time = std::time::SystemTime::now();
        let len = letters.len();

        let regex0 = Regex::new(r#"['!)}\],./?|%“”‘’"]"#).unwrap();
        let regex1 = Regex::new(r#"[A-Za-z0-9~`@#$^&*(_+={\[\\<>￥（【《]"#).unwrap();
        let regex3 = Regex::new(r" ").unwrap();

//        if let Ok(d) = std::time::SystemTime::now().duration_since(time) {
//            println!("创建正则 {:?}", d);
//            time = std::time::SystemTime::now();
//        }

        for letter in letters {
//            let mut time = std::time::SystemTime::now();
            let (b, d) = letter;
            let text = &b.text;

            if point == len - 1 {
                blocks.push((b, d));
                words.push(Word { letters: blocks.to_vec() })
            } else if regex3.is_match(text) {
                if blocks.len() > 0 {
                    words.push(Word { letters: blocks.to_vec() });
                    blocks = vec![];
                }
                words.push(Word { letters: vec![(b, d)] });
            } else if regex0.is_match(text) {
                blocks.push((b, d));
            } else if !regex1.is_match(text) {
                if blocks.len() > 0 {
                    words.push(Word { letters: blocks.to_vec() });
                    blocks = vec![];
                }
                words.push(Word { letters: vec![(b, d)] });
            } else {
                blocks.push((b, d));
            }
            point += 1;
//            if let Ok(d) = std::time::SystemTime::now().duration_since(time) {
//                println!("处理数据 {:?}， {}", d, point);
//                time = std::time::SystemTime::now();
//            }
        }
//        if let Ok(d) = std::time::SystemTime::now().duration_since(time) {
//            println!("pp {:?}", d);
//            time = std::time::SystemTime::now();
//        }
        words
    }

    pub fn get_advance_width(&self) -> f32 {
        let mut width = 0f32;
        for (b, d) in self.letters.iter() {
            width += d.glyph.get_advance_width(b.font_size as f32) + b.font_size as f32 * b.letter_spacing as f32
        }
        width
    }

    pub fn is_blank(&self) -> bool {
        if self.letters.len() == 1 {
            if let Some(v) = self.letters.get(0) {
                let regex = Regex::new(r" ").unwrap();
                regex.is_match(&v.0.text)
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl<'a> std::ops::Deref for Word<'a> {
    type Target = Vec<(TextBlock, TextBlockDetail<'a>)>;

    fn deref(&self) -> &Self::Target {
        &self.letters
    }
}