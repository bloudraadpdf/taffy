#[cfg(feature = "grid")]
mod tests {
    use taffy::prelude::*;

    /// Step 2 of the sparse placement places an item locked to a row past the items this step placed earlier with
    /// the same row start (CSS Grid 1 §8.5): an item that spans rows 1 and 2 does not move the cursor of row 2.
    #[test]
    fn a_row_locked_item_is_placed_past_the_items_of_its_start_row() {
        let mut tree = TaffyTree::<()>::new();
        let mut item = |grid_row: Line<GridPlacement>, column: GridPlacement| {
            let grid_column = Line { start: column, end: GridPlacement::Auto };
            tree.new_leaf(Style { grid_row, grid_column, ..Default::default() }).unwrap()
        };
        let row = |start| Line { start: line(start), end: GridPlacement::Auto };
        let items = [
            item(row(1), line(2)),
            item(row(2), line(1)),
            item(Line { start: line(1), end: span(2) }, GridPlacement::Auto),
            item(row(1), GridPlacement::Auto),
            item(row(2), GridPlacement::Auto),
        ];
        let root = tree
            .new_with_children(
                Style {
                    display: Display::Grid,
                    grid_auto_columns: vec![length(50.0)],
                    grid_auto_rows: vec![length(50.0)],
                    ..Default::default()
                },
                &items,
            )
            .unwrap();
        tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
        let origins = items.map(|node| {
            let location = tree.layout(node).unwrap().location;
            (location.x, location.y)
        });
        assert_eq!(origins, [(50.0, 0.0), (0.0, 50.0), (100.0, 0.0), (150.0, 0.0), (50.0, 50.0)]);
    }
}
