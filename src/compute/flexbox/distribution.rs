use crate::compute::common::alignment::compute_alignment_offset;
use crate::style::AlignContentKeyword;

/// Allocate shares from the space that remains after each rounded result.
pub(super) struct RemainingSpace {
    /// Space not consumed by previously rounded shares.
    space: f64,
    /// Sum of the original weights still to allocate.
    weight: f64,
}

impl RemainingSpace {
    /// Begin an allocation with the exact sum of its input weights.
    pub(super) fn new(space: f32, weight: f64) -> Self {
        Self { space: f64::from(space), weight }
    }

    /// Remove one weighted share from the remaining budget.
    pub(super) fn take(&mut self, weight: f32) -> f32 {
        self.add_to(0.0, weight)
    }

    /// Add a share to a base, accounting for the rounded result actually used.
    pub(super) fn add_to(&mut self, base: f32, weight: f32) -> f32 {
        if weight <= 0.0 || self.weight <= 0.0 {
            return base;
        }
        let weight = f64::from(weight).min(self.weight);
        let share = if weight == self.weight { self.space } else { self.space * weight / self.weight };
        let result = (f64::from(base) + share) as f32;
        self.space -= f64::from(result) - f64::from(base);
        self.weight -= weight;
        result
    }

    /// Return the space left after the consumed shares.
    pub(super) fn remaining(&self) -> f32 {
        self.space as f32
    }
}

/// Leading, intervening and trailing space from one alignment budget.
pub(super) struct AlignmentDistribution {
    /// Space and weight still available to intervening and trailing positions.
    remaining: RemainingSpace,
    /// Allocated space before the first item.
    leading: f32,
    /// Weight of each position between adjacent items.
    between_weight: f32,
}

impl AlignmentDistribution {
    /// Partition the free space according to the resolved alignment mode.
    pub(super) fn new(space: f32, count: usize, mode: AlignContentKeyword, reversed: bool) -> Self {
        let (weight, leading_weight, between_weight) = match mode {
            AlignContentKeyword::SpaceBetween => ((count.saturating_sub(1)) as f64, 0.0, 1.0),
            AlignContentKeyword::SpaceAround => (2.0 * count as f64, 1.0, 2.0),
            AlignContentKeyword::SpaceEvenly => ((count + 1) as f64, 1.0, 1.0),
            _ => (0.0, 0.0, 0.0),
        };
        let mut remaining = RemainingSpace::new(space, weight);
        let leading = if weight > 0.0 {
            remaining.take(leading_weight)
        } else {
            let leading = compute_alignment_offset(space, count, 0.0, mode, reversed, true);
            remaining.space -= f64::from(leading);
            leading
        };
        Self { remaining, leading, between_weight }
    }

    /// Return the allocated leading space.
    pub(super) fn leading(&self) -> f32 {
        self.leading
    }

    /// Allocate the space before the next item, including its authored gap.
    pub(super) fn next(&mut self, first: bool, gap: f32) -> f32 {
        if first {
            self.leading
        } else {
            gap + self.remaining.take(self.between_weight)
        }
    }

    /// Return the space retained after the final item.
    pub(super) fn trailing(&self) -> f32 {
        if self.between_weight > 0.0 && self.remaining.weight == 0.0 {
            0.0
        } else {
            self.remaining.remaining()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RemainingSpace;

    #[test]
    fn fractional_weights_consume_the_exact_original_weight_sum() {
        let weights = [0.1_f32, 0.2, 0.3, 0.7];
        let mut remaining = RemainingSpace::new(47.25, weights.into_iter().map(f64::from).sum());
        let shares = weights.map(|weight| remaining.take(weight));
        assert_eq!(remaining.weight, 0.0);
        assert!((shares.into_iter().map(f64::from).sum::<f64>() - 47.25).abs() < f64::from(f32::EPSILON) * 47.25);
    }
}
