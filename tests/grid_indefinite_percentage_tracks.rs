#[cfg(feature = "grid")]
mod tests {
    use taffy::prelude::*;

    /// A percentage min track sizing function of an indefinite grid size behaves as `auto` (CSS Grid 2 §7.2.1): the
    /// base size takes the item's minimum contribution, whose automatic minimum is zero because the specified min
    /// is not `auto` (§6.6), so the row grows to its 15px max and not to the 20px min-content contribution.
    #[test]
    fn an_indefinite_percentage_min_track_sizes_as_auto() {
        let mut tree = TaffyTree::<()>::new();
        let content = tree
            .new_leaf(Style { size: Size { width: length(10.0), height: length(20.0) }, ..Default::default() })
            .unwrap();
        let item = tree.new_with_children(Style::default(), &[content]).unwrap();
        let root = tree
            .new_with_children(
                Style {
                    display: Display::Grid,
                    grid_template_columns: vec![length(30.0)],
                    grid_template_rows: vec![minmax(percent(1.0), length(15.0))],
                    ..Default::default()
                },
                &[item],
            )
            .unwrap();
        tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
        assert_eq!(tree.layout(root).unwrap().size.height, 15.0);
    }
}
