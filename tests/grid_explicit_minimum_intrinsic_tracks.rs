#![cfg(feature = "grid")]

use taffy::prelude::*;

#[test]
fn intrinsic_auto_tracks_respect_the_explicit_item_minimum() {
    for (minimum, auto_track_minimum) in [(auto(), 40.0), (length(0.0), 0.0), (length(20.0), 20.0), (percent(0.5), 0.0)]
    {
        for horizontal in [true, false] {
            for span in [1, 2] {
                for (track, expected_minimum) in [
                    (minmax::<GridTemplateComponent<String>>(auto(), auto()), auto_track_minimum),
                    (minmax(min_content(), auto()), 40.0),
                    (minmax(max_content(), auto()), 80.0),
                ] {
                    for max_content in [false, true] {
                        let mut tree = TaffyTree::<()>::new();
                        let item = tree
                            .new_leaf(Style {
                                min_size: if horizontal {
                                    Size { width: minimum, height: auto() }
                                } else {
                                    Size { width: auto(), height: minimum }
                                },
                                grid_column: Line { start: line(1), end: line(span + 1) },
                                grid_row: Line { start: line(1), end: line(span + 1) },
                                ..Default::default()
                            })
                            .unwrap();
                        let root = tree
                            .new_with_children(
                                Style {
                                    display: Display::Grid,
                                    grid_template_columns: if horizontal {
                                        vec![track.clone(); span as usize]
                                    } else {
                                        vec![length(10.0)]
                                    },
                                    grid_template_rows: if horizontal {
                                        vec![length(10.0)]
                                    } else {
                                        vec![track.clone(); span as usize]
                                    },
                                    ..Default::default()
                                },
                                &[item],
                            )
                            .unwrap();
                        let constraint =
                            if max_content { AvailableSpace::MaxContent } else { AvailableSpace::MinContent };
                        let available = if horizontal {
                            Size { width: constraint, height: AvailableSpace::MaxContent }
                        } else {
                            Size { width: AvailableSpace::MaxContent, height: constraint }
                        };
                        tree.compute_layout_with_measure(root, available, |known, available, _, _, _| {
                            let extent = match if horizontal { available.width } else { available.height } {
                                AvailableSpace::MinContent => 40.0,
                                AvailableSpace::MaxContent => 80.0,
                                AvailableSpace::Definite(value) => value,
                            };
                            known.unwrap_or(if horizontal {
                                Size { width: extent, height: 10.0 }
                            } else {
                                Size { width: 10.0, height: extent }
                            })
                        })
                        .unwrap();
                        let size = tree.layout(root).unwrap().size;
                        assert_eq!(
                            if horizontal { size.width } else { size.height },
                            if max_content { 80.0 } else { expected_minimum },
                            "minimum {minimum:?}, horizontal {horizontal}, span {span}, max-content {max_content}"
                        );
                    }
                }
            }
        }
    }
}
