# Function declaration validation research (batch 46)

Implementation and release qualification are separate. The last qualified baseline
is batch 45. Current development receipts are under
`target/strategy-gap-research46-20261018`; any batch 46 release qualification must
bind a fresh CLI and unchanged production freeze to its own verification receipts.

The isolated declaration pass checks typed v5/v6 function and method bodies even
when unused, while deferring type constraints dependent on unbound generic
parameters. It never merges temporary HIR, bindings, or specialization metadata
into execution. Libraries use their own source dialect and namespace; private
types exist only in the semantic child environment. Actual runtime capability
limits are unchanged. Imported nested field metadata retains its receiver alias.

Scalar matrix constructors accept series integer dimensions. Native v5/v6
controls cover all five scalar types and float-dimension rejection. Allocation
and runtime size limits are unchanged.

Evidence manifests:

- `function-body-oracles-46.json`: 26 complete-source admission/rejection controls.
- `function-body-shape-oracles-46.json`: 10 supplementary rejection controls.
- `function-body-positive-migration-oracles-46.json`: eight legal call-result and
  matrix dimension controls, including two type rejections.
- `function-body-supplementary-oracles-46.json`: three unchanged runtime originals
  rejected by Native, two legal generic forwarding controls, and two method
  default-parameter controls exposing a separate parser gap. The latter are
  explicitly excluded from claims of implemented support.
- `function-body-method-forwarding-oracles-46.json`: 12 additional whole-source
  controls for generic forwarding through a method into a typed function,
  actual valid/invalid calls, function-form methods, recursion, and unknown
  methods on known receivers. The dependency walk covers both method forms and
  returns only owned diagnostic provenance, so it can use the isolated library
  environment without retaining or copying another analyzer. Known method-name
  failures and recursion remain structural errors with generic arguments.

Original Pine fixtures and historical runtime/analysis goldens are preserved.
Three sema positives were actually illegal forward-global references; their
tests retain rejection assertions and add separate Native-legal galleries.
Three broad runtime stress inputs have separate derived inputs with globals
initialized before methods and ordinary function-form method calls. They retain
the existing expected output and mutations but are synthetic regression inputs,
not Native whole-source qualification. One library stress input has a separate
byte-offset-preserving copy excluding an unused incompatible typed return. Its
generic exported signatures are also synthetic coverage.

Seven negative sema fixture tests retain their exact 460 prior call diagnostics
as an ordered prefix. Any appended declaration diagnostics must belong to an
original function/method body with the correct source identity. See the separate
baseline and migration JSON receipts for hashes and scope.

Remaining confirmed research boundaries include method default parameters and
type-qualified local method calls. Complete original strategy output and broker
ledger parity, exhaustive inference, and host-provided data are separate work.
