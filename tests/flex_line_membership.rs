use taffy::prelude::*;
use taffy::tree::DetailedLayoutInfo;
use taffy::Direction;

#[test]
fn margin_trim_changes_line_collection_in_each_direction() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for flex_direction in
            [FlexDirection::Row, FlexDirection::RowReverse, FlexDirection::Column, FlexDirection::ColumnReverse]
        {
            for flex_wrap in [FlexWrap::Wrap, FlexWrap::WrapReverse, FlexWrap::Balance, FlexWrap::BalanceReverse] {
                let mut tree: TaffyTree<()> = TaffyTree::new();
                let items: Vec<_> = (0..4)
                    .map(|_| {
                        tree.new_leaf(Style {
                            size: Size { width: length(50.0), height: length(50.0) },
                            margin: Rect {
                                left: length(10.0),
                                right: length(10.0),
                                top: length(10.0),
                                bottom: length(10.0),
                            },
                            ..Style::default()
                        })
                        .unwrap()
                    })
                    .collect();
                let parent = tree
                    .new_with_children(
                        Style {
                            display: Display::Flex,
                            direction,
                            flex_direction,
                            flex_wrap,
                            size: Size { width: length(120.0), height: length(120.0) },
                            margin_trim: Rect { left: true, right: true, top: true, bottom: true },
                            ..Style::default()
                        },
                        &items,
                    )
                    .unwrap();
                tree.compute_layout(parent, Size::MAX_CONTENT).unwrap();
                let mut positions: Vec<_> = items
                    .iter()
                    .map(|&item| {
                        let layout = tree.layout(item).unwrap();
                        let location = layout.location;
                        assert_eq!(
                            layout.margin,
                            Rect {
                                left: if location.x == 0.0 { 0.0 } else { 10.0 },
                                right: if location.x == 70.0 { 0.0 } else { 10.0 },
                                top: if location.y == 0.0 { 0.0 } else { 10.0 },
                                bottom: if location.y == 70.0 { 0.0 } else { 10.0 },
                            },
                            "{direction:?} {flex_direction:?} {flex_wrap:?} at {location:?}"
                        );
                        (location.x, location.y)
                    })
                    .collect();
                positions.sort_by(|a, b| a.partial_cmp(b).unwrap());
                assert_eq!(
                    positions,
                    [(0.0, 0.0), (0.0, 70.0), (70.0, 0.0), (70.0, 70.0)],
                    "{direction:?} {flex_direction:?} {flex_wrap:?}"
                );
            }
        }
    }
}

#[test]
fn a_trimmed_auto_margin_does_not_absorb_free_space() {
    let mut tree: TaffyTree<()> = TaffyTree::new();
    let item = tree
        .new_leaf(Style {
            size: Size { width: length(50.0), height: length(50.0) },
            margin: Rect { left: auto(), ..Rect::zero() },
            ..Style::default()
        })
        .unwrap();
    let parent = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                size: Size { width: length(200.0), height: length(100.0) },
                margin_trim: Rect { left: true, right: false, top: false, bottom: false },
                ..Style::default()
            },
            &[item],
        )
        .unwrap();
    tree.compute_layout(parent, Size::MAX_CONTENT).unwrap();
    assert_eq!(tree.layout(item).unwrap().location.x, 0.0);
    assert_eq!(tree.layout(item).unwrap().margin.left, 0.0);
}

#[test]
fn trimmed_cross_margins_affect_the_stretched_aspect_ratio_flex_basis() {
    let mut tree: TaffyTree<()> = TaffyTree::new();
    let item = tree
        .new_leaf(Style {
            aspect_ratio: Some(1.0),
            margin: Rect { top: length(10.0), bottom: length(10.0), ..Rect::zero() },
            ..Style::default()
        })
        .unwrap();
    let parent = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                size: Size { width: length(200.0), height: length(100.0) },
                margin_trim: Rect { left: false, right: false, top: true, bottom: true },
                ..Style::default()
            },
            &[item],
        )
        .unwrap();
    tree.compute_layout_with_measure(parent, Size::MAX_CONTENT, |known, _, _, _, _| Size {
        width: known.width.or(known.height).unwrap_or(0.0),
        height: known.height.or(known.width).unwrap_or(0.0),
    })
    .unwrap();
    assert_eq!(tree.layout(item).unwrap().size, Size { width: 100.0, height: 100.0 });
}

#[test]
fn flex_lines_remain_observable_when_item_alignment_changes_offsets() {
    let mut tree: TaffyTree<()> = TaffyTree::new();
    let first = tree
        .new_leaf(Style {
            size: Size { width: length(50.0), height: length(20.0) },
            margin: Rect { top: length(10.0), ..Rect::zero() },
            ..Style::default()
        })
        .unwrap();
    let second = tree
        .new_leaf(Style {
            size: Size { width: length(50.0), height: length(40.0) },
            margin: Rect { top: length(20.0), ..Rect::zero() },
            ..Style::default()
        })
        .unwrap();
    let third =
        tree.new_leaf(Style { size: Size { width: length(50.0), height: length(50.0) }, ..Style::default() }).unwrap();
    let parent = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                size: Size { width: length(100.0), height: auto() },
                flex_wrap: FlexWrap::Wrap,
                align_items: Some(AlignItems::CENTER),
                ..Style::default()
            },
            &[first, second, third],
        )
        .unwrap();
    tree.compute_layout(parent, Size::MAX_CONTENT).unwrap();
    assert_ne!(tree.layout(first).unwrap().location.y, tree.layout(second).unwrap().location.y);
    let DetailedLayoutInfo::Flex(info) = tree.detailed_layout_info(parent) else {
        panic!("flex layout must retain its line partition");
    };
    assert_eq!(info.lines, [vec![first, second], vec![third]]);
}

#[test]
fn percentage_ratio_roundoff_does_not_create_an_extra_flex_line() {
    for direction in
        [FlexDirection::Row, FlexDirection::RowReverse, FlexDirection::Column, FlexDirection::ColumnReverse]
    {
        let row = matches!(direction, FlexDirection::Row | FlexDirection::RowReverse);
        let size =
            |main, cross| if row { Size { width: main, height: cross } } else { Size { width: cross, height: main } };
        for wrap in [FlexWrap::Wrap, FlexWrap::WrapReverse, FlexWrap::Balance, FlexWrap::BalanceReverse] {
            for real_overflow in [false, true] {
                let mut tree = TaffyTree::<()>::new();
                let items: Vec<_> = (0..4)
                    .map(|_| {
                        tree.new_leaf(Style {
                            size: size(if real_overflow { length(45.01) } else { auto() }, percent(0.6)),
                            aspect_ratio: Some(1.0),
                            flex_shrink: 0.0,
                            ..Default::default()
                        })
                        .unwrap()
                    })
                    .collect();
                let root = tree
                    .new_with_children(
                        Style {
                            display: Display::Flex,
                            flex_direction: direction,
                            flex_wrap: wrap,
                            size: size(length(105.0), length(75.0)),
                            gap: Size { width: length(15.0), height: length(15.0) },
                            ..Default::default()
                        },
                        &items,
                    )
                    .unwrap();
                tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
                let DetailedLayoutInfo::Flex(info) = tree.detailed_layout_info(root) else {
                    panic!("flex layout must retain its line partition");
                };
                let expected = if real_overflow {
                    items.iter().map(|&item| vec![item]).collect::<Vec<_>>()
                } else {
                    vec![items[0..2].to_vec(), items[2..4].to_vec()]
                };
                assert_eq!(info.lines, expected, "{direction:?}/{wrap:?}/overflow:{real_overflow}");
            }
        }
    }
}
