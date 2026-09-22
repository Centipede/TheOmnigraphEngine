pub use std::collections::HashMap;

use super::*;

const SAMPLE1: &str = r#"
        <html>
            <body>
                <div class='ocr_page' id='page_1' title='image "/tmp/side-014.jpg"; bbox 0 0 2537 3160; ppageno 0; scan_res 320 320'>
                   <div class='ocr_carea' id='block_1_1' title="bbox 483 286 1645 422">
                    <p class='ocr_par' id='par_1_1' lang='eng' title="bbox 483 286 1645 422">
                     <span class='ocr_line' id='line_1_1' title="bbox 483 286 1645 336; baseline -0.004 0; x_size 59.625; x_descenders 14.90625; x_ascenders 14.90625">
                      <span class='ocrx_word' id='word_1_1' title='bbox 483 290 586 336; x_wconf 96'>Line1-Word1</span>
                      <span class='ocrx_word' id='word_1_2' title='bbox 604 290 926 335; x_wconf 93'>Line1-Word2</span>
                      <span class='ocrx_word' id='word_1_3' title='bbox 951 290 1025 335; x_wconf 28'>Line1-Word3</span>
                      <span class='ocrx_word' id='word_1_4' title='bbox 1077 286 1645 334; x_wconf 91'>Line1-Word4</span>
                     </span>
                     <span class='ocr_line' id='line_1_2' title="bbox 483 361 1256 422; baseline -0.003 -14; x_size 59.625; x_descenders 14.90625; x_ascenders 14.90625">
                      <span class='ocrx_word' id='word_1_5' title='bbox 483 363 1005 408; x_wconf 96'>Line2-Word1</span>
                      <span class='ocrx_word' id='word_1_6' title='bbox 1024 361 1256 422; x_wconf 95'>Line2-Word2</span>
                     </span>
                    </p>
                   </div>
                </div>
            </body>
        </html>
    "#;

const SAMPLE2: &str = r#"
        <html>
            <body>
                <div class='ocr_page' id='page_1' title='image "/tmp/side-014.jpg"; bbox 0 0 2537 3160; ppageno 0; scan_res 320 320'>

                   <div class='ocr_carea' id='block_1_1' title="bbox 483 286 1645 422">
                    <p class='ocr_par' id='par_1_1' lang='eng' title="bbox 483 286 1645 422">
                     <span class='ocr_line' id='line_1_1' title="bbox 483 286 1645 336; baseline -0.004 0; x_size 59.625; x_descenders 14.90625; x_ascenders 14.90625">
                      <span class='ocrx_word' id='word_1_1' title='bbox 483 290 586 336; x_wconf 96'>Line1-Word1</span>
                      <span class='ocrx_word' id='word_1_2' title='bbox 604 290 926 335; x_wconf 93'>Line1-Word2</span>
                      <span class='ocrx_word' id='word_1_3' title='bbox 951 290 1025 335; x_wconf 28'>Line1-Word3</span>
                      <span class='ocrx_word' id='word_1_4' title='bbox 1077 286 1645 334; x_wconf 91'>Line1-Word4</span>
                     </span>
                     <span class='ocr_line' id='line_1_2' title="bbox 483 361 1256 422; baseline -0.003 -14; x_size 59.625; x_descenders 14.90625; x_ascenders 14.90625">
                      <span class='ocrx_word' id='word_1_5' title='bbox 483 363 1005 408; x_wconf 96'>Line2-Word1</span>
                      <span class='ocrx_word' id='word_1_6' title='bbox 1024 361 1256 422; x_wconf 95'>Line2-Word2</span>
                     </span>
                    </p>
                   </div>

                   <div class='ocr_carea' id='block_1_2' title="bbox 485 440 2298 785">
                    <p class='ocr_par' id='par_1_2' lang='eng' title="bbox 485 440 2298 785">
                     <span class='ocr_line' id='line_1_3' title="bbox 485 440 2298 492; baseline -0.004 -11; x_size 46; x_descenders 11; x_ascenders 12">
                      <span class='ocrx_word' id='word_1_7' title='bbox 485 446 625 492; x_wconf 96'>Line3-Word1</span>
                      <span class='ocrx_word' id='word_1_8' title='bbox 642 446 702 480; x_wconf 96'>Line3-Word2</span>
                      <span class='ocrx_word' id='word_1_9' title='bbox 720 445 961 490; x_wconf 96'>Line3-Word3</span>
                     </span>

                     <span class='ocr_line' id='line_1_4' title="bbox 486 499 2295 549; baseline -0.004 -9; x_size 46; x_descenders 11; x_ascenders 12">
                      <span class='ocrx_word' id='word_1_10' title='bbox 486 505 632 540; x_wconf 91'>Line4-Word1</span>
                      <span class='ocrx_word' id='word_1_11' title='bbox 645 504 709 540; x_wconf 96'>Line4-Word2</span>
                      <span class='ocrx_word' id='word_1_12' title='bbox 723 505 842 539; x_wconf 96'>Line4-Word3</span>
                     </span>
                    </p>
                   </div>
                   </div>
            </body>
        </html>
    "#;

const SAMPLE_3CAREAS: &str = r#"
        <html>
            <body>
                <div class='ocr_page' id='page_1' title='bbox 0 0 1000 1000'>
                   <div class='ocr_carea' id='carea_1' title="bbox 10 10 100 100">
                     <p class='ocr_par' id='par_1' title="bbox 10 10 100 100">
                       <span class='ocr_line' id='line_1' title="bbox 10 10 100 50">
                         <span class='ocrx_word' id='word_1' title='bbox 10 10 50 50'>Word1</span>
                       </span>
                     </p>
                   </div>
                   <div class='ocr_carea' id='carea_2' title="bbox 110 110 200 200">
                     <p class='ocr_par' id='par_2' title="bbox 110 110 200 200">
                       <span class='ocr_line' id='line_2' title="bbox 110 110 200 150">
                         <span class='ocrx_word' id='word_2' title='bbox 110 110 150 150'>Word2</span>
                       </span>
                     </p>
                   </div>
                   <div class='ocr_carea' id='carea_3' title="bbox 210 210 300 300">
                     <p class='ocr_par' id='par_3' title="bbox 210 210 300 300">
                       <span class='ocr_line' id='line_3' title="bbox 210 210 300 250">
                         <span class='ocrx_word' id='word_3' title='bbox 210 210 250 250'>Word3</span>
                       </span>
                     </p>
                   </div>
                </div>
            </body>
        </html>
"#;

