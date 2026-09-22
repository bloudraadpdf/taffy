#![cfg(all(feature = "flexbox", feature = "detailed_layout_info"))]

use taffy::geometry::Rect;
use taffy::prelude::*;
use taffy::{Direction, FlexItemEdgeAnchor};

fn anchors(tree: &TaffyTree<()>, root: NodeId, item: NodeId) -> Size<Option<FlexItemEdgeAnchor>> {
    let taffy::tree::DetailedLayoutInfo::Flex(info) = tree.detailed_layout_info(root) else {
        panic!("flex metadata");
    };
    info.items.iter().find(|entry| entry.node == item).unwrap().edge_anchors
}

#[test]
fn fractional_flex_budgets_retain_the_terminal_parent_edge() {
    for mechanism in ["grow", "margins", "justify", "lines", "stretch"] {
        for step in 0..10 {
            let extent = 40.0 + step as f32 / 10.0;
            let cross = matches!(mechanism, "lines" | "stretch");
            let mut tree: TaffyTree<()> = TaffyTree::new();
            tree.disable_rounding();
            let items = (0..8)
                .map(|_| {
                    tree.new_leaf(Style {
                        size: Size {
                            width: length(if cross { 10.0 } else { 5.0 }),
                            height: if mechanism == "stretch" { auto() } else { length(5.0) },
                        },
                        min_size: Size { width: auto(), height: length(5.0) },
                        flex_grow: if mechanism == "grow" { 1.0 } else { 0.0 },
                        margin: Rect { left: if mechanism == "margins" { auto() } else { zero() }, ..Rect::zero() },
                        ..Style::default()
                    })
                    .unwrap()
                })
                .collect::<Vec<_>>();
            let root = tree
                .new_with_children(
                    Style {
                        size: Size {
                            width: length(if cross { 10.0 } else { extent }),
                            height: if cross { length(extent) } else { auto() },
                        },
                        flex_wrap: if cross { FlexWrap::Wrap } else { FlexWrap::NoWrap },
                        justify_content: Some(if mechanism == "justify" {
                            JustifyContent::SPACE_BETWEEN
                        } else {
                            JustifyContent::FLEX_START
                        }),
                        align_content: Some(if mechanism == "lines" {
                            AlignContent::SPACE_BETWEEN
                        } else {
                            AlignContent::STRETCH
                        }),
                        ..Style::default()
                    },
                    &items,
                )
                .unwrap();
            tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
            let edges = anchors(&tree, root, items[7]);
            assert_eq!(
                if cross { edges.height } else { edges.width },
                Some(FlexItemEdgeAnchor::End(0.0)),
                "{mechanism}; {extent}"
            );
            let last = tree.unrounded_layout(items[7]);
            let root = tree.unrounded_layout(root);
            assert_eq!(
                if cross { last.location.y + last.size.height } else { last.location.x + last.size.width },
                if cross { root.size.height } else { root.size.width },
                "native geometry: {mechanism}; {extent}"
            );
        }
    }
}

#[test]
fn unconsumed_space_and_overflow_are_not_reported_as_filled() {
    for (grow, basis, maximum, expected_inset) in [
        (0.25, 0.0, None, 50.0),
        (1.0, 0.0, Some(20.0), 60.0),
        (0.0, 60.0, None, -20.0),
        (0.0, 49.999, None, 100.0 - 2.0 * 49.999_f32),
    ] {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        tree.disable_rounding();
        let items = (0..2)
            .map(|_| {
                tree.new_leaf(Style {
                    size: Size { width: length(basis), height: length(10.0) },
                    max_size: Size { width: maximum.map_or(auto(), length), height: auto() },
                    flex_grow: grow,
                    flex_shrink: 0.0,
                    ..Style::default()
                })
                .unwrap()
            })
            .collect::<Vec<_>>();
        let root = tree
            .new_with_children(
                Style { size: Size { width: length(100.0), height: auto() }, ..Style::default() },
                &items,
            )
            .unwrap();
        tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
        assert_eq!(anchors(&tree, root, items[1]).width, Some(FlexItemEdgeAnchor::End(expected_inset)));
    }
}

