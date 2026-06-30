# Layer stack owns viewport visibility

Houdini Clone uses the layer stack as the primary user-facing control for showing, hiding, and ordering multiple viewport layers. Node outputs remain the source of layer data, but v1 should not copy Houdini's display and render flags directly; instead, users promote compatible node outputs into layers and control visibility through ArcGIS/QGIS-style layer toggles while using the current node for parameter focus and inspection.
