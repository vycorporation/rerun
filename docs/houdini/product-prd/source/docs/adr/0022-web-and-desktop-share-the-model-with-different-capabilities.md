# Web and desktop share the model with different capabilities

Houdini Clone uses the same frontend, project model, graph model, layer model, and asset model across web and Tauri desktop builds, but the capability level differs by runtime. Desktop supports full local files, indexing, prepared representations, native plugin operators, and large dataset execution, while web starts with browser-safe sources, uploaded or remote datasets, graph assets, and smaller prepared representations.
