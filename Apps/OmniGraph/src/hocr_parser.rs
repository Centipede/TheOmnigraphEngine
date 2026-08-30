use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

pub fn stem_from_id(id: &str) -> String {

    let stem = id.chars()
        .rev()
        .skip_while(|c| c.is_numeric() )
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();

    if stem.ends_with('_') {
        stem.chars()
            .rev()
            .skip_while(|c| *c == '_' )
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>()
    } else {
        stem
    }
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


fn signature_word(word: &HocrWord) -> String {
    word.id.clone()
}

fn signature_line(line: &HocrLine) -> String {
    let word_sigs = line.words.iter().map(signature_word).collect::<Vec<String>>();
    format!("{}({})", line.id.clone(), word_sigs.join(","))
}

fn signature_block(block: &HocrBlock) -> String {
    let line_sigs = block.lines.iter().map(signature_line).collect::<Vec<String>>();
    format!("{}:{}({})", block.id.clone(), block.kind, line_sigs.join(","))
}

fn signature_carea(carea: &HocrCarea) -> String {
    let block_sigs = carea.blocks.iter().map(signature_block).collect::<Vec<String>>();
    format!("{}({})", carea.id.clone(), block_sigs.join(","))
}

fn signature(page: &HocrPage) -> String {
    let carea_sigs = page.careas.iter().map(signature_carea).collect::<Vec<String>>();
    format!("{}({})", page.page_id, carea_sigs.join(","))
}

fn to_sig(s: &str) -> String {
    s.replace(' ', "").replace('\n', "").replace('\r', "")
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

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AddBlockType {
    Text,
    Image,
}

/// A color can either be determined by a base color, or by a hue shift, lightness shift, and/or saturation shift.
/// The point is that either a specification defines a color directly (base color) or it provides adjustment for an already given color.
/// This way you can stack specifications. If a carea has its own base color, then a flow and a layout may define two stackable modifications to that base color.
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
    pub fn shift(&mut self, dx: i32, dy: i32) {
        self.bbox.shift(dx, dy);
        for carea in &mut self.careas {
            carea.shift(dx, dy);
        }
    }

    pub fn cascade_lang(&mut self, default_lang: Option<&str>) {
        for carea in &mut self.careas {
            carea.cascade_lang(default_lang);
        }
    }

    pub fn replace_or_merge_carea(&mut self, carea_index: usize, mut new_careas: Vec<HocrCarea>) {
        // 1. Assign unique IDs recursively
        self.assign_unique_ids_to_careas(&mut new_careas);

        // 2. Merge or Insert logic
        if new_careas.len() == 1 {
            let new_carea = new_careas.pop().unwrap();
            self.careas[carea_index].blocks.extend(new_carea.blocks);
            self.careas[carea_index].rebuild_bbox();
        } else if new_careas.len() > 1 {
            self.insert_careas_after(carea_index, new_careas);
        }
    }

    pub fn insert_careas_after(&mut self, index: usize, new_careas: Vec<HocrCarea>) {
        if index < self.careas.len() {
            let mut tail = self.careas.split_off(index + 1);
            self.careas.extend(new_careas);
            self.careas.append(&mut tail);
        } else {
            self.careas.extend(new_careas);
        }
    }

    pub fn change_carea_flow(&mut self, carea_index: usize, flow: Option<String>) {
        if carea_index < self.careas.len() {
            self.careas[carea_index].flow = flow;
        }
    }

    pub fn change_carea_layout(&mut self, carea_index: usize, layout: Option<String>) {
        if carea_index < self.careas.len() {
            self.careas[carea_index].layout = layout;
        }
    }

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

    pub fn inject_dropcaps(&mut self, injections: Vec<DropCapInjection>) {
        for injection in injections {
            let mut best_match: Option<(usize, usize, usize, i32)> = None;

            for (c_idx, carea) in self.careas.iter().enumerate() {
                for (b_idx, block) in carea.blocks.iter().enumerate() {
                    for (l_idx, line) in block.lines.iter().enumerate() {
                        if line.words.is_empty() {
                            continue;
                        }

                        let v_diff = (line.bbox.top() - injection.bbox.top()).abs();
                        let h_dist = (line.bbox.left() - injection.bbox.right()).abs();

                        if v_diff <= 50 && h_dist <= 100 {
                            match best_match {
                                Some((_, _, _, best_v_diff)) if v_diff < best_v_diff => {
                                    best_match = Some((c_idx, b_idx, l_idx, v_diff));
                                }
                                None => {
                                    best_match = Some((c_idx, b_idx, l_idx, v_diff));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            if let Some((c_idx, b_idx, l_idx, _)) = best_match {
                if let Some(word) = self.careas[c_idx].blocks[b_idx].lines[l_idx].words.get_mut(0) {
                    word.dropcap = Some(injection.text);
                }
            }
        }
    }
    pub fn rebuild_bbox(&mut self) {
        let subboxes = self.careas.iter().map(|c| c.bbox).collect::<Vec<_>>();
        match HocrBbox::union_all(&subboxes) {
            Some(union) => self.bbox = union,
            None => self.bbox = HocrBbox::empty(),
        }
    }

    pub fn collect_all_words(&self) -> Vec<HocrWord> {
        self.careas
            .iter()
            .flat_map(|c| c.blocks.iter())
            .flat_map(|b| b.lines.iter())
            .flat_map(|l| l.words.iter())
            .cloned()
            .collect()
    }

    pub fn replace_words(&mut self, word_id: &str, mut with_words: Vec<HocrWord>) {
        if let Some(HocrPath::Word {
            carea,
            block,
            line,
            word,
        }) = find_node(self, word_id)
        {
            // Assign unique IDs to new words
            let preferred_stem = stem_from_id(word_id);
            let mut next_number = self
                .get_next_number_with_stem(preferred_stem.as_str())
                .unwrap_or(1);

            for new_word in &mut with_words {
                new_word.id = format!("{}_{}", preferred_stem, next_number);
                next_number += 1;
            }

            // Remove the original word and insert new words at its position
            self.careas[carea].blocks[block].lines[line]
                .words
                .splice(word..word + 1, with_words);

            // Rebuild the bounding box of the containing line
            self.cleanup_line(carea, block, line);
        }
    }

    pub fn auto_flow(&mut self, flows: Vec<FlowSchema>, _layouts: Vec<LayoutSchema>, merge: bool) {
        if flows.is_empty() {
            return;
        }

        let default_flow = flows[0].name.clone();

        // 1. Assign default flow to careas with no current assignment.
        for carea in &mut self.careas {
            if carea.flow.as_ref().map_or(true, |f| f.is_empty()) {
                carea.flow = Some(default_flow.clone());
            }
        }

        if self.careas.is_empty() {
            return;
        }

        // 2. Group consecutive careas by layout.
        let mut new_careas: Vec<HocrCarea> = Vec::new();
        let old_careas = std::mem::take(&mut self.careas);

        let mut current_layout_group: Vec<HocrCarea> = Vec::new();
        let mut current_layout = old_careas[0].layout.clone();

        for carea in old_careas {
            if carea.layout != current_layout {
                // Process previous layout group
                Self::auto_flow_for_layout(&mut new_careas, current_layout_group, merge);
                current_layout_group = Vec::new();
                current_layout = carea.layout.clone();
            }
            current_layout_group.push(carea);
        }
        // Process last group
        if !current_layout_group.is_empty() {
            Self::auto_flow_for_layout(&mut new_careas, current_layout_group, merge);
        }

        self.careas = new_careas;
        self.rebuild_bbox();
    }

    fn auto_flow_for_layout(target: &mut Vec<HocrCarea>, group: Vec<HocrCarea>, merge: bool) {
        if !merge {
            target.extend(group);
            return;
        }

        let mut flow_order: Vec<String> = Vec::new();
        let mut merged_careas: HashMap<String, HocrCarea> = HashMap::new();

        for mut carea in group {
            let flow = carea.flow.clone().unwrap_or_default();
            if !merged_careas.contains_key(&flow) {
                flow_order.push(flow.clone());
                merged_careas.insert(flow, carea);
            } else {
                let existing = merged_careas.get_mut(&flow).unwrap();
                existing.blocks.append(&mut carea.blocks);
                existing.unknowns.append(&mut carea.unknowns);
            }
        }

        for flow in flow_order {
            let mut merged = merged_careas.remove(&flow).unwrap();
            merged.rebuild_bbox();
            target.push(merged);
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

    pub fn cleanup_carea(&mut self, carea: usize) {
        if self.careas[carea].blocks.is_empty() {
            self.careas.remove(carea);
        } else {
            self.careas[carea].rebuild_bbox();
        }
    }
    pub fn cleanup_block(&mut self, carea: usize, block: usize) {
        if self.careas[carea].blocks[block].lines.is_empty() {
            if self.careas[carea].blocks[block].kind != HocrBlockKind::Image {
                self.careas[carea].blocks.remove(block);
            }
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
    pub fn merge_careas(&mut self, careas: &mut Vec<usize>) -> Result<(), String> {

        if careas.len() < 2 {
            return Err(format!("merge_careas: not enough careas: {:?}", careas));
        }

        careas.sort();
        careas.dedup();

        for i in 0..careas.len() - 1 {
            if careas[i]+1 != careas[i+1] {
                return Err(format!("merge_careas: careas not consecutive: {:?}", careas));
            }
            else {}
        }

        for i in (0..careas.len() - 1).rev() {
            self.merge_carea(careas[i], careas[i+1]);
        };

        self.cleanup_carea(careas[0]);

        Ok(())
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
    pub fn merge_blocks(&mut self, blocks: &mut Vec<(usize, usize)>) -> Result<(), String> {
        if blocks.len() < 2 {
            return Err(format!("merge_blocks: not enough blocks: {:?}", blocks));
        }

        blocks.sort();
        blocks.dedup();

        let docorder = self.careas.iter()
            .map(|c| c.blocks.iter()
                .map(|b| b.id.clone())
                .collect::<Vec<String>>())
            .collect::<Vec<Vec<String>>>()
            .into_iter()
            .flatten()
            .collect::<Vec<String>>();

        let block_ids = blocks.iter()
            .map(|(c, b)| self.careas[*c].blocks[*b].id.clone())
            .collect::<Vec<String>>();

        for i in 0..blocks.len() - 1 {
            let pos1 = docorder.iter().position(|id| id == &block_ids[i]).unwrap();
            let pos2 = docorder.iter().position(|id| id == &block_ids[i+1]).unwrap();
            if pos1+1 != pos2 {
                return Err(format!("merge_blocks: blocks not consecutive: {:?}", blocks));
            }
        }

        // We simply take out all blocks from the old position and insert in one go right after the first block.

        let mut moving_blocks = vec![];

        for (c,b) in blocks.iter().skip(1).rev() {
            moving_blocks.push(self.careas[*c].blocks.remove(*b));
        }

        moving_blocks.reverse();

        let (carea, block) = blocks[0];
        let insert_at = block + 1;

        self.careas[carea].blocks.splice(insert_at..insert_at, moving_blocks);

        // Now all blocks exist in one contiguous range in the first carea. Time to actually merge the blocks.
        for i in (0..blocks.len() - 1).rev() {

            self.merge_block(carea, block+i
                             , block+i+1);
        }

        let mut affected_careas: Vec<usize> = blocks.iter().map(|(c, _)| *c).collect();
        affected_careas.dedup();
        for c in affected_careas.into_iter().rev() {
            self.cleanup_carea(c);
        }

        Ok(())
    }
    pub fn merge_line(&mut self, carea: usize, block: usize, line1: usize, line2: usize) {
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

    pub fn merge_lines(&mut self, lines: &mut Vec<(usize, usize, usize)>) -> Result<(), String> {
        if lines.len() < 2 {
            return Err(format!("merge_lines: not enough lines: {:?}", lines));
        }

        lines.sort();
        lines.dedup();

        let docorder = self
            .careas
            .iter()
            .flat_map(|c| c.blocks.iter().flat_map(|b| b.lines.iter().map(|l| l.id.clone())))
            .collect::<Vec<String>>();

        let line_ids = lines
            .iter()
            .map(|(c, b, l)| self.careas[*c].blocks[*b].lines[*l].id.clone())
            .collect::<Vec<String>>();

        for i in 0..lines.len() - 1 {
            let pos1 = docorder.iter().position(|id| id == &line_ids[i]).unwrap();
            let pos2 = docorder.iter().position(|id| id == &line_ids[i + 1]).unwrap();
            if pos1 + 1 != pos2 {
                return Err(format!("merge_lines: lines not consecutive: {:?}", lines));
            }
        }

        let mut moving_lines = vec![];

        for (c, b, l) in lines.iter().skip(1).rev() {
            moving_lines.push(self.careas[*c].blocks[*b].lines.remove(*l));
        }

        moving_lines.reverse();

        let (carea, block, line) = lines[0];
        let insert_at = line + 1;

        self.careas[carea].blocks[block]
            .lines
            .splice(insert_at..insert_at, moving_lines);

        for i in (0..lines.len() - 1).rev() {
            self.merge_line(carea, block, line + i, line + i + 1);
        }

        let mut affected_blocks: Vec<(usize, usize)> =
            lines.iter().map(|(c, b, _)| (*c, *b)).collect();
        affected_blocks.dedup();
        for (c, b) in affected_blocks.into_iter().rev() {
            self.cleanup_block(c, b);
        }

        Ok(())
    }

    pub fn merge_word(
        &mut self,
        carea: usize,
        block: usize,
        line: usize,
        word1: usize,
        word2: usize,
    ) {
        if word1 != word2 {
            let w2 = self.careas[carea].blocks[block].lines[line]
                .words
                .remove(word2);
            let w1 = &mut self.careas[carea].blocks[block].lines[line].words[word1];

            w1.text = format!("{} {}", w1.text, w2.text);
            w1.bbox = w1.bbox.union(w2.bbox);
            w1.wconf = std::cmp::min(w1.wconf, w2.wconf);
            self.careas[carea].blocks[block].lines[line].rebuild_bbox();
        }
    }

    pub fn merge_words(
        &mut self,
        words: &mut Vec<(usize, usize, usize, usize)>,
    ) -> Result<(), String> {
        if words.len() < 2 {
            return Err(format!("merge_words: not enough words: {:?}", words));
        }

        words.sort();
        words.dedup();

        let docorder = self
            .careas
            .iter()
            .flat_map(|c| {
                c.blocks.iter().flat_map(|b| {
                    b.lines
                        .iter()
                        .flat_map(|l| l.words.iter().map(|w| w.id.clone()))
                })
            })
            .collect::<Vec<String>>();

        let word_ids = words
            .iter()
            .map(|(c, b, l, w)| {
                self.careas[*c].blocks[*b].lines[*l].words[*w]
                    .id
                    .clone()
            })
            .collect::<Vec<String>>();

        for i in 0..words.len() - 1 {
            let pos1 = docorder.iter().position(|id| id == &word_ids[i]).unwrap();
            let pos2 = docorder.iter().position(|id| id == &word_ids[i + 1]).unwrap();
            if pos1 + 1 != pos2 {
                return Err(format!("merge_words: words not consecutive: {:?}", words));
            }
        }

        let mut moving_words = vec![];

        for (c, b, l, w) in words.iter().skip(1).rev() {
            moving_words.push(self.careas[*c].blocks[*b].lines[*l].words.remove(*w));
        }

        moving_words.reverse();

        let (carea, block, line, word) = words[0];
        let insert_at = word + 1;

        self.careas[carea].blocks[block].lines[line]
            .words
            .splice(insert_at..insert_at, moving_words);

        for i in (0..words.len() - 1).rev() {
            self.merge_word(carea, block, line, word + i, word + i + 1);
        }

        let mut affected_lines: Vec<(usize, usize, usize)> =
            words.iter().map(|(c, b, l, _)| (*c, *b, *l)).collect();
        affected_lines.dedup();
        for (c, b, l) in affected_lines.into_iter().rev() {
            self.cleanup_line(c, b, l);
        }

        Ok(())
    }

    pub fn split_carea(&mut self, carea: usize, block_before: usize, block_after: usize) {
        if block_before == block_after {
            return;
        }

        let new_id = self.get_unique_id(self.careas[carea].id.as_str(), &mut HashMap::new());
        let (flow, layout) = {
            let old_carea = &self.careas[carea];
            (old_carea.flow.clone(), old_carea.layout.clone())
        };
        let old_carea = &mut self.careas[carea];
        let (_, right) = old_carea.blocks.split_at_mut(block_after);
        let new_carea = HocrCarea {
            level: "carea".to_string(),
            id: new_id,
            bbox: HocrBbox::empty(),
            flow,
            layout,
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
        let new_id = self.get_unique_id(self.careas[carea].blocks[block].id.as_str(), &mut HashMap::new());
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

    pub fn add_carea(&mut self, bbox: HocrBbox, erase_underneath: Option<bool>, erase_overlap: Option<u8>) -> Result<String, String> {

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

        let new_id = self.get_unique_id("carea", &mut HashMap::new());
        self.careas.insert(carea_index, HocrCarea {
            level: "carea".to_string(),
            id: new_id.clone(),
            bbox,
            flow: None,
            layout: None,
            blocks: vec![],
            unknowns: vec![],
        });

        self.rebuild_bbox();

        Ok(new_id)
    }
    pub fn add_block(&mut self, carea: Option<usize>, bbox: HocrBbox, block_type: Option<AddBlockType>, shrink_wrap_carea: Option<bool>, erase_underneath: Option<bool>, erase_overlap: Option<u8>) -> Result<String, String> {

        let new_id = match carea {

            None => {
                if erase_underneath.unwrap_or(false) && erase_overlap.is_some() {
                    let mut removals: HashMap<usize, Vec<String>> = HashMap::new();
                    let overlap_threshold = erase_overlap.unwrap();
                    for (c_idx, carea) in self.careas.iter().enumerate() {
                        let erase_block_ids: Vec<String> = carea.blocks
                            .iter()
                            .filter(|block| bbox.overlap_percentage(block.bbox).overlapping_other_pct as u8 >= overlap_threshold)
                            .map(|block| block.id.clone())
                            .collect();
                        if !erase_block_ids.is_empty() {
                            removals.insert(c_idx, erase_block_ids);
                        }
                    }

                    // Sort carea indices descending to avoid shifting issues when calling cleanup_carea
                    let mut affected_careas: Vec<usize> = removals.keys().cloned().collect();
                    affected_careas.sort_by(|a, b| b.cmp(a));

                    for c_idx in affected_careas {
                        if let Some(block_ids) = removals.get(&c_idx) {
                            self.careas[c_idx].blocks.retain(|block| !block_ids.contains(&block.id));
                            self.cleanup_carea(c_idx);
                        }
                    }
                }

                // Addition:
                // 1. Find the insertion index for a new carea based on the new block's vertical center.
                let vmid = bbox.center().1;
                let mut carea_index = None;
                for (i, carea) in self.careas.iter().enumerate() {
                    if carea.bbox.center().1 > vmid {
                        carea_index = Some(i);
                        break;
                    }
                }
                let carea_index = carea_index.unwrap_or(self.careas.len());

                // 2. Create a new HocrCarea with a unique ID and the new block's bbox.
                let new_carea_id = self.get_unique_id("carea", &mut HashMap::new());
                let new_block_id = self.get_unique_id("par", &mut HashMap::new());
                let block_kind = if block_type.unwrap_or(AddBlockType::Text) == AddBlockType::Image { HocrBlockKind::Image } else { HocrBlockKind::Paragraph };

                // 3. Create a new HocrBlock (Paragraph or Image kind) inside the new carea.
                let new_block = HocrBlock {
                    id: new_block_id.clone(),
                    level: "block".to_string(),
                    kind: block_kind,
                    lang: None,
                    bbox,
                    lines: vec![],
                };

                // 4. Insert the new carea into self.careas.
                self.careas.insert(carea_index, HocrCarea {
                    level: "carea".to_string(),
                    id: new_carea_id,
                    bbox,
                    flow: None,
                    layout: None,
                    blocks: vec![new_block],
                    unknowns: vec![],
                });

                Ok(new_block_id)
            }
            Some(carea) => {
                if erase_underneath.unwrap_or(false) && erase_overlap.is_some() {
                    let erase_block_ids: Vec<String> = self.careas[carea].blocks
                        .iter()
                        .filter(|block| bbox.overlap_percentage(block.bbox).overlapping_other_pct as u8 >= erase_overlap.unwrap())
                        .map(|block| block.id.clone())
                        .collect();

                    self.careas[carea].blocks.retain(|block| !erase_block_ids.contains(&block.id));
                    if !erase_block_ids.is_empty() {
                        self.cleanup_carea(carea);
                    }
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
                let new_id = self.get_unique_id("par", &mut HashMap::new());
                let block_kind = if block_type.unwrap_or(AddBlockType::Text) == AddBlockType::Image { HocrBlockKind::Image } else { HocrBlockKind::Paragraph };

                self.careas[carea].blocks.insert(block_index, HocrBlock {
                    id: new_id.clone(),
                    level: "block".to_string(),
                    kind: block_kind,
                    lang: None,
                    bbox,
                    lines: vec![],
                });

                if shrink_wrap_carea.unwrap_or(true) {
                    self.cleanup_carea(carea);
                };
                Ok(new_id)
            }
        };

        self.rebuild_bbox();

        new_id
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
        self.cleanup_carea(carea);
    }
    pub fn remove_line(&mut self, carea: usize, block: usize, line: usize) {
        self.careas[carea].blocks[block].lines.remove(line);
        self.cleanup_block(carea, block);
    }
    pub fn remove_word(&mut self, carea: usize, block: usize, line: usize, word: usize) {
        self.careas[carea].blocks[block].lines[line]
            .words
            .remove(word);
        self.cleanup_line(carea, block, line);
    }

    pub fn change_block_kind(&mut self, carea: usize, block: usize, kind: HocrBlockKind) {
        self.careas[carea].blocks[block].kind = kind;
        self.careas[carea].blocks[block].rebuild_bbox();
        self.cleanup_carea(carea);
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

    pub fn to_hocr_html(&self) -> String {
        let mut title = format!(
            "bbox {} {} {} {}",
            self.bbox.left(),
            self.bbox.top(),
            self.bbox.right(),
            self.bbox.bottom(),
        );

        if let Some(flow) = &self.flow {
            let flow_val = escape_attr(flow);
            if flow_val.is_empty() {
                title.push_str("; flow");
            } else {
                title.push_str(&format!("; flow {}", flow_val));
            }
        }

        if let Some(layout) = &self.layout {
            let layout_val = escape_attr(layout);
            if layout_val.is_empty() {
                title.push_str("; layout");
            } else {
                title.push_str(&format!("; layout {}", layout_val));
            }
        }

        let mut html = format!(
            "<div class=\"ocr_carea\" id=\"{}\" title=\"{}\">",
            escape_attr(&self.id),
            title
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

impl Display for HocrBlockKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            HocrBlockKind::Image => "IMG",
            HocrBlockKind::Paragraph => "P",
            HocrBlockKind::Part => "PART",
            HocrBlockKind::Chapter => "H1",
            HocrBlockKind::Section => "H2",
            HocrBlockKind::Subsection => "H3",
            HocrBlockKind::Subsubsection => "H4",
            HocrBlockKind::Subsubsubsection => "H5",
            HocrBlockKind::Subsubsubsubsection => "H6",
        };
        write!(f, "{}", code)
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

    pub fn to_hocr_html(&self) -> String {
        let lang_attr = self
            .lang
            .as_deref()
            .map(|lang| format!(" lang=\"{}\"", escape_attr(lang)))
            .unwrap_or_default();

        let mut html = format!(
            "<span class=\"ocr_line\" id=\"{}\" title=\"bbox {} {} {} {}; baseline {} {}; x_size {}; x_descenders {}; x_ascenders {}\"{}>",
            escape_attr(&self.id),
            self.bbox.left(),
            self.bbox.top(),
            self.bbox.right(),
            self.bbox.bottom(),
            self.baseline.unwrap_or((0.0, 0.0)).0,
            self.baseline.unwrap_or((0.0, 0.0)).1,
            self.x_size.unwrap_or(0.0),
            self.x_descenders.unwrap_or(0.0),
            self.x_ascenders.unwrap_or(0.0),
            lang_attr
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
    pub fn shift(&mut self, dx: i32, dy: i32) {
        self.bbox.shift(dx, dy);
    }

    pub fn cascade_lang(&mut self, default_lang: Option<&str>) {
        if self.lang.is_none() {
            self.lang = default_lang.map(|s| s.to_string());
        }
    }

    pub fn to_hocr_html(&self) -> String {
        let lang_attr = match &self.lang {
            Some(l) => format!(" lang=\"{}\"", escape_attr(l)),
            None => "".to_string(),
        };
        let dropcap_html = match &self.dropcap {
            Some(d) => format!("<span class=\"dropcap\">{}</span>", escape_text(d)),
            None => "".to_string(),
        };
        format!(
            "<span class=\"ocrx_word\" id=\"{}\" title=\"bbox {} {} {} {}; x_wconf {}\"{}>{}{}</span>",
            escape_attr(&self.id),
            self.bbox.left(),
            self.bbox.top(),
            self.bbox.right(),
            self.bbox.bottom(),
            self.wconf,
            lang_attr,
            dropcap_html,
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
    let sel_dropcap = Selector::parse("span.dropcap").ok()?;


    let page_el = document.select(&sel_page).next()?;
    let page_id = page_el.attr("id").unwrap_or("page_1").to_string();
    let page_bbox = bbox(page_el.attr("title").unwrap_or(""))?;

    let careas = page_el
        .select(&sel_carea)
        .filter_map(|carea_el| {
            let title_str = carea_el.attr("title").unwrap_or("");
            let carea_bbox = bbox(title_str)?;
            let carea_id = carea_el.attr("id").unwrap_or("").to_string();

            let title_keyvals = split_title(title_str);
            let flow = title_keyvals.get("flow").cloned();
            let layout = title_keyvals.get("layout").cloned();

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
                            let line_lang = line_el.attr("lang").map(str::to_string);

                            let words = line_el
                                .select(&sel_word)
                                .filter_map(|word_el| {
                                    let title = word_el.attr("title").unwrap_or("");
                                    let word_bbox = bbox(title)?;
                                    let dropcap = word_el.select(&sel_dropcap).next().map(|el| el.text().collect::<String>());
                                    let full_text = word_el.text().collect::<String>();
                                    let text = if let Some(ref d) = dropcap {
                                        if full_text.starts_with(d) {
                                            full_text[d.len()..].trim().to_string()
                                        } else {
                                            full_text.trim().to_string()
                                        }
                                    } else {
                                        full_text.trim().to_string()
                                    };
                                    Some(HocrWord {
                                        level: "word".to_string(),
                                        id: word_el.attr("id").unwrap_or("").to_string(),
                                        bbox: word_bbox,
                                        lang: word_el.attr("lang").map(str::to_string),
                                        text,
                                        wconf: wconf(title),
                                        dropcap,
                                    })
                                })
                                .collect();

                            Some(HocrLine {
                                level: "line".to_string(),
                                id: line_id,
                                bbox: line_bbox,
                                lang: line_lang,
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
                flow,
                layout,
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

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE1: &str = r#"
            <html>
                <body>
                    <div class='ocr_page' id='page_1' title='image "/tmp/side-014.jpg"; bbox 0 0 2537 3160; ppageno 0; scan_res 320 320'>
                       <div class='ocr_carea' id='block_1_1' title="bbox 483 286 1645 422">
                        <p class='ocr_par' id='par_1_1' lang='eng' title="bbox 483 286 1645 422">
                         <span class='ocr_line' id='line_1_1' title="bbox 483 286 1645 336; baseline -0.004 0; x_size 59.625; x_descenders 14.90625; x_ascenders 14.90625">
                          <span class='ocrx_word' id='word_1_1' title='bbox 483 290 586 336; x_wconf 96'>Line1-Word1</span>
                          <span class='ocrx_word' id='word_1_2' title='bbox 604 290 926 335; x_wconf 93'>Line1-Word2</span>
                          <span class='ocrx_word' id='word_1_3' title='bbox 951 290 1025 335; x_wconf 28'>Line1-Word3</span>
                          <span class='ocrx_word' id='word_1_4' title='bbox 1077 286 1645 334; x_wconf 91'>Line1-Word4</span>
                         </span>
                         <span class='ocr_line' id='line_1_2' title="bbox 483 361 1256 422; baseline -0.003 -14; x_size 59.625; x_descenders 14.90625; x_ascenders 14.90625">
                          <span class='ocrx_word' id='word_1_5' title='bbox 483 363 1005 408; x_wconf 96'>Line2-Word1</span>
                          <span class='ocrx_word' id='word_1_6' title='bbox 1024 361 1256 422; x_wconf 95'>Line2-Word2</span>
                         </span>
                        </p>
                       </div>
                    </div>
                </body>
            </html>
        "#;

    const SAMPLE2: &str = r#"
            <html>
                <body>
                    <div class='ocr_page' id='page_1' title='image "/tmp/side-014.jpg"; bbox 0 0 2537 3160; ppageno 0; scan_res 320 320'>

                       <div class='ocr_carea' id='block_1_1' title="bbox 483 286 1645 422">
                        <p class='ocr_par' id='par_1_1' lang='eng' title="bbox 483 286 1645 422">
                         <span class='ocr_line' id='line_1_1' title="bbox 483 286 1645 336; baseline -0.004 0; x_size 59.625; x_descenders 14.90625; x_ascenders 14.90625">
                          <span class='ocrx_word' id='word_1_1' title='bbox 483 290 586 336; x_wconf 96'>Line1-Word1</span>
                          <span class='ocrx_word' id='word_1_2' title='bbox 604 290 926 335; x_wconf 93'>Line1-Word2</span>
                          <span class='ocrx_word' id='word_1_3' title='bbox 951 290 1025 335; x_wconf 28'>Line1-Word3</span>
                          <span class='ocrx_word' id='word_1_4' title='bbox 1077 286 1645 334; x_wconf 91'>Line1-Word4</span>
                         </span>
                         <span class='ocr_line' id='line_1_2' title="bbox 483 361 1256 422; baseline -0.003 -14; x_size 59.625; x_descenders 14.90625; x_ascenders 14.90625">
                          <span class='ocrx_word' id='word_1_5' title='bbox 483 363 1005 408; x_wconf 96'>Line2-Word1</span>
                          <span class='ocrx_word' id='word_1_6' title='bbox 1024 361 1256 422; x_wconf 95'>Line2-Word2</span>
                         </span>
                        </p>
                       </div>

                       <div class='ocr_carea' id='block_1_2' title="bbox 485 440 2298 785">
                        <p class='ocr_par' id='par_1_2' lang='eng' title="bbox 485 440 2298 785">
                         <span class='ocr_line' id='line_1_3' title="bbox 485 440 2298 492; baseline -0.004 -11; x_size 46; x_descenders 11; x_ascenders 12">
                          <span class='ocrx_word' id='word_1_7' title='bbox 485 446 625 492; x_wconf 96'>Line3-Word1</span>
                          <span class='ocrx_word' id='word_1_8' title='bbox 642 446 702 480; x_wconf 96'>Line3-Word2</span>
                          <span class='ocrx_word' id='word_1_9' title='bbox 720 445 961 490; x_wconf 96'>Line3-Word3</span>
                         </span>

                         <span class='ocr_line' id='line_1_4' title="bbox 486 499 2295 549; baseline -0.004 -9; x_size 46; x_descenders 11; x_ascenders 12">
                          <span class='ocrx_word' id='word_1_10' title='bbox 486 505 632 540; x_wconf 91'>Line4-Word1</span>
                          <span class='ocrx_word' id='word_1_11' title='bbox 645 504 709 540; x_wconf 96'>Line4-Word2</span>
                          <span class='ocrx_word' id='word_1_12' title='bbox 723 505 842 539; x_wconf 96'>Line4-Word3</span>
                         </span>
                        </p>
                       </div>
                       </div>
                </body>
            </html>
        "#;

    const SAMPLE_3CAREAS: &str = r#"
            <html>
                <body>
                    <div class='ocr_page' id='page_1' title='bbox 0 0 1000 1000'>
                       <div class='ocr_carea' id='carea_1' title="bbox 10 10 100 100">
                         <p class='ocr_par' id='par_1' title="bbox 10 10 100 100">
                           <span class='ocr_line' id='line_1' title="bbox 10 10 100 50">
                             <span class='ocrx_word' id='word_1' title='bbox 10 10 50 50'>Word1</span>
                           </span>
                         </p>
                       </div>
                       <div class='ocr_carea' id='carea_2' title="bbox 110 110 200 200">
                         <p class='ocr_par' id='par_2' title="bbox 110 110 200 200">
                           <span class='ocr_line' id='line_2' title="bbox 110 110 200 150">
                             <span class='ocrx_word' id='word_2' title='bbox 110 110 150 150'>Word2</span>
                           </span>
                         </p>
                       </div>
                       <div class='ocr_carea' id='carea_3' title="bbox 210 210 300 300">
                         <p class='ocr_par' id='par_3' title="bbox 210 210 300 300">
                           <span class='ocr_line' id='line_3' title="bbox 210 210 300 250">
                             <span class='ocrx_word' id='word_3' title='bbox 210 210 250 250'>Word3</span>
                           </span>
                         </p>
                       </div>
                    </div>
                </body>
            </html>
    "#;

    const SAMPLE_COMPLEX: &str = r#"
            <html>
                <body>
                    <div class='ocr_page' id='page_1' title='bbox 0 0 1000 1000'>
                       <div class='ocr_carea' id='carea_1' title="bbox 10 10 100 100">
                         <p class='ocr_par' id='par_1' title="bbox 10 10 100 50">
                           <span class='ocr_line' id='line_1' title="bbox 10 10 100 30">
                             <span class='ocrx_word' id='word_1' title='bbox 10 10 50 30'>W1</span>
                           </span>
                         </p>
                         <p class='ocr_par' id='par_2' title="bbox 10 60 100 100">
                           <span class='ocr_line' id='line_2' title="bbox 10 60 100 80">
                             <span class='ocrx_word' id='word_2' title='bbox 10 60 50 80'>W2</span>
                           </span>
                         </p>
                       </div>
                       <div class='ocr_carea' id='carea_2' title="bbox 110 110 200 200">
                         <p class='ocr_par' id='par_3' title="bbox 110 110 200 150">
                           <span class='ocr_line' id='line_3' title="bbox 110 110 200 130">
                             <span class='ocrx_word' id='word_3' title='bbox 110 110 150 130'>W3</span>
                           </span>
                         </p>
                         <p class='ocr_par' id='par_4' title="bbox 110 160 200 200">
                           <span class='ocr_line' id='line_4' title="bbox 110 160 200 180">
                             <span class='ocrx_word' id='word_4' title='bbox 110 160 150 180'>W4</span>
                           </span>
                         </p>
                       </div>
                    </div>
                </body>
            </html>
    "#;


    #[test]
    fn stem_from_id_removed_trailing_digits() {
        assert_eq!(stem_from_id(""), "");
        assert_eq!(stem_from_id("page_1"), "page");
        assert_eq!(stem_from_id("line_1_100"), "line_1");
        assert_eq!(stem_from_id("page_1_shouldnotbe"), "page_1_shouldnotbe");
        assert_eq!(stem_from_id("id___4"), "id");
        assert_eq!(stem_from_id("id__1__4"), "id__1");
    }

    #[test]
    fn count_from_id_correct_number() {
        assert_eq!(count_from_id("page_1"), Ok(1));
        assert_eq!(count_from_id("line_1_100"), Ok(100));
        assert!(count_from_id("linebad").is_err());
    }

    #[test]
    fn next_unique_id() {

        let page = parse(SAMPLE1).unwrap();

        assert_eq!(page.page_id, "page_1");
        assert_eq!(page.get_next_number_with_stem("block"), Some(2));
        assert_eq!(page.get_next_number_with_stem("par"), Some(2));
        assert_eq!(page.get_next_number_with_stem("line"), Some(3));
        assert_eq!(page.get_next_number_with_stem("word"), Some(7));
        assert_eq!(page.get_unique_id("par_1_1", &mut HashMap::new()), "par_1_2");
        assert_eq!(page.get_unique_id("line_1_2", &mut HashMap::new()), "line_1_3");
        assert_eq!(page.get_unique_id("word_1_1", &mut HashMap::new()), "word_1_7");
    }

    #[test]
    fn signatures_match() {
        let page = parse(SAMPLE1).unwrap();
        let sig = signature(&page);
        assert_eq!(sig, to_sig(r#"page_1(
            block_1_1(
                par_1_1:P(
                    line_1_1(word_1_1, word_1_2, word_1_3, word_1_4),
                    line_1_2(word_1_5, word_1_6)))
        )"#));

        let page = parse(SAMPLE2).unwrap();
        let sig = signature(&page);
        assert_eq!(sig, to_sig(r#"page_1(
            block_1_1(
                par_1_1:P(
                    line_1_1(word_1_1, word_1_2, word_1_3, word_1_4),
                    line_1_2(word_1_5, word_1_6)
                )
            ),
            block_1_2(
                par_1_2:P(
                    line_1_3(word_1_7, word_1_8, word_1_9),
                    line_1_4(word_1_10, word_1_11, word_1_12)
                )
            )
        )"#));
    }

    #[test]
    fn locating_items() {
        let page = parse(SAMPLE2).unwrap();
        assert_eq!(find_node(&page, "line_1_1"), Some(HocrPath::Line {carea:0, block:0, line:0}))
    }

    #[test]
    fn move_line_up() {
        let mut page = parse(SAMPLE2).unwrap();
        let orig_sig = signature(&page);
        page.move_line_up(0,0,0);
        assert_eq!(signature(&page), orig_sig);

        page.move_line_up(1,0,0);

        assert_eq!(signature(&page), to_sig(r#"page_1(
            block_1_1(
                par_1_1:P(
                    line_1_1(word_1_1, word_1_2, word_1_3, word_1_4),
                    line_1_2(word_1_5, word_1_6),
                    line_1_3(word_1_7, word_1_8, word_1_9)
                )
            ),
            block_1_2(
                par_1_2:P(
                    line_1_4(word_1_10, word_1_11, word_1_12)
                )
            )
        )"#));

    }

    #[test]
    fn move_line_down() {
        let mut page = parse(SAMPLE2).unwrap();

        // Move line_1_1 down within the same block
        page.move_line_down(0, 0, 0);
        assert_eq!(signature(&page), to_sig(r#"page_1(
            block_1_1(
                par_1_1:P(
                    line_1_2(word_1_5, word_1_6),
                    line_1_1(word_1_1, word_1_2, word_1_3, word_1_4)
                )
            ),
            block_1_2(
                par_1_2:P(
                    line_1_3(word_1_7, word_1_8, word_1_9),
                    line_1_4(word_1_10, word_1_11, word_1_12)
                )
            )
        )"#));

        // Reset page
        let mut page = parse(SAMPLE2).unwrap();

        // Move line_1_2 down to next block
        page.move_line_down(0, 0, 1);
        assert_eq!(signature(&page), to_sig(r#"page_1(
            block_1_1(
                par_1_1:P(
                    line_1_1(word_1_1, word_1_2, word_1_3, word_1_4)
                )
            ),
            block_1_2(
                par_1_2:P(
                    line_1_2(word_1_5, word_1_6),
                    line_1_3(word_1_7, word_1_8, word_1_9),
                    line_1_4(word_1_10, word_1_11, word_1_12)
                )
            )
        )"#));
    }

    #[test]
    fn merge_carea() {
        let mut page = parse(SAMPLE2).unwrap();

        // Merge block_1_2 into block_1_1 (carea 1 into carea 0)
        page.merge_carea(0, 1);
        assert_eq!(signature(&page), to_sig(r#"page_1(
            block_1_1(
                par_1_1:P(
                    line_1_1(word_1_1, word_1_2, word_1_3, word_1_4),
                    line_1_2(word_1_5, word_1_6)
                ),
                par_1_2:P(
                    line_1_3(word_1_7, word_1_8, word_1_9),
                    line_1_4(word_1_10, word_1_11, word_1_12)
                )
            )
        )"#));
    }

    #[test]
    fn merge_block() {
        let mut page = parse(SAMPLE2).unwrap();
        // First merge careas so we have two blocks in one carea
        page.merge_carea(0, 1);

        // Now merge par_1_2 into par_1_1 (block 1 into block 0)
        page.merge_block(0, 0, 1);
        assert_eq!(signature(&page), to_sig(r#"page_1(
            block_1_1(
                par_1_1:P(
                    line_1_1(word_1_1, word_1_2, word_1_3, word_1_4),
                    line_1_2(word_1_5, word_1_6),
                    line_1_3(word_1_7, word_1_8, word_1_9),
                    line_1_4(word_1_10, word_1_11, word_1_12)
                )
            )
        )"#));
    }

    #[test]
    fn test_merge_careas_success() {
        // Test 2 careas
        let mut page = parse(SAMPLE_3CAREAS).unwrap();
        page.merge_careas(&mut vec![0, 1]).unwrap();
        assert_eq!(signature(&page), to_sig(r#"page_1(
            carea_1(par_1:P(line_1(word_1)),par_2:P(line_2(word_2))),
            carea_3(par_3:P(line_3(word_3)))
        )"#));

        // Test 3 careas
        let mut page = parse(SAMPLE_3CAREAS).unwrap();
        page.merge_careas(&mut vec![0, 1, 2]).unwrap();
        assert_eq!(signature(&page), to_sig(r#"page_1(
            carea_1(par_1:P(line_1(word_1)),par_2:P(line_2(word_2)),par_3:P(line_3(word_3)))
        )"#));
    }

    #[test]
    fn test_merge_careas_failures() {
        let mut page = parse(SAMPLE_3CAREAS).unwrap();
        
        // Less than 2
        assert!(page.merge_careas(&mut vec![0]).is_err());
        
        // Non-consecutive
        assert!(page.merge_careas(&mut vec![0, 2]).is_err());
    }

    #[test]
    fn test_merge_blocks_success() {
        // Test 2 blocks in same carea
        let mut page = parse(SAMPLE_COMPLEX).unwrap();
        let orig_sig = signature(&page);
        page.merge_blocks(&mut vec![(0, 0), (0, 1)]).unwrap();
        assert_eq!(signature(&page), to_sig(r#"page_1(
            carea_1(par_1:P(line_1(word_1),line_2(word_2))),
            carea_2(par_3:P(line_3(word_3)),par_4:P(line_4(word_4)))
        )"#));

        // Test 2 blocks across careas
        let mut page = parse(SAMPLE_COMPLEX).unwrap();
        page.merge_blocks(&mut vec![(0, 1), (1, 0)]).unwrap();
        assert_eq!(signature(&page), to_sig(r#"page_1(
            carea_1(par_1:P(line_1(word_1)),par_2:P(line_2(word_2),line_3(word_3))),
            carea_2(par_4:P(line_4(word_4)))
        )"#));

        // Test 3 blocks
        let mut page = parse(SAMPLE_COMPLEX).unwrap();
        page.merge_blocks(&mut vec![(0, 0), (0, 1), (1, 0)]).unwrap();
        assert_eq!(signature(&page), to_sig(r#"page_1(
            carea_1(par_1:P(line_1(word_1),line_2(word_2),line_3(word_3))),
            carea_2(par_4:P(line_4(word_4)))
        )"#));
    }

    #[test]
    fn test_merge_blocks_failures() {
        let mut page = parse(SAMPLE_COMPLEX).unwrap();
        
        // Less than 2
        assert!(page.merge_blocks(&mut vec![(0, 0)]).is_err());
        
        // Non-consecutive
        assert!(page.merge_blocks(&mut vec![(0, 0), (1, 0)]).is_err());
    }

    #[test]
    fn test_merge_blocks_cleanup() {
        let mut page = parse(SAMPLE_COMPLEX).unwrap();
        
        // Merge all blocks from carea_2 into carea_1
        page.merge_blocks(&mut vec![(0, 1), (1, 0), (1, 1)]).unwrap();
        
        // carea_2 should be removed because it's empty
        assert_eq!(signature(&page), to_sig(r#"page_1(
            carea_1(par_1:P(line_1(word_1)),par_2:P(line_2(word_2),line_3(word_3),line_4(word_4)))
        )"#));
        assert_eq!(page.careas.len(), 1);
    }

    #[test]
    fn add_block_none_carea_no_erase() {
        let mut page = parse(SAMPLE1).unwrap();
        let bbox = HocrBbox([483, 500, 1645, 600]);
        page.add_block(None, bbox, Some(AddBlockType::Text), None, Some(false), None);

        // Should have 2 careas now
        assert_eq!(page.careas.len(), 2);
        // The new carea should be at index 1 because its center Y (550) is greater than block_1_1 center Y (~354)
        assert_eq!(page.careas[1].bbox, bbox);
        assert_eq!(page.careas[1].blocks.len(), 1);
        assert_eq!(page.careas[1].blocks[0].bbox, bbox);
    }

    #[test]
    fn add_block_none_carea_with_erase() {
        let mut page = parse(SAMPLE1).unwrap();
        // This bbox overlaps with both lines of block_1_1
        let bbox = HocrBbox([483, 280, 1645, 430]);
        page.add_block(None, bbox, Some(AddBlockType::Text), None, Some(true), Some(50));

        // block_1_1 should have been erased because its blocks overlap with the new bbox
        // Since all blocks in block_1_1 are erased, block_1_1 itself should be removed
        assert_eq!(page.careas.len(), 1);
        assert_eq!(page.careas[0].bbox, bbox);
        assert_eq!(page.careas[0].blocks.len(), 1);
        assert_eq!(page.careas[0].id.starts_with("carea"), true);
    }

    #[test]
    fn add_block_none_carea_vertical_positioning() {
        let mut page = parse(SAMPLE1).unwrap();
        // Add one above
        let bbox_above = HocrBbox([483, 100, 1645, 200]);
        page.add_block(None, bbox_above, None, None, None, None);

        // Add one below
        let bbox_below = HocrBbox([483, 500, 1645, 600]);
        page.add_block(None, bbox_below, None, None, None, None);

        assert_eq!(page.careas.len(), 3);
        assert_eq!(page.careas[0].bbox, bbox_above);
        assert_eq!(page.careas[2].bbox, bbox_below);
    }

    #[test]
    fn add_block_with_shrink_wrap_false() {
        let mut page = parse(SAMPLE1).unwrap();
        let original_carea_bbox = page.careas[0].bbox;

        // Add a block that is outside the current carea bbox
        let new_block_bbox = HocrBbox([original_carea_bbox.0[2] + 10, original_carea_bbox.0[1], original_carea_bbox.0[2] + 100, original_carea_bbox.0[3]]);

        // Add to carea 0 with shrink_wrap_carea = false
        page.add_block(Some(0), new_block_bbox, None, Some(false), None, None);

        // Carea bbox should remain the same
        assert_eq!(page.careas[0].bbox, original_carea_bbox);
        // Block should be added
        assert_eq!(page.careas[0].blocks.len(), 2);
    }

    #[test]
    fn add_block_with_shrink_wrap_true() {
        let mut page = parse(SAMPLE1).unwrap();
        let original_carea_bbox = page.careas[0].bbox;

        // Add a block that is outside the current carea bbox
        let new_block_bbox = HocrBbox([original_carea_bbox.0[2] + 10, original_carea_bbox.0[1], original_carea_bbox.0[2] + 100, original_carea_bbox.0[3]]);

        // Add to carea 0 with shrink_wrap_carea = true (default)
        page.add_block(Some(0), new_block_bbox, None, Some(true), None, None);

        // Carea bbox should have changed (it should now include the new block)
        assert_ne!(page.careas[0].bbox, original_carea_bbox);
        assert_eq!(page.careas[0].blocks.len(), 2);
    }

    #[test]
    fn test_coordinate_shifting() {
        let mut page = parse(SAMPLE1).unwrap();
        let dx = 100;
        let dy = 200;

        let original_page_bbox = page.bbox;
        let original_carea_bbox = page.careas[0].bbox;
        let original_block_bbox = page.careas[0].blocks[0].bbox;
        let original_line_bbox = page.careas[0].blocks[0].lines[0].bbox;
        let original_word_bbox = page.careas[0].blocks[0].lines[0].words[0].bbox;

        page.shift(dx, dy);

        assert_eq!(page.bbox.0, [original_page_bbox.0[0] + dx, original_page_bbox.0[1] + dy, original_page_bbox.0[2] + dx, original_page_bbox.0[3] + dy]);
        assert_eq!(page.careas[0].bbox.0, [original_carea_bbox.0[0] + dx, original_carea_bbox.0[1] + dy, original_carea_bbox.0[2] + dx, original_carea_bbox.0[3] + dy]);
        assert_eq!(page.careas[0].blocks[0].bbox.0, [original_block_bbox.0[0] + dx, original_block_bbox.0[1] + dy, original_block_bbox.0[2] + dx, original_block_bbox.0[3] + dy]);
        assert_eq!(page.careas[0].blocks[0].lines[0].bbox.0, [original_line_bbox.0[0] + dx, original_line_bbox.0[1] + dy, original_line_bbox.0[2] + dx, original_line_bbox.0[3] + dy]);
        assert_eq!(page.careas[0].blocks[0].lines[0].words[0].bbox.0, [original_word_bbox.0[0] + dx, original_word_bbox.0[1] + dy, original_word_bbox.0[2] + dx, original_word_bbox.0[3] + dy]);
    }

    #[test]
    fn test_insert_careas_after() {
        let mut page = parse(SAMPLE_3CAREAS).unwrap();
        let new_carea = page.careas[0].clone();
        let mut new_careas = vec![new_carea];
        new_careas[0].id = "new_carea".to_string();

        page.insert_careas_after(0, new_careas);

        assert_eq!(page.careas.len(), 4);
        assert_eq!(page.careas[1].id, "new_carea");
        assert_eq!(page.careas[0].id, "carea_1");
        assert_eq!(page.careas[2].id, "carea_2");
    }

    #[test]
    fn test_merge_lines_success() {
        let mut page = parse(SAMPLE2).unwrap();
        // par_1_1 has line_1_1 and line_1_2.
        // indices for par_1_1: carea 0, block 0.
        // lines: (0, 0, 0) and (0, 0, 1).

        let mut lines = vec![(0, 0, 0), (0, 0, 1)];
        page.merge_lines(&mut lines).unwrap();

        assert_eq!(page.careas[0].blocks[0].lines.len(), 1);
        // line_1_1 had 4 words, line_1_2 had 2 words. Total 6.
        assert_eq!(page.careas[0].blocks[0].lines[0].words.len(), 6);
    }

    #[test]
    fn test_merge_words_success() {
        let mut page = parse(SAMPLE2).unwrap();
        // line_1_1 has words 1, 2, 3, 4.
        // indices: (0, 0, 0, 0), (0, 0, 0, 1).

        let mut words = vec![(0, 0, 0, 0), (0, 0, 0, 1)];
        page.merge_words(&mut words).unwrap();

        assert_eq!(page.careas[0].blocks[0].lines[0].words.len(), 3);
        assert_eq!(
            page.careas[0].blocks[0].lines[0].words[0].text,
            "Line1-Word1 Line1-Word2"
        );
    }

    #[test]
    fn test_merge_words_across_lines() {
        let mut page = parse(SAMPLE2).unwrap();
        // word_1_4 (last of line 1) and word_1_5 (first of line 2) are consecutive.
        // word_1_4: (0, 0, 0, 3)
        // word_1_5: (0, 0, 1, 0)

        let mut words = vec![(0, 0, 0, 3), (0, 0, 1, 0)];
        page.merge_words(&mut words).unwrap();

        assert_eq!(page.careas[0].blocks[0].lines[0].words.len(), 4);
        assert_eq!(
            page.careas[0].blocks[0].lines[0].words[3].text,
            "Line1-Word4 Line2-Word1"
        );
        // Line 2 should now have only 1 word remaining (word_1_6)
        assert_eq!(page.careas[0].blocks[0].lines[1].words.len(), 1);
    }

    #[test]
    fn test_merge_words_cleanup_line() {
        let mut page = parse(SAMPLE2).unwrap();
        // Merge all words of line_1_2 into line_1_1
        // line_1_2 words: word_1_5 (0,0,1,0), word_1_6 (0,0,1,1)
        // line_1_1 words: word_1_1 (0,0,0,0) to word_1_4 (0,0,0,3)

        let mut words = vec![(0, 0, 0, 3), (0, 0, 1, 0), (0, 0, 1, 1)];
        page.merge_words(&mut words).unwrap();

        assert_eq!(page.careas[0].blocks[0].lines[0].words.len(), 4);
        // line_1_2 should have been removed because all its words were moved
        assert_eq!(page.careas[0].blocks[0].lines.len(), 1);
    }

    #[test]
    fn test_carea_metadata_parsing() {
        let html = r#"
            <div class="ocr_page" id="page_1" title="bbox 0 0 1000 1000">
                <div class="ocr_carea" id="carea_1" title="bbox 10 10 100 100; flow footnotes; layout center">
                </div>
                <div class="ocr_carea" id="carea_2" title="bbox 110 110 200 200; flow; layout">
                </div>
            </div>
        "#;
        let page = parse(html).unwrap();
        assert_eq!(page.careas[0].flow, Some("footnotes".to_string()));
        assert_eq!(page.careas[0].layout, Some("center".to_string()));
        assert_eq!(page.careas[1].flow, Some("".to_string()));
        assert_eq!(page.careas[1].layout, Some("".to_string()));
    }

    #[test]
    fn test_carea_metadata_serialization() {
        let carea = HocrCarea {
            level: "carea".to_string(),
            id: "carea_1".to_string(),
            bbox: HocrBbox::new(10, 10, 100, 100),
            flow: Some("main".to_string()),
            layout: Some("left".to_string()),
            blocks: vec![],
            unknowns: vec![],
        };
        let html = carea.to_hocr_html();
        assert!(html.contains("flow main"));
        assert!(html.contains("layout left"));

        let carea_empty = HocrCarea {
            level: "carea".to_string(),
            id: "carea_2".to_string(),
            bbox: HocrBbox::new(10, 10, 100, 100),
            flow: Some("".to_string()),
            layout: Some("".to_string()),
            blocks: vec![],
            unknowns: vec![],
        };
        let html_empty = carea_empty.to_hocr_html();
        assert!(html_empty.contains("; flow"));
        assert!(!html_empty.contains("flow "));
        assert!(html_empty.contains("; layout"));
        assert!(!html_empty.contains("layout "));
    }

    #[test]
    fn test_carea_metadata_defaulting() {
        let html = r#"
            <div class="ocr_page" id="page_1" title="bbox 0 0 1000 1000">
                <div class="ocr_carea" id="carea_1" title="bbox 10 10 100 100">
                </div>
            </div>
        "#;
        let page = parse(html).unwrap();
        assert_eq!(page.careas[0].flow, None);
        assert_eq!(page.careas[0].layout, None);
    }

    #[test]
    fn test_split_carea_metadata_preservation() {
        let carea = HocrCarea {
            level: "carea".to_string(),
            id: "carea_1".to_string(),
            bbox: HocrBbox::new(0, 0, 100, 200),
            flow: Some("special".to_string()),
            layout: Some("right".to_string()),
            blocks: vec![
                HocrBlock {
                    level: "block".to_string(),
                    id: "b1".to_string(),
                    bbox: HocrBbox::new(0, 0, 100, 100),
                    kind: HocrBlockKind::Paragraph,
                    lang: None,
                    lines: vec![],
                },
                HocrBlock {
                    level: "block".to_string(),
                    id: "b2".to_string(),
                    bbox: HocrBbox::new(0, 101, 100, 200),
                    kind: HocrBlockKind::Paragraph,
                    lang: None,
                    lines: vec![],
                },
            ],
            unknowns: vec![],
        };
        let mut page = HocrPage {
            level: "page".to_string(),
            page_id: "p1".to_string(),
            bbox: HocrBbox::new(0, 0, 1000, 1000),
            careas: vec![carea],
            unknowns: vec![],
        };

        page.split_carea(0, 0, 1);

        assert_eq!(page.careas.len(), 2);
        assert_eq!(page.careas[0].flow, Some("special".to_string()));
        assert_eq!(page.careas[0].layout, Some("right".to_string()));
        assert_eq!(page.careas[1].flow, Some("special".to_string()));
        assert_eq!(page.careas[1].layout, Some("right".to_string()));
    }

    #[test]
    fn test_add_carea_metadata_initialization() {
        let mut page = HocrPage {
            level: "page".to_string(),
            page_id: "p1".to_string(),
            bbox: HocrBbox::new(0, 0, 1000, 1000),
            careas: vec![],
            unknowns: vec![],
        };
        page.add_carea(HocrBbox::new(10, 10, 100, 100), None, None).unwrap();
        assert_eq!(page.careas[0].flow, None);
        assert_eq!(page.careas[0].layout, None);
    }

    #[test]
    fn test_add_block_new_carea_metadata_initialization() {
        let mut page = HocrPage {
            level: "page".to_string(),
            page_id: "p1".to_string(),
            bbox: HocrBbox::new(0, 0, 1000, 1000),
            careas: vec![],
            unknowns: vec![],
        };
        page.add_block(None, HocrBbox::new(10, 10, 100, 100), None, None, None, None).unwrap();
        assert_eq!(page.careas[0].flow, None);
        assert_eq!(page.careas[0].layout, None);
    }

    #[test]
    fn test_auto_layout_default_flow() {
        let mut page = HocrPage {
            level: "page".to_string(),
            page_id: "p1".to_string(),
            bbox: HocrBbox::new(0, 0, 100, 100),
            careas: vec![
                HocrCarea {
                    level: "carea".to_string(),
                    id: "c1".to_string(),
                    bbox: HocrBbox::new(0, 0, 10, 10),
                    flow: None,
                    layout: None,
                    blocks: vec![],
                    unknowns: vec![],
                },
                HocrCarea {
                    level: "carea".to_string(),
                    id: "c2".to_string(),
                    bbox: HocrBbox::new(10, 10, 20, 20),
                    flow: Some("existing".to_string()),
                    layout: None,
                    blocks: vec![],
                    unknowns: vec![],
                },
            ],
            unknowns: vec![],
        };

        let flows = vec![FlowSchema { name: "default".to_string(), color: Some(ColorSpecification::default()) }];
        page.auto_flow(flows, vec![], true);

        assert_eq!(page.careas[0].flow, Some("default".to_string()));
        assert_eq!(page.careas[1].flow, Some("existing".to_string()));
    }

    #[test]
    fn test_auto_layout_merging() {
        let mut page = HocrPage {
            level: "page".to_string(),
            page_id: "p1".to_string(),
            bbox: HocrBbox::new(0, 0, 100, 100),
            careas: vec![
                HocrCarea {
                    level: "carea".to_string(),
                    id: "c1".to_string(),
                    bbox: HocrBbox::new(0, 0, 10, 10),
                    flow: Some("F1".to_string()),
                    layout: Some("L1".to_string()),
                    blocks: vec![HocrBlock {
                        level: "block".to_string(),
                        id: "b1".to_string(),
                        bbox: HocrBbox::new(0, 0, 10, 10),
                        kind: HocrBlockKind::Paragraph,
                        lang: None,
                        lines: vec![],
                    }],
                    unknowns: vec![],
                },
                HocrCarea {
                    level: "carea".to_string(),
                    id: "c2".to_string(),
                    bbox: HocrBbox::new(20, 20, 30, 30),
                    flow: Some("F2".to_string()),
                    layout: Some("L1".to_string()),
                    blocks: vec![HocrBlock {
                        level: "block".to_string(),
                        id: "b2".to_string(),
                        bbox: HocrBbox::new(20, 20, 30, 30),
                        kind: HocrBlockKind::Paragraph,
                        lang: None,
                        lines: vec![],
                    }],
                    unknowns: vec![],
                },
                HocrCarea {
                    level: "carea".to_string(),
                    id: "c3".to_string(),
                    bbox: HocrBbox::new(40, 40, 50, 50),
                    flow: Some("F1".to_string()),
                    layout: Some("L1".to_string()),
                    blocks: vec![HocrBlock {
                        level: "block".to_string(),
                        id: "b3".to_string(),
                        bbox: HocrBbox::new(40, 40, 50, 50),
                        kind: HocrBlockKind::Paragraph,
                        lang: None,
                        lines: vec![],
                    }],
                    unknowns: vec![],
                },
            ],
            unknowns: vec![],
        };

        let flows = vec![
            FlowSchema { name: "F1".to_string(), color: Some(ColorSpecification::default()) },
            FlowSchema { name: "F2".to_string(), color: Some(ColorSpecification::default()) },
        ];
        page.auto_flow(flows, vec![], true);

        // Result should be 2 careas in Group L1
        assert_eq!(page.careas.len(), 2);
        
        // First carea should be F1 (c1 + c3)
        assert_eq!(page.careas[0].flow, Some("F1".to_string()));
        assert_eq!(page.careas[0].blocks.len(), 2);
        assert_eq!(page.careas[0].blocks[0].id, "b1");
        assert_eq!(page.careas[0].blocks[1].id, "b3");
        
        // Second carea should be F2 (c2)
        assert_eq!(page.careas[1].flow, Some("F2".to_string()));
        assert_eq!(page.careas[1].blocks.len(), 1);
        assert_eq!(page.careas[1].blocks[0].id, "b2");
    }

    #[test]
    fn test_auto_layout_consecutive_layout_grouping() {
        let mut page = HocrPage {
            level: "page".to_string(),
            page_id: "p1".to_string(),
            bbox: HocrBbox::new(0, 0, 100, 100),
            careas: vec![
                HocrCarea {
                    level: "carea".to_string(),
                    id: "c1".to_string(),
                    bbox: HocrBbox::new(0, 0, 10, 10),
                    flow: Some("F1".to_string()),
                    layout: Some("L1".to_string()),
                    blocks: vec![HocrBlock { id: "b1".to_string(), level: "block".to_string(), bbox: HocrBbox::new(0, 0, 10, 10), kind: HocrBlockKind::Paragraph, lang: None, lines: vec![] }],
                    unknowns: vec![],
                },
                HocrCarea {
                    level: "carea".to_string(),
                    id: "c2".to_string(),
                    bbox: HocrBbox::new(20, 20, 30, 30),
                    flow: Some("F1".to_string()),
                    layout: Some("L2".to_string()),
                    blocks: vec![HocrBlock { id: "b2".to_string(), level: "block".to_string(), bbox: HocrBbox::new(20, 20, 30, 30), kind: HocrBlockKind::Paragraph, lang: None, lines: vec![] }],
                    unknowns: vec![],
                },
                HocrCarea {
                    level: "carea".to_string(),
                    id: "c3".to_string(),
                    bbox: HocrBbox::new(40, 40, 50, 50),
                    flow: Some("F1".to_string()),
                    layout: Some("L1".to_string()),
                    blocks: vec![HocrBlock { id: "b3".to_string(), level: "block".to_string(), bbox: HocrBbox::new(40, 40, 50, 50), kind: HocrBlockKind::Paragraph, lang: None, lines: vec![] }],
                    unknowns: vec![],
                },
            ],
            unknowns: vec![],
        };

        let flows = vec![FlowSchema { name: "F1".to_string(), color: Some(ColorSpecification::default()) }];
        page.auto_flow(flows, vec![], true);

        // Result should be 3 careas because L1 is interrupted by L2
        assert_eq!(page.careas.len(), 3);
        assert_eq!(page.careas[0].layout, Some("L1".to_string()));
        assert_eq!(page.careas[1].layout, Some("L2".to_string()));
        assert_eq!(page.careas[2].layout, Some("L1".to_string()));
    }

    #[test]
    fn test_replace_or_merge_carea_uniqueness() {
        let mut page = parse(SAMPLE_3CAREAS).unwrap();
        // The sample has careas with IDs: carea_1, carea_2, carea_3.
        // It has blocks (pars): par_1, par_2, par_3.
        // Lines: line_1, line_2, line_3.
        // Words: word_1, word_2, word_3.

        // Create a new carea that has overlapping IDs.
        let mut new_carea = page.careas[0].clone();
        // new_carea has id 'carea_1', block 'par_1', line 'line_1', word 'word_1'.

        let original_carea_count = page.careas.len();
        page.replace_or_merge_carea(0, vec![new_carea]);

        // It should have merged blocks into the first carea.
        assert_eq!(page.careas.len(), original_carea_count);

        // Now check for duplicates across the entire page
        let mut ids = std::collections::HashSet::new();
        for carea in &page.careas {
            assert!(ids.insert(carea.id.clone()), "Duplicate carea ID: {}", carea.id);
            for block in &carea.blocks {
                assert!(ids.insert(block.id.clone()), "Duplicate block ID: {}", block.id);
                for line in &block.lines {
                    assert!(ids.insert(line.id.clone()), "Duplicate line ID: {}", line.id);
                    for word in &line.words {
                        assert!(ids.insert(word.id.clone()), "Duplicate word ID: {}", word.id);
                    }
                }
            }
        }

        // Specifically verify that the merged block got a new ID.
        // The first carea should now have 2 blocks.
        assert_eq!(page.careas[0].blocks.len(), 2);
        // Original was 'par_1', new one should be 'par_4' because par_1, par_2, par_3 exist.
        assert_eq!(page.careas[0].blocks[0].id, "par_1");
        assert_eq!(page.careas[0].blocks[1].id, "par_4");
    }

    #[test]
    fn test_dropcap_roundtrip() {
        let html = r#"
            <span class="ocrx_word" id="word_1" title="bbox 10 10 50 50; x_wconf 90"><span class="dropcap">W</span>hen</span>
        "#;
        let full_html = format!(r#"
            <div class="ocr_page" id="page_1" title="bbox 0 0 100 100">
                <div class="ocr_carea" id="carea_1" title="bbox 0 0 100 100">
                    <p class="ocr_par" id="par_1" title="bbox 0 0 100 100">
                        <span class="ocr_line" id="line_1" title="bbox 0 0 100 100">
                            {}
                        </span>
                    </p>
                </div>
            </div>
        "#, html);

        let page = parse(&full_html).unwrap();
        let word = &page.careas[0].blocks[0].lines[0].words[0];
        assert_eq!(word.dropcap, Some("W".to_string()));
        assert_eq!(word.text, "hen".to_string());

        let rendered = word.to_hocr_html();
        assert!(rendered.contains(r#"<span class="dropcap">W</span>"#));
        assert!(rendered.contains("hen</span>"));
    }

    #[test]
    fn test_inject_dropcaps() {
        let mut page = parse(r#"
            <div class="ocr_page" id="page_1" title="bbox 0 0 1000 1000">
                <div class="ocr_carea" id="carea_1" title="bbox 100 100 500 500">
                    <p class="ocr_par" id="par_1" title="bbox 100 100 500 500">
                        <span class="ocr_line" id="line_1" title="bbox 110 105 400 130">
                            <span class="ocrx_word" id="word_1" title="bbox 110 105 200 130">hen</span>
                        </span>
                    </p>
                </div>
            </div>
        "#).unwrap();

        let injections = vec![DropCapInjection {
            text: "W".to_string(),
            bbox: HocrBbox([50, 100, 105, 200]),
        }];

        page.inject_dropcaps(injections);

        let word = &page.careas[0].blocks[0].lines[0].words[0];
        assert_eq!(word.dropcap, Some("W".to_string()));
        assert_eq!(word.text, "hen");
    }
}