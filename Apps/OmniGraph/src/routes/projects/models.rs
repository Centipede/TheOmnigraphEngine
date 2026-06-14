use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Author {
    pub full_name: String,
    pub abbrev: String,
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
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
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

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Page {
    pub index: usize,
    pub name: String,
    pub scan: String,
    pub scan_width: u32,
    pub scan_height: u32,
    #[serde(default)]
    pub thumb: String,
    pub thumb_width: u32,
    pub thumb_height: u32,
    #[serde(default)]
    pub batch: u32,
    #[serde(default)]
    pub import_order: u32,
    #[serde(default)]
    pub crop_edges: CropEdges,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PageDb {
    pub pages: Vec<Page>,
    #[serde(default)]
    pub next_batch: u32,
}

pub const IMPORT_ORDER_GAP: u32 = 1000;


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