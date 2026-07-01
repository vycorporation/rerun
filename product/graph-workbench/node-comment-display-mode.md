# Node comment display mode tracker

## Issue #94 - network comment display mode

Status: implemented in the Rerun product fork.

PRD sources:

- `product-prd/source/docs/adr/0073-first-malware-artifact-workflow-proves-node-network-mvp.md`
- `product-prd/source/docs/adr/0043-user-node-organization-and-system-status-have-separate-visual-lanes.md`

This slice adds a viewport-level Network comment display preference. Manual mode preserves the durable per-node `show_comment_in_network` toggle, while All Commented renders comment text for every node with non-empty comment text.

The comment badge remains a separate system-status display option.
