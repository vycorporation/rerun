# Houdini execution work-items tracker

This note maps the demand-driven evaluation/work-items PRD lane to implementation issues in the `vycorporation/rerun` product fork.

## Source PRD

- `product-prd/source/docs/adr/0017-graph-evaluation-is-demand-driven-and-work-item-aware.md`
- `product-prd/source/docs/adr/0018-work-items-use-the-source-graph-with-a-separate-execution-view.md`
- `product-prd/source/docs/adr/0019-evaluation-supports-automatic-and-manual-control.md`

## Issue slices

- Issue `#84`: add a native dockable Execution view backed by graph-owned runtime work-item records.
- Issue `#86`: add the project-level evaluation mode policy for automatic, on-interaction-complete, and manual evaluation.

## Implementation notes

- Work items are operational graph evaluation state, not a second durable graph model.
- The first slice records waiting, running, cached, canceled, superseded, failed, and complete status vocabulary against node/output labels and current evaluation fingerprints.
- Queue, run, cancel, retry, and complete actions update runtime work-item state only; they do not execute Python or native plugin code.
- Sidecar save/load intentionally omits work items so ordinary project persistence does not preserve cached outputs, running work, or historical runtime evaluation state.
- The evaluation mode is durable project intent and does persist with the sidecar. Work items created because of that mode remain runtime state.
