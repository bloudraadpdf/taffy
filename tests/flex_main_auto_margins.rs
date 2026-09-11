use taffy::prelude::*;

#[test]
fn main_auto_margins_consume_space_before_justification() {
    for direction in
        [FlexDirection::Row, FlexDirection::RowReverse, FlexDirection::Column, FlexDirection::ColumnReverse]
    {
        for justify in [
            JustifyContent::CENTER,
            JustifyContent::END,
            JustifyContent::SPACE_AROUND,
            JustifyContent::SPACE_BETWEEN,
            JustifyContent::SPACE_EVENLY,
        ] {
            for (start, end, position) in [(true, false, 80.0), (false, true, 0.0), (true, true, 40.0)] {
                let mut tree: TaffyTree<()> = TaffyTree::new();
                let start_margin = if start { auto() } else { zero() };
                let end_margin = if end { auto() } else { zero() };
                let row = matches!(direction, FlexDirection::Row | FlexDirection::RowReverse);
                let margin = if row {
                    Rect { left: start_margin, right: end_margin, ..Rect::zero() }
                } else {
                    Rect { top: start_margin, bottom: end_margin, ..Rect::zero() }
                };
                let item = tree
                    .new_leaf(Style {
                        size: Size { width: length(20.0), height: length(20.0) },
                        flex_shrink: 0.0,
                        margin,
                        ..Style::default()
                    })
                    .unwrap();
                let parent = tree
                    .new_with_children(
                        Style {
                            display: Display::Flex,
                            flex_direction: direction,
                            justify_content: Some(justify),
                            size: Size { width: length(100.0), height: length(100.0) },
                            ..Style::default()
                        },
                        &[item],
                    )
                    .unwrap();
                tree.compute_layout(parent, Size::MAX_CONTENT).unwrap();
                let layout = tree.layout(item).unwrap();
                let actual = if row { layout.location.x } else { layout.location.y };
                assert_eq!(actual, position, "{direction:?}/{justify:?}/{start}/{end}");
            }
        }
    }
}

#[test]
fn nonpositive_space_keeps_auto_margins_zero_and_justifies_overflow() {
    for extent in [20.0, 10.0] {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        let item = tree
            .new_leaf(Style {
                size: Size { width: length(20.0), height: length(20.0) },
                flex_shrink: 0.0,
                margin: Rect { left: auto(), right: auto(), ..Rect::zero() },
                ..Style::default()
            })
            .unwrap();
        let parent = tree
            .new_with_children(
                Style {
                    display: Display::Flex,
                    justify_content: Some(JustifyContent::CENTER),
                    size: Size { width: length(extent), height: length(100.0) },
                    ..Style::default()
                },
                &[item],
            )
            .unwrap();
        tree.compute_layout(parent, Size::MAX_CONTENT).unwrap();
        let layout = tree.layout(item).unwrap();
        assert_eq!(layout.location.x, (extent - 20.0) / 2.0);
        assert_eq!(layout.margin.left, 0.0);
        assert_eq!(layout.margin.right, 0.0);
    }
}
