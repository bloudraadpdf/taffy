#[cfg(feature = "grid")]
mod tests {
    use taffy::prelude::*;

    /// The track sizing distributes an item's contributions to the tracks it spans so that their sizes sum to the
    /// contribution (CSS Grid 2 §12.5.1): the 79 auto tracks of an item with a 45 min-content and a 90 max-content
    /// contribution sum to 90, without pixel rounding.
    #[test]
    fn the_tracks_of_a_spanning_item_sum_to_its_contribution() {
        let mut tree = TaffyTree::<()>::new();
        tree.disable_rounding();
        let item =
            tree.new_leaf(Style { grid_column: Line { start: line(1), end: span(79) }, ..Default::default() }).unwrap();
        let root = tree.new_with_children(Style { display: Display::Grid, ..Default::default() }, &[item]).unwrap();
        tree.compute_layout_with_measure(root, Size::MAX_CONTENT, |known, available, _, _, _| {
            let width = known.width.unwrap_or(match available.width {
                AvailableSpace::MinContent => 45.0,
                AvailableSpace::MaxContent => 90.0,
                AvailableSpace::Definite(width) => width.clamp(45.0, 90.0),
            });
            Size { width, height: 10.0 }
        })
        .unwrap();
        assert_eq!(tree.layout(item).unwrap().size.width, 90.0);
    }

    /// A space below the distribution threshold is still distributed: under a min-content constraint the tracks of
    /// an item that needs 0.005 more than the tracks it spans grow by it.
    #[test]
    fn a_small_extra_space_is_distributed() {
        let mut tree = TaffyTree::<()>::new();
        tree.disable_rounding();
        let leaf = |tree: &mut TaffyTree<()>, width: f32, column: Line<GridPlacement>| {
            let size = Size { width: length(width), height: length(10.0) };
            tree.new_leaf(Style { size, grid_column: column, ..Default::default() }).unwrap()
        };
        let first = leaf(&mut tree, 45.0, Line { start: line(1), end: GridPlacement::Auto });
        let spanning = leaf(&mut tree, 45.005, Line { start: line(1), end: span(2) });
        let root =
            tree.new_with_children(Style { display: Display::Grid, ..Default::default() }, &[first, spanning]).unwrap();
        tree.compute_layout(root, Size::MIN_CONTENT).unwrap();
        assert_eq!(tree.layout(root).unwrap().size.width, 45.005);
    }
}
