#![cfg(feature = "flexbox")]

use taffy::prelude::*;
use taffy::{Overflow, Point};

fn minimum_content_measurements(direction: FlexDirection, item: Style) -> usize {
    let row = matches!(direction, FlexDirection::Row | FlexDirection::RowReverse);
    let mut tree: TaffyTree<()> = TaffyTree::new();
    let item = tree.new_leaf(item).unwrap();
    let root = tree
        .new_with_children(
            Style {
                flex_direction: direction,
                size: Size { width: length(100.0), height: length(100.0) },
                ..Default::default()
            },
            &[item],
        )
        .unwrap();
    let mut measurements = 0;
    tree.compute_layout_with_measure_and_calc(
        root,
        Size::MAX_CONTENT,
        |input, _, _, _| {
            let main = if row { input.available_space.width } else { input.available_space.height };
            measurements += usize::from(main == AvailableSpace::MinContent);
            input.known_dimensions.unwrap_or(Size { width: 300.0, height: 300.0 })
        },
        |_, _| unreachable!("this fixture has no calc values"),
    )
    .unwrap();
    measurements
}

#[test]
fn only_an_automatic_minimum_measures_the_minimum_content_size() {
    for direction in
        [FlexDirection::Row, FlexDirection::RowReverse, FlexDirection::Column, FlexDirection::ColumnReverse]
    {
        let zero_minimum = Style { min_size: Size { width: length(0.0), height: length(0.0) }, ..Default::default() };
        let scroll_container =
            Style { overflow: Point { x: Overflow::Hidden, y: Overflow::Hidden }, ..Default::default() };

        assert_eq!(minimum_content_measurements(direction, Style::default()), 1, "{direction:?}");
        assert_eq!(minimum_content_measurements(direction, zero_minimum), 0, "{direction:?}");
        assert_eq!(minimum_content_measurements(direction, scroll_container), 0, "{direction:?}");
    }
}
