use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use arrow::array::{Float32Array, Float64Array, RecordBatch};
use arrow::datatypes::SchemaRef;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GraphPoint {
    pub x: f32,
    pub y: f32,
}

impl GraphPoint {
    const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn clamped_to_unit(self) -> Self {
        Self {
            x: self.x.clamp(0.0, 1.0),
            y: self.y.clamp(0.0, 1.0),
        }
    }
}

pub(crate) struct GraphDocument {
    pub source: GraphSource,
    pub nodes: Vec<GraphNode>,
    pub layers: Vec<Layer>,
    pub style: GraphStyle,
    pub geometry: Vec<Geometry>,
    pub recording_geometry: Vec<Geometry>,
}

const GENERATED_NODE_LANE_Y: f32 = 0.82;

impl GraphDocument {
    pub fn sample() -> Self {
        let geometry = vec![
            Geometry::Polygon(Polygon {
                points: vec![
                    GraphPoint::new(0.0, 0.0),
                    GraphPoint::new(1.0, 0.1),
                    GraphPoint::new(0.8, 1.0),
                    GraphPoint::new(0.0, 0.8),
                ],
                score: 0.62,
            }),
            Geometry::Polygon(Polygon {
                points: vec![
                    GraphPoint::new(0.08, 0.25),
                    GraphPoint::new(0.36, 0.18),
                    GraphPoint::new(0.42, 0.44),
                    GraphPoint::new(0.2, 0.56),
                ],
                score: 0.48,
            }),
            Geometry::CubicBezier(CubicBezier {
                start: GraphPoint::new(0.0, 0.0),
                control_1: GraphPoint::new(0.25, 1.0),
                control_2: GraphPoint::new(0.75, -0.4),
                end: GraphPoint::new(1.0, 0.6),
                score: 0.82,
            }),
            Geometry::CubicBezier(CubicBezier {
                start: GraphPoint::new(0.18, 0.12),
                control_1: GraphPoint::new(0.35, 0.9),
                control_2: GraphPoint::new(0.68, 0.05),
                end: GraphPoint::new(0.92, 0.88),
                score: 0.35,
            }),
        ];

        Self {
            source: GraphSource::demo_fallback(SourceMetadata::from_geometry(
                SourceProvenance::DemoFallback,
                None,
                &geometry,
                Vec::new(),
            )),
            nodes: vec![
                GraphNode {
                    name: "Source",
                    kind: NodeKind::Source,
                    layout_position: GraphPoint::new(0.0, 0.5),
                    generated: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    parameter: NodeParameter::scalar(
                        "Read",
                        1.0,
                        0.0..=1.0,
                        "Source readiness placeholder for the spike graph.",
                    ),
                    info: "Loads polygon and cubic Bezier records.",
                },
                GraphNode {
                    name: "Filter",
                    kind: NodeKind::Filter,
                    layout_position: GraphPoint::new(0.33, 0.5),
                    generated: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    parameter: NodeParameter::attribute_rule(
                        "Minimum score",
                        "score",
                        FilterComparison::GreaterOrEqual,
                        0.55,
                        0.0..=1.0,
                        "Controls the minimum sample score emitted by the filter.",
                    ),
                    info: "Filters features by sample score.",
                },
                GraphNode {
                    name: "Style",
                    kind: NodeKind::Style,
                    layout_position: GraphPoint::new(0.66, 0.5),
                    generated: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    parameter: NodeParameter::scalar(
                        "Stroke scale",
                        0.75,
                        0.0..=1.0,
                        "Controls output stroke scale without mutating native geometry.",
                    ),
                    info: "Assigns visual parameters before viewer output.",
                },
                GraphNode {
                    name: "Rerun Output",
                    kind: NodeKind::Output,
                    layout_position: GraphPoint::new(1.0, 0.5),
                    generated: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    parameter: NodeParameter::scalar(
                        "Adaptive segments",
                        1.0,
                        0.0..=1.0,
                        "Controls only the prepared export polyline. The native cubic remains four points.",
                    ),
                    info: "Prepares adaptive viewer geometry only at the output edge.",
                },
            ],
            layers: vec![
                Layer {
                    name: "Polygons".to_owned(),
                    kind: LayerKind::Polygons,
                    visible: true,
                    order: 0,
                    style: GraphStyle::default(),
                },
                Layer {
                    name: "Curves".to_owned(),
                    kind: LayerKind::Curves,
                    visible: true,
                    order: 1,
                    style: GraphStyle {
                        color: GraphColor {
                            r: 239,
                            g: 188,
                            b: 84,
                        },
                        opacity: 0.85,
                        stroke_scale: 0.75,
                    },
                },
                Layer {
                    name: "Debug Output".to_owned(),
                    kind: LayerKind::Debug,
                    visible: false,
                    order: 2,
                    style: GraphStyle {
                        color: GraphColor {
                            r: 115,
                            g: 210,
                            b: 155,
                        },
                        opacity: 0.85,
                        stroke_scale: 0.75,
                    },
                },
            ],
            style: GraphStyle::default(),
            geometry,
            recording_geometry: Vec::new(),
        }
    }

    pub fn polygon_count(&self) -> usize {
        self.active_geometry()
            .iter()
            .filter(|geometry| matches!(geometry, Geometry::Polygon(_)))
            .count()
    }

    pub fn cubic_bezier_count(&self) -> usize {
        self.active_geometry()
            .iter()
            .filter(|geometry| matches!(geometry, Geometry::CubicBezier(_)))
            .count()
    }

    pub fn polygon_vertex_count(&self) -> usize {
        self.active_geometry()
            .iter()
            .map(|geometry| match geometry {
                Geometry::Polygon(polygon) => polygon.points.len(),
                Geometry::CubicBezier(_) => 0,
            })
            .sum()
    }

    pub fn cubic_control_point_count(&self) -> usize {
        self.active_geometry()
            .iter()
            .map(|geometry| match geometry {
                Geometry::Polygon(_) => 0,
                Geometry::CubicBezier(curve) => curve.control_points().len(),
            })
            .sum()
    }

    #[allow(dead_code)]
    pub fn filter_minimum_score(&self) -> f32 {
        self.filter_rule()
            .and_then(|rule| rule.value.as_f32())
            .unwrap_or(0.0)
    }

    pub fn filter_rule(&self) -> Option<AttributeFilterRule> {
        self.nodes
            .iter()
            .find(|node| node.kind == NodeKind::Filter)
            .and_then(|node| node.parameter.as_attribute_rule())
    }

    pub fn style_scale(&self) -> f32 {
        self.nodes
            .iter()
            .find(|node| node.kind == NodeKind::Style)
            .map_or(0.5, |node| node.parameter.value)
    }

    pub fn resolved_style(&self) -> GraphStyle {
        self.style.with_stroke_scale(self.style_scale())
    }

    pub fn export_segments(&self) -> usize {
        let segment_factor = self
            .nodes
            .iter()
            .find(|node| node.kind == NodeKind::Output)
            .map_or(0.5, |node| node.parameter.value);

        (2.0 + segment_factor * 14.0).round() as usize
    }

    pub fn layer_visible(&self, kind: LayerKind) -> bool {
        self.layers
            .iter()
            .any(|layer| layer.kind == kind && layer.visible)
    }

    pub fn ordered_layer_views(&self) -> Vec<&Layer> {
        let mut layers = self.layers.iter().collect::<Vec<_>>();
        layers.sort_by_key(|layer| layer.order);
        layers
    }

    pub fn duplicate_layer_view(&mut self, kind: LayerKind, name: impl Into<String>) -> bool {
        let Some(source_layer) = self.layers.iter().find(|layer| layer.kind == kind) else {
            return false;
        };

        let next_order = self
            .layers
            .iter()
            .map(|layer| layer.order)
            .max()
            .unwrap_or_default()
            + 1;
        self.layers.push(Layer {
            name: name.into(),
            kind,
            visible: source_layer.visible,
            order: next_order,
            style: source_layer.style,
        });
        true
    }

    pub fn emits(&self, geometry: &Geometry) -> bool {
        let layer_visible = match geometry {
            Geometry::Polygon(_) => self.layer_visible(LayerKind::Polygons),
            Geometry::CubicBezier(_) => self.layer_visible(LayerKind::Curves),
        };

        layer_visible && self.passes_filter(geometry)
    }

    pub fn visible_output_count(&self) -> usize {
        self.viewer_output().items.len()
    }

    pub fn attribute_table_rows(&self, query: &AttributeTableQuery) -> Vec<AttributeTableRow> {
        let source_path = self
            .source
            .source_path
            .clone()
            .or_else(|| self.source.metadata.source_path.clone());
        let provenance = self.source.metadata.provenance;
        let search = query.search.trim().to_ascii_lowercase();

        let mut rows = self
            .active_geometry()
            .iter()
            .enumerate()
            .filter(|(_, geometry)| self.emits(geometry))
            .map(|(record_index, geometry)| AttributeTableRow {
                record_index,
                geometry_kind: geometry.kind(),
                score: geometry.score(),
                layer: geometry.layer(),
                point_count: geometry.control_or_vertex_count(),
                source_path: source_path.clone(),
                provenance,
                is_native_cubic_bezier: matches!(geometry, Geometry::CubicBezier(_)),
            })
            .filter(|row| {
                query
                    .minimum_score
                    .is_none_or(|minimum_score| row.score >= minimum_score)
            })
            .filter(|row| search.is_empty() || row.matches_search(&search))
            .collect::<Vec<_>>();

        rows.sort_by(|left, right| query.sort.compare(left, right));
        if query.sort_descending {
            rows.reverse();
        }
        rows
    }

    pub fn commit_attribute_table_query_as_filter(&mut self, query: &AttributeTableQuery) -> bool {
        let Some(minimum_score) = query.minimum_score else {
            return false;
        };

        let Some(filter_node) = self
            .nodes
            .iter_mut()
            .find(|node| node.kind == NodeKind::Filter)
        else {
            return false;
        };

        filter_node.parameter.value = minimum_score.clamp(
            *filter_node.parameter.range.start(),
            *filter_node.parameter.range.end(),
        );
        filter_node.parameter.kind = NodeParameterKind::AttributeRule;
        filter_node.parameter.rule_spec = Some(AttributeFilterRuleSpec {
            attribute_name: "score".to_owned(),
            comparison: FilterComparison::GreaterOrEqual,
        });
        filter_node.layout_position = GraphPoint::new(0.33, GENERATED_NODE_LANE_Y);
        filter_node.generated = Some(GeneratedNodeInfo {
            source: GeneratedNodeSource::AttributeTableCommit,
        });
        true
    }

    pub fn pipeline_stages(&self) -> Vec<PipelineStage> {
        let source_count = self.active_geometry().len();
        let filtered_count = self
            .active_geometry()
            .iter()
            .filter(|geometry| self.passes_filter(geometry))
            .count();
        let styled_count = filtered_count;
        let output_count = self.visible_output_count();

        vec![
            PipelineStage {
                name: "Source",
                input_count: 0,
                output_count: source_count,
                note: "Loaded native graph geometry.",
            },
            PipelineStage {
                name: "Filter",
                input_count: source_count,
                output_count: filtered_count,
                note: "Applied minimum score threshold.",
            },
            PipelineStage {
                name: "Style",
                input_count: filtered_count,
                output_count: styled_count,
                note: "Prepared stroke scale for viewer output.",
            },
            PipelineStage {
                name: "Rerun Output",
                input_count: styled_count,
                output_count,
                note: "Applied layer visibility and boundary preparation.",
            },
        ]
    }

    pub fn graph_layout(&self) -> GraphLayout {
        let nodes = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| GraphLayoutNode {
                node_index: index,
                name: node.name,
                position: node.layout_position,
            })
            .collect();

