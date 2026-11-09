#![cfg(all(feature = "flexbox", feature = "grid", feature = "block_layout"))]

use taffy::geometry::Point;
use taffy::prelude::*;
use taffy::style::{Overflow, ResolvedScrollbarGutters};

#[test]
fn resolved_gutters_preserve_independent_axes_and_both_edges() {
    for display in [Display::Flex, Display::Grid, Display::Block] {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        let child = tree
            .new_leaf(Style { size: Size { width: length(20.0), height: length(10.0) }, ..Default::default() })
            .unwrap();
        let gutters = ResolvedScrollbarGutters::new(Rect { left: 8.0, right: 8.0, top: 3.0, bottom: 9.0 }).unwrap();
        let root = tree
            .new_with_children(
                Style {
                    display,
                    size: Size { width: length(100.0), height: length(80.0) },
                    resolved_scrollbar_gutters: Some(gutters),
                    scrollbar_width: 99.0,
                    overflow: Point { x: Overflow::Scroll, y: Overflow::Scroll },
                    ..Default::default()
                },
                &[child],
            )
            .unwrap();
        tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
        assert_eq!(tree.layout(root).unwrap().scrollbar_size, Size { width: 16.0, height: 12.0 });
        assert_eq!(tree.layout(child).unwrap().location, Point { x: 8.0, y: 3.0 }, "{display:?}");
        assert_eq!(tree.layout(child).unwrap().size, Size { width: 20.0, height: 10.0 });
    }
}

#[test]
fn leaf_gutters_adjust_measurement_once_and_keep_scalar_defaults() {
    for custom in [false, true] {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        let root = tree
            .new_leaf_with_context(
                Style {
                    size: Size { width: length(100.0), height: auto() },
                    overflow: Point { x: Overflow::Scroll, y: Overflow::Scroll },
                    scrollbar_width: 15.0,
                    resolved_scrollbar_gutters: custom.then(|| {
                        ResolvedScrollbarGutters::new(Rect { left: 8.0, right: 8.0, top: 0.0, bottom: 9.0 }).unwrap()
                    }),
                    ..Default::default()
                },
                (),
            )
            .unwrap();
        let expected = if custom { Size { width: 16.0, height: 9.0 } } else { Size { width: 15.0, height: 15.0 } };
        tree.compute_layout_with_measure(root, Size::MAX_CONTENT, |_, available, _, _, _| {
            assert_eq!(available.width, AvailableSpace::Definite(100.0 - expected.width));
            Size { width: 100.0 - expected.width, height: 20.0 }
        })
        .unwrap();
        assert_eq!(tree.layout(root).unwrap().size, Size { width: 100.0, height: 20.0 + expected.height });
        assert_eq!(tree.layout(root).unwrap().scrollbar_size, expected);
    }
}

#[test]
#[cfg(feature = "serde")]
fn resolved_gutter_deserialization_preserves_validation_and_legacy_defaults() {
    let legacy: Style = serde_json::from_str("{}").unwrap();
    assert_eq!(legacy.resolved_scrollbar_gutters, None);
    assert!(serde_json::from_str::<ResolvedScrollbarGutters>(r#"{"left":-1,"right":0,"top":0,"bottom":0}"#).is_err());
    let gutters = ResolvedScrollbarGutters::new(Rect { left: 8.0, right: 8.0, top: 0.0, bottom: 9.0 }).unwrap();
    assert_eq!(
        serde_json::from_str::<ResolvedScrollbarGutters>(&serde_json::to_string(&gutters).unwrap()).unwrap(),
        gutters
    );
}
