macro_rules! into_str {
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

pub mod svg_methods {
    use svg::{Document, Node};
    use svg::node::element::{Element, Style};

    pub fn create_svg_tag(width: f32, height: f32) -> Document {
        let mut svg = Document::new();
        svg.assign("width", into_str![width]);
        svg.assign("height", into_str![height]);
        svg.assign("viewBox", into_str!["0 0 ",width," ", height]);
        svg.assign("xmlns:xlink", "http://www.w3.org/1999/xlink");
        svg
    }

    pub fn create_style_tag(style: &str) -> Style {
        Style::new(style)
    }

    pub fn create_image_tag(href: String) -> Element {
        let mut image = Element::new("image");
        image.assign("xlink:href", href);
        image
    }

    pub fn create_group() -> Element {
        Element::new("g")
    }

    pub fn group(elements: Vec<Element>) -> Element {
        let mut g = Element::new("g");
        for item in elements {
            g.append(item)
        }
        g
    }

    pub fn create_path_tag(d: String) -> Element {
        let mut path_tag = Element::new("path");
        path_tag.assign("d", d);
        path_tag.assign("stroke-linecap", "round");
        path_tag.assign("stroke-linejoin", "round");
        path_tag
    }

    pub fn create_rect_tag(width: f32, height: f32, x: f32, y: f32) -> Element {
        let mut rect = Element::new("rect");
        rect.assign("width", into_str![width]);
        rect.assign("height", into_str![height]);
        rect.assign("x", into_str![x]);
        rect.assign("y", into_str![y]);
        rect
    }

    pub fn create_defs_tag() -> Element {
        Element::new("defs")
    }

    pub fn create_filter_tag() -> Element {
        Element::new("filter")
    }

    pub fn create_linear_gradient(vector: (f32, f32), stop: Vec<(String, String)>, key: String) -> Element {
        let mut linear_gradient = Element::new("linearGradient");
        linear_gradient.assign("id", key);
        linear_gradient.assign("x1", into_str![0]);
        linear_gradient.assign("y1", into_str![0]);
        linear_gradient.assign("x2", into_str![vector.0 * 100.0, "%"]);
        linear_gradient.assign("y2", into_str![vector.1 * 100.0, "%"]);
        for (key, value) in stop {
            let mut stop = Element::new("stop");
            stop.assign("offset", key.clone());
            stop.assign("stop-color", value.clone());
            linear_gradient.append(stop);
        }
        linear_gradient
    }

    pub fn create_use_tag(id: String) -> Element {
        let mut use_tag = Element::new("use");
        use_tag.assign("xlink:href", into_str!["#", id]);
        use_tag
    }

    pub fn create_mask_tag(elements: Vec<Element>, id: String) -> Element {
        let mut mask_tag = Element::new("mask");
        mask_tag.assign("id", id);
        for element in elements {
            mask_tag.append(element);
        }
        mask_tag
    }

    pub fn create_clip_tag(elements: Vec<Element>, id: String) -> Element {
        let mut clip_tag = Element::new("clipPath");
        clip_tag.assign("id", id);
        for element in elements {
            clip_tag.append(element);
        }
        clip_tag
    }

    pub fn apply_shadow(defs: &mut Element, element: &mut Element, color: (u8, u8, u8, f32), offset: (f32, f32), blur: f32) {
        let mut filter = create_filter_tag();
        let (r, g, b, a) = color;
        let (dx, dy) = offset;
        let id = into_str!["shadow-", r.clone(), g.clone(), b.clone(), a.clone(),dx.clone(), dy.clone(), blur.clone()];

        let mut fe_color_matrix = Element::new("feColorMatrix");
        fe_color_matrix.assign("type", "matrix");
        fe_color_matrix.assign("in", "SourceAlpha");
        fe_color_matrix.assign("result", "matrix");
        fe_color_matrix.assign("color-interpolation-filters", "sRGB");
        fe_color_matrix.assign("values", into_str![
            " 0 0 0 0 ",r as f32 / 255.0,
            " 0 0 0 0 ",g as f32 / 255.0,
            " 0 0 0 0 ",b as f32 / 255.0,
            " 0 0 0 ",a," 0"
        ]);

        let mut fe_offset = Element::new("feOffset");
        fe_offset.assign("dx", into_str![dx]);
        fe_offset.assign("dy", into_str![dy]);
        fe_offset.assign("in", "matrix");
        fe_offset.assign("result", "offset");

        let mut fe_gaussian_blur = Element::new("feGaussianBlur");

        fe_gaussian_blur.assign("stdDeviation", into_str![blur]);
        fe_gaussian_blur.assign("in", "offset");
        fe_gaussian_blur.assign("result", "blur");

        let mut fe_merge = Element::new("feMerge");

        let mut fe_merge_node = Element::new("feMergeNode");
        fe_merge_node.assign("in", "blur");
        let mut fe_merge_node1 = Element::new("feMergeNode");
        fe_merge_node1.assign("in", "SourceGraphic");

        fe_merge.append(fe_merge_node);
        fe_merge.append(fe_merge_node1);

        filter.append(fe_color_matrix);
        filter.append(fe_offset);
        filter.append(fe_gaussian_blur);
        filter.append(fe_merge);

        filter.assign("x", "-150%");
        filter.assign("y", "-150%");
        filter.assign("width", "400%");
        filter.assign("height", "400%");
        filter.assign("id", into_str![&id]);

        defs.append(filter);
        element.assign("filter", into_str!["url(#",&id,")"]);
    }
}

pub mod url {
    use hyper::client::Client;
    use hyper::body::to_bytes;
    use base64::encode;
    use bytes::Bytes;
    use tokio::runtime::Runtime;

    pub async fn fetch_async(url: &str) -> Option<Bytes> {
        let mut url = if !url.contains("http:") && !url.contains("https:") {
            format!("http:{}", url)
        } else {
            url.to_string()
        };
        url = url.replace("https:", "http:");
        let p = url.parse();
        if p.is_err() { return None; }
        let url = p.unwrap();
        let client = Client::new();
        let response = client.get(url).await;
        if response.is_err() { return None; }
        let response = response.unwrap();
        let bytes = to_bytes(response.into_body()).await;
        if bytes.is_err() { return None; }
        Some(bytes.unwrap())
    }

    pub fn fetch(url: &str) -> Option<Bytes> {
        let mut rt = Runtime::new().unwrap();
        rt.block_on(fetch_async(url))
    }

    pub async fn fetch_base64_async(url: &str) -> Option<String> {
        let p = url.parse();
        if p.is_err() { return None; }
        let url = p.unwrap();
        let client = Client::new();
        let response = client.get(url).await;
        if response.is_err() { return None; }
        let response = response.unwrap();
        let bytes = to_bytes(response.into_body()).await;
        if bytes.is_err() { return None; }
        let bytes = bytes.unwrap();
        Some(encode(&bytes))
    }

    pub fn fetch_base64(url: &str) -> Option<String> {
        let mut rt = Runtime::new().unwrap();
        rt.block_on(fetch_base64_async(url))
    }
}


pub mod render {
    use super::svg_methods::*;
    use core::open_type_like::transform::Transform;
    use core::open_type_like::path::PathData;
    use svg::Node;
    use svg::node::element::Element;

    pub fn split_color_string(s: &str) -> Option<(u8, u8, u8, f32)> {
        let s = s.replace(" ", "");
        if !s.contains("rgb") {
            if !s.contains("#") {
                return None;
            }
            let without_prefix = s.trim_start_matches("#");
            let color = i64::from_str_radix(without_prefix, 16u32).unwrap_or(0) as i64;
            let r = (color >> 16) as u8;
            let g = (color >> 8) as u8;
            let b = color as u8;
            let a = 1.0;
            Some((r, g, b, a))
        } else {
            let mode = if s.contains("rgba(") {
                "rgba("
            } else {
                "rgb("
            };
            let s_v: Vec<&str> = s.split(mode).collect();
            let s_v: Vec<&str> = s_v.get(1)?.split(")").collect();
            let s_v: Vec<&str> = s_v.get(0)?.split(",").collect();
            let r = s_v.get(0).unwrap_or(&"0").parse::<u8>().unwrap_or(0);
            let g = s_v.get(1).unwrap_or(&"0").parse::<u8>().unwrap_or(0);
            let b = s_v.get(2).unwrap_or(&"0").parse::<u8>().unwrap_or(0);
            let a = s_v.get(3).unwrap_or(&"1.0").parse::<f32>().unwrap_or(0.0f32);
            Some((r, g, b, a))
        }
    }

    pub enum Style {
        Color(u8, u8, u8, f32),
        Href(String),
    }

    impl From<&Style> for String {
        fn from(style: &Style) -> Self {
            match style {
                Style::Color(r, g, b, _a) => format!("rgb({:?},{:?},{:?})", *r, *g, *b),
                Style::Href(s) => s.clone()
            }
        }
    }

    pub struct Context {
        document: Element,
        last_path: PathData,
        transform: Transform,
        use_id: usize,
        defs: Element,
        width: f32,
        height: f32,
        pub fill_style: Style,
        pub stroke_style: Style,
        pub line_cap: String,
        pub line_join: String,
        pub line_width: f32,
        pub shadow: Option<((u8, u8, u8, f32), (f32, f32), f32)>,
    }

    impl Context {
        pub fn new(width: f32, height: f32) -> Self {
            Context {
                width,
                height,
                document: Element::new("g"),
                last_path: PathData::new(),
                transform: Default::default(),
                use_id: 0,
                defs: create_defs_tag(),
                fill_style: Style::Color(0, 0, 0, 1.0),
                stroke_style: Style::Color(0, 0, 0, 1.0),
                line_cap: "button".to_string(),
                line_join: "miter".to_string(),
                line_width: 2.0,
                shadow: None,
            }
        }

        pub fn create_linear_gradient(vector: (f32, f32), stop: Vec<(String, String)>, key: String) -> Element {
            create_linear_gradient(vector, stop, key)
        }

        pub fn begin_path(&mut self) {
            self.last_path = PathData::new();
        }

        pub fn move_to(&mut self, x: f32, y: f32) {
            self.last_path.move_to(x as f32, y as f32)
        }

        pub fn line_to(&mut self, x: f32, y: f32) {
            self.last_path.line_to(x, y)
        }

        pub fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
            self.last_path.curve_to(x, y, x1, y1, x2, y2)
        }

        pub fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
            self.last_path.quad_to(x, y, x1, y1)
        }

        pub fn close(&mut self) {
            self.last_path.close()
        }

        pub fn stroke(&mut self, p: Option<PathData>) {
            let mut path = p.unwrap_or(self.last_path.clone());
            path.transform(self.transform);
            let d = String::from(&path);
            let mut path = create_path_tag(d);
            path.assign("stroke", String::from(&self.stroke_style));
            path.assign("stroke-width", *&self.line_width);
            path.assign("fill", String::from(&self.fill_style));
            path.assign("stroke-linecap", String::from(&self.line_cap));
            path.assign("stroke-linejoin", String::from(&self.line_join));
            if self.shadow.is_some() { self.apply_shadow(&mut path) }
            self.document.append(path);
        }

        pub fn fill(&mut self, p: Option<PathData>) {
            let mut path = p.unwrap_or(self.last_path.clone());
            path.transform(self.transform);
            let d = String::from(&path);
            let mut path = create_path_tag(d);
            path.assign("stroke", String::from(&self.stroke_style));
            path.assign("stroke-width", "0");
            path.assign("fill", String::from(&self.fill_style));
            path.assign("stroke-linecap", String::from(&self.line_cap));
            path.assign("stroke-linejoin", String::from(&self.line_join));
            if self.shadow.is_some() { self.apply_shadow(&mut path) }
            self.document.append(path);
        }

        fn apply_shadow(&mut self, source: &mut Element) {
            let (color, offset, blur) = self.shadow.unwrap();
            apply_shadow(&mut self.defs, source, color, offset, blur);
        }

        pub fn clip(&mut self, p: Option<PathData>) -> String {
            let path = p.unwrap_or(self.last_path.clone());
            let d = String::from(&path);
            let mut path = create_path_tag(d);
            path.assign("fill", "#ffffff");
            let id = into_str!["mask-", self.use_id];
            let mask = create_mask_tag(vec![path], id.clone());
            self.defs.append(mask);
            self.use_id += 1;
            id
        }

        pub fn transform(&mut self, t: Transform) {
            self.transform.append(&t);
        }

        pub fn set_transform(&mut self, t: Transform) {
            self.transform = t;
        }

        pub fn reset_transform(&mut self) {
            self.transform = Default::default();
        }

        pub fn as_svg(&mut self) -> String {
            create_svg_tag(self.width.clone(), self.height.clone())
                .add(self.defs.clone())
                .add(self.document.clone())
                .to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::svg_methods::*;

    #[test]
    fn it_works() {
        let map = vec![("0".to_string(), "#ffffff".to_string()), ("1".to_string(), "#00ffcc".to_string())];
        let l = create_linear_gradient((0.0, 1.0), map, "hello".to_string());
        let mut defs = create_defs_tag();
        let mut rect = create_rect_tag(40.0, 40.0, 0.0, 0.0);
        apply_shadow(&mut defs, &mut rect, (0, 255, 128, 0.6), (2.0, 2.0), 0.6);
        let svg = create_svg_tag(500.0, 500.0).add(l).add(defs).add(rect);
        println!("{:?}", svg.to_string());
    }
}