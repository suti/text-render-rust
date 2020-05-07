use super::super::data::text_data::{TextBlock, TextBlockDetail};
use regex::Regex;

lazy_static! {
    static ref REGEX0: Regex = Regex::new(r#"['!)}\],./?|%“”‘’"]"#).unwrap();
    static ref REGEX1: Regex = Regex::new(r#"[A-Za-z0-9~`@#$^&*(_+={\[\\<>￥（【《]"#).unwrap();
    static ref REGEX3: Regex = Regex::new(r" ").unwrap();
}

#[derive(Debug, Clone)]
pub struct Word<'a> {
    pub letters: Vec<(TextBlock, TextBlockDetail<'a>)>,
}

impl<'a> Word<'a> {
    pub fn pick_words(letters: Vec<(TextBlock, TextBlockDetail<'a>)>) -> Vec<Word<'a>> {
        let mut blocks = Vec::<(TextBlock, TextBlockDetail)>::new();
        let mut words = Vec::<Word>::new();
        let mut point = 0usize;
        let len = letters.len();

        for letter in letters {
            let (b, d) = letter;
            let text = &b.text;

            if point == len - 1 {
                blocks.push((b, d));
                words.push(Word { letters: blocks.to_vec() })
            } else if REGEX3.is_match(text) {
                if blocks.len() > 0 {
                    words.push(Word { letters: blocks.to_vec() });
                    blocks = vec![];
                }
                words.push(Word { letters: vec![(b, d)] });
            } else if REGEX0.is_match(text) {
                blocks.push((b, d));
            } else if !REGEX1.is_match(text) {
                if blocks.len() > 0 {
                    words.push(Word { letters: blocks.to_vec() });
                    blocks = vec![];
                }
                words.push(Word { letters: vec![(b, d)] });
            } else {
                blocks.push((b, d));
            }
            point += 1;
        }
        words
    }

    pub fn get_spacing(&self) -> f32 {
        let mut width = 0f32;
        for (b, d) in self.letters.iter() {
            width += d.glyph.get_spacing(b.font_size as f32, &d.writing_mode) + b.font_size as f32 * b.letter_spacing as f32
        }
        width
    }

    pub fn is_blank(&self) -> bool {
        if self.letters.len() == 1 {
            if let Some(v) = self.letters.get(0) {
                REGEX3.is_match(&v.0.text)
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