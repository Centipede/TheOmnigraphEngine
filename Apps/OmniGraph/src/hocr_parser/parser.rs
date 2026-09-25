use std::collections::HashMap;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use crate::hocr_parser::models::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParserConfig {
    #[serde(default)]
    pub block_metrics: bool,
    #[serde(default)]
    pub line_metrics: bool,
}

pub fn collect_unknowns(el: ElementRef, selector: &Selector) -> Vec<HocrUnknown> {
    el.child_elements()
        .filter(|el| !selector.matches(el))
        .map(|el| HocrUnknown { string: el.html() })
        .collect()
}

pub fn parse(html: &str, _config: ParserConfig) -> Option<HocrPage> {
    let document = Html::parse_document(html);

    let sel_page = Selector::parse("div.ocr_page").ok()?;
    let sel_carea = Selector::parse("div.ocr_carea").ok()?;
    let sel_block = Selector::parse("img, p.ocr_par, h1.ocr_part, h1.ocr_chapter, h2.ocr_section, h3.ocr_subsection, h4.ocr_subsubsection, h5.ocr_subsubsubsection, h6.ocr_subsubsubsubsection").ok()?;
    let sel_line = Selector::parse("span.ocr_line, span.ocr_caption").ok()?;
    let sel_word = Selector::parse("span.ocrx_word").ok()?;
    let sel_dropcap = Selector::parse("span.dropcap").ok()?;


    let page_el = document.select(&sel_page).next()?;
    let page_id = page_el.attr("id").unwrap_or("page_1").to_string();
    let page_title_map = split_title(page_el.attr("title").unwrap_or(""));
    let page_bbox = page_title_map.get("bbox").and_then(|s| to_bbox_opt(s))?;

    let careas = page_el
        .select(&sel_carea)
        .filter_map(|carea_el| {
            let title_str = carea_el.attr("title").unwrap_or("");
            let title_keyvals = split_title(title_str);
            let carea_bbox = title_keyvals.get("bbox").and_then(|s| to_bbox_opt(s))?;
            let carea_id = carea_el.attr("id").unwrap_or("").to_string();

            let flow = title_keyvals.get("flow").cloned();
            let layout = title_keyvals.get("layout").cloned();

            let blocks = carea_el
                .select(&sel_block)
                .filter_map(|block_el| {
                    let block_title_map = split_title(block_el.attr("title").unwrap_or(""));
                    let block_bbox = block_title_map.get("bbox").and_then(|s| to_bbox_opt(s))?;
                    let block_id = block_el.attr("id").unwrap_or("").to_string();
                    let block_lang = block_el.attr("lang").map(str::to_string);

                    let lines = block_el
                        .select(&sel_line)
                        .filter_map(|line_el| {
                            let title_keyvals = split_title(line_el.attr("title").unwrap_or(""));
                            let line_bbox = title_keyvals.get("bbox").and_then(|s| to_bbox_opt(s)).unwrap_or(HocrBbox::empty());
                            let line_baseline = title_keyvals.get("baseline").and_then(|s| to_baseline(s));
                            let line_x_size = title_keyvals.get("x_size").and_then(|s| s.parse::<f32>().ok());
                            let line_x_ascenders = title_keyvals.get("x_ascenders").and_then(|s| s.parse::<f32>().ok());
                            let line_x_descenders = title_keyvals.get("x_descenders").and_then(|s| s.parse::<f32>().ok());
                            let line_id = line_el.attr("id").unwrap_or("").to_string();
                            let line_lang = line_el.attr("lang").map(str::to_string);

                            let words = line_el
                                .select(&sel_word)
                                .filter_map(|word_el| {
                                    let title_str = word_el.attr("title").unwrap_or("");
                                    let title_map = split_title(title_str);
                                    let word_bbox = title_map.get("bbox").and_then(|s| to_bbox_opt(s))?;
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
                                        wconf: title_map.get("x_wconf").and_then(|s| to_wconf(s)).unwrap_or(0),
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
                    let mut hints = HocrBlockHints::default();

                    // Legacy hints
                    if block_title_map.contains_key("continue_from_previous") {
                        hints.break_from_preceding = Evidence::Assigned(false);
                    }
                    if block_title_map.contains_key("continue_to_following") {
                        hints.break_from_following = Evidence::Assigned(false);
                    }

                    // Modern hints
                    if let Some(s) = block_title_map.get("test_x_indent") {
                        hints.test_x_indent = Evidence::from_hocr_string(s);
                    }
                    if let Some(s) = block_title_map.get("test_x_dedent") {
                        hints.test_x_dedent = Evidence::from_hocr_string(s);
                    }
                    if let Some(s) = block_title_map.get("test_hyphenation") {
                        hints.test_hyphenation = Evidence::from_hocr_string(s);
                    }
                    if let Some(s) = block_title_map.get("test_y_advance") {
                        hints.test_y_advance = Evidence::from_hocr_string(s);
                    }
                    if let Some(s) = block_title_map.get("test_y_reverse") {
                        hints.test_y_reverse = Evidence::from_hocr_string(s);
                    }
                    if let Some(s) = block_title_map.get("test_preceding_terminal") {
                        hints.test_preceding_terminal = Evidence::from_hocr_string(s);
                    }
                    if let Some(s) = block_title_map.get("test_following_terminal") {
                        hints.test_following_terminal = Evidence::from_hocr_string(s);
                    }
                    if let Some(s) = block_title_map.get("break_from_preceding") {
                        hints.break_from_preceding = Evidence::from_hocr_string(s);
                    }
                    if let Some(s) = block_title_map.get("break_from_following") {
                        hints.break_from_following = Evidence::from_hocr_string(s);
                    }
                    let block = HocrBlock {
                        level: "block".to_string(),
                        id: block_id,
                        bbox: block_bbox,
                        lang: block_lang,
                        hints,
                        metrics: None,
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
pub(crate) fn has_class(el: &scraper::ElementRef<'_>, class_name: &str) -> bool {
    el.attr("class")
        .unwrap_or("")
        .split_whitespace()
        .any(|c| c == class_name)
}

pub(crate) fn split_title(title: &str) -> HashMap<String, String> {
    let mut keyvals: HashMap<String, String> = HashMap::new();

    for part in title.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let mut parts = part.split_whitespace();
        if let Some(key) = parts.next() {
            let value = parts.collect::<Vec<_>>().join(" ");
            keyvals.insert(key.to_string(), value);
        }
    }
    keyvals
}

#[allow(dead_code)]
pub(crate) fn join_title(keyvals: &HashMap<String, String>) -> String {
    keyvals.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ")
}

pub(crate) fn to_bbox_opt(bbox_str: &str) -> Option<HocrBbox> {
    let v: Vec<i32> = bbox_str
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if v.len() >= 4 {
        Some(HocrBbox([v[0], v[1], v[2], v[3]]))
    } else {
        None
    }
}

pub(crate) fn to_baseline(baseline_str: &str) -> Option<(f32, f32)> {
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

pub(crate) fn to_wconf(wconf_str: &str) -> Option<i32> {
    wconf_str.trim().parse().ok()
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
