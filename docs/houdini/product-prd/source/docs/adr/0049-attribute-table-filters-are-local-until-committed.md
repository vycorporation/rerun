# Attribute table filters are local until committed

Houdini Clone treats attribute table search, sort, and temporary filters as table-local inspection state by default, including filters over projected provenance attributes from reference imports. Users can explicitly commit a table filter into graph-backed filter data, avoiding graph clutter from casual browsing while preserving reproducibility when a filter becomes part of the workflow.
