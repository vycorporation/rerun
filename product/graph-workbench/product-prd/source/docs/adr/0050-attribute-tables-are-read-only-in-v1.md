# Attribute tables are read-only in v1

Houdini Clone v1 keeps attribute tables read-only, supporting inspection, search, sort, and temporary filtering without direct value editing. Bulk attribute editing is deferred because it raises source mutability, graph provenance, undo/redo, prepared representation, validation, and filter-ordering questions before the core procedural workflow is stable.
