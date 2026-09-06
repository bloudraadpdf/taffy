# Auto-repeat constraints and used grid sizes

Local CSS Grid 2 sections 7.2.3.2 and 12 in
`../moegoe/docs/specs/css-grid-2.md` distinguish repeat counting from final
track layout. Intrinsic preferred sizes remain indefinite under Sizing 3
section 2, even after the used container size is known.

An embedding layout engine can supply a resolved container width for
flexible-track expansion while retaining an authored maximum for repeat
counting. A single preferred-size field cannot represent both facts.
Moegoe's renderer reducer currently produces a 70px flexible column in a
30px container, preventing text from wrapping to its required 20px height.

- [x] Record duplication and the failing renderer geometry.
- [x] Represent repeat-count constraints independently from used sizes.
- [x] Reduce native repeat counts, collapsed tracks and used-size controls.
- [x] Reuse the native constraint resolution and repetition algorithm.
- [x] Run native checks and prepare the dependency revision.

`GridAutoRepeatConstraints` retains the preferred, minimum and maximum
size inputs for repeat counting. Box sizing and percentage resolution
remain native. When the override is absent, the grid's ordinary style
and layout constraints supply those inputs.

The native reducer first returns one 100px auto-fit track instead of two
collapsed tracks followed by 100px. All 32 used-size cases now pass across
both physical axes, auto-fill/auto-fit, gaps, box sizing, borders, padding
and length/percentage maxima. Twelve controls cover indefinite inputs,
minimum/maximum selection and clamping of definite preferred sizes.

The default-feature suite passes 154 library, 66 hand-written, 5,541
generated and five documentation tests, with four existing ignored tests.
With serde enabled, 156 library and 68 hand-written tests pass, along with
the same generated and documentation tests. Library Clippy passes with
warnings denied. Duplication remains 22 clones (262 lines) in the native
scope and zero in the XML fixture loader. The optional constraint payload
adds 56 bytes to Style on the tested 64-bit target; size assertions record
the resulting 608-byte String style and 576-byte Arc style.

Moegoe's `docs/wpt-geometry-recovery.md` tracks the push, pin and renderer
validation.
