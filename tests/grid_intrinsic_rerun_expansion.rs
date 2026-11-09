#![cfg(feature = "grid")]

use taffy::prelude::*;

#[test]
fn intrinsic_rerun_does_not_stretch_into_its_previous_intrinsic_sum() {
    for definite_width in [false, true] {
        let mut tree = TaffyTree::<()>::new();
        let item = tree
            .new_leaf(Style {
                max_size: Size { width: auto(), height: percent(0.5) },
                justify_self: Some(AlignSelf::START),
                align_self: Some(AlignSelf::START),
                ..Default::default()
            })
            .unwrap();
        let root = tree
            .new_with_children(
                Style {
                    display: Display::Grid,
                    // The row resolves from the container after the initial column pass;
                    // its minimum must not grow to the image's unconstrained 400px height.
                    grid_template_rows: vec![minmax(length(0.0), fr(1.0))],
                    size: Size { width: if definite_width { length(200.0) } else { auto() }, height: length(300.0) },
                    ..Default::default()
                },
                &[item],
            )
            .unwrap();
        tree.compute_layout_with_measure(root, Size::MAX_CONTENT, |known, available, _, _, _| {
            let width = match available.height {
                AvailableSpace::Definite(height) => height.min(400.0) / 2.0,
                _ => 200.0,
            };
            known.unwrap_or(Size { width, height: 400.0 })
        })
        .unwrap();
        assert_eq!(tree.layout(root).unwrap().size.width, if definite_width { 200.0 } else { 75.0 });
        assert_eq!(tree.layout(item).unwrap().size, Size { width: 75.0, height: 150.0 });
    }
}
