---
status: superseded by ADR-0064
---

# Use Svelte Flow as the initial graph editor

Houdini Clone starts with Svelte Flow as the graph editor view/controller for the external orchestration UI because xyflow provides both React Flow and Svelte Flow for node-based UIs, and Svelte Flow fits the SvelteKit frontend choice. The serialized graph model remains independent of Svelte Flow so the editor library can be replaced or heavily customized later without rewriting projects, procedural assets, plugins, graph evaluation, or viewer targets.