const SAMPLE_COMPLEX: &str = r#"
        <html>
            <body>
                <div class='ocr_page' id='page_1' title='bbox 0 0 1000 1000'>
                   <div class='ocr_carea' id='carea_1' title="bbox 10 10 100 100">
                     <p class='ocr_par' id='par_1' title="bbox 10 10 100 50">
                       <span class='ocr_line' id='line_1' title="bbox 10 10 100 30">
                         <span class='ocrx_word' id='word_1' title='bbox 10 10 50 30'>W1</span>
                       </span>
                     </p>
                     <p class='ocr_par' id='par_2' title="bbox 10 60 100 100">
                       <span class='ocr_line' id='line_2' title="bbox 10 60 100 80">
                         <span class='ocrx_word' id='word_2' title='bbox 10 60 50 80'>W2</span>
                       </span>
                     </p>
                   </div>
                   <div class='ocr_carea' id='carea_2' title="bbox 110 110 200 200">
                     <p class='ocr_par' id='par_3' title="bbox 110 110 200 150">
                       <span class='ocr_line' id='line_3' title="bbox 110 110 200 130">
                         <span class='ocrx_word' id='word_3' title='bbox 110 110 150 130'>W3</span>
                       </span>
                     </p>
                     <p class='ocr_par' id='par_4' title="bbox 110 160 200 200">
                       <span class='ocr_line' id='line_4' title="bbox 110 160 200 180">
                         <span class='ocrx_word' id='word_4' title='bbox 110 160 150 180'>W4</span>
                       </span>
                     </p>
                   </div>
                </div>
            </body>
        </html>
"#;


fn signature_word(word: &HocrWord) -> String {
    word.id.clone()
}

fn signature_line(line: &HocrLine) -> String {
    let word_sigs = line.words.iter().map(signature_word).collect::<Vec<String>>();
    format!("{}({})", line.id.clone(), word_sigs.join(","))
}

fn signature_block(block: &HocrBlock) -> String {
    let line_sigs = block.lines.iter().map(signature_line).collect::<Vec<String>>();
    format!("{}:{}({})", block.id.clone(), block.kind, line_sigs.join(","))
}

fn signature_carea(carea: &HocrCarea) -> String {
    let block_sigs = carea.blocks.iter().map(signature_block).collect::<Vec<String>>();
    format!("{}({})", carea.id.clone(), block_sigs.join(","))
}

fn signature(page: &HocrPage) -> String {
    let carea_sigs = page.careas.iter().map(signature_carea).collect::<Vec<String>>();
    format!("{}({})", page.page_id, carea_sigs.join(","))
}

fn to_sig(s: &str) -> String {
    s.replace(' ', "").replace('\n', "").replace('\r', "")
}


#[test]
fn stem_from_id_removed_trailing_digits() {
    assert_eq!(stem_from_id(""), "");
    assert_eq!(stem_from_id("page_1"), "page");
    assert_eq!(stem_from_id("line_1_100"), "line_1");
    assert_eq!(stem_from_id("page_1_shouldnotbe"), "page_1_shouldnotbe");
    assert_eq!(stem_from_id("id___4"), "id");
    assert_eq!(stem_from_id("id__1__4"), "id__1");
}

#[test]
fn count_from_id_correct_number() {
    assert_eq!(count_from_id("page_1"), Ok(1));
    assert_eq!(count_from_id("line_1_100"), Ok(100));
    assert!(count_from_id("linebad").is_err());
}

#[test]
fn next_unique_id() {

    let page = parse(SAMPLE1, ParserConfig::default()).unwrap();

    assert_eq!(page.page_id, "page_1");
    assert_eq!(page.get_next_number_with_stem("block"), Some(2));
    assert_eq!(page.get_next_number_with_stem("par"), Some(2));
    assert_eq!(page.get_next_number_with_stem("line"), Some(3));
    assert_eq!(page.get_next_number_with_stem("word"), Some(7));
    assert_eq!(page.get_unique_id("par_1_1", &mut HashMap::new()), "par_1_2");
    assert_eq!(page.get_unique_id("line_1_2", &mut HashMap::new()), "line_1_3");
    assert_eq!(page.get_unique_id("word_1_1", &mut HashMap::new()), "word_1_7");
}

