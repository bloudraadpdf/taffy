//! Utility functions used within the grid module

#[cfg(test)]
pub(super) mod test_helpers;
#[cfg(test)]
pub(super) use test_helpers::{CreateChildTestNode, CreateExpectedPlacement, CreateParentTestNode};

/// The sum of track sizes, accumulated in `f64` and rounded once, so it does not lose the precision of many additions
pub(in crate::compute::grid) fn sum_f64(sizes: impl Iterator<Item = f32>) -> f32 {
    sizes.map(f64::from).sum::<f64>() as f32
}

/// The sum of track sizes that may be indefinite, accumulated as [`sum_f64`]
pub(in crate::compute::grid) fn sum_f64_option(sizes: impl Iterator<Item = Option<f32>>) -> Option<f32> {
    sizes.map(|size| size.map(f64::from)).sum::<Option<f64>>().map(|sum| sum as f32)
}
