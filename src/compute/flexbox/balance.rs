use core::num::NonZeroU32;
use core::ops::Range;

use crate::util::sys::Vec;

/// Squared cost of one line plus the optimal remaining suffix.
#[derive(Clone, Copy)]
struct LineCost {
    /// Exclusive item index at this line break.
    end: usize,
    /// Prefix length at the break, excluding the trailing gap.
    center: f64,
    /// Optimal cost of the remaining lines.
    suffix: f64,
}

impl LineCost {
    /// Evaluate the quadratic at a candidate start coordinate.
    fn at(self, start: f64) -> f64 {
        let length = self.center - start;
        length * length + self.suffix
    }

    /// Prefer lower error, then more items on this line.
    fn precedes(self, other: Self, start: f64) -> bool {
        let cost = self.at(start);
        let other_cost = other.at(start);
        cost < other_cost || (cost == other_cost && self.end > other.end)
    }
}

/// Range-restricted Li Chao tree over sorted prefix coordinates.
struct IntervalCosts<'a> {
    /// Prefix coordinate for each possible line start.
    starts: &'a [f64],
    /// Best quadratic stored at each segment-tree node.
    nodes: Vec<Option<LineCost>>,
}

impl<'a> IntervalCosts<'a> {
    /// Allocate an empty tree for these start coordinates.
    fn new(starts: &'a [f64]) -> Self {
        Self { starts, nodes: (0..4 * starts.len()).map(|_| None).collect() }
    }

    /// Add a cost only where the line satisfies capacity and zero-item rules.
    fn insert(&mut self, range: Range<usize>, cost: LineCost) {
        if range.is_empty() || !cost.suffix.is_finite() {
            return;
        }
        self.insert_interval(1, 0..self.starts.len(), &range, cost);
    }

    /// Decompose a valid start interval into covered tree segments.
    fn insert_interval(&mut self, node: usize, span: Range<usize>, range: &Range<usize>, cost: LineCost) {
        if range.start <= span.start && span.end <= range.end {
            self.insert_cost(node, span, cost);
            return;
        }
        let middle = span.start + (span.end - span.start) / 2;
        if range.start < middle {
            self.insert_interval(2 * node, span.start..middle, range, cost);
        }
        if range.end > middle {
            self.insert_interval(2 * node + 1, middle..span.end, range, cost);
        }
    }

    /// Retain the midpoint winner and recurse where the other cost can win.
    fn insert_cost(&mut self, node: usize, span: Range<usize>, mut cost: LineCost) {
        let Some(mut current) = self.nodes[node] else {
            self.nodes[node] = Some(cost);
            return;
        };
        let middle = span.start + (span.end - span.start) / 2;
        let better_left = cost.precedes(current, self.starts[span.start]);
        let better_middle = cost.precedes(current, self.starts[middle]);
        if better_middle {
            core::mem::swap(&mut cost, &mut current);
            self.nodes[node] = Some(current);
        }
        if span.end - span.start == 1 {
            return;
        }
        if better_left != better_middle {
            self.insert_cost(2 * node, span.start..middle, cost);
        } else {
            self.insert_cost(2 * node + 1, middle..span.end, cost);
        }
    }

    /// Find the minimum cost on the path to this start coordinate.
    fn best(&self, start: usize) -> Option<LineCost> {
        let mut node = 1;
        let mut span = 0..self.starts.len();
        let mut best: Option<LineCost> = None;
        loop {
            if let Some(cost) = self.nodes[node] {
                if best.map_or(true, |current| cost.precedes(current, self.starts[start])) {
                    best = Some(cost);
                }
            }
            if span.end - span.start == 1 {
                return best;
            }
            let middle = span.start + (span.end - span.start) / 2;
            if start < middle {
                span.end = middle;
                node *= 2;
            } else {
                span.start = middle;
                node = 2 * node + 1;
            }
        }
    }
}

/// Nonnegative line-collection sizes and their prefix sums.
struct BalanceInput {
    /// Outer hypothetical main sizes, floored at zero.
    sizes: Vec<f64>,
    /// Cumulative item sizes including one gap per item.
    prefix: Vec<f64>,
    /// Main-axis gap between adjacent items.
    gap: f64,
    /// Finite inner main size, or an unbounded main constraint.
    limit: Option<f64>,
}

