#[cfg(feature = "grid")]
mod tests {
    use taffy::prelude::*;
    use taffy::{geometry::Point, style::Overflow};

    #[test]
    fn ratio_transfer_does_not_make_the_preferred_axis_definite() {
        for flow in [GridAutoFlow::Column, GridAutoFlow::Row] {
            let columns = flow == GridAutoFlow::Column;
            let size = |main, cross| {
                if columns {
                    Size { width: main, height: cross }
                } else {
                    Size { width: cross, height: main }
                }
            };
            for available in [AvailableSpace::MinContent, AvailableSpace::Definite(0.0)] {
                for overflow in [Overflow::Visible, Overflow::Clip, Overflow::Hidden] {
                    let automatic_minimum: f32 = if overflow == Overflow::Hidden { 0.0 } else { 200.0 };
                    for (preferred, minimum, maximum, expected) in [
                        (auto(), auto(), auto(), automatic_minimum),
                        (auto(), length(0.0), auto(), 0.0),
                        (auto(), length(80.0), auto(), 80.0),
                        (length(80.0), auto(), auto(), 80.0),
                        (auto(), auto(), length(80.0), automatic_minimum.min(80.0)),
                    ] {
                        let mut tree = TaffyTree::<()>::new();
                        let first = tree
                            .new_leaf(Style {
                                size: size(preferred, length(100.0)),
                                min_size: size(minimum, auto()),
                                max_size: size(maximum, auto()),
                                aspect_ratio: Some(if columns { 2.0 } else { 0.5 }),
                                overflow: Point { x: overflow, y: overflow },
                                ..Default::default()
                            })
                            .unwrap();
                        let second = tree
                            .new_leaf(Style {
                                size: size(auto(), length(100.0)),
                                min_size: size(length(100.0), auto()),
                                aspect_ratio: Some(if columns { 0.5 } else { 2.0 }),
                                overflow: Point { x: overflow, y: overflow },
                                ..Default::default()
                            })
                            .unwrap();
                        let root = tree
                            .new_with_children(
                                Style {
                                    display: Display::Grid,
                                    grid_auto_flow: flow,
                                    size: size(
                                        if available == AvailableSpace::MinContent { auto() } else { length(0.0) },
                                        length(100.0),
                                    ),
                                    ..Default::default()
                                },
                                &[first, second],
                            )
                            .unwrap();
                        let available_space = if columns {
                            Size { width: available, height: AvailableSpace::MaxContent }
                        } else {
                            Size { width: AvailableSpace::MaxContent, height: available }
                        };
                        tree.compute_layout(root, available_space).unwrap();
                        let root_layout = tree.layout(root).unwrap();
                        let second_layout = tree.layout(second).unwrap();
                        let (extent, origin) = if columns {
                            (root_layout.size.width, second_layout.location.x)
                        } else {
                            (root_layout.size.height, second_layout.location.y)
                        };
                        let expected_extent =
                            if available == AvailableSpace::MinContent { expected + 100.0 } else { 0.0 };
                        assert_eq!((extent, origin), (expected_extent, expected),
                            "{flow:?}/{available:?}/{overflow:?}, preferred={preferred:?}, min={minimum:?}, max={maximum:?}");
                    }
                }
            }
        }
    }
}
