---
status: superseded by ADR-0064
---

# SvelteKit TypeScript frontend

Houdini Clone uses SvelteKit with TypeScript for the graph, layer, parameter, asset, and orchestration UI because the app needs a highly interactive interface with low ceremony, built-in reactive state primitives, animation ergonomics, and fewer external dependencies. React has a broader ecosystem, but SvelteKit better matches the desired development style as long as the project model, serialized graph model, viewer targets, and operator contracts stay independent of the UI framework.
