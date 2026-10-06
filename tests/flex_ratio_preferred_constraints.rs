#![cfg(all(feature = "flexbox", feature = "block_layout"))]

use taffy::prelude::*;

#[test]
fn cross_constraints_affect_automatic_basis_without_becoming_main_minima() {
    let mut failures = Vec::new();
    let mut count = 0;
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        let row = direction == FlexDirection::Row;
        let size =
            |main, cross| if row { Size { width: main, height: cross } } else { Size { width: cross, height: main } };
        for (name, content, minimum, maximum, basis, main_minimum, available, expected_main, expected_cross) in [
            ("cross-minimum", 0.0, Some(50.0), None, None, Some(0.0), 240.0, 100.0, 50.0),
            ("cross-minimum-shrink", 0.0, Some(50.0), None, None, Some(0.0), 0.0, 0.0, 50.0),
            ("automatic-main-minimum", 0.0, Some(50.0), None, None, None, 240.0, 100.0, 50.0),
            ("automatic-main-minimum-shrink", 0.0, Some(50.0), None, None, None, 0.0, 100.0, 50.0),
            ("large-content", 200.0, Some(50.0), None, None, Some(0.0), 240.0, 200.0, 100.0),
            ("large-content-shrink", 200.0, Some(50.0), None, None, Some(0.0), 0.0, 0.0, 50.0),
            ("content-automatic-minimum", 200.0, Some(50.0), None, None, None, 0.0, 200.0, 100.0),
            ("cross-maximum", 200.0, None, Some(50.0), None, Some(0.0), 240.0, 100.0, 50.0),
            ("cross-maximum-shrink", 200.0, None, Some(50.0), None, Some(0.0), 0.0, 0.0, 0.0),
            ("cross-maximum-automatic-minimum", 200.0, None, Some(50.0), None, None, 0.0, 100.0, 50.0),
            ("explicit-basis", 0.0, Some(50.0), None, Some(30.0), Some(0.0), 240.0, 30.0, 50.0),
            ("explicit-basis-shrink", 0.0, Some(50.0), None, Some(30.0), Some(0.0), 0.0, 0.0, 50.0),
            ("explicit-basis-automatic-minimum", 0.0, Some(50.0), None, Some(30.0), None, 240.0, 100.0, 50.0),
        ] {
            let mut tree = TaffyTree::<()>::new();
            let content_node =
                tree.new_leaf(Style { size: size(length(content), length(0.0)), ..Default::default() }).unwrap();
            let item = tree
                .new_with_children(
                    Style {
                        display: Display::Block,
                        min_size: size(main_minimum.map_or(auto(), length), minimum.map_or(auto(), length)),
                        max_size: size(auto(), maximum.map_or(auto(), length)),
                        flex_basis: basis.map_or(auto(), length),
                        aspect_ratio: Some(if row { 2.0 } else { 0.5 }),
                        ..Default::default()
                    },
                    &[content_node],
                )
                .unwrap();
            let root = tree
                .new_with_children(
                    Style { flex_direction: direction, size: size(length(available), auto()), ..Default::default() },
                    &[item],
                )
                .unwrap();
            tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
            let actual = tree.layout(item).unwrap().size;
            let expected = if row {
                Size { width: expected_main, height: expected_cross }
            } else {
                Size { width: expected_cross, height: expected_main }
            };
            if actual != expected {
                failures.push(format!("{direction:?}/{name}: {actual:?}, expected {expected:?}"));
            }
            count += 1;
        }
    }
    assert!(failures.is_empty(), "{}/{} failed:\n{}", failures.len(), count, failures.join("\n"));
}
