# Panel layouts are project defaults with user overrides

Houdini Clone allows projects to save a recommended workspace layout while users can override that layout with personal preferences. This supports curated project workspaces without causing individual panel arrangements to constantly rewrite shared project state.

The product should also support Houdini-style workbench layout presets: named layout recipes such as Graph, Inspect, Data, Output, Debug, and Asset Authoring that users can load as starting points. Loading a workbench changes panel arrangement, tab grouping, active panel choices, and viewport emphasis only. It must not create graph nodes, change evaluation state, rewrite layer bindings, or hide workflow semantics in viewer-only state.

Workbench presets are layered above the same workspace layout model. Project defaults can recommend an initial workbench, while user overrides can save personal variants without rewriting shared project state.
