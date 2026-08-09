use std::cmp::PartialEq;
use std::collections::HashMap;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use crate::routes::projects::forms::AddBlockType;

pub fn stem_from_id(id: &str) -> String {
    id.chars()
        .rev()
        .skip_while(|c| c.is_numeric())
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>()
}

pub fn count_from_id(id: &str) -> Result<usize, std::num::ParseIntError> {
    let counter = id
        .chars()
        .rev()
        .take_while(|c| c.is_numeric())
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    counter.parse::<usize>()
}

fn page_level() -> String {
    "page".to_string()
}

fn carea_level() -> String {
    "carea".to_string()
}

fn block_level() -> String {
    "block".to_string()
}

fn line_level() -> String {
    "line".to_string()
}

fn word_level() -> String {
    "word".to_string()
}

/// Bounding box in scan pixel coordinates: [left, top, right, bottom]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HocrBbox(pub [i32; 4]);

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
    pub text: String,
    pub wconf: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrLine {
    #[serde(default = "line_level", skip_deserializing)]
    pub level: String,
    pub id: String,
    pub bbox: HocrBbox,
    pub words: Vec<HocrWord>,
    pub baseline: Option<(f32, f32)>,
    pub x_size: Option<f32>,
    pub x_descenders: Option<f32>,
    pub x_ascenders: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrBlock {
    #[serde(default = "block_level", skip_deserializing)]
    pub level: String,
    pub id: String,
    pub bbox: HocrBbox,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrCarea {
    #[serde(default = "carea_level", skip_deserializing)]
    pub level: String,
    pub id: String,
    pub bbox: HocrBbox,
    pub blocks: Vec<HocrBlock>,
    pub unknowns: Vec<HocrUnknown>,
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

impl HocrPage {
    pub fn to_hocr_html(&self) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html>\n");
        html.push_str("<body>\n");
        html.push_str(&format!(
            "<div class=\"ocr_page\" id=\"{}\" title=\"bbox {} {} {} {}\">",
            escape_attr(&self.page_id),
            self.bbox.left(),
            self.bbox.top(),
            self.bbox.right(),
            self.bbox.bottom(),
        ));

        for carea in &self.careas {
            html.push_str(&carea.to_hocr_html());
        }

        for unknown in &self.unknowns {
            html.push_str(&unknown.to_hocr_html());
        }

        html.push_str("</div>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        html
    }
    pub fn rebuild_bbox(&mut self) {
        let subboxes = self.careas.iter().map(|c| c.bbox).collect::<Vec<_>>();
        match HocrBbox::union_all(&subboxes) {
            Some(union) => self.bbox = union,
            None => self.bbox = HocrBbox::empty(),
        }
    }

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
        // println!(
        //     "get_next_number_with_stem: numbers: {:?} preferred_stem {}  stem_upto_underscore: {}",
        //     numbers, preferred_stem, stem_upto_underscore
        // );
        if numbers.is_empty() {
            None
        } else {
            Some(numbers.into_iter().max().unwrap() + 1)
        }
    }
    pub fn get_unique_id(&self, from_id: &str) -> String {
        let preferred_stem = stem_from_id(from_id);
        let next = self.get_next_number_with_stem(preferred_stem.as_str());

        match next {
            Some(n) => format!("{preferred_stem}{n}"),
            None => format!("{preferred_stem}1"),
        }
    }

    pub fn cleanup_carea(&mut self, carea: usize) {
        if self.careas[carea].blocks.is_empty() {
            self.careas.remove(carea);
        } else {
            self.careas[carea].rebuild_bbox();
        }
    }
    pub fn cleanup_block(&mut self, carea: usize, block: usize) {
        if self.careas[carea].blocks[block].lines.is_empty() {
            self.careas[carea].blocks.remove(block);
        } else {
            self.careas[carea].blocks[block].rebuild_bbox();
        }
        self.cleanup_carea(carea);
    }
    pub fn cleanup_line(&mut self, carea: usize, block: usize, line: usize) {
        if self.careas[carea].blocks[block].lines[line].words.is_empty() {
            self.careas[carea].blocks[block].lines.remove(line);
            self.cleanup_block(carea, block);
        } else {
            self.careas[carea].blocks[block].lines[line].rebuild_bbox();
        }
    }

    pub fn next_carea_path(&self, path: HocrPath) -> Option<HocrPath> {
        if let HocrPath::Carea { carea } = path {
            if carea < self.careas.len() - 1 {
                Some(HocrPath::Carea { carea: carea + 1 })
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn next_block_path(&self, path: HocrPath) -> Option<HocrPath> {
        if let HocrPath::Block { carea, block } = path {
            if block < self.careas[carea].blocks.len() - 1 {
                Some(HocrPath::Block {
                    carea,
                    block: block + 1,
                })
            } else {
                loop {
                    let path = self.next_carea_path(path.to_carea()?);
                    if let Some (HocrPath::Carea { carea }) = path {
                        if self.careas[carea].blocks.len() > 0 {
                            return Some(HocrPath::Block { carea, block: 0 });
                        }
                    }
                    else {
                        return None;
                    }
                }
            }
        } else {
            None
        }
    }
    pub fn previous_carea_path(&self, path: HocrPath) -> Option<HocrPath> {
        if let HocrPath::Carea { carea } = path {
            if carea > 0 {
                Some(HocrPath::Carea { carea: carea - 1 })
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn previous_block_path(&self, path: HocrPath) -> Option<HocrPath> {
        if let HocrPath::Block { carea, block } = path {
            if block > 0 {
                Some(HocrPath::Block {
                    carea,
                    block: block - 1,
                })
            } else {
                loop {
                    let path = self.previous_carea_path(path.to_carea()?);
                    if let Some(HocrPath::Carea { carea }) = path {
                        if self.careas[carea].blocks.len() > 0 {
                            return Some(HocrPath::Block { carea, block: self.careas[carea].blocks.len() - 1 });
                        }
                    } else {
                        return None;
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn move_carea_up(&mut self, carea: usize) {
        if carea > 0 {
            println!("move_carea_up: carea {}", carea);

            let moving_carea = self.careas.remove(carea);
            self.careas.insert(carea - 1, moving_carea);

            self.cleanup_carea(carea);
        }
        else {
            println!("move_carea_up: not moving");
        }
    }
    pub fn move_carea_down(&mut self, carea: usize) {
        if carea < self.careas.len() - 1 {
            println!("move_carea_down: carea {}", carea);

            let moving_carea = self.careas.remove(carea);
            self.careas.insert(carea + 1, moving_carea);

            self.cleanup_carea(carea);
        }
        else {
            println!("move_carea_down: not moving");
        }
    }
    pub fn move_block_up(&mut self, carea: usize, block: usize) {

        // Move within the same carea?
        if block > 0 {
            println!("move_block_up - within same carea: block {}", block);
            let moving_block = self.careas[carea].blocks.remove(block - 1);
            self.careas[carea].blocks.insert(block, moving_block);

            self.cleanup_carea(carea);
        }
        else if block == 0 {
            let previous_carea = self.previous_carea_path(HocrPath::Carea { carea });
            println!("move_block_up: previous_carea {:?}", previous_carea);
            if let Some(HocrPath::Carea { carea:to_carea }) = previous_carea {
                let moving_block = self.careas[carea].blocks.remove(block);
                let to_carea_size = self.careas[to_carea].blocks.len();

                self.careas[to_carea].blocks.insert(to_carea_size, moving_block);

                self.cleanup_carea(to_carea); // Note! Order... cleanup may remove empty careas.
                self.cleanup_carea(carea);    // Note! If so, indices are invalidated. Hence: to_ goes first.
            }
        }
    }
    pub fn move_block_down(&mut self, carea: usize, block: usize) {
        let carea_size = self.careas[carea].blocks.len();
        if block < carea_size - 1 {
            println!("move_block_down - within same carea: block {}", block);
            let block_down = self.careas[carea].blocks.remove(block);
            self.careas[carea].blocks.insert(block+1, block_down);

            self.cleanup_carea(carea);
        }
        else if block == carea_size - 1 {
            let next_carea = self.next_carea_path(HocrPath::Carea { carea });
            println!("move_block_down: next_carea {:?}", next_carea);
            if let Some (HocrPath::Carea { carea:to_carea }) = next_carea {
                let moving_block = self.careas[carea].blocks.remove(block);
                self.careas[to_carea].blocks.insert(0, moving_block);

                self.cleanup_carea(to_carea); // Note! Order... cleanup may remove empty careas.
                self.cleanup_carea(carea);    // Note! If so, indices are invalidated. Hence: to_ goes first.
            }
        }
        else {
            panic!("move_block_down: invalid block index");
        }
    }
    pub fn move_line_up(&mut self, carea: usize, block: usize, line: usize) {

        // Move within the same block?
        if line > 0 {
            println!("move_line_up - within same block: line {}", line);
            let moving_line = self.careas[carea].blocks[block].lines.remove(line - 1);
            self.careas[carea].blocks[block].lines.insert(line, moving_line);

            self.cleanup_block(carea, block);
        }
        else if line == 0 {
            let previous_block = self.previous_block_path(HocrPath::Block { carea, block });
            println!("move_line_up: previous_block {:?}", previous_block);
            if let Some(HocrPath::Block { carea:to_carea, block:to_block }) = previous_block {
                let moving_line = self.careas[carea].blocks[block].lines.remove(line);
                let to_block_size = self.careas[to_carea].blocks[to_block].lines.len();
                self.careas[to_carea].blocks[to_block].lines.insert(to_block_size, moving_line);

                self.cleanup_block(to_carea, to_block);     // Note! Order... cleanup may remove empty blocks.
                self.cleanup_block(carea, block);           // Note! If so, indices are invalidated. Hence: to_ goes first.
            }
        }
        else {
            println!("move_line_up: not moving");
        }
    }
    pub fn move_line_down(&mut self, carea: usize, block: usize, line: usize) {

        let block_size = self.careas[carea].blocks[block].lines.len();

        // Move within the same block?
        if line < block_size - 1 {
            println!("move_line_down - within same block: line {}", line);
            let moving_line = self.careas[carea].blocks[block].lines.remove(line);
            self.careas[carea].blocks[block].lines.insert(line+1, moving_line);

            self.cleanup_block(carea, block);
        }
        // Move line to next block ... even if that requires skipping over empty blocks?
        else if line == block_size - 1 {
            let next_block = self.next_block_path(HocrPath::Block { carea, block });
            println!("move_line_down: next_block {:?}", next_block);
            if let Some (HocrPath::Block { carea:to_carea, block:to_block }) = next_block {
                let moving_line = self.careas[carea].blocks[block].lines.remove(line);
                self.careas[to_carea].blocks[to_block].lines.insert(0, moving_line);

                self.cleanup_block(to_carea, to_block);     // Note! Order... cleanup may remove empty blocks.
                self.cleanup_block(carea, block);           // Note! If so, indices are invalidated. Hence: to_ goes first.
            }
        }
        else {
            println!("move_line_down: not moving");
        }
    }

    pub fn merge_carea(&mut self, carea1: usize, carea2: usize) {
        // Thought: Optionally, we could complain if the careas were not consecutive. But the algorithm is robust enough to handle that, so why?
        if carea1 != carea2 {
            let mut blocks = std::mem::take(&mut self.careas[carea2].blocks);

            self.careas[carea1].blocks.append(&mut blocks);
            self.careas[carea1].rebuild_bbox();
            self.remove_carea(carea2);
        }
    }
    pub fn merge_block(&mut self, carea: usize, block1: usize, block2: usize) {
        // Thought: Optionally, we could complain if the blocks were not consecutive. But the algorithm is robust enough to handle that, so why?
        if block1 != block2 {
            let mut lines = std::mem::take(&mut self.careas[carea].blocks[block2].lines);

            self.careas[carea].blocks[block1].lines.append(&mut lines);
            self.careas[carea].blocks[block1].rebuild_bbox();
            self.remove_block(carea, block2);
        }
    }
    pub fn merge_line(&mut self, carea: usize, block: usize, line1: usize, line2: usize) {
        // Thought: Optionally, we could complain if the lines were not consecutive. But the algorithm is robust enough to handle that, so why?
        if line1 != line2 {
            let mut words =
                std::mem::take(&mut self.careas[carea].blocks[block].lines[line2].words);

            self.careas[carea].blocks[block].lines[line1]
                .words
                .append(&mut words);
            self.careas[carea].blocks[block].lines[line1].rebuild_bbox();
            self.remove_line(carea, block, line2);
        }
    }

    pub fn split_carea(&mut self, carea: usize, block_before: usize, block_after: usize) {
        if block_before == block_after {
            return;
        }

        let new_id = self.get_unique_id(self.careas[carea].id.as_str());
        let old_carea = &mut self.careas[carea];
        let (_, right) = old_carea.blocks.split_at_mut(block_after);
        let new_carea = HocrCarea {
            level: "carea".to_string(),
            id: new_id,
            bbox: HocrBbox::empty(),
            blocks: right.to_vec(),
            unknowns: vec![],
        };
        old_carea.blocks.truncate(block_after);
        self.careas.insert(carea + 1, new_carea);
        self.careas[carea].rebuild_bbox();
        self.careas[carea + 1].rebuild_bbox();
    }
    pub fn split_block(&mut self, carea: usize, block: usize, line_before: usize, line_after: usize) {
        if line_before == line_after {
            return;
        }
        let new_id = self.get_unique_id(self.careas[carea].blocks[block].id.as_str());
        let carea = &mut self.careas[carea];
        let old_block = &mut carea.blocks[block];
        let (_, right) = old_block.lines.split_at_mut(line_after);
        let new_block = HocrBlock {
            lang: old_block.lang.clone(),
            kind: HocrBlockKind::Paragraph,
            level: "block".to_string(),
            id: new_id,
            bbox: HocrBbox::empty(),
            lines: right.to_vec(),
        };
        old_block.lines.truncate(line_after);
        carea.blocks.insert(block + 1, new_block);
        carea.blocks[block].rebuild_bbox();
        carea.blocks[block + 1].rebuild_bbox();
    }

    pub fn add_carea(&mut self, bbox: HocrBbox, erase_underneath: Option<bool>, erase_overlap: Option<u8>) {

        if erase_underneath.unwrap_or(false) && erase_overlap.is_some() {
            let erase_carea_ids: Vec<String> = self.careas
                .iter()
                .filter(|carea| bbox.overlap_percentage(carea.bbox).overlapping_other_pct as u8 >= erase_overlap.unwrap())
                .map(|carea| carea.id.clone())
                .collect();

            self.careas.retain(|carea| !erase_carea_ids.contains(&carea.id));
        }

        // Nodes are not really meant to overlap. It is up to the user to handle this case.
        // We try to place it in a suitable place. Until we have layout information (columns etc.) we find the first vertical slot between two existing areas and place it there.

        let vmid = bbox.center().1;

        let mut carea_index = None;
        for (i, carea) in self.careas.iter().enumerate() {
            if carea.bbox.center().1 > vmid {
                carea_index = Some(i);
                break;
            }
        }
        let carea_index = carea_index.unwrap_or(self.careas.len());

        let new_id = self.get_unique_id("carea");
        self.careas.insert(carea_index, HocrCarea {
            level: "carea".to_string(),
            id: new_id,
            bbox,
            blocks: vec![],
            unknowns: vec![],
        });

        self.rebuild_bbox();
    }
    pub fn add_block(&mut self, carea: usize, bbox: HocrBbox, block_type: Option<AddBlockType>, erase_underneath: Option<bool>, erase_overlap: Option<u8>) {

        if erase_underneath.unwrap_or(false) && erase_overlap.is_some() {
            let erase_block_ids: Vec<String> = self.careas[carea].blocks
                .iter()
                .filter(|carea| bbox.overlap_percentage(carea.bbox).overlapping_other_pct as u8 >= erase_overlap.unwrap())
                .map(|carea| carea.id.clone())
                .collect();

            self.careas[carea].blocks.retain(|block| !erase_block_ids.contains(&block.id));
        }

        // Nodes are not really meant to overlap. It is up to the user to handle this case.
        // We try to place it in a suitable place. Until we have layout information (columns etc.) we find the first vertical slot between two existing areas and place it there.

        let vmid = bbox.center().1;

        let mut block_index = None;

        for (i, block) in self.careas[carea].blocks.iter().enumerate() {
            if block.bbox.center().1 > vmid {
                block_index = Some(i);
                break;
            }
        }
        let block_index = block_index.unwrap_or(self.careas[carea].blocks.len());
        let new_id = self.get_unique_id("par");
        let block_kind = if block_type.unwrap_or(AddBlockType::Text) == AddBlockType::Image { HocrBlockKind::Image } else { HocrBlockKind::Paragraph };

        self.careas[carea].blocks.insert(block_index, HocrBlock {
            id: new_id,
            level: "block".to_string(),
            kind: block_kind,
            lang: None,
            bbox,
            lines: vec![],
        });

        self.cleanup_carea(carea);
    }
    pub fn add_line(&mut self, _carea: usize, _block: usize, _bbox: HocrBbox) {
        // Not implemented yet
    }
    pub fn add_word(&mut self, _carea: usize, _block: usize, _line: usize, _bbox: HocrBbox, _text: Option<String>) {
        // Not implemented yet
    }

    pub fn remove_carea(&mut self, carea: usize) {
        self.careas.remove(carea);
        self.rebuild_bbox();
    }
    pub fn remove_block(&mut self, carea: usize, block: usize) {
        self.careas[carea].blocks.remove(block);
        if self.careas[carea].blocks.is_empty() {
            self.remove_carea(carea);
        } else {
            self.careas[carea].rebuild_bbox();
        }
    }
    pub fn remove_line(&mut self, carea: usize, block: usize, line: usize) {
        self.careas[carea].blocks[block].lines.remove(line);
        if self.careas[carea].blocks[block].lines.is_empty() {
            self.remove_block(carea, block);
        } else {
            self.careas[carea].blocks[block].rebuild_bbox();
        }
    }
    pub fn remove_word(&mut self, carea: usize, block: usize, line: usize, word: usize) {
        self.careas[carea].blocks[block].lines[line]
            .words
            .remove(word);
        if self.careas[carea].blocks[block].lines[line]
            .words
            .is_empty()
        {
            self.remove_line(carea, block, line);
        } else {
            self.careas[carea].blocks[block].lines[line].rebuild_bbox();
        }
    }

    pub fn change_block_kind(&mut self, carea: usize, block: usize, kind: HocrBlockKind) {
        self.careas[carea].blocks[block].kind = kind;
        self.careas[carea].blocks[block].rebuild_bbox();
    }
}

impl HocrCarea {
    pub fn to_hocr_html(&self) -> String {
        let mut html = format!(
            "<div class=\"ocr_carea\" id=\"{}\" title=\"bbox {} {} {} {}\">",
            escape_attr(&self.id),
            self.bbox.left(),
            self.bbox.top(),
            self.bbox.right(),
            self.bbox.bottom(),
        );

        for block in &self.blocks {
            html.push_str(&block.to_hocr_html());
        }

        html.push_str("</div>");
        html
    }
    pub fn rebuild_bbox(&mut self) {
        let subboxes = self.blocks.iter().map(|b| b.bbox).collect::<Vec<_>>();
        match HocrBbox::union_all(&subboxes) {
            Some(union) => self.bbox = union,
            None => self.bbox = HocrBbox::empty(),
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
            _ => None,
        }
    }
}

impl HocrBlock {
    pub fn to_hocr_html(&self) -> String {
        let tag = self.kind.tag_name();
        let class = self.kind.class_name();

        let lang_attr = self
            .lang
            .as_deref()
            .map(|lang| format!(" lang=\"{}\"", escape_attr(lang)))
            .unwrap_or_default();

        let mut html = format!(
            "<{tag} class=\"{class}\" id=\"{}\" title=\"bbox {} {} {} {}\"{}>",
            escape_attr(&self.id),
            self.bbox.left(),
            self.bbox.top(),
            self.bbox.right(),
            self.bbox.bottom(),
            lang_attr,
        );

        for line in &self.lines {
            html.push_str(&line.to_hocr_html());
        }

        html.push_str(&format!("</{tag}>"));
        html
    }
    pub fn rebuild_bbox(&mut self) {
        let subboxes = self.lines.iter().map(|l| l.bbox).collect::<Vec<_>>();
        match HocrBbox::union_all(&subboxes) {
            Some(union) => self.bbox = union,
            None => self.bbox = HocrBbox::empty(),
        }
    }
}

impl HocrLine {
    pub fn to_hocr_html(&self) -> String {
        let mut html = format!(
            "<span class=\"ocr_line\" id=\"{}\" title=\"bbox {} {} {} {}; baseline {} {}; x_size {}; x_descenders {}; x_ascenders {}\">",
            escape_attr(&self.id),
            self.bbox.left(),
            self.bbox.top(),
            self.bbox.right(),
            self.bbox.bottom(),
            self.baseline.unwrap_or((0.0, 0.0)).0,
            self.baseline.unwrap_or((0.0, 0.0)).1,
            self.x_size.unwrap_or(0.0),
            self.x_descenders.unwrap_or(0.0),
            self.x_ascenders.unwrap_or(0.0)
        );

        for word in &self.words {
            html.push_str(&word.to_hocr_html());
        }

        html.push_str("</span>");
        html
    }
    pub fn rebuild_bbox(&mut self) {
        let subboxes = self.words.iter().map(|w| w.bbox).collect::<Vec<_>>();
        match HocrBbox::union_all(&subboxes) {
            Some(union) => self.bbox = union,
            None => self.bbox = HocrBbox::empty(),
        }
    }
}

impl HocrWord {
    pub fn to_hocr_html(&self) -> String {
        format!(
            "<span class=\"ocrx_word\" id=\"{}\" title=\"bbox {} {} {} {}; x_wconf {}\">{}</span>",
            escape_attr(&self.id),
            self.bbox.left(),
            self.bbox.top(),
            self.bbox.right(),
            self.bbox.bottom(),
            self.wconf,
            escape_text(&self.text),
        )
    }
}

impl HocrUnknown {
    pub fn to_hocr_html(&self) -> String {
        self.string.clone()
    }
}

pub fn collect_unknowns(el: ElementRef, selector: &Selector) -> Vec<HocrUnknown> {
    el.child_elements()
        .filter(|el| !selector.matches(el))
        .map(|el| HocrUnknown { string: el.html() })
        .collect()
}

pub fn parse(html: &str) -> Option<HocrPage> {
    let document = Html::parse_document(html);

    let sel_page = Selector::parse("div.ocr_page").ok()?;
    let sel_carea = Selector::parse("div.ocr_carea").ok()?;
    let sel_block = Selector::parse("img, p.ocr_par, h1.ocr_part, h1.ocr_chapter, h2.ocr_section, h3.ocr_subsection, h4.ocr_subsubsection, h5.ocr_subsubsubsection, h6.ocr_subsubsubsubsection").ok()?;
    let sel_line = Selector::parse("span.ocr_line, span.ocr_caption").ok()?;
    let sel_word = Selector::parse("span.ocrx_word").ok()?;


    let page_el = document.select(&sel_page).next()?;
    let page_id = page_el.attr("id").unwrap_or("page_1").to_string();
    let page_bbox = bbox(page_el.attr("title").unwrap_or(""))?;

    let careas = page_el
        .select(&sel_carea)
        .filter_map(|carea_el| {
            let carea_bbox = bbox(carea_el.attr("title").unwrap_or(""))?;
            let carea_id = carea_el.attr("id").unwrap_or("").to_string();

            let blocks = carea_el
                .select(&sel_block)
                .filter_map(|block_el| {
                    let block_bbox = bbox(block_el.attr("title").unwrap_or(""))?;
                    let block_id = block_el.attr("id").unwrap_or("").to_string();
                    let block_lang = block_el.attr("lang").map(str::to_string);

                    let lines = block_el
                        .select(&sel_line)
                        .filter_map(|line_el| {
                            let title_keyvals = split_title(line_el.attr("title").unwrap_or(""));
                            let line_bbox = title_keyvals.get("bbox").map(|s| to_bbox(s)).unwrap_or(HocrBbox::empty());
                            let line_baseline = title_keyvals.get("baseline").and_then(|s| to_baseline(s));
                            let line_x_size = title_keyvals.get("x_size").and_then(|s| s.parse::<f32>().ok());
                            let line_x_ascenders = title_keyvals.get("x_ascenders").and_then(|s| s.parse::<f32>().ok());
                            let line_x_descenders = title_keyvals.get("x_descenders").and_then(|s| s.parse::<f32>().ok());
                            let line_id = line_el.attr("id").unwrap_or("").to_string();

                            let words = line_el
                                .select(&sel_word)
                                .filter_map(|word_el| {
                                    let title = word_el.attr("title").unwrap_or("");
                                    let word_bbox = bbox(title)?;
                                    Some(HocrWord {
                                        level: "word".to_string(),
                                        id: word_el.attr("id").unwrap_or("").to_string(),
                                        bbox: word_bbox,
                                        text: word_el.text().collect::<String>().trim().to_string(),
                                        wconf: wconf(title),
                                    })
                                })
                                .collect();

                            Some(HocrLine {
                                level: "line".to_string(),
                                id: line_id,
                                bbox: line_bbox,
                                baseline: line_baseline,
                                x_size: line_x_size,
                                x_ascenders: line_x_ascenders,
                                x_descenders: line_x_descenders,
                                words,
                            })
                        })
                        .collect();

                    let kind = if block_el.value().name() == "img" {
                        Some(HocrBlockKind::Image)
                    } else {
                        block_el
                            .attr("class")
                            .unwrap_or("")
                            .split_whitespace()
                            .find_map(HocrBlockKind::from_class_name)
                    }?;
                    let block = HocrBlock {
                        level: "block".to_string(),
                        id: block_id,
                        bbox: block_bbox,
                        lang: block_lang,
                        kind,
                        lines,
                    };

                    Some(block)
                })
                .collect();

            let unknowns = collect_unknowns(page_el, &sel_carea);

            Some(HocrCarea {
                level: "carea".to_string(),
                id: carea_id,
                bbox: carea_bbox,
                blocks,
                unknowns
            })
        })
        .collect();

    let unknowns = collect_unknowns(page_el, &sel_carea);

    Some(HocrPage {
        level: "page".to_string(),
        page_id,
        bbox: page_bbox,
        careas,
        unknowns,
    })
}

#[allow(dead_code)]
fn has_class(el: &scraper::ElementRef<'_>, class_name: &str) -> bool {
    el.attr("class")
        .unwrap_or("")
        .split_whitespace()
        .any(|c| c == class_name)
}

#[allow(dead_code)]
fn split_title(title: &str) -> HashMap<String, String> {

    let mut keyvals : HashMap<String, String> = HashMap::new();

    for part in title.split(";").map(|s| s.trim()) {
        let mut parts = part.split(" ");
        let key = parts.next().unwrap();
        let value = parts.collect::<Vec<_>>().join(" ");
        keyvals.insert(key.to_string(), value);
    };
    keyvals
}

#[allow(dead_code)]
fn join_title(keyvals: &HashMap<String, String>) -> String {
    keyvals.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ")
}


fn to_bbox(bbox_str: &str) -> HocrBbox {
    let v: Vec<i32> = bbox_str
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if v.len() >= 4 {
        HocrBbox([v[0], v[1], v[2], v[3]])
    } else {
        HocrBbox::empty()
    }
}

fn to_baseline(baseline_str: &str) -> Option<(f32, f32)> {
    let v: Vec<f32> = baseline_str
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if v.len() >= 2 {
        Some((v[0], v[1]))
    } else {
        None
    }
}

fn to_wconf(wconf_str: &str) -> Option<i32> {
    wconf_str.trim().parse().ok()
}

fn bbox(title: &str) -> Option<HocrBbox> {
    for part in title.split(';') {
        if let Some(rest) = part.trim().strip_prefix("bbox ") {
            let v: Vec<i32> = rest
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if v.len() >= 4 {
                return Some(HocrBbox([v[0], v[1], v[2], v[3]]));
            }
        }
    }
    None
}

fn wconf(title: &str) -> i32 {
    for part in title.split(';') {
        if let Some(rest) = part.trim().strip_prefix("x_wconf ") {
            if let Ok(n) = rest.trim().parse() {
                return n;
            }
        }
    }
    0
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    escape_text(s).replace('"', "&quot;")
}

pub fn find_node(page: &HocrPage, id: &str) -> Option<HocrPath> {
    if page.page_id == id {
        return Some(HocrPath::Page);
    }

    for (i, carea) in page.careas.iter().enumerate() {
        if carea.id == id {
            return Some(HocrPath::Carea { carea: i });
        }

        for (j, block) in carea.blocks.iter().enumerate() {
            if block.id == id {
                return Some(HocrPath::Block { carea: i, block: j });
            }

            for (k, line) in block.lines.iter().enumerate() {
                if line.id == id {
                    return Some(HocrPath::Line {
                        carea: i,
                        block: j,
                        line: k,
                    });
                }

                for (l, word) in line.words.iter().enumerate() {
                    if word.id == id {
                        return Some(HocrPath::Word {
                            carea: i,
                            block: j,
                            line: k,
                            word: l,
                        });
                    }
                }
            }
        }
    }

    None
}
