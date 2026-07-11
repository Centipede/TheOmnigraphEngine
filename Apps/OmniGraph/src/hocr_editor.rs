use std::path::Path;
use minijinja::filters::attr;
use xmlparser::{Token, Tokenizer};

pub struct HocrElementSpan {
    pub id: String,
    pub tag_name: String,

    /// `<p class="ocr_par" ...>`
    pub open_tag: std::ops::Range<usize>,

    /// Inner content between open and close tags.
    pub inner: std::ops::Range<usize>,

    /// `</p>`
    pub close_tag: Option<std::ops::Range<usize>>,

    /// Whole element, including open and close tags.
    pub outer: std::ops::Range<usize>,

    /// Attribute source ranges.
    pub attrs: std::collections::HashMap<String, HocrAttrSpan>,
}

pub struct HocrAttrSpan {
    pub name: String,

    /// Whole attribute range, e.g. `class="ocr_par"`.
    pub whole: std::ops::Range<usize>,

    /// Only the value range, without quotes, e.g. `ocr_par`.
    pub value: std::ops::Range<usize>,
}

pub enum HocrEditError {
    InvalidHocr,
}


fn match_element(tokens: &mut [Token]) -> Option<HocrElementSpan> {
    // if tokens.is_empty() { return None; }
    //
    // let Token::ElementStart { prefix, local, span } = tokens[0] else { return None };
    // let tag = local.as_str().to_string();
    // let mut attrs: Vec<HocrAttrSpan> = Vec::new();
    // let mut id: Option<String> = None;
    // let mut open_tag_start = span.start();
    //
    // for attr_token in tokens {
    //     match attr_token {
    //         Token::Attribute { prefix, local, value, span } => {
    //             attrs.push(HocrAttrSpan {
    //                 name: local.as_str().to_string(),
    //                 whole: span.start()..span.end(),
    //                 value: value.start()..value.end(),
    //             });
    //             if local.as_str() == "id" {
    //                 id = Some(value.as_str().to_string());
    //             }
    //
    //             println!("<{:}{:}", prefix.as_str(), local.as_str());
    //         }
    //         _ => { break; }
    //     }
    //     //tokens.remove(0); WIP
    // }
    //
    // let Token::ElementEnd { end, span } = tokens[0] else { return None };
    // let open_tag_end = span.end();
    //
    // Some(HocrElementSpan {
    //     id: tag.clone(),
    //     tag_name: tag.as_str().to_string(),
    //     attrs: attrs.into_iter().map(|a| (a.name.clone(), a)).collect(),
    //     open_tag: open_tag_start..open_tag_end,
    //     inner: open_tag_end + 1..open_tag_end + 1,
    //     outer: open_tag_end + 1..open_tag_end,
    //     close_tag: None,
    // })

    None
}


pub fn prepare_hocr_for_edit(path: &Path) -> Result<(), HocrEditError> {
    let html = std::fs::read_to_string(path).map_err(|_| HocrEditError::InvalidHocr)?;

    let mut tokens = Tokenizer::from(html.as_str());
    //let mut tokens:Vec<Token> = Vec::from(tokens); WIP


    // for token in Tokenizer::from(html.as_str()) {
    //     println!("{:?}", token);
    //
    //     let token = token.map_err(|_| HocrEditError::InvalidHocr)?;
    //
    //     match token {
    //         Token::ElementStart { prefix, local, span } => {
    //             let tag = local.as_str();
    //
    //             println!("<{:}{:}", prefix.as_str(), local.as_str());
    //         }
    //         _ => {}
    //     }
    // }

    Ok(())
}