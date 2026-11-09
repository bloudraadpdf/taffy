#![cfg(feature = "grid")]

use taffy::prelude::*;

#[test]
fn intrinsic_items_subtract_their_margins_once() {
    for horizontal in [true, false] {
        for margin in [15.0, 30.0, -5.0] {
            let mut tree = TaffyTree::<()>::new();
            let item = tree
                .new_leaf(Style {
                    margin: if horizontal {
                        Rect { left: length(margin), ..Rect::zero() }
                    } else {
                        Rect { top: length(margin), ..Rect::zero() }
                    },
                    justify_self: Some(AlignSelf::START),
                    align_self: Some(AlignSelf::START),
                    ..Default::default()
                })
                .unwrap();
            let root = tree
                .new_with_children(
                    Style {
                        display: Display::Grid,
                        grid_template_columns: vec![length(if horizontal { 100.0 } else { 20.0 })],
                        grid_template_rows: vec![length(if horizontal { 20.0 } else { 100.0 })],
                        ..Default::default()
                    },
                    &[item],
                )
                .unwrap();
            tree.compute_layout_with_measure(root, Size::MAX_CONTENT, |known, available, _, _, _| {
                let available = if horizontal { available.width } else { available.height };
                let extent = match available {
                    AvailableSpace::MinContent => 40.0,
                    AvailableSpace::MaxContent => 80.0,
                    AvailableSpace::Definite(size) => size.clamp(40.0, 80.0),
                };
                let measured = if horizontal {
                    Size { width: extent, height: 10.0 }
                } else {
                    Size { width: 10.0, height: extent }
                };
                known.unwrap_or(measured)
            })
            .unwrap();
            let size = tree.layout(item).unwrap().size;
            assert_eq!(if horizontal { size.width } else { size.height }, (100.0_f32 - margin).clamp(40.0, 80.0));
        }
    }
}
