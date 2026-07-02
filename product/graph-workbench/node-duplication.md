# Node duplication tracker

## Issue-backed slices

| Issue | Status | Slice | Outcome |
| --- | --- | --- | --- |
| `#92` | complete | Single-node graph duplication | Adds Duplicate Selected for one graph node. The duplicated node receives a new stable node ID and unique graph-local name, keeps editable graph material such as parameters and comments, and drops generated, runtime, output participation, and output-operator publication state. |
| `#284` | complete | Selected-set graph duplication | Adds multi-node Duplicate Selected as one undoable project command that duplicates the selected nodes together, selects the duplicated set, preserves editable node material, and keeps internal wiring/clipboard behavior deferred. |
| `#286` | complete | Internal wiring for selected-set duplication | Copies selected-internal data-flow edges onto duplicated node sets, remapping endpoints to the duplicated stable node IDs while leaving external incoming/outgoing connections unmodified. |
| `#290` | complete | Graph workbench node clipboard paste | Adds Copy Selected and Paste Nodes actions for project-local graph-node clipboard snapshots, pasting fresh node identities and copied selected-internal wiring while leaving external connections disconnected for later reconnect-policy slices. |
| `#292` | complete | External connection paste choices | Adds explicit graph-node clipboard paste choices for reconnecting copied nodes to original external inputs and/or outputs when the resulting data-flow edges validate. |
| `#294` | complete | Presentation-aware duplication | Adds a separate graph workbench duplication action that preserves selected nodes' output participation and output-operator publication metadata while ordinary duplicate and paste remain conservative. |

PRD sources:

- `product-prd/source/docs/adr/0004-default-to-houdini-like-graph-ergonomics.md`
- `product-prd/source/docs/adr/0075-node-references-use-stable-internal-ids.md`
- `product-prd/source/docs/adr/0002-node-graph-is-the-source-of-truth.md`
- `product-prd/source/CONTEXT.md`

Deferred follow-up slices: none currently tracked for node duplication.
