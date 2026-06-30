# Renderer accelerates graph-owned filtering and styling

Houdini Clone keeps durable filtering and styling definitions in the node graph while allowing the renderer to apply equivalent prepared representations or GPU-side operations for speed. Rendering must not become an invisible second data pipeline with separate semantics from graph evaluation.