#[test]
fn a_single_item_retains_its_aligned_edge_with_fractional_free_space() {
    for rtl in [false, true] {
        for reverse in [false, true] {
            for end in [false, true] {
                for extent in [45.000004, 175.00002] {
                    let mut tree: TaffyTree<()> = TaffyTree::new();
                    tree.disable_rounding();
                    let item = tree
                        .new_leaf(Style {
                            size: Size { width: length(extent), height: length(10.0) },
                            flex_shrink: 0.0,
                            ..Style::default()
                        })
                        .unwrap();
                    let root = tree
                        .new_with_children(
                            Style {
                                size: Size { width: length(165.0), height: auto() },
                                direction: if rtl { Direction::Rtl } else { Direction::Ltr },
                                flex_direction: if reverse { FlexDirection::RowReverse } else { FlexDirection::Row },
                                justify_content: Some(if end {
                                    JustifyContent::FLEX_END
                                } else {
                                    JustifyContent::FLEX_START
                                }),
                                ..Style::default()
                            },
                            &[item],
                        )
                        .unwrap();
                    tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
                    let at_end = rtl ^ reverse ^ end;
                    assert_eq!(
                        anchors(&tree, root, item).width,
                        Some(if at_end { FlexItemEdgeAnchor::End(0.0) } else { FlexItemEdgeAnchor::Start(0.0) })
                    );
                    if !at_end {
                        assert_eq!(tree.unrounded_layout(item).location.x, 0.0);
                    }
                }
            }
        }
    }
}

#[test]
fn edge_constraints_include_physical_insets_margins_and_relative_position() {
    for rtl in [false, true] {
        for reverse in [false, true] {
            let mut tree: TaffyTree<()> = TaffyTree::new();
            let items = (0..2)
                .map(|_| {
                    tree.new_leaf(Style {
                        size: Size { width: length(10.0), height: length(10.0) },
                        margin: Rect { left: length(1.0), right: length(4.0), ..Rect::zero() },
                        inset: Rect { left: length(2.0), ..Rect::auto() },
                        ..Style::default()
                    })
                    .unwrap()
                })
                .collect::<Vec<_>>();
            let root = tree
                .new_with_children(
                    Style {
                        direction: if rtl { Direction::Rtl } else { Direction::Ltr },
                        flex_direction: if reverse { FlexDirection::RowReverse } else { FlexDirection::Row },
                        size: Size { width: length(100.0), height: auto() },
                        padding: Rect { left: length(3.0), right: length(7.0), ..Rect::zero() },
                        border: Rect { left: length(2.0), right: length(5.0), ..Rect::zero() },
                        justify_content: Some(JustifyContent::SPACE_BETWEEN),
                        ..Style::default()
                    },
                    &items,
                )
                .unwrap();
            tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
            let left = items[usize::from(rtl ^ reverse)];
            let right = items[usize::from(!(rtl ^ reverse))];
            assert_eq!(anchors(&tree, root, left).width, Some(FlexItemEdgeAnchor::Start(8.0)));
            assert_eq!(anchors(&tree, root, right).width, Some(FlexItemEdgeAnchor::End(14.0)));
        }
    }
}

#[test]
fn cross_edges_follow_wrap_reversal_and_rtl_columns() {
    for column in [false, true] {
        for rtl in [false, true] {
            for reverse in [false, true] {
                let mut tree: TaffyTree<()> = TaffyTree::new();
                tree.disable_rounding();
                let items = (0..8)
                    .map(|_| {
                        tree.new_leaf(Style {
                            size: if column {
                                Size { width: length(5.0), height: length(10.0) }
                            } else {
                                Size { width: length(10.0), height: length(5.0) }
                            },
                            ..Style::default()
                        })
                        .unwrap()
                    })
                    .collect::<Vec<_>>();
                let root = tree
                    .new_with_children(
                        Style {
                            direction: if rtl { Direction::Rtl } else { Direction::Ltr },
                            flex_direction: if column { FlexDirection::Column } else { FlexDirection::Row },
                            flex_wrap: if reverse { FlexWrap::WrapReverse } else { FlexWrap::Wrap },
                            size: if column {
                                Size { width: length(40.9), height: length(10.0) }
                            } else {
                                Size { width: length(10.0), height: length(40.9) }
                            },
                            align_content: Some(AlignContent::SPACE_BETWEEN),
                            ..Style::default()
                        },
                        &items,
                    )
                    .unwrap();
                tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
                let item = items[if reverse ^ (column && rtl) { 0 } else { 7 }];
                let edges = anchors(&tree, root, item);
                assert_eq!(
                    if column { edges.width } else { edges.height },
                    Some(FlexItemEdgeAnchor::End(0.0)),
                    "column={column}; rtl={rtl}; reverse={reverse}"
                );
            }
        }
    }
}
