# V1 source dataset formats

Houdini Clone v1 targets GeoJSON, FlatGeobuf, Parquet or GeoParquet-like tabular geometry, CSV with coordinate or geometry columns, LAS/LAZ point clouds, and generic SQLite tables or views with configurable geometry columns. GeoPackage and SpatiaLite are later compatibility layers, and shapefile support is explicitly deferred because it brings legacy GIS packaging, encoding, and CRS expectations that conflict with the projection-agnostic v1 scope.
