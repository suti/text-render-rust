use super::super::data::font_data;

use super::transform::Transform;

macro_rules! join_str {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = String::from("");
            $(
                temp_vec.push_str(&$x.to_string());
            )*
            temp_vec
        }
    };
}

#[derive(Clone, Copy, Debug)]
pub enum PathSegment {
    MoveTo {
        x: f32,
        y: f32,
    },
    LineTo {
        x: f32,
        y: f32,
    },
    CurveTo {
        x: f32,
        y: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    ClosePath,
}

impl From<&PathSegment> for String {
    fn from(path: &PathSegment) -> Self {
        let result = match path {
            PathSegment::MoveTo { x, y } => {
                join_str!["{\"type\": \"move\", \"x\": ", (*x) as i32,", \"y\": ", (*y) as i32, "}"]
            }
            PathSegment::LineTo { x, y } => {
                join_str!["{\"type\": \"line\", \"x\": ", (*x) as i32,", \"y\": ", (*y) as i32, "}"]
            }
            PathSegment::CurveTo { x, y, x1, y1, x2, y2 } => {
                join_str!["{\"type\": \"curve\", \"x\": ", (*x) as i32,", \"y\": ", (*y) as i32,", \"x1\": ", (*x1) as i32,", \"y1\": ", (*y1) as i32,", \"x2\": ", (*x2) as i32,", \"y2\": ", (*y2) as i32, "}"]
            }
            PathSegment::ClosePath => {
                join_str!["{\"type\": \"close\"}"]
            }
        };
        result
    }
}

impl From<&PathSegment> for Vec<f32> {
    fn from(item: &PathSegment) -> Self {
        match item {
            PathSegment::MoveTo { x, y } => {
                vec![0f32, *x, *y]
            }
            PathSegment::LineTo { x, y } => {
                vec![1f32, *x, *y]
            }
            PathSegment::CurveTo { x, y, x1, y1, x2, y2 } => {
                vec![2f32, *x, *y, *x1, *y1, *x2, *y2]
            }
            PathSegment::ClosePath => {
                vec![3f32]
            }
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct PathData(pub Vec<PathSegment>);

impl From<&PathData> for String {
    fn from(p: &PathData) -> Self {
        let mut result = String::from("[");
        for segment in p.iter() {
            result.push_str(&String::from(segment));
            result.push_str(",");
        }
        if p.len() > 0 { result.pop(); }
        result.push_str("]");
        result
    }
}

impl From<&PathData> for Vec<f32> {
    fn from(item: &PathData) -> Self {
        let mut result = Vec::<f32>::new();
        result.push(item.len() as f32);
        for segment in item.iter() {
            let vec: Vec<f32> = segment.into();
            for item in vec.iter() {
                result.push(*item)
            }
        }
        result
    }
}

impl PathData {
    pub fn new() -> Self {
        PathData(Vec::<PathSegment>::new())
    }

    #[inline]
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.push(PathSegment::MoveTo { x, y });
    }

    #[inline]
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.push(PathSegment::LineTo { x, y });
    }

    #[inline]
    pub fn curve_to(&mut self, x: f32, y: f32, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.push(PathSegment::CurveTo { x, y, x1, y1, x2, y2 });
    }

    #[inline]
    pub fn quad_to(&mut self, x: f32, y: f32, x1: f32, y1: f32) {
        let (prev_x, prev_y) = self.last_pos();
        self.push(quad_to_curve(prev_x, prev_y, x, y, x1, y1));
    }

    #[inline]
    pub fn close(&mut self) {
        self.push(PathSegment::ClosePath);
    }

    #[inline]
    fn last_pos(&self) -> (f32, f32) {
        let seg = self.last().expect("path must not be empty").clone();
        match seg {
            PathSegment::MoveTo { x, y }
            | PathSegment::LineTo { x, y }
            | PathSegment::CurveTo { x, y, .. } => {
                (x, y)
            }
            PathSegment::ClosePath => {
                panic!("the previous segment must be M/L/C")
            }
        }
    }

    #[inline]
    pub fn transform(&mut self, ts: Transform) {
        transform_path(self, ts);
    }

    #[inline]
    pub fn transform_from(&mut self, offset: usize, ts: Transform) {
        transform_path(&mut self[offset..], ts);
    }
}

impl std::ops::Deref for PathData {
    type Target = Vec<PathSegment>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PathData {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[inline]
fn quad_to_curve(px: f32, py: f32, x: f32, y: f32, x1: f32, y1: f32) -> PathSegment {
    #[inline]
    fn calc(n1: f32, n2: f32) -> f32 {
        (n1 + n2 * 2.0) / 3.0
    }

    PathSegment::CurveTo {
        x,
        y,
        x1: calc(px, x1),
        y1: calc(py, y1),
        x2: calc(x, x1),
        y2: calc(y, y1),
    }
}

fn transform_path(segments: &mut [PathSegment], ts: Transform) {
    for seg in segments {
        match seg {
            PathSegment::MoveTo { ref mut x, ref mut y } => {
                ts.apply_to(x, y);
            }
            PathSegment::LineTo { ref mut x, ref mut y } => {
                ts.apply_to(x, y);
            }
            PathSegment::CurveTo { ref mut x1, ref mut y1, ref mut x2, ref mut y2, ref mut x, ref mut y } => {
                ts.apply_to(x1, y1);
                ts.apply_to(x2, y2);
                ts.apply_to(x, y);
            }
            PathSegment::ClosePath => {}
        }
    }
}