impl BalanceInput {
    /// Normalise collection inputs without changing flexible-length inputs.
    fn new(sizes: &[f32], gap: f32, limit: Option<f32>) -> Self {
        let sizes: Vec<_> = sizes.iter().map(|&size| f64::from(size.max(0.0))).collect();
        let gap = f64::from(gap.max(0.0));
        let mut prefix = Vec::new();
        prefix.push(0.0);
        for size in &sizes {
            prefix.push(prefix.last().copied().unwrap() + size + gap);
        }
        Self {
            sizes,
            prefix,
            gap,
            limit: limit.filter(|value| value.is_finite()).map(|value| f64::from(value.max(0.0))),
        }
    }

    /// Measure a nonempty line, including only its internal gaps.
    fn length(&self, start: usize, end: usize) -> f64 {
        self.prefix[end] - self.prefix[start] - self.gap
    }

    /// Whether a following zero item could instead join this line.
    fn forces_singleton(&self, start: usize, end: usize) -> bool {
        self.sizes.get(end) == Some(&0.0)
            && self.limit.map_or(true, |limit| self.length(start, end) + self.gap <= limit)
    }

    /// Count the fewest capacity-constrained lines.
    fn greedy_line_count(&self) -> usize {
        let Some(limit) = self.limit else { return 1 };
        let mut lines = 1;
        let mut start = 0;
        for end in 1..=self.sizes.len() {
            if end > start + 1 && self.length(start, end) > limit {
                lines += 1;
                start = end - 1;
            }
        }
        lines
    }

    /// Insert the normal and zero-constrained suffix costs in their valid ranges.
    fn insert_suffix(&self, costs: &mut IntervalCosts<'_>, end: usize, normal: f64, singleton: f64) {
        let center = self.prefix[end] - self.gap;
        let upper = end.min(costs.starts.len());
        let lower =
            self.limit.map_or(0, |limit| costs.starts.partition_point(|&start| start < center - limit)).min(upper);
        if self.sizes.get(end) == Some(&0.0) {
            let split = self
                .limit
                .map_or(lower, |limit| costs.starts.partition_point(|&start| start < self.prefix[end] - limit))
                .clamp(lower, upper);
            costs.insert(lower..split, LineCost { end, center, suffix: normal });
            costs.insert(split..upper, LineCost { end, center, suffix: singleton });
        } else {
            costs.insert(lower..upper, LineCost { end, center, suffix: normal });
        }
        if end <= costs.starts.len() && self.limit.is_some_and(|limit| self.sizes[end - 1] > limit) {
            costs.insert(end - 1..end, LineCost { end, center, suffix: normal });
        }
    }
}

