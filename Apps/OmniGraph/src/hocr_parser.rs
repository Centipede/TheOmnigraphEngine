use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

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
pub type HocrBbox = [i32; 4];

pub fn bbox_union(bbox1: HocrBbox, bbox2: HocrBbox) -> HocrBbox {
    let mut bbox = bbox1;
    bbox[0] = bbox[0].min(bbox2[0]);
    bbox[1] = bbox[1].min(bbox2[1]);
    bbox[2] = bbox[2].max(bbox2[2]);
    bbox[3] = bbox[3].max(bbox2[3]);
    bbox
}

pub fn bbox_intersection(bbox1: HocrBbox, bbox2: HocrBbox) -> Option<HocrBbox> {
    let mut bbox = bbox1;
    bbox[0] = bbox[0].max(bbox2[0]);
    bbox[1] = bbox[1].max(bbox2[1]);
    bbox[2] = bbox[2].min(bbox2[2]);
    bbox[3] = bbox[3].min(bbox2[3]);
    if bbox[0] < bbox[2] && bbox[1] < bbox[3] {
        Some(bbox)
    } else {
        None
    }
}

pub fn bbox_union_all(bboxes: &[HocrBbox]) -> Option<HocrBbox> {
    if bboxes.is_empty() {
        return None;
    }
    let mut bbox = bboxes[0];
    for b in bboxes.iter().skip(1) {
        bbox = bbox_union(bbox, *b);
    }
    Some(bbox)
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrCarea {
    #[serde(default = "carea_level", skip_deserializing)]
    pub level: String,
    pub id: String,
    pub bbox: HocrBbox,
    pub blocks: Vec<HocrBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrPage {
    #[serde(default = "page_level", skip_deserializing)]
    pub level: String,
    pub page_id: String,
    pub bbox: HocrBbox,
    pub careas: Vec<HocrCarea>,
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

impl HocrPath {
    pub fn to_carea(&self) -> Option<HocrPath> {
        match self {
            HocrPath::Carea { carea } => Some(HocrPath::Carea { carea: *carea }),
            HocrPath::Block { carea, block } => Some(HocrPath::Carea { carea: *carea }),
            HocrPath::Line { carea, block, line } => Some(HocrPath::Carea { carea: *carea }),
            HocrPath::Word { carea, block, line, word } => Some(HocrPath::Carea { carea: *carea }),
            _ => None,
        }
    }

    pub fn to_block(&self) -> Option<HocrPath> {
        match self {
            HocrPath::Block { carea, block } => Some(HocrPath::Block { carea: *carea, block: *block }),
            HocrPath::Line { carea, block, .. } => Some(HocrPath::Block { carea: *carea, block: *block }),
            HocrPath::Word { carea, block, .. } => Some(HocrPath::Block { carea: *carea, block: *block }),
            _ => None,
        }
    }

    pub fn to_line(&self) -> Option<HocrPath> {
        match self {
            HocrPath::Line { carea, block, line } => Some(HocrPath::Line { carea: *carea, block: *block, line: *line }),
            HocrPath::Word { carea, block, line, .. } => Some(HocrPath::Line { carea: *carea, block: *block, line: *line }),
            _ => None,
        }
    }
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
            self.bbox[0],
            self.bbox[1],
            self.bbox[2],
            self.bbox[3],
        ));

        for carea in &self.careas {
            html.push_str(&carea.to_hocr_html());
        }

        html.push_str("</div>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        html
    }
    pub fn rebuild_bbox(&mut self) {
        let subboxes = self.careas.iter().map(|c| c.bbox).collect::<Vec<_>>();
        match bbox_union_all(&subboxes) {
            Some(union) => self.bbox = union,
            None => self.bbox = [0, 0, 0, 0],
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
        println!(
            "get_next_number_with_stem: numbers: {:?} preferred_stem {}  stem_upto_underscore: {}",
            numbers, preferred_stem, stem_upto_underscore
        );
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

    pub fn advance_carea_path(&self, path: HocrPath) -> Option<HocrPath> {
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
    pub fn advance_block_path(&self, path: HocrPath) -> Option<HocrPath> {
        if let HocrPath::Block { carea, block } = path {
            if block < self.careas[carea].blocks.len() - 1 {
                Some(HocrPath::Block {
                    carea,
                    block: block + 1,
                })
            } else if block == self.careas[carea].blocks.len() - 1 {
                loop {
                    let path = self.advance_carea_path(path.to_carea()?);
                    if let Some (HocrPath::Carea { carea }) = path {
                        if self.careas[carea].blocks.len() > 0 {
                            return Some(HocrPath::Block { carea, block: 0 });
                        }
                    }
                    else {
                        return None;
                    }
                }
                None
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn move_line_down(&mut self, carea: usize, block: usize, line: usize) {

        let block_size = self.careas[carea].blocks[block].lines.len();

        // Move within the same block?
        if line < block_size - 1 {
            println!("move_line_down - within same block: line {}", line);
            let line_down = self.careas[carea].blocks[block].lines.remove(line + 1);
            self.careas[carea].blocks[block]
                .lines
                .insert(line, line_down);
            self.careas[carea].blocks[block].rebuild_bbox();
        }
        // Move line to next block ... even it that requires skipping over empty blocks?
        else if line == block_size - 1 {
            let next_block = self.advance_block_path(HocrPath::Block { carea, block });
            println!("move_line_down: next_block {:?}", next_block);
            if let Some (HocrPath::Block { carea:to_carea, block:to_block }) = next_block {
                let line_down = self.careas[carea].blocks[block].lines.remove(line);
                println!(" prepending line to block {:?}: {} lines", next_block, self.careas[to_carea].blocks[to_block].lines.len() );
                self.careas[to_carea].blocks[to_block]
                    .lines
                    .insert(0, line_down);
                println!(" prepended line to block {:?}: {} lines", next_block, self.careas[to_carea].blocks[to_block].lines.len() );
                self.careas[carea].blocks[block].rebuild_bbox();
                self.careas[to_carea].blocks[to_block].rebuild_bbox();
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
        let (left, right) = old_carea.blocks.split_at_mut(block_after);
        let new_carea = HocrCarea {
            level: "carea".to_string(),
            id: new_id,
            bbox: [0, 0, 0, 0],
            blocks: right.to_vec(),
        };
        old_carea.blocks.truncate(block_after);
        self.careas.insert(carea + 1, new_carea);
        self.careas[carea].rebuild_bbox();
        self.careas[carea + 1].rebuild_bbox();
    }

    pub fn split_block(
        &mut self,
        carea: usize,
        block: usize,
        line_before: usize,
        line_after: usize,
    ) {
        if line_before == line_after {
            return;
        }
        let new_id = self.get_unique_id(self.careas[carea].blocks[block].id.as_str());
        let carea = &mut self.careas[carea];
        let old_block = &mut carea.blocks[block];
        let (left, right) = old_block.lines.split_at_mut(line_after);
        let new_block = HocrBlock {
            lang: old_block.lang.clone(),
            kind: HocrBlockKind::Paragraph,
            level: "block".to_string(),
            id: new_id,
            bbox: [0, 0, 0, 0],
            lines: right.to_vec(),
        };
        old_block.lines.truncate(line_after);
        carea.blocks.insert(block + 1, new_block);
        carea.blocks[block].rebuild_bbox();
        carea.blocks[block + 1].rebuild_bbox();
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
}

impl HocrCarea {
    pub fn to_hocr_html(&self) -> String {
        let mut html = format!(
            "<div class=\"ocr_carea\" id=\"{}\" title=\"bbox {} {} {} {}\">",
            escape_attr(&self.id),
            self.bbox[0],
            self.bbox[1],
            self.bbox[2],
            self.bbox[3],
        );

        for block in &self.blocks {
            html.push_str(&block.to_hocr_html());
        }

        html.push_str("</div>");
        html
    }
    pub fn rebuild_bbox(&mut self) {
        let subboxes = self.blocks.iter().map(|b| b.bbox).collect::<Vec<_>>();
        match bbox_union_all(&subboxes) {
            Some(union) => self.bbox = union,
            None => self.bbox = [0, 0, 0, 0],
        }
    }
}

impl HocrBlockKind {
    pub fn tag_name(self) -> &'static str {
        match self {
            HocrBlockKind::Paragraph => "p",
            HocrBlockKind::Part => "h1",
            HocrBlockKind::Chapter => "h1",
            HocrBlockKind::Section => "h2",
            HocrBlockKind::Subsection => "h3",
            HocrBlockKind::Subsubsection => "h4",
        }
    }
    pub fn class_name(self) -> &'static str {
        match self {
            HocrBlockKind::Paragraph => "ocr_par",
            HocrBlockKind::Part => "ocr_part",
            HocrBlockKind::Chapter => "ocr_chapter",
            HocrBlockKind::Section => "ocr_section",
            HocrBlockKind::Subsection => "ocr_subsection",
            HocrBlockKind::Subsubsection => "ocr_subsubsection",
        }
    }
    pub fn from_class_name(class_name: &str) -> Option<Self> {
        match class_name {
            "ocr_par" => Some(HocrBlockKind::Paragraph),
            "ocr_part" => Some(HocrBlockKind::Part),
            "ocr_chapter" => Some(HocrBlockKind::Chapter),
            "ocr_section" => Some(HocrBlockKind::Section),
            "ocr_subsection" => Some(HocrBlockKind::Subsection),
            "ocr_subsubsection" => Some(HocrBlockKind::Subsubsection),
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
            self.bbox[0],
            self.bbox[1],
            self.bbox[2],
            self.bbox[3],
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
        match bbox_union_all(&subboxes) {
            Some(union) => self.bbox = union,
            None => self.bbox = [0, 0, 0, 0],
        }
    }
}

impl HocrLine {
    pub fn to_hocr_html(&self) -> String {
        let mut html = format!(
            "<span class=\"ocr_line\" id=\"{}\" title=\"bbox {} {} {} {}\">",
            escape_attr(&self.id),
            self.bbox[0],
            self.bbox[1],
            self.bbox[2],
            self.bbox[3],
        );

        for word in &self.words {
            html.push_str(&word.to_hocr_html());
        }

        html.push_str("</span>");
        html
    }
    pub fn rebuild_bbox(&mut self) {
        let subboxes = self.words.iter().map(|w| w.bbox).collect::<Vec<_>>();
        match bbox_union_all(&subboxes) {
            Some(union) => self.bbox = union,
            None => self.bbox = [0, 0, 0, 0],
        }
    }
}

impl HocrWord {
    pub fn to_hocr_html(&self) -> String {
        format!(
            "<span class=\"ocrx_word\" id=\"{}\" title=\"bbox {} {} {} {}; x_wconf {}\">{}</span>",
            escape_attr(&self.id),
            self.bbox[0],
            self.bbox[1],
            self.bbox[2],
            self.bbox[3],
            self.wconf,
            escape_text(&self.text),
        )
    }
}

pub fn parse(html: &str) -> Option<HocrPage> {
    let document = Html::parse_document(html);

    let sel_page = Selector::parse("div.ocr_page").ok()?;
    let sel_carea = Selector::parse("div.ocr_carea").ok()?;
    let sel_par = Selector::parse("p.ocr_par").ok()?;
    let sel_line = Selector::parse("span.ocr_line").ok()?;
    let sel_word = Selector::parse("span.ocrx_word").ok()?;

    let sel_block = Selector::parse("p.ocr_par, h1.ocr_part, h1.ocr_chapter, h2.ocr_section, h3.ocr_subsection, h4.ocr_subsubsection").ok()?;

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
                            let line_bbox = bbox(line_el.attr("title").unwrap_or(""))?;
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
                                words,
                            })
                        })
                        .collect();

                    let kind = block_el
                        .attr("class")
                        .unwrap_or("")
                        .split_whitespace()
                        .find_map(HocrBlockKind::from_class_name)?;
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

            Some(HocrCarea {
                level: "carea".to_string(),
                id: carea_id,
                bbox: carea_bbox,
                blocks,
            })
        })
        .collect();

    Some(HocrPage {
        level: "page".to_string(),
        page_id,
        bbox: page_bbox,
        careas,
    })
}

fn has_class(el: &scraper::ElementRef<'_>, class_name: &str) -> bool {
    el.attr("class")
        .unwrap_or("")
        .split_whitespace()
        .any(|c| c == class_name)
}

fn bbox(title: &str) -> Option<[i32; 4]> {
    for part in title.split(';') {
        if let Some(rest) = part.trim().strip_prefix("bbox ") {
            let v: Vec<i32> = rest
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if v.len() >= 4 {
                return Some([v[0], v[1], v[2], v[3]]);
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
