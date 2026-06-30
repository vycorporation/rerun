---
status: superseded by ADR-0064
---

# Rerun prototype starts with spawned viewer

The first Rerun integration prototype should drive a spawned or separately targeted Rerun viewer through logging and existing APIs before embedding the Rerun web viewer in a panel. Embedding is important for a unified app feel later, but the first spike should prove the graph-to-Rerun data and control loop before spending effort on shell integration.