        let output_nodes = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| node.participates_in_output.then_some(index))
            .collect::<Vec<_>>();
        let edges = output_nodes
            .windows(2)
            .map(|nodes| GraphEdge {
                from_node: nodes[0],
                to_node: nodes[1],
            })
            .collect();

        GraphLayout { nodes, edges }
    }

    pub fn set_node_layout_position(&mut self, index: usize, position: GraphPoint) {
        if let Some(node) = self.nodes.get_mut(index) {
            node.layout_position = position.clamped_to_unit();
        }
    }

    #[allow(dead_code)]
    pub fn mark_node_stale(&mut self, index: usize) {
        if let Some(node) = self.nodes.get_mut(index) {
            node.evaluation.state = EvaluationState::Stale;
            node.evaluation.message = None;
        }
    }

    pub fn set_node_manual(&mut self, index: usize, manual: bool) {
        if let Some(node) = self.nodes.get_mut(index) {
            node.evaluation.manual = manual;
            node.evaluation.state = if manual {
                EvaluationState::Manual
            } else {
                EvaluationState::Stale
            };
            node.evaluation.message = None;
        }
    }

    pub fn demand_output_evaluation(&mut self) {
        for node in &mut self.nodes {
            if !node.participates_in_output || node.evaluation.manual {
                continue;
            }
            if matches!(
                node.evaluation.state,
                EvaluationState::Stale | EvaluationState::Running
            ) {
                node.evaluation.state = EvaluationState::Cached;
                node.evaluation.message = None;
            }
        }
    }

    pub fn request_node_run(&mut self, index: usize) {
        if let Some(node) = self.nodes.get_mut(index) {
            node.evaluation.state = EvaluationState::Running;
            node.evaluation.message = None;
        }
    }

    pub fn complete_node_run(&mut self, index: usize) {
        if let Some(node) = self.nodes.get_mut(index) {
            node.evaluation.state = EvaluationState::Clean;
            node.evaluation.manual = false;
            node.evaluation.message = None;
        }
    }

    pub fn cancel_node_run(&mut self, index: usize) {
        if let Some(node) = self.nodes.get_mut(index)
            && node.evaluation.state == EvaluationState::Running
        {
            node.evaluation.state = EvaluationState::Manual;
            node.evaluation.manual = true;
            node.evaluation.message = Some("Run cancelled".to_owned());
        }
    }

    #[allow(dead_code)]
    pub fn fail_node_run(&mut self, index: usize, message: impl Into<String>) {
        if let Some(node) = self.nodes.get_mut(index) {
            node.evaluation.state = EvaluationState::Failed;
            node.evaluation.message = Some(message.into());
        }
    }

    pub fn selected_node_info(&self, index: usize) -> Option<NodeInfo> {
        let node = self.nodes.get(index)?;
        let stages = self.pipeline_stages();
        let source_metadata = self.source.metadata.clone();
        let filter_warnings = self.filter_rule_warning().into_iter().collect::<Vec<_>>();
        let style_warnings = self.style_warnings();

        Some(match node.kind {
            NodeKind::Source => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                input_count: stages[0].input_count,
                output_count: stages[0].output_count,
                status: if self.source.import_error.is_some() {
                    NodeStatus::Failed
                } else {
                    NodeStatus::Healthy
                },
                data_kind: "Source geometry",
                record_count: source_metadata.record_count,
                bounds: source_metadata.bounds,
                provenance: Some(source_metadata.provenance),
                attributes: source_metadata.attribute_names,
                parameter: node.parameter.clone(),
                summary: "Source geometry lives in the graph model before any viewer adaptation.",
                source_metadata: Some(self.source.metadata.clone()),
                source_error: self.source.import_error.clone(),
                style: None,
                generated: node.generated,
                evaluation: node.evaluation.clone(),
                warnings: Vec::new(),
            },
            NodeKind::Filter => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                input_count: stages[1].input_count,
                output_count: stages[1].output_count,
                status: if filter_warnings.is_empty() {
                    NodeStatus::Healthy
                } else {
                    NodeStatus::Warning
                },
                data_kind: "Filtered geometry",
                record_count: stages[1].output_count,
                bounds: self.filtered_bounds(),
                provenance: Some(self.source.metadata.provenance),
                attributes: self.source.metadata.attribute_names.clone(),
                parameter: node.parameter.clone(),
                summary: "Filter removes geometry that does not satisfy its typed attribute rule.",
                source_metadata: None,
                source_error: None,
                style: None,
                generated: node.generated,
                evaluation: node.evaluation.clone(),
                warnings: filter_warnings,
            },
            NodeKind::Style => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                input_count: stages[2].input_count,
                output_count: stages[2].output_count,
                status: if style_warnings.is_empty() {
                    NodeStatus::Healthy
                } else {
                    NodeStatus::Warning
                },
                data_kind: "Styled geometry",
                record_count: stages[2].output_count,
                bounds: self.filtered_bounds(),
                provenance: Some(self.source.metadata.provenance),
                attributes: self.source.metadata.attribute_names.clone(),
                parameter: node.parameter.clone(),
                summary: "Style changes viewer presentation without mutating graph geometry.",
                source_metadata: None,
                source_error: None,
                style: Some(self.resolved_style()),
                generated: node.generated,
                evaluation: node.evaluation.clone(),
                warnings: style_warnings,
            },
            NodeKind::Output => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                input_count: stages[3].input_count,
                output_count: stages[3].output_count,
                status: NodeStatus::Healthy,
                data_kind: "Rerun scene output",
                record_count: stages[3].output_count,
                bounds: self.output_bounds(),
                provenance: Some(self.source.metadata.provenance),
                attributes: self.source.metadata.attribute_names.clone(),
                parameter: node.parameter.clone(),
                summary: "Output prepares boundary data while preserving native graph geometry.",
                source_metadata: None,
                source_error: None,
                style: None,
                generated: node.generated,
                evaluation: node.evaluation.clone(),
                warnings: Vec::new(),
            },
        })
    }

    pub fn viewer_output(&self) -> ViewerOutput {
        let style = self.resolved_style();
        ViewerOutput {
            stroke_scale: style.stroke_scale,
            style,
            items: self
                .ordered_layer_views()
                .into_iter()
                .filter(|layer| layer.visible && layer.kind != LayerKind::Debug)
                .flat_map(|layer| {
                    self.active_geometry()
                        .iter()
                        .filter(move |geometry| geometry.layer() == layer.kind)
                        .filter(|geometry| self.passes_filter(geometry))
                        .map(move |geometry| ViewerItem {
                            layer: LayerView {
                                name: layer.name.clone(),
                                kind: layer.kind,
                                order: layer.order,
                                style: layer.style.with_stroke_scale(style.stroke_scale),
                            },
                            geometry: match geometry {
                                Geometry::Polygon(polygon) => {
                                    ViewerGeometry::Polygon(polygon.clone())
                                }
                                Geometry::CubicBezier(curve) => ViewerGeometry::CubicBezier(*curve),
                            },
                        })
                })
                .collect(),
        }
    }

    #[cfg(test)]
    pub fn adaptive_export_output(&self) -> ExportOutput {
        let curve_segments = self.export_segments().max(1);
        ExportOutput {
            items: self
                .active_geometry()
                .iter()
                .filter(|geometry| self.emits(geometry))
                .map(|geometry| match geometry {
                    Geometry::Polygon(polygon) => ExportGeometry::Polygon(polygon.points.clone()),
                    Geometry::CubicBezier(curve) => {
                        ExportGeometry::Polyline(curve.adaptive_polyline(curve_segments))
                    }
                })
                .collect(),
        }
    }

    pub fn prepared_export_point_count(&self) -> usize {
        let curve_point_count = self.export_segments().max(1) + 1;
        self.active_geometry()
            .iter()
            .filter(|geometry| self.emits(geometry))
            .map(|geometry| match geometry {
                Geometry::Polygon(polygon) => polygon.points.len(),
                Geometry::CubicBezier(_) => curve_point_count,
            })
            .sum()
    }

    pub fn render_feasibility_summary(&self) -> RenderFeasibilitySummary {
        let mut summary = RenderFeasibilitySummary {
            export_segments_per_cubic: self.export_segments(),
            ..RenderFeasibilitySummary::default()
        };

        for geometry in self
            .active_geometry()
            .iter()
            .filter(|geometry| self.emits(geometry))
        {
            summary.native_viewer_primitive_count += 1;
            summary.graph_owned_point_count += geometry.control_or_vertex_count();
            match geometry {
                Geometry::Polygon(_) => {
                    summary.polygon_count += 1;
                }
                Geometry::CubicBezier(_) => {
                    summary.native_cubic_bezier_count += 1;
                    summary.prepared_boundary_debug_point_count +=
                        summary.export_segments_per_cubic + 1 + 4;
                }
            }
        }

        summary
    }

    pub fn load_synthetic_render_benchmark(
        &mut self,
        native_cubic_bezier_count: usize,
        polygon_count: usize,
    ) -> RenderFeasibilitySummary {
        let geometry =
            synthetic_render_benchmark_geometry(native_cubic_bezier_count, polygon_count);
        self.source = GraphSource::recording_import(
            geometry.len(),
            Some(format!(
                "synthetic native render benchmark: {native_cubic_bezier_count} cubics, {polygon_count} polygons"
            )),
            SourceMetadata::from_geometry(
                SourceProvenance::SyntheticBenchmark,
                None,
                &geometry,
                Vec::new(),
            ),
        );
        self.recording_geometry = geometry;
        self.update_source_node_readiness();
        self.render_feasibility_summary()
    }

    #[cfg(test)]
    pub fn rerun_scene_output(&self) -> RerunSceneOutput {
        self.rerun_scene_output_with_query_bridge(None)
    }

    pub fn rerun_scene_output_with_query_bridge(
        &self,
        query_bridge: Option<RerunQueryBridge>,
    ) -> RerunSceneOutput {
        self.rerun_scene_output_with_debug_items(query_bridge, true)
    }

    pub fn rerun_scene_output_for_view(
        &self,
        query_bridge: Option<RerunQueryBridge>,
        include_debug_items: bool,
    ) -> RerunSceneOutput {
        self.rerun_scene_output_with_debug_items(query_bridge, include_debug_items)
    }

    fn rerun_scene_output_with_debug_items(
        &self,
        query_bridge: Option<RerunQueryBridge>,
        include_debug_items: bool,
    ) -> RerunSceneOutput {
        let viewer_output = self.viewer_output();
        let style = viewer_output.style;
        let export_segments = self.export_segments();
        let mut items = Vec::with_capacity(viewer_output.items.len());
        let mut debug_items = Vec::new();

        for item in viewer_output.items {
            match item.geometry {
                ViewerGeometry::Polygon(polygon) => {
                    items.push(RerunSceneItem::Polygon {
                        points: polygon.points,
                        layer: item.layer.kind,
                        layer_name: item.layer.name,
                        layer_order: item.layer.order,
                        score: polygon.score,
                        style: item.layer.style,
                    });
                }
                ViewerGeometry::CubicBezier(curve) => {
                    if include_debug_items {
                        debug_items.push(RerunSceneDebugItem::PreparedExportPolyline(
                            curve.adaptive_polyline(export_segments.max(1)),
                        ));
                        debug_items.push(RerunSceneDebugItem::NativeCubicControlPolygon(
                            curve.control_points(),
                        ));
                    }
                    items.push(RerunSceneItem::NativeCubicBezier {
                        curve,
                        layer: item.layer.kind,
                        layer_name: item.layer.name,
                        layer_order: item.layer.order,
                        score: curve.score,
                        style: item.layer.style,
                    });
                }
            }
        }

        RerunSceneOutput {
            stroke_scale: viewer_output.stroke_scale,
            style,
            export_segments,
            query_bridge,
            items,
            debug_items,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_rerun_recording(
        &self,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<DurableRecordingResult> {
        use re_sdk::RecordingStreamBuilder;
        use re_sdk_types::{
            archetypes::{LineStrips2D, Points2D, TextDocument},
            components::{LineStrip2D, Position2D, Radius},
        };

        let path = path.as_ref();
        let scene = self.rerun_scene_output_with_query_bridge(None);
        let rec = RecordingStreamBuilder::new("houdini_graph_output").save(path)?;

        rec.log_static(
            "houdini_graph/metadata",
            &TextDocument::new(scene.recording_metadata_markdown(self)),
        )?;

        for (index, item) in scene.items.iter().enumerate() {
            let entity_base = format!(
                "houdini_graph/output/{:03}_{}_{}",
                index,
                item.kind_slug(),
                sanitize_entity_path_part(item.layer_name())
            );
            let color = recording_color(item.style());
            let radius = Radius::new_ui_points((item.style().stroke_scale * 3.0).max(1.0));

            match item {
                RerunSceneItem::Polygon {
                    points,
                    layer_order,
                    ..
                } => {
                    let mut strip_points = points
                        .iter()
                        .copied()
                        .map(recording_position)
                        .collect::<Vec<_>>();
                    if let Some(first) = strip_points.first().copied() {
                        strip_points.push(first);
                    }
                    rec.log(
                        format!("{entity_base}/geometry"),
                        &LineStrips2D::new([LineStrip2D::from_iter(
                            strip_points.into_iter().map(|point| [point.x(), point.y()]),
                        )])
                        .with_colors([color])
                        .with_radii([radius])
                        .with_labels([item.recording_label(index)])
                        .with_draw_order(*layer_order as f32),
                    )?;
                }
                RerunSceneItem::NativeCubicBezier {
                    curve, layer_order, ..
                } => {
                    let control_points = curve
                        .control_points()
                        .into_iter()
                        .map(recording_position)
                        .collect::<Vec<Position2D>>();
                    rec.log(
                        format!("{entity_base}/native_control_points"),
                        &Points2D::new(control_points.clone())
                            .with_colors([color])
                            .with_radii([Radius::new_ui_points(4.0)])
                            .with_labels(["P0", "P1", "P2", "P3"])
                            .with_draw_order((*layer_order as f32) + 10.0),
                    )?;
                    rec.log(
                        format!("{entity_base}/control_polygon_preview"),
                        &LineStrips2D::new([LineStrip2D::from_iter(
                            control_points.iter().map(|point| [point.x(), point.y()]),
                        )])
                        .with_colors([color])
                        .with_radii([radius])
                        .with_labels([item.recording_label(index)])
                        .with_draw_order(*layer_order as f32),
                    )?;
                }
            }

            rec.log_static(
                format!("{entity_base}/metadata"),
                &TextDocument::new(item.recording_metadata_markdown(index)),
            )?;
        }

        rec.flush_blocking()?;

        Ok(DurableRecordingResult {
            path: path.to_path_buf(),
            item_count: scene.items.len(),
            polygon_count: scene.polygon_count(),
            native_cubic_bezier_count: scene.native_cubic_bezier_count(),
            limitation_note: CUBIC_RECORDING_LIMITATION.to_owned(),
        })
    }

    pub fn update_source_from_query_bridge(&mut self, query_bridge: &RerunQueryBridge) {
        if self.source.metadata.provenance == SourceProvenance::ParquetImport {
            return;
        }

        let previous_metadata = self.source.metadata.clone();
        self.source = GraphSource::from_query_bridge(query_bridge);
        if self.source.mode == GraphSourceMode::DemoFallback {
            self.source.metadata = previous_metadata;
        }
        self.update_source_node_readiness();
    }

    pub fn import_cubic_bezier_parquet_path(
        &mut self,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<usize> {
        let path = path.as_ref();
        match load_cubic_bezier_parquet_with_metadata(path) {
            Ok(load) => {
                let count = load.records.len();
                self.source = GraphSource::recording_import(
                    count,
                    Some(path.display().to_string()),
                    load.metadata,
                );
                self.recording_geometry = load
                    .records
                    .into_iter()
                    .map(|record| record.geometry)
                    .collect();
                self.update_source_node_readiness();
                Ok(count)
            }
            Err(err) => {
                self.source = self.source.clone().with_import_error(path, &err);
                Err(err)
            }
        }
    }

    pub fn save_sidecar_json(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        std::fs::write(path, self.to_sidecar_json()?)?;
        Ok(())
    }

    pub fn load_sidecar_json(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let json = std::fs::read_to_string(path)?;
        self.apply_sidecar_json(&json)
    }

    pub fn to_sidecar_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(
            &HoudiniGraphSidecar::from_document(self),
        )?)
    }

    pub fn apply_sidecar_json(&mut self, json: &str) -> anyhow::Result<()> {
        let sidecar = serde_json::from_str::<HoudiniGraphSidecar>(json)?;
        sidecar.apply_to_document(self)
    }

    fn update_source_node_readiness(&mut self) {
        if let Some(source_node) = self
            .nodes
            .iter_mut()
            .find(|node| node.kind == NodeKind::Source)
        {
            source_node.parameter.value = if self.source.mode == GraphSourceMode::DemoFallback {
                1.0
            } else {
                0.0
            };
        }
    }

    #[allow(dead_code)]
    pub fn import_recording_geometry(
        &mut self,
        query_bridge: &RerunQueryBridge,
        records: impl IntoIterator<Item = HoudiniGeometryRecord>,
    ) {
        self.update_source_from_query_bridge(query_bridge);
        self.source.mode = GraphSourceMode::RecordingQuery;
        self.recording_geometry = records.into_iter().map(|record| record.geometry).collect();
        self.source.metadata = SourceMetadata::from_geometry(
            SourceProvenance::RecordingQuery,
            None,
            &self.recording_geometry,
            Vec::new(),
        );
        self.update_source_node_readiness();
    }

    #[allow(dead_code)]
    pub fn import_cubic_bezier_parquet(
        &mut self,
        query_bridge: &RerunQueryBridge,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<usize> {
        let path = path.as_ref();
        match load_cubic_bezier_parquet_with_metadata(path) {
            Ok(load) => {
                let count = load.records.len();
                self.import_recording_geometry(query_bridge, load.records);
                self.source.metadata = load.metadata;
                self.source.source_path = Some(path.display().to_string());
                Ok(count)
            }
            Err(err) => {
                self.source = self.source.clone().with_import_error(path, &err);
                Err(err)
            }
        }
    }

    fn active_geometry(&self) -> &[Geometry] {
        match self.source.mode {
            GraphSourceMode::DemoFallback => &self.geometry,
            GraphSourceMode::RecordingQuery => &self.recording_geometry,
        }
    }

    fn passes_filter(&self, geometry: &Geometry) -> bool {
        self.filter_rule().map_or(true, |rule| {
            rule.matches_geometry(geometry).unwrap_or(false)
        })
    }

    fn filtered_bounds(&self) -> Option<GeometryBounds> {
        GeometryBounds::from_geometry(
            self.active_geometry()
                .iter()
                .filter(|geometry| self.passes_filter(geometry)),
        )
    }

    fn output_bounds(&self) -> Option<GeometryBounds> {
        GeometryBounds::from_geometry(
            self.active_geometry()
                .iter()
                .filter(|geometry| self.emits(geometry)),
        )
    }

    fn filter_rule_warning(&self) -> Option<String> {
        let rule = self.filter_rule()?;
        self.active_geometry()
            .first()
            .and_then(|geometry| rule.matches_geometry(geometry).err())
            .map(|err| err.to_string())
    }

    fn style_warnings(&self) -> Vec<String> {
        let style = self.resolved_style();
        let mut warnings = Vec::new();

        if !(0.0..=1.0).contains(&style.opacity) {
            warnings.push(format!(
                "Style opacity must be between 0.0 and 1.0; got {:.2}",
                style.opacity
            ));
        }

        if !(0.0..=1.0).contains(&style.stroke_scale) {
            warnings.push(format!(
                "Style stroke scale must be between 0.0 and 1.0; got {:.2}",
                style.stroke_scale
            ));
        }

        warnings
    }
}

#[cfg(not(target_arch = "wasm32"))]
const CUBIC_RECORDING_LIMITATION: &str = "Rerun recordings preserve cubic Bezier semantics as graph-owned control-point metadata. The current replay path visualizes cubic curves as native control points plus a control-polygon preview; dense polyline tessellation remains an adaptive boundary/export representation only.";

#[allow(dead_code)]
pub(crate) struct HoudiniGeometrySchema;

#[allow(dead_code)]
impl HoudiniGeometrySchema {
    pub const ARCHETYPE_NAME: &'static str = "vy.houdini.Geometry2D";
    pub const KIND_COMPONENT: &'static str = "HoudiniGeometry2D:kind";
    pub const POINTS_COMPONENT: &'static str = "HoudiniGeometry2D:points";
    pub const SCORE_COMPONENT: &'static str = "HoudiniGeometry2D:score";
    pub const LAYER_COMPONENT: &'static str = "HoudiniGeometry2D:layer";

    pub fn component_names() -> [&'static str; 4] {
        [
            Self::KIND_COMPONENT,
            Self::POINTS_COMPONENT,
            Self::SCORE_COMPONENT,
            Self::LAYER_COMPONENT,
        ]
    }
}

pub(crate) struct HoudiniCubicBezierParquetSchema;

#[allow(dead_code)]
impl HoudiniCubicBezierParquetSchema {
    pub const CONTROL_POINT_COLUMNS: [&'static str; 8] = [
        "cp0_x", "cp0_y", "cp1_x", "cp1_y", "cp2_x", "cp2_y", "cp3_x", "cp3_y",
    ];

    const CONTROL_POINT_ALIASES: [[&'static str; 2]; 8] = [
        ["cp0_x", "p0_x"],
        ["cp0_y", "p0_y"],
        ["cp1_x", "p1_x"],
        ["cp1_y", "p1_y"],
        ["cp2_x", "p2_x"],
        ["cp2_y", "p2_y"],
        ["cp3_x", "p3_x"],
        ["cp3_y", "p3_y"],
    ];

    pub fn required_column_count() -> usize {
        Self::CONTROL_POINT_COLUMNS.len()
    }
}

#[allow(dead_code)]
pub(crate) fn load_cubic_bezier_parquet(
    path: impl AsRef<Path>,
) -> anyhow::Result<Vec<HoudiniGeometryRecord>> {
    Ok(load_cubic_bezier_parquet_with_metadata(path)?.records)
}

pub(crate) fn load_cubic_bezier_parquet_with_metadata(
    path: impl AsRef<Path>,
) -> anyhow::Result<HoudiniParquetLoad> {
    let path = path.as_ref();
    let file = std::fs::File::open(path)?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)?.build()?;

    let mut records = Vec::new();
    let mut recognized_control_point_columns = None;
    for batch in reader {
        append_cubic_bezier_batch(batch?, &mut records, &mut recognized_control_point_columns)?;
    }

    let geometry = records
        .iter()
        .map(|record| record.geometry.clone())
        .collect::<Vec<_>>();
    let metadata = SourceMetadata::from_geometry(
        SourceProvenance::ParquetImport,
        Some(path.display().to_string()),
        &geometry,
        recognized_control_point_columns.unwrap_or_default(),
    );

    Ok(HoudiniParquetLoad { records, metadata })
}

pub(crate) struct HoudiniParquetLoad {
    pub records: Vec<HoudiniGeometryRecord>,
    pub metadata: SourceMetadata,
}

fn append_cubic_bezier_batch(
    batch: RecordBatch,
    records: &mut Vec<HoudiniGeometryRecord>,
    recognized_control_point_columns: &mut Option<Vec<String>>,
) -> anyhow::Result<()> {
    if recognized_control_point_columns.is_none() {
        *recognized_control_point_columns =
            Some(recognized_control_point_columns_for_schema(batch.schema())?);
    }

    let columns = HoudiniCubicBezierParquetSchema::CONTROL_POINT_ALIASES
        .iter()
        .map(|aliases| numeric_column(&batch, aliases))
        .collect::<anyhow::Result<Vec<_>>>()?;

    for row_index in 0..batch.num_rows() {
        let curve = CubicBezier {
            start: GraphPoint::new(columns[0](row_index), columns[1](row_index)),
            control_1: GraphPoint::new(columns[2](row_index), columns[3](row_index)),
            control_2: GraphPoint::new(columns[4](row_index), columns[5](row_index)),
            end: GraphPoint::new(columns[6](row_index), columns[7](row_index)),
            score: 1.0,
        };
        records.push(HoudiniGeometryRecord::cubic_bezier(
            LayerKind::Curves,
            curve,
        ));
    }

    Ok(())
}

fn recognized_control_point_columns_for_schema(schema: SchemaRef) -> anyhow::Result<Vec<String>> {
    HoudiniCubicBezierParquetSchema::CONTROL_POINT_ALIASES
        .iter()
        .map(|aliases| {
            aliases
                .iter()
                .find(|name| schema.index_of(name).is_ok())
                .map(|name| (*name).to_owned())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Missing Houdini cubic Bezier parquet column: {}",
                        aliases[0]
                    )
                })
        })
        .collect()
}

