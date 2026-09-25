use super::*;

#[test]
fn test_auto_detection_reset_and_assigned() {
    let mut block = HocrBlock {
        id: "b1".to_string(),
        level: "block".to_string(),
        kind: HocrBlockKind::Paragraph,
        lang: None,
        bbox: HocrBbox([100, 100, 500, 200]),
        hints: HocrBlockHints {
            test_x_indent: Evidence::Determined(true),
            test_x_dedent: Evidence::Suggested(true),
            test_hyphenation: Evidence::Determined(false),
            test_y_advance: Evidence::Determined(true),
            test_y_reverse: Evidence::Determined(false),
            test_preceding_terminal: Evidence::Untested,
            test_following_terminal: Evidence::Untested,
            break_from_preceding: Evidence::Assigned(true), // Should be preserved
            break_from_following: Evidence::Suggested(false),
        },
        metrics: None,
        lines: vec![],
    };

    let thresholds = DetectionThresholds {
        x_indent: DetectionRange { certainly_true: 20, suggested_true: 15, suggested_false: 10, certainly_false: 5 },
        x_dedent: DetectionRange { certainly_true: 20, suggested_true: 15, suggested_false: 10, certainly_false: 5 },
        y_advance: DetectionRange { certainly_true: 20, suggested_true: 15, suggested_false: 10, certainly_false: 5 },
        use_x_indent: false, // Should result in Untested
        use_x_dedent: false,
        use_hyphenation: false,
        use_y_advance: false,
    };

    block.update_metrics(None, None, "page1", HocrPath::Block { carea: 0, block: 0 });
    block.auto_bridge(None, None, "page1", HocrPath::Block { carea: 0, block: 0 }, &thresholds);

    assert_eq!(block.hints.test_x_indent, Evidence::Untested);
    assert_eq!(block.hints.test_x_dedent, Evidence::Untested);
    assert_eq!(block.hints.test_hyphenation, Evidence::Untested);
    assert_eq!(block.hints.test_y_advance, Evidence::Untested);
    assert_eq!(block.hints.test_y_reverse, Evidence::Untested);
    assert_eq!(block.hints.break_from_preceding, Evidence::Assigned(true)); // Preserved
    assert_eq!(block.hints.break_from_following, Evidence::Untested); // Recalculated from Untested inputs -> Untested
}

#[test]
fn test_derive_evidence_assigned_and_untested() {
    // Test Assigned values
    assert_eq!(operations::derive_evidence(&[Evidence::Assigned(true), Evidence::Determined(false)]), Evidence::Error);
    assert_eq!(operations::derive_evidence(&[Evidence::Assigned(true), Evidence::Untested]), Evidence::Determined(true));
    assert_eq!(operations::derive_evidence(&[Evidence::Assigned(false), Evidence::Suggested(true)]), Evidence::Determined(false));

    // Test All Untested
    assert_eq!(operations::derive_evidence(&[Evidence::Untested, Evidence::Untested]), Evidence::Untested);

    // Test Mixed Untested and Evidence
    assert_eq!(operations::derive_evidence(&[Evidence::Untested, Evidence::Suggested(true)]), Evidence::Suggested(true));
}

#[test]
fn test_terminal_block_detection() {
    let header = HocrBlock {
        id: "h1".to_string(),
        level: "block".to_string(),
        kind: HocrBlockKind::Chapter,
        lang: None,
        bbox: HocrBbox([100, 100, 500, 150]),
        hints: HocrBlockHints::default(),
        metrics: None,
        lines: vec![],
    };

    let mut paragraph = HocrBlock {
        id: "p1".to_string(),
        level: "block".to_string(),
        kind: HocrBlockKind::Paragraph,
        lang: None,
        bbox: HocrBbox([100, 160, 500, 250]),
        hints: HocrBlockHints::default(),
        metrics: None,
        lines: vec![],
    };

    let thresholds = DetectionThresholds::default();

    paragraph.update_metrics(
        Some((&header, "page1", HocrPath::Block { carea: 0, block: 0 })),
        None,
        "page1",
        HocrPath::Block { carea: 0, block: 1 },
    );
    paragraph.auto_bridge(
        Some((&header, "page1", HocrPath::Block { carea: 0, block: 0 })),
        None,
        "page1",
        HocrPath::Block { carea: 0, block: 1 },
        &thresholds
    );

    assert_eq!(paragraph.hints.test_preceding_terminal, Evidence::Determined(true));
    // break_from_preceding is derived from test_x_indent, test_y_advance, and test_preceding_terminal.
    // Since lines is empty, x_indent is 0. With default thresholds, x_indent_min=5, so x_indent is 0 < 5 -> Determined(false).
    // y_advance is (160 - 150) = 10. Default y_advance_min=0, y_advance_max=0, so y_advance is 10 >= 0 -> Determined(true).
    // test_preceding_terminal is Determined(true).
    // So derive_evidence([Determined(false), Determined(true), Determined(true)]) -> Evidence::Error because of conflicting Determinations.
    // Wait, let's check default thresholds again in models.rs.
    // y_advance_min: 0, y_advance_max: 0.
    // val >= thresholds.y_advance_max { Evidence::Determined(true) } -> 10 >= 0 is true.
    // val < thresholds.x_indent_min { Evidence::Determined(false) } -> 0 < 5 is true.
    // test_preceding_terminal is Determined(true).
    // So [Determined(false), Determined(true), Determined(true)] -> Error.
    
    // Actually, I should probably check that it correctly identified the terminal block.
    // To make break_from_preceding Determined(true), I should make others Untested or Determined(true).
    
    let mut thresholds_only_terminal = DetectionThresholds::default();
    thresholds_only_terminal.use_x_indent = false;
    thresholds_only_terminal.use_y_advance = false;

    paragraph.hints = HocrBlockHints::default(); // Reset hints
    paragraph.update_metrics(
        Some((&header, "page1", HocrPath::Block { carea: 0, block: 0 })),
        None,
        "page1",
        HocrPath::Block { carea: 0, block: 1 },
    );
    paragraph.auto_bridge(
        Some((&header, "page1", HocrPath::Block { carea: 0, block: 0 })),
        None,
        "page1",
        HocrPath::Block { carea: 0, block: 1 },
        &thresholds_only_terminal
    );

    assert_eq!(paragraph.hints.test_preceding_terminal, Evidence::Determined(true));
    assert_eq!(paragraph.hints.break_from_preceding, Evidence::Determined(true));
}

