use std::collections::HashMap;
use crate::hocr_parser::models::*;
use crate::hocr_parser::utils::*;
use crate::hocr_parser::parser::find_node;

impl HocrPage {
    // -- HIGHER ORDER OPERATIONS --

    pub fn cascade_lang(&mut self, default_lang: Option<&str>) {
        for carea in &mut self.careas {
            carea.cascade_lang(default_lang);
        }
    }
    pub fn inject_dropcaps(&mut self, injections: Vec<DropCapInjection>) {
        for injection in injections {
            let mut best_match: Option<(usize, usize, usize, i32)> = None;

            for (c_idx, carea) in self.careas.iter().enumerate() {
                for (b_idx, block) in carea.blocks.iter().enumerate() {
                    for (l_idx, line) in block.lines.iter().enumerate() {
                        if line.words.is_empty() {
                            continue;
                        }

                        let v_diff = (line.bbox.top() - injection.bbox.top()).abs();
                        let h_dist = (line.bbox.left() - injection.bbox.right()).abs();

                        if v_diff <= 50 && h_dist <= 100 {
                            match best_match {
                                Some((_, _, _, best_v_diff)) if v_diff < best_v_diff => {
                                    best_match = Some((c_idx, b_idx, l_idx, v_diff));
                                }
                                None => {
                                    best_match = Some((c_idx, b_idx, l_idx, v_diff));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            if let Some((c_idx, b_idx, l_idx, _)) = best_match {
                if let Some(word) = self.careas[c_idx].blocks[b_idx].lines[l_idx].words.get_mut(0) {
                    word.dropcap = Some(injection.text);
                }
            }
        }
    }
    pub fn inject_images(&mut self, bboxes: Vec<HocrBbox>) {
        for bbox in bboxes {
            // Add as an image block, letting the engine find the best carea/vertical position
            let _ = self.add_block(None, bbox, Some(AddBlockType::Image), None, Some(false), None);
        }
        self.rebuild_bbox();
    }
    pub fn auto_flow(&mut self, flows: Vec<FlowSchema>, _layouts: Vec<LayoutSchema>, merge: bool, carea_ids: Option<Vec<String>>) {
        if flows.is_empty() {
            return;
        }

        let default_flow = flows[0].name.clone();

        // 1. Assign default flow to careas with no current assignment.
        for carea in &mut self.careas {
            if let Some(ref ids) = carea_ids {
                if !ids.contains(&carea.id) {
                    continue;
                }
            }
            if carea.flow.as_ref().map_or(true, |f| f.is_empty()) {
                carea.flow = Some(default_flow.clone());
            }
        }

        if self.careas.is_empty() {
            return;
        }

        // 2. Group consecutive careas by layout.
        let mut new_careas: Vec<HocrCarea> = Vec::new();
        let old_careas = std::mem::take(&mut self.careas);

        let mut current_layout_group: Vec<HocrCarea> = Vec::new();
        let mut current_layout = old_careas[0].layout.clone();

        for carea in old_careas {
            if carea.layout != current_layout {
                // Process previous layout group
                Self::auto_flow_for_layout(&mut new_careas, current_layout_group, merge, &carea_ids);
                current_layout_group = Vec::new();
                current_layout = carea.layout.clone();
            }
            current_layout_group.push(carea);
        }
        // Process last group
        if !current_layout_group.is_empty() {
            Self::auto_flow_for_layout(&mut new_careas, current_layout_group, merge, &carea_ids);
        }

        self.careas = new_careas;
        self.rebuild_bbox();
    }
    fn auto_flow_for_layout(target: &mut Vec<HocrCarea>, group: Vec<HocrCarea>, merge: bool, carea_ids: &Option<Vec<String>>) {
        if !merge {
            target.extend(group);
            return;
        }

        let mut flow_order: Vec<String> = Vec::new();
        let mut merged_careas: HashMap<String, HocrCarea> = HashMap::new();

        for mut carea in group {
            let is_selected = carea_ids.as_ref().map_or(true, |ids| ids.contains(&carea.id));
            
            if !is_selected {
                // Barrier! Flush current merges to maintain document order and prevent merging across unselected items.
                for flow in flow_order {
                    let mut merged = merged_careas.remove(&flow).unwrap();
                    merged.rebuild_bbox();
                    target.push(merged);
                }
                flow_order = Vec::new();
                
                // Add the unselected carea as its own item
                carea.rebuild_bbox();
                target.push(carea);
                continue;
            }

            let flow = carea.flow.clone().unwrap_or_default();
            if !merged_careas.contains_key(&flow) {
                flow_order.push(flow.clone());
                merged_careas.insert(flow, carea);
            } else {
                let existing = merged_careas.get_mut(&flow).unwrap();
                existing.blocks.append(&mut carea.blocks);
                existing.unknowns.append(&mut carea.unknowns);
            }
        }

        for flow in flow_order {
            let mut merged = merged_careas.remove(&flow).unwrap();
            merged.rebuild_bbox();
            target.push(merged);
        }
    }

    // -- CLEANUP --

    pub fn cleanup_carea(&mut self, carea: usize) {
        if self.careas[carea].blocks.is_empty() {
            self.careas.remove(carea);
        } else {
            self.careas[carea].rebuild_bbox();
        }
    }
    pub fn cleanup_block(&mut self, carea: usize, block: usize) {
        if self.careas[carea].blocks[block].lines.is_empty() {
            if self.careas[carea].blocks[block].kind != HocrBlockKind::Image {
                self.careas[carea].blocks.remove(block);
            }
        } else {
            self.careas[carea].blocks[block].rebuild_bbox();
        }
        self.cleanup_carea(carea);
    }
    pub fn cleanup_line(&mut self, carea: usize, block: usize, line: usize) {
        if self.careas[carea].blocks[block].lines[line].words.is_empty() {
            self.careas[carea].blocks[block].lines.remove(line);
            self.cleanup_block(carea, block);
        } else {
            self.careas[carea].blocks[block].lines[line].rebuild_bbox();
        }
    }

    // -- NAVIGATION --

    pub fn next_carea_path(&self, path: HocrPath) -> Option<HocrPath> {
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
    pub fn next_block_path(&self, path: HocrPath) -> Option<HocrPath> {
        if let HocrPath::Block { carea, block } = path {
            if block < self.careas[carea].blocks.len() - 1 {
                Some(HocrPath::Block {
                    carea,
                    block: block + 1,
                })
            } else {
                loop {
                    let path = self.next_carea_path(path.to_carea()?);
                    if let Some (HocrPath::Carea { carea }) = path {
                        if self.careas[carea].blocks.len() > 0 {
                            return Some(HocrPath::Block { carea, block: 0 });
                        }
                    }
                    else {
                        return None;
                    }
                }
            }
        } else {
            None
        }
    }
    pub fn previous_carea_path(&self, path: HocrPath) -> Option<HocrPath> {
        if let HocrPath::Carea { carea } = path {
            if carea > 0 {
                Some(HocrPath::Carea { carea: carea - 1 })
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn previous_block_path(&self, path: HocrPath) -> Option<HocrPath> {
        if let HocrPath::Block { carea, block } = path {
            if block > 0 {
                Some(HocrPath::Block {
                    carea,
                    block: block - 1,
                })
            } else {
                loop {
                    let path = self.previous_carea_path(path.to_carea()?);
                    if let Some(HocrPath::Carea { carea }) = path {
                        if self.careas[carea].blocks.len() > 0 {
                            return Some(HocrPath::Block { carea, block: self.careas[carea].blocks.len() - 1 });
                        }
                    } else {
                        return None;
                    }
                }
            }
        } else {
            None
        }
    }

    // -- MOVING --

    pub fn move_carea_up(&mut self, carea: usize) {
        if carea > 0 {
            println!("move_carea_up: carea {}", carea);

            let moving_carea = self.careas.remove(carea);
            self.careas.insert(carea - 1, moving_carea);

            self.cleanup_carea(carea);
        }
        else {
            println!("move_carea_up: not moving");
        }
    }
    pub fn move_carea_down(&mut self, carea: usize) {
        if carea < self.careas.len() - 1 {
            println!("move_carea_down: carea {}", carea);

            let moving_carea = self.careas.remove(carea);
            self.careas.insert(carea + 1, moving_carea);

            self.cleanup_carea(carea);
        }
        else {
            println!("move_carea_down: not moving");
        }
    }
    pub fn move_block_up(&mut self, carea: usize, block: usize) {

        // Move within the same carea?
        if block > 0 {
            println!("move_block_up - within same carea: block {}", block);
            let moving_block = self.careas[carea].blocks.remove(block - 1);
            self.careas[carea].blocks.insert(block, moving_block);

            self.cleanup_carea(carea);
        }
        else if block == 0 {
            let previous_carea = self.previous_carea_path(HocrPath::Carea { carea });
            println!("move_block_up: previous_carea {:?}", previous_carea);
            if let Some(HocrPath::Carea { carea:to_carea }) = previous_carea {
                let moving_block = self.careas[carea].blocks.remove(block);
                let to_carea_size = self.careas[to_carea].blocks.len();

                self.careas[to_carea].blocks.insert(to_carea_size, moving_block);

                self.cleanup_carea(to_carea); // Note! Order... cleanup may remove empty careas.
                self.cleanup_carea(carea);    // Note! If so, indices are invalidated. Hence: to_ goes first.
            }
        }
    }
    pub fn move_block_down(&mut self, carea: usize, block: usize) {
        let carea_size = self.careas[carea].blocks.len();
        if block < carea_size - 1 {
            println!("move_block_down - within same carea: block {}", block);
            let block_down = self.careas[carea].blocks.remove(block);
            self.careas[carea].blocks.insert(block+1, block_down);

            self.cleanup_carea(carea);
        }
        else if block == carea_size - 1 {
            let next_carea = self.next_carea_path(HocrPath::Carea { carea });
            println!("move_block_down: next_carea {:?}", next_carea);
            if let Some (HocrPath::Carea { carea:to_carea }) = next_carea {
                let moving_block = self.careas[carea].blocks.remove(block);
                self.careas[to_carea].blocks.insert(0, moving_block);

                self.cleanup_carea(to_carea); // Note! Order... cleanup may remove empty careas.
                self.cleanup_carea(carea);    // Note! If so, indices are invalidated. Hence: to_ goes first.
            }
        }
        else {
            panic!("move_block_down: invalid block index");
        }
    }
    pub fn move_line_up(&mut self, carea: usize, block: usize, line: usize) {

        // Move within the same block?
        if line > 0 {
            println!("move_line_up - within same block: line {}", line);
            let moving_line = self.careas[carea].blocks[block].lines.remove(line - 1);
            self.careas[carea].blocks[block].lines.insert(line, moving_line);

            self.cleanup_block(carea, block);
        }
        else if line == 0 {
            let previous_block = self.previous_block_path(HocrPath::Block { carea, block });
            println!("move_line_up: previous_block {:?}", previous_block);
            if let Some(HocrPath::Block { carea:to_carea, block:to_block }) = previous_block {
                let moving_line = self.careas[carea].blocks[block].lines.remove(line);
                let to_block_size = self.careas[to_carea].blocks[to_block].lines.len();
                self.careas[to_carea].blocks[to_block].lines.insert(to_block_size, moving_line);

                self.cleanup_block(to_carea, to_block);     // Note! Order... cleanup may remove empty blocks.
                self.cleanup_block(carea, block);           // Note! If so, indices are invalidated. Hence: to_ goes first.
            }
        }
        else {
            println!("move_line_up: not moving");
        }
    }
    pub fn move_line_down(&mut self, carea: usize, block: usize, line: usize) {

        let block_size = self.careas[carea].blocks[block].lines.len();

        // Move within the same block?
        if line < block_size - 1 {
            println!("move_line_down - within same block: line {}", line);
            let moving_line = self.careas[carea].blocks[block].lines.remove(line);
            self.careas[carea].blocks[block].lines.insert(line+1, moving_line);

            self.cleanup_block(carea, block);
        }
        // Move line to next block ... even if that requires skipping over empty blocks?
        else if line == block_size - 1 {
            let next_block = self.next_block_path(HocrPath::Block { carea, block });
            println!("move_line_down: next_block {:?}", next_block);
            if let Some (HocrPath::Block { carea:to_carea, block:to_block }) = next_block {
                let moving_line = self.careas[carea].blocks[block].lines.remove(line);
                self.careas[to_carea].blocks[to_block].lines.insert(0, moving_line);

                self.cleanup_block(to_carea, to_block);     // Note! Order... cleanup may remove empty blocks.
                self.cleanup_block(carea, block);           // Note! If so, indices are invalidated. Hence: to_ goes first.
            }
        }
        else {
            println!("move_line_down: not moving");
        }
    }

    // -- MERGES --

    pub fn merge_carea(&mut self, carea1: usize, carea2: usize) {
        if carea1 != carea2 {
            let mut blocks = std::mem::take(&mut self.careas[carea2].blocks);

            self.careas[carea1].blocks.append(&mut blocks);
            self.careas[carea1].rebuild_bbox();
            self.remove_carea(carea2);
        }
    }
    pub fn merge_careas(&mut self, careas: &mut Vec<usize>) -> Result<(), String> {

        if careas.len() < 2 {
            return Err(format!("merge_careas: not enough careas: {:?}", careas));
        }

        careas.sort();
        careas.dedup();

        for i in 0..careas.len() - 1 {
            if careas[i]+1 != careas[i+1] {
                return Err(format!("merge_careas: careas not consecutive: {:?}", careas));
            }
        }

        for i in (0..careas.len() - 1).rev() {
            self.merge_carea(careas[i], careas[i+1]);
        };

        self.cleanup_carea(careas[0]);

        Ok(())
    }
    pub fn merge_block(&mut self, carea: usize, block1: usize, block2: usize) {
        if block1 != block2 {
            let mut lines = std::mem::take(&mut self.careas[carea].blocks[block2].lines);

            self.careas[carea].blocks[block1].lines.append(&mut lines);
            self.careas[carea].blocks[block1].rebuild_bbox();
            self.remove_block(carea, block2);
        }
    }
    pub fn merge_blocks(&mut self, blocks: &mut Vec<(usize, usize)>) -> Result<(), String> {
        if blocks.len() < 2 {
            return Err(format!("merge_blocks: not enough blocks: {:?}", blocks));
        }

        blocks.sort();
        blocks.dedup();

        let docorder = self.careas.iter()
            .map(|c| c.blocks.iter()
                .map(|b| b.id.clone())
                .collect::<Vec<String>>())
            .collect::<Vec<Vec<String>>>()
            .into_iter()
            .flatten()
            .collect::<Vec<String>>();

        let block_ids = blocks.iter()
            .map(|(c, b)| self.careas[*c].blocks[*b].id.clone())
            .collect::<Vec<String>>();

        for i in 0..blocks.len() - 1 {
            let pos1 = docorder.iter().position(|id| id == &block_ids[i]).unwrap();
            let pos2 = docorder.iter().position(|id| id == &block_ids[i+1]).unwrap();
            if pos1+1 != pos2 {
                return Err(format!("merge_blocks: blocks not consecutive: {:?}", blocks));
            }
        }

        let mut moving_blocks = vec![];

        for (c,b) in blocks.iter().skip(1).rev() {
            moving_blocks.push(self.careas[*c].blocks.remove(*b));
        }

        moving_blocks.reverse();

        let (carea, block) = blocks[0];
        let insert_at = block + 1;

        self.careas[carea].blocks.splice(insert_at..insert_at, moving_blocks);

        for i in (0..blocks.len() - 1).rev() {

            self.merge_block(carea, block+i
                             , block+i+1);
        }

        let mut affected_careas: Vec<usize> = blocks.iter().map(|(c, _)| *c).collect();
        affected_careas.dedup();
        for c in affected_careas.into_iter().rev() {
            self.cleanup_carea(c);
        }

        Ok(())
    }
    pub fn merge_line(&mut self, carea: usize, block: usize, line1: usize, line2: usize) {
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
    pub fn merge_lines(&mut self, lines: &mut Vec<(usize, usize, usize)>) -> Result<(), String> {
        if lines.len() < 2 {
            return Err(format!("merge_lines: not enough lines: {:?}", lines));
        }

        lines.sort();
        lines.dedup();

        let docorder = self
            .careas
            .iter()
            .flat_map(|c| c.blocks.iter().flat_map(|b| b.lines.iter().map(|l| l.id.clone())))
            .collect::<Vec<String>>();

        let line_ids = lines
            .iter()
            .map(|(c, b, l)| self.careas[*c].blocks[*b].lines[*l].id.clone())
            .collect::<Vec<String>>();

        for i in 0..lines.len() - 1 {
            let pos1 = docorder.iter().position(|id| id == &line_ids[i]).unwrap();
            let pos2 = docorder.iter().position(|id| id == &line_ids[i + 1]).unwrap();
            if pos1 + 1 != pos2 {
                return Err(format!("merge_lines: lines not consecutive: {:?}", lines));
            }
        }

        let mut moving_lines = vec![];

        for (c, b, l) in lines.iter().skip(1).rev() {
            moving_lines.push(self.careas[*c].blocks[*b].lines.remove(*l));
        }

        moving_lines.reverse();

        let (carea, block, line) = lines[0];
        let insert_at = line + 1;

        self.careas[carea].blocks[block]
            .lines
            .splice(insert_at..insert_at, moving_lines);

        for i in (0..lines.len() - 1).rev() {
            self.merge_line(carea, block, line + i, line + i + 1);
        }

        let mut affected_blocks: Vec<(usize, usize)> =
            lines.iter().map(|(c, b, _)| (*c, *b)).collect();
        affected_blocks.dedup();
        for (c, b) in affected_blocks.into_iter().rev() {
            self.cleanup_block(c, b);
        }

        Ok(())
    }
    pub fn merge_word(
        &mut self,
        carea: usize,
        block: usize,
        line: usize,
        word1: usize,
        word2: usize,
    ) {
        if word1 != word2 {
            let w2 = self.careas[carea].blocks[block].lines[line]
                .words
                .remove(word2);
            let w1 = &mut self.careas[carea].blocks[block].lines[line].words[word1];

            w1.text = format!("{} {}", w1.text, w2.text);
            w1.bbox = w1.bbox.union(w2.bbox);
            w1.wconf = std::cmp::min(w1.wconf, w2.wconf);
            self.careas[carea].blocks[block].lines[line].rebuild_bbox();
        }
    }
    pub fn merge_words(
        &mut self,
        words: &mut Vec<(usize, usize, usize, usize)>,
    ) -> Result<(), String> {
        if words.len() < 2 {
            return Err(format!("merge_words: not enough words: {:?}", words));
        }

        words.sort();
        words.dedup();

        let docorder = self
            .careas
            .iter()
            .flat_map(|c| {
                c.blocks.iter().flat_map(|b| {
                    b.lines
                        .iter()
                        .flat_map(|l| l.words.iter().map(|w| w.id.clone()))
                })
            })
            .collect::<Vec<String>>();

        let word_ids = words
            .iter()
            .map(|(c, b, l, w)| {
                self.careas[*c].blocks[*b].lines[*l].words[*w]
                    .id
                    .clone()
            })
            .collect::<Vec<String>>();

        for i in 0..words.len() - 1 {
            let pos1 = docorder.iter().position(|id| id == &word_ids[i]).unwrap();
            let pos2 = docorder.iter().position(|id| id == &word_ids[i + 1]).unwrap();
            if pos1 + 1 != pos2 {
                return Err(format!("merge_words: words not consecutive: {:?}", words));
            }
        }

        let mut moving_words = vec![];

        for (c, b, l, w) in words.iter().skip(1).rev() {
            moving_words.push(self.careas[*c].blocks[*b].lines[*l].words.remove(*w));
        }

        moving_words.reverse();

        let (carea, block, line, word) = words[0];
        let insert_at = word + 1;

        self.careas[carea].blocks[block].lines[line]
            .words
            .splice(insert_at..insert_at, moving_words);

        for i in (0..words.len() - 1).rev() {
            self.merge_word(carea, block, line, word + i, word + i + 1);
        }

        let mut affected_lines: Vec<(usize, usize, usize)> =
            words.iter().map(|(c, b, l, _)| (*c, *b, *l)).collect();
        affected_lines.dedup();
        for (c, b, l) in affected_lines.into_iter().rev() {
            self.cleanup_line(c, b, l);
        }

        Ok(())
    }

    // -- COMPLEX OPERATIONS --

    pub fn split_carea(&mut self, carea: usize, _block_before: usize, block_after: usize) {
        let new_id = self.get_unique_id(self.careas[carea].id.as_str(), &mut HashMap::new());
        let (flow, layout) = {
            let old_carea = &self.careas[carea];
            (old_carea.flow.clone(), old_carea.layout.clone())
        };
        let old_carea = &mut self.careas[carea];
        let (_, right) = old_carea.blocks.split_at_mut(block_after);
        let new_carea = HocrCarea {
            level: "carea".to_string(),
            id: new_id,
            bbox: HocrBbox::empty(),
            flow,
            layout,
            blocks: right.to_vec(),
            unknowns: vec![],
        };
        old_carea.blocks.truncate(block_after);
        self.careas.insert(carea + 1, new_carea);
        self.careas[carea].rebuild_bbox();
        self.careas[carea + 1].rebuild_bbox();
    }
    pub fn split_block(&mut self, carea: usize, block: usize, _line_before: usize, line_after: usize) {
        let new_id = self.get_unique_id(self.careas[carea].blocks[block].id.as_str(), &mut HashMap::new());
        let carea = &mut self.careas[carea];
        let old_block = &mut carea.blocks[block];
        let (_, right) = old_block.lines.split_at_mut(line_after);
        let new_block = HocrBlock {
            lang: old_block.lang.clone(),
            kind: HocrBlockKind::Paragraph,
            level: "block".to_string(),
            id: new_id,
            bbox: HocrBbox::empty(),
            hints: HocrBlockHints::default(),
            lines: right.to_vec(),
        };
        old_block.lines.truncate(line_after);
        carea.blocks.insert(block + 1, new_block);
        carea.blocks[block].rebuild_bbox();
        carea.blocks[block + 1].rebuild_bbox();
    }
    pub fn replace_or_merge_carea(&mut self, carea_index: usize, mut new_careas: Vec<HocrCarea>) {
        // 1. Assign unique IDs recursively
        self.assign_unique_ids_to_careas(&mut new_careas);

        // 2. Merge or Insert logic
        if new_careas.len() == 1 {
            let new_carea = new_careas.pop().unwrap();
            self.careas[carea_index].blocks.extend(new_carea.blocks);
            self.careas[carea_index].rebuild_bbox();
        } else if new_careas.len() > 1 {
            self.insert_careas_after(carea_index, new_careas);
        }
    }
    pub fn insert_careas_after(&mut self, index: usize, new_careas: Vec<HocrCarea>) {
        if index < self.careas.len() {
            let mut tail = self.careas.split_off(index + 1);
            self.careas.extend(new_careas);
            self.careas.append(&mut tail);
        } else {
            self.careas.extend(new_careas);
        }
    }
    pub fn collect_all_words(&self) -> Vec<HocrWord> {
        self.careas
            .iter()
            .flat_map(|c| c.blocks.iter())
            .flat_map(|b| b.lines.iter())
            .flat_map(|l| l.words.iter())
            .cloned()
            .collect()
    }
    pub fn replace_words(&mut self, word_id: &str, mut with_words: Vec<HocrWord>) {
        if let Some(HocrPath::Word {
                        carea,
                        block,
                        line,
                        word,
                    }) = find_node(self, word_id)
        {
            // Assign unique IDs to new words
            let preferred_stem = stem_from_id(word_id);
            let mut next_number = self
                .get_next_number_with_stem(preferred_stem.as_str())
                .unwrap_or(1);

            for new_word in &mut with_words {
                new_word.id = format!("{}_{}", preferred_stem, next_number);
                next_number += 1;
            }

            // Remove the original word and insert new words at its position
            self.careas[carea].blocks[block].lines[line]
                .words
                .splice(word..word + 1, with_words);

            // Rebuild the bounding box of the containing line
            self.cleanup_line(carea, block, line);
        }
    }

    // -- ADDITIONS --

    pub fn add_carea(&mut self, bbox: HocrBbox, erase_underneath: Option<bool>, erase_overlap: Option<u8>) -> Result<String, String> {

        if erase_underneath.unwrap_or(false) && erase_overlap.is_some() {
            let erase_carea_ids: Vec<String> = self.careas
                .iter()
                .filter(|carea| bbox.overlap_percentage(carea.bbox).overlapping_other_pct as u8 >= erase_overlap.unwrap())
                .map(|carea| carea.id.clone())
                .collect();

            self.careas.retain(|carea| !erase_carea_ids.contains(&carea.id));
        }

        let vmid = bbox.center().1;

        let mut carea_index = None;
        for (i, carea) in self.careas.iter().enumerate() {
            if carea.bbox.center().1 > vmid {
                carea_index = Some(i);
                break;
            }
        }
        let carea_index = carea_index.unwrap_or(self.careas.len());

        let new_id = self.get_unique_id("carea", &mut HashMap::new());
        self.careas.insert(carea_index, HocrCarea {
            level: "carea".to_string(),
            id: new_id.clone(),
            bbox,
            flow: None,
            layout: None,
            blocks: vec![],
            unknowns: vec![],
        });

        self.rebuild_bbox();

        Ok(new_id)
    }
    pub fn add_block(&mut self, carea: Option<usize>, bbox: HocrBbox, block_type: Option<AddBlockType>, shrink_wrap_carea: Option<bool>, erase_underneath: Option<bool>, erase_overlap: Option<u8>) -> Result<String, String> {

        let new_id = match carea {

            None => {
                if erase_underneath.unwrap_or(false) && erase_overlap.is_some() {
                    let mut removals: HashMap<usize, Vec<String>> = HashMap::new();
                    let overlap_threshold = erase_overlap.unwrap();
                    for (c_idx, carea) in self.careas.iter().enumerate() {
                        let erase_block_ids: Vec<String> = carea.blocks
                            .iter()
                            .filter(|block| bbox.overlap_percentage(block.bbox).overlapping_other_pct as u8 >= overlap_threshold)
                            .map(|block| block.id.clone())
                            .collect();
                        if !erase_block_ids.is_empty() {
                            removals.insert(c_idx, erase_block_ids);
                        }
                    }

                    // Sort carea indices descending to avoid shifting issues when calling cleanup_carea
                    let mut affected_careas: Vec<usize> = removals.keys().cloned().collect();
                    affected_careas.sort_by(|a, b| b.cmp(a));

                    for c_idx in affected_careas {
                        if let Some(block_ids) = removals.get(&c_idx) {
                            self.careas[c_idx].blocks.retain(|block| !block_ids.contains(&block.id));
                            self.cleanup_carea(c_idx);
                        }
                    }
                }

                // Addition:
                // 1. Find the insertion index for a new carea based on the new block's vertical center.
                let vmid = bbox.center().1;
                let mut carea_index = None;
                for (i, carea) in self.careas.iter().enumerate() {
                    if carea.bbox.center().1 > vmid {
                        carea_index = Some(i);
                        break;
                    }
                }
                let carea_index = carea_index.unwrap_or(self.careas.len());

                // 2. Create a new HocrCarea with a unique ID and the new block's bbox.
                let new_carea_id = self.get_unique_id("carea", &mut HashMap::new());
                let new_block_id = self.get_unique_id("par", &mut HashMap::new());
                let block_kind = if block_type.unwrap_or(AddBlockType::Text) == AddBlockType::Image { HocrBlockKind::Image } else { HocrBlockKind::Paragraph };

                // 3. Create a new HocrBlock (Paragraph or Image kind) inside the new carea.
                let new_block = HocrBlock {
                    id: new_block_id.clone(),
                    level: "block".to_string(),
                    kind: block_kind,
                    lang: None,
                    bbox,
                    hints: HocrBlockHints::default(),
                    lines: vec![],
                };

                // 4. Insert the new carea into self.careas.
                self.careas.insert(carea_index, HocrCarea {
                    level: "carea".to_string(),
                    id: new_carea_id,
                    bbox,
                    flow: None,
                    layout: None,
                    blocks: vec![new_block],
                    unknowns: vec![],
                });

                Ok(new_block_id)
            }
            Some(carea) => {
                if erase_underneath.unwrap_or(false) && erase_overlap.is_some() {
                    let erase_block_ids: Vec<String> = self.careas[carea].blocks
                        .iter()
                        .filter(|block| bbox.overlap_percentage(block.bbox).overlapping_other_pct as u8 >= erase_overlap.unwrap())
                        .map(|block| block.id.clone())
                        .collect();

                    self.careas[carea].blocks.retain(|block| !erase_block_ids.contains(&block.id));
                    if !erase_block_ids.is_empty() {
                        self.cleanup_carea(carea);
                    }
                }

                let vmid = bbox.center().1;

                let mut block_index = None;

                for (i, block) in self.careas[carea].blocks.iter().enumerate() {
                    if block.bbox.center().1 > vmid {
                        block_index = Some(i);
                        break;
                    }
                }
                let block_index = block_index.unwrap_or(self.careas[carea].blocks.len());
                let new_id = self.get_unique_id("par", &mut HashMap::new());
                let block_kind = if block_type.unwrap_or(AddBlockType::Text) == AddBlockType::Image { HocrBlockKind::Image } else { HocrBlockKind::Paragraph };

                self.careas[carea].blocks.insert(block_index, HocrBlock {
                    id: new_id.clone(),
                    level: "block".to_string(),
                    kind: block_kind,
                    lang: None,
                    bbox,
                    hints: HocrBlockHints::default(),
                    lines: vec![],
                });

                if shrink_wrap_carea.unwrap_or(true) {
                    self.cleanup_carea(carea);
                };
                Ok(new_id)
            }
        };

        self.rebuild_bbox();

        new_id
    }
    pub fn add_line(&mut self, _carea: usize, _block: usize, _bbox: HocrBbox) {
        // Not implemented yet
    }
    pub fn add_word(&mut self, _carea: usize, _block: usize, _line: usize, _bbox: HocrBbox, _text: Option<String>) {
        // Not implemented yet
    }

    // -- REMOVALS --

    pub fn remove_carea(&mut self, carea: usize) {
        self.careas.remove(carea);
        self.rebuild_bbox();
    }
    pub fn remove_block(&mut self, carea: usize, block: usize) {
        self.careas[carea].blocks.remove(block);
        self.cleanup_carea(carea);
    }
    pub fn remove_line(&mut self, carea: usize, block: usize, line: usize) {
        self.careas[carea].blocks[block].lines.remove(line);
        self.cleanup_block(carea, block);
    }
    pub fn remove_word(&mut self, carea: usize, block: usize, line: usize, word: usize) {
        self.careas[carea].blocks[block].lines[line]
            .words
            .remove(word);
        self.cleanup_line(carea, block, line);
    }

    // -- CHANGES --

    #[allow(dead_code)]
    pub fn shift(&mut self, dx: i32, dy: i32) {
        self.bbox.shift(dx, dy);
        for carea in &mut self.careas {
            carea.shift(dx, dy);
        }
    }
    pub fn change_carea_flow(&mut self, carea_index: usize, flow: Option<String>) {
        if carea_index < self.careas.len() {
            self.careas[carea_index].flow = flow;
        }
    }
    pub fn change_carea_layout(&mut self, carea_index: usize, layout: Option<String>) {
        if carea_index < self.careas.len() {
            self.careas[carea_index].layout = layout;
        }
    }
    pub fn change_block_kind(&mut self, carea: usize, block: usize, kind: HocrBlockKind) {
        self.careas[carea].blocks[block].kind = kind;
        self.careas[carea].blocks[block].rebuild_bbox();
        self.cleanup_carea(carea);
    }
    pub fn toggle_block_hint(&mut self, carea: usize, block: usize, hint_name: &str) -> Result<(), String> {
        let block = &mut self.careas[carea].blocks[block];
        match hint_name {
            "continue_from_previous" => {
                let current = match block.hints.break_from_preceding {
                    Evidence::Assigned(b) => b,
                    Evidence::Determined(b) => b,
                    Evidence::Suggested(b) => b,
                    _ => false,
                };
                block.hints.break_from_preceding = Evidence::Assigned(!current);
            }
            "continue_to_following" => {
                let current = match block.hints.break_from_following {
                    Evidence::Assigned(b) => b,
                    Evidence::Determined(b) => b,
                    Evidence::Suggested(b) => b,
                    _ => false,
                };
                block.hints.break_from_following = Evidence::Assigned(!current);
            }
            _ => return Err("Invalid hint name".to_string()),
        }
        Ok(())
    }

    pub fn get_block_flow(&self, block_id: &str) -> Option<String> {
        for carea in &self.careas {
            for block in &carea.blocks {
                if block.id == block_id {
                    return carea.flow.clone();
                }
            }
        }
        None
    }

    pub fn filter_careas(&self, flow: &str) -> Vec<&HocrCarea> {
        self.careas
            .iter()
            .filter(|c| c.flow.as_deref() == Some(flow))
            .collect()
    }

    pub fn filter_blocks(&self, flow: &str) -> Vec<&HocrBlock> {
        self.filter_careas(flow)
            .into_iter()
            .flat_map(|c| c.blocks.iter())
            .collect()
    }
}

fn derive_evidence(tests: &[Evidence]) -> Evidence {
    let mut det_true = false;
    let mut det_false = false;
    let mut suggested = None;

    for t in tests {
        match t {
            Evidence::Determined(true) => det_true = true,
            Evidence::Determined(false) => det_false = true,
            Evidence::Suggested(b) => {
                if suggested.is_none() {
                    suggested = Some(*b);
                }
            }
            _ => {}
        }
    }

    if det_true && det_false {
        Evidence::Error
    } else if det_true {
        Evidence::Determined(true)
    } else if det_false {
        Evidence::Determined(false)
    } else if let Some(b) = suggested {
        Evidence::Suggested(b)
    } else {
        Evidence::Undetermined
    }
}

impl HocrBlock {
    pub fn x_indent(&self) -> i32 {
        if self.lines.is_empty() {
            return 0;
        }
        self.lines[0].bbox.left() - self.bbox.left()
    }

    pub fn x_dedent(&self) -> i32 {
        if self.lines.is_empty() {
            return 0;
        }
        self.bbox.right() - self.lines.last().unwrap().bbox.right()
    }

    pub fn test_hyphenation(&self) -> bool {
        if self.lines.is_empty() {
            return false;
        }
        let last_line = self.lines.last().unwrap();
        if last_line.words.is_empty() {
            return false;
        }
        let last_word = last_line.words.last().unwrap();
        last_word.text.ends_with('-')
            || last_word.text.ends_with('—')
            || last_word.text.ends_with('–')
    }

    pub fn apply_auto_detection(
        &mut self,
        preceding: Option<&HocrBlock>,
        following: Option<&HocrBlock>,
        thresholds: &DetectionThresholds,
    ) {
        // 1. Low level indicators - per block
        if thresholds.use_x_indent && !matches!(self.hints.test_x_indent, Evidence::Assigned(_)) {
            let val = self.x_indent();
            self.hints.test_x_indent = if val >= thresholds.x_indent_max {
                Evidence::Determined(true)
            } else if val < thresholds.x_indent_min {
                Evidence::Determined(false)
            } else {
                Evidence::Suggested(true)
            };
        }

        if thresholds.use_x_dedent && !matches!(self.hints.test_x_dedent, Evidence::Assigned(_)) {
            let val = self.x_dedent();
            self.hints.test_x_dedent = if val >= thresholds.x_dedent_max {
                Evidence::Determined(true)
            } else if val < thresholds.x_dedent_min {
                Evidence::Determined(false)
            } else {
                Evidence::Suggested(true)
            };
        }

        if thresholds.use_hyphenation
            && !matches!(self.hints.test_hyphenation, Evidence::Assigned(_))
        {
            self.hints.test_hyphenation = if self.test_hyphenation() {
                Evidence::Suggested(true)
            } else {
                Evidence::Suggested(false)
            };
        }

        // 2. Low level indicators - per boundary
        if thresholds.use_y_advance {
            if let Some(prev) = preceding {
                if !matches!(self.hints.test_y_advance, Evidence::Assigned(_)) {
                    let val = y_advance(prev, self);
                    self.hints.test_y_advance = if val >= thresholds.y_advance_max {
                        Evidence::Determined(true)
                    } else if val < thresholds.y_advance_min {
                        Evidence::Determined(false)
                    } else {
                        Evidence::Suggested(true)
                    };
                }
            }

            if let Some(next) = following {
                if !matches!(self.hints.test_y_reverse, Evidence::Assigned(_)) {
                    let val = y_advance(self, next);
                    self.hints.test_y_reverse = if val >= thresholds.y_advance_max {
                        Evidence::Determined(true)
                    } else if val < thresholds.y_advance_min {
                        Evidence::Determined(false)
                    } else {
                        Evidence::Suggested(true)
                    };
                }
            }
        }

        // 3. Mid level calculations
        if !matches!(self.hints.break_from_preceding, Evidence::Assigned(_)) {
            self.hints.break_from_preceding =
                derive_evidence(&[self.hints.test_x_indent, self.hints.test_y_advance]);
        }

        if !matches!(self.hints.break_from_following, Evidence::Assigned(_)) {
            // Use test_y_reverse here because it's the space between this block and the following one
            self.hints.break_from_following =
                derive_evidence(&[self.hints.test_x_dedent, self.hints.test_y_reverse]);
        }
    }

    pub fn get_continued_evidence(a: &HocrBlock, b: &HocrBlock) -> Evidence {
        let a_break = a.hints.break_from_following.is_true();
        let b_break = b.hints.break_from_preceding.is_true();

        match (a_break, b_break) {
            (Some(false), Some(false)) => Evidence::Determined(true),
            (Some(true), Some(true)) => Evidence::Determined(false),
            (Some(av), Some(bv)) if av != bv => Evidence::Error,
            _ => Evidence::Undetermined,
        }
    }
}

pub fn y_advance(a: &HocrBlock, b: &HocrBlock) -> i32 {
    b.bbox.top() - a.bbox.bottom()
}
