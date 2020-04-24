use super::transform::Transform;

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
                format!("M {:?} {:?}", *x, *y)
            }
            PathSegment::LineTo { x, y } => {
                format!("L {:?} {:?}", *x, *y)
            }
            PathSegment::CurveTo { x, y, x1, y1, x2, y2 } => {
                format!("C {:?} {:?} {:?} {:?} {:?} {:?}", *x1, *y1, *x2, *y2, *x, *y)
            }
            PathSegment::ClosePath => {
                "Z".to_string()
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
        let mut result = String::from("");
        for segment in p.iter() {
            result.push_str(&String::from(segment));
            result.push_str(" ");
        }
        if p.len() > 0 { result.pop(); }
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

    pub fn get_bounding_box(&self) -> Option<BoundingBox> {
        let first = self.0.get(0);
        if first.is_none() { return None; }
        let first = first.unwrap();
        let bbox = match first {
            &PathSegment::MoveTo { ref x, ref y } => { Some(BoundingBox::new(*x, *y)) }
            &PathSegment::LineTo { ref x, ref y } => { Some(BoundingBox::new(*x, *y)) }
            &PathSegment::CurveTo { x: _, y: _, ref x1, ref y1, x2: _, y2: _ } => { Some(BoundingBox::new(*x1, *y1)) }
            _ => return None
        };
        if bbox.is_none() { return None; }
        let mut bbox = bbox.unwrap();
        let mut start_x = 0f32;
        let mut start_y = 0f32;
        let mut prev_x = 0f32;
        let mut prev_y = 0f32;
        for command in self.0.iter() {
            match command {
                &PathSegment::MoveTo { ref x, ref y } => {
                    bbox.add_point(*x, *y);
                    start_x = *x;
                    prev_x = *x;
                    start_y = *y;
                    prev_y = *y;
                }
                &PathSegment::LineTo { ref x, ref y } => {
                    bbox.add_point(*x, *y);
                    prev_x = *x;
                    prev_y = *y;
                }
                &PathSegment::CurveTo { ref x, ref y, ref x1, ref y1, ref x2, ref y2 } => {
                    bbox.add_bezier(prev_x, prev_y, *x1, *y1, *x2, *y2, *x, *y);
                    prev_x = *x;
                    prev_y = *y;
                }
                &PathSegment::ClosePath => {
                    prev_x = start_x;
                    prev_y = start_y;
                }
            }
        }
        Some(bbox)
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

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl BoundingBox {
    pub fn new(x: f32, y: f32) -> Self {
        BoundingBox {
            x1: x,
            y1: y,
            x2: x,
            y2: y,
        }
    }

    pub fn get_width(&self) -> f32 {
        self.x2 - self.x1
    }

    pub fn get_height(&self) -> f32 {
        self.y2 - self.y1
    }

    pub fn merge(&self, other: &BoundingBox) -> Self {
        let mut bbox = self.clone();
        if other.x1 < bbox.x1 {
            bbox.x1 = other.x1
        }
        if other.y1 < bbox.y1 {
            bbox.y1 = other.y1
        }
        if other.x2 > bbox.x2 {
            bbox.x2 = other.x2
        }
        if other.y2 > bbox.y2 {
            bbox.y2 = other.y2
        }
        bbox
    }

    pub fn extends(&mut self, width: f32) {
        self.x1 -= width;
        self.y1 -= width;
        self.x2 += width;
        self.y2 += width;
    }

    pub fn move_t(&mut self, x: f32, y: f32) {
        self.x1 += x;
        self.y1 += y;
        self.x2 += x;
        self.y2 += y;
    }

    pub fn add_point(&mut self, x: f32, y: f32) {
        if x < self.x1 {
            self.x1 = x
        }
        if x > self.x2 {
            self.x2 = x
        }
        if y < self.y1 {
            self.y1 = y
        }
        if y > self.y2 {
            self.y2 = y
        }
    }

    pub fn add_point_x(&mut self, x: f32) {
        if x < self.x1 {
            self.x1 = x
        }
        if x > self.x2 {
            self.x2 = x
        }
    }

    pub fn add_point_y(&mut self, y: f32) {
        if y < self.y1 {
            self.y1 = y
        }
        if y > self.y2 {
            self.y2 = y
        }
    }

    pub fn add_bezier(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.add_point(x0, y0);
        self.add_point(x, y);

        let mut compute = |p0: f32, p1: f32, p2: f32, p3: f32, i: usize| {
            let b = 6.0 * p0 - 12.0 * p1 + 6.0 * p2;
            let a = -3.0 * p0 + 9.0 * p1 - 9.0 * p2 + 3.0 * p3;
            let c = 3.0 * p1 - 3.0 * p0;

            if a == 0.0 {
                if b == 0.0 { return; }

                let t = -c / b;
                if 0.0 < t && t < 1.0 {
                    if i == 0 {
                        self.add_point_x(derive(p0, p1, p2, p3, t));
                    }

                    if i == 1 {
                        self.add_point_y(derive(p0, p1, p2, p3, t))
                    }
                }
                return;
            }

            let b2ac = b.powf(2.0) - 4.0 * c * a;
            if b2ac < 0.0 {
                return;
            }

            let t1 = (-b + b2ac.sqrt()) / (2.0 * a);
            if 0.0 < t1 && t1 < 1.0 {
                if i == 0 {
                    self.add_point_x(derive(p0, p1, p2, p3, t1));
                }

                if i == 1 {
                    self.add_point_y(derive(p0, p1, p2, p3, t1));
                }
            }
            let t2 = (-b - b2ac.sqrt()) / (2.0 * a);
            if 0.0 < t2 && t2 < 1.0 {
                if i == 0 {
                    self.add_point_x(derive(p0, p1, p2, p3, t2));
                }

                if i == 1 {
                    self.add_point_y(derive(p0, p1, p2, p3, t2));
                }
            }
        };
        compute(x0, x1, x2, x, 0);
        compute(y0, y1, y2, y, 1);
    }

    pub fn add_quad(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, x: f32, y: f32) {
        let cp1x = x0 + 2.0 / 3.0 * (x1 - x0);
        let cp1y = y0 + 2.0 / 3.0 * (y1 - y0);
        let cp2x = cp1x + 1.0 / 3.0 * (x - x0);
        let cp2y = cp1y + 1.0 / 3.0 * (y - y0);
        self.add_bezier(x0, y0, cp1x, cp1y, cp2x, cp2y, x, y);
    }
}

fn derive(v0: f32, v1: f32, v2: f32, v3: f32, t: f32) -> f32 {
    return (1.0 - t).powf(3.0) * v0 +
        3.0 * (1.0 - t).powf(2.0) * t * v1 +
        3.0 * (1.0 - t) * t.powf(2.0) * v2 +
        t.powf(3.0) * v3;
}