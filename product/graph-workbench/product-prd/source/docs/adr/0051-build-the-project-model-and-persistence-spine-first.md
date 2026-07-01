# Build the project model and persistence spine first

Houdini Clone should start implementation with the real project model, serialized graph model, project-command history, and persistence skeleton, while using small in-memory or demo operators to exercise the first UI. The model is central to the product, so a pure UI mock would not validate the core workflow and a pure backend skeleton would miss important interaction risks.
