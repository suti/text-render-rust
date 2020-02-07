#[derive(Debug, Clone)]
pub struct BBox {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl BBox {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> BBox {
        BBox {
            x1,
            y1,
            x2,
            y2,
        }
    }

    pub fn get_width(&self) -> f64 {
        self.x2
    }

    pub fn get_height(&self) -> f64 {
        self.y2
    }

    pub fn get_real_width(&self) -> f64 {
        self.x2 - self.x1
    }
    pub fn get_real_height(&self) -> f64 {
        self.y2 - self.y1
    }

    pub fn compare(&self, b: &Self) -> bool {
        self.x1 * self.x2 > b.x1 * b.x2
    }
}

impl std::default::Default for BBox {
    fn default() -> Self {
        BBox {
            x1: 0f64,
            y1: 0f64,
            x2: 0f64,
            y2: 0f64,
        }
    }
}

impl From<&BBox> for String {
    fn from(BBox { x1, y1, x2, y2 }: &BBox) -> Self {
        format!("[{},{},{},{}]", x1, y1, x2, y2)
    }
}

impl From<&BBox> for Vec<f32> {
    fn from(BBox { x1, y1, x2, y2 }: &BBox) -> Self {
        vec![*x1 as f32, *y1 as f32, *x2 as f32, *y2 as f32]
    }
}


#[derive(Debug, Clone)]
pub struct BBoxes(Vec<BBox>);

impl BBoxes {
    pub fn new() -> Self {
        BBoxes(Vec::<BBox>::new())
    }

    pub fn get_total_box(&self) -> BBox {
        let mut b_box: BBox = Default::default();
        if self.len() > 0 {
            let b_box_1 = self.get(0).unwrap();
            b_box = b_box_1.clone();
            for item in self.iter() {
                if item.x1 < b_box.x1 {
                    b_box.x1 = item.x1
                }
                if item.y1 < b_box.y1 {
                    b_box.y1 = item.y1
                }
                if item.x2 > b_box.x2 {
                    b_box.x2 = item.x2
                }
                if item.y2 > b_box.y2 {
                    b_box.y2 = item.y2
                }
            }
        }
        b_box
    }
}


impl std::ops::Deref for BBoxes {
    type Target = Vec<BBox>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for BBoxes {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<&BBoxes> for String {
    fn from(boxes: &BBoxes) -> Self {
        let mut result = String::from("[");
        for segment in boxes.iter() {
            result.push_str(&String::from(segment));
            result.push_str(",");
        }
        if boxes.len() > 0 { result.pop(); }
        result.push_str("]");
        result.to_string()
    }
}

impl From<&BBoxes> for Vec<f32> {
    fn from(b_boxes: &BBoxes) -> Self {
        let mut result = Vec::<f32>::new();
        result.push(b_boxes.len() as f32);
        for b_box in b_boxes.iter() {
            let vec: Vec<f32> = b_box.into();
            for b in vec.iter() {
                result.push(*b);
            }
        }
        result
    }
}