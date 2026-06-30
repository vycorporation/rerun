# Python and ML operators use declared runtime contracts

Houdini Clone treats Python and ML operators as graph operators with declared inputs, outputs, parameters, dependency requirements, runtime requirements, and provenance by default, modeled after Houdini's ML workflows and plugin-style dependency management. Arbitrary side effects such as file writes, network calls, source mutation, or project mutation require an explicit capability tier, output operator, or project command so Python does not become a hidden escape hatch around the graph model.
