#[cfg(test)]
mod min_max_overrides {
    use taffy::prelude::*;
    use taffy_test_helpers::new_test_tree;

    #[test]
    fn intrinsic_track_minimum_can_grow_past_an_initial_zero_growth_limit() {
        for minimum in
            [MinTrackSizingFunction::AUTO, MinTrackSizingFunction::MIN_CONTENT, MinTrackSizingFunction::MAX_CONTENT]
        {
            for explicit_minimum in [false, true] {
                let mut tree = new_test_tree();
                let size = Size { width: length(60.0), height: length(40.0) };
                let child = tree
                    .new_leaf(Style {
                        size: if explicit_minimum { Size::AUTO } else { size },
                        min_size: if explicit_minimum { size } else { Size::AUTO },
                        ..Default::default()
                    })
                    .unwrap();
                let stretched = tree.new_leaf(Style::default()).unwrap();
                let track = GridTemplateComponent::Single(minmax(minimum, length(0.0)));
                let grid = tree
                    .new_with_children(
                        Style {
                            display: Display::Grid,
                            grid_template_columns: vec![track.clone()],
                            grid_template_rows: vec![track],
                            ..Default::default()
                        },
                        &[child, stretched],
                    )
                    .unwrap();
                tree.compute_layout(grid, Size::MAX_CONTENT).unwrap();
                assert_eq!(
                    tree.layout(stretched).unwrap().size.width,
                    60.0,
                    "{minimum:?}; explicit minimum {explicit_minimum}"
                );
                assert_eq!(
                    tree.layout(grid).unwrap().size.height,
                    40.0,
                    "{minimum:?}; explicit minimum {explicit_minimum}"
                );
            }
        }
    }

    #[test]
    fn min_overrides_max() {
        let mut taffy = new_test_tree();

        let child = taffy
            .new_leaf(Style {
                size: Size { width: Dimension::from_length(50.0), height: Dimension::from_length(50.0) },
                min_size: Size { width: Dimension::from_length(100.0), height: Dimension::from_length(100.0) },
                max_size: Size { width: Dimension::from_length(10.0), height: Dimension::from_length(10.0) },
                ..Default::default()
            })
            .unwrap();

        taffy
            .compute_layout(
                child,
                Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(100.0) },
            )
            .unwrap();

        assert_eq!(taffy.layout(child).unwrap().size, Size { width: 100.0, height: 100.0 });
    }

    #[test]
    fn max_overrides_size() {
        let mut taffy = new_test_tree();

        let child = taffy
            .new_leaf(Style {
                size: Size { width: Dimension::from_length(50.0), height: Dimension::from_length(50.0) },
                max_size: Size { width: Dimension::from_length(10.0), height: Dimension::from_length(10.0) },
                ..Default::default()
            })
            .unwrap();

        taffy
            .compute_layout(
                child,
                Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(100.0) },
            )
            .unwrap();

        assert_eq!(taffy.layout(child).unwrap().size, Size { width: 10.0, height: 10.0 });
    }

    #[test]
    fn min_overrides_size() {
        let mut taffy = new_test_tree();

        let child = taffy
            .new_leaf(Style {
                size: Size { width: Dimension::from_length(50.0), height: Dimension::from_length(50.0) },
                min_size: Size { width: Dimension::from_length(100.0), height: Dimension::from_length(100.0) },
                ..Default::default()
            })
            .unwrap();

        taffy
            .compute_layout(
                child,
                Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(100.0) },
            )
            .unwrap();

        assert_eq!(taffy.layout(child).unwrap().size, Size { width: 100.0, height: 100.0 });
    }
}