fn numeric_column<'a>(
    batch: &'a RecordBatch,
    aliases: &[&'static str],
) -> anyhow::Result<Box<dyn Fn(usize) -> f32 + 'a>> {
    let Some((name, column)) = aliases.iter().find_map(|name| {
        batch
            .schema()
            .index_of(name)
            .ok()
            .map(|index| (*name, batch.column(index)))
    }) else {
        anyhow::bail!(
            "Missing Houdini cubic Bezier parquet column: {}",
            aliases[0]
        );
    };

    if let Some(values) = column.as_any().downcast_ref::<Float64Array>() {
        Ok(Box::new(|row_index| values.value(row_index) as f32))
    } else if let Some(values) = column.as_any().downcast_ref::<Float32Array>() {
        Ok(Box::new(|row_index| values.value(row_index)))
    } else {
        anyhow::bail!("Houdini cubic Bezier parquet column must be float32 or float64: {name}");
    }
}

#[allow(dead_code)]
pub(crate) struct HoudiniGeometryRecord {
    pub kind: HoudiniGeometryKind,
    pub layer: LayerKind,
    pub score: f32,
    pub geometry: Geometry,
}

impl HoudiniGeometryRecord {
    #[allow(dead_code)]
    pub fn polygon(layer: LayerKind, points: Vec<GraphPoint>, score: f32) -> Option<Self> {
        if points.len() < 3 {
            return None;
        }

        Some(Self {
            kind: HoudiniGeometryKind::Polygon,
            layer,
            score,
            geometry: Geometry::Polygon(Polygon { points, score }),
        })
    }

