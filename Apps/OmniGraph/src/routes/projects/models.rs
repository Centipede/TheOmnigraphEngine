use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Author {
    pub full_name: String,
    pub abbrev: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ProcessingSettings {
    #[serde(default)]
    pub desaturate: bool,
    #[serde(default)]
    pub contrast: f32,
    #[serde(default)]
    pub brightness: f32,
}

impl ProcessingSettings {
    pub fn has_effect(&self) -> bool {
        self.desaturate || self.contrast != 0.0 || self.brightness != 0.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EditorPalette {
    pub keep_color: String,
    pub discard_color: String,
    pub carea_overlay_color: String,
    pub block_overlay_color: String,
    pub line_overlay_color: String,
    pub word_overlay_color: String,
    #[serde(default)]
    pub part_color: crate::hocr_parser::ColorSpecification,
    #[serde(default)]
    pub h1_color: crate::hocr_parser::ColorSpecification,
    #[serde(default)]
    pub h2_color: crate::hocr_parser::ColorSpecification,
    #[serde(default)]
    pub h3_color: crate::hocr_parser::ColorSpecification,
    #[serde(default)]
    pub h4_color: crate::hocr_parser::ColorSpecification,
    #[serde(default)]
    pub h5_color: crate::hocr_parser::ColorSpecification,
    #[serde(default)]
    pub h6_color: crate::hocr_parser::ColorSpecification,
    #[serde(default)]
    pub p_color: crate::hocr_parser::ColorSpecification,
    #[serde(default)]
    pub img_color: crate::hocr_parser::ColorSpecification,
    #[serde(default)]
    pub lst_color: crate::hocr_parser::ColorSpecification,
    #[serde(default)]
    pub tbl_color: crate::hocr_parser::ColorSpecification,
}

impl Default for EditorPalette {
    fn default() -> Self {
        Self {
            keep_color: "rgba(0, 180, 0, 0.12)".to_string(),
            discard_color: "rgba(220, 0, 0, 0.35)".to_string(),
            carea_overlay_color: "rgba(249, 115, 22, 1)".to_string(),
            block_overlay_color: "rgba(168, 85, 247, 1)".to_string(),
            line_overlay_color: "rgba(59, 130, 246, 1)".to_string(),
            word_overlay_color: "rgba(34, 197, 94, 1)".to_string(),
            part_color: crate::hocr_parser::ColorSpecification {
                hue_shift: Some(0.0),
                ..Default::default()
            },
            h1_color: crate::hocr_parser::ColorSpecification {
                hue_shift: Some(30.0),
                ..Default::default()
            },
            h2_color: crate::hocr_parser::ColorSpecification {
                hue_shift: Some(60.0),
                ..Default::default()
            },
            h3_color: crate::hocr_parser::ColorSpecification {
                hue_shift: Some(90.0),
                ..Default::default()
            },
            h4_color: crate::hocr_parser::ColorSpecification {
                hue_shift: Some(120.0),
                ..Default::default()
            },
            h5_color: crate::hocr_parser::ColorSpecification {
                hue_shift: Some(150.0),
                ..Default::default()
            },
            h6_color: crate::hocr_parser::ColorSpecification {
                hue_shift: Some(180.0),
                ..Default::default()
            },
            p_color: crate::hocr_parser::ColorSpecification {
                hue_shift: Some(210.0),
                ..Default::default()
            },
            img_color: crate::hocr_parser::ColorSpecification {
                hue_shift: Some(240.0),
                ..Default::default()
            },
            lst_color: crate::hocr_parser::ColorSpecification {
                hue_shift: Some(270.0),
                ..Default::default()
            },
            tbl_color: crate::hocr_parser::ColorSpecification {
                hue_shift: Some(300.0),
                ..Default::default()
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub machine_name: String,
    pub abbrev: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<Author>,
    #[serde(default, with = "optional_date")]
    pub published: Option<time::Date>,
    pub ocr_language: Option<String>,
    #[serde(default)]
    pub flows: Vec<crate::hocr_parser::FlowSchema>,
    #[serde(default)]
    pub layouts: Vec<crate::hocr_parser::LayoutSchema>,
    #[serde(default)]
    pub processing: Option<ProcessingSettings>,
    #[serde(default)]
    pub editor_palette: EditorPalette,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct CropEdges {
    /// Scan pixels from left edge.
    pub left: u32,
    /// Scan pixels from top edge.
    pub top: u32,
    /// Scan pixels from right edge.
    pub right: u32,
    /// Scan pixels from bottom edge.
    pub bottom: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum HintType {
    DropCap {
        #[serde(default)]
        letter: String,
    },
    Image,
    Callout,
    Garbage,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Hint {
    #[serde(flatten)]
    pub hint_type: HintType,
    pub area: CropEdges,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Page {
    pub index: usize,
    pub name: String,
    pub scan: String,
    #[serde(default)]
    pub scan_width: u32,
    #[serde(default)]
    pub scan_height: u32,
    #[serde(default)]
    pub thumb: String,
    #[serde(default)]
    pub thumb_width: u32,
    #[serde(default)]
    pub thumb_height: u32,
    #[serde(default)]
    pub batch: u32,
    #[serde(default)]
    pub import_order: u32,
    #[serde(default)]
    pub crop_edges: CropEdges,
    #[serde(default)]
    pub hints: Vec<Hint>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PageDb {
    pub pages: Vec<Page>,
    #[serde(default)]
    pub next_batch: u32,
}

pub const IMPORT_ORDER_GAP: u32 = 1000;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Headline {
    pub page: String,
    pub block_id: String,
    pub is_linked: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SectionHeadline {
    pub page: String,
    pub block_id: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SubsectionType {
    Sections,
    Flows,
}

impl Default for SubsectionType {
    fn default() -> Self {
        Self::Sections
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SectionKind {
    Part,
    Chapter,
    Section,
    Subsection,
    Subsubsection,
    Subsubsubsection,
    Subsubsubsubsection,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Section {
    pub path_id: String,
    pub kind: SectionKind,
    pub title: String,
    pub is_linked: bool,
    pub is_orphaned: bool,
    pub is_suggested: bool,
    pub headline: Option<SectionHeadline>,
    pub subsection_type: SubsectionType,
    pub subsections: Vec<Section>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct StructureDb {
    pub sections: Vec<Section>,
    pub headlines: Vec<Headline>,
}


mod optional_date {
    use serde::{Deserialize, Deserializer, Serializer};
    use time::{Date, Month};

    pub fn serialize<S>(date: &Option<Date>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(date) => serializer.serialize_str(&format!(
                "{:04}-{:02}-{:02}",
                date.year(),
                date.month() as u8,
                date.day()
            )),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Date>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Some(s) = Option::<String>::deserialize(deserializer)? else {
            return Ok(None);
        };

        let mut parts = s.splitn(3, '-');
        let year: i32 = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing year"))?
            .parse()
            .map_err(serde::de::Error::custom)?;
        let month: u8 = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing month"))?
            .parse()
            .map_err(serde::de::Error::custom)?;
        let day: u8 = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing day"))?
            .parse()
            .map_err(serde::de::Error::custom)?;

        let month = Month::try_from(month).map_err(serde::de::Error::custom)?;
        let date = Date::from_calendar_date(year, month, day).map_err(serde::de::Error::custom)?;

        Ok(Some(date))
    }
}