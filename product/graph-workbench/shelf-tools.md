# Shelf tools tracker

## Issue #90 - built-in graph-backed shelf

Status: implemented in the Rerun product fork.

PRD sources:

- `product-prd/source/docs/adr/0044-shelf-tools-create-graph-backed-project-commands.md`
- `product-prd/source/docs/adr/0045-custom-shelf-tools-are-deferred.md`
- `product-prd/source/docs/adr/0034-generated-nodes-are-visible-but-organized.md`
- `product-prd/source/docs/adr/0073-first-malware-artifact-workflow-proves-node-network-mvp.md`

This slice adds a native dockable Shelf view to the graph workbench. The built-in shelf actions call the shared graph document for output evaluation, selected-node work item actions, starter graph loading, and project-local asset node creation.

Custom user-defined shelf tools remain deferred until procedural assets, Python permissions, plugin capabilities, and the security model are mature enough to support them deliberately.
