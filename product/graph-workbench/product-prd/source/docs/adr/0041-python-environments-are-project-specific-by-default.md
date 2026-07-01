# Python environments are project-specific by default

Houdini Clone uses project-specific Python environments by default. Python operators, procedural assets, plugins, and automation can declare dependency requirements, but the project resolves them into one environment when possible; asset-specific environments are deferred because they are heavy, and global environments are avoided because they are difficult to reproduce.
