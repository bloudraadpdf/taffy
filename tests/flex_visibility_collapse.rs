#![cfg(feature = "flexbox")]

use taffy::geometry::Point;
use taffy::prelude::*;
use taffy::FlexItemVisibility;

#[test]
fn fit_content_cross_size_uses_the_final_line_without_changing_the_main_size() {
    let mut tree: TaffyTree<()> = TaffyTree::new();
    let item = tree
        .new_leaf(Style {
            size: Size { width: auto(), height: length(200.0) },
            margin: taffy::geometry::Rect { left: length(5.0), right: length(5.0), top: zero(), bottom: zero() },
            flex_cross_size: taffy::FlexCrossSize::FitContent,
            align_self: Some(AlignSelf::FLEX_START),
            ..Style::default()
        })
        .unwrap();
    let wide =
        tree.new_leaf(Style { size: Size { width: length(200.0), height: length(0.0) }, ..Style::default() }).unwrap();
    let root = tree
        .new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                flex_wrap: FlexWrap::Wrap,
                size: Size { width: length(100.0), height: auto() },
                ..Style::default()
            },
            &[item, wide],
        )
        .unwrap();
    tree.compute_layout_with_measure(root, Size::MAX_CONTENT, |known, available, _, _, _| Size {
        width: known.width.unwrap_or(match available.width {
            AvailableSpace::Definite(width) => width.clamp(100.0, 200.0),
            AvailableSpace::MinContent => 100.0,
            AvailableSpace::MaxContent => 200.0,
        }),
        height: known.height.unwrap_or(200.0),
    })
    .unwrap();
    assert_eq!(tree.layout(item).unwrap().size, Size { width: 190.0, height: 200.0 });
}

#[test]
fn embedding_baseline_strut_overrides_size_only_measurement() {
    let mut tree: TaffyTree<()> = TaffyTree::new();
    let child = tree
        .new_leaf(Style {
            size: Size { width: length(20.0), height: length(40.0) },
            flex_visibility: FlexItemVisibility::CollapseWithStrut(60.0),
            ..Style::default()
        })
        .unwrap();
    let visible =
        tree.new_leaf(Style { size: Size { width: length(20.0), height: length(20.0) }, ..Style::default() }).unwrap();
    let root = tree.new_with_children(Style::default(), &[child, visible]).unwrap();
    tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
    assert_eq!(tree.layout(root).unwrap().size, Size { width: 20.0, height: 60.0 });
}

#[test]
fn collapsed_items_leave_struts_after_line_collection() {
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        let dimensions = |main, cross| match direction {
            FlexDirection::Row => Size { width: length(main), height: length(cross) },
            _ => Size { width: length(cross), height: length(main) },
        };
        let collapsed = tree
            .new_leaf(Style {
                size: dimensions(30.0, 40.0),
                flex_visibility: FlexItemVisibility::Collapse,
                ..Style::default()
            })
            .unwrap();
        let first = tree.new_leaf(Style { size: dimensions(30.0, 20.0), ..Style::default() }).unwrap();
        let second = tree.new_leaf(Style { size: dimensions(30.0, 20.0), ..Style::default() }).unwrap();
        let root = tree
            .new_with_children(
                Style {
                    flex_direction: direction,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: Some(AlignItems::FLEX_START),
                    size: if direction == FlexDirection::Row {
                        Size { width: length(60.0), height: auto() }
                    } else {
                        Size { width: auto(), height: length(60.0) }
                    },
                    ..Style::default()
                },
                &[collapsed, first, second],
            )
            .unwrap();
        tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
        let root = tree.layout(root).unwrap();
        let first = tree.layout(first).unwrap();
        let second = tree.layout(second).unwrap();
        if direction == FlexDirection::Row {
            assert_eq!(root.size, Size { width: 60.0, height: 40.0 });
            assert_eq!(second.location, Point { x: 30.0, y: 0.0 });
        } else {
            assert_eq!(root.size, Size { width: 40.0, height: 60.0 });
            assert_eq!(second.location, Point { x: 0.0, y: 30.0 });
        }
        assert_eq!(first.location, Point::ZERO);
    }
}

#[test]
fn collapsed_struts_move_with_zero_main_size_and_keep_original_line_height() {
    let mut tree: TaffyTree<()> = TaffyTree::new();
    let first =
        tree.new_leaf(Style { size: Size { width: length(30.0), height: length(20.0) }, ..Style::default() }).unwrap();
    let collapsed = tree
        .new_leaf(Style {
            size: Size { width: length(15.0), height: length(25.0) },
            flex_visibility: FlexItemVisibility::Collapse,
            ..Style::default()
        })
        .unwrap();
    let tall =
        tree.new_leaf(Style { size: Size { width: length(15.0), height: length(40.0) }, ..Style::default() }).unwrap();
    let root = tree
        .new_with_children(
            Style {
                flex_wrap: FlexWrap::Wrap,
                align_items: Some(AlignItems::FLEX_START),
                size: Size { width: length(30.0), height: auto() },
                ..Style::default()
            },
            &[first, collapsed, tall],
        )
        .unwrap();
    tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
    assert_eq!(tree.layout(tall).unwrap().location.y, 40.0);
    assert_eq!(tree.layout(root).unwrap().size.height, 80.0);
    #[cfg(feature = "detailed_layout_info")]
    {
        let taffy::tree::DetailedLayoutInfo::Flex(info) = tree.detailed_layout_info(root) else {
            panic!("flex metadata");
        };
        assert_eq!(info.collapse_struts, vec![(collapsed, 40.0)]);
        let strut = info.collapse_struts[0].1;
        let mut root_style = tree.style(root).unwrap().clone();
        root_style.size.height = length(80.0);
        root_style.min_size.height = length(80.0);
        root_style.max_size.height = length(80.0);
        tree.set_style(root, root_style).unwrap();
        let mut collapsed_style = tree.style(collapsed).unwrap().clone();
        collapsed_style.flex_visibility = FlexItemVisibility::CollapseWithStrut(strut);
        tree.set_style(collapsed, collapsed_style).unwrap();
        tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
        assert_eq!(tree.layout(tall).unwrap().location.y, 40.0);
        assert_eq!(tree.layout(root).unwrap().size.height, 80.0);
    }
}

#[test]
fn collapsed_items_do_not_leave_gaps_or_intrinsic_main_space() {
    for collapsed_index in 0..3 {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        let items: Vec<_> = (0..3)
            .map(|index| {
                tree.new_leaf(Style {
                    size: Size { width: length(50.0), height: length(50.0) },
                    flex_visibility: if index == collapsed_index {
                        FlexItemVisibility::Collapse
                    } else {
                        FlexItemVisibility::Visible
                    },
                    ..Style::default()
                })
                .unwrap()
            })
            .collect();
        let root = tree
            .new_with_children(
                Style { gap: Size { width: length(50.0), height: length(50.0) }, ..Style::default() },
                &items,
            )
            .unwrap();
        tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
        assert_eq!(tree.layout(root).unwrap().size, Size { width: 150.0, height: 50.0 });
        let visible: Vec<_> =
            items.iter().enumerate().filter(|(index, _)| *index != collapsed_index).map(|(_, id)| *id).collect();
        assert_eq!(tree.layout(visible[0]).unwrap().location.x, 0.0);
        assert_eq!(tree.layout(visible[1]).unwrap().location.x, 100.0);
    }
}
