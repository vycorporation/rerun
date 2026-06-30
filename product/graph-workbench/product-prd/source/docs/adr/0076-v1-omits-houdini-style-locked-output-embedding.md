# V1 omits Houdini-style locked output embedding

Houdini Clone v1 does not model Houdini-style locked-output embedding as a node flag, even though the graph UX otherwise follows Houdini closely. Cached outputs and prepared representations remain runtime or acceleration state outside ordinary project graph data, while durable artifacts are created through explicit packaging, export, recording, or output operators. This gives up a Houdini convenience to avoid hidden large-file project growth and to keep the graph/output-target interface clean for Rerun and future non-Rerun targets.
