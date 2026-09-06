# Grid minimum contributions and box edges

The local Moegoe CSS Grid Level 2 §6.6 limits a content-based minimum
by the stretch fit into fixed maximum tracks. CSS Sizing Level 3 §2 and
§3.3 keep the inner size non-negative: a border-box contribution cannot
be smaller than its padding and border. Track sizing adds margins after
the native minimum contribution calculation.

`minmax(auto, 0px)` currently loses an empty item's 10px padding or border
when the minimum contribution is capped at zero. The track and a second
item sharing it consequently remain 0px wide or tall.

The same Grid section tests only the tracks an item spans. Scanning the
whole axis incorrectly discards an 80px automatic minimum across two
automatic tracks when a third, unspanned track is flexible. The native
reducer produces 0px instead of 40px for each spanned track.

- [x] Read the local Grid and Sizing rules and record duplication.
- [x] Reproduce the lost track extent through native grid layout.
- [x] Preserve box edges when calculating the native minimum contribution.
- [x] Reproduce automatic minimum suppression by an unspanned flexible track.
- [x] Restrict automatic-minimum track checks to the item's span.
- [x] Verify constraints, both physical axes and item track spans.
- [x] Run native checks and prepare the dependency revision.

The sixty edge and constraint cases and eight flexible-track controls
pass. The full suite passes 156 library, 66 hand-written, 5,541 generated
and five documentation tests; four existing tests remain ignored.
Library Clippy passes with warnings denied. All-target Clippy exposes
ten existing test diagnostics: six nested module names and four copies
written as clones. Duplication remains nine clones (66 lines) across
the grid implementation and renderer reducer, and five clones (46 lines)
in the native constraint tests.

Moegoe's `docs/wpt-geometry-recovery.md` tracks the pin, renderer checks
and priority WPT comparison.
