#![cfg(feature = "grid")]

use taffy::prelude::*;

#[test]
fn virtual_subgrid_margins_reduce_the_intrinsic_ratio_cross_extent() {
    for horizontal in [false, true] {
        for (track, margin) in [(211.5, 0.75), (213.0, 2.25), (210.0, -0.75)] {
            for percentage in [false, true] {
                let expected_ratio_extent = if percentage { track } else { 210.75 };
                let mut tree = TaffyTree::<()>::new();
                tree.disable_rounding();
                let item = tree
                    .new_leaf(Style {
                        aspect_ratio: Some(1.0),
                        size: if percentage {
                            if horizontal {
                                Size { width: percent(1.0), height: auto() }
                            } else {
                                Size { width: auto(), height: percent(1.0) }
                            }
                        } else {
                            Size::AUTO
                        },
                        min_size: Size { width: length(0.0), height: length(0.0) },
                        align_self: Some(AlignItems::STRETCH),
                        justify_self: Some(AlignItems::STRETCH),
                        grid_subgrid_margin: if horizontal {
                            Rect { left: margin, ..Rect::zero() }
                        } else {
                            Rect { top: margin, ..Rect::zero() }
                        },
                        ..Default::default()
                    })
                    .unwrap();
                let root = tree
                    .new_with_children(
                        Style {
                            display: Display::Grid,
                            grid_template_columns: vec![if horizontal { length(track) } else { auto() }],
                            grid_template_rows: vec![if horizontal { auto() } else { length(track) }],
                            ..Default::default()
                        },
                        &[item],
                    )
                    .unwrap();
                tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
                let expected = if horizontal {
                    Size { width: track, height: expected_ratio_extent }
                } else {
                    Size { width: expected_ratio_extent, height: track }
                };
                assert_eq!(
                    tree.layout(root).unwrap().size,
                    expected,
                    "horizontal={horizontal}; margin={margin}; percentage={percentage}"
                );
                assert_eq!(
                    tree.layout(item).unwrap().size,
                    Size { width: expected_ratio_extent, height: expected_ratio_extent },
                    "horizontal={horizontal}; margin={margin}; percentage={percentage}"
                );
            }
        }
    }
}
