# Flex intrinsic contribution clamps

CSS Flexbox §9.9.3 includes the preferred main size in the intrinsic
contribution, then caps the contribution by the flex base size if the item
cannot grow. The base size itself must not be increased to the preferred
size. The specification was checked in `moegoe/docs/specs/css-flexbox-1.md`.

- [x] Read the contribution rule and record the duplication baseline.
- [x] Reduce an inflexible item with a preferred size larger than its basis.
- [x] Apply the flex-base cap independently of the preferred size.
- [x] Run the native suites, duplication check and Clippy.

The 32-case reducer also verifies that authored bounds follow the basis
clamp. The former flex-fraction conversion used inconsistent shrink-factor
floors and could turn a 30px contribution into a negative size. Intrinsic
sizing now sums the contributions directly, as specified by §9.9.1.2.

All 156 library, 65 hand-written, 5,541 generated and five documentation
tests pass with all features. Four existing hand-written tests remain
ignored. Library Clippy passes with warnings denied; duplication remains
25 clones (198 lines).

## Generated fixtures

The generated fixtures hold the layout of Chromium for horizontal documents.
Two rules of this fork need a correction in them.

- The main axis of a column container in a fixture is its block axis. The
  fixture harness gives such a container `FlexMainSizing::HypotheticalItems`,
  as an embedder does. Without it, 16 column fixtures with a `flex-basis`
  length got the intrinsic contributions of a row.
- `width_smaller_then_content_with_flex_grow_unconstraint_size` has items with
  `width: 0` and wider content. By §9.9.3 the content size is the floor of the
  contribution (WPT `css-flexbox/intrinsic-size/row-005.html` uses the same
  rule), so the container is 90px wide. Chromium 148 uses the preferred size
  only and gives 0px. The 4 fixture files hold the §9.9.3 result. A fixture
  generation from Chromium must keep it.

All 165 library, 73 hand-written and 5,541 generated tests pass.
