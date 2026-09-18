use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::hocr_parser::utils::*;

/// Bounding box in scan pixel coordinates: [left, top, right, bottom]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HocrBbox(pub [i32; 4]);

#[allow(dead_code)]
pub struct Overlap {
    pub overlapping_self_pct: f32,
    pub overlapping_other_pct: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrUnknown {
    pub string: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrWord {
    #[serde(default = "word_level", skip_deserializing)]
    pub level: String,
    pub id: String,
    pub bbox: HocrBbox,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    pub text: String,
    pub wconf: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dropcap: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrLine {
    #[serde(default = "line_level", skip_deserializing)]
    pub level: String,
    pub id: String,
    pub bbox: HocrBbox,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    pub words: Vec<HocrWord>,
    pub baseline: Option<(f32, f32)>,
    pub x_size: Option<f32>,
    pub x_descenders: Option<f32>,
    pub x_ascenders: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Evidence {
    Untested,
    Undetermined,
    Suggested(bool),
    Determined(bool),
    Assigned(bool),
    Error,
}

impl Evidence {
    pub fn is_true(&self) -> Option<bool> {
        match self {
            Evidence::Suggested(b) | Evidence::Determined(b) | Evidence::Assigned(b) => Some(*b),
            _ => None,
        }
    }
}

impl Default for Evidence {
    fn default() -> Self {
        Evidence::Untested
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionThresholds {
    pub x_indent_min: i32,
    pub x_indent_max: i32,
    pub x_dedent_min: i32,
    pub x_dedent_max: i32,
    pub y_advance_min: i32,
    pub y_advance_max: i32,
    pub use_x_indent: bool,
    pub use_x_dedent: bool,
    pub use_y_advance: bool,
    pub use_hyphenation: bool,
}

impl Default for DetectionThresholds {
    fn default() -> Self {
        Self {
            x_indent_min: 5,
            x_indent_max: 15,
            x_dedent_min: 0,
            x_dedent_max: 20,
            y_advance_min: 0,
            y_advance_max: 0,
            use_x_indent: true,
            use_x_dedent: true,
            use_y_advance: true,
            use_hyphenation: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HocrBlockHints {
    #[serde(default)]
    pub test_x_indent: Evidence,
    #[serde(default)]
    pub test_x_dedent: Evidence,
    #[serde(default)]
    pub test_hyphenation: Evidence,
    #[serde(default)]
    pub test_y_advance: Evidence,
    #[serde(default)]
    pub test_y_reverse: Evidence,
    #[serde(default)]
    pub break_from_preceding: Evidence,
    #[serde(default)]
    pub break_from_following: Evidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrBlock {
    #[serde(default = "block_level", skip_deserializing)]
    pub level: String,
    pub id: String,
    pub bbox: HocrBbox,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(default)]
    pub hints: HocrBlockHints,
    pub kind: HocrBlockKind,
    pub lines: Vec<HocrLine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HocrBlockKind {
    Paragraph,
    Part,
    Chapter,
    Section,
    Subsection,
    Subsubsection,
    Subsubsubsection,
    Subsubsubsubsection,
    Image,
    Table,
    List,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AddBlockType {
    Text,
    Image,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColorSpecification {
    pub base_color: Option<String>,
    pub hue_shift: Option<f32>,
    pub lightness_shift: Option<f32>,
    pub saturation_shift: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSchema {
    pub name: String,
    pub color: Option<ColorSpecification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSchema {
    pub name: String,
    pub color: Option<ColorSpecification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrCarea {
    #[serde(default = "carea_level", skip_deserializing)]
    pub level: String,
    pub id: String,
    pub bbox: HocrBbox,
    #[serde(default)]
    pub flow: Option<String>,
    #[serde(default)]
    pub layout: Option<String>,
    pub blocks: Vec<HocrBlock>,
    pub unknowns: Vec<HocrUnknown>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropCapInjection {
    pub text: String,
    pub bbox: HocrBbox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrPage {
    #[serde(default = "page_level", skip_deserializing)]
    pub level: String,
    pub page_id: String,
    pub bbox: HocrBbox,
    pub careas: Vec<HocrCarea>,
    pub unknowns: Vec<HocrUnknown>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HocrPath {
    Page,
    Carea {
        carea: usize,
    },
    Block {
        carea: usize,
        block: usize,
    },
    Line {
        carea: usize,
        block: usize,
        line: usize,
    },
    Word {
        carea: usize,
        block: usize,
        line: usize,
        word: usize,
    },
}

impl HocrBbox {
    pub fn empty() -> Self {
        Self([0, 0, 0, 0])
    }

    #[allow(dead_code)]
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self([left, top, right, bottom])
    }

    pub fn left(self) -> i32 {
        self.0[0]
    }

    pub fn top(self) -> i32 {
        self.0[1]
    }

    pub fn right(self) -> i32 {
        self.0[2]
    }

    pub fn bottom(self) -> i32 {
        self.0[3]
    }

    pub fn center(self) -> (f32, f32) {
        (
            (self.left() + self.right()) as f32 / 2.0,
            (self.top() + self.bottom()) as f32 / 2.0,
        )
    }

    pub fn width(self) -> i32 {
        std::cmp::max(self.right() - self.left(), 0)
    }

    pub fn height(self) -> i32 {
        std::cmp::max(self.bottom() - self.top(), 0)
    }

    pub fn area(self) -> i32 {
        self.width() * self.height()
    }

    pub fn shift(&mut self, dx: i32, dy: i32) {
        self.0[0] += dx;
        self.0[1] += dy;
        self.0[2] += dx;
        self.0[3] += dy;
    }

    #[allow(dead_code)]
    pub fn aspect_ratio(self) -> f32 {
        self.width() as f32 / self.height() as f32
    }

    pub fn union(self, other: HocrBbox) -> HocrBbox {
        HocrBbox([
            std::cmp::min(self.left(), other.left()),
            std::cmp::min(self.top(), other.top()),
            std::cmp::max(self.right(), other.right()),
            std::cmp::max(self.bottom(), other.bottom()),
        ])
    }

    pub fn union_all(bboxes: &[HocrBbox]) -> Option<HocrBbox> {
        if bboxes.is_empty() {
            return None;
        }
        let mut bbox = bboxes[0];
        for b in bboxes.iter().skip(1) {
            bbox = bbox.union(*b);
        }
        Some(bbox)
    }

    pub fn intersection(self, other: HocrBbox) -> HocrBbox {
        HocrBbox([
            std::cmp::max(self.left(), other.left()),
            std::cmp::max(self.top(), other.top()),
            std::cmp::min(self.right(), other.right()),
            std::cmp::min(self.bottom(), other.bottom()),
        ])
    }

    pub fn overlap_percentage(self, other: HocrBbox) -> Overlap {
        let intersection = self.intersection(other);
        let overlapping_me = (intersection.area() as f32 / self.area() as f32) * 100.0;
        let overlapping_other = (intersection.area() as f32 / other.area() as f32) * 100.0;
        Overlap{overlapping_self_pct: overlapping_me, overlapping_other_pct: overlapping_other}
    }
}

impl HocrPath {
    #[allow(dead_code)]
    pub fn to_carea(&self) -> Option<HocrPath> {
        match self {
            HocrPath::Carea { carea } => Some(HocrPath::Carea { carea: *carea }),
            HocrPath::Block { carea, .. } => Some(HocrPath::Carea { carea: *carea }),
            HocrPath::Line { carea, .. } => Some(HocrPath::Carea { carea: *carea }),
            HocrPath::Word { carea, .. } => Some(HocrPath::Carea { carea: *carea }),
            _ => None,
        }
    }
    #[allow(dead_code)]
    pub fn to_block(&self) -> Option<HocrPath> {
        match self {
            HocrPath::Block { carea, block } => Some(HocrPath::Block { carea: *carea, block: *block }),
            HocrPath::Line { carea, block, .. } => Some(HocrPath::Block { carea: *carea, block: *block }),
            HocrPath::Word { carea, block, .. } => Some(HocrPath::Block { carea: *carea, block: *block }),
            _ => None,
        }
    }
    #[allow(dead_code)]
    pub fn to_line(&self) -> Option<HocrPath> {
        match self {
            HocrPath::Line { carea, block, line } => Some(HocrPath::Line { carea: *carea, block: *block, line: *line }),
            HocrPath::Word { carea, block, line, .. } => Some(HocrPath::Line { carea: *carea, block: *block, line: *line }),
            _ => None,
        }
    }
    #[allow(dead_code)]
    pub fn to_word(&self) -> Option<HocrPath> {
        match self {
            HocrPath::Word { carea, block, line, word } => Some(HocrPath::Word { carea: *carea, block: *block, line: *line, word: *word }),
            _ => None,
        }
    }
}

impl HocrBlockKind {
    pub fn tag_name(self) -> &'static str {
        match self {
            HocrBlockKind::Image => "img",
            HocrBlockKind::Paragraph => "p",
            HocrBlockKind::Part => "h1",
            HocrBlockKind::Chapter => "h1",
            HocrBlockKind::Section => "h2",
            HocrBlockKind::Subsection => "h3",
            HocrBlockKind::Subsubsection => "h4",
            HocrBlockKind::Subsubsubsection => "h5",
            HocrBlockKind::Subsubsubsubsection => "h6",
            HocrBlockKind::Table => "div",
            HocrBlockKind::List => "div",
        }
    }
    pub fn class_name(self) -> &'static str {
        match self {
            HocrBlockKind::Image => "ocr_photo",
            HocrBlockKind::Paragraph => "ocr_par",
            HocrBlockKind::Part => "ocr_part",
            HocrBlockKind::Chapter => "ocr_chapter",
            HocrBlockKind::Section => "ocr_section",
            HocrBlockKind::Subsection => "ocr_subsection",
            HocrBlockKind::Subsubsection => "ocr_subsubsection",
            HocrBlockKind::Subsubsubsection => "ocr_subsubsubsection",
            HocrBlockKind::Subsubsubsubsection => "ocr_subsubsubsubsection",
            HocrBlockKind::Table => "ocr_table",
            HocrBlockKind::List => "ocr_list",
        }
    }
    pub fn from_class_name(class_name: &str) -> Option<Self> {
        match class_name {
            "ocr_photo" => Some(HocrBlockKind::Image),
            "ocr_par" => Some(HocrBlockKind::Paragraph),
            "ocr_part" => Some(HocrBlockKind::Part),
            "ocr_chapter" => Some(HocrBlockKind::Chapter),
            "ocr_section" => Some(HocrBlockKind::Section),
            "ocr_subsection" => Some(HocrBlockKind::Subsection),
            "ocr_subsubsection" => Some(HocrBlockKind::Subsubsection),
            "ocr_subsubsubsection" => Some(HocrBlockKind::Subsubsubsection),
            "ocr_subsubsubsubsection" => Some(HocrBlockKind::Subsubsubsubsection),
            "ocr_table" => Some(HocrBlockKind::Table),
            "ocr_list" => Some(HocrBlockKind::List),
            _ => None,
        }
    }
    pub fn from_json_name(class_name: &str) -> Option<Self> {
        match class_name {
            "image" => Some(HocrBlockKind::Image),
            "paragraph" => Some(HocrBlockKind::Paragraph),
            "part" => Some(HocrBlockKind::Part),
            "chapter" => Some(HocrBlockKind::Chapter),
            "section" => Some(HocrBlockKind::Section),
            "subsection" => Some(HocrBlockKind::Subsection),
            "subsubsection" => Some(HocrBlockKind::Subsubsection),
            "subsubsubsection" => Some(HocrBlockKind::Subsubsubsection),
            "subsubsubsubsection" => Some(HocrBlockKind::Subsubsubsubsection),
            "table" => Some(HocrBlockKind::Table),
            "list" => Some(HocrBlockKind::List),
            _ => None,
        }
    }
}

impl HocrWord {
    pub fn shift(&mut self, dx: i32, dy: i32) {
        self.bbox.shift(dx, dy);
    }

    pub fn cascade_lang(&mut self, default_lang: Option<&str>) {
        if self.lang.is_none() {
            self.lang = default_lang.map(|s| s.to_string());
        }
    }
}

impl HocrLine {
    pub fn shift(&mut self, dx: i32, dy: i32) {
        self.bbox.shift(dx, dy);
        for word in &mut self.words {
            word.shift(dx, dy);
        }
    }

    pub fn cascade_lang(&mut self, default_lang: Option<&str>) {
        if self.lang.is_none() {
            self.lang = default_lang.map(|s| s.to_string());
        }
        let current_lang = self.lang.as_deref();
        for word in &mut self.words {
            word.cascade_lang(current_lang);
        }
    }

    pub fn rebuild_bbox(&mut self) {
        let subboxes = self.words.iter().map(|w| w.bbox).collect::<Vec<_>>();
        match HocrBbox::union_all(&subboxes) {
            Some(union) => self.bbox = union,
            None => self.bbox = HocrBbox::empty(),
        }
    }
}

impl HocrBlock {
    pub fn shift(&mut self, dx: i32, dy: i32) {
        self.bbox.shift(dx, dy);
        for line in &mut self.lines {
            line.shift(dx, dy);
        }
    }

    pub fn cascade_lang(&mut self, default_lang: Option<&str>) {
        if self.lang.is_none() {
            self.lang = default_lang.map(|s| s.to_string());
        }
        let current_lang = self.lang.as_deref();
        for line in &mut self.lines {
            line.cascade_lang(current_lang);
        }
    }

    pub fn rebuild_bbox(&mut self) {
        if self.kind == HocrBlockKind::Image {
            return;
        }
        let subboxes = self.lines.iter().map(|l| l.bbox).collect::<Vec<_>>();
        match HocrBbox::union_all(&subboxes) {
            Some(union) => self.bbox = union,
            None => self.bbox = HocrBbox::empty(),
        }
    }
}

impl HocrCarea {
    pub fn shift(&mut self, dx: i32, dy: i32) {
        self.bbox.shift(dx, dy);
        for block in &mut self.blocks {
            block.shift(dx, dy);
        }
    }

    pub fn cascade_lang(&mut self, default_lang: Option<&str>) {
        for block in &mut self.blocks {
            block.cascade_lang(default_lang);
        }
    }

    pub fn rebuild_bbox(&mut self) {
        let subboxes = self.blocks.iter().map(|b| b.bbox).collect::<Vec<_>>();
        match HocrBbox::union_all(&subboxes) {
            Some(union) => self.bbox = union,
            None => self.bbox = HocrBbox::empty(),
        }
    }
}

impl HocrPage {
    pub fn get_next_number_with_stem(&self, preferred_stem: &str) -> Option<usize> {
        let mut numbers: Vec<usize> = Vec::new();
        let stem_upto_underscore = preferred_stem
            .split_at(preferred_stem.find('_').unwrap_or(preferred_stem.len()))
            .0;

        match stem_upto_underscore {
            "carea" | "block" => {
                let carea_ids = self.careas.iter().map(|c| c.id.clone());
                numbers = carea_ids
                    .map(|id| count_from_id(&id))
                    .collect::<Result<Vec<usize>, std::num::ParseIntError>>()
                    .unwrap_or_default();
            }
            "par" => {
                let par_ids = self
                    .careas
                    .iter()
                    .flat_map(|c| c.blocks.iter().map(|b| b.id.clone()));
                numbers = par_ids
                    .map(|id| count_from_id(&id))
                    .collect::<Result<Vec<usize>, std::num::ParseIntError>>()
                    .unwrap_or_default();
            }
            "line" => {
                let line_ids = self.careas.iter().flat_map(|c| {
                    c.blocks
                        .iter()
                        .flat_map(|b| b.lines.iter().map(|l| l.id.clone()))
                });
                numbers = line_ids
                    .map(|id| count_from_id(&id))
                    .collect::<Result<Vec<usize>, std::num::ParseIntError>>()
                    .unwrap_or_default();
            }
            "word" => {
                let word_ids = self.careas.iter().flat_map(|c| {
                    c.blocks.iter().flat_map(|b| {
                        b.lines
                            .iter()
                            .flat_map(|l| l.words.iter().map(|w| w.id.clone()))
                    })
                });
                numbers = word_ids
                    .map(|id| count_from_id(&id))
                    .collect::<Result<Vec<usize>, std::num::ParseIntError>>()
                    .unwrap_or_default();
            }
            _ => {}
        }
        if numbers.is_empty() {
            None
        } else {
            Some(numbers.into_iter().max().unwrap() + 1)
        }
    }
    pub fn assign_unique_ids_to_careas(&self, careas: &mut [HocrCarea]) {
        let mut next_numbers: HashMap<String, usize> = HashMap::new();
        for carea in careas {
            carea.id = self.get_unique_id(&carea.id, &mut next_numbers);
            for block in &mut carea.blocks {
                block.id = self.get_unique_id(&block.id, &mut next_numbers);
                for line in &mut block.lines {
                    line.id = self.get_unique_id(&line.id, &mut next_numbers);
                    for word in &mut line.words {
                        word.id = self.get_unique_id(&word.id, &mut next_numbers);
                    }
                }
            }
        }
    }
    pub fn get_unique_id(&self, from_id: &str, next_numbers: &mut HashMap<String, usize>) -> String {
        let stem = stem_from_id(from_id);
        let next = next_numbers.entry(stem.clone()).or_insert_with(|| {
            self.get_next_number_with_stem(&stem).unwrap_or(1)
        });
        let id = format!("{}_{}", stem, *next);
        *next += 1;
        id
    }
    pub fn rebuild_bbox(&mut self) {
        let subboxes = self.careas.iter().map(|c| c.bbox).collect::<Vec<_>>();
        match HocrBbox::union_all(&subboxes) {
            Some(union) => self.bbox = union,
            None => self.bbox = HocrBbox::empty(),
        }
    }
}