#[test]
fn signatures_match() {
    let page = parse(SAMPLE1, ParserConfig::default()).unwrap();
    let sig = signature(&page);
    assert_eq!(sig, to_sig(r#"page_1(
        block_1_1(
            par_1_1:P(
                line_1_1(word_1_1, word_1_2, word_1_3, word_1_4),
                line_1_2(word_1_5, word_1_6)))
    )"#));

    let page = parse(SAMPLE2, ParserConfig::default()).unwrap();
    let sig = signature(&page);
    assert_eq!(sig, to_sig(r#"page_1(
        block_1_1(
            par_1_1:P(
                line_1_1(word_1_1, word_1_2, word_1_3, word_1_4),
                line_1_2(word_1_5, word_1_6)
            )
        ),
        block_1_2(
            par_1_2:P(
                line_1_3(word_1_7, word_1_8, word_1_9),
                line_1_4(word_1_10, word_1_11, word_1_12)
            )
        )
    )"#));
}

#[test]
fn locating_items() {
    let page = parse(SAMPLE2, ParserConfig::default()).unwrap();
    assert_eq!(find_node(&page, "line_1_1"), Some(HocrPath::Line {carea:0, block:0, line:0}))
}

#[test]
fn move_line_up() {
    let mut page = parse(SAMPLE2, ParserConfig::default()).unwrap();
    let orig_sig = signature(&page);
    page.move_line_up(0,0,0);
    assert_eq!(signature(&page), orig_sig);

    page.move_line_up(1,0,0);

    assert_eq!(signature(&page), to_sig(r#"page_1(
        block_1_1(
            par_1_1:P(
                line_1_1(word_1_1, word_1_2, word_1_3, word_1_4),
                line_1_2(word_1_5, word_1_6),
                line_1_3(word_1_7, word_1_8, word_1_9)
            )
        ),
        block_1_2(
            par_1_2:P(
                line_1_4(word_1_10, word_1_11, word_1_12)
            )
        )
    )"#));

}

#[test]
fn move_line_down() {
    let mut page = parse(SAMPLE2, ParserConfig::default()).unwrap();

    // Move line_1_1 down within the same block
    page.move_line_down(0, 0, 0);
    assert_eq!(signature(&page), to_sig(r#"page_1(
        block_1_1(
            par_1_1:P(
                line_1_2(word_1_5, word_1_6),
                line_1_1(word_1_1, word_1_2, word_1_3, word_1_4)
            )
        ),
        block_1_2(
            par_1_2:P(
                line_1_3(word_1_7, word_1_8, word_1_9),
                line_1_4(word_1_10, word_1_11, word_1_12)
            )
        )
    )"#));

    // Reset page
    let mut page = parse(SAMPLE2, ParserConfig::default()).unwrap();

    // Move line_1_2 down to next block
    page.move_line_down(0, 0, 1);
    assert_eq!(signature(&page), to_sig(r#"page_1(
        block_1_1(
            par_1_1:P(
                line_1_1(word_1_1, word_1_2, word_1_3, word_1_4)
            )
        ),
        block_1_2(
            par_1_2:P(
                line_1_2(word_1_5, word_1_6),
                line_1_3(word_1_7, word_1_8, word_1_9),
                line_1_4(word_1_10, word_1_11, word_1_12)
            )
        )
    )"#));
}

#[test]
fn merge_carea() {
    let mut page = parse(SAMPLE2, ParserConfig::default()).unwrap();

    // Merge block_1_2 into block_1_1 (carea 1 into carea 0)
    page.merge_carea(0, 1);
    assert_eq!(signature(&page), to_sig(r#"page_1(
        block_1_1(
            par_1_1:P(
                line_1_1(word_1_1, word_1_2, word_1_3, word_1_4),
                line_1_2(word_1_5, word_1_6)
            ),
            par_1_2:P(
                line_1_3(word_1_7, word_1_8, word_1_9),
                line_1_4(word_1_10, word_1_11, word_1_12)
            )
        )
    )"#));
}

#[test]
fn merge_block() {
    let mut page = parse(SAMPLE2, ParserConfig::default()).unwrap();
    // First merge careas so we have two blocks in one carea
    page.merge_carea(0, 1);

    // Now merge par_1_2 into par_1_1 (block 1 into block 0)
    page.merge_block(0, 0, 1);
    assert_eq!(signature(&page), to_sig(r#"page_1(
        block_1_1(
            par_1_1:P(
                line_1_1(word_1_1, word_1_2, word_1_3, word_1_4),
                line_1_2(word_1_5, word_1_6),
                line_1_3(word_1_7, word_1_8, word_1_9),
                line_1_4(word_1_10, word_1_11, word_1_12)
            )
        )
    )"#));
}

#[test]
fn test_merge_careas_success() {
    // Test 2 careas
    let mut page = parse(SAMPLE_3CAREAS, ParserConfig::default()).unwrap();
    page.merge_careas(&mut vec![0, 1]).unwrap();
    assert_eq!(signature(&page), to_sig(r#"page_1(
        carea_1(par_1:P(line_1(word_1)),par_2:P(line_2(word_2))),
        carea_3(par_3:P(line_3(word_3)))
    )"#));

    // Test 3 careas
    let mut page = parse(SAMPLE_3CAREAS, ParserConfig::default()).unwrap();
    page.merge_careas(&mut vec![0, 1, 2]).unwrap();
    assert_eq!(signature(&page), to_sig(r#"page_1(
        carea_1(par_1:P(line_1(word_1)),par_2:P(line_2(word_2)),par_3:P(line_3(word_3)))
    )"#));
}

#[test]
fn test_merge_careas_failures() {
    let mut page = parse(SAMPLE_3CAREAS, ParserConfig::default()).unwrap();
    
    // Less than 2
    assert!(page.merge_careas(&mut vec![0]).is_err());
    
    // Non-consecutive
    assert!(page.merge_careas(&mut vec![0, 2]).is_err());
}

#[test]
fn test_merge_blocks_success() {
    // Test 2 blocks in same carea
    let mut page = parse(SAMPLE_COMPLEX, ParserConfig::default()).unwrap();
    let _orig_sig = signature(&page);
    page.merge_blocks(&mut vec![(0, 0), (0, 1)]).unwrap();
    assert_eq!(signature(&page), to_sig(r#"page_1(
        carea_1(par_1:P(line_1(word_1),line_2(word_2))),
        carea_2(par_3:P(line_3(word_3)),par_4:P(line_4(word_4)))
    )"#));

    // Test 2 blocks across careas
    let mut page = parse(SAMPLE_COMPLEX, ParserConfig::default()).unwrap();
    page.merge_blocks(&mut vec![(0, 1), (1, 0)]).unwrap();
    assert_eq!(signature(&page), to_sig(r#"page_1(
        carea_1(par_1:P(line_1(word_1)),par_2:P(line_2(word_2),line_3(word_3))),
        carea_2(par_4:P(line_4(word_4)))
    )"#));

    // Test 3 blocks
    let mut page = parse(SAMPLE_COMPLEX, ParserConfig::default()).unwrap();
    page.merge_blocks(&mut vec![(0, 0), (0, 1), (1, 0)]).unwrap();
    assert_eq!(signature(&page), to_sig(r#"page_1(
        carea_1(par_1:P(line_1(word_1),line_2(word_2),line_3(word_3))),
        carea_2(par_4:P(line_4(word_4)))
    )"#));
}

#[test]
fn test_merge_blocks_failures() {
    let mut page = parse(SAMPLE_COMPLEX, ParserConfig::default()).unwrap();
    
    // Less than 2
    assert!(page.merge_blocks(&mut vec![(0, 0)]).is_err());
    
    // Non-consecutive
    assert!(page.merge_blocks(&mut vec![(0, 0), (1, 0)]).is_err());
}

#[test]
fn test_merge_blocks_cleanup() {
    let mut page = parse(SAMPLE_COMPLEX, ParserConfig::default()).unwrap();
    
    // Merge all blocks from carea_2 into carea_1
    page.merge_blocks(&mut vec![(0, 1), (1, 0), (1, 1)]).unwrap();
    
    // carea_2 should be removed because it's empty
    assert_eq!(signature(&page), to_sig(r#"page_1(
        carea_1(par_1:P(line_1(word_1)),par_2:P(line_2(word_2),line_3(word_3),line_4(word_4)))
    )"#));
    assert_eq!(page.careas.len(), 1);
}

#[test]
fn add_block_none_carea_no_erase() {
    let mut page = parse(SAMPLE1, ParserConfig::default()).unwrap();
    let bbox = HocrBbox([483, 500, 1645, 600]);
    let _ = page.add_block(None, bbox, Some(AddBlockType::Text), None, Some(false), None);

    // Should have 2 careas now
    assert_eq!(page.careas.len(), 2);
    // The new carea should be at index 1 because its center Y (550) is greater than block_1_1 center Y (~354)
    assert_eq!(page.careas[1].bbox, bbox);
    assert_eq!(page.careas[1].blocks.len(), 1);
    assert_eq!(page.careas[1].blocks[0].bbox, bbox);
}

#[test]
fn add_block_none_carea_with_erase() {
    let mut page = parse(SAMPLE1, ParserConfig::default()).unwrap();
    // This bbox overlaps with both lines of block_1_1
    let bbox = HocrBbox([483, 280, 1645, 430]);
    let _ = page.add_block(None, bbox, Some(AddBlockType::Text), None, Some(true), Some(50));

    // block_1_1 should have been erased because its blocks overlap with the new bbox
    // Since all blocks in block_1_1 are erased, block_1_1 itself should be removed
    assert_eq!(page.careas.len(), 1);
    assert_eq!(page.careas[0].bbox, bbox);
    assert_eq!(page.careas[0].blocks.len(), 1);
    assert_eq!(page.careas[0].id.starts_with("carea"), true);
}

#[test]
fn add_block_none_carea_vertical_positioning() {
    let mut page = parse(SAMPLE1, ParserConfig::default()).unwrap();
    // Add one above
    let bbox_above = HocrBbox([483, 100, 1645, 200]);
    let _ = page.add_block(None, bbox_above, None, None, None, None);

    // Add one below
    let bbox_below = HocrBbox([483, 500, 1645, 600]);
    let _ = page.add_block(None, bbox_below, None, None, None, None);

    assert_eq!(page.careas.len(), 3);
    assert_eq!(page.careas[0].bbox, bbox_above);
    assert_eq!(page.careas[2].bbox, bbox_below);
}

#[test]
fn add_block_with_shrink_wrap_false() {
    let mut page = parse(SAMPLE1, ParserConfig::default()).unwrap();
    let original_carea_bbox = page.careas[0].bbox;

    // Add a block that is outside the current carea bbox
    let new_block_bbox = HocrBbox([original_carea_bbox.0[2] + 10, original_carea_bbox.0[1], original_carea_bbox.0[2] + 100, original_carea_bbox.0[3]]);

    // Add to carea 0 with shrink_wrap_carea = false
    let _ = page.add_block(Some(0), new_block_bbox, None, Some(false), None, None);

    // Carea bbox should remain the same
    assert_eq!(page.careas[0].bbox, original_carea_bbox);
    // Block should be added
    assert_eq!(page.careas[0].blocks.len(), 2);
}

#[test]
fn add_block_with_shrink_wrap_true() {
    let mut page = parse(SAMPLE1, ParserConfig::default()).unwrap();
    let original_carea_bbox = page.careas[0].bbox;

    // Add a block that is outside the current carea bbox
    let new_block_bbox = HocrBbox([original_carea_bbox.0[2] + 10, original_carea_bbox.0[1], original_carea_bbox.0[2] + 100, original_carea_bbox.0[3]]);

    // Add to carea 0 with shrink_wrap_carea = true (default)
    let _ = page.add_block(Some(0), new_block_bbox, None, Some(true), None, None);

    // Carea bbox should have changed (it should now include the new block)
    assert_ne!(page.careas[0].bbox, original_carea_bbox);
    assert_eq!(page.careas[0].blocks.len(), 2);
}

#[test]
fn test_coordinate_shifting() {
    let mut page = parse(SAMPLE1, ParserConfig::default()).unwrap();
    let dx = 100;
    let dy = 200;

    let original_page_bbox = page.bbox;
    let original_carea_bbox = page.careas[0].bbox;
    let original_block_bbox = page.careas[0].blocks[0].bbox;
    let original_line_bbox = page.careas[0].blocks[0].lines[0].bbox;
    let original_word_bbox = page.careas[0].blocks[0].lines[0].words[0].bbox;

    page.shift(dx, dy);

    assert_eq!(page.bbox.0, [original_page_bbox.0[0] + dx, original_page_bbox.0[1] + dy, original_page_bbox.0[2] + dx, original_page_bbox.0[3] + dy]);
    assert_eq!(page.careas[0].bbox.0, [original_carea_bbox.0[0] + dx, original_carea_bbox.0[1] + dy, original_carea_bbox.0[2] + dx, original_carea_bbox.0[3] + dy]);
    assert_eq!(page.careas[0].blocks[0].bbox.0, [original_block_bbox.0[0] + dx, original_block_bbox.0[1] + dy, original_block_bbox.0[2] + dx, original_block_bbox.0[3] + dy]);
    assert_eq!(page.careas[0].blocks[0].lines[0].bbox.0, [original_line_bbox.0[0] + dx, original_line_bbox.0[1] + dy, original_line_bbox.0[2] + dx, original_line_bbox.0[3] + dy]);
    assert_eq!(page.careas[0].blocks[0].lines[0].words[0].bbox.0, [original_word_bbox.0[0] + dx, original_word_bbox.0[1] + dy, original_word_bbox.0[2] + dx, original_word_bbox.0[3] + dy]);
}

#[test]
fn test_insert_careas_after() {
    let mut page = parse(SAMPLE_3CAREAS, ParserConfig::default()).unwrap();
    let new_carea = page.careas[0].clone();
    let mut new_careas = vec![new_carea];
    new_careas[0].id = "new_carea".to_string();

    page.insert_careas_after(0, new_careas);

    assert_eq!(page.careas.len(), 4);
    assert_eq!(page.careas[1].id, "new_carea");
    assert_eq!(page.careas[0].id, "carea_1");
    assert_eq!(page.careas[2].id, "carea_2");
}

#[test]
fn test_merge_lines_success() {
    let mut page = parse(SAMPLE2, ParserConfig::default()).unwrap();
    // par_1_1 has line_1_1 and line_1_2.
    // indices for par_1_1: carea 0, block 0.
    // lines: (0, 0, 0) and (0, 0, 1).

    let mut lines = vec![(0, 0, 0), (0, 0, 1)];
    page.merge_lines(&mut lines).unwrap();

    assert_eq!(page.careas[0].blocks[0].lines.len(), 1);
    // line_1_1 had 4 words, line_1_2 had 2 words. Total 6.
    assert_eq!(page.careas[0].blocks[0].lines[0].words.len(), 6);
}

#[test]
fn test_merge_words_success() {
    let mut page = parse(SAMPLE2, ParserConfig::default()).unwrap();
    // line_1_1 has words 1, 2, 3, 4.
    // indices: (0, 0, 0, 0), (0, 0, 0, 1).

    let mut words = vec![(0, 0, 0, 0), (0, 0, 0, 1)];
    page.merge_words(&mut words).unwrap();

    assert_eq!(page.careas[0].blocks[0].lines[0].words.len(), 3);
    assert_eq!(
        page.careas[0].blocks[0].lines[0].words[0].text,
        "Line1-Word1 Line1-Word2"
    );
}

#[test]
fn test_merge_words_across_lines() {
    let mut page = parse(SAMPLE2, ParserConfig::default()).unwrap();
    // word_1_4 (last of line 1) and word_1_5 (first of line 2) are consecutive.
    // word_1_4: (0, 0, 0, 3)
    // word_1_5: (0, 0, 1, 0)

    let mut words = vec![(0, 0, 0, 3), (0, 0, 1, 0)];
    page.merge_words(&mut words).unwrap();

    assert_eq!(page.careas[0].blocks[0].lines[0].words.len(), 4);
    assert_eq!(
        page.careas[0].blocks[0].lines[0].words[3].text,
        "Line1-Word4 Line2-Word1"
    );
    // Line 2 should now have only 1 word remaining (word_1_6)
    assert_eq!(page.careas[0].blocks[0].lines[1].words.len(), 1);
}

#[test]
fn test_merge_words_cleanup_line() {
    let mut page = parse(SAMPLE2, ParserConfig::default()).unwrap();
    // Merge all words of line_1_2 into line_1_1
    // line_1_2 words: word_1_5 (0,0,1,0), word_1_6 (0,0,1,1)
    // line_1_1 words: word_1_1 (0,0,0,0) to word_1_4 (0,0,0,3)

    let mut words = vec![(0, 0, 0, 3), (0, 0, 1, 0), (0, 0, 1, 1)];
    page.merge_words(&mut words).unwrap();

    assert_eq!(page.careas[0].blocks[0].lines[0].words.len(), 4);
    // line_1_2 should have been removed because all its words were moved
    assert_eq!(page.careas[0].blocks[0].lines.len(), 1);
}

#[test]
fn test_carea_metadata_parsing() {
    let html = r#"
        <div class="ocr_page" id="page_1" title="bbox 0 0 1000 1000">
            <div class="ocr_carea" id="carea_1" title="bbox 10 10 100 100; flow footnotes; layout center">
            </div>
            <div class="ocr_carea" id="carea_2" title="bbox 110 110 200 200; flow; layout">
            </div>
        </div>
    "#;
    let page = parse(html, ParserConfig::default()).unwrap();
    assert_eq!(page.careas[0].flow, Some("footnotes".to_string()));
    assert_eq!(page.careas[0].layout, Some("center".to_string()));
    assert_eq!(page.careas[1].flow, Some("".to_string()));
    assert_eq!(page.careas[1].layout, Some("".to_string()));
}

#[test]
fn test_carea_metadata_serialization() {
    let carea = HocrCarea {
        level: "carea".to_string(),
        id: "carea_1".to_string(),
        bbox: HocrBbox::new(10, 10, 100, 100),
        flow: Some("main".to_string()),
        layout: Some("left".to_string()),
        blocks: vec![],
        unknowns: vec![],
    };
    let html = carea.to_hocr_html();
    assert!(html.contains("flow main"));
    assert!(html.contains("layout left"));

    let carea_empty = HocrCarea {
        level: "carea".to_string(),
        id: "carea_2".to_string(),
        bbox: HocrBbox::new(10, 10, 100, 100),
        flow: Some("".to_string()),
        layout: Some("".to_string()),
        blocks: vec![],
        unknowns: vec![],
    };
    let html_empty = carea_empty.to_hocr_html();
    assert!(html_empty.contains("; flow"));
    assert!(!html_empty.contains("flow "));
    assert!(html_empty.contains("; layout"));
    assert!(!html_empty.contains("layout "));
}

#[test]
fn test_carea_metadata_defaulting() {
    let html = r#"
        <div class="ocr_page" id="page_1" title="bbox 0 0 1000 1000">
            <div class="ocr_carea" id="carea_1" title="bbox 10 10 100 100">
            </div>
        </div>
    "#;
    let page = parse(html, ParserConfig::default()).unwrap();
    assert_eq!(page.careas[0].flow, None);
    assert_eq!(page.careas[0].layout, None);
}

#[test]
fn test_split_carea_metadata_preservation() {
    let carea = HocrCarea {
        level: "carea".to_string(),
        id: "carea_1".to_string(),
        bbox: HocrBbox::new(0, 0, 100, 200),
        flow: Some("special".to_string()),
        layout: Some("right".to_string()),
        blocks: vec![
            HocrBlock {
                level: "block".to_string(),
                id: "b1".to_string(),
                bbox: HocrBbox::new(0, 0, 100, 100),
                kind: HocrBlockKind::Paragraph,
                lang: None,
                hints: HocrBlockHints::default(),
                metrics: None,
                lines: vec![],
            },
            HocrBlock {
                level: "block".to_string(),
                id: "b2".to_string(),
                bbox: HocrBbox::new(0, 101, 100, 200),
                kind: HocrBlockKind::Paragraph,
                lang: None,
                hints: HocrBlockHints::default(),
                metrics: None,
                lines: vec![],
            },
        ],
        unknowns: vec![],
    };
    let mut page = HocrPage {
        level: "page".to_string(),
        page_id: "p1".to_string(),
        bbox: HocrBbox::new(0, 0, 1000, 1000),
        careas: vec![carea],
        unknowns: vec![],
    };

    page.split_carea(0, 0, 1);

    assert_eq!(page.careas.len(), 2);
    assert_eq!(page.careas[0].flow, Some("special".to_string()));
    assert_eq!(page.careas[0].layout, Some("right".to_string()));
    assert_eq!(page.careas[1].flow, Some("special".to_string()));
    assert_eq!(page.careas[1].layout, Some("right".to_string()));
}

#[test]
fn test_add_carea_metadata_initialization() {
    let mut page = HocrPage {
        level: "page".to_string(),
        page_id: "p1".to_string(),
        bbox: HocrBbox::new(0, 0, 1000, 1000),
        careas: vec![],
        unknowns: vec![],
    };
    page.add_carea(HocrBbox::new(10, 10, 100, 100), None, None).unwrap();
    assert_eq!(page.careas[0].flow, None);
    assert_eq!(page.careas[0].layout, None);
}

#[test]
fn test_add_block_new_carea_metadata_initialization() {
    let mut page = HocrPage {
        level: "page".to_string(),
        page_id: "p1".to_string(),
        bbox: HocrBbox::new(0, 0, 1000, 1000),
        careas: vec![],
        unknowns: vec![],
    };
    page.add_block(None, HocrBbox::new(10, 10, 100, 100), None, None, None, None).unwrap();
    assert_eq!(page.careas[0].flow, None);
    assert_eq!(page.careas[0].layout, None);
}

#[test]
fn test_auto_layout_default_flow() {
    let mut page = HocrPage {
        level: "page".to_string(),
        page_id: "p1".to_string(),
        bbox: HocrBbox::new(0, 0, 100, 100),
        careas: vec![
            HocrCarea {
                level: "carea".to_string(),
                id: "c1".to_string(),
                bbox: HocrBbox::new(0, 0, 10, 10),
                flow: None,
                layout: None,
                blocks: vec![],
                unknowns: vec![],
            },
            HocrCarea {
                level: "carea".to_string(),
                id: "c2".to_string(),
                bbox: HocrBbox::new(10, 10, 20, 20),
                flow: Some("existing".to_string()),
                layout: None,
                blocks: vec![],
                unknowns: vec![],
            },
        ],
        unknowns: vec![],
    };

    let flows = vec![FlowSchema { name: "default".to_string(), color: Some(ColorSpecification::default()) }];
    page.auto_flow(flows, vec![], true, None);

    assert_eq!(page.careas[0].flow, Some("default".to_string()));
    assert_eq!(page.careas[1].flow, Some("existing".to_string()));
}

#[test]
fn test_auto_layout_merging() {
    let mut page = HocrPage {
        level: "page".to_string(),
        page_id: "p1".to_string(),
        bbox: HocrBbox::new(0, 0, 100, 100),
        careas: vec![
            HocrCarea {
                level: "carea".to_string(),
                id: "c1".to_string(),
                bbox: HocrBbox::new(0, 0, 10, 10),
                flow: Some("F1".to_string()),
                layout: Some("L1".to_string()),
                blocks: vec![HocrBlock {
                    level: "block".to_string(),
                    id: "b1".to_string(),
                    bbox: HocrBbox::new(0, 0, 10, 10),
                    kind: HocrBlockKind::Paragraph,
                    lang: None,
                    hints: HocrBlockHints::default(),
                    metrics: None,
                    lines: vec![],
                }],
                unknowns: vec![],
            },
            HocrCarea {
                level: "carea".to_string(),
                id: "c2".to_string(),
                bbox: HocrBbox::new(20, 20, 30, 30),
                flow: Some("F2".to_string()),
                layout: Some("L1".to_string()),
                blocks: vec![HocrBlock {
                    level: "block".to_string(),
                    id: "b2".to_string(),
                    bbox: HocrBbox::new(20, 20, 30, 30),
                    kind: HocrBlockKind::Paragraph,
                    lang: None,
                    hints: HocrBlockHints::default(),
                    metrics: None,
                    lines: vec![],
                }],
                unknowns: vec![],
            },
            HocrCarea {
                level: "carea".to_string(),
                id: "c3".to_string(),
                bbox: HocrBbox::new(40, 40, 50, 50),
                flow: Some("F1".to_string()),
                layout: Some("L1".to_string()),
                blocks: vec![HocrBlock {
                    level: "block".to_string(),
                    id: "b3".to_string(),
                    bbox: HocrBbox::new(40, 40, 50, 50),
                    kind: HocrBlockKind::Paragraph,
                    lang: None,
                    hints: HocrBlockHints::default(),
                    metrics: None,
                    lines: vec![],
                }],
                unknowns: vec![],
            },
        ],
        unknowns: vec![],
    };

    let flows = vec![
        FlowSchema { name: "F1".to_string(), color: Some(ColorSpecification::default()) },
        FlowSchema { name: "F2".to_string(), color: Some(ColorSpecification::default()) },
    ];
    page.auto_flow(flows, vec![], true, None);

    // Result should be 2 careas in Group L1
    assert_eq!(page.careas.len(), 2);
    
    // First carea should be F1 (c1 + c3)
    assert_eq!(page.careas[0].flow, Some("F1".to_string()));
    assert_eq!(page.careas[0].blocks.len(), 2);
    assert_eq!(page.careas[0].blocks[0].id, "b1");
    assert_eq!(page.careas[0].blocks[1].id, "b3");
    
    // Second carea should be F2 (c2)
    assert_eq!(page.careas[1].flow, Some("F2".to_string()));
    assert_eq!(page.careas[1].blocks.len(), 1);
    assert_eq!(page.careas[1].blocks[0].id, "b2");
}

#[test]
fn test_auto_layout_consecutive_layout_grouping() {
    let mut page = HocrPage {
        level: "page".to_string(),
        page_id: "p1".to_string(),
        bbox: HocrBbox::new(0, 0, 100, 100),
        careas: vec![
            HocrCarea {
                level: "carea".to_string(),
                id: "c1".to_string(),
                bbox: HocrBbox::new(0, 0, 10, 10),
                flow: Some("F1".to_string()),
                layout: Some("L1".to_string()),
                blocks: vec![HocrBlock { id: "b1".to_string(), level: "block".to_string(), bbox: HocrBbox::new(0, 0, 10, 10), kind: HocrBlockKind::Paragraph, lang: None, hints: HocrBlockHints::default(), metrics: None, lines: vec![] }],
                unknowns: vec![],
            },
            HocrCarea {
                level: "carea".to_string(),
                id: "c2".to_string(),
                bbox: HocrBbox::new(20, 20, 30, 30),
                flow: Some("F1".to_string()),
                layout: Some("L2".to_string()),
                blocks: vec![HocrBlock { id: "b2".to_string(), level: "block".to_string(), bbox: HocrBbox::new(20, 20, 30, 30), kind: HocrBlockKind::Paragraph, lang: None, hints: HocrBlockHints::default(), metrics: None, lines: vec![] }],
                unknowns: vec![],
            },
            HocrCarea {
                level: "carea".to_string(),
                id: "c3".to_string(),
                bbox: HocrBbox::new(40, 40, 50, 50),
                flow: Some("F1".to_string()),
                layout: Some("L1".to_string()),
                blocks: vec![HocrBlock { id: "b3".to_string(), level: "block".to_string(), bbox: HocrBbox::new(40, 40, 50, 50), kind: HocrBlockKind::Paragraph, lang: None, hints: HocrBlockHints::default(), metrics: None, lines: vec![] }],
                unknowns: vec![],
            },
        ],
        unknowns: vec![],
    };

    let flows = vec![FlowSchema { name: "F1".to_string(), color: Some(ColorSpecification::default()) }];
    page.auto_flow(flows, vec![], true, None);

    // Result should be 3 careas because L1 is interrupted by L2
    assert_eq!(page.careas.len(), 3);
    assert_eq!(page.careas[0].layout, Some("L1".to_string()));
    assert_eq!(page.careas[1].layout, Some("L2".to_string()));
    assert_eq!(page.careas[2].layout, Some("L1".to_string()));
}

#[test]
fn test_auto_flow_carea_selection() {
    let mut page = HocrPage {
        level: "page".to_string(),
        page_id: "p1".to_string(),
        bbox: HocrBbox::new(0, 0, 100, 100),
        careas: vec![
            HocrCarea {
                level: "carea".to_string(),
                id: "c1".to_string(),
                bbox: HocrBbox::new(0, 0, 10, 10),
                flow: Some("F1".to_string()),
                layout: Some("L1".to_string()),
                blocks: vec![HocrBlock { id: "b1".to_string(), level: "block".to_string(), bbox: HocrBbox::new(0, 0, 10, 10), kind: HocrBlockKind::Paragraph, lang: None, hints: HocrBlockHints::default(), metrics: None, lines: vec![] }],
                unknowns: vec![],
            },
            HocrCarea {
                level: "carea".to_string(),
                id: "c2".to_string(),
                bbox: HocrBbox::new(20, 20, 30, 30),
                flow: Some("F2".to_string()),
                layout: Some("L1".to_string()),
                blocks: vec![HocrBlock { id: "b2".to_string(), level: "block".to_string(), bbox: HocrBbox::new(20, 20, 30, 30), kind: HocrBlockKind::Paragraph, lang: None, hints: HocrBlockHints::default(), metrics: None, lines: vec![] }],
                unknowns: vec![],
            },
            HocrCarea {
                level: "carea".to_string(),
                id: "c3".to_string(),
                bbox: HocrBbox::new(40, 40, 50, 50),
                flow: Some("F1".to_string()),
                layout: Some("L1".to_string()),
                blocks: vec![HocrBlock { id: "b3".to_string(), level: "block".to_string(), bbox: HocrBbox::new(40, 40, 50, 50), kind: HocrBlockKind::Paragraph, lang: None, hints: HocrBlockHints::default(), metrics: None, lines: vec![] }],
                unknowns: vec![],
            },
        ],
        unknowns: vec![],
    };

    let flows = vec![
        FlowSchema { name: "F1".to_string(), color: Some(ColorSpecification::default()) },
        FlowSchema { name: "F2".to_string(), color: Some(ColorSpecification::default()) },
    ];
    
    // Only select c1 and c3. c2 should act as a barrier and NOT be merged.
    // F1 is in c1 and c3. Normally they would merge, but c2 is in between and unselected.
    page.auto_flow(flows, vec![], true, Some(vec!["c1".to_string(), "c3".to_string()]));

    // Result should be 3 careas because c2 is not selected and acts as a barrier.
    assert_eq!(page.careas.len(), 3);
    assert_eq!(page.careas[0].id, "c1");
    assert_eq!(page.careas[1].id, "c2");
    assert_eq!(page.careas[2].id, "c3");
}

#[test]
fn test_replace_or_merge_carea_uniqueness() {
    let mut page = parse(SAMPLE_3CAREAS, ParserConfig::default()).unwrap();
    // The sample has careas with IDs: carea_1, carea_2, carea_3.
    // It has blocks (pars): par_1, par_2, par_3.
    // Lines: line_1, line_2, line_3.
    // Words: word_1, word_2, word_3.

    // Create a new carea that has overlapping IDs.
    let new_carea = page.careas[0].clone();
    // new_carea has id 'carea_1', block 'par_1', line 'line_1', word 'word_1'.

    let original_carea_count = page.careas.len();
    page.replace_or_merge_carea(0, vec![new_carea]);

    // It should have merged blocks into the first carea.
    assert_eq!(page.careas.len(), original_carea_count);

    // Now check for duplicates across the entire page
    let mut ids = std::collections::HashSet::new();
    for carea in &page.careas {
        assert!(ids.insert(carea.id.clone()), "Duplicate carea ID: {}", carea.id);
        for block in &carea.blocks {
            assert!(ids.insert(block.id.clone()), "Duplicate block ID: {}", block.id);
            for line in &block.lines {
                assert!(ids.insert(line.id.clone()), "Duplicate line ID: {}", line.id);
                for word in &line.words {
                    assert!(ids.insert(word.id.clone()), "Duplicate word ID: {}", word.id);
                }
            }
        }
    }

    // Specifically verify that the merged block got a new ID.
    // The first carea should now have 2 blocks.
    assert_eq!(page.careas[0].blocks.len(), 2);
    // Original was 'par_1', new one should be 'par_4' because par_1, par_2, par_3 exist.
    assert_eq!(page.careas[0].blocks[0].id, "par_1");
    assert_eq!(page.careas[0].blocks[1].id, "par_4");
}

#[test]
fn test_dropcap_roundtrip() {
    let html = r#"
        <span class="ocrx_word" id="word_1" title="bbox 10 10 50 50; x_wconf 90"><span class="dropcap">W</span>hen</span>
    "#;
    let full_html = format!(r#"
        <div class="ocr_page" id="page_1" title="bbox 0 0 100 100">
            <div class="ocr_carea" id="carea_1" title="bbox 0 0 100 100">
                <p class="ocr_par" id="par_1" title="bbox 0 0 100 100">
                    <span class="ocr_line" id="line_1" title="bbox 0 0 100 100">
                        {}
                    </span>
                </p>
            </div>
        </div>
    "#, html);

    let page = parse(&full_html, ParserConfig::default()).unwrap();
    let word = &page.careas[0].blocks[0].lines[0].words[0];
    assert_eq!(word.dropcap, Some("W".to_string()));
    assert_eq!(word.text, "hen".to_string());

    let rendered = word.to_hocr_html();
    assert!(rendered.contains(r#"<span class="dropcap">W</span>"#));
    assert!(rendered.contains("hen</span>"));
}

#[test]
fn test_inject_dropcaps() {
    let mut page = parse(r#"
        <div class="ocr_page" id="page_1" title="bbox 0 0 1000 1000">
            <div class="ocr_carea" id="carea_1" title="bbox 100 100 500 500">
                <p class="ocr_par" id="par_1" title="bbox 100 100 500 500">
                    <span class="ocr_line" id="line_1" title="bbox 110 105 400 130">
                        <span class="ocrx_word" id="word_1" title="bbox 110 105 200 130">hen</span>
                    </span>
                </p>
            </div>
        </div>
    "#, ParserConfig::default()).unwrap();

    let injections = vec![DropCapInjection {
        text: "W".to_string(),
        bbox: HocrBbox([50, 100, 105, 200]),
    }];

    page.inject_dropcaps(injections);

    let word = &page.careas[0].blocks[0].lines[0].words[0];
    assert_eq!(word.dropcap, Some("W".to_string()));
    assert_eq!(word.text, "hen");
}

#[test]
fn test_inject_images() {
    let mut page = parse(r#"
        <div class="ocr_page" id="page_1" title="bbox 0 0 1000 1000">
            <div class="ocr_carea" id="carea_1" title="bbox 100 100 500 500">
                <p class="ocr_par" id="par_1" title="bbox 100 100 500 500">
                    <span class="ocr_line" id="line_1" title="bbox 110 105 400 130">
                        <span class="ocrx_word" id="word_1" title="bbox 110 105 200 130">Text</span>
                    </span>
                </p>
            </div>
        </div>
    "#, ParserConfig::default()).unwrap();

    let image_bboxes = vec![HocrBbox([600, 600, 800, 800])];

    page.inject_images(image_bboxes);

    // Should have 2 careas now
    assert_eq!(page.careas.len(), 2);
    
    // The new carea should be at index 1 because its center Y (700) is greater than carea_1 center Y (300)
    let new_carea = &page.careas[1];
    assert_eq!(new_carea.blocks.len(), 1);
    assert_eq!(new_carea.blocks[0].kind, HocrBlockKind::Image);
    assert_eq!(new_carea.blocks[0].bbox, HocrBbox([600, 600, 800, 800]));

    // Verify page bbox was rebuilt
    assert_eq!(page.bbox, HocrBbox([100, 100, 800, 800]));
    
    let html = page.to_hocr_html();
    assert!(html.contains("<img class=\"ocr_photo\""));
    assert!(html.contains("title=\"bbox 600 600 800 800\""));
}

#[test]
fn test_block_hints_parsing_and_html() {
    let hocr = r#"
        <div class="ocr_page" id="page_1" title="bbox 0 0 1000 1000">
            <div class="ocr_carea" id="carea_1" title="bbox 0 0 1000 1000">
                <p class="ocr_par" id="par_1" title="bbox 0 0 1000 500; continue_from_previous; continue_to_following">
                    <span class="ocr_line" id="line_1" title="bbox 0 0 1000 100">
                        <span class="ocrx_word" id="word_1" title="bbox 0 0 100 100; x_wconf 90">Word</span>
                    </span>
                </p>
                <p class="ocr_par" id="par_2" title="bbox 0 500 1000 1000">
                    <span class="ocr_line" id="line_2" title="bbox 0 500 1000 600">
                        <span class="ocrx_word" id="word_2" title="bbox 0 500 100 600; x_wconf 90">Another</span>
                    </span>
                </p>
            </div>
        </div>
    "#;

    let page = parse(hocr, ParserConfig::default()).unwrap();
    let block1 = &page.careas[0].blocks[0];
    let block2 = &page.careas[0].blocks[1];

    assert_eq!(block1.hints.break_from_preceding, Evidence::Assigned(false));
    assert_eq!(block1.hints.break_from_following, Evidence::Assigned(false));
    assert_eq!(block2.hints.break_from_preceding, Evidence::Untested);
    assert_eq!(block2.hints.break_from_following, Evidence::Untested);

    let html = block1.to_hocr_html();
    assert!(!html.contains("continue_from_previous"));
    assert!(!html.contains("continue_to_following"));
    assert!(html.contains("break_from_preceding assigned_false"));
    assert!(html.contains("break_from_following assigned_false"));
    
    let html2 = block2.to_hocr_html();
    assert!(!html2.contains("break_from_preceding"));
    assert!(!html2.contains("break_from_following"));
}

#[test]
fn test_block_hints_roundtrip() {
    let hocr = r#"<!DOCTYPE html>
<html>
<body>
<div class="ocr_page" id="page_1" title="bbox 0 0 1000 1000">
<div class="ocr_carea" id="carea_1" title="bbox 0 0 1000 1000">
<p class="ocr_par" id="par_1" title="bbox 0 0 1000 500; continue_from_previous; continue_to_following">
<span class="ocr_line" id="line_1" title="bbox 0 0 1000 100; baseline 0 0; x_size 0; x_descenders 0; x_ascenders 0">
<span class="ocrx_word" id="word_1" title="bbox 0 0 100 100; x_wconf 90">Word</span>
</span>
</p>
</div>
</div>
</body>
</html>
"#;

    let page = parse(hocr, ParserConfig::default()).unwrap();
    let generated_html = page.to_hocr_html();
    
    // Legacy hints removed from output, modern hints added
    assert!(!generated_html.contains("continue_from_previous"));
    assert!(!generated_html.contains("continue_to_following"));
    assert!(generated_html.contains("break_from_preceding assigned_false"));
    assert!(generated_html.contains("break_from_following assigned_false"));
    
    // Parse again
    let page2 = parse(&generated_html, ParserConfig::default()).unwrap();
    assert_eq!(page2.careas[0].blocks[0].hints.break_from_preceding, Evidence::Assigned(false));
    assert_eq!(page2.careas[0].blocks[0].hints.break_from_following, Evidence::Assigned(false));
}

#[test]
fn test_modern_block_hints_roundtrip() {
    let mut hints = HocrBlockHints::default();
    hints.test_x_indent = Evidence::Suggested(true);
    hints.test_x_dedent = Evidence::Determined(false);
    hints.test_hyphenation = Evidence::Assigned(true);
    hints.test_y_advance = Evidence::Error;
    hints.test_y_reverse = Evidence::Undetermined;
    hints.break_from_preceding = Evidence::Assigned(false);
    hints.break_from_following = Evidence::Assigned(true);

    let block = HocrBlock {
        level: "block".to_string(),
        id: "par_1".to_string(),
        bbox: HocrBbox::new(0, 0, 100, 100),
        kind: HocrBlockKind::Paragraph,
        lang: None,
        hints,
        metrics: None,
        lines: vec![],
    };

    let carea = HocrCarea {
        level: "carea".to_string(),
        id: "carea_1".to_string(),
        bbox: HocrBbox::new(0, 0, 100, 100),
        flow: None,
        layout: None,
        blocks: vec![block],
        unknowns: vec![],
    };

    let page = HocrPage {
        level: "page".to_string(),
        page_id: "page_1".to_string(),
        bbox: HocrBbox::new(0, 0, 100, 100),
        careas: vec![carea],
        unknowns: vec![],
    };

    let html = page.to_hocr_html();
    
    assert!(html.contains("test_x_indent suggested_true"));
    assert!(html.contains("test_x_dedent determined_false"));
    assert!(html.contains("test_hyphenation assigned_true"));
    assert!(html.contains("test_y_advance error"));
    assert!(html.contains("test_y_reverse undetermined"));
    assert!(html.contains("break_from_preceding assigned_false"));
    assert!(html.contains("break_from_following assigned_true"));

    let page2 = parse(&html, ParserConfig::default()).unwrap();
    let hints2 = &page2.careas[0].blocks[0].hints;

    assert_eq!(hints2.test_x_indent, Evidence::Suggested(true));
    assert_eq!(hints2.test_x_dedent, Evidence::Determined(false));
    assert_eq!(hints2.test_hyphenation, Evidence::Assigned(true));
    assert_eq!(hints2.test_y_advance, Evidence::Error);
    assert_eq!(hints2.test_y_reverse, Evidence::Undetermined);
    assert_eq!(hints2.break_from_preceding, Evidence::Assigned(false));
    assert_eq!(hints2.break_from_following, Evidence::Assigned(true));
}
