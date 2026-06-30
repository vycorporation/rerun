---
status: superseded by ADR-0064
---

# Graph UX starts beside Rerun as orchestration

The Rerun graph UX spike starts with an external graph and layer prototype driving upstream Rerun through existing APIs rather than embedding the graph inside Rerun's UI immediately. This keeps the graph as an orchestration surface that can later drive other viewers, tools, runtimes, or applications, and only moves inside Rerun if the external integration proves the workflow and the boundary becomes the bottleneck.