    #[allow(dead_code)]
    pub fn cubic_bezier(layer: LayerKind, curve: CubicBezier) -> Self {
        Self {
            kind: HoudiniGeometryKind::CubicBezier,
            layer,
            score: curve.score,
            geometry: Geometry::CubicBezier(curve),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum HoudiniGeometryKind {
    Polygon,
    CubicBezier,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct SourceMetadata {
    pub provenance: SourceProvenance,
    pub source_path: Option<String>,
    pub record_count: usize,
    pub polygon_count: usize,
    pub cubic_bezier_count: usize,
    pub bounds: Option<GeometryBounds>,
    pub attribute_names: Vec<String>,
    pub recognized_control_point_columns: Vec<String>,
}

impl Default for SourceMetadata {
    fn default() -> Self {
        Self {
            provenance: SourceProvenance::DemoFallback,
            source_path: None,
            record_count: 0,
            polygon_count: 0,
            cubic_bezier_count: 0,
            bounds: None,
            attribute_names: Vec::new(),
            recognized_control_point_columns: Vec::new(),
        }
    }
}

impl SourceMetadata {
    fn from_geometry(
        provenance: SourceProvenance,
        source_path: Option<String>,
        geometry: &[Geometry],
        recognized_control_point_columns: Vec<String>,
    ) -> Self {
        let mut bounds = None;
        let mut polygon_count = 0;
        let mut cubic_bezier_count = 0;

        for item in geometry {
            match item {
                Geometry::Polygon(polygon) => {
                    polygon_count += 1;
                    for point in &polygon.points {
                        GeometryBounds::include_point(&mut bounds, *point);
                    }
                }
                Geometry::CubicBezier(curve) => {
                    cubic_bezier_count += 1;
                    for point in curve.control_points() {
                        GeometryBounds::include_point(&mut bounds, point);
                    }
                }
            }
        }

        Self {
            provenance,
            source_path,
            record_count: geometry.len(),
            polygon_count,
            cubic_bezier_count,
            bounds,
            attribute_names: vec!["score".to_owned()],
            recognized_control_point_columns,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SourceProvenance {
    DemoFallback,
    ParquetImport,
    RecordingQuery,
    SyntheticBenchmark,
}

impl SourceProvenance {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DemoFallback => "demo fallback",
            Self::ParquetImport => "parquet import",
            Self::RecordingQuery => "recording query",
            Self::SyntheticBenchmark => "synthetic benchmark",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GeometryBounds {
    pub min: GraphPoint,
    pub max: GraphPoint,
}

impl GeometryBounds {
    fn from_geometry<'a>(geometry: impl IntoIterator<Item = &'a Geometry>) -> Option<Self> {
        let mut bounds = None;
        for item in geometry {
            match item {
                Geometry::Polygon(polygon) => {
                    for point in &polygon.points {
                        Self::include_point(&mut bounds, *point);
                    }
                }
                Geometry::CubicBezier(curve) => {
                    for point in curve.control_points() {
                        Self::include_point(&mut bounds, point);
                    }
                }
            }
        }
        bounds
    }

    fn include_point(bounds: &mut Option<Self>, point: GraphPoint) {
        if let Some(bounds) = bounds {
            bounds.min.x = bounds.min.x.min(point.x);
            bounds.min.y = bounds.min.y.min(point.y);
            bounds.max.x = bounds.max.x.max(point.x);
            bounds.max.y = bounds.max.y.max(point.y);
        } else {
            *bounds = Some(Self {
                min: point,
                max: point,
            });
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphSource {
    pub mode: GraphSourceMode,
    pub matching_entity_count: usize,
    pub visible_data_result_count: usize,
    pub source_path: Option<String>,
    pub metadata: SourceMetadata,
    pub import_error: Option<String>,
}

impl GraphSource {
    fn demo_fallback(metadata: SourceMetadata) -> Self {
        Self {
            mode: GraphSourceMode::DemoFallback,
            matching_entity_count: 0,
            visible_data_result_count: 0,
            source_path: metadata.source_path.clone(),
            metadata,
            import_error: None,
        }
    }

    fn from_query_bridge(query_bridge: &RerunQueryBridge) -> Self {
        let has_recording_input =
            query_bridge.matching_entity_count > 0 || query_bridge.visible_data_result_count > 0;

        Self {
            mode: if has_recording_input {
                GraphSourceMode::RecordingQuery
            } else {
                GraphSourceMode::DemoFallback
            },
            matching_entity_count: query_bridge.matching_entity_count,
            visible_data_result_count: query_bridge.visible_data_result_count,
            source_path: None,
            metadata: SourceMetadata {
                provenance: if has_recording_input {
                    SourceProvenance::RecordingQuery
                } else {
                    SourceProvenance::DemoFallback
                },
                record_count: query_bridge.visible_data_result_count,
                ..Default::default()
            },
            import_error: None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self.mode {
            GraphSourceMode::DemoFallback => "demo fallback",
            GraphSourceMode::RecordingQuery => "recording query",
        }
    }

    fn recording_import(
        imported_geometry_count: usize,
        source_path: Option<String>,
        metadata: SourceMetadata,
    ) -> Self {
        Self {
            mode: GraphSourceMode::RecordingQuery,
            matching_entity_count: imported_geometry_count,
            visible_data_result_count: imported_geometry_count,
            source_path,
            metadata,
            import_error: None,
        }
    }

    fn with_import_error(mut self, source_path: &Path, err: &anyhow::Error) -> Self {
        self.source_path = Some(source_path.display().to_string());
        self.import_error = Some(format!("{err}"));
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum GraphSourceMode {
    DemoFallback,
    RecordingQuery,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct HoudiniGraphSidecar {
    version: u32,
    source: GraphSourceSidecar,
    nodes: Vec<NodeSidecar>,
    layers: Vec<LayerSidecar>,
    #[serde(default)]
    style: GraphStyle,
    demo_geometry: Vec<Geometry>,
    recording_geometry: Vec<Geometry>,
}

impl HoudiniGraphSidecar {
    const VERSION: u32 = 1;

    fn from_document(graph: &GraphDocument) -> Self {
        Self {
            version: Self::VERSION,
            source: GraphSourceSidecar {
                mode: graph.source.mode,
                matching_entity_count: graph.source.matching_entity_count,
                visible_data_result_count: graph.source.visible_data_result_count,
                source_path: graph.source.source_path.clone(),
                metadata: graph.source.metadata.clone(),
                import_error: graph.source.import_error.clone(),
            },
            nodes: graph
                .nodes
                .iter()
                .map(|node| NodeSidecar {
                    kind: node.kind,
                    layout_position: node.layout_position,
                    parameter_value: node.parameter.value,
                    parameter_rule: node.parameter.rule_spec.clone(),
                    generated: node.generated,
                })
                .collect(),
            layers: graph
                .layers
                .iter()
                .map(|layer| LayerSidecar {
                    name: layer.name.clone(),
                    kind: layer.kind,
                    visible: layer.visible,
                    order: Some(layer.order),
                    style: layer.style,
                })
                .collect(),
            style: graph.resolved_style(),
            demo_geometry: graph.geometry.clone(),
            recording_geometry: graph.recording_geometry.clone(),
        }
    }

    fn apply_to_document(self, graph: &mut GraphDocument) -> anyhow::Result<()> {
        if self.version != Self::VERSION {
            anyhow::bail!(
                "unsupported Houdini graph sidecar version {}; expected {}",
                self.version,
                Self::VERSION
            );
        }

        graph.source = GraphSource {
            mode: self.source.mode,
            matching_entity_count: self.source.matching_entity_count,
            visible_data_result_count: self.source.visible_data_result_count,
            source_path: self.source.source_path,
            metadata: self.source.metadata,
            import_error: self.source.import_error,
        };
        graph.geometry = self.demo_geometry;
        graph.recording_geometry = self.recording_geometry;
        graph.style = self.style;

        for node_snapshot in self.nodes {
            if let Some(node) = graph
                .nodes
                .iter_mut()
                .find(|node| node.kind == node_snapshot.kind)
            {
                node.layout_position = node_snapshot.layout_position.clamped_to_unit();
                node.parameter.value = node_snapshot
                    .parameter_value
                    .clamp(*node.parameter.range.start(), *node.parameter.range.end());
                if let Some(parameter_rule) = node_snapshot.parameter_rule {
                    node.parameter.rule_spec = Some(parameter_rule);
                }
                node.generated = node_snapshot.generated;
            }
        }

        if !self.layers.is_empty() {
            graph.layers = self
                .layers
                .into_iter()
                .enumerate()
                .map(|(index, layer_snapshot)| layer_snapshot.into_layer(index as i32))
                .collect();
        }

        graph.update_source_node_readiness();
        graph.style.stroke_scale = graph.style_scale();
        Ok(())
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct GraphSourceSidecar {
    mode: GraphSourceMode,
    matching_entity_count: usize,
    visible_data_result_count: usize,
    source_path: Option<String>,
    #[serde(default)]
    metadata: SourceMetadata,
    #[serde(default)]
    import_error: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct NodeSidecar {
    kind: NodeKind,
    layout_position: GraphPoint,
    parameter_value: f32,
    #[serde(default)]
    parameter_rule: Option<AttributeFilterRuleSpec>,
    #[serde(default)]
    generated: Option<GeneratedNodeInfo>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct LayerSidecar {
    #[serde(default)]
    name: String,
    kind: LayerKind,
    visible: bool,
    #[serde(default)]
    order: Option<i32>,
    #[serde(default)]
    style: GraphStyle,
}

impl LayerSidecar {
    fn into_layer(self, fallback_order: i32) -> Layer {
        Layer {
            name: if self.name.is_empty() {
                self.kind.as_str().to_owned()
            } else {
                self.name
            },
            kind: self.kind,
            visible: self.visible,
            order: self.order.unwrap_or(fallback_order),
            style: self.style,
        }
    }
}

pub(crate) struct GraphNode {
    pub name: &'static str,
    pub kind: NodeKind,
    pub layout_position: GraphPoint,
    pub generated: Option<GeneratedNodeInfo>,
    pub evaluation: NodeEvaluation,
    pub participates_in_output: bool,
    pub parameter: NodeParameter,
    pub info: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NodeEvaluation {
    pub state: EvaluationState,
    pub manual: bool,
    pub message: Option<String>,
}

impl NodeEvaluation {
    fn clean() -> Self {
        Self {
            state: EvaluationState::Clean,
            manual: false,
            message: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EvaluationState {
    Clean,
    Cached,
    Stale,
    Running,
    #[allow(dead_code)]
    Failed,
    Manual,
}

impl EvaluationState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "Clean",
            Self::Cached => "Cached",
            Self::Stale => "Stale",
            Self::Running => "Running",
            Self::Failed => "Failed",
            Self::Manual => "Manual",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GeneratedNodeInfo {
    pub source: GeneratedNodeSource,
}

impl GeneratedNodeInfo {
    pub fn as_str(self) -> &'static str {
        self.source.as_str()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum GeneratedNodeSource {
    AttributeTableCommit,
}

impl GeneratedNodeSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AttributeTableCommit => "Generated from attribute table commit",
        }
    }
}

#[derive(Clone)]
pub(crate) struct NodeParameter {
    pub name: &'static str,
    pub kind: NodeParameterKind,
    pub value: f32,
    pub range: std::ops::RangeInclusive<f32>,
    pub help: &'static str,
    pub rule_spec: Option<AttributeFilterRuleSpec>,
}

impl NodeParameter {
    pub fn scalar(
        name: &'static str,
        value: f32,
        range: std::ops::RangeInclusive<f32>,
        help: &'static str,
    ) -> Self {
        Self {
            name,
            kind: NodeParameterKind::Scalar,
            value,
            range,
            help,
            rule_spec: None,
        }
    }

    pub fn attribute_rule(
        name: &'static str,
        attribute_name: &'static str,
        comparison: FilterComparison,
        value: f32,
        range: std::ops::RangeInclusive<f32>,
        help: &'static str,
    ) -> Self {
        Self {
            name,
            kind: NodeParameterKind::AttributeRule,
            value,
            range,
            help,
            rule_spec: Some(AttributeFilterRuleSpec {
                attribute_name: attribute_name.to_owned(),
                comparison,
            }),
        }
    }

    pub fn as_attribute_rule(&self) -> Option<AttributeFilterRule> {
        let rule_spec = self.rule_spec.clone()?;
        Some(AttributeFilterRule {
            attribute_name: rule_spec.attribute_name,
            comparison: rule_spec.comparison,
            value: AttributeValue::Float32(self.value),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NodeParameterKind {
    Scalar,
    AttributeRule,
}

impl NodeParameterKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scalar => "Scalar",
            Self::AttributeRule => "Attribute rule",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct AttributeFilterRuleSpec {
    pub attribute_name: String,
    pub comparison: FilterComparison,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AttributeFilterRule {
    pub attribute_name: String,
    pub comparison: FilterComparison,
    pub value: AttributeValue,
}

impl AttributeFilterRule {
    fn matches_geometry(&self, geometry: &Geometry) -> anyhow::Result<bool> {
        let actual = match self.attribute_name.as_str() {
            "score" => geometry.score(),
            attribute_name => {
                anyhow::bail!("Unknown filter attribute: {attribute_name}");
            }
        };
        self.comparison
            .matches(actual, self.value.as_f32().unwrap_or(actual))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum FilterComparison {
    GreaterOrEqual,
    LessOrEqual,
}

impl FilterComparison {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GreaterOrEqual => ">=",
            Self::LessOrEqual => "<=",
        }
    }

    fn matches(self, actual: f32, expected: f32) -> anyhow::Result<bool> {
        Ok(match self {
            Self::GreaterOrEqual => actual >= expected,
            Self::LessOrEqual => actual <= expected,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AttributeValue {
    Float32(f32),
}

impl AttributeValue {
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Self::Float32(value) => Some(*value),
        }
    }
}

pub(crate) struct GraphLayout {
    pub nodes: Vec<GraphLayoutNode>,
    pub edges: Vec<GraphEdge>,
}

pub(crate) struct GraphLayoutNode {
    pub node_index: usize,
    pub name: &'static str,
    pub position: GraphPoint,
}

pub(crate) struct GraphEdge {
    pub from_node: usize,
    pub to_node: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum NodeKind {
    Source,
    Filter,
    Style,
    Output,
}

impl NodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Source => "Source",
            Self::Filter => "Filter",
            Self::Style => "Style",
            Self::Output => "Output",
        }
    }

    pub fn role(self) -> &'static str {
        match self {
            Self::Source => "Read",
            Self::Filter => "Cull",
            Self::Style => "Style",
            Self::Output => "Publish",
        }
    }
}

pub(crate) struct NodeInfo {
    pub kind: NodeKind,
    pub role: &'static str,
    pub input_count: usize,
    pub output_count: usize,
    pub status: NodeStatus,
    pub data_kind: &'static str,
    pub record_count: usize,
    pub bounds: Option<GeometryBounds>,
    pub provenance: Option<SourceProvenance>,
    pub attributes: Vec<String>,
    pub parameter: NodeParameter,
    pub summary: &'static str,
    pub source_metadata: Option<SourceMetadata>,
    pub source_error: Option<String>,
    pub style: Option<GraphStyle>,
    pub generated: Option<GeneratedNodeInfo>,
    pub evaluation: NodeEvaluation,
    pub warnings: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NodeStatus {
    Healthy,
    Warning,
    Failed,
}

impl NodeStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "Healthy",
            Self::Warning => "Warning",
            Self::Failed => "Failed",
        }
    }
}

pub(crate) struct PipelineStage {
    pub name: &'static str,
    pub input_count: usize,
    pub output_count: usize,
    pub note: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AttributeTableQuery {
    pub search: String,
    pub minimum_score: Option<f32>,
    pub sort: AttributeTableSort,
    pub sort_descending: bool,
}

impl Default for AttributeTableQuery {
    fn default() -> Self {
        Self {
            search: String::new(),
            minimum_score: None,
            sort: AttributeTableSort::RecordIndex,
            sort_descending: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AttributeTableSort {
    RecordIndex,
    GeometryKind,
    Score,
    Layer,
}

impl AttributeTableSort {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RecordIndex => "Index",
            Self::GeometryKind => "Kind",
            Self::Score => "Score",
            Self::Layer => "Layer",
        }
    }

    fn compare(self, left: &AttributeTableRow, right: &AttributeTableRow) -> std::cmp::Ordering {
        match self {
            Self::RecordIndex => left.record_index.cmp(&right.record_index),
            Self::GeometryKind => left
                .geometry_kind
                .as_str()
                .cmp(right.geometry_kind.as_str()),
            Self::Score => left.score.total_cmp(&right.score),
            Self::Layer => left.layer.as_str().cmp(right.layer.as_str()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AttributeTableRow {
    pub record_index: usize,
    pub geometry_kind: GeometryKind,
    pub score: f32,
    pub layer: LayerKind,
    pub point_count: usize,
    pub source_path: Option<String>,
    pub provenance: SourceProvenance,
    pub is_native_cubic_bezier: bool,
}

impl AttributeTableRow {
    fn matches_search(&self, search: &str) -> bool {
        self.geometry_kind
            .as_str()
            .to_ascii_lowercase()
            .contains(search)
            || self.layer.as_str().to_ascii_lowercase().contains(search)
            || self
                .provenance
                .as_str()
                .to_ascii_lowercase()
                .contains(search)
            || self
                .source_path
                .as_deref()
                .is_some_and(|path| path.to_ascii_lowercase().contains(search))
    }
}

pub(crate) struct Layer {
    pub name: String,
    pub kind: LayerKind,
    pub visible: bool,
    pub order: i32,
    pub style: GraphStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum LayerKind {
    Polygons,
    Curves,
    Debug,
}

impl LayerKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Polygons => "Polygons",
            Self::Curves => "Curves",
            Self::Debug => "Debug",
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub(crate) enum Geometry {
    Polygon(Polygon),
    CubicBezier(CubicBezier),
}

impl Geometry {
    pub fn score(&self) -> f32 {
        match self {
            Self::Polygon(polygon) => polygon.score,
            Self::CubicBezier(curve) => curve.score,
        }
    }

    pub fn kind(&self) -> GeometryKind {
        match self {
            Self::Polygon(_) => GeometryKind::Polygon,
            Self::CubicBezier(_) => GeometryKind::CubicBezier,
        }
    }

    pub fn layer(&self) -> LayerKind {
        match self {
            Self::Polygon(_) => LayerKind::Polygons,
            Self::CubicBezier(_) => LayerKind::Curves,
        }
    }

    pub fn control_or_vertex_count(&self) -> usize {
        match self {
            Self::Polygon(polygon) => polygon.points.len(),
            Self::CubicBezier(curve) => curve.control_points().len(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GeometryKind {
    Polygon,
    CubicBezier,
}

impl GeometryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Polygon => "Polygon",
            Self::CubicBezier => "Native cubic Bezier",
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct Polygon {
    pub points: Vec<GraphPoint>,
    pub score: f32,
}

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
pub(crate) struct CubicBezier {
    pub start: GraphPoint,
    pub control_1: GraphPoint,
    pub control_2: GraphPoint,
    pub end: GraphPoint,
    pub score: f32,
}

impl CubicBezier {
    pub fn control_points(&self) -> [GraphPoint; 4] {
        [self.start, self.control_1, self.control_2, self.end]
    }

    fn adaptive_polyline(&self, segments: usize) -> Vec<GraphPoint> {
        (0..=segments)
            .map(|index| {
                let t = index as f32 / segments as f32;
                self.point_at(t)
            })
            .collect()
    }

    fn point_at(&self, t: f32) -> GraphPoint {
        let inv_t = 1.0 - t;
        let b0 = inv_t * inv_t * inv_t;
        let b1 = 3.0 * inv_t * inv_t * t;
        let b2 = 3.0 * inv_t * t * t;
        let b3 = t * t * t;

        GraphPoint::new(
            self.start.x * b0 + self.control_1.x * b1 + self.control_2.x * b2 + self.end.x * b3,
            self.start.y * b0 + self.control_1.y * b1 + self.control_2.y * b2 + self.end.y * b3,
        )
    }
}

pub(crate) struct ViewerOutput {
    pub items: Vec<ViewerItem>,
    pub stroke_scale: f32,
    pub style: GraphStyle,
}

pub(crate) struct ViewerItem {
    pub layer: LayerView,
    pub geometry: ViewerGeometry,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LayerView {
    pub name: String,
    pub kind: LayerKind,
    pub order: i32,
    pub style: GraphStyle,
}

pub(crate) enum ViewerGeometry {
    Polygon(Polygon),
    CubicBezier(CubicBezier),
}

#[cfg(test)]
pub(crate) struct ExportOutput {
    pub items: Vec<ExportGeometry>,
}

#[cfg(test)]
pub(crate) enum ExportGeometry {
    Polygon(Vec<GraphPoint>),
    Polyline(Vec<GraphPoint>),
}

pub(crate) struct RerunSceneOutput {
    pub items: Vec<RerunSceneItem>,
    pub debug_items: Vec<RerunSceneDebugItem>,
    pub stroke_scale: f32,
    pub style: GraphStyle,
    pub export_segments: usize,
    pub query_bridge: Option<RerunQueryBridge>,
}

impl RerunSceneOutput {
    pub fn polygon_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, RerunSceneItem::Polygon { .. }))
            .count()
    }

    pub fn native_cubic_bezier_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, RerunSceneItem::NativeCubicBezier { .. }))
            .count()
    }

    pub fn graph_owned_point_count(&self) -> usize {
        self.items
            .iter()
            .map(RerunSceneItem::control_or_vertex_count)
            .sum()
    }

    pub fn prepared_boundary_debug_point_count(&self) -> usize {
        self.debug_items
            .iter()
            .map(RerunSceneDebugItem::point_count)
            .sum()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn recording_metadata_markdown(&self, graph: &GraphDocument) -> String {
        let source_path = graph
            .source
            .source_path
            .as_deref()
            .or(graph.source.metadata.source_path.as_deref())
            .unwrap_or("none");
        let mut markdown = format!(
            "# Houdini Graph Recording\n\n\
             Source path: `{source_path}`\n\n\
             Source provenance: `{}`\n\n\
             Output items: `{}`\n\n\
             Polygons: `{}`\n\n\
             Native cubic Beziers: `{}`\n\n\
             Limitation: {}\n\n\
             | index | kind | layer | score | style |\n\
             | --- | --- | --- | ---: | --- |\n",
            graph.source.metadata.provenance.as_str(),
            self.items.len(),
            self.polygon_count(),
            self.native_cubic_bezier_count(),
            CUBIC_RECORDING_LIMITATION
        );

        for (index, item) in self.items.iter().enumerate() {
            markdown.push_str(&format!(
                "| {index} | {} | {} | {:.3} | {} |\n",
                item.kind_name(),
                item.layer_name(),
                item.score(),
                item.style().recording_summary()
            ));
        }

        markdown
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderFeasibilitySummary {
    pub native_viewer_primitive_count: usize,
    pub polygon_count: usize,
    pub native_cubic_bezier_count: usize,
    pub graph_owned_point_count: usize,
    pub prepared_boundary_debug_point_count: usize,
    pub export_segments_per_cubic: usize,
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct DurableRecordingResult {
    pub path: PathBuf,
    pub item_count: usize,
    pub polygon_count: usize,
    pub native_cubic_bezier_count: usize,
    pub limitation_note: String,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GraphStyle {
    pub color: GraphColor,
    pub opacity: f32,
    pub stroke_scale: f32,
}

impl GraphStyle {
    fn with_stroke_scale(self, stroke_scale: f32) -> Self {
        Self {
            stroke_scale,
            ..self
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn recording_summary(self) -> String {
        format!(
            "rgb({}, {}, {}), opacity {:.2}, stroke {:.2}",
            self.color.r, self.color.g, self.color.b, self.opacity, self.stroke_scale
        )
    }
}

impl Default for GraphStyle {
    fn default() -> Self {
        Self {
            color: GraphColor {
                r: 91,
                g: 169,
                b: 255,
            },
            opacity: 0.55,
            stroke_scale: 0.75,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GraphColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RerunQueryBridge {
    pub mode: RerunQueryBridgeMode,
    pub view_id: String,
    pub space_origin: String,
    pub timeline: String,
    pub latest_at: i64,
    pub matching_entity_count: usize,
    pub visualized_entity_count: usize,
    pub visible_data_result_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RerunQueryBridgeMode {
    ProductForkViewOwned,
}

impl RerunQueryBridgeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProductForkViewOwned => "product-fork view-owned query bridge",
        }
    }
}

pub(crate) enum RerunSceneItem {
    Polygon {
        points: Vec<GraphPoint>,
        layer: LayerKind,
        layer_name: String,
        layer_order: i32,
        score: f32,
        style: GraphStyle,
    },
    NativeCubicBezier {
        curve: CubicBezier,
        layer: LayerKind,
        layer_name: String,
        layer_order: i32,
        score: f32,
        style: GraphStyle,
    },
}

impl RerunSceneItem {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Polygon { .. } => "Polygon",
            Self::NativeCubicBezier { .. } => "Native cubic Bezier",
        }
    }

    pub fn layer(&self) -> LayerKind {
        match self {
            Self::Polygon { layer, .. } | Self::NativeCubicBezier { layer, .. } => *layer,
        }
    }

    pub fn layer_name(&self) -> &str {
        match self {
            Self::Polygon { layer_name, .. } | Self::NativeCubicBezier { layer_name, .. } => {
                layer_name
            }
        }
    }

    pub fn layer_order(&self) -> i32 {
        match self {
            Self::Polygon { layer_order, .. } | Self::NativeCubicBezier { layer_order, .. } => {
                *layer_order
            }
        }
    }

    pub fn score(&self) -> f32 {
        match self {
            Self::Polygon { score, .. } | Self::NativeCubicBezier { score, .. } => *score,
        }
    }

    pub fn style(&self) -> GraphStyle {
        match self {
            Self::Polygon { style, .. } | Self::NativeCubicBezier { style, .. } => *style,
        }
    }

    pub fn control_or_vertex_count(&self) -> usize {
        match self {
            Self::Polygon { points, .. } => points.len(),
            Self::NativeCubicBezier { curve, .. } => curve.control_points().len(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn kind_slug(&self) -> &'static str {
        match self {
            Self::Polygon { .. } => "polygon",
            Self::NativeCubicBezier { .. } => "native_cubic_bezier",
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn recording_label(&self, index: usize) -> String {
        format!(
            "#{index} {} {} score {:.3}",
            self.kind_name(),
            self.layer_name(),
            self.score()
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn recording_metadata_markdown(&self, index: usize) -> String {
        format!(
            "# Output item {index}\n\n\
             Kind: `{}`\n\n\
             Layer: `{}`\n\n\
             Layer kind: `{}`\n\n\
             Layer order: `{}`\n\n\
             Score: `{:.3}`\n\n\
             Style: `{}`\n\n\
             Control or vertex count: `{}`\n",
            self.kind_name(),
            self.layer_name(),
            self.layer().as_str(),
            self.layer_order(),
            self.score(),
            self.style().recording_summary(),
            self.control_or_vertex_count()
        )
    }
}

pub(crate) enum RerunSceneDebugItem {
    NativeCubicControlPolygon([GraphPoint; 4]),
    PreparedExportPolyline(Vec<GraphPoint>),
}

impl RerunSceneDebugItem {
    fn point_count(&self) -> usize {
        match self {
            Self::NativeCubicControlPolygon(points) => points.len(),
            Self::PreparedExportPolyline(points) => points.len(),
        }
    }
}

fn synthetic_render_benchmark_geometry(
    native_cubic_bezier_count: usize,
    polygon_count: usize,
) -> Vec<Geometry> {
    let total = native_cubic_bezier_count.saturating_add(polygon_count);
    let mut geometry = Vec::with_capacity(total);
    let columns = ((total as f32).sqrt().ceil() as usize).max(1);
    let cell = 1.0 / columns as f32;

    for index in 0..native_cubic_bezier_count {
        let origin = benchmark_cell_origin(index, columns, cell);
        let wobble = ((index % 11) as f32) * cell * 0.01;
        geometry.push(Geometry::CubicBezier(CubicBezier {
            start: GraphPoint::new(origin.x + cell * 0.10, origin.y + cell * 0.18),
            control_1: GraphPoint::new(origin.x + cell * 0.32, origin.y + cell * (0.88 - wobble)),
            control_2: GraphPoint::new(origin.x + cell * 0.70, origin.y + cell * (0.12 + wobble)),
            end: GraphPoint::new(origin.x + cell * 0.92, origin.y + cell * 0.78),
            score: benchmark_score(index),
        }));
    }

    for polygon_index in 0..polygon_count {
        let index = native_cubic_bezier_count + polygon_index;
        let origin = benchmark_cell_origin(index, columns, cell);
        geometry.push(Geometry::Polygon(Polygon {
            points: vec![
                GraphPoint::new(origin.x + cell * 0.16, origin.y + cell * 0.16),
                GraphPoint::new(origin.x + cell * 0.86, origin.y + cell * 0.20),
                GraphPoint::new(origin.x + cell * 0.78, origin.y + cell * 0.82),
                GraphPoint::new(origin.x + cell * 0.22, origin.y + cell * 0.74),
            ],
            score: benchmark_score(index),
        }));
    }

    geometry
}

fn benchmark_cell_origin(index: usize, columns: usize, cell: f32) -> GraphPoint {
    let column = index % columns;
    let row = index / columns;
    GraphPoint::new(column as f32 * cell, row as f32 * cell)
}

fn benchmark_score(index: usize) -> f32 {
    0.55 + ((index % 100) as f32 / 250.0)
}

#[cfg(not(target_arch = "wasm32"))]
fn recording_position(point: GraphPoint) -> re_sdk_types::components::Position2D {
    re_sdk_types::components::Position2D::new(point.x, point.y)
}

#[cfg(not(target_arch = "wasm32"))]
fn recording_color(style: GraphStyle) -> re_sdk_types::components::Color {
    let alpha = (style.opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
    re_sdk_types::components::Color::from_unmultiplied_rgba(
        style.color.r,
        style.color.g,
        style.color.b,
        alpha,
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn sanitize_entity_path_part(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();

    if sanitized.is_empty() {
        "unnamed".to_owned()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AttributeTableQuery, AttributeTableSort, EvaluationState, ExportGeometry,
        GeneratedNodeSource, Geometry, GeometryKind, GraphColor, GraphDocument, GraphNode,
        GraphPoint, GraphStyle, HoudiniCubicBezierParquetSchema, HoudiniGeometryRecord,
        HoudiniGeometrySchema, LayerKind, NodeEvaluation, NodeKind, NodeParameter,
        NodeParameterKind, NodeStatus, RerunSceneDebugItem, RerunSceneItem, SourceProvenance,
        ViewerGeometry, load_cubic_bezier_parquet, load_cubic_bezier_parquet_with_metadata,
    };
    use std::sync::Arc;

    use arrow::array::Float64Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use parquet::arrow::ArrowWriter;

    #[test]
    fn sample_curve_is_native_cubic_with_four_points() {
        let graph = GraphDocument::sample();
        let curves = graph
            .geometry
            .iter()
            .filter_map(|geometry| match geometry {
                Geometry::CubicBezier(curve) => Some(*curve),
                Geometry::Polygon(_) => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(curves.len(), graph.cubic_bezier_count());
        assert!(curves.iter().all(|curve| curve.control_points().len() == 4));
        assert_eq!(graph.cubic_control_point_count(), curves.len() * 4);
    }

    #[test]
    fn viewer_output_keeps_cubic_bezier_native() {
        let graph = GraphDocument::sample();
        let output = graph.viewer_output();

        assert!(
            output
                .items
                .iter()
                .any(|item| matches!(item.geometry, ViewerGeometry::CubicBezier(_)))
        );
    }

    #[test]
    fn rerun_scene_output_keeps_native_cubic_and_marks_boundary_debug() {
        let graph = GraphDocument::sample();
        let scene = graph.rerun_scene_output();

        assert!(
            scene
                .items
                .iter()
                .any(|item| matches!(item, RerunSceneItem::NativeCubicBezier { .. }))
        );
        assert!(scene.debug_items.iter().any(|item| {
            matches!(item, RerunSceneDebugItem::PreparedExportPolyline(points) if points.len() == graph.export_segments() + 1)
        }));
        assert!(scene.debug_items.iter().any(|item| {
            matches!(item, RerunSceneDebugItem::NativeCubicControlPolygon(points) if points.len() == 4)
        }));
        assert!(scene.query_bridge.is_none());
    }

    #[test]
    fn rerun_scene_output_for_view_can_skip_debug_boundary_geometry() {
        let graph = GraphDocument::sample();
        let scene = graph.rerun_scene_output_for_view(None, false);

        assert!(
            scene
                .items
                .iter()
                .any(|item| matches!(item, RerunSceneItem::NativeCubicBezier { .. }))
        );
        assert!(scene.debug_items.is_empty());
    }

    #[test]
    fn prepared_export_point_count_does_not_require_export_geometry_materialization() {
        let graph = GraphDocument::sample();
        let materialized_count = graph
            .adaptive_export_output()
            .items
            .iter()
            .map(|geometry| match geometry {
                ExportGeometry::Polygon(points) | ExportGeometry::Polyline(points) => points.len(),
            })
            .sum::<usize>();

        assert_eq!(graph.prepared_export_point_count(), materialized_count);
    }

    #[test]
    fn rerun_scene_output_can_be_tagged_with_query_bridge_context() {
        let graph = GraphDocument::sample();
        let bridge = super::RerunQueryBridge {
            mode: super::RerunQueryBridgeMode::ProductForkViewOwned,
            view_id: "view(1234)".to_owned(),
            space_origin: "/".to_owned(),
            timeline: "frame".to_owned(),
            latest_at: 42,
            matching_entity_count: 3,
            visualized_entity_count: 2,
            visible_data_result_count: 1,
        };

        let scene = graph.rerun_scene_output_with_query_bridge(Some(bridge.clone()));

        assert_eq!(scene.query_bridge, Some(bridge));
        assert!(
            scene
                .items
                .iter()
                .any(|item| matches!(item, RerunSceneItem::NativeCubicBezier { .. }))
        );
    }

    #[test]
    fn style_node_reports_graph_owned_defaults() {
        let graph = GraphDocument::sample();
        let style = graph.resolved_style();

        assert_eq!(style, GraphStyle::default());
        assert_eq!(style.color, GraphStyle::default().color);
        assert_eq!(style.opacity, 0.55);
        assert_eq!(style.stroke_scale, 0.75);

        let info = graph
            .selected_node_info(2)
            .expect("sample graph should include style node");
        assert_eq!(info.style, Some(style));
        assert_eq!(info.parameter.name, "Stroke scale");
    }

    #[test]
    fn rerun_scene_output_carries_graph_owned_style_metadata() {
        let mut graph = GraphDocument::sample();
        graph.style = GraphStyle {
            color: GraphColor {
                r: 12,
                g: 34,
                b: 56,
            },
            opacity: 0.42,
            stroke_scale: 0.75,
        };
        graph
            .nodes
            .iter_mut()
            .find(|node| node.kind == super::NodeKind::Style)
            .expect("sample graph should include style node")
            .parameter
            .value = 0.33;

        let scene = graph.rerun_scene_output();

        assert_eq!(scene.style.color, graph.style.color);
        assert_eq!(scene.style.opacity, graph.style.opacity);
        assert_eq!(scene.style.stroke_scale, 0.33);
        assert!(
            scene
                .items
                .iter()
                .all(|item| item.style().stroke_scale == 0.33)
        );
        assert!(scene.items.iter().any(|item| item.layer_name() == "Curves"));
    }

    #[test]
    fn adaptive_export_is_only_a_boundary_representation() {
        let mut graph = GraphDocument::sample();
        graph
            .nodes
            .iter_mut()
            .find(|node| node.name == "Rerun Output")
            .expect("sample graph should include output node")
            .parameter
            .value = 0.43;
        let output = graph.adaptive_export_output();

        assert!(output.items.iter().any(|geometry| match geometry {
            ExportGeometry::Polyline(points) => points.len() == 9,
            ExportGeometry::Polygon(_) => false,
        }));
        assert_eq!(
            graph.cubic_control_point_count(),
            graph.cubic_bezier_count() * 4
        );
    }

    #[test]
    fn output_node_controls_export_segments_not_native_curve() {
        let mut graph = GraphDocument::sample();

        graph
            .nodes
            .iter_mut()
            .find(|node| node.name == "Rerun Output")
            .expect("sample graph should include output node")
            .parameter
            .value = 0.0;
        assert_eq!(graph.export_segments(), 2);

        graph
            .nodes
            .iter_mut()
            .find(|node| node.name == "Rerun Output")
            .expect("sample graph should include output node")
            .parameter
            .value = 1.0;
        assert_eq!(graph.export_segments(), 16);
        assert_eq!(
            graph.cubic_control_point_count(),
            graph.cubic_bezier_count() * 4
        );
    }

    #[test]
    fn synthetic_render_benchmark_loads_native_geometry() {
        let mut graph = GraphDocument::sample();

        let summary = graph.load_synthetic_render_benchmark(128, 16);
        let scene = graph.rerun_scene_output();

        assert_eq!(
            graph.source.metadata.provenance,
            SourceProvenance::SyntheticBenchmark
        );
        assert_eq!(summary.native_cubic_bezier_count, 128);
        assert_eq!(summary.polygon_count, 16);
        assert_eq!(scene.native_cubic_bezier_count(), 128);
        assert_eq!(scene.polygon_count(), 16);
        assert!(
            graph
                .recording_geometry
                .iter()
                .any(|geometry| matches!(geometry, Geometry::CubicBezier(_)))
        );
        assert_eq!(graph.cubic_control_point_count(), 128 * 4);
    }

    #[test]
    fn render_feasibility_summary_labels_boundary_debug_points() {
        let mut graph = GraphDocument::sample();
        let summary = graph.load_synthetic_render_benchmark(10, 2);

        assert_eq!(summary.native_viewer_primitive_count, 12);
        assert_eq!(summary.graph_owned_point_count, 10 * 4 + 2 * 4);
        assert_eq!(
            summary.prepared_boundary_debug_point_count,
            10 * (summary.export_segments_per_cubic + 1) + 10 * 4
        );
        assert!(summary.prepared_boundary_debug_point_count > summary.graph_owned_point_count);
    }

    #[test]
    fn selected_node_info_reports_pipeline_counts() {
        let graph = GraphDocument::sample();

        let source = graph
            .selected_node_info(0)
            .expect("sample graph should include source node");
        assert_eq!(source.input_count, 0);
        assert_eq!(source.output_count, 4);
        assert_eq!(source.status, NodeStatus::Healthy);
        assert_eq!(source.data_kind, "Source geometry");
        assert_eq!(source.record_count, 4);
        assert_eq!(source.provenance, Some(SourceProvenance::DemoFallback));
        assert_eq!(source.attributes, vec!["score".to_owned()]);
        assert!(source.bounds.is_some());
        let source_metadata = source
            .source_metadata
            .expect("source node should report source metadata");
        assert_eq!(source_metadata.provenance, SourceProvenance::DemoFallback);
        assert_eq!(source_metadata.record_count, 4);
        assert_eq!(source_metadata.polygon_count, 2);
        assert_eq!(source_metadata.cubic_bezier_count, 2);
        assert!(source_metadata.bounds.is_some());

        let filter = graph
            .selected_node_info(1)
            .expect("sample graph should include filter node");
        assert_eq!(filter.input_count, 4);
        assert_eq!(filter.output_count, 2);
        assert_eq!(filter.role, "Cull");
        assert_eq!(filter.parameter.name, "Minimum score");
        assert_eq!(filter.parameter.kind, NodeParameterKind::AttributeRule);
        assert_eq!(filter.parameter.value, 0.55);
        assert_eq!(filter.status, NodeStatus::Healthy);
        assert_eq!(filter.data_kind, "Filtered geometry");
        assert_eq!(filter.record_count, 2);
        assert!(filter.bounds.is_some());
        assert!(filter.warnings.is_empty());

        let output = graph
            .selected_node_info(3)
            .expect("sample graph should include output node");
        assert_eq!(output.output_count, 2);
    }

    #[test]
    fn filter_node_models_score_threshold_as_typed_attribute_rule() {
        let graph = GraphDocument::sample();
        let rule = graph
            .filter_rule()
            .expect("sample graph should include filter rule");

        assert_eq!(rule.attribute_name, "score");
        assert_eq!(rule.comparison, super::FilterComparison::GreaterOrEqual);
        assert_eq!(rule.value.as_f32(), Some(0.55));
        assert_eq!(graph.visible_output_count(), 2);
    }

    #[test]
    fn invalid_filter_attribute_reports_warning_and_emits_no_output() {
        let mut graph = GraphDocument::sample();
        let filter = graph
            .nodes
            .iter_mut()
            .find(|node| node.kind == super::NodeKind::Filter)
            .expect("sample graph should include filter node");
        filter
            .parameter
            .rule_spec
            .as_mut()
            .expect("filter node should have rule spec")
            .attribute_name = "missing_attribute".to_owned();

        assert_eq!(graph.visible_output_count(), 0);
        let info = graph
            .selected_node_info(1)
            .expect("sample graph should include filter node");
        assert_eq!(info.status, NodeStatus::Warning);
        assert_eq!(info.record_count, 0);
        assert!(info.bounds.is_none());
        assert_eq!(info.warnings.len(), 1);
        assert!(info.warnings[0].contains("Unknown filter attribute"));
    }

    #[test]
    fn invalid_style_parameter_reports_node_warning() {
        let mut graph = GraphDocument::sample();
        graph.style.opacity = 1.4;

        let info = graph
            .selected_node_info(2)
            .expect("sample graph should include style node");

        assert_eq!(info.status, NodeStatus::Warning);
        assert_eq!(info.record_count, 2);
        assert!(info.bounds.is_some());
        assert_eq!(info.warnings.len(), 1);
        assert!(info.warnings[0].contains("Style opacity"));
    }

    #[test]
    fn pipeline_stages_report_execution_trace() {
        let graph = GraphDocument::sample();
        let stages = graph.pipeline_stages();

        assert_eq!(stages.len(), 4);
        assert_eq!(stages[0].name, "Source");
        assert_eq!(stages[0].output_count, 4);
        assert_eq!(stages[1].name, "Filter");
        assert_eq!(stages[1].input_count, 4);
        assert_eq!(stages[1].output_count, 2);
        assert_eq!(stages[3].name, "Rerun Output");
        assert_eq!(stages[3].output_count, graph.visible_output_count());
    }

    #[test]
    fn attribute_table_rows_report_visible_native_output_records() {
        let graph = GraphDocument::sample();
        let rows = graph.attribute_table_rows(&AttributeTableQuery::default());

        assert_eq!(rows.len(), graph.visible_output_count());
        assert_eq!(rows[0].record_index, 0);
        assert_eq!(rows[0].geometry_kind, GeometryKind::Polygon);
        assert_eq!(rows[0].layer, LayerKind::Polygons);
        assert_eq!(rows[0].score, 0.62);
        assert_eq!(rows[0].point_count, 4);
        assert_eq!(rows[0].provenance, SourceProvenance::DemoFallback);
        assert!(rows[0].source_path.is_none());

        let cubic_row = rows
            .iter()
            .find(|row| row.geometry_kind == GeometryKind::CubicBezier)
            .expect("sample visible output should include a cubic row");
        assert!(cubic_row.is_native_cubic_bezier);
        assert_eq!(cubic_row.point_count, 4);
    }

    #[test]
    fn attribute_table_query_filters_searches_and_sorts_locally() {
        let graph = GraphDocument::sample();
        let rows = graph.attribute_table_rows(&AttributeTableQuery {
            search: "curves".to_owned(),
            minimum_score: Some(0.8),
            sort: AttributeTableSort::Score,
            sort_descending: true,
        });

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].geometry_kind, GeometryKind::CubicBezier);
        assert_eq!(rows[0].layer, LayerKind::Curves);
        assert_eq!(rows[0].score, 0.82);
        assert_eq!(graph.visible_output_count(), 2);
        assert!(
            graph
                .viewer_output()
                .items
                .iter()
                .any(|item| matches!(item.geometry, ViewerGeometry::CubicBezier(_)))
        );
    }

    #[test]
    fn attribute_table_filter_stays_transient_until_committed() {
        let graph = GraphDocument::sample();
        let rows = graph.attribute_table_rows(&AttributeTableQuery {
            search: String::new(),
            minimum_score: Some(0.8),
            sort: AttributeTableSort::RecordIndex,
            sort_descending: false,
        });

        assert_eq!(rows.len(), 1);
        assert_eq!(graph.visible_output_count(), 2);
        assert_eq!(
            graph
                .filter_rule()
                .expect("sample graph should include filter rule")
                .value
                .as_f32(),
            Some(0.55)
        );
    }

    #[test]
    fn committed_attribute_table_filter_updates_graph_filter_node() {
        let mut graph = GraphDocument::sample();
        let committed = graph.commit_attribute_table_query_as_filter(&AttributeTableQuery {
            search: String::new(),
            minimum_score: Some(0.8),
            sort: AttributeTableSort::RecordIndex,
            sort_descending: false,
        });

        assert!(committed);
        let rule = graph
            .filter_rule()
            .expect("committed table filter should become graph filter rule");
        assert_eq!(rule.attribute_name, "score");
        assert_eq!(rule.comparison, super::FilterComparison::GreaterOrEqual);
        assert_eq!(rule.value.as_f32(), Some(0.8));
        assert_eq!(graph.visible_output_count(), 1);
        let filter_node = graph
            .nodes
            .iter()
            .find(|node| node.kind == NodeKind::Filter)
            .expect("sample graph should include filter node");
        assert_eq!(
            filter_node
                .generated
                .expect("filter should be generated")
                .source,
            GeneratedNodeSource::AttributeTableCommit
        );
        assert!(filter_node.layout_position.y >= 0.8);
        assert_eq!(
            graph
                .selected_node_info(1)
                .expect("filter node info should exist")
                .generated
                .expect("filter node info should expose generated metadata")
                .source,
            GeneratedNodeSource::AttributeTableCommit
        );
    }

    #[test]
    fn generated_filter_node_remains_editable_as_graph_data() {
        let mut graph = GraphDocument::sample();
        assert!(
            graph.commit_attribute_table_query_as_filter(&AttributeTableQuery {
                search: String::new(),
                minimum_score: Some(0.8),
                sort: AttributeTableSort::RecordIndex,
                sort_descending: false,
            })
        );

        graph
            .nodes
            .iter_mut()
            .find(|node| node.kind == NodeKind::Filter)
            .expect("generated filter should remain an editable graph node")
            .parameter
            .value = 0.6;

        assert_eq!(
            graph
                .filter_rule()
                .expect("generated filter should still expose typed rule")
                .value
                .as_f32(),
            Some(0.6)
        );
        assert_eq!(graph.visible_output_count(), 2);
    }

    #[test]
    fn committed_attribute_table_filter_round_trips_through_sidecar() {
        let mut graph = GraphDocument::sample();
        assert!(
            graph.commit_attribute_table_query_as_filter(&AttributeTableQuery {
                search: String::new(),
                minimum_score: Some(0.8),
                sort: AttributeTableSort::RecordIndex,
                sort_descending: false,
            },)
        );

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        let rule = restored
            .filter_rule()
            .expect("restored graph should include committed table filter");
        assert_eq!(rule.attribute_name, "score");
        assert_eq!(rule.value.as_f32(), Some(0.8));
        let restored_filter = restored
            .nodes
            .iter()
            .find(|node| node.kind == NodeKind::Filter)
            .expect("restored graph should include filter node");
        assert_eq!(
            restored_filter
                .generated
                .expect("generated filter metadata should round trip")
                .source,
            GeneratedNodeSource::AttributeTableCommit
        );
        assert!(restored_filter.layout_position.y >= 0.8);
        assert_eq!(restored.visible_output_count(), 1);
    }

    #[test]
    fn graph_layout_reports_model_owned_node_positions_and_edges() {
        let graph = GraphDocument::sample();
        let layout = graph.graph_layout();

        assert_eq!(layout.nodes.len(), graph.nodes.len());
        assert_eq!(layout.edges.len(), graph.nodes.len() - 1);
        assert_eq!(layout.nodes[0].node_index, 0);
        assert_eq!(layout.nodes[0].name, "Source");
        assert_eq!(layout.nodes[0].position.x, 0.0);
        assert_eq!(layout.nodes[0].position.y, 0.5);
        assert_eq!(layout.nodes[3].name, "Rerun Output");
        assert_eq!(layout.nodes[3].position.x, 1.0);
        assert_eq!(layout.edges[0].from_node, 0);
        assert_eq!(layout.edges[0].to_node, 1);
        assert_eq!(layout.edges[2].from_node, 2);
        assert_eq!(layout.edges[2].to_node, 3);
    }

    #[test]
    fn output_demand_evaluates_stale_connected_nodes_only() {
        let mut graph = GraphDocument::sample();
        graph.nodes.push(GraphNode {
            name: "Scratch Filter",
            kind: NodeKind::Filter,
            layout_position: GraphPoint::new(0.5, 0.1),
            generated: None,
            evaluation: NodeEvaluation {
                state: EvaluationState::Stale,
                manual: false,
                message: None,
            },
            participates_in_output: false,
            parameter: NodeParameter::attribute_rule(
                "Scratch score",
                "score",
                super::FilterComparison::GreaterOrEqual,
                0.2,
                0.0..=1.0,
                "Disconnected exploratory filter.",
            ),
            info: "Disconnected exploratory filter.",
        });
        graph.mark_node_stale(1);

        graph.demand_output_evaluation();

        assert_eq!(graph.nodes[1].evaluation.state, EvaluationState::Cached);
        assert_eq!(
            graph
                .nodes
                .last()
                .expect("scratch node should exist")
                .evaluation
                .state,
            EvaluationState::Stale
        );
        assert_eq!(graph.graph_layout().edges.len(), graph.nodes.len() - 2);
    }

    #[test]
    fn manual_node_waits_for_explicit_run() {
        let mut graph = GraphDocument::sample();
        graph.set_node_manual(1, true);

        graph.demand_output_evaluation();

        assert_eq!(graph.nodes[1].evaluation.state, EvaluationState::Manual);
        graph.request_node_run(1);
        assert_eq!(graph.nodes[1].evaluation.state, EvaluationState::Running);
        graph.complete_node_run(1);
        assert_eq!(graph.nodes[1].evaluation.state, EvaluationState::Clean);
        assert!(!graph.nodes[1].evaluation.manual);
    }

    #[test]
    fn failed_node_supports_cancel_and_retry_transitions() {
        let mut graph = GraphDocument::sample();
        graph.fail_node_run(1, "bad attribute");

        let info = graph
            .selected_node_info(1)
            .expect("filter node info should exist");
        assert_eq!(info.evaluation.state, EvaluationState::Failed);
        assert_eq!(info.evaluation.message.as_deref(), Some("bad attribute"));

        graph.request_node_run(1);
        assert_eq!(graph.nodes[1].evaluation.state, EvaluationState::Running);
        graph.cancel_node_run(1);
        assert_eq!(graph.nodes[1].evaluation.state, EvaluationState::Manual);
        assert_eq!(
            graph.nodes[1].evaluation.message.as_deref(),
            Some("Run cancelled")
        );

        graph.request_node_run(1);
        graph.complete_node_run(1);
        assert_eq!(graph.nodes[1].evaluation.state, EvaluationState::Clean);
        assert!(graph.nodes[1].evaluation.message.is_none());
    }

    #[test]
    fn graph_layout_node_positions_are_editable_and_clamped() {
        let mut graph = GraphDocument::sample();

        graph.set_node_layout_position(1, GraphPoint::new(0.25, 0.75));
        let layout = graph.graph_layout();
        assert_eq!(layout.nodes[1].position, GraphPoint::new(0.25, 0.75));

        graph.set_node_layout_position(1, GraphPoint::new(-1.0, 2.0));
        let layout = graph.graph_layout();
        assert_eq!(layout.nodes[1].position, GraphPoint::new(0.0, 1.0));
    }

    #[test]
    fn layer_visibility_and_filter_threshold_control_output() {
        let mut graph = GraphDocument::sample();
        assert_eq!(graph.visible_output_count(), 2);

        graph
            .layers
            .iter_mut()
            .find(|layer| layer.kind == LayerKind::Curves)
            .expect("sample graph should include curve layer")
            .visible = false;
        assert_eq!(graph.visible_output_count(), 1);

        graph
            .nodes
            .iter_mut()
            .find(|node| node.name == "Filter")
            .expect("sample graph should include filter node")
            .parameter
            .value = 0.9;
        assert_eq!(graph.visible_output_count(), 0);
    }

    #[test]
    fn layer_order_controls_viewer_output_order() {
        let mut graph = GraphDocument::sample();
        graph
            .layers
            .iter_mut()
            .find(|layer| layer.kind == LayerKind::Polygons)
            .expect("sample graph should include polygon layer")
            .order = 10;
        graph
            .layers
            .iter_mut()
            .find(|layer| layer.kind == LayerKind::Curves)
            .expect("sample graph should include curve layer")
            .order = 0;

        let output = graph.viewer_output();

        assert_eq!(output.items.len(), 2);
        assert_eq!(output.items[0].layer.kind, LayerKind::Curves);
        assert_eq!(output.items[0].layer.order, 0);
        assert_eq!(output.items[1].layer.kind, LayerKind::Polygons);
        assert_eq!(output.items[1].layer.order, 10);
    }

    #[test]
    fn duplicate_layer_views_emit_same_native_output_with_distinct_metadata() {
        let mut graph = GraphDocument::sample();
        assert!(graph.duplicate_layer_view(LayerKind::Curves, "Curves Copy"));

        let scene = graph.rerun_scene_output();
        let curve_items = scene
            .items
            .iter()
            .filter_map(|item| match item {
                RerunSceneItem::NativeCubicBezier {
                    layer_name,
                    layer_order,
                    ..
                } => Some((layer_name.as_str(), *layer_order)),
                RerunSceneItem::Polygon { .. } => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(curve_items.len(), 2);
        assert!(curve_items.contains(&("Curves", 1)));
        assert!(curve_items.contains(&("Curves Copy", 3)));
        assert!(
            scene
                .items
                .iter()
                .any(|item| matches!(item, RerunSceneItem::NativeCubicBezier { .. }))
        );
    }

    #[test]
    fn layer_views_round_trip_through_sidecar() {
        let mut graph = GraphDocument::sample();
        assert!(graph.duplicate_layer_view(LayerKind::Curves, "Curves Copy"));
        let duplicate = graph
            .layers
            .iter_mut()
            .find(|layer| layer.name == "Curves Copy")
            .expect("duplicated layer view should exist");
        duplicate.order = -2;
        duplicate.visible = false;
        duplicate.style = GraphStyle {
            color: GraphColor { r: 7, g: 8, b: 9 },
            opacity: 0.4,
            stroke_scale: 0.6,
        };

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        let restored_duplicate = restored
            .layers
            .iter()
            .find(|layer| layer.name == "Curves Copy")
            .expect("duplicated layer view should round trip");
        assert_eq!(restored.layers.len(), 4);
        assert_eq!(restored_duplicate.kind, LayerKind::Curves);
        assert_eq!(restored_duplicate.order, -2);
        assert!(!restored_duplicate.visible);
        assert_eq!(
            restored_duplicate.style,
            GraphStyle {
                color: GraphColor { r: 7, g: 8, b: 9 },
                opacity: 0.4,
                stroke_scale: 0.6,
            }
        );
    }

    #[test]
    fn recording_query_source_disables_demo_geometry_until_native_geometry_is_imported() {
        let mut graph = GraphDocument::sample();
        let bridge = super::RerunQueryBridge {
            mode: super::RerunQueryBridgeMode::ProductForkViewOwned,
            view_id: "view(1234)".to_owned(),
            space_origin: "/".to_owned(),
            timeline: "frame".to_owned(),
            latest_at: 42,
            matching_entity_count: 5,
            visualized_entity_count: 5,
            visible_data_result_count: 5,
        };

        graph.update_source_from_query_bridge(&bridge);

        assert_eq!(graph.source.mode, super::GraphSourceMode::RecordingQuery);
        assert_eq!(graph.visible_output_count(), 0);
        assert_eq!(graph.polygon_count(), 0);
        assert_eq!(graph.cubic_bezier_count(), 0);
    }

    #[test]
    fn empty_query_keeps_demo_geometry_fallback() {
        let mut graph = GraphDocument::sample();
        let bridge = super::RerunQueryBridge {
            mode: super::RerunQueryBridgeMode::ProductForkViewOwned,
            view_id: "view(1234)".to_owned(),
            space_origin: "/".to_owned(),
            timeline: "frame".to_owned(),
            latest_at: 42,
            matching_entity_count: 0,
            visualized_entity_count: 0,
            visible_data_result_count: 0,
        };

        graph.update_source_from_query_bridge(&bridge);

        assert_eq!(graph.source.mode, super::GraphSourceMode::DemoFallback);
        assert_eq!(graph.visible_output_count(), 2);
    }

    #[test]
    fn houdini_geometry_schema_names_native_geometry_without_polyline_storage() {
        assert_eq!(
            HoudiniGeometrySchema::ARCHETYPE_NAME,
            "vy.houdini.Geometry2D"
        );
        assert!(HoudiniGeometrySchema::component_names().contains(&"HoudiniGeometry2D:points"));
        assert!(
            !HoudiniGeometrySchema::component_names()
                .iter()
                .any(|component| component.contains("polyline"))
        );
    }

    #[test]
    fn imported_recording_geometry_preserves_native_cubic_bezier() {
        let mut graph = GraphDocument::sample();
        let bridge = super::RerunQueryBridge {
            mode: super::RerunQueryBridgeMode::ProductForkViewOwned,
            view_id: "view(1234)".to_owned(),
            space_origin: "/".to_owned(),
            timeline: "frame".to_owned(),
            latest_at: 42,
            matching_entity_count: 1,
            visualized_entity_count: 1,
            visible_data_result_count: 1,
        };
        let curve = super::CubicBezier {
            start: GraphPoint::new(0.0, 0.0),
            control_1: GraphPoint::new(0.2, 0.8),
            control_2: GraphPoint::new(0.8, 0.2),
            end: GraphPoint::new(1.0, 1.0),
            score: 0.9,
        };

        graph.import_recording_geometry(
            &bridge,
            [HoudiniGeometryRecord::cubic_bezier(
                LayerKind::Curves,
                curve,
            )],
        );

        assert_eq!(graph.source.mode, super::GraphSourceMode::RecordingQuery);
        assert_eq!(graph.cubic_bezier_count(), 1);
        assert!(
            graph
                .viewer_output()
                .items
                .iter()
                .any(|item| matches!(item.geometry, ViewerGeometry::CubicBezier(_)))
        );
    }

    #[test]
    fn cubic_bezier_parquet_loader_builds_native_curves_from_eight_columns() {
        let parquet_file = write_cubic_bezier_parquet(&[
            ("cp0_x", vec![0.0, 0.1]),
            ("cp0_y", vec![0.0, 0.2]),
            ("cp1_x", vec![0.2, 0.3]),
            ("cp1_y", vec![0.8, 0.9]),
            ("cp2_x", vec![0.8, 0.7]),
            ("cp2_y", vec![0.2, 0.1]),
            ("cp3_x", vec![1.0, 0.9]),
            ("cp3_y", vec![1.0, 0.8]),
        ]);

        let records = load_cubic_bezier_parquet(parquet_file.path()).unwrap();

        assert_eq!(HoudiniCubicBezierParquetSchema::required_column_count(), 8);
        assert_eq!(records.len(), 2);
        assert!(
            records
                .iter()
                .all(|record| matches!(record.geometry, Geometry::CubicBezier(_)))
        );
        let Geometry::CubicBezier(curve) = records[0].geometry.clone() else {
            panic!("expected native cubic Bezier");
        };
        assert_eq!(curve.control_points()[0], GraphPoint::new(0.0, 0.0));
        assert_eq!(curve.control_points()[1], GraphPoint::new(0.2, 0.8));
        assert_eq!(curve.control_points()[2], GraphPoint::new(0.8, 0.2));
        assert_eq!(curve.control_points()[3], GraphPoint::new(1.0, 1.0));
    }

    #[test]
    fn cubic_bezier_parquet_loader_requires_all_control_point_columns() {
        let parquet_file = write_cubic_bezier_parquet(&[
            ("cp0_x", vec![0.0]),
            ("cp0_y", vec![0.0]),
            ("cp1_x", vec![0.2]),
            ("cp1_y", vec![0.8]),
            ("cp2_x", vec![0.8]),
            ("cp2_y", vec![0.2]),
            ("cp3_x", vec![1.0]),
        ]);

        let err = match load_cubic_bezier_parquet(parquet_file.path()) {
            Ok(_) => panic!("missing control-point column should fail"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("cp3_y"));
    }

    #[test]
    fn graph_document_can_import_cubic_bezier_parquet_at_load_boundary() {
        let mut graph = GraphDocument::sample();
        let bridge = super::RerunQueryBridge {
            mode: super::RerunQueryBridgeMode::ProductForkViewOwned,
            view_id: "view(1234)".to_owned(),
            space_origin: "/".to_owned(),
            timeline: "frame".to_owned(),
            latest_at: 42,
            matching_entity_count: 1,
            visualized_entity_count: 1,
            visible_data_result_count: 1,
        };
        let parquet_file = write_cubic_bezier_parquet(&[
            ("cp0_x", vec![0.0]),
            ("cp0_y", vec![0.0]),
            ("cp1_x", vec![0.2]),
            ("cp1_y", vec![0.8]),
            ("cp2_x", vec![0.8]),
            ("cp2_y", vec![0.2]),
            ("cp3_x", vec![1.0]),
            ("cp3_y", vec![1.0]),
        ]);

        let imported = graph
            .import_cubic_bezier_parquet(&bridge, parquet_file.path())
            .unwrap();

        assert_eq!(imported, 1);
        assert_eq!(graph.source.mode, super::GraphSourceMode::RecordingQuery);
        assert_eq!(
            graph.source.metadata.provenance,
            SourceProvenance::ParquetImport
        );
        assert_eq!(graph.source.metadata.record_count, 1);
        assert_eq!(graph.source.metadata.cubic_bezier_count, 1);
        assert!(graph.source.metadata.bounds.is_some());
        assert_eq!(graph.cubic_bezier_count(), 1);
        assert_eq!(graph.visible_output_count(), 1);
    }

    #[test]
    fn graph_document_can_import_cubic_bezier_parquet_path_without_query_bridge() {
        let mut graph = GraphDocument::sample();
        let parquet_file = write_cubic_bezier_parquet(&[
            ("cp0_x", vec![0.0]),
            ("cp0_y", vec![0.0]),
            ("cp1_x", vec![0.2]),
            ("cp1_y", vec![0.8]),
            ("cp2_x", vec![0.8]),
            ("cp2_y", vec![0.2]),
            ("cp3_x", vec![1.0]),
            ("cp3_y", vec![1.0]),
        ]);

        let imported = graph
            .import_cubic_bezier_parquet_path(parquet_file.path())
            .unwrap();

        assert_eq!(imported, 1);
        assert_eq!(graph.source.mode, super::GraphSourceMode::RecordingQuery);
        assert_eq!(graph.source.matching_entity_count, 1);
        assert_eq!(
            graph.source.metadata.provenance,
            SourceProvenance::ParquetImport
        );
        assert_eq!(graph.source.metadata.record_count, 1);
        assert_eq!(graph.source.metadata.cubic_bezier_count, 1);
        assert!(graph.source.metadata.bounds.is_some());
        assert_eq!(graph.visible_output_count(), 1);
    }

    #[test]
    fn checked_in_sample_parquet_loads_native_cubic_beziers() {
        let sample_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/houdini_cubic_sample.parquet");

        let records = load_cubic_bezier_parquet(sample_path).unwrap();

        assert_eq!(records.len(), 4);
        assert!(
            records
                .iter()
                .all(|record| matches!(record.geometry, Geometry::CubicBezier(_)))
        );
    }

    #[test]
    fn checked_in_sample_parquet_reports_source_metadata() {
        let sample_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/houdini_cubic_sample.parquet");

        let load = load_cubic_bezier_parquet_with_metadata(&sample_path).unwrap();

        assert_eq!(load.records.len(), 4);
        assert_eq!(load.metadata.provenance, SourceProvenance::ParquetImport);
        assert_eq!(
            load.metadata.source_path,
            Some(sample_path.display().to_string())
        );
        assert_eq!(load.metadata.record_count, 4);
        assert_eq!(load.metadata.polygon_count, 0);
        assert_eq!(load.metadata.cubic_bezier_count, 4);
        assert_eq!(
            load.metadata.recognized_control_point_columns,
            HoudiniCubicBezierParquetSchema::CONTROL_POINT_COLUMNS
                .iter()
                .map(|name| (*name).to_owned())
                .collect::<Vec<_>>()
        );
        assert_eq!(load.metadata.attribute_names, vec!["score".to_owned()]);
        assert!(load.metadata.bounds.is_some());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn checked_in_sample_parquet_can_write_durable_rerun_recording() {
        let sample_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/houdini_cubic_sample.parquet");
        let recording_dir = tempfile::tempdir().unwrap();
        let recording_path = recording_dir.path().join("houdini-graph-output.rrd");
        let mut graph = GraphDocument::sample();

        let imported = graph
            .import_cubic_bezier_parquet_path(&sample_path)
            .unwrap();
        let recording = graph.save_rerun_recording(&recording_path).unwrap();

        assert_eq!(imported, 4);
        assert_eq!(recording.path, recording_path);
        assert_eq!(recording.item_count, 4);
        assert_eq!(recording.polygon_count, 0);
        assert_eq!(recording.native_cubic_bezier_count, 4);
        assert!(
            recording
                .limitation_note
                .contains("cubic Bezier semantics as graph-owned control-point metadata")
        );
        assert!(recording_path.exists());
        assert!(std::fs::metadata(&recording_path).unwrap().len() > 0);
    }

    #[test]
    fn malformed_parquet_import_records_source_error_without_replacing_geometry() {
        let mut graph = GraphDocument::sample();
        let previous_cubic_count = graph.cubic_bezier_count();
        let parquet_file = write_cubic_bezier_parquet(&[
            ("cp0_x", vec![0.0]),
            ("cp0_y", vec![0.0]),
            ("cp1_x", vec![0.2]),
            ("cp1_y", vec![0.8]),
            ("cp2_x", vec![0.8]),
            ("cp2_y", vec![0.2]),
            ("cp3_x", vec![1.0]),
        ]);

        let err = graph
            .import_cubic_bezier_parquet_path(parquet_file.path())
            .expect_err("missing control-point column should fail import");

        assert!(
            err.to_string()
                .contains("Missing Houdini cubic Bezier parquet column")
        );
        assert_eq!(graph.cubic_bezier_count(), previous_cubic_count);
        assert_eq!(
            graph.source.metadata.provenance,
            SourceProvenance::DemoFallback
        );
        assert!(graph.source.import_error.is_some());
        assert_eq!(
            graph.source.source_path,
            Some(parquet_file.path().display().to_string())
        );

        let info = graph
            .selected_node_info(0)
            .expect("sample graph should include source node");
        assert_eq!(info.status, NodeStatus::Failed);
        assert!(info.source_error.is_some());
        assert_eq!(info.provenance, Some(SourceProvenance::DemoFallback));
    }

    #[test]
    fn sidecar_json_round_trips_source_layers_nodes_and_native_geometry() {
        let mut graph = GraphDocument::sample();
        let sample_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/houdini_cubic_sample.parquet");
        graph
            .import_cubic_bezier_parquet_path(&sample_path)
            .unwrap();
        graph.set_node_layout_position(1, GraphPoint::new(0.25, 0.75));
        graph
            .nodes
            .iter_mut()
            .find(|node| node.kind == super::NodeKind::Filter)
            .expect("sample graph should include filter node")
            .parameter
            .value = 0.25;
        graph
            .nodes
            .iter_mut()
            .find(|node| node.kind == super::NodeKind::Filter)
            .expect("sample graph should include filter node")
            .parameter
            .rule_spec
            .as_mut()
            .expect("filter node should have rule spec")
            .comparison = super::FilterComparison::LessOrEqual;
        graph.style = GraphStyle {
            color: GraphColor {
                r: 22,
                g: 44,
                b: 66,
            },
            opacity: 0.38,
            stroke_scale: 0.75,
        };
        graph
            .nodes
            .iter_mut()
            .find(|node| node.kind == super::NodeKind::Style)
            .expect("sample graph should include style node")
            .parameter
            .value = 0.63;
        graph
            .layers
            .iter_mut()
            .find(|layer| layer.kind == LayerKind::Curves)
            .expect("sample graph should include curve layer")
            .visible = false;

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(restored.source.mode, super::GraphSourceMode::RecordingQuery);
        assert_eq!(
            restored.source.source_path,
            Some(sample_path.display().to_string())
        );
        assert_eq!(
            restored.source.metadata.provenance,
            SourceProvenance::ParquetImport
        );
        assert_eq!(restored.source.metadata.record_count, 4);
        assert_eq!(restored.source.metadata.cubic_bezier_count, 4);
        assert_eq!(restored.cubic_bezier_count(), 4);
        assert!(!restored.layer_visible(LayerKind::Curves));
        assert_eq!(
            restored
                .graph_layout()
                .nodes
                .iter()
                .find(|node| node.node_index == 1)
                .expect("filter node layout should exist")
                .position,
            GraphPoint::new(0.25, 0.75)
        );
        assert_eq!(restored.filter_minimum_score(), 0.25);
        assert_eq!(
            restored
                .filter_rule()
                .expect("restored graph should include filter rule")
                .comparison,
            super::FilterComparison::LessOrEqual
        );
        assert_eq!(
            restored.resolved_style(),
            GraphStyle {
                color: GraphColor {
                    r: 22,
                    g: 44,
                    b: 66,
                },
                opacity: 0.38,
                stroke_scale: 0.63,
            }
        );
    }

    #[test]
    fn sidecar_json_does_not_persist_adaptive_export_polyline() {
        let mut graph = GraphDocument::sample();
        graph
            .nodes
            .iter_mut()
            .find(|node| node.kind == super::NodeKind::Output)
            .expect("sample graph should include output node")
            .parameter
            .value = 1.0;
        assert!(
            graph
                .adaptive_export_output()
                .items
                .iter()
                .any(|geometry| matches!(geometry, ExportGeometry::Polyline(_)))
        );

        let json = graph.to_sidecar_json().unwrap();

        assert!(!json.contains("Polyline"));
        assert!(!json.contains("PreparedExportPolyline"));
        assert!(json.contains("CubicBezier"));
    }

    fn write_cubic_bezier_parquet(columns: &[(&str, Vec<f64>)]) -> tempfile::NamedTempFile {
        let parquet_file = tempfile::NamedTempFile::new().unwrap();
        let schema = Arc::new(Schema::new(
            columns
                .iter()
                .map(|(name, _values)| Field::new(*name, DataType::Float64, false))
                .collect::<Vec<_>>(),
        ));
        let arrays = columns
            .iter()
            .map(|(_name, values)| Arc::new(Float64Array::from(values.clone())) as _)
            .collect::<Vec<_>>();
        let batch = RecordBatch::try_new(schema.clone(), arrays).unwrap();
        let file = parquet_file.reopen().unwrap();
        let mut writer = ArrowWriter::try_new(file, schema, None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();

        parquet_file
    }
}