#[test]
fn test_terminal_hints_persistence() {
    let block = HocrBlock {
        id: "b1".to_string(),
        level: "block".to_string(),
        kind: HocrBlockKind::Paragraph,
        lang: None,
        bbox: HocrBbox([100, 100, 500, 200]),
        hints: HocrBlockHints {
            test_preceding_terminal: Evidence::Determined(true),
            test_following_terminal: Evidence::Determined(false),
            ..HocrBlockHints::default()
        },
        metrics: None,
        lines: vec![],
    };

    let html = block.to_hocr_html();
    assert!(html.contains("test_preceding_terminal determined_true"));
    assert!(html.contains("test_following_terminal determined_false"));

    // To test parsing, we need a full page or a way to parse just a block.
    // parser::parse takes a full HTML string.
    let full_html = format!(
        r#"<!DOCTYPE html><html><body><div class="ocr_page" id="page_1" title="bbox 0 0 1000 1000"><div class="ocr_carea" id="carea_1" title="bbox 0 0 1000 1000">{}</div></div></body></html>"#,
        html
    );

    let parsed_page = parser::parse(&full_html, parser::ParserConfig::default()).unwrap();
    let parsed_block = &parsed_page.careas[0].blocks[0];

    assert_eq!(parsed_block.hints.test_preceding_terminal, Evidence::Determined(true));
    assert_eq!(parsed_block.hints.test_following_terminal, Evidence::Determined(false));
}

#[test]
fn test_x_indent_dedent_omitted_for_single_line() {
    let line = HocrLine {
        id: "l1".to_string(),
        level: "line".to_string(),
        bbox: HocrBbox([110, 110, 490, 130]),
        lang: None,
        words: vec![],
        baseline: None,
        x_size: None,
        x_descenders: None,
        x_ascenders: None,
    };
    let mut block = HocrBlock {
        id: "b1".to_string(),
        level: "block".to_string(),
        kind: HocrBlockKind::Paragraph,
        lang: None,
        bbox: HocrBbox([100, 100, 500, 140]),
        hints: HocrBlockHints::default(),
        metrics: None,
        lines: vec![line],
    };

    let thresholds = DetectionThresholds {
        use_x_indent: true,
        use_x_dedent: true,
        x_indent: DetectionRange { certainly_true: 15, suggested_true: 10, suggested_false: 5, certainly_false: 2 },
        x_dedent: DetectionRange { certainly_true: 15, suggested_true: 10, suggested_false: 5, certainly_false: 2 },
        ..Default::default()
    };

    block.update_metrics(None, None, "page1", HocrPath::Block { carea: 0, block: 0 });
    block.auto_bridge(None, None, "page1", HocrPath::Block { carea: 0, block: 0 }, &thresholds);

    assert_eq!(block.hints.test_x_indent, Evidence::Untested);
    assert_eq!(block.hints.test_x_dedent, Evidence::Untested);
}

#[test]
fn test_x_indent_dedent_omitted_for_zero_lines() {
    let mut block = HocrBlock {
        id: "b1".to_string(),
        level: "block".to_string(),
        kind: HocrBlockKind::Paragraph,
        lang: None,
        bbox: HocrBbox([100, 100, 500, 140]),
        hints: HocrBlockHints::default(),
        metrics: None,
        lines: vec![],
    };

    let thresholds = DetectionThresholds {
        use_x_indent: true,
        use_x_dedent: true,
        ..Default::default()
    };

    block.update_metrics(None, None, "page1", HocrPath::Block { carea: 0, block: 0 });
    block.auto_bridge(None, None, "page1", HocrPath::Block { carea: 0, block: 0 }, &thresholds);

    assert_eq!(block.hints.test_x_indent, Evidence::Untested);
    assert_eq!(block.hints.test_x_dedent, Evidence::Untested);
}

