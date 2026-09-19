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
            break_from_preceding: Evidence::Assigned(true), // Should be preserved
            break_from_following: Evidence::Suggested(false),
        },
        lines: vec![],
    };

    let thresholds = DetectionThresholds {
        x_indent_min: 10,
        x_indent_max: 20,
        x_dedent_min: 10,
        x_dedent_max: 20,
        y_advance_min: 10,
        y_advance_max: 20,
        use_x_indent: false, // Should result in Untested
        use_x_dedent: false,
        use_hyphenation: false,
        use_y_advance: false,
    };

    block.apply_auto_detection(None, None, "page1", HocrPath::Block { carea: 0, block: 0 }, &thresholds);

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
