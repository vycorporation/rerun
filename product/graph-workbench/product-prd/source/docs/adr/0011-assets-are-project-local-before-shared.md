# Assets are project-local before shared

Houdini Clone supports both project assets and shared assets, but v1 assets are project-local by default. Users can explicitly publish a project asset into an asset library once it is stable enough for reuse across projects, keeping experimentation cheap while preserving a clean path to shared workflows. Publishing an asset carries its graph structure, interface, documentation, and dependency declarations, but heavy external artifacts remain explicit references or typed inputs unless the user separately packages a project or artifact bundle.
