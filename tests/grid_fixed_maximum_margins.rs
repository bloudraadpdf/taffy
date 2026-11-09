#![cfg(feature = "grid")]

use taffy::prelude::*;

#[test]
fn automatic_minimum_fits_the_fixed_track_after_margins() {
    for margin in [0.0, 5.0, -5.0] {
        let mut tree = TaffyTree::<()>::new();
        let content = tree
            .new_leaf(Style { size: Size { width: length(20.0), height: length(100.0) }, ..Default::default() })
            .unwrap();
        let item = tree
            .new_with_children(
                Style {
                    margin: Rect { top: length(margin), bottom: length(margin), ..Rect::zero() },
                    ..Default::default()
                },
                &[content],
            )
            .unwrap();
        let root = tree
            .new_with_children(
                Style {
                    display: Display::Grid,
                    grid_template_columns: vec![length(50.0)],
                    grid_auto_rows: vec![minmax(auto(), length(40.0))],
                    ..Default::default()
                },
                &[item],
            )
            .unwrap();
        tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
        assert_eq!(tree.layout(root).unwrap().size.height, 40.0, "margin {margin}");
        assert_eq!(tree.layout(item).unwrap().size.height, 40.0 - 2.0 * margin, "margin {margin}");
    }
}
