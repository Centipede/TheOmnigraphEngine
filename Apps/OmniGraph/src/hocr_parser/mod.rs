pub mod models;
pub mod operations;
pub mod parser;
pub mod html;
pub mod utils;
pub mod navigation;

pub use models::*;
#[allow(unused_imports)]
pub use parser::{parse, find_node, collect_unknowns, ParserConfig};
#[allow(unused_imports)]
pub use utils::{stem_from_id, count_from_id};

pub struct FileSystemPageProvider {
    pub projects_dir: std::path::PathBuf,
    pub machine_name: String,
    pub page_order: Vec<String>,
}

impl FileSystemPageProvider {
    pub fn new(projects_dir: std::path::PathBuf, machine_name: &str) -> Option<Self> {
        use crate::routes::projects::storage;
        let pagedb_path = projects_dir
            .join(machine_name)
            .join("pages")
            .join("pagedata.json");
        let pagedb = storage::load_page_db(&pagedb_path);

        let page_order = pagedb
            .pages
            .iter()
            .filter_map(|page| {
                std::path::Path::new(&page.scan)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(str::to_string)
            })
            .collect();
        Some(Self {
            projects_dir,
            machine_name: machine_name.to_string(),
            page_order,
        })
    }
}

impl crate::hocr_parser::navigation::HocrPageProvider for FileSystemPageProvider {
    fn get_page(&self, page_id: &str) -> Option<crate::hocr_parser::HocrPage> {
        use crate::routes::projects::storage;
        let hocr_path = storage::hocr_active_path(&self.projects_dir, &self.machine_name, page_id)?;
        let html = std::fs::read_to_string(&hocr_path).ok()?;
        crate::hocr_parser::parse(&html, crate::hocr_parser::ParserConfig::default())
    }

    fn get_page_order(&self) -> Vec<String> {
        self.page_order.clone()
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_auto_detection;
