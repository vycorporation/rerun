# Native backend executes through typed operators

Houdini Clone's native backend provides local data access, indexing, prepared representations, caching, and heavy operator execution for desktop builds. Native execution must still happen through the same typed operator contract used by the graph so backend capabilities remain inspectable, reproducible, cacheable, and compatible with procedural assets and plugins.
