# Grid minimum and maximum bounds

Moegoe A106 exposes two regressions in
`css-grid/layout-algorithm/flex-sizing-rows-min-max-height-001.html` after
projecting intrinsic block bounds into native grid style. The same file
already fails with a definite 70px minimum and 60px maximum.

The local Moegoe `docs/specs/css-grid-2.md` §12.7 sizes flexible tracks
against the container's inner size when constrained by its minimum or
maximum. The container already applies the minimum after the maximum.
Track sizing must use those same effective bounds. The current fraction
restart checks whether the natural 83px grid is below its minimum first,
then whether it exceeds its maximum. With a 70px minimum and 60px maximum,
it restarts at 60px even though the container's used minimum is 70px.

- [x] Read the local Grid contract and trace the native constraint paths.
- [x] Record strict duplication: seven clones, 60 lines, 575 tokens across
      grid layout, track sizing and the native minimum/maximum tests.
- [x] Run a native two-axis reducer before production edits: 16 of 40 cases
      fail, including both WPT failure values (17px and 10px).
- [x] Normalise effective maximum bounds before every grid sizing phase,
      retaining an absent maximum as absent.
- [x] Run native tests, library lint and formatting. All 40 reducer cases
      pass; the complete native suite passes. Strict duplication stays at
      seven clones, 60 lines and 575 tokens.
- [ ] Commit and push the native repair.
- [ ] Update Moegoe pins, restore WPT assertions and compare full CSS.
