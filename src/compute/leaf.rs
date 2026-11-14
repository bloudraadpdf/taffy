//! Computes size using styles and measure functions

use crate::geometry::{Point, Size};
use crate::style::{AvailableSpace, Position};
use crate::tree::{CollapsibleMarginSet, RunMode};
use crate::tree::{LayoutInput, LayoutOutput, SizingMode};
use crate::util::debug::debug_log;
use crate::util::sys::f32_max;
use crate::util::MaybeMath;
use crate::util::{MaybeResolve, ResolveOrZero};
use crate::{BoxSizing, CoreStyle};
use core::unreachable;

/// Compute the size of a leaf node (node with no children)
pub fn compute_leaf_layout<MeasureFunction>(
    inputs: LayoutInput,
    style: &impl CoreStyle,
    resolve_calc_value: impl Fn(*const (), f32) -> f32,
    measure_function: MeasureFunction,
) -> LayoutOutput
where
    MeasureFunction: FnOnce(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>,
{
    let LayoutInput {
        known_dimensions,
        parent_size,
        inline_percentage_basis,
        available_space,
        sizing_mode,
        run_mode,
        ..
    } = inputs;

    // Note: both horizontal and vertical percentage padding/borders are resolved against the container's inline size (i.e. width).
    // This is not a bug, but is how CSS is specified (see: https://developer.mozilla.org/en-US/docs/Web/CSS/padding#values)
    let edge_basis = inline_percentage_basis.resolve(parent_size);
    let margin = style.margin().resolve_or_zero(edge_basis, &resolve_calc_value);
    let padding = style.padding().resolve_or_zero(edge_basis, &resolve_calc_value);
    let border = style.border().resolve_or_zero(edge_basis, &resolve_calc_value);
    let padding_border = padding + border;
    let pb_sum = padding_border.sum_axes();
    let box_sizing_adjustment = if style.box_sizing() == BoxSizing::ContentBox { pb_sum } else { Size::ZERO };

    // Resolve node's preferred/min/max sizes (width/heights) against the available space (percentages resolve to pixel values)
    // For ContentSize mode, we pretend that the node has no size styles as these should be ignored.
    let (node_size, node_min_size, node_max_size, aspect_ratio) = match sizing_mode {
        SizingMode::ContentSize => {
            let node_size = known_dimensions;
            let node_min_size = Size::NONE;
            let node_max_size = Size::NONE;
            (node_size, node_min_size, node_max_size, None)
        }
        SizingMode::InherentSize => {
            let aspect_ratio = style.aspect_ratio();
            let style_size = style.size().maybe_resolve(parent_size, &resolve_calc_value);
            let style_min_size = style
                .min_size()
                .maybe_resolve(parent_size, &resolve_calc_value)
                .maybe_apply_aspect_ratio(aspect_ratio)
                .maybe_add(box_sizing_adjustment);
            let style_max_size =
                style.max_size().maybe_resolve(parent_size, &resolve_calc_value).maybe_add(box_sizing_adjustment);

            let preferred_size = if style.is_compressible_replaced() {
                known_dimensions.maybe_sub(box_sizing_adjustment).or(style_size)
            } else {
                style_size
            };
            let node_size = known_dimensions
                .or(preferred_size.maybe_apply_aspect_ratio(aspect_ratio).maybe_add(box_sizing_adjustment));
            (node_size, style_min_size, style_max_size, aspect_ratio)
        }
    };

    let content_box_inset = padding_border + crate::compute::scrollbar_gutters(&style);

    let has_styles_preventing_being_collapsed_through = !style.is_block()
        || style.overflow().x.is_scroll_container()
        || style.overflow().y.is_scroll_container()
        || style.position() == Position::Absolute
        || padding.top > 0.0
        || padding.bottom > 0.0
        || border.top > 0.0
        || border.bottom > 0.0
        || matches!(node_size.height, Some(h) if h > 0.0)
        || matches!(node_min_size.height, Some(h) if h > 0.0);

    debug_log!("LEAF");
    debug_log!("node_size", dbg:node_size);
    debug_log!("min_size ", dbg:node_min_size);
    debug_log!("max_size ", dbg:node_max_size);

    // Return early if both width and height are known
    if run_mode == RunMode::ComputeSize && has_styles_preventing_being_collapsed_through {
        if let Size { width: Some(width), height: Some(height) } = node_size {
            let size = Size { width, height }
                .maybe_clamp(node_min_size, node_max_size)
                .maybe_max(padding_border.sum_axes().map(Some));
            return LayoutOutput {
                size,
                #[cfg(feature = "content_size")]
                content_size: Size::ZERO,
                first_baselines: Point::NONE,
                top_margin: CollapsibleMarginSet::ZERO,
                bottom_margin: CollapsibleMarginSet::ZERO,
                margins_can_collapse_through: false,
            };
        };
    }

    // Compute available space
    let available_space = Size {
        width: known_dimensions
            .width
            .map(AvailableSpace::from)
            .unwrap_or(available_space.width)
            .maybe_sub(margin.horizontal_axis_sum())
            .maybe_set(known_dimensions.width)
            .maybe_set(node_size.width)
            .map_definite_value(|size| {
                size.maybe_clamp(node_min_size.width, node_max_size.width) - content_box_inset.horizontal_axis_sum()
            }),
        height: known_dimensions
            .height
            .map(AvailableSpace::from)
            .unwrap_or(available_space.height)
            .maybe_sub(margin.vertical_axis_sum())
            .maybe_set(known_dimensions.height)
            .maybe_set(node_size.height)
            .map_definite_value(|size| {
                size.maybe_clamp(node_min_size.height, node_max_size.height) - content_box_inset.vertical_axis_sum()
            }),
    };

    // Measure node
    let known_size = known_dimensions.or(node_size);
    let measured_size = if reads_measured_size(
        known_size,
        cfg!(feature = "content_size"),
        has_styles_preventing_being_collapsed_through,
    ) {
        measure_function(
            match run_mode {
                RunMode::ComputeSize => known_dimensions,
                RunMode::PerformLayout => Size::NONE,
                RunMode::PerformHiddenLayout => unreachable!(),
            },
            available_space,
        )
    } else {
        Size::ZERO
    };
    let clamped_size =
        known_size.unwrap_or(measured_size + content_box_inset.sum_axes()).maybe_clamp(node_min_size, node_max_size);
    let size = Size {
        width: clamped_size.width,
        height: f32_max(clamped_size.height, aspect_ratio.map(|ratio| clamped_size.width / ratio).unwrap_or(0.0)),
    };
    let size = size.maybe_max(padding_border.sum_axes().map(Some));

    LayoutOutput {
        size,
        #[cfg(feature = "content_size")]
        content_size: measured_size + padding.sum_axes(),
        first_baselines: Point::NONE,
        top_margin: CollapsibleMarginSet::ZERO,
        bottom_margin: CollapsibleMarginSet::ZERO,
        margins_can_collapse_through: !has_styles_preventing_being_collapsed_through
            && size.height == 0.0
            && measured_size.height == 0.0,
    }
}

/// Return true if the layout of a leaf reads the size that its measure function gives
///
/// The measured size gives each dimension that is not known, the content size, and the collapse-through state of a
/// node whose styles let margins collapse through it.
fn reads_measured_size(
    known_size: Size<Option<f32>>,
    content_size_is_output: bool,
    styles_prevent_collapse_through: bool,
) -> bool {
    content_size_is_output
        || !styles_prevent_collapse_through
        || known_size.width.is_none()
        || known_size.height.is_none()
}

#[cfg(test)]
mod tests {
    use super::reads_measured_size;
    use crate::geometry::Size;

    #[test]
    fn a_known_size_needs_a_measurement_only_for_the_content_size_or_the_collapse_state() {
        let known = Size { width: Some(1.0), height: Some(1.0) };

        assert!(!reads_measured_size(known, false, true));
        assert!(reads_measured_size(known, true, true));
        assert!(reads_measured_size(known, false, false));
        assert!(reads_measured_size(Size { width: None, ..known }, false, true));
        assert!(reads_measured_size(Size { height: None, ..known }, false, true));
    }
}
