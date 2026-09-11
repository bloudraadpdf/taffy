#[cfg(test)]
mod tests {
    use taffy::prelude::*;
    use taffy_test_helpers::new_test_tree;

    #[test]
    fn transferred_flex_limits_preserve_definite_preferred_sizes() {
        for direction in [FlexDirection::Row, FlexDirection::Column] {
            for replaced in [false, true] {
                let row = direction == FlexDirection::Row;
                let axes = |main, cross| {
                    if row {
                        Size { width: main, height: cross }
                    } else {
                        Size { width: cross, height: main }
                    }
                };
                for (preferred, minimum, maximum, expected_main, expected_cross) in [
                    (30.0, Size::zero(), axes(auto(), length(10.0)), 30.0, 10.0),
                    (30.0, Size::zero(), axes(length(20.0), length(10.0)), 20.0, 10.0),
                    (10.0, axes(auto(), length(30.0)), Size::auto(), 10.0, 30.0),
                    (10.0, axes(length(20.0), length(30.0)), Size::auto(), 20.0, 30.0),
                ] {
                    let mut tree = new_test_tree();
                    let child = tree
                        .new_leaf(Style {
                            item_is_replaced: replaced,
                            size: axes(length(preferred), auto()),
                            min_size: minimum,
                            max_size: maximum,
                            aspect_ratio: Some(1.0),
                            ..Default::default()
                        })
                        .unwrap();
                    let parent = tree
                        .new_with_children(
                            Style {
                                display: Display::Flex,
                                flex_direction: direction,
                                align_items: Some(AlignItems::FLEX_START),
                                size: Size::length(40.0),
                                ..Default::default()
                            },
                            &[child],
                        )
                        .unwrap();
                    tree.compute_layout(parent, Size::MAX_CONTENT).unwrap();
                    let actual = tree.layout(child).unwrap().size;
                    let expected = if row {
                        Size { width: expected_main, height: expected_cross }
                    } else {
                        Size { width: expected_cross, height: expected_main }
                    };
                    assert_eq!(
                        actual, expected,
                        "{direction:?}; replaced:{replaced}; preferred:{preferred}; min:{minimum:?}; max:{maximum:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn intrinsic_flex_contributions_transfer_the_known_cross_size() {
        for direction in [FlexDirection::Row, FlexDirection::Column] {
            for box_sizing in [BoxSizing::ContentBox, BoxSizing::BorderBox] {
                for padding in [0.0, 5.0] {
                    let row = direction == FlexDirection::Row;
                    let edges = 2.0 * padding;
                    let expected_main =
                        if box_sizing == BoxSizing::ContentBox { 2.0 * (20.0 - edges) + edges } else { 40.0 };
                    let mut tree = new_test_tree();
                    let child = tree
                        .new_leaf(Style {
                            box_sizing,
                            item_is_replaced: true,
                            flex_basis: length(
                                expected_main - if box_sizing == BoxSizing::ContentBox { edges } else { 0.0 },
                            ),
                            min_size: Size::zero(),
                            padding: Rect::length(padding),
                            aspect_ratio: Some(if row { 2.0 } else { 0.5 }),
                            ..Default::default()
                        })
                        .unwrap();
                    let parent = tree
                        .new_with_children(
                            Style {
                                display: Display::Flex,
                                flex_direction: direction,
                                size: if row {
                                    Size { width: auto(), height: length(20.0) }
                                } else {
                                    Size { width: length(20.0), height: auto() }
                                },
                                ..Default::default()
                            },
                            &[child],
                        )
                        .unwrap();
                    tree.compute_layout_with_measure(parent, Size::MAX_CONTENT, |_, _, _, _, _| Size::length(1.0))
                        .unwrap();
                    for node in [parent, child] {
                        let size = tree.layout(node).unwrap().size;
                        assert_eq!(
                            size,
                            if row {
                                Size { width: expected_main, height: 20.0 }
                            } else {
                                Size { width: 20.0, height: expected_main }
                            },
                            "{direction:?}; {box_sizing:?}; {padding}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn intrinsic_flex_cross_sizes_use_the_flexed_main_size() {
        use taffy::style::{FlexCrossIntrinsicBounds, FlexCrossSize};
        for direction in [FlexDirection::Row, FlexDirection::Column] {
            for (preferred, bounds, cross) in [
                (FlexCrossSize::Content, FlexCrossIntrinsicBounds::None, 500.0),
                (FlexCrossSize::Style, FlexCrossIntrinsicBounds::Minimum, 0.0),
                (FlexCrossSize::Style, FlexCrossIntrinsicBounds::Maximum, 500.0),
                (FlexCrossSize::Style, FlexCrossIntrinsicBounds::Both, 500.0),
            ] {
                let mut tree = new_test_tree();
                let row = direction == FlexDirection::Row;
                let child = tree
                    .new_leaf(Style {
                        size: if row {
                            Size { width: length(140.0), height: length(cross) }
                        } else {
                            Size { width: length(cross), height: length(140.0) }
                        },
                        flex_shrink: 0.0,
                        flex_cross_size: preferred,
                        flex_cross_intrinsic_bounds: bounds,
                        padding: Rect::length(5.0),
                        ..Default::default()
                    })
                    .unwrap();
                let parent = tree
                    .new_with_children(
                        Style {
                            display: Display::Flex,
                            flex_direction: direction,
                            size: Size::length(100.0),
                            ..Default::default()
                        },
                        &[child],
                    )
                    .unwrap();
                tree.compute_layout_with_measure(parent, Size::MAX_CONTENT, |known, _, _, _, _| {
                    let main = if row { known.width } else { known.height }.unwrap_or(140.0);
                    let cross = if main >= 140.0 { 20.0 } else { 40.0 };
                    if row {
                        Size { width: main, height: cross }
                    } else {
                        Size { width: cross, height: main }
                    }
                })
                .unwrap();
                let actual = tree.layout(child).unwrap().size;
                assert_eq!(
                    if row { actual.height } else { actual.width },
                    30.0,
                    "{direction:?}; {preferred:?}; {bounds:?}"
                );
            }
        }
    }

    #[test]
    fn grid_minimum_contributions_clamp_definite_preferred_sizes() {
        for box_sizing in [BoxSizing::ContentBox, BoxSizing::BorderBox] {
            for replaced in [false, true] {
                for (preferred, minimum, maximum, expected) in
                    [(100.0, 0.0, 50.0, 50.0), (100.0, 70.0, 50.0, 70.0), (10.0, 80.0, 500.0, 80.0)]
                {
                    let mut tree = new_test_tree();
                    let child = tree
                        .new_leaf(Style {
                            box_sizing,
                            item_is_replaced: replaced,
                            size: Size::length(preferred),
                            min_size: Size::length(minimum),
                            max_size: Size::length(maximum),
                            padding: Rect::length(5.0),
                            ..Default::default()
                        })
                        .unwrap();
                    let grid = tree
                        .new_with_children(Style { display: Display::Grid, ..Default::default() }, &[child])
                        .unwrap();
                    tree.compute_layout(grid, Size::MAX_CONTENT).unwrap();
                    let expected = expected + if box_sizing == BoxSizing::ContentBox { 10.0 } else { 0.0 };
                    for node in [grid, child] {
                        assert_eq!(
                            tree.layout(node).unwrap().size,
                            Size { width: expected, height: expected },
                            "{box_sizing:?}; replaced:{replaced}; preferred:{preferred}; min:{minimum}; max:{maximum}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn conflicting_grid_bounds_use_the_minimum_when_sizing_flexible_tracks() {
        let mut failures = Vec::new();
        for horizontal in [true, false] {
            for box_sizing in [BoxSizing::ContentBox, BoxSizing::BorderBox] {
                for edge in [0.0, 7.0] {
                    for (minimum, maximum, extent, first, second) in [
                        (Some(70.0), Some(60.0), 70.0, 10.0, 27.0),
                        (Some(83.0), Some(53.0), 83.0, 10.0, 40.0),
                        (Some(108.0), Some(60.0), 108.0, 15.0, 60.0),
                        (Some(70.0), None, 83.0, 10.0, 40.0),
                        (None, Some(70.0), 70.0, 10.0, 27.0),
                    ] {
                        let mut tree = new_test_tree();
                        tree.disable_rounding();
                        let children =
                            [tree.new_leaf(Style::default()).unwrap(), tree.new_leaf(Style::default()).unwrap()];
                        let adjustment = if box_sizing == BoxSizing::BorderBox { edge * 2.0 } else { 0.0 };
                        let bound = |value: Option<f32>| value.map_or(auto(), |value| length(value + adjustment));
                        let tracks = vec![
                            GridTemplateComponent::Single(minmax(length(10.0), fr(1.0))),
                            GridTemplateComponent::Single(minmax(length(10.0), fr(4.0))),
                        ];
                        let other = vec![GridTemplateComponent::Single(length(50.0))];
                        let grid = tree
                            .new_with_children(
                                Style {
                                    display: Display::Grid,
                                    box_sizing,
                                    border: Rect::length(edge),
                                    min_size: if horizontal {
                                        Size { width: bound(minimum), height: auto() }
                                    } else {
                                        Size { width: auto(), height: bound(minimum) }
                                    },
                                    max_size: if horizontal {
                                        Size { width: bound(maximum), height: auto() }
                                    } else {
                                        Size { width: auto(), height: bound(maximum) }
                                    },
                                    grid_template_columns: if horizontal { tracks.clone() } else { other.clone() },
                                    grid_template_rows: if horizontal { other } else { tracks },
                                    gap: Size::length(33.0),
                                    ..Default::default()
                                },
                                &children,
                            )
                            .unwrap();
                        tree.compute_layout(grid, Size::MAX_CONTENT).unwrap();
                        let selected_extent = |node| {
                            let size = tree.layout(node).unwrap().size;
                            if horizontal {
                                size.width
                            } else {
                                size.height
                            }
                        };
                        let actual =
                            [selected_extent(grid), selected_extent(children[0]), selected_extent(children[1])];
                        let expected = [extent + edge * 2.0, first, second];
                        if actual.iter().zip(expected).any(|(actual, expected)| (actual - expected).abs() > 0.001) {
                            failures.push(format!("{horizontal};{box_sizing:?};{edge};{minimum:?};{maximum:?}: expected {expected:?}, got {actual:?}"));
                        }
                    }
                }
            }
        }
        assert!(failures.is_empty(), "{} failures:\n{}", failures.len(), failures.join("\n"));
    }

    #[test]
    #[cfg(feature = "detailed_layout_info")]
    fn auto_repeat_constraints_survive_an_intrinsic_used_size() {
        for horizontal in [true, false] {
            for repeat in [RepetitionCount::AutoFill, RepetitionCount::AutoFit] {
                for gap in [0.0, 10.0] {
                    for box_sizing in [BoxSizing::ContentBox, BoxSizing::BorderBox] {
                        for percentage in [false, true] {
                            let mut tree = new_test_tree();
                            let placement = Line { start: line(-2), end: line(-1) };
                            let child = tree
                                .new_leaf(Style {
                                    grid_column: if horizontal { placement.clone() } else { Default::default() },
                                    grid_row: if horizontal { Default::default() } else { placement },
                                    ..Default::default()
                                })
                                .expect("the last repeated-track item must be created");
                            let edge_size = 20.0;
                            let adjustment = if box_sizing == BoxSizing::BorderBox { edge_size } else { 0.0 };
                            let used_content =
                                if repeat == RepetitionCount::AutoFit { 100.0 } else { 300.0 + gap * 2.0 };
                            let repeated_size = length(used_content + adjustment);
                            let max_size = if percentage { percent(0.5) } else { length(320.0 + adjustment) };
                            let tracks = vec![taffy::style_helpers::repeat(repeat, vec![length(100.0)])];
                            let grid = tree
                                .new_with_children(
                                    Style {
                                        display: Display::Grid,
                                        box_sizing,
                                        padding: Rect::length(5.0),
                                        border: Rect::length(5.0),
                                        gap: Size::length(gap),
                                        size: if horizontal {
                                            Size { width: repeated_size, height: auto() }
                                        } else {
                                            Size { width: auto(), height: repeated_size }
                                        },
                                        grid_auto_repeat_constraints: Some(taffy::GridAutoRepeatConstraints {
                                            size: Size::auto(),
                                            min_size: Size::auto(),
                                            max_size: if horizontal {
                                                Size { width: max_size, height: auto() }
                                            } else {
                                                Size { width: auto(), height: max_size }
                                            },
                                        }),
                                        grid_template_columns: if horizontal { tracks.clone() } else { Vec::new() },
                                        grid_template_rows: if horizontal { Vec::new() } else { tracks },
                                        ..Default::default()
                                    },
                                    &[child],
                                )
                                .expect("the externally sized grid must be created");
                            tree.compute_layout(
                                grid,
                                Size {
                                    width: AvailableSpace::Definite((320.0 + adjustment) * 2.0),
                                    height: AvailableSpace::Definite((320.0 + adjustment) * 2.0),
                                },
                            )
                            .expect("the externally sized grid must lay out");
                            let taffy::DetailedLayoutInfo::Grid(info) = tree.detailed_layout_info(grid) else {
                                panic!("grid tracks must remain available")
                            };
                            let tracks = if horizontal { &info.columns } else { &info.rows };
                            assert_eq!(
                                tracks.sizes,
                                if repeat == RepetitionCount::AutoFit { vec![0.0, 0.0, 100.0] } else { vec![100.0; 3] },
                                "{horizontal};{repeat:?};{gap};{box_sizing:?};{percentage}"
                            );
                            let size = tree.layout(grid).expect("the grid must have a used size").size;
                            assert_eq!(if horizontal { size.width } else { size.height }, used_content + edge_size);
                        }
                    }
                }
            }
        }
    }

    #[test]
    #[cfg(feature = "detailed_layout_info")]
    fn auto_repeat_uses_the_retained_constraint_kind() {
        for (preferred, minimum, maximum, count) in [
            (auto(), auto(), auto(), 1),
            (auto(), length(250.0), auto(), 3),
            (auto(), auto(), length(250.0), 2),
            (length(250.0), auto(), length(350.0), 2),
            (length(350.0), auto(), length(250.0), 2),
            (length(150.0), length(250.0), length(200.0), 2),
        ] {
            for horizontal in [true, false] {
                let axis_size = |value| {
                    if horizontal {
                        Size { width: value, height: auto() }
                    } else {
                        Size { width: auto(), height: value }
                    }
                };
                let tracks = vec![taffy::style_helpers::repeat(RepetitionCount::AutoFill, vec![length(100.0)])];
                let mut tree = new_test_tree();
                let grid = tree
                    .new_leaf(Style {
                        display: Display::Grid,
                        size: axis_size(length(500.0)),
                        grid_auto_repeat_constraints: Some(taffy::GridAutoRepeatConstraints {
                            size: axis_size(preferred),
                            min_size: axis_size(minimum),
                            max_size: axis_size(maximum),
                        }),
                        grid_template_columns: if horizontal { tracks.clone() } else { Vec::new() },
                        grid_template_rows: if horizontal { Vec::new() } else { tracks },
                        ..Default::default()
                    })
                    .expect("the retained-constraint grid must be created");
                tree.compute_layout(grid, Size::MAX_CONTENT).expect("the retained-constraint grid must lay out");
                let taffy::DetailedLayoutInfo::Grid(info) = tree.detailed_layout_info(grid) else {
                    panic!("grid tracks must remain available")
                };
                assert_eq!(
                    if horizontal { info.columns.sizes.len() } else { info.rows.sizes.len() },
                    count,
                    "{horizontal};{preferred:?};{minimum:?};{maximum:?}"
                );
            }
        }
    }

    #[test]
    #[cfg(feature = "calc")]
    fn fit_content_calculation_keeps_its_limit_and_intrinsic_minimum() {
        static LIMIT: f64 = 0.0;
        let handle = core::ptr::addr_of!(LIMIT).cast();
        for (width, minimum, expected) in [(200.0, 10.0, 70.0), (100.0, 10.0, 45.0), (100.0, 80.0, 80.0)] {
            let mut tree: TaffyTree<()> = TaffyTree::new();
            let child = tree.new_leaf(Style::default()).unwrap();
            let grid = tree
                .new_with_children(
                    Style {
                        display: Display::Grid,
                        size: Size { width: length(width), height: auto() },
                        grid_template_columns: vec![GridTemplateComponent::Single(fit_content(
                            LengthPercentage::calc(handle),
                        ))],
                        ..Default::default()
                    },
                    &[child],
                )
                .unwrap();
            tree.compute_layout_with_measure_and_calc(
                grid,
                Size::MAX_CONTENT,
                |input, _, _, _| Size {
                    width: input.known_dimensions.width.unwrap_or(
                        if input.available_space.width == AvailableSpace::MinContent { minimum } else { 300.0 },
                    ),
                    height: input.known_dimensions.height.unwrap_or(10.0),
                },
                |actual, basis| {
                    assert_eq!(actual, handle);
                    20.0 + basis * 0.25
                },
            )
            .unwrap();
            assert_eq!(tree.layout(child).unwrap().size.width, expected);
        }
    }

    #[test]
    fn intrinsic_track_minimum_keeps_preferred_minimum_and_edge_contributions() {
        for minimum in
            [MinTrackSizingFunction::AUTO, MinTrackSizingFunction::MIN_CONTENT, MinTrackSizingFunction::MAX_CONTENT]
        {
            let size = Size { width: length(60.0), height: length(40.0) };
            let zero = Size { width: length(0.0), height: length(0.0) };
            let edges = Rect { left: length(5.0), right: length(5.0), top: length(5.0), bottom: length(5.0) };
            let edge_size = Size { width: 10.0, height: 10.0 };
            for (style, expected) in [
                (Style { size, ..Default::default() }, Size { width: 60.0, height: 40.0 }),
                (Style { min_size: size, ..Default::default() }, Size { width: 60.0, height: 40.0 }),
                (Style { padding: edges, ..Default::default() }, edge_size),
                (Style { border: edges, ..Default::default() }, edge_size),
                (Style { padding: edges, border: edges, ..Default::default() }, Size { width: 20.0, height: 20.0 }),
                (Style { padding: edges, size: zero, ..Default::default() }, edge_size),
                (Style { padding: edges, min_size: zero, ..Default::default() }, edge_size),
                (Style { padding: edges, max_size: zero, ..Default::default() }, edge_size),
                (
                    Style { padding: edges, margin: Rect::length(2.0), ..Default::default() },
                    Size { width: 14.0, height: 14.0 },
                ),
                (
                    Style { padding: edges, margin: Rect::length(-2.0), ..Default::default() },
                    Size { width: 6.0, height: 6.0 },
                ),
            ] {
                for box_sizing in [BoxSizing::ContentBox, BoxSizing::BorderBox] {
                    let mut tree = new_test_tree();
                    let child = tree.new_leaf(Style { box_sizing, ..style.clone() }).unwrap();
                    let stretched = tree.new_leaf(Style::default()).unwrap();
                    let track = GridTemplateComponent::Single(minmax(minimum, length(0.0)));
                    let grid = tree
                        .new_with_children(
                            Style {
                                display: Display::Grid,
                                grid_template_columns: vec![track.clone()],
                                grid_template_rows: vec![track],
                                ..Default::default()
                            },
                            &[child, stretched],
                        )
                        .unwrap();
                    tree.compute_layout(grid, Size::MAX_CONTENT).unwrap();
                    assert_eq!(
                        tree.layout(stretched).unwrap().size.width,
                        expected.width,
                        "{minimum:?}; expected {expected:?}"
                    );
                    assert_eq!(
                        tree.layout(grid).unwrap().size.height,
                        expected.height,
                        "{minimum:?}; expected {expected:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn an_unspanned_flexible_track_does_not_remove_the_automatic_minimum() {
        for horizontal in [true, false] {
            for (tracks, item_span, expected) in [
                (vec![auto(), auto(), fr(1.0)], 2, 40.0),
                (vec![auto(), auto(), auto()], 2, 40.0),
                (vec![auto(), fr(1.0), fr(1.0)], 2, 0.0),
                (vec![fr(1.0), fr(1.0), fr(1.0)], 1, 80.0),
            ] {
                let mut tree = new_test_tree();
                let content = tree
                    .new_leaf(Style { size: Size { width: length(80.0), height: length(80.0) }, ..Default::default() })
                    .unwrap();
                let spanned_tracks = Line { start: line(1), end: span(item_span) };
                let probe_track = Line { start: line(item_span as i16), end: span(1) };
                let first_track = Line { start: line(1), end: span(1) };
                let item = tree
                    .new_with_children(
                        Style {
                            grid_column: if horizontal { spanned_tracks.clone() } else { first_track.clone() },
                            grid_row: if horizontal { first_track.clone() } else { spanned_tracks },
                            ..Default::default()
                        },
                        &[content],
                    )
                    .unwrap();
                let probe = tree
                    .new_leaf(Style {
                        grid_column: if horizontal { probe_track.clone() } else { first_track.clone() },
                        grid_row: if horizontal { first_track } else { probe_track },
                        ..Default::default()
                    })
                    .unwrap();
                let cross = vec![length(100.0)];
                let grid = tree
                    .new_with_children(
                        Style {
                            display: Display::Grid,
                            size: if horizontal {
                                Size { width: length(0.0), height: length(100.0) }
                            } else {
                                Size { width: length(100.0), height: length(0.0) }
                            },
                            grid_template_columns: if horizontal { tracks.clone() } else { cross.clone() },
                            grid_template_rows: if horizontal { cross } else { tracks },
                            ..Default::default()
                        },
                        &[item, probe],
                    )
                    .unwrap();
                tree.compute_layout(grid, Size::MAX_CONTENT).unwrap();
                let size = tree.layout(probe).unwrap().size;
                assert_eq!(
                    if horizontal { size.width } else { size.height },
                    expected,
                    "horizontal:{horizontal}; span:{item_span}"
                );
            }
        }
    }

    #[test]
    fn min_overrides_max() {
        let mut taffy = new_test_tree();

        let child = taffy
            .new_leaf(Style {
                size: Size { width: Dimension::from_length(50.0), height: Dimension::from_length(50.0) },
                min_size: Size { width: Dimension::from_length(100.0), height: Dimension::from_length(100.0) },
                max_size: Size { width: Dimension::from_length(10.0), height: Dimension::from_length(10.0) },
                ..Default::default()
            })
            .unwrap();

        taffy
            .compute_layout(
                child,
                Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(100.0) },
            )
            .unwrap();

        assert_eq!(taffy.layout(child).unwrap().size, Size { width: 100.0, height: 100.0 });
    }

    #[test]
    fn max_overrides_size() {
        let mut taffy = new_test_tree();

        let child = taffy
            .new_leaf(Style {
                size: Size { width: Dimension::from_length(50.0), height: Dimension::from_length(50.0) },
                max_size: Size { width: Dimension::from_length(10.0), height: Dimension::from_length(10.0) },
                ..Default::default()
            })
            .unwrap();

        taffy
            .compute_layout(
                child,
                Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(100.0) },
            )
            .unwrap();

        assert_eq!(taffy.layout(child).unwrap().size, Size { width: 10.0, height: 10.0 });
    }

    #[test]
    fn min_overrides_size() {
        let mut taffy = new_test_tree();

        let child = taffy
            .new_leaf(Style {
                size: Size { width: Dimension::from_length(50.0), height: Dimension::from_length(50.0) },
                min_size: Size { width: Dimension::from_length(100.0), height: Dimension::from_length(100.0) },
                ..Default::default()
            })
            .unwrap();

        taffy
            .compute_layout(
                child,
                Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(100.0) },
            )
            .unwrap();

        assert_eq!(taffy.layout(child).unwrap().size, Size { width: 100.0, height: 100.0 });
    }
}
