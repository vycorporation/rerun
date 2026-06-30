# Timeline and animation are deferred

Houdini Clone v1 does not include a Houdini-style VFX animation model. Time-varying data can exist as time attributes or observation recordings used for filtering, styling, and inspection, but a full authored animation system is deferred because it would add temporal complexity to graph evaluation, cache keys, styling, viewport behavior, Python APIs, and project state before the core spatial workflow is proven.
