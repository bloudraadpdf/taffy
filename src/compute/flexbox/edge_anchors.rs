use super::{AlgoConstants, FlexItem, FlexLine};
use crate::geometry::Size;
use crate::style::AlignItemsKeyword;
use crate::tree::NodeId;
use crate::util::sys::Vec;

/// An item border edge constrained relative to its parent's border edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexItemEdgeAnchor {
    /// Physical start edge at this distance from the parent border start.
    Start(f32),
    /// Physical end edge at this distance inward from the parent border end.
    End(f32),
}

impl FlexItemEdgeAnchor {
    /// Resolve a constrained physical start coordinate without intermediate rounding.
    pub(super) fn position(self, parent_extent: f32, item_extent: f32) -> f32 {
        match self {
            Self::Start(inset) => inset,
            Self::End(inset) => (f64::from(parent_extent) - f64::from(inset) - f64::from(item_extent)) as f32,
        }
    }
}

/// Constraints retained from flex space distribution for one item.
#[derive(Debug, Clone, PartialEq)]
pub struct DetailedFlexItemInfo {
    /// The flex item's node.
    pub node: NodeId,
    /// Independent physical horizontal and vertical border-edge constraints.
    pub edge_anchors: Size<Option<FlexItemEdgeAnchor>>,
}

/// Resolve the relative inset in physical horizontal and vertical coordinates.
fn relative_shift(item: &FlexItem, constants: &AlgoConstants) -> Size<f32> {
    let direction = constants.dir;
    let rtl = constants.layout_direction.is_rtl();
    let main = if constants.is_row && rtl {
        -item.inset.main_end(direction).or(item.inset.main_start(direction).map(|value| -value)).unwrap_or(0.0)
    } else {
        item.inset.main_start(direction).or(item.inset.main_end(direction).map(|value| -value)).unwrap_or(0.0)
    };
    let cross = if constants.is_column && rtl {
        item.inset.cross_end(direction).map(|value| -value).or(item.inset.cross_start(direction)).unwrap_or(0.0)
    } else {
        item.inset.cross_start(direction).or(item.inset.cross_end(direction).map(|value| -value)).unwrap_or(0.0)
    };
    Size::ZERO.with_main(direction, main).with_cross(direction, cross)
}

/// Find the cross-axis distances authorised by the item's resolved alignment.
fn cross_edge_distances(item: &FlexItem, line: &FlexLine, constants: &AlgoConstants) -> (Option<f32>, Option<f32>) {
    let direction = constants.dir;
    let start = item.margin.cross_start(direction);
    let end = item.margin.cross_end(direction);
    if item.cross_fills_line || item.margin_is_auto.cross_start(direction) || item.margin_is_auto.cross_end(direction) {
        return (Some(start), Some(end));
    }
    let free = line.cross_size - item.outer_target_size.cross(direction);
    let rtl = constants.is_column && constants.layout_direction.is_rtl();
    let keyword =
        if item.align_self.is_safe() && free < 0.0 { AlignItemsKeyword::Start } else { item.align_self.keyword };
    let at_end = match keyword {
        AlignItemsKeyword::Start => rtl,
        AlignItemsKeyword::End => !rtl,
        AlignItemsKeyword::FlexStart | AlignItemsKeyword::Stretch => constants.is_wrap_reverse ^ rtl,
        AlignItemsKeyword::FlexEnd => !(constants.is_wrap_reverse ^ rtl),
        AlignItemsKeyword::Center => return (Some(free / 2.0 + start), Some(free / 2.0 + end)),
        AlignItemsKeyword::Baseline => return (Some(item.offset_cross + start), None),
        AlignItemsKeyword::SelfStart | AlignItemsKeyword::SelfEnd => unreachable!(),
    };
    if at_end {
        (None, Some(end))
    } else {
        (Some(start), None)
    }
}

/// Combine line and item alignment constraints into parent border-edge constraints.
fn collect(lines: &[FlexLine], constants: &AlgoConstants) -> Vec<DetailedFlexItemInfo> {
    let direction = constants.dir;
    let rtl = constants.is_row && constants.layout_direction.is_rtl();
    let mut result = Vec::new();
    for line in lines {
        for (index, item) in line.items.iter().enumerate() {
            let shift = relative_shift(item, constants);
            let mut edge_anchors = Size { width: None, height: None };
            let first = if direction.is_reverse() { index + 1 == line.items.len() } else { index == 0 };
            let last = if direction.is_reverse() { index == 0 } else { index + 1 == line.items.len() };
            if first {
                let edge = if rtl {
                    FlexItemEdgeAnchor::End(
                        constants.content_box_inset.main_end(direction)
                            + line.leading_main_space
                            + item.margin.main_end(direction)
                            - shift.main(direction),
                    )
                } else {
                    FlexItemEdgeAnchor::Start(
                        constants.content_box_inset.main_start(direction)
                            + line.leading_main_space
                            + item.margin.main_start(direction)
                            + shift.main(direction),
                    )
                };
                edge_anchors.set_main(direction, Some(edge));
            }
            if last {
                let edge = if rtl {
                    FlexItemEdgeAnchor::Start(
                        constants.content_box_inset.main_start(direction)
                            + line.trailing_main_space
                            + item.margin.main_start(direction)
                            + shift.main(direction),
                    )
                } else {
                    FlexItemEdgeAnchor::End(
                        constants.content_box_inset.main_end(direction)
                            + line.trailing_main_space
                            + item.margin.main_end(direction)
                            - shift.main(direction),
                    )
                };
                edge_anchors.set_main(direction, Some(edge));
            }
            let (start, end) = cross_edge_distances(item, line, constants);
            let cross = line
                .cross_end_inset
                .zip(end)
                .map(|(line, item)| FlexItemEdgeAnchor::End(line + item - shift.cross(direction)))
                .or_else(|| {
                    line.cross_start_inset
                        .zip(start)
                        .map(|(line, item)| FlexItemEdgeAnchor::Start(line + item + shift.cross(direction)))
                });
            edge_anchors.set_cross(direction, cross);
            if constants.container_size.width < constants.content_box_inset.horizontal_axis_sum() {
                edge_anchors.width = None;
            }
            if constants.container_size.height < constants.content_box_inset.vertical_axis_sum() {
                edge_anchors.height = None;
            }
            result.push(DetailedFlexItemInfo { node: item.node, edge_anchors });
        }
    }
    result
}

/// Retain the derived constraints for both native placement and detailed layout output.
pub(super) fn assign(lines: &mut [FlexLine], constants: &AlgoConstants) {
    let metadata = collect(lines, constants);
    for (item, info) in lines.iter_mut().flat_map(|line| line.items.iter_mut()).zip(metadata) {
        item.edge_anchors = info.edge_anchors;
    }
}
