# Node duplication tracker

## Issue #92 - single-node graph duplication

Status: implemented in the Rerun product fork.

PRD sources:

- `product-prd/source/docs/adr/0004-default-to-houdini-like-graph-ergonomics.md`
- `product-prd/source/docs/adr/0075-node-references-use-stable-internal-ids.md`
- `product-prd/source/docs/adr/0002-node-graph-is-the-source-of-truth.md`
- `product-prd/source/CONTEXT.md`

This slice adds Duplicate Selected for one graph node. The duplicated node receives a new stable node ID and unique graph-local name, keeps editable graph material such as parameters and comments, and drops generated, runtime, output participation, and output-operator publication state.

Deferred follow-up slices: multi-node clipboard paste, copied internal wiring, reconnect options for external connections, selected-set duplication, and explicit presentation-aware duplication.
