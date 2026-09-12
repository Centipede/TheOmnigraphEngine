use std::fmt;
use std::fmt::Display;
use crate::hocr_parser::models::*;
use crate::hocr_parser::utils::*;

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
}

impl HocrCarea {
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

        html.push_str("</div>\n");
        html
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

        html.push_str(&format!("</{tag}>\n"));
        html
    }
}

impl HocrLine {
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

        html.push_str("</span>\n");
        html
    }
}

impl HocrWord {
    pub fn to_hocr_html(&self) -> String {
        let lang_attr = match &self.lang {
            Some(l) => format!(" lang=\"{}\"", escape_attr(l)),
            None => "".to_string(),
        };
        let dropcap_html = match &self.dropcap {
            Some(d) => format!("<span class=\"dropcap\">{}</span>\n", escape_text(d)),
            None => "".to_string(),
        };
        format!(
            "<span class=\"ocrx_word\" id=\"{}\" title=\"bbox {} {} {} {}; x_wconf {}\"{}>{}{}</span>\n",
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
        format!("{}\n", self.string)
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
            HocrBlockKind::Table => "TBL",
            HocrBlockKind::List => "LST",
        };
        write!(f, "{}", code)
    }
}
