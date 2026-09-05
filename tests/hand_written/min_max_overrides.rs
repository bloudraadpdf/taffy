#[cfg(test)]
mod min_max_overrides {
    use taffy::prelude::*;
    use taffy_test_helpers::new_test_tree;

    #[test]
    #[cfg(feature = "calc")]
    fn fit_content_calculation_keeps_its_limit_and_intrinsic_minimum() {
        static LIMIT: f64 = 0.0;
        let handle = core::ptr::addr_of!(LIMIT).cast();
        for (width, minimum, expected) in [(200.0, 10.0, 70.0), (100.0, 10.0, 45.0), (100.0, 80.0, 80.0)] {
            let mut tree: TaffyTree<()> = TaffyTree::new();
            let child = tree.new_leaf(Style::default()).unwrap();
            let grid = tree
                .new_with_children(
                    Style {
                        display: Display::Grid,
                        size: Size { width: length(width), height: auto() },
                        grid_template_columns: vec![GridTemplateComponent::Single(fit_content(
                            LengthPercentage::calc(handle),
                        ))],
                        ..Default::default()
                    },
                    &[child],
                )
                .unwrap();
            tree.compute_layout_with_measure_and_calc(
                grid,
                Size::MAX_CONTENT,
                |input, _, _, _| Size {
                    width: input.known_dimensions.width.unwrap_or(
                        if input.available_space.width == AvailableSpace::MinContent { minimum } else { 300.0 },
                    ),
                    height: input.known_dimensions.height.unwrap_or(10.0),
                },
                |actual, basis| {
                    assert_eq!(actual, handle);
                    20.0 + basis * 0.25
                },
            )
            .unwrap();
            assert_eq!(tree.layout(child).unwrap().size.width, expected);
        }
    }

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
