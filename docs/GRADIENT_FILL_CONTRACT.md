# Vertical gradient fill contract

Implementation work started 2026-09-12 from `a481b4644`. This is an active
contract; acceptance evidence is in `GRADIENT_FILL_AUDIT.md`. Required by the unchanged official RSI.

`fill(plot1, plot2, top_value, bottom_value, top_color, bottom_color, title?,
editable?, fillgaps?, display?)` retains the two masking plot IDs and each bar's
two value/color stops. The host renderer clips the vertical gradient to the
two plot series; the core must not replace it with one sampled solid color.
Missing values remain explicit `na`; no renderer or chart-service dependency
is introduced. Existing solid fills remain unchanged in meaning.

Wire migration: runtime result schema 9 adds an optional `gradient` array to
fill objects; each entry has `topValue`, `bottomValue`, `topColor`,
`bottomColor`. Stream changes schema 4 adds `setGradient` with an absolute
start index and sample suffix. Both producer and replica prune these samples
with the display origin. Existing schemas 2/3 remain readable for deltas that
do not use the new action; new gradient actions must not be accepted under an
older schema. Old result snapshots without gradient data remain readable.
All new serializers advertise the new versions; a schema-8 snapshot
is retained as a legacy-read test, while current expected output snapshots
change only their schema field until a feature-specific assertion changes.

Acceptance requires source argument binding/validation, deterministic samples,
typed JSON read/write, retention across physical pruning, realtime replacement
and confirmation, replica equality and old-schema rejection. The original RSI
must compile without source edits and match all 218 frozen monthly values
including initialization nulls. Native screenshots establish syntax/rendering
controls separately; numerical RSI equality does not prove host rendering.
