use crate::hocr_parser::models::*;

pub trait HocrPageProvider {
    fn get_page(&self, page_id: &str) -> Option<HocrPage>;
    fn get_page_order(&self) -> Vec<String>;
}

pub fn preceding_block(
    provider: &dyn HocrPageProvider,
    page_id: &str,
    block_id: &str,
) -> Option<(HocrBlock, String, HocrPath)> {
    let page = provider.get_page(page_id)?;
    let flow = page.get_block_flow(block_id)?;
    
    let page_order = provider.get_page_order();
    let current_page_idx = page_order.iter().position(|id| id == page_id)?;

    // 1. Search in current page
    let blocks_in_flow = page.filter_blocks_with_paths(&flow);
    let block_idx = blocks_in_flow.iter().position(|(b, _)| b.id == block_id)?;
    
    if block_idx > 0 {
        let (block, path) = &blocks_in_flow[block_idx - 1];
        return Some(((*block).clone(), page_id.to_string(), *path));
    }

    // 2. Search in previous pages
    for i in (0..current_page_idx).rev() {
        let prev_page_id = &page_order[i];
        if let Some(prev_page) = provider.get_page(prev_page_id) {
            let prev_blocks = prev_page.filter_blocks_with_paths(&flow);
            if !prev_blocks.is_empty() {
                let (block, path) = &prev_blocks[prev_blocks.len() - 1];
                return Some(((*block).clone(), prev_page_id.clone(), *path));
            }
        }
    }

    None
}

pub fn succeeding_block(
    provider: &dyn HocrPageProvider,
    page_id: &str,
    block_id: &str,
) -> Option<(HocrBlock, String, HocrPath)> {
    let page = provider.get_page(page_id)?;
    let flow = page.get_block_flow(block_id)?;
    
    let page_order = provider.get_page_order();
    let current_page_idx = page_order.iter().position(|id| id == page_id)?;

    // 1. Search in current page
    let blocks_in_flow = page.filter_blocks_with_paths(&flow);
    let block_idx = blocks_in_flow.iter().position(|(b, _)| b.id == block_id)?;
    
    if block_idx < blocks_in_flow.len() - 1 {
        let (block, path) = &blocks_in_flow[block_idx + 1];
        return Some(((*block).clone(), page_id.to_string(), *path));
    }

    // 2. Search in following pages
    for i in (current_page_idx + 1)..page_order.len() {
        let next_page_id = &page_order[i];
        if let Some(next_page) = provider.get_page(next_page_id) {
            let next_blocks = next_page.filter_blocks_with_paths(&flow);
            if !next_blocks.is_empty() {
                let (block, path) = &next_blocks[0];
                return Some(((*block).clone(), next_page_id.clone(), *path));
            }
        }
    }

    None
}
