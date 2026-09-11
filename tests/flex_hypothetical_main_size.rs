#![cfg(feature = "flexbox")]

use taffy::prelude::*;

#[test]
fn column_main_size_uses_hypothetical_items_before_flexing() {
    let mut failures = Vec::new();
    for direction in
        [FlexDirection::Row, FlexDirection::RowReverse, FlexDirection::Column, FlexDirection::ColumnReverse]
    {
        let row = matches!(direction, FlexDirection::Row | FlexDirection::RowReverse);
        let size =
            |main, cross| if row { Size { width: main, height: cross } } else { Size { width: cross, height: main } };
        for basis in [0.0_f32, 20.0, 50.0] {
            for grow in [0.0, 0.5, 1.0] {
                for maximum in [None, Some(30.0)] {
                    let mut tree: TaffyTree<()> = TaffyTree::new();
                    let child = tree
                        .new_leaf(Style {
                            size: size(length(200.0), length(10.0)),
                            min_size: Size { width: length(0.0), height: length(0.0) },
                            flex_basis: length(basis),
                            flex_grow: grow,
                            ..Default::default()
                        })
                        .unwrap();
                    let container = tree
                        .new_with_children(
                            Style {
                                display: Display::Flex,
                                flex_direction: direction,
                                flex_main_sizing: taffy::style::FlexMainSizing::HypotheticalItems,
                                size: size(auto(), length(100.0)),
                                max_size: size(maximum.map_or(auto(), length), auto()),
                                ..Default::default()
                            },
                            &[child],
                        )
                        .unwrap();
                    tree.compute_layout(container, Size::MAX_CONTENT).unwrap();
                    let expected = basis.min(maximum.unwrap_or(f32::INFINITY));
                    for node in [container, child] {
                        let layout = tree.layout(node).unwrap();
                        let actual = if row { layout.size.width } else { layout.size.height };
                        if actual != expected {
                            failures.push(format!(
                                "{direction:?}/{basis}/{grow}/{maximum:?}/{node:?}: {actual} != {expected}"
                            ));
                        }
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn hypothetical_main_size_keeps_bounds_edges_and_margin_trim() {
    for trim in [false, true] {
        for minimum in [0.0, 100.0] {
            let mut tree: TaffyTree<()> = TaffyTree::new();
            let children = [20.0, 50.0].map(|basis| {
                tree.new_leaf(Style {
                    size: Size { width: length(10.0), height: length(200.0) },
                    min_size: Size { width: length(0.0), height: length(0.0) },
                    max_size: Size { width: auto(), height: if basis == 50.0 { length(30.0) } else { auto() } },
                    flex_basis: length(basis),
                    flex_grow: 1.0,
                    margin: Rect { top: length(2.0), bottom: length(2.0), left: zero(), right: zero() },
                    ..Default::default()
                })
                .unwrap()
            });
            let container = tree
                .new_with_children(
                    Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        flex_main_sizing: taffy::style::FlexMainSizing::HypotheticalItems,
                        size: Size { width: length(100.0), height: auto() },
                        min_size: Size { width: auto(), height: length(minimum) },
                        gap: Size { width: zero(), height: length(10.0) },
                        padding: Rect { top: length(5.0), bottom: length(5.0), left: zero(), right: zero() },
                        border: Rect { top: length(1.0), bottom: length(1.0), left: zero(), right: zero() },
                        margin_trim: Rect { top: trim, bottom: trim, left: false, right: false },
                        ..Default::default()
                    },
                    &children,
                )
                .unwrap();
            tree.compute_layout(container, Size::MAX_CONTENT).unwrap();
            let natural = if trim { 76.0_f32 } else { 80.0 };
            let expected = natural.max(minimum);
            assert_eq!(tree.layout(container).unwrap().size.height, expected);
            assert_eq!(tree.layout(children[0]).unwrap().size.height, 20.0 + expected - natural);
            assert_eq!(tree.layout(children[1]).unwrap().size.height, 30.0);
        }
    }
}
