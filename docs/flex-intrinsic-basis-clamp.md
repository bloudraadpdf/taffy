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
