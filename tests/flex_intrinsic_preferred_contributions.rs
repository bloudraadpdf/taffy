#![cfg(feature = "flexbox")]

use taffy::prelude::*;

fn axis_size<T>(row: bool, main: T, cross: T) -> Size<T> {
    if row {
        Size { width: main, height: cross }
    } else {
        Size { width: cross, height: main }
    }
}

#[test]
fn intrinsic_main_contributions_measure_content_before_applying_preferred_size() {
    for direction in
        [FlexDirection::Row, FlexDirection::RowReverse, FlexDirection::Column, FlexDirection::ColumnReverse]
    {
        let row = matches!(direction, FlexDirection::Row | FlexDirection::RowReverse);
        for preferred in [10.0_f32, 200.0] {
            for grow in [0.0, 1.0] {
                let mut tree: TaffyTree<()> = TaffyTree::new();
                let item = tree
                    .new_leaf(Style {
                        size: axis_size(row, length(preferred), length(100.0)),
                        min_size: Size { width: length(0.0), height: length(0.0) },
                        flex_basis: length(150.0),
                        flex_grow: grow,
                        ..Default::default()
                    })
                    .unwrap();
                let root = tree
                    .new_with_children(
                        Style {
                            flex_direction: direction,
                            size: axis_size(row, auto(), length(100.0)),
                            ..Default::default()
                        },
                        &[item],
                    )
                    .unwrap();
                tree.compute_layout_with_measure(root, Size::MAX_CONTENT, |known, _, _, _, _| {
                    let intrinsic = axis_size(row, 150.0, 100.0);
                    Size {
                        width: known.width.unwrap_or(intrinsic.width),
                        height: known.height.unwrap_or(intrinsic.height),
                    }
                })
                .unwrap();
                // Flexbox 9.9.3: max(content, preferred), capped by the base only
                // when the item cannot grow. The preferred size is not the content size.
                let expected = if grow == 0.0 { 150.0 } else { preferred.max(150.0) };
                for node in [root, item] {
                    let actual = tree.layout(node).unwrap().size;
                    assert_eq!(actual, axis_size(row, expected, 100.0), "{direction:?}/{preferred}/{grow}");
                }
            }
        }
    }
}
