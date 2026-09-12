#![cfg(feature = "flexbox")]

use taffy::prelude::*;
use taffy::{geometry::Point, style::Overflow};

#[test]
fn ratio_dependent_cross_size_uses_the_automatic_content_minimum() {
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        let row = direction == FlexDirection::Row;
        let size =
            |main, cross| if row { Size { width: main, height: cross } } else { Size { width: cross, height: main } };
        for (minimum, maximum, preferred, overflow, replaced, expected) in [
            (auto(), auto(), auto(), Overflow::Visible, false, 200.0_f32),
            (auto(), auto(), auto(), Overflow::Clip, false, 200.0),
            (auto(), auto(), auto(), Overflow::Hidden, false, 100.0),
            (auto(), auto(), auto(), Overflow::Scroll, false, 100.0),
            (length(0.0), auto(), auto(), Overflow::Visible, false, 100.0),
            (auto(), length(150.0), auto(), Overflow::Visible, false, 150.0),
            (auto(), auto(), length(80.0), Overflow::Visible, false, 80.0),
            (auto(), auto(), auto(), Overflow::Visible, true, 100.0),
        ] {
            for main_maximum in [None, Some(50.0)] {
                let mut tree: TaffyTree<()> = TaffyTree::new();
                let item = tree
                    .new_leaf_with_context(
                        Style {
                            size: size(length(50.0), preferred),
                            min_size: size(auto(), minimum),
                            max_size: size(main_maximum.map_or_else(auto, length), maximum),
                            aspect_ratio: Some(if row { 0.5 } else { 2.0 }),
                            overflow: Point { x: overflow, y: overflow },
                            item_is_replaced: replaced,
                            ..Style::default()
                        },
                        (),
                    )
                    .unwrap();
                let parent = tree
                    .new_with_children(
                        Style {
                            display: Display::Flex,
                            flex_direction: direction,
                            size: size(length(50.0), auto()),
                            ..Style::default()
                        },
                        &[item],
                    )
                    .unwrap();
                tree.compute_layout_with_measure(parent, Size::MAX_CONTENT, |known, _, _, _, _| {
                    let content =
                        if row { Size { width: 50.0, height: 200.0 } } else { Size { width: 200.0, height: 50.0 } };
                    Size { width: known.width.unwrap_or(content.width), height: known.height.unwrap_or(content.height) }
                })
                .unwrap();
                let actual = tree.layout(item).unwrap().size;
                let expected = if main_maximum.is_some() && maximum.is_auto() { expected.min(100.0) } else { expected };
                assert_eq!(
                    if row { actual.height } else { actual.width },
                    expected,
                    "{direction:?} {minimum:?} {maximum:?} {preferred:?} {overflow:?} replaced={replaced}"
                );
            }
        }
    }
}
