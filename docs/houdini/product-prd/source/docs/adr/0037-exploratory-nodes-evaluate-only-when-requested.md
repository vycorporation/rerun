# Exploratory nodes evaluate only when requested

Houdini Clone allows disconnected exploratory nodes and temporary graph branches, but demand-driven evaluation ignores them until there is an evaluation request such as node info inspection, layer promotion, export, downstream dependency, or an explicit run. This preserves Houdini-like graph freedom without making random experiments expensive.