/// Return exclusive item indices at the ends of balanced flex lines.
///
/// Item sizes are outer hypothetical main sizes; their zero floor applies only
/// to line assignment. An absent main limit denotes unbounded available space.
/// The solver minimises squared line error and resolves ties towards earlier
/// lines. Each line-count layer takes O(n log² n), with O(n) working storage;
/// reconstruction retains O(n k) break indices for k lines.
pub fn balanced_flex_line_ends(
    item_sizes: &[f32],
    main_gap: f32,
    main_limit: Option<f32>,
    minimum_line_count: NonZeroU32,
) -> Vec<usize> {
    let count = item_sizes.len();
    if count == 0 {
        return Vec::new();
    }
    let input = BalanceInput::new(item_sizes, main_gap, main_limit);
    let line_count = input.greedy_line_count().max(minimum_line_count.get() as usize).min(count);
    if line_count == count {
        return (1..=count).collect();
    }
    if line_count == 1 {
        return core::iter::once(count).collect();
    }
    let mut previous: Vec<f64> = (0..=count).map(|_| f64::INFINITY).collect();
    let mut previous_singleton = previous.clone();
    previous[count] = 0.0;
    previous_singleton[count] = 0.0;
    let mut breaks: Vec<Vec<usize>> = Vec::new();
    for lines in 1..=line_count {
        let starts = count - lines + 1;
        let mut costs = IntervalCosts::new(&input.prefix[..starts]);
        for end in 1..=count {
            input.insert_suffix(&mut costs, end, previous[end], previous_singleton[end]);
        }
        let mut current: Vec<f64> = (0..=count).map(|_| f64::INFINITY).collect();
        let mut current_singleton = current.clone();
        let mut next_break: Vec<usize> = (0..starts).map(|_| 0).collect();
        for start in 0..starts {
            if let Some(best) = costs.best(start) {
                current[start] = best.at(input.prefix[start]);
                next_break[start] = best.end;
            }
            let suffix = if input.forces_singleton(start, start + 1) {
                previous_singleton[start + 1]
            } else {
                previous[start + 1]
            };
            current_singleton[start] = input.sizes[start] * input.sizes[start] + suffix;
        }
        previous = current;
        previous_singleton = current_singleton;
        breaks.push(next_break);
    }
    let mut result = Vec::new();
    let mut start = 0;
    let mut singleton = false;
    for lines in (1..=line_count).rev() {
        let end = if singleton { start + 1 } else { breaks[lines - 1][start] };
        assert!(end > start, "balanced flex partition must contain every requested nonempty line");
        result.push(end);
        singleton = input.forces_singleton(start, end);
        start = end;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exhaustive(sizes: &[f32], gap: f32, limit: Option<f32>, minimum: usize) -> Vec<usize> {
        let mut best: Option<(usize, f64, Vec<usize>)> = None;
        for mask in 0..1usize << sizes.len().saturating_sub(1) {
            let ends: Vec<_> = (1..sizes.len())
                .filter(|index| mask & (1 << (index - 1)) != 0)
                .chain(core::iter::once(sizes.len()))
                .collect();
            if ends.len() < minimum.min(sizes.len()) {
                continue;
            }
            let mut start = 0;
            let mut cost = 0.0;
            let mut valid = true;
            for (line, &end) in ends.iter().enumerate() {
                let width = sizes[start..end].iter().map(|value| f64::from(value.max(0.0))).sum::<f64>()
                    + f64::from(gap) * (end - start - 1) as f64;
                if end - start > 1 && limit.is_some_and(|limit| width > f64::from(limit)) {
                    valid = false;
                }
                if end < sizes.len()
                    && sizes[end] <= 0.0
                    && ends[line + 1] > end + 1
                    && limit.map_or(true, |limit| width + f64::from(gap) <= f64::from(limit))
                {
                    valid = false;
                }
                cost += width * width;
                start = end;
            }
            if valid
                && best.as_ref().map_or(true, |(lines, previous, indices)| {
                    ends.len() < *lines
                        || (ends.len() == *lines && (cost < *previous || (cost == *previous && ends > *indices)))
                })
            {
                best = Some((ends.len(), cost, ends));
            }
        }
        best.unwrap().2
    }

    #[test]
    fn balance_matches_exhaustive_partitions_with_zero_negative_and_overflowing_items() {
        for count in 1..=6 {
            for encoded in 0usize..4usize.pow(count) {
                let mut value = encoded;
                let sizes: Vec<_> = (0..count)
                    .map(|_| {
                        let size = [-5.0, 0.0, 5.0, 20.0][value % 4];
                        value /= 4;
                        size
                    })
                    .collect();
                for gap in [0.0, 2.0] {
                    for limit in [None, Some(10.0), Some(25.0)] {
                        for minimum in [1, 2, 3] {
                            let actual = balanced_flex_line_ends(&sizes, gap, limit, NonZeroU32::new(minimum).unwrap());
                            let expected = exhaustive(&sizes, gap, limit, minimum as usize);
                            assert_eq!(actual, expected, "{sizes:?};gap {gap};limit {limit:?};minimum {minimum}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn balance_handles_long_unbounded_lines_and_clamps_the_requested_count() {
        let items: Vec<_> = (0..10_000).map(|_| 1.0).collect();
        assert_eq!(balanced_flex_line_ends(&items, 0.0, None, NonZeroU32::new(2).unwrap()), [5_000, 10_000]);
        let ends = balanced_flex_line_ends(&items, 0.0, None, NonZeroU32::new(u32::MAX).unwrap());
        assert_eq!(ends.len(), items.len());
        assert_eq!(ends.last(), Some(&items.len()));
    }
}
