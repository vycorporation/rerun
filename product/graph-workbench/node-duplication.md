# Node duplication tracker

## Issue-backed slices

| Issue | Status | Slice | Outcome |
| --- | --- | --- | --- |
| `#92` | complete | Single-node graph duplication | Adds Duplicate Selected for one graph node. The duplicated node receives a new stable node ID and unique graph-local name, keeps editable graph material such as parameters and comments, and drops generated, runtime, output participation, and output-operator publication state. |
| `#284` | complete | Selected-set graph duplication | Adds multi-node Duplicate Selected as one undoable project command that duplicates the selected nodes together, selects the duplicated set, preserves editable node material, and keeps internal wiring/clipboard behavior deferred. |

PRD sources:

- `product-prd/source/docs/adr/0004-default-to-houdini-like-graph-ergonomics.md`
- `product-prd/source/docs/adr/0075-node-references-use-stable-internal-ids.md`
- `product-prd/source/docs/adr/0002-node-graph-is-the-source-of-truth.md`
- `product-prd/source/CONTEXT.md`

Deferred follow-up slices: multi-node clipboard paste, copied internal wiring, reconnect options for external connections, and explicit presentation-aware duplication.
