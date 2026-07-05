use scraper::{Html, Selector};
use serde::Serialize;

/// Bounding box in scan pixel coordinates: [left, top, right, bottom]
pub type HocrBbox = [i32; 4];

#[derive(Serialize)]
pub struct HocrWord {
    pub id: String,
    pub bbox: HocrBbox,
    pub text: String,
    pub wconf: i32,
}

#[derive(Serialize)]
pub struct HocrLine {
    pub id: String,
    pub bbox: HocrBbox,
    pub words: Vec<HocrWord>,
}

#[derive(Serialize)]
pub struct HocrPar {
    pub id: String,
    pub bbox: HocrBbox,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    pub lines: Vec<HocrLine>,
}

#[derive(Serialize)]
pub struct HocrCarea {
    pub id: String,
    pub bbox: HocrBbox,
    pub pars: Vec<HocrPar>,
}

#[derive(Serialize)]
pub struct HocrPage {
    pub page_id: String,
    pub bbox: HocrBbox,
    pub careas: Vec<HocrCarea>,
}

pub fn parse(html: &str) -> Option<HocrPage> {
    let document = Html::parse_document(html);

    let sel_page  = Selector::parse("div.ocr_page").ok()?;
    let sel_carea = Selector::parse("div.ocr_carea").ok()?;
    let sel_par   = Selector::parse("p.ocr_par").ok()?;
    let sel_line  = Selector::parse("span.ocr_line").ok()?;
    let sel_word  = Selector::parse("span.ocrx_word").ok()?;

    let page_el = document.select(&sel_page).next()?;
    let page_id = page_el.attr("id").unwrap_or("page_1").to_string();
    let page_bbox = bbox(page_el.attr("title").unwrap_or(""))?;

    let careas = page_el
        .select(&sel_carea)
        .filter_map(|carea_el| {
            let carea_bbox = bbox(carea_el.attr("title").unwrap_or(""))?;
            let carea_id = carea_el.attr("id").unwrap_or("").to_string();

            let pars = carea_el
                .select(&sel_par)
                .filter_map(|par_el| {
                    let par_bbox = bbox(par_el.attr("title").unwrap_or(""))?;
                    let par_id = par_el.attr("id").unwrap_or("").to_string();
                    let par_lang = par_el.attr("lang").map(str::to_string);

                    let lines = par_el
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

                    Some(HocrPar { id: par_id, bbox: par_bbox, lang: par_lang, lines })
                })
                .collect();

            Some(HocrCarea { id: carea_id, bbox: carea_bbox, pars })
        })
        .collect();

    Some(HocrPage { page_id, bbox: page_bbox, careas })
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