#[test]
fn test_x_indent_dedent_included_for_multiple_lines() {
    let line1 = HocrLine {
        id: "l1".to_string(),
        level: "line".to_string(),
        bbox: HocrBbox([120, 110, 490, 130]), // x_indent = 20
        lang: None,
        words: vec![],
        baseline: None,
        x_size: None,
        x_descenders: None,
        x_ascenders: None,
    };
    let line2 = HocrLine {
        id: "l2".to_string(),
        level: "line".to_string(),
        bbox: HocrBbox([100, 140, 480, 160]), // x_dedent = 20
        lang: None,
        words: vec![],
        baseline: None,
        x_size: None,
        x_descenders: None,
        x_ascenders: None,
    };
    let mut block = HocrBlock {
        id: "b1".to_string(),
        level: "block".to_string(),
        kind: HocrBlockKind::Paragraph,
        lang: None,
        bbox: HocrBbox([100, 100, 500, 170]),
        hints: HocrBlockHints::default(),
        metrics: None,
        lines: vec![line1, line2],
    };

    let thresholds = DetectionThresholds {
        use_x_indent: true,
        use_x_dedent: true,
        x_indent: DetectionRange { certainly_true: 15, suggested_true: 10, suggested_false: 5, certainly_false: 2 },
        x_dedent: DetectionRange { certainly_true: 15, suggested_true: 10, suggested_false: 5, certainly_false: 2 },
        ..Default::default()
    };

    block.update_metrics(None, None, "page1", HocrPath::Block { carea: 0, block: 0 });
    block.auto_bridge(None, None, "page1", HocrPath::Block { carea: 0, block: 0 }, &thresholds);

    assert_eq!(block.hints.test_x_indent, Evidence::Determined(true)); // 20 >= 15
    assert_eq!(block.hints.test_x_dedent, Evidence::Determined(true)); // 20 >= 15
}

#[test]
fn test_assigned_preserved_for_single_line() {
    let line = HocrLine {
        id: "l1".to_string(),
        level: "line".to_string(),
        bbox: HocrBbox([110, 110, 490, 130]),
        lang: None,
        words: vec![],
        baseline: None,
        x_size: None,
        x_descenders: None,
        x_ascenders: None,
    };
    let mut block = HocrBlock {
        id: "b1".to_string(),
        level: "block".to_string(),
        kind: HocrBlockKind::Paragraph,
        lang: None,
        bbox: HocrBbox([100, 100, 500, 140]),
        hints: HocrBlockHints {
            test_x_indent: Evidence::Assigned(true),
            test_x_dedent: Evidence::Assigned(false),
            ..HocrBlockHints::default()
        },
        metrics: None,
        lines: vec![line],
    };

    let thresholds = DetectionThresholds {
        use_x_indent: true,
        use_x_dedent: true,
        ..Default::default()
    };

    block.update_metrics(None, None, "page1", HocrPath::Block { carea: 0, block: 0 });
    block.auto_bridge(None, None, "page1", HocrPath::Block { carea: 0, block: 0 }, &thresholds);

    assert_eq!(block.hints.test_x_indent, Evidence::Assigned(true));
    assert_eq!(block.hints.test_x_dedent, Evidence::Assigned(false));
}

#[test]
fn test_four_value_detection_logic() {
    let mut block = HocrBlock {
        id: "b1".to_string(),
        level: "block".to_string(),
        kind: HocrBlockKind::Paragraph,
        lang: None,
        bbox: HocrBbox([100, 100, 500, 300]),
        hints: HocrBlockHints::default(),
        metrics: Some(HocrBlockMetrics {
            x_indent: 10,
            x_dedent: 0,
            y_advance: Some(2),
            y_reverse: None,
            has_final_hyphen: false,
        }),
        lines: vec![
            HocrLine { id: "l1".to_string(), level: "line".to_string(), bbox: HocrBbox([110, 110, 490, 130]), words: vec![], ..Default::default() },
            HocrLine { id: "l2".to_string(), level: "line".to_string(), bbox: HocrBbox([100, 140, 490, 160]), words: vec![], ..Default::default() },
        ],
    };

    let thresholds = DetectionThresholds {
        x_indent: DetectionRange {
            certainly_true: 15,
            suggested_true: 8,
            suggested_false: 4,
            certainly_false: 1,
        },
        y_advance: DetectionRange {
            certainly_true: 10,
            suggested_true: 5,
            suggested_false: 3,
            certainly_false: 0,
        },
        ..DetectionThresholds::default()
    };

    block.auto_bridge(None, None, "page1", HocrPath::Block { carea: 0, block: 0 }, &thresholds);

    // x_indent = 10. certainly_true: 15, suggested_true: 8. 10 >= 8 -> Suggested(true)
    assert_eq!(block.hints.test_x_indent, Evidence::Suggested(true));

    // y_advance = 2. certainly_true: 10, suggested_true: 5, suggested_false: 3. 2 <= 3 -> Suggested(false)
    assert_eq!(block.hints.test_y_advance, Evidence::Suggested(false));
}
