use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// Bounding box in scan pixel coordinates: [left, top, right, bottom]
pub type HocrBbox = [i32; 4];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrWord {
    pub id: String,
    pub bbox: HocrBbox,
    pub text: String,
    pub wconf: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrLine {
    pub id: String,
    pub bbox: HocrBbox,
    pub words: Vec<HocrWord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrTextBlock {
    pub id: String,
    pub bbox: HocrBbox,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    pub lines: Vec<HocrLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrBlock {
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
    pub id: String,
    pub bbox: HocrBbox,
    pub blocks: Vec<HocrBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HocrPage {
    pub page_id: String,
    pub bbox: HocrBbox,
    pub careas: Vec<HocrCarea>,
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

        let lang_attr = self.lang.as_deref()
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

    let sel_page  = Selector::parse("div.ocr_page").ok()?;
    let sel_carea = Selector::parse("div.ocr_carea").ok()?;
    let sel_par   = Selector::parse("p.ocr_par").ok()?;
    let sel_line  = Selector::parse("span.ocr_line").ok()?;
    let sel_word  = Selector::parse("span.ocrx_word").ok()?;

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
                                        id: word_el.attr("id").unwrap_or("").to_string(),
                                        bbox: word_bbox,
                                        text: word_el.text().collect::<String>().trim().to_string(),
                                        wconf: wconf(title),
                                    })
                                })
                                .collect();

                            Some(HocrLine { id: line_id, bbox: line_bbox, words })
                        })
                        .collect();

                    let kind = block_el
                        .attr("class")
                        .unwrap_or("")
                        .split_whitespace()
                        .find_map(HocrBlockKind::from_class_name)?;
                    let block = HocrBlock { id: block_id, bbox: block_bbox, lang: block_lang, kind, lines };

                    Some(block)
                }).collect();

            Some(HocrCarea { id: carea_id, bbox: carea_bbox, blocks })
        })
        .collect();

    Some(HocrPage { page_id, bbox: page_bbox, careas })
}

fn has_class(el: &scraper::ElementRef<'_>, class_name: &str) -> bool {
    el.attr("class").unwrap_or("").split_whitespace().any(|c| c == class_name)
}

fn bbox(title: &str) -> Option<[i32; 4]> {
    for part in title.split(';') {
        if let Some(rest) = part.trim().strip_prefix("bbox ") {
            let v: Vec<i32> = rest.split_whitespace().filter_map(|s| s.parse().ok()).collect();
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