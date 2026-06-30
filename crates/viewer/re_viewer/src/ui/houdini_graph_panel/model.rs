use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

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
}

pub(crate) struct GraphDocument {
    pub source: GraphSource,
    pub nodes: Vec<GraphNode>,
    pub annotations: Vec<GraphAnnotation>,
    pub network_view: NetworkViewDisplayOptions,
    pub layers: Vec<Layer>,
    pub style: GraphStyle,
    pub geometry: Vec<Geometry>,
    pub recording_geometry: Vec<Geometry>,
    pub python_operator_declarations: Vec<PythonOperatorDeclaration>,
    pub procedural_asset_declarations: Vec<ProceduralAssetDeclaration>,
    pub native_operator_declarations: Vec<NativeOperatorDeclaration>,
    pub native_operator_trust: NativeOperatorTrustPolicy,
    pub python_environment: PythonEnvironmentDescriptor,
}

const GENERATED_NODE_LANE_Y: f32 = 0.82;
const NATIVE_OPERATOR_HOST_COMPATIBILITY_VERSION: &str = "re_viewer-houdini-graph-0.1";
const MAIN_GRAPH_ID: &str = "main";
const PRIMARY_GEOMETRY_OUTPUT: &str = "geometry";

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
                    node_id: "source.main".to_owned(),
                    name: "Source".to_owned(),
                    kind: NodeKind::Source,
                    layout_position: GraphPoint::new(0.0, 0.5),
                    generated: None,
                    coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    substrate_projection: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    comment: String::new(),
                    show_comment_in_network: false,
                    parameter: NodeParameter::scalar(
                        "Read",
                        1.0,
                        0.0..=1.0,
                        "Source readiness placeholder for the spike graph.",
                    ),
                    info: "Loads polygon and cubic Bezier records.",
                },
                GraphNode {
                    node_id: "filter.main".to_owned(),
                    name: "Filter".to_owned(),
                    kind: NodeKind::Filter,
                    layout_position: GraphPoint::new(0.33, 0.5),
                    generated: None,
                    coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    substrate_projection: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    comment: String::new(),
                    show_comment_in_network: false,
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
                    node_id: "style.main".to_owned(),
                    name: "Style".to_owned(),
                    kind: NodeKind::Style,
                    layout_position: GraphPoint::new(0.66, 0.5),
                    generated: None,
                    coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    substrate_projection: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    comment: String::new(),
                    show_comment_in_network: false,
                    parameter: NodeParameter::scalar(
                        "Stroke scale",
                        0.75,
                        0.0..=1.0,
                        "Controls output stroke scale without mutating native geometry.",
                    ),
                    info: "Assigns visual parameters before viewer output.",
                },
                GraphNode {
                    node_id: "output.rerun".to_owned(),
                    name: "Rerun Output".to_owned(),
                    kind: NodeKind::Output,
                    layout_position: GraphPoint::new(1.0, 0.5),
                    generated: None,
                    coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
                    output_operator: Some(OutputOperatorNode::rerun_scene()),
                    null_operator: None,
                    reference_input: None,
                    substrate_projection: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    comment: String::new(),
                    show_comment_in_network: false,
                    parameter: NodeParameter::scalar(
                        "Adaptive segments",
                        1.0,
                        0.0..=1.0,
                        "Controls only the prepared export polyline. The native cubic remains four points.",
                    ),
                    info: "Prepares adaptive viewer geometry only at the output edge.",
                },
            ],
            annotations: vec![
                GraphAnnotation::network_box(
                    "box.prep".to_owned(),
                    "Prep".to_owned(),
                    GraphPoint::new(0.03, 0.24),
                    GraphPoint::new(0.62, 0.48),
                    vec!["source.main".to_owned(), "filter.main".to_owned()],
                ),
                GraphAnnotation::sticky_note(
                    "note.review".to_owned(),
                    "Review".to_owned(),
                    "Check score cutoff before publishing output.".to_owned(),
                    GraphPoint::new(0.60, 0.12),
                    GraphPoint::new(0.30, 0.28),
                ),
            ],
            network_view: NetworkViewDisplayOptions::default(),
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
            python_operator_declarations: Vec::new(),
            procedural_asset_declarations: Vec::new(),
            native_operator_declarations: Vec::new(),
            native_operator_trust: NativeOperatorTrustPolicy::default(),
            python_environment: PythonEnvironmentDescriptor::default(),
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

    pub fn add_network_box_for_node(&mut self, node_index: usize) -> Option<usize> {
        let node = self.nodes.get(node_index)?;
        let position =
            GraphPoint::new(node.layout_position.x - 0.08, node.layout_position.y - 0.16);
        let annotation = GraphAnnotation::network_box(
            self.unique_annotation_id("box"),
            self.unique_annotation_title("Network Box"),
            position,
            GraphPoint::new(0.22, 0.24),
            vec![node.node_id.clone()],
        );
        self.annotations.push(annotation);
        Some(self.annotations.len() - 1)
    }

    pub fn add_sticky_note_near_node(&mut self, node_index: usize) -> Option<usize> {
        let node = self.nodes.get(node_index)?;
        let position =
            GraphPoint::new(node.layout_position.x + 0.08, node.layout_position.y - 0.18);
        let annotation = GraphAnnotation::sticky_note(
            self.unique_annotation_id("note"),
            self.unique_annotation_title("Sticky Note"),
            String::new(),
            position,
            GraphPoint::new(0.22, 0.20),
        );
        self.annotations.push(annotation);
        Some(self.annotations.len() - 1)
    }

    pub fn settle_node_drag_for_network_boxes(
        &mut self,
        node_index: usize,
        fast_drag: bool,
    ) -> bool {
        let Some(node) = self.nodes.get(node_index) else {
            return false;
        };
        let node_id = node.node_id.clone();
        let node_position = node.layout_position;
        let mut changed = false;

        for annotation in &mut self.annotations {
            if annotation.kind != GraphAnnotationKind::NetworkBox {
                continue;
            }

            let is_member = annotation.member_node_ids.iter().any(|id| id == &node_id);
            let contains_node = network_box_contains_position(annotation, node_position);
            if is_member && !contains_node {
                if fast_drag {
                    annotation.member_node_ids.retain(|id| id != &node_id);
                } else {
                    expand_network_box_to_include_position(annotation, node_position);
                }
                changed = true;
            } else if !is_member && contains_node {
                annotation.member_node_ids.push(node_id.clone());
                changed = true;
            }
        }

        changed
    }

    pub fn translate_annotation(&mut self, annotation_index: usize, delta: GraphPoint) -> bool {
        let Some(annotation) = self.annotations.get_mut(annotation_index) else {
            return false;
        };

        annotation.position.x += delta.x;
        annotation.position.y += delta.y;

        if annotation.kind == GraphAnnotationKind::NetworkBox {
            let member_node_ids = annotation.member_node_ids.clone();
            for node in &mut self.nodes {
                if member_node_ids
                    .iter()
                    .any(|member_node_id| member_node_id == &node.node_id)
                {
                    node.layout_position.x += delta.x;
                    node.layout_position.y += delta.y;
                }
            }
        }

        true
    }

    pub fn resize_network_box_to_contents(&mut self, annotation_index: usize) -> bool {
        let Some(annotation) = self.annotations.get(annotation_index) else {
            return false;
        };
        if annotation.kind != GraphAnnotationKind::NetworkBox
            || annotation.member_node_ids.is_empty()
        {
            return false;
        }

        let member_positions = self
            .nodes
            .iter()
            .filter(|node| {
                annotation
                    .member_node_ids
                    .iter()
                    .any(|member_id| member_id == &node.node_id)
            })
            .map(|node| node.layout_position)
            .collect::<Vec<_>>();
        let Some((position, size)) = network_box_bounds_for_positions(&member_positions) else {
            return false;
        };
        let Some(annotation) = self.annotations.get_mut(annotation_index) else {
            return false;
        };
        annotation.position = position;
        annotation.size = size;
        true
    }

    fn unique_annotation_id(&self, prefix: &str) -> String {
        let mut suffix = 1;
        loop {
            let annotation_id = format!("{prefix}.{suffix}");
            if !self
                .annotations
                .iter()
                .any(|annotation| annotation.annotation_id == annotation_id)
            {
                return annotation_id;
            }
            suffix += 1;
        }
    }

    fn unique_annotation_title(&self, candidate: &str) -> String {
        if !self
            .annotations
            .iter()
            .any(|annotation| annotation.title == candidate)
        {
            return candidate.to_owned();
        }

        let mut suffix = 2;
        loop {
            let title = format!("{candidate} {suffix}");
            if !self
                .annotations
                .iter()
                .any(|annotation| annotation.title == title)
            {
                return title;
            }
            suffix += 1;
        }
    }

    pub fn emits(&self, geometry: &Geometry) -> bool {
        let layer_visible = match geometry {
            Geometry::Polygon(_) => self.layer_visible(LayerKind::Polygons),
            Geometry::CubicBezier(_) => self.layer_visible(LayerKind::Curves),
        };

        layer_visible && self.passes_filter(geometry)
    }

    pub fn visible_output_count(&self) -> usize {
        self.active_geometry()
            .iter()
            .filter(|geometry| self.emits(geometry))
            .count()
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

    pub fn attribute_table_preview_rows(
        &self,
        query: &AttributeTableQuery,
        limit: usize,
    ) -> Vec<AttributeTableRow> {
        let source_path = self
            .source
            .source_path
            .clone()
            .or_else(|| self.source.metadata.source_path.clone());
        let provenance = self.source.metadata.provenance;
        let search = query.search.trim().to_ascii_lowercase();

        self.active_geometry()
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
            .take(limit)
            .collect()
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
        filter_node.generated = Some(GeneratedNodeInfo::managed(
            GeneratedNodeSource::AttributeTableCommit,
        ));
        true
    }

    #[allow(dead_code)]
    pub fn add_null_operator_node(&mut self, name: impl Into<String>) -> usize {
        let mut name = name.into();
        if name.trim().is_empty() {
            name = format!(
                "NULL_{}",
                self.nodes
                    .iter()
                    .filter(|node| node.kind == NodeKind::Null)
                    .count()
                    + 1
            );
        } else {
            name = name.trim().to_owned();
        }
        name = self.unique_node_name(&name);

        let insert_index = self
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .unwrap_or(self.nodes.len());
        let mut node = GraphNode::null_operator(name);
        node.node_id = self.unique_node_id("null");
        node.layout_position = GraphPoint::new(0.82, 0.5);
        self.nodes.insert(insert_index, node);
        insert_index
    }

    fn unique_node_name(&self, candidate: &str) -> String {
        if !self.nodes.iter().any(|node| node.name == candidate) {
            return candidate.to_owned();
        }

        let mut suffix = 2;
        loop {
            let name = format!("{candidate}_{suffix}");
            if !self.nodes.iter().any(|node| node.name == name) {
                return name;
            }
            suffix += 1;
        }
    }

    pub fn set_node_name(&mut self, node_index: usize, candidate: impl Into<String>) -> bool {
        let candidate = candidate.into().trim().to_owned();
        if candidate.is_empty() {
            return false;
        }
        let Some(current_name) = self.nodes.get(node_index).map(|node| node.name.clone()) else {
            return false;
        };
        if current_name == candidate {
            return true;
        }

        let mut name = candidate.clone();
        if self
            .nodes
            .iter()
            .enumerate()
            .any(|(index, node)| index != node_index && node.name == name)
        {
            let mut suffix = 2;
            loop {
                name = format!("{candidate}_{suffix}");
                if !self
                    .nodes
                    .iter()
                    .enumerate()
                    .any(|(index, node)| index != node_index && node.name == name)
                {
                    break;
                }
                suffix += 1;
            }
        }

        if let Some(node) = self.nodes.get_mut(node_index) {
            node.name = name;
            true
        } else {
            false
        }
    }

    fn unique_node_id(&self, prefix: &str) -> String {
        let mut suffix = 1;
        loop {
            let node_id = format!("{prefix}.{suffix}");
            if !self.node_id_is_reserved(&node_id) {
                return node_id;
            }
            suffix += 1;
        }
    }

    fn node_id_is_reserved(&self, node_id: &str) -> bool {
        self.nodes.iter().any(|node| node.node_id == node_id)
            || self.nodes.iter().any(|node| {
                node.reference_input.as_ref().is_some_and(|reference| {
                    reference
                        .targets
                        .iter()
                        .any(|target| target.target.node_id == node_id)
                })
            })
    }

    #[allow(dead_code)]
    pub fn add_reference_input_node(&mut self, target_node_index: usize) -> Option<usize> {
        let target = self.reference_target_for_node(target_node_index)?;
        let provenance =
            ReferenceTargetProvenance::from_node(self.nodes.get(target_node_index)?, &target);
        let insert_index = self
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .unwrap_or(self.nodes.len());
        let mut node = GraphNode::reference_input(
            self.unique_node_id("reference_input"),
            ReferenceTargetEntry {
                target,
                enabled: true,
                provenance,
            },
        );
        node.layout_position = GraphPoint::new(0.88, 0.5);
        self.nodes.insert(insert_index, node);
        Some(insert_index)
    }

    #[allow(dead_code)]
    pub fn add_reference_target_to_node(
        &mut self,
        reference_node_index: usize,
        target_node_index: usize,
    ) -> bool {
        let Some(target) = self.reference_target_for_node(target_node_index) else {
            return false;
        };
        let Some(source_node) = self.nodes.get(target_node_index) else {
            return false;
        };
        let provenance = ReferenceTargetProvenance::from_node(source_node, &target);
        let Some(reference_input) = self
            .nodes
            .get_mut(reference_node_index)
            .and_then(|node| node.reference_input.as_mut())
        else {
            return false;
        };
        if reference_input
            .targets
            .iter()
            .any(|entry| entry.target == target)
        {
            return false;
        }

        reference_input.targets.push(ReferenceTargetEntry {
            target,
            enabled: true,
            provenance,
        });
        if let Some(node) = self.nodes.get_mut(reference_node_index) {
            node.evaluation.state = EvaluationState::Stale;
            node.evaluation.message = Some("Reference target set changed.".to_owned());
        }
        true
    }

    #[allow(dead_code)]
    pub fn set_reference_target_enabled(
        &mut self,
        reference_node_index: usize,
        target_node_id: &str,
        enabled: bool,
    ) -> bool {
        let Some(node) = self.nodes.get_mut(reference_node_index) else {
            return false;
        };
        let Some(reference_input) = node.reference_input.as_mut() else {
            return false;
        };
        let Some(entry) = reference_input
            .targets
            .iter_mut()
            .find(|entry| entry.target.node_id == target_node_id)
        else {
            return false;
        };
        if entry.enabled == enabled {
            return false;
        }

        entry.enabled = enabled;
        node.evaluation.state = EvaluationState::Stale;
        node.evaluation.message = Some("Reference target enablement changed.".to_owned());
        true
    }

    #[allow(dead_code)]
    pub fn remove_reference_target_from_node(
        &mut self,
        reference_node_index: usize,
        target_node_id: &str,
    ) -> bool {
        let Some(node) = self.nodes.get_mut(reference_node_index) else {
            return false;
        };
        let Some(reference_input) = node.reference_input.as_mut() else {
            return false;
        };
        let original_len = reference_input.targets.len();
        reference_input
            .targets
            .retain(|entry| entry.target.node_id != target_node_id);
        if reference_input.targets.len() == original_len {
            return false;
        }

        node.evaluation.state = EvaluationState::Stale;
        node.evaluation.message = Some("Reference target removed.".to_owned());
        true
    }

    pub fn reference_target_for_node(
        &self,
        target_node_index: usize,
    ) -> Option<ReferenceTargetIdentity> {
        let node = self.nodes.get(target_node_index)?;
        self.node_primary_output_kind(node.kind)?;
        Some(ReferenceTargetIdentity {
            graph_id: MAIN_GRAPH_ID.to_owned(),
            node_id: node.node_id.clone(),
            output_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
        })
    }

    fn node_primary_output_kind(&self, kind: NodeKind) -> Option<HoudiniDataKind> {
        match kind {
            NodeKind::Source
            | NodeKind::Filter
            | NodeKind::Style
            | NodeKind::Null
            | NodeKind::SubstrateProjection
            | NodeKind::PythonOperator
            | NodeKind::ProceduralAsset
            | NodeKind::NativeOperator => Some(HoudiniDataKind::GeometryTable),
            NodeKind::ReferenceInput | NodeKind::Output => None,
        }
    }

    fn default_coordinate_contract_for_kind(kind: NodeKind) -> Option<SubstrateCoordinateContract> {
        matches!(
            kind,
            NodeKind::Source
                | NodeKind::Filter
                | NodeKind::Style
                | NodeKind::Null
                | NodeKind::ReferenceInput
                | NodeKind::SubstrateProjection
                | NodeKind::PythonOperator
                | NodeKind::ProceduralAsset
                | NodeKind::NativeOperator
                | NodeKind::Output
        )
        .then(SubstrateCoordinateContract::demo_byteplot)
    }

    pub fn resolve_reference_target(
        &self,
        target: &ReferenceTargetIdentity,
    ) -> ReferenceTargetResolution {
        if target.graph_id != MAIN_GRAPH_ID {
            if let Some(resolution) = self.resolve_unlocked_asset_internal_target(target) {
                return resolution;
            }
            if self
                .procedural_asset_declarations
                .iter()
                .any(|declaration| declaration.wrapped_subgraph.graph_id == target.graph_id)
            {
                return ReferenceTargetResolution::diagnostic(
                    target,
                    ReferenceDiagnosticStatus::AssetPrivateInternal,
                    "Matched procedural asset internals are private; reference the asset boundary output or unlock the instance for local editing.",
                );
            }
            return ReferenceTargetResolution::diagnostic(
                target,
                ReferenceDiagnosticStatus::DisallowedBoundary,
                "Reference target is outside the current project graph.",
            );
        }

        let Some((target_node_index, target_node)) = self
            .nodes
            .iter()
            .enumerate()
            .find(|(_, node)| node.node_id == target.node_id)
        else {
            return ReferenceTargetResolution::diagnostic(
                target,
                ReferenceDiagnosticStatus::MissingNode,
                "Reference target node is missing.",
            );
        };

        if target.output_name != PRIMARY_GEOMETRY_OUTPUT {
            return ReferenceTargetResolution::diagnostic(
                target,
                ReferenceDiagnosticStatus::MissingOutput,
                "Reference target output is missing.",
            );
        }

        let Some(output_kind) = self.node_primary_output_kind(target_node.kind) else {
            return ReferenceTargetResolution::diagnostic(
                target,
                ReferenceDiagnosticStatus::MissingOutput,
                "Reference target node does not expose a compatible geometry output.",
            );
        };

        ReferenceTargetResolution {
            target: target.clone(),
            status: ReferenceDiagnosticStatus::Resolved,
            readable_path: readable_reference_path(target_node, &target.output_name),
            target_node_index: Some(target_node_index),
            output_kind: Some(output_kind),
            coordinate_contract: target_node.coordinate_contract.clone(),
            record_count: self.node_output_record_count_for_index(target_node_index),
            source_provenance: Some(self.source.metadata.provenance),
            diagnostic: None,
        }
    }

    fn resolve_unlocked_asset_internal_target(
        &self,
        target: &ReferenceTargetIdentity,
    ) -> Option<ReferenceTargetResolution> {
        let (asset_node_index, asset_node) =
            self.nodes.iter().enumerate().find_map(|(index, node)| {
                let asset = node.procedural_asset.as_ref()?;
                (asset.contents_unlocked
                    && target.graph_id == unlocked_asset_graph_id(&asset.instance_id))
                .then_some((index, node))
            })?;

        if target.output_name != PRIMARY_GEOMETRY_OUTPUT {
            return Some(ReferenceTargetResolution::diagnostic(
                target,
                ReferenceDiagnosticStatus::MissingOutput,
                "Unlocked asset internal target output is missing.",
            ));
        }

        Some(ReferenceTargetResolution {
            target: target.clone(),
            status: ReferenceDiagnosticStatus::Resolved,
            readable_path: format!(
                "{}/{}:{}",
                asset_node.name, target.node_id, target.output_name
            ),
            target_node_index: Some(asset_node_index),
            output_kind: Some(HoudiniDataKind::GeometryTable),
            coordinate_contract: asset_node.coordinate_contract.clone(),
            record_count: self.node_output_record_count_for_index(asset_node_index),
            source_provenance: Some(self.source.metadata.provenance),
            diagnostic: None,
        })
    }

    #[allow(dead_code)]
    pub fn reference_input_resolution(
        &self,
        node_index: usize,
    ) -> Option<ReferenceTargetResolution> {
        self.reference_input_resolutions(node_index)?
            .into_iter()
            .next()
            .map(|entry| entry.resolution)
    }

    pub fn reference_input_resolutions(
        &self,
        node_index: usize,
    ) -> Option<Vec<ReferenceTargetEntryResolution>> {
        let node = self.nodes.get(node_index)?;
        let reference_input = node.reference_input.as_ref()?;
        let raw_resolutions = reference_input
            .targets
            .iter()
            .map(|entry| (entry, self.resolve_reference_target(&entry.target)))
            .collect::<Vec<_>>();
        let expected_coordinate_contract = raw_resolutions
            .iter()
            .filter(|(entry, _)| entry.enabled)
            .find_map(|(_, resolution)| resolution.coordinate_contract.clone());
        Some(
            raw_resolutions
                .into_iter()
                .map(|(entry, resolution)| ReferenceTargetEntryResolution {
                    enabled: entry.enabled,
                    provenance: entry.provenance.clone(),
                    resolution: Self::with_coordinate_compatibility(
                        resolution,
                        expected_coordinate_contract.as_ref(),
                    ),
                    expected_coordinate_contract: expected_coordinate_contract.clone(),
                })
                .collect(),
        )
    }

    pub fn reference_consumers_for_node(
        &self,
        target_node_index: usize,
    ) -> Vec<ReferenceConsumerInfo> {
        let Some(target_node) = self.nodes.get(target_node_index) else {
            return Vec::new();
        };
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.reference_input.is_some())
            .flat_map(|(reference_node_index, reference_node)| {
                self.reference_input_resolutions(reference_node_index)
                    .unwrap_or_default()
                    .into_iter()
                    .filter(move |entry| {
                        entry.resolution.target.graph_id == MAIN_GRAPH_ID
                            && entry.resolution.target.node_id == target_node.node_id
                            && entry.resolution.target.output_name == PRIMARY_GEOMETRY_OUTPUT
                    })
                    .map(move |entry| ReferenceConsumerInfo {
                        reference_node_index,
                        reference_node_id: reference_node.node_id.clone(),
                        reference_node_name: reference_node.name.clone(),
                        target_output_name: entry.resolution.target.output_name,
                        readable_source_path: entry.resolution.readable_path,
                        enabled: entry.enabled,
                        status: entry.resolution.status,
                        diagnostic: entry.resolution.diagnostic,
                    })
            })
            .collect()
    }

    pub fn reference_output_change_warning_for_node(
        &self,
        target_node_index: usize,
    ) -> Option<ReferenceOutputChangeWarning> {
        let target_node = self.nodes.get(target_node_index)?;
        let affected_references = self.reference_consumers_for_node(target_node_index);
        (!affected_references.is_empty()).then(|| ReferenceOutputChangeWarning {
            target_node_index,
            target_node_id: target_node.node_id.clone(),
            target_node_name: target_node.name.clone(),
            output_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
            affected_references,
        })
    }

    pub fn reference_coordinate_repair_summary(&self, node_index: usize) -> Option<String> {
        self.reference_input_resolutions(node_index)?
            .into_iter()
            .find(|entry| {
                entry.resolution.status
                    == ReferenceDiagnosticStatus::CoordinateIncompatibleRepairable
            })
            .and_then(|entry| entry.resolution.diagnostic)
    }

    pub fn create_assisted_projection_for_first_repairable_reference_target(
        &mut self,
        reference_node_index: usize,
    ) -> Option<usize> {
        let repair = self
            .reference_input_resolutions(reference_node_index)?
            .into_iter()
            .find(|entry| {
                entry.resolution.status
                    == ReferenceDiagnosticStatus::CoordinateIncompatibleRepairable
            })?;
        let from_contract = repair.resolution.coordinate_contract.clone()?;
        let to_contract = repair.expected_coordinate_contract.clone()?;
        let source_target = repair.resolution.target.clone();
        let source_node_index = repair.resolution.target_node_index?;
        let projection_node_id = self.unique_node_id("substrate_projection");
        let projection_target = ReferenceTargetIdentity {
            graph_id: MAIN_GRAPH_ID.to_owned(),
            node_id: projection_node_id.clone(),
            output_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
        };
        let repair_summary = from_contract.repair_summary_to(&to_contract);
        let mut projection_node = GraphNode::substrate_projection(
            projection_node_id,
            SubstrateProjectionNode {
                source_target: source_target.clone(),
                from_contract,
                to_contract,
                repair_summary,
            },
        );
        let source_position = self
            .nodes
            .get(source_node_index)
            .map(|node| node.layout_position)
            .unwrap_or(GraphPoint::new(0.5, 0.5));
        projection_node.layout_position =
            GraphPoint::new(source_position.x + 0.08, source_position.y + 0.12);

        let insert_index = reference_node_index.min(self.nodes.len());
        self.nodes.insert(insert_index, projection_node);
        let reference_node = self.nodes.get_mut(reference_node_index + 1)?;
        let reference_input = reference_node.reference_input.as_mut()?;
        let target_entry = reference_input
            .targets
            .iter_mut()
            .find(|entry| entry.target == source_target)?;
        target_entry.target = projection_target;
        reference_node.evaluation.state = EvaluationState::Stale;
        reference_node.evaluation.message =
            Some("Reference target repaired through a visible substrate projection.".to_owned());
        Some(insert_index)
    }

    fn with_coordinate_compatibility(
        mut resolution: ReferenceTargetResolution,
        expected_coordinate_contract: Option<&SubstrateCoordinateContract>,
    ) -> ReferenceTargetResolution {
        if resolution.status != ReferenceDiagnosticStatus::Resolved {
            return resolution;
        }

        let Some(coordinate_contract) = &resolution.coordinate_contract else {
            resolution.status = ReferenceDiagnosticStatus::CoordinateContractMissing;
            resolution.record_count = 0;
            resolution.diagnostic = Some(
                "Reference target has no substrate coordinate contract; fix metadata or disable the target.".to_owned(),
            );
            return resolution;
        };

        let Some(expected_coordinate_contract) = expected_coordinate_contract else {
            return resolution;
        };

        if coordinate_contract == expected_coordinate_contract {
            return resolution;
        }

        resolution.status = ReferenceDiagnosticStatus::CoordinateIncompatibleRepairable;
        resolution.record_count = 0;
        resolution.diagnostic = Some(format!(
            "Reference target substrate coordinates differ from the enabled target set baseline; create a visible substrate projection or disable the target. {}",
            coordinate_contract.repair_summary_to(expected_coordinate_contract),
        ));
        resolution
    }

    #[allow(dead_code)]
    pub fn remove_node(&mut self, index: usize) -> Option<GraphNode> {
        if self.nodes.get(index)?.kind == NodeKind::Output {
            return None;
        }
        Some(self.nodes.remove(index))
    }

    pub fn mark_reference_inputs_stale_for_target_index(&mut self, target_index: usize) {
        let Some(target_node_id) = self
            .nodes
            .get(target_index)
            .map(|node| node.node_id.clone())
        else {
            return;
        };
        self.mark_reference_inputs_stale_for_target_node_id(&target_node_id);
    }

    fn mark_reference_inputs_stale_for_target_node_id(&mut self, target_node_id: &str) {
        for node in &mut self.nodes {
            let Some(reference_input) = node.reference_input.as_ref() else {
                continue;
            };
            if reference_input
                .targets
                .iter()
                .any(|entry| entry.enabled && entry.target.node_id == target_node_id)
            {
                node.evaluation.state = EvaluationState::Stale;
                node.evaluation.message =
                    Some("Referenced output changed; reference input is stale.".to_owned());
            }
        }
    }

    pub fn null_operator_contract(&self, node_index: usize) -> Option<NullOperatorContract> {
        let node = self.nodes.get(node_index)?;
        if node.kind != NodeKind::Null {
            return None;
        }
        let input_count = self.pass_through_input_count_for_node(node_index);
        Some(NullOperatorContract {
            node_name: node.name.clone(),
            convention: NullNameConvention::from_name(&node.name),
            input_kind: HoudiniDataKind::GeometryTable,
            output_kind: HoudiniDataKind::GeometryTable,
            input_record_count: input_count,
            output_record_count: input_count,
            source_provenance: self.source.metadata.provenance,
            preserves_record_identity: true,
            preserves_source_provenance: true,
            preserves_evaluation_state: true,
        })
    }

    fn pass_through_input_count_for_node(&self, node_index: usize) -> usize {
        self.nodes
            .iter()
            .enumerate()
            .take(node_index)
            .rev()
            .find(|(_, node)| node.participates_in_output)
            .map_or_else(
                || self.active_geometry().len(),
                |(index, _)| self.node_output_record_count_for_index(index),
            )
    }

    fn node_output_record_count_for_index(&self, node_index: usize) -> usize {
        let Some(node) = self.nodes.get(node_index) else {
            return 0;
        };
        if node.kind == NodeKind::ReferenceInput {
            return self
                .reference_input_resolutions(node_index)
                .map_or(0, |entries| {
                    if entries.iter().any(|entry| {
                        entry.enabled
                            && entry.resolution.status != ReferenceDiagnosticStatus::Resolved
                    }) {
                        return 0;
                    }
                    entries
                        .iter()
                        .filter(|entry| entry.enabled)
                        .map(|entry| {
                            if entry.resolution.status == ReferenceDiagnosticStatus::Resolved {
                                entry.resolution.record_count
                            } else {
                                0
                            }
                        })
                        .sum()
                });
        }
        if let Some(projection) = &node.substrate_projection {
            let resolution = self.resolve_reference_target(&projection.source_target);
            return if resolution.status == ReferenceDiagnosticStatus::Resolved {
                resolution.record_count
            } else {
                0
            };
        }
        self.node_output_record_count(node.kind)
    }

    fn node_output_record_count(&self, kind: NodeKind) -> usize {
        let source_count = self.active_geometry().len();
        let filtered_count = self
            .active_geometry()
            .iter()
            .filter(|geometry| self.passes_filter(geometry))
            .count();

        match kind {
            NodeKind::Source => source_count,
            NodeKind::Filter
            | NodeKind::Style
            | NodeKind::Null
            | NodeKind::SubstrateProjection
            | NodeKind::PythonOperator
            | NodeKind::ProceduralAsset
            | NodeKind::NativeOperator => filtered_count,
            NodeKind::ReferenceInput => 0,
            NodeKind::Output => self.visible_output_count(),
        }
    }

    #[allow(dead_code)]
    pub fn add_python_operator_node(&mut self, declaration_id: impl Into<String>) -> usize {
        let declaration_id = declaration_id.into();
        let instance_id = format!(
            "python_operator_{}",
            self.nodes
                .iter()
                .filter(|node| node.kind == NodeKind::PythonOperator)
                .count()
                + 1
        );
        let insert_index = self
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .unwrap_or(self.nodes.len());
        let node = GraphNode::python_operator(instance_id, declaration_id);
        self.nodes.insert(insert_index, node);
        insert_index
    }

    #[allow(dead_code)]
    pub fn add_procedural_asset_node(&mut self, asset_id: impl Into<String>) -> usize {
        let asset_id = asset_id.into();
        let instance_version = self
            .procedural_asset_declarations
            .iter()
            .find(|declaration| declaration.asset_id == asset_id)
            .map(|declaration| declaration.version.clone())
            .unwrap_or_else(|| "unknown".to_owned());
        let instance_id = format!(
            "asset_{}",
            self.nodes
                .iter()
                .filter(|node| node.kind == NodeKind::ProceduralAsset)
                .count()
                + 1
        );
        let insert_index = self
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .unwrap_or(self.nodes.len());
        let node = GraphNode::procedural_asset(instance_id, asset_id, instance_version);
        self.nodes.insert(insert_index, node);
        insert_index
    }

    #[allow(dead_code)]
    pub fn create_asset_draft_from_graph(
        &self,
        display_name: impl Into<String>,
        description: impl Into<String>,
        help: impl Into<String>,
    ) -> CreateAssetDraft {
        let display_name = display_name.into();
        let asset_slug = sanitize_asset_id_part(&display_name);
        CreateAssetDraft {
            asset_id: format!("project.asset.{asset_slug}"),
            display_name,
            version: "0.1.0".to_owned(),
            description: description.into(),
            help: help.into(),
            inputs: vec![HoudiniOperatorPort {
                name: "geometry".to_owned(),
                data_kind: HoudiniDataKind::GeometryTable,
                required: true,
                help: "Input graph geometry.".to_owned(),
            }],
            outputs: vec![HoudiniOperatorPort {
                name: "geometry".to_owned(),
                data_kind: HoudiniDataKind::GeometryTable,
                required: true,
                help: "Output graph geometry preserving native cubic Beziers.".to_owned(),
            }],
            promoted_parameters: self.promotable_asset_parameters(),
            graph_snapshot: ProceduralAssetGraphSnapshot {
                node_count: self.nodes.len(),
                edge_count: self.graph_layout().edges.len(),
                layer_count: self.layers.len(),
                geometry_contract: "HoudiniGeometryRecord polygons and native cubic Beziers"
                    .to_owned(),
            },
        }
    }

    #[allow(dead_code)]
    pub fn commit_asset_draft(&mut self, draft: CreateAssetDraft) -> String {
        let asset_id = draft.asset_id.clone();
        let declaration = draft.into_declaration();
        if let Some(existing) = self
            .procedural_asset_declarations
            .iter_mut()
            .find(|existing| existing.asset_id == declaration.asset_id)
        {
            *existing = declaration;
        } else {
            self.procedural_asset_declarations.push(declaration);
        }
        self.refresh_asset_version_statuses();
        asset_id
    }

    #[allow(dead_code)]
    pub fn refresh_asset_version_statuses(&mut self) {
        for node in &mut self.nodes {
            let Some(asset_node) = node.procedural_asset.as_mut() else {
                continue;
            };
            let declaration = self
                .procedural_asset_declarations
                .iter()
                .find(|declaration| declaration.asset_id == asset_node.asset_id);
            asset_node.version_status = match declaration {
                Some(declaration) if declaration.version == asset_node.instance_version => {
                    OperatorVersionStatus::Current
                }
                Some(_) => OperatorVersionStatus::NewerAvailable,
                None => OperatorVersionStatus::MissingDeclaration,
            };
            if asset_node.version_status == OperatorVersionStatus::NewerAvailable {
                node.evaluation.state = EvaluationState::Stale;
                node.evaluation.message = Some(
                    "Asset declaration version changed after this instance was created.".to_owned(),
                );
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_procedural_asset_contents_unlocked(
        &mut self,
        node_index: usize,
        contents_unlocked: bool,
    ) -> bool {
        let Some(node) = self.nodes.get_mut(node_index) else {
            return false;
        };
        let Some(asset_node) = node.procedural_asset.as_mut() else {
            return false;
        };
        if asset_node.contents_unlocked == contents_unlocked {
            return false;
        }

        asset_node.contents_unlocked = contents_unlocked;
        node.evaluation.state = EvaluationState::Stale;
        node.evaluation.message = if contents_unlocked {
            Some("Asset contents unlocked for local internal references.".to_owned())
        } else {
            Some("Asset contents matched to the pinned definition.".to_owned())
        };
        true
    }

    #[allow(dead_code)]
    pub fn unlocked_asset_graph_id_for_node(&self, node_index: usize) -> Option<String> {
        self.nodes
            .get(node_index)?
            .procedural_asset
            .as_ref()
            .map(|asset| unlocked_asset_graph_id(&asset.instance_id))
    }

    fn promotable_asset_parameters(&self) -> Vec<HoudiniParameterDeclaration> {
        let mut parameters = Vec::new();
        if let Some(filter_node) = self.nodes.iter().find(|node| node.kind == NodeKind::Filter) {
            parameters.push(HoudiniParameterDeclaration {
                name: "minimum_score".to_owned(),
                kind: HoudiniParameterKind::Float,
                default_value: HoudiniParameterValue::Float(filter_node.parameter.value),
                range: Some(HoudiniNumericRange { min: 0.0, max: 1.0 }),
                allowed_values: Vec::new(),
                help: "Promoted graph filter threshold.".to_owned(),
            });
        }
        if let Some(style_node) = self.nodes.iter().find(|node| node.kind == NodeKind::Style) {
            parameters.push(HoudiniParameterDeclaration {
                name: "stroke_scale".to_owned(),
                kind: HoudiniParameterKind::Float,
                default_value: HoudiniParameterValue::Float(style_node.parameter.value),
                range: Some(HoudiniNumericRange { min: 0.0, max: 1.0 }),
                allowed_values: Vec::new(),
                help: "Promoted graph style stroke scale.".to_owned(),
            });
        }
        parameters
    }

    #[allow(dead_code)]
    pub fn add_native_operator_node(&mut self, operator_id: impl Into<String>) -> usize {
        let operator_id = operator_id.into();
        let instance_id = format!(
            "native_operator_{}",
            self.nodes
                .iter()
                .filter(|node| node.kind == NodeKind::NativeOperator)
                .count()
                + 1
        );
        let insert_index = self
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .unwrap_or(self.nodes.len());
        let node = GraphNode::native_operator(instance_id, operator_id);
        self.nodes.insert(insert_index, node);
        insert_index
    }

    fn native_operator_node_inputs(
        &self,
        node_index: usize,
    ) -> Option<(&GraphNode, &NativeOperatorNode, &NativeOperatorDeclaration)> {
        let node = self.nodes.get(node_index)?;
        let native_operator = node.native_operator.as_ref()?;
        let declaration = self
            .native_operator_declarations
            .iter()
            .find(|declaration| declaration.operator_id == native_operator.operator_id)?;
        Some((node, native_operator, declaration))
    }

    #[allow(dead_code)]
    pub fn native_operator_cache_key(&self, node_index: usize) -> Option<NativeOperatorCacheKey> {
        let (node, native_operator, declaration) = self.native_operator_node_inputs(node_index)?;
        let implementation_digest = declaration.implementation_digest();
        let parameter_digest = declaration.parameter_digest(node.parameter.value);
        let input_cache_keys = self.input_cache_keys_for_node(node_index);
        let host_compatibility_version = declaration.host_compatibility_version.clone();
        let capability_digest =
            declaration.capability_digest(&self.native_operator_trust.granted_capabilities);
        let key_digest = stable_digest(&serde_json::json!({
            "operator_id": &declaration.operator_id,
            "implementation_digest": &implementation_digest,
            "declaration_version": &declaration.version,
            "parameter_digest": &parameter_digest,
            "input_cache_keys": &input_cache_keys,
            "host_compatibility_version": &host_compatibility_version,
            "capability_digest": &capability_digest,
        }));

        Some(NativeOperatorCacheKey {
            key_digest,
            operator_id: declaration.operator_id.clone(),
            node_instance_id: native_operator.instance_id.clone(),
            implementation_digest,
            declaration_version: declaration.version.clone(),
            parameter_digest,
            input_cache_keys,
            host_compatibility_version,
            capability_digest,
        })
    }

    #[allow(dead_code)]
    pub fn record_native_operator_output(
        &mut self,
        node_index: usize,
        output_counts: NativeOperatorOutputCounts,
    ) -> Option<NativeOperatorProvenanceRecord> {
        let (_, native_operator, declaration) = self.native_operator_node_inputs(node_index)?;
        let cache_key = self.native_operator_cache_key(node_index)?;
        let record = NativeOperatorProvenanceRecord {
            operator_id: declaration.operator_id.clone(),
            version: declaration.version.clone(),
            node_instance_id: native_operator.instance_id.clone(),
            implementation_digest: cache_key.implementation_digest.clone(),
            host_compatibility_version: cache_key.host_compatibility_version.clone(),
            parameter_digest: cache_key.parameter_digest.clone(),
            input_cache_keys: cache_key.input_cache_keys.clone(),
            timestamp: current_timestamp_millis(),
            output_counts,
        };

        if let Some(node) = self.nodes.get_mut(node_index)
            && let Some(native_operator) = node.native_operator.as_mut()
        {
            native_operator.cache_key = Some(cache_key.clone());
            native_operator.provenance = Some(record.clone());
            native_operator.provenance_summary = Some(record.summary());
            native_operator.last_valid_cache_key = Some(cache_key.key_digest);
            native_operator.last_failure_summary = None;
        }

        Some(record)
    }

    #[allow(dead_code)]
    pub fn refresh_native_operator_cache_statuses(&mut self) {
        for index in 0..self.nodes.len() {
            let Some((_, native_operator, declaration)) = self.native_operator_node_inputs(index)
            else {
                continue;
            };
            let version_status = if declaration.version
                == native_operator
                    .cache_key
                    .as_ref()
                    .map(|key| key.declaration_version.as_str())
                    .unwrap_or(declaration.version.as_str())
            {
                OperatorVersionStatus::Current
            } else {
                OperatorVersionStatus::NewerAvailable
            };
            let cache_stale = native_operator
                .cache_key
                .as_ref()
                .zip(self.native_operator_cache_key(index).as_ref())
                .is_some_and(|(recorded, current)| recorded.key_digest != current.key_digest);

            if let Some(node) = self.nodes.get_mut(index)
                && let Some(native_operator) = node.native_operator.as_mut()
            {
                native_operator.version_status = version_status;
                if cache_stale {
                    node.evaluation.state = EvaluationState::Stale;
                    node.evaluation.message =
                        Some("Native operator cache key changed after the last run.".to_owned());
                }
            }
        }
    }

    fn native_operator_load_status(&self, operator_id: Option<&str>) -> NativeOperatorLoadStatus {
        let Some(operator_id) = operator_id else {
            return NativeOperatorLoadStatus::DeclarationMissing;
        };
        let Some(declaration) = self
            .native_operator_declarations
            .iter()
            .find(|declaration| declaration.operator_id == operator_id)
        else {
            return NativeOperatorLoadStatus::DeclarationMissing;
        };
        if !self.native_operator_trust.project_trusted
            && !self
                .native_operator_trust
                .enabled_operator_ids
                .iter()
                .any(|enabled| enabled == operator_id)
        {
            return NativeOperatorLoadStatus::TrustRequired;
        }
        if declaration.host_compatibility_version != NATIVE_OPERATOR_HOST_COMPATIBILITY_VERSION {
            return NativeOperatorLoadStatus::HostIncompatible;
        }
        if declaration.provenance.build_digest.is_none() {
            return NativeOperatorLoadStatus::ImplementationDigestMissing;
        }
        if declaration.capabilities.iter().any(|capability| {
            !self
                .native_operator_trust
                .granted_capabilities
                .contains(capability)
        }) {
            return NativeOperatorLoadStatus::MissingCapabilityGrant;
        }
        NativeOperatorLoadStatus::Ready
    }

    fn block_native_operator_run(&self, node_index: usize) -> Option<NativeOperatorLoadStatus> {
        let node = self.nodes.get(node_index)?;
        let native_operator = node.native_operator.as_ref()?;
        let load_status = self.native_operator_load_status(Some(&native_operator.operator_id));
        (load_status != NativeOperatorLoadStatus::Ready).then_some(load_status)
    }

    fn apply_native_operator_block(node: &mut GraphNode, load_status: NativeOperatorLoadStatus) {
        node.evaluation.state = EvaluationState::Manual;
        node.evaluation.manual = true;
        node.evaluation.message = Some(load_status.summary().to_owned());
        if let Some(native_operator) = node.native_operator.as_mut() {
            native_operator.last_failure_summary = Some(load_status.summary().to_owned());
        }
    }

    #[allow(dead_code)]
    pub fn python_environment_resolve_plan(
        &self,
        trigger: PythonEnvironmentResolveTrigger,
    ) -> PythonEnvironmentResolvePlan {
        let mut requirements = Vec::new();
        for requirement in &self.python_environment.project_requirements.requirements {
            push_unique_requirement(
                &mut requirements,
                PythonRequirementContribution {
                    requirement: requirement.clone(),
                    source: PythonRequirementSource::Project,
                },
            );
        }

        for node in self.nodes.iter().filter(|node| node.participates_in_output) {
            let Some(python_operator) = &node.python_operator else {
                continue;
            };
            let Some(declaration) = self
                .python_operator_declarations
                .iter()
                .find(|declaration| declaration.operator_id == python_operator.declaration_id)
            else {
                continue;
            };
            for requirement in &declaration.dependencies.requirements {
                push_unique_requirement(
                    &mut requirements,
                    PythonRequirementContribution {
                        requirement: requirement.clone(),
                        source: PythonRequirementSource::Operator {
                            operator_id: declaration.operator_id.clone(),
                        },
                    },
                );
            }
        }

        let conflicts = dependency_conflicts(&requirements);
        PythonEnvironmentResolvePlan {
            trigger,
            requirements,
            conflicts,
        }
    }

    #[allow(dead_code)]
    pub fn begin_python_environment_resolve(
        &mut self,
        trigger: PythonEnvironmentResolveTrigger,
    ) -> PythonEnvironmentResolvePlan {
        let plan = self.python_environment_resolve_plan(trigger);
        let previous_ready = (self.python_environment.lock_status
            == PythonEnvironmentStatus::Ready)
            .then(|| PythonEnvironmentReadySnapshot {
                lock_digest: self.python_environment.lock_digest.clone(),
                resolver_version: self.python_environment.resolver.version.clone(),
                resolver_executable_path: self.python_environment.resolver.executable_path.clone(),
                environment_path: self.python_environment.environment_path.clone(),
                paths: self.python_environment.paths.clone(),
                dependency_health: self.python_environment.dependency_health.clone(),
                last_health_check: self.python_environment.last_health_check.clone(),
            });

        self.python_environment.lock_status = PythonEnvironmentStatus::Resolving;
        self.python_environment.last_failure_summary = None;
        self.python_environment.resolve_state.last_plan = Some(plan.clone());
        self.python_environment.resolve_state.in_progress = Some(PythonEnvironmentResolveRun {
            trigger,
            resolver_tool: self.python_environment.resolver.tool.clone(),
            resolver_executable_path: self.python_environment.resolver.executable_path.clone(),
            started_at: current_timestamp_millis(),
        });
        if previous_ready.is_some() {
            self.python_environment.resolve_state.previous_ready = previous_ready;
        }

        plan
    }

    #[allow(dead_code)]
    pub fn configure_python_uv_executable_path(&mut self, path: impl Into<String>) {
        let path = path.into();
        self.python_environment.resolver.executable_path =
            (!path.trim().is_empty()).then(|| path.trim().to_owned());
    }

    #[allow(dead_code)]
    pub fn select_existing_python_environment(&mut self, path: impl Into<String>) {
        let path = path.into();
        let path = path.trim();
        if path.is_empty() {
            return;
        }
        self.python_environment.paths.mode = PythonEnvironmentPathMode::ExistingEnvironment;
        self.python_environment.paths.existing_environment_path = Some(path.to_owned());
        self.python_environment.environment_path = Some(path.to_owned());
        self.python_environment.lock_status = PythonEnvironmentStatus::Locked;
    }

    #[allow(dead_code)]
    pub fn select_python_environment_create_path(&mut self, path: impl Into<String>) {
        let path = path.into();
        let path = path.trim();
        if path.is_empty() {
            return;
        }
        self.python_environment.paths.mode = PythonEnvironmentPathMode::CreateProjectLocal;
        self.python_environment.paths.create_environment_path = path.to_owned();
        self.python_environment.environment_path = Some(path.to_owned());
        if self.python_environment.lock_status == PythonEnvironmentStatus::Missing {
            self.python_environment.lock_status = PythonEnvironmentStatus::Unlocked;
        }
    }

    #[allow(dead_code)]
    pub fn complete_python_environment_resolve(
        &mut self,
        lock_digest: impl Into<String>,
        resolver_version: impl Into<String>,
        interpreter_path: impl Into<String>,
        package_count: usize,
    ) {
        let interpreter_path = interpreter_path.into();
        self.python_environment.lock_status = PythonEnvironmentStatus::Ready;
        self.python_environment.lock_digest = Some(lock_digest.into());
        self.python_environment.resolver.version = Some(resolver_version.into());
        self.python_environment.environment_path = Some(interpreter_path.clone());
        match self.python_environment.paths.mode {
            PythonEnvironmentPathMode::ExistingEnvironment => {
                self.python_environment.paths.existing_environment_path = Some(interpreter_path);
            }
            PythonEnvironmentPathMode::CreateProjectLocal => {
                self.python_environment.paths.create_environment_path = interpreter_path;
            }
        }
        self.python_environment.last_health_check = Some(current_timestamp_millis().to_string());
        self.python_environment.last_failure_summary = None;
        self.python_environment.dependency_health = PythonDependencyHealth {
            package_count,
            missing_packages: Vec::new(),
            conflicts: Vec::new(),
            failed_imports: Vec::new(),
        };
        self.python_environment.resolve_state.in_progress = None;
        self.python_environment.resolve_state.previous_ready = None;
    }

    #[allow(dead_code)]
    pub fn fail_python_environment_resolve(&mut self, failure_summary: impl Into<String>) {
        let failure_summary = failure_summary.into();
        self.python_environment.lock_status = PythonEnvironmentStatus::Failed;
        self.python_environment.last_failure_summary = Some(failure_summary.clone());
        self.python_environment.dependency_health.conflicts = self
            .python_environment
            .resolve_state
            .last_plan
            .as_ref()
            .map(|plan| {
                plan.conflicts
                    .iter()
                    .map(PythonDependencyConflict::summary)
                    .collect()
            })
            .unwrap_or_default();
        if self
            .python_environment
            .dependency_health
            .conflicts
            .is_empty()
        {
            self.python_environment
                .dependency_health
                .conflicts
                .push(failure_summary);
        }
        self.python_environment.resolve_state.in_progress = None;
    }

    #[allow(dead_code)]
    pub fn cancel_python_environment_resolve(&mut self) {
        if let Some(previous_ready) = self.python_environment.resolve_state.previous_ready.take() {
            self.python_environment.lock_status = PythonEnvironmentStatus::Ready;
            self.python_environment.lock_digest = previous_ready.lock_digest;
            self.python_environment.resolver.version = previous_ready.resolver_version;
            self.python_environment.resolver.executable_path =
                previous_ready.resolver_executable_path;
            self.python_environment.environment_path = previous_ready.environment_path;
            self.python_environment.paths = previous_ready.paths;
            self.python_environment.dependency_health = previous_ready.dependency_health;
            self.python_environment.last_health_check = previous_ready.last_health_check;
            self.python_environment.last_failure_summary = None;
        } else {
            self.python_environment.lock_status = PythonEnvironmentStatus::Unlocked;
        }
        self.python_environment.resolve_state.in_progress = None;
    }

    fn python_operator_node_inputs(
        &self,
        node_index: usize,
    ) -> Option<(&GraphNode, &PythonOperatorNode, &PythonOperatorDeclaration)> {
        let node = self.nodes.get(node_index)?;
        let python_operator = node.python_operator.as_ref()?;
        let declaration = self
            .python_operator_declarations
            .iter()
            .find(|declaration| declaration.operator_id == python_operator.declaration_id)?;
        Some((node, python_operator, declaration))
    }

    #[allow(dead_code)]
    pub fn python_operator_cache_key(&self, node_index: usize) -> Option<PythonOperatorCacheKey> {
        let (node, python_operator, declaration) = self.python_operator_node_inputs(node_index)?;
        let source_digest = declaration.source_digest();
        let parameter_digest = declaration.parameter_digest(node.parameter.value);
        let input_cache_keys = self.input_cache_keys_for_node(node_index);
        let dependency_lock_digest = self.python_environment.lock_digest.clone();
        let capability_digest = declaration.capability_digest();
        let key_digest = stable_digest(&serde_json::json!({
            "operator_id": &declaration.operator_id,
            "source_digest": &source_digest,
            "declaration_version": &declaration.version,
            "parameter_digest": &parameter_digest,
            "input_cache_keys": &input_cache_keys,
            "dependency_lock_digest": &dependency_lock_digest,
            "capability_digest": &capability_digest,
        }));

        Some(PythonOperatorCacheKey {
            key_digest,
            operator_id: declaration.operator_id.clone(),
            node_instance_id: python_operator.instance_id.clone(),
            source_digest,
            declaration_version: declaration.version.clone(),
            parameter_digest,
            input_cache_keys,
            dependency_lock_digest,
            capability_digest,
        })
    }

    #[allow(dead_code)]
    pub fn record_python_operator_output(
        &mut self,
        node_index: usize,
        output_counts: PythonOperatorOutputCounts,
    ) -> Option<PythonOperatorProvenanceRecord> {
        let (_, python_operator, declaration) = self.python_operator_node_inputs(node_index)?;
        let cache_key = self.python_operator_cache_key(node_index)?;
        let record = PythonOperatorProvenanceRecord {
            operator_id: declaration.operator_id.clone(),
            version: declaration.version.clone(),
            node_instance_id: python_operator.instance_id.clone(),
            source_path: declaration.source_path(),
            source_digest: cache_key.source_digest.clone(),
            parameter_digest: cache_key.parameter_digest.clone(),
            input_cache_keys: cache_key.input_cache_keys.clone(),
            dependency_identity: self.python_environment.dependency_identity(),
            timestamp: current_timestamp_millis(),
            output_counts,
        };

        if let Some(node) = self.nodes.get_mut(node_index)
            && let Some(python_operator) = node.python_operator.as_mut()
        {
            python_operator.cache_key = Some(cache_key);
            python_operator.provenance = Some(record.clone());
            python_operator.provenance_summary = Some(record.summary());
        }

        Some(record)
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    pub fn execute_python_operator_process(
        &mut self,
        node_index: usize,
        project_root: impl AsRef<Path>,
        timeout: Duration,
    ) -> anyhow::Result<PythonProcessRunReport> {
        let (_, python_operator, declaration) = self
            .python_operator_node_inputs(node_index)
            .ok_or_else(|| anyhow::anyhow!("Python operator node or declaration is missing."))?;
        let operator_id = declaration.operator_id.clone();
        let node_instance_id = python_operator.instance_id.clone();
        let dependency_status = self.python_operator_dependency_status(Some(operator_id.as_str()));
        if dependency_status != PythonOperatorDependencyStatus::Ready {
            anyhow::bail!("{}", dependency_status.summary());
        }

        let interpreter_path = self
            .python_environment
            .environment_path
            .clone()
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Project Python environment path is not configured."))?
            .to_owned();
        let PythonOperatorSource::File { path: source_path } = &declaration.entry_point.source
        else {
            anyhow::bail!(
                "Process-boundary Python execution requires a project-local file entry point."
            );
        };
        let source_path = source_path.clone();

        let project_root = project_root.as_ref();
        let run_dir = std::env::temp_dir().join(format!(
            "houdini-python-{}-{}",
            sanitize_asset_id_part(&node_instance_id),
            current_timestamp_millis()
        ));
        std::fs::create_dir_all(&run_dir)?;
        let input_path = run_dir.join("input.houdini_geometry.json");
        let output_path = run_dir.join("output.houdini_geometry.json");
        let input = PythonGeometryExchange::from_geometry(self.active_geometry());
        std::fs::write(&input_path, serde_json::to_vec_pretty(&input)?)?;

        let output = run_python_process(
            project_root.join(&interpreter_path),
            project_root.join(&source_path),
            &input_path,
            &output_path,
            timeout,
        )?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_status = output.exit_status;
        let timed_out = output.timed_out;
        let traceback_summary = traceback_summary(&stderr);

        let mut output_record_count = 0;
        if exit_status == Some(0) && output_path.exists() {
            let exchange =
                serde_json::from_slice::<PythonGeometryExchange>(&std::fs::read(&output_path)?)?;
            output_record_count = exchange.records.len();
            self.recording_geometry = exchange.into_geometry()?;
            self.source = GraphSource::recording_import(
                output_record_count,
                Some(format!("python operator {operator_id}")),
                SourceMetadata::from_geometry(
                    SourceProvenance::PythonOperator,
                    Some(source_path.clone()),
                    &self.recording_geometry,
                    Vec::new(),
                ),
            );
            self.update_source_node_readiness();
            self.record_python_operator_output(
                node_index,
                PythonOperatorOutputCounts {
                    geometry_records: output_record_count,
                    attribute_records: 0,
                    layer_records: 0,
                },
            );
            self.complete_node_run(node_index);
        } else if let Some(node) = self.nodes.get_mut(node_index) {
            node.evaluation.state = EvaluationState::Failed;
            node.evaluation.message = Some(
                traceback_summary
                    .clone()
                    .unwrap_or_else(|| "Python process execution failed.".to_owned()),
            );
            if let Some(python_operator) = node.python_operator.as_mut() {
                python_operator.last_failure_summary = node.evaluation.message.clone();
            }
        }

        Ok(PythonProcessRunReport {
            entry_point: source_path.clone(),
            interpreter_path,
            input_path,
            output_path,
            stdout,
            stderr,
            exit_status,
            timed_out,
            traceback_summary,
            output_record_count,
        })
    }

    #[allow(dead_code)]
    fn input_cache_keys_for_node(&self, node_index: usize) -> Vec<String> {
        self.nodes
            .iter()
            .take(node_index)
            .filter(|node| node.participates_in_output)
            .map(|node| {
                if let Some(python_operator) = &node.python_operator {
                    python_operator
                        .cache_key
                        .as_ref()
                        .map(|cache_key| cache_key.key_digest.clone())
                        .unwrap_or_else(|| format!("{}:uncached", python_operator.instance_id))
                } else if let Some(native_operator) = &node.native_operator {
                    native_operator
                        .cache_key
                        .as_ref()
                        .map(|cache_key| cache_key.key_digest.clone())
                        .unwrap_or_else(|| format!("{}:uncached", native_operator.instance_id))
                } else if let Some(reference_input) = &node.reference_input {
                    let target_set = reference_input
                        .targets
                        .iter()
                        .map(|entry| {
                            let resolution = self.resolve_reference_target(&entry.target);
                            serde_json::json!({
                                "target": &entry.target,
                                "enabled": entry.enabled,
                                "provenance": &entry.provenance,
                                "target_status": resolution.status.as_str(),
                                "coordinate_contract": &resolution.coordinate_contract,
                                "target_cache_key": resolution
                                    .target_node_index
                                    .map(|target_index| self.node_cache_key_material(target_index)),
                            })
                        })
                        .collect::<Vec<_>>();
                    stable_digest(&serde_json::json!({
                        "kind": node.kind.as_str(),
                        "node_id": &node.node_id,
                        "target_set": target_set,
                    }))
                } else {
                    self.node_cache_key_material_for_node(node)
                }
            })
            .collect()
    }

    fn node_cache_key_material(&self, node_index: usize) -> String {
        self.nodes
            .get(node_index)
            .map(|node| self.node_cache_key_material_for_node(node))
            .unwrap_or_else(|| format!("missing-node-{node_index}"))
    }

    fn node_cache_key_material_for_node(&self, node: &GraphNode) -> String {
        stable_digest(&serde_json::json!({
            "node_id": &node.node_id,
            "kind": node.kind.as_str(),
            "parameter": node.parameter.value,
            "rule": &node.parameter.rule_spec,
        }))
    }

    fn block_python_operator_run(
        &self,
        node_index: usize,
    ) -> Option<PythonOperatorDependencyStatus> {
        let (_, _, declaration) = self.python_operator_node_inputs(node_index)?;
        let dependency_status =
            self.python_operator_dependency_status(Some(declaration.operator_id.as_str()));
        (dependency_status != PythonOperatorDependencyStatus::Ready).then_some(dependency_status)
    }

    fn apply_python_operator_block(
        node: &mut GraphNode,
        dependency_status: PythonOperatorDependencyStatus,
    ) {
        node.evaluation.state = match dependency_status {
            PythonOperatorDependencyStatus::StaleEnvironment => EvaluationState::Stale,
            _ => EvaluationState::Manual,
        };
        node.evaluation.manual =
            dependency_status != PythonOperatorDependencyStatus::StaleEnvironment;
        node.evaluation.message = Some(dependency_status.summary().to_owned());
        if let Some(python_operator) = node.python_operator.as_mut() {
            python_operator.last_failure_summary = Some(dependency_status.summary().to_owned());
        }
    }

    fn python_operator_dependency_status(
        &self,
        declaration_id: Option<&str>,
    ) -> PythonOperatorDependencyStatus {
        let Some(declaration_id) = declaration_id else {
            return PythonOperatorDependencyStatus::DeclarationMissing;
        };
        if !self
            .python_operator_declarations
            .iter()
            .any(|declaration| declaration.operator_id == declaration_id)
        {
            return PythonOperatorDependencyStatus::DeclarationMissing;
        }

        match self.python_environment.lock_status {
            PythonEnvironmentStatus::Missing => PythonOperatorDependencyStatus::MissingEnvironment,
            PythonEnvironmentStatus::Unlocked => PythonOperatorDependencyStatus::StaleEnvironment,
            PythonEnvironmentStatus::Resolving | PythonEnvironmentStatus::Locked => {
                PythonOperatorDependencyStatus::ResolvingEnvironment
            }
            PythonEnvironmentStatus::Ready
                if self.python_environment.dependency_health.is_healthy() =>
            {
                PythonOperatorDependencyStatus::Ready
            }
            PythonEnvironmentStatus::Ready | PythonEnvironmentStatus::Failed => {
                PythonOperatorDependencyStatus::FailedEnvironment
            }
            PythonEnvironmentStatus::Stale => PythonOperatorDependencyStatus::StaleEnvironment,
            PythonEnvironmentStatus::Disabled => {
                PythonOperatorDependencyStatus::DisabledEnvironment
            }
        }
    }

    fn reference_input_diagnostic(&self, node_index: usize) -> Option<ReferenceTargetResolution> {
        self.reference_input_resolutions(node_index)?
            .into_iter()
            .filter(|entry| entry.enabled)
            .map(|entry| entry.resolution)
            .find(|resolution| resolution.status != ReferenceDiagnosticStatus::Resolved)
    }

    fn apply_reference_input_diagnostic(
        node: &mut GraphNode,
        resolution: ReferenceTargetResolution,
    ) {
        node.evaluation.state = EvaluationState::Failed;
        node.evaluation.manual = true;
        node.evaluation.message = resolution.diagnostic;
    }

    pub fn pipeline_stages(&self) -> Vec<PipelineStage> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.participates_in_output)
            .map(|(index, node)| self.pipeline_stage_for_node(index, node))
            .collect()
    }

    fn pipeline_stage_for_node(&self, index: usize, node: &GraphNode) -> PipelineStage {
        let input_count = if node.kind == NodeKind::Source {
            0
        } else {
            self.pass_through_input_count_for_node(index)
        };
        PipelineStage {
            name: node.name.clone(),
            input_count,
            output_count: self.node_output_record_count_for_index(index),
            note: match node.kind {
                NodeKind::Source => "Loaded native graph geometry.".to_owned(),
                NodeKind::Filter => "Applied minimum score threshold.".to_owned(),
                NodeKind::Style => "Prepared stroke scale for viewer output.".to_owned(),
                NodeKind::Null => {
                    "Typed pass-through anchor; geometry, provenance, and evaluation flow are unchanged.".to_owned()
                }
                NodeKind::ReferenceInput => {
                    "Live one-way reference to a compatible graph output.".to_owned()
                }
                NodeKind::SubstrateProjection => {
                    "Visible substrate coordinate projection; reference inputs do not hide transforms."
                        .to_owned()
                }
                NodeKind::PythonOperator => "Deferred graph-visible Python operator.".to_owned(),
                NodeKind::ProceduralAsset => {
                    "Graph-backed procedural asset instance.".to_owned()
                }
                NodeKind::NativeOperator => "Deferred trusted native operator.".to_owned(),
                NodeKind::Output => {
                    "Applied layer visibility and boundary preparation.".to_owned()
                }
            },
        }
    }

    pub fn graph_layout(&self) -> GraphLayout {
        let nodes = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| GraphLayoutNode {
                node_index: index,
                name: node.name.clone(),
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
            node.layout_position = position;
        }
    }

    #[allow(dead_code)]
    pub fn mark_node_stale(&mut self, index: usize) {
        if let Some(node) = self.nodes.get_mut(index) {
            node.evaluation.state = EvaluationState::Stale;
            node.evaluation.message = None;
        }
        self.mark_reference_inputs_stale_for_target_index(index);
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
        for index in 0..self.nodes.len() {
            if let Some(resolution) = self.reference_input_diagnostic(index) {
                let node = &mut self.nodes[index];
                Self::apply_reference_input_diagnostic(node, resolution);
                continue;
            }
            if let Some(dependency_status) = self.block_python_operator_run(index) {
                let node = &mut self.nodes[index];
                Self::apply_python_operator_block(node, dependency_status);
                continue;
            }
            if let Some(load_status) = self.block_native_operator_run(index) {
                let node = &mut self.nodes[index];
                Self::apply_native_operator_block(node, load_status);
                continue;
            }

            let node = &mut self.nodes[index];
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
        if let Some(resolution) = self.reference_input_diagnostic(index) {
            if let Some(node) = self.nodes.get_mut(index) {
                Self::apply_reference_input_diagnostic(node, resolution);
            }
            return;
        }
        if let Some(dependency_status) = self.block_python_operator_run(index) {
            if let Some(node) = self.nodes.get_mut(index) {
                Self::apply_python_operator_block(node, dependency_status);
            }
            return;
        }
        if let Some(load_status) = self.block_native_operator_run(index) {
            if let Some(node) = self.nodes.get_mut(index) {
                Self::apply_native_operator_block(node, load_status);
            }
            return;
        }

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
        let stage = self.pipeline_stage_for_node(index, node);
        let source_metadata = self.source.metadata.clone();
        let filter_warnings = self.filter_rule_warning().into_iter().collect::<Vec<_>>();
        let style_warnings = self.style_warnings();
        let reference_consumers = self.reference_consumers_for_node(index);
        let reference_output_warning = self.reference_output_change_warning_for_node(index);

        Some(match node.kind {
            NodeKind::Source => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                input_count: stage.input_count,
                output_count: stage.output_count,
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
                reference_consumers: reference_consumers.clone(),
                reference_output_warning: reference_output_warning.clone(),
                output_operator: None,
                null_operator: None,
                reference_input: None,
                python_operator: None,
                procedural_asset: None,
                native_operator: None,
            },
            NodeKind::Filter => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                input_count: stage.input_count,
                output_count: stage.output_count,
                status: if filter_warnings.is_empty() {
                    NodeStatus::Healthy
                } else {
                    NodeStatus::Warning
                },
                data_kind: "Filtered geometry",
                record_count: stage.output_count,
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
                reference_consumers: reference_consumers.clone(),
                reference_output_warning: reference_output_warning.clone(),
                output_operator: None,
                null_operator: None,
                reference_input: None,
                python_operator: None,
                procedural_asset: None,
                native_operator: None,
            },
            NodeKind::Style => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                input_count: stage.input_count,
                output_count: stage.output_count,
                status: if style_warnings.is_empty() {
                    NodeStatus::Healthy
                } else {
                    NodeStatus::Warning
                },
                data_kind: "Styled geometry",
                record_count: stage.output_count,
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
                reference_consumers: reference_consumers.clone(),
                reference_output_warning: reference_output_warning.clone(),
                output_operator: None,
                null_operator: None,
                reference_input: None,
                python_operator: None,
                procedural_asset: None,
                native_operator: None,
            },
            NodeKind::Null => {
                let contract = self.null_operator_contract(index)?;
                NodeInfo {
                    kind: node.kind,
                    role: node.kind.role(),
                    input_count: stage.input_count,
                    output_count: stage.output_count,
                    status: NodeStatus::Healthy,
                    data_kind: "Geometry table pass-through",
                    record_count: contract.output_record_count,
                    bounds: self.filtered_bounds(),
                    provenance: Some(contract.source_provenance),
                    attributes: self.source.metadata.attribute_names.clone(),
                    parameter: node.parameter.clone(),
                    summary: "Null operator is a visible typed pass-through anchor. OUT_* and IN_* are naming conventions only.",
                    source_metadata: None,
                    source_error: None,
                    style: Some(self.resolved_style()),
                    generated: node.generated,
                    evaluation: node.evaluation.clone(),
                    warnings: Vec::new(),
                    reference_consumers: reference_consumers.clone(),
                    reference_output_warning: reference_output_warning.clone(),
                    output_operator: None,
                    null_operator: Some(NullOperatorNodeInfo {
                        convention: contract.convention,
                        input_kind: contract.input_kind,
                        output_kind: contract.output_kind,
                        preserves_record_identity: contract.preserves_record_identity,
                        preserves_source_provenance: contract.preserves_source_provenance,
                    }),
                    reference_input: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                }
            }
            NodeKind::ReferenceInput => {
                let target_entries = self.reference_input_resolutions(index)?;
                let primary = target_entries.first()?;
                let warnings = target_entries
                    .iter()
                    .filter(|entry| entry.enabled)
                    .filter(|entry| entry.resolution.status != ReferenceDiagnosticStatus::Resolved)
                    .map(|entry| {
                        entry
                            .resolution
                            .diagnostic
                            .clone()
                            .unwrap_or_else(|| entry.resolution.status.as_str().to_owned())
                    })
                    .collect::<Vec<_>>();
                NodeInfo {
                    kind: node.kind,
                    role: node.kind.role(),
                    input_count: target_entries.iter().filter(|entry| entry.enabled).count(),
                    output_count: stage.output_count,
                    status: if warnings.is_empty() {
                        NodeStatus::Healthy
                    } else {
                        NodeStatus::Failed
                    },
                    data_kind: "Referenced geometry table",
                    record_count: stage.output_count,
                    bounds: (warnings.is_empty() && stage.output_count > 0)
                        .then(|| self.filtered_bounds())
                        .flatten(),
                    provenance: primary.resolution.source_provenance,
                    attributes: self.source.metadata.attribute_names.clone(),
                    parameter: node.parameter.clone(),
                    summary: "Reference input imports one compatible graph output by stable identity. It is live, one-way, and does not copy source data.",
                    source_metadata: None,
                    source_error: None,
                    style: None,
                    generated: node.generated,
                    evaluation: node.evaluation.clone(),
                    warnings,
                    reference_consumers: reference_consumers.clone(),
                    reference_output_warning: reference_output_warning.clone(),
                    output_operator: None,
                    null_operator: None,
                    reference_input: Some(ReferenceInputNodeInfo {
                        target: primary.resolution.target.clone(),
                        readable_path: primary.resolution.readable_path.clone(),
                        status: primary.resolution.status,
                        output_kind: primary.resolution.output_kind,
                        coordinate_contract: primary.resolution.coordinate_contract.clone(),
                        source_provenance: primary.resolution.source_provenance,
                        targets: target_entries
                            .into_iter()
                            .map(|entry| ReferenceTargetNodeInfo {
                                target: entry.resolution.target,
                                readable_path: entry.resolution.readable_path,
                                status: entry.resolution.status,
                                enabled: entry.enabled,
                                target_node_index: entry.resolution.target_node_index,
                                output_kind: entry.resolution.output_kind,
                                coordinate_contract: entry.resolution.coordinate_contract,
                                expected_coordinate_contract: entry.expected_coordinate_contract,
                                record_count: entry.resolution.record_count,
                                source_provenance: entry.resolution.source_provenance,
                                diagnostic: entry.resolution.diagnostic,
                                provenance: entry.provenance,
                            })
                            .collect(),
                        preserves_source_data: true,
                        applies_hidden_transform: false,
                    }),
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                }
            }
            NodeKind::SubstrateProjection => {
                let projection = node.substrate_projection.as_ref()?;
                NodeInfo {
                    kind: node.kind,
                    role: node.kind.role(),
                    input_count: 1,
                    output_count: stage.output_count,
                    status: NodeStatus::Healthy,
                    data_kind: "Projected geometry table",
                    record_count: stage.output_count,
                    bounds: self.filtered_bounds(),
                    provenance: Some(self.source.metadata.provenance),
                    attributes: self.source.metadata.attribute_names.clone(),
                    parameter: node.parameter.clone(),
                    summary: "Substrate projection is a visible graph operator created by assisted repair.",
                    source_metadata: None,
                    source_error: None,
                    style: None,
                    generated: node.generated,
                    evaluation: node.evaluation.clone(),
                    warnings: vec![format!(
                        "Projection contract: {}",
                        projection.repair_summary
                    )],
                    reference_consumers: reference_consumers.clone(),
                    reference_output_warning: reference_output_warning.clone(),
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                }
            }
            NodeKind::PythonOperator => {
                let python_operator = node.python_operator.as_ref()?;
                let declaration = self
                    .python_operator_declarations
                    .iter()
                    .find(|declaration| declaration.operator_id == python_operator.declaration_id);
                let dependency_status = self.python_operator_dependency_status(
                    declaration.map(|declaration| declaration.operator_id.as_str()),
                );
                let warnings = match dependency_status {
                    PythonOperatorDependencyStatus::Ready => Vec::new(),
                    _ => vec![dependency_status.summary().to_owned()],
                };
                NodeInfo {
                    kind: node.kind,
                    role: node.kind.role(),
                    input_count: declaration.map_or(0, |declaration| declaration.inputs.len()),
                    output_count: declaration.map_or(0, |declaration| declaration.outputs.len()),
                    status: match dependency_status {
                        PythonOperatorDependencyStatus::Ready => NodeStatus::Healthy,
                        PythonOperatorDependencyStatus::FailedEnvironment
                        | PythonOperatorDependencyStatus::DeclarationMissing => NodeStatus::Failed,
                        _ => NodeStatus::Warning,
                    },
                    data_kind: "Python geometry operator",
                    record_count: 0,
                    bounds: None,
                    provenance: Some(self.source.metadata.provenance),
                    attributes: declaration.map_or_else(Vec::new, |declaration| {
                        declaration.dependencies.requirements.clone()
                    }),
                    parameter: node.parameter.clone(),
                    summary: "Python operator is graph-visible but execution is deferred to the trusted project environment lane.",
                    source_metadata: None,
                    source_error: None,
                    style: None,
                    generated: node.generated,
                    evaluation: node.evaluation.clone(),
                    warnings,
                    reference_consumers: reference_consumers.clone(),
                    reference_output_warning: reference_output_warning.clone(),
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    python_operator: Some(PythonOperatorNodeInfo {
                        declaration_id: python_operator.declaration_id.clone(),
                        display_name: declaration
                            .map(|declaration| declaration.display_name.clone())
                            .unwrap_or_else(|| "Missing declaration".to_owned()),
                        version: declaration
                            .map(|declaration| declaration.version.clone())
                            .unwrap_or_else(|| "unknown".to_owned()),
                        dependency_status,
                        dependency_summary: dependency_status.summary().to_owned(),
                        requirements: declaration.map_or_else(Vec::new, |declaration| {
                            declaration.dependencies.requirements.clone()
                        }),
                        provenance_summary: python_operator.provenance_summary.clone(),
                        cache_key_summary: python_operator
                            .cache_key
                            .as_ref()
                            .map(PythonOperatorCacheKey::summary),
                        last_failure_summary: python_operator.last_failure_summary.clone(),
                    }),
                    procedural_asset: None,
                    native_operator: None,
                }
            }
            NodeKind::ProceduralAsset => {
                let asset_node = node.procedural_asset.as_ref()?;
                let declaration = self
                    .procedural_asset_declarations
                    .iter()
                    .find(|declaration| declaration.asset_id == asset_node.asset_id);
                let version_status = declaration
                    .map_or(OperatorVersionStatus::MissingDeclaration, |_| {
                        asset_node.version_status
                    });
                let warnings = match version_status {
                    OperatorVersionStatus::Current => Vec::new(),
                    _ => vec![format!("Asset version status: {}", version_status.as_str())],
                };
                NodeInfo {
                    kind: node.kind,
                    role: node.kind.role(),
                    input_count: declaration.map_or(0, |declaration| declaration.inputs.len()),
                    output_count: declaration.map_or(0, |declaration| declaration.outputs.len()),
                    status: match version_status {
                        OperatorVersionStatus::Current => NodeStatus::Healthy,
                        OperatorVersionStatus::MissingDeclaration
                        | OperatorVersionStatus::Incompatible => NodeStatus::Failed,
                        OperatorVersionStatus::NewerAvailable => NodeStatus::Warning,
                    },
                    data_kind: "Procedural asset",
                    record_count: self.visible_output_count(),
                    bounds: self.output_bounds(),
                    provenance: Some(self.source.metadata.provenance),
                    attributes: self.source.metadata.attribute_names.clone(),
                    parameter: node.parameter.clone(),
                    summary: "Procedural asset instance wraps a typed graph subgraph without depending on viewer state.",
                    source_metadata: None,
                    source_error: None,
                    style: None,
                    generated: node.generated,
                    evaluation: node.evaluation.clone(),
                    warnings,
                    reference_consumers: reference_consumers.clone(),
                    reference_output_warning: reference_output_warning.clone(),
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    python_operator: None,
                    procedural_asset: Some(ProceduralAssetNodeInfo {
                        asset_id: asset_node.asset_id.clone(),
                        display_name: declaration
                            .map(|declaration| declaration.display_name.clone())
                            .unwrap_or_else(|| "Missing asset declaration".to_owned()),
                        instance_version: asset_node.instance_version.clone(),
                        current_version: declaration.map(|declaration| declaration.version.clone()),
                        contents_unlocked: asset_node.contents_unlocked,
                        local_graph_id: asset_node
                            .contents_unlocked
                            .then(|| unlocked_asset_graph_id(&asset_node.instance_id)),
                        description: declaration
                            .map(|declaration| declaration.description.clone())
                            .unwrap_or_default(),
                        labels: declaration
                            .map(|declaration| declaration.labels.clone())
                            .unwrap_or_default(),
                        promoted_parameters: declaration
                            .map(|declaration| {
                                declaration
                                    .promoted_parameters
                                    .iter()
                                    .map(|parameter| parameter.name.clone())
                                    .collect()
                            })
                            .unwrap_or_default(),
                        input_bindings: asset_node.input_bindings.clone(),
                        output_summary: asset_node.output_summary.clone(),
                        version_status,
                    }),
                    native_operator: None,
                }
            }
            NodeKind::NativeOperator => {
                let native_node = node.native_operator.as_ref()?;
                let declaration = self
                    .native_operator_declarations
                    .iter()
                    .find(|declaration| declaration.operator_id == native_node.operator_id);
                let version_status = declaration
                    .map_or(OperatorVersionStatus::MissingDeclaration, |_| {
                        native_node.version_status
                    });
                let load_status = self.native_operator_load_status(
                    declaration.map(|declaration| declaration.operator_id.as_str()),
                );
                let mut warnings = match version_status {
                    OperatorVersionStatus::Current => Vec::new(),
                    _ => vec![format!(
                        "Native operator version status: {}",
                        version_status.as_str()
                    )],
                };
                if load_status != NativeOperatorLoadStatus::Ready {
                    warnings.push(load_status.summary().to_owned());
                }
                NodeInfo {
                    kind: node.kind,
                    role: node.kind.role(),
                    input_count: declaration.map_or(0, |declaration| declaration.inputs.len()),
                    output_count: declaration.map_or(0, |declaration| declaration.outputs.len()),
                    status: native_operator_node_status(version_status, load_status),
                    data_kind: "Trusted native operator",
                    record_count: self.visible_output_count(),
                    bounds: self.output_bounds(),
                    provenance: Some(self.source.metadata.provenance),
                    attributes: self.source.metadata.attribute_names.clone(),
                    parameter: node.parameter.clone(),
                    summary: "Native operator node is graph-visible; loading and execution are handled by the trusted native lane.",
                    source_metadata: None,
                    source_error: None,
                    style: None,
                    generated: node.generated,
                    evaluation: node.evaluation.clone(),
                    warnings,
                    reference_consumers: reference_consumers.clone(),
                    reference_output_warning: reference_output_warning.clone(),
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: Some(NativeOperatorNodeInfo {
                        operator_id: native_node.operator_id.clone(),
                        display_name: declaration
                            .map(|declaration| declaration.display_name.clone())
                            .unwrap_or_else(|| "Missing native declaration".to_owned()),
                        version: declaration
                            .map(|declaration| declaration.version.clone())
                            .unwrap_or_else(|| "unknown".to_owned()),
                        host_compatibility_version: declaration
                            .map(|declaration| declaration.host_compatibility_version.clone())
                            .unwrap_or_else(|| "unknown".to_owned()),
                        inputs: declaration
                            .map(|declaration| port_names(&declaration.inputs))
                            .unwrap_or_default(),
                        outputs: declaration
                            .map(|declaration| port_names(&declaration.outputs))
                            .unwrap_or_default(),
                        parameters: declaration
                            .map(|declaration| {
                                declaration
                                    .parameters
                                    .iter()
                                    .map(|parameter| parameter.name.clone())
                                    .collect()
                            })
                            .unwrap_or_default(),
                        capabilities: declaration
                            .map(|declaration| {
                                declaration
                                    .capabilities
                                    .iter()
                                    .map(|capability| format!("{capability:?}"))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        provenance_summary: declaration
                            .map(|declaration| declaration.provenance.summary())
                            .unwrap_or_else(|| "none".to_owned()),
                        output_provenance_summary: native_node.provenance_summary.clone(),
                        cache_key_summary: native_node
                            .cache_key
                            .as_ref()
                            .map(NativeOperatorCacheKey::summary),
                        failure_modes: declaration
                            .map(|declaration| {
                                declaration
                                    .failure_modes
                                    .iter()
                                    .map(NativeOperatorFailureMode::summary)
                                    .collect()
                            })
                            .unwrap_or_default(),
                        version_status,
                        load_status,
                        last_valid_cache_key: native_node.last_valid_cache_key.clone(),
                        last_failure_summary: native_node.last_failure_summary.clone(),
                    }),
                }
            }
            NodeKind::Output => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                input_count: stage.input_count,
                output_count: stage.output_count,
                status: NodeStatus::Healthy,
                data_kind: "Rerun scene output",
                record_count: stage.output_count,
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
                reference_consumers,
                reference_output_warning,
                output_operator: node.output_operator.as_ref().map(|output_operator| {
                    OutputOperatorNodeInfo {
                        kind: output_operator.kind,
                        semantic_payload: output_operator.contract.semantic_payload,
                        command: output_operator.contract.command,
                        preferred_target: output_operator.contract.preferred_target,
                        negotiations: [
                            OutputTargetId::GenericGraph,
                            OutputTargetId::Rerun,
                            OutputTargetId::DebugPreparedPolyline,
                            OutputTargetId::UnsupportedExternal,
                        ]
                        .into_iter()
                        .map(|target| self.negotiate_output_target(output_operator, target))
                        .collect(),
                        rerun_options: output_operator.rerun_options.clone(),
                        graph_viewport_state_separate: true,
                    }
                }),
                null_operator: None,
                reference_input: None,
                python_operator: None,
                procedural_asset: None,
                native_operator: None,
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

    #[allow(dead_code)]
    pub fn output_target_contract_for_node(
        &self,
        node_index: usize,
    ) -> Option<OutputTargetContract> {
        self.nodes
            .get(node_index)?
            .output_operator
            .as_ref()
            .map(|operator| operator.contract.clone())
    }

    #[allow(dead_code)]
    pub fn negotiate_output_target_for_node(
        &self,
        node_index: usize,
        target: OutputTargetId,
    ) -> Option<OutputTargetNegotiation> {
        let node = self.nodes.get(node_index)?;
        let output_operator = node.output_operator.as_ref()?;
        Some(self.negotiate_output_target(output_operator, target))
    }

    fn negotiate_output_target(
        &self,
        output_operator: &OutputOperatorNode,
        target: OutputTargetId,
    ) -> OutputTargetNegotiation {
        match target {
            OutputTargetId::GenericGraph => OutputTargetNegotiation {
                target,
                mapping: OutputCapabilityMapping::NativeMapping,
                reason: "Graph-owned layered geometry can be consumed through the generic output contract.".to_owned(),
            },
            OutputTargetId::Rerun => {
                let has_native_cubic = self
                    .active_geometry()
                    .iter()
                    .filter(|geometry| self.emits(geometry))
                    .any(|geometry| matches!(geometry, Geometry::CubicBezier(_)));
                if output_operator.kind == OutputOperatorKind::RerunSpecialized
                    && !has_native_cubic
                {
                    OutputTargetNegotiation {
                        target,
                        mapping: OutputCapabilityMapping::NativeMapping,
                        reason: "Rerun target can map the current polygon output natively through its adapter.".to_owned(),
                    }
                } else if output_operator.kind == OutputOperatorKind::RerunSpecialized
                    && has_native_cubic
                {
                    OutputTargetNegotiation {
                        target,
                        mapping: OutputCapabilityMapping::LowerFidelityWithWarning,
                        reason: "Rerun target preserves cubic control points as graph metadata but visualizes cubic curves through adapter-owned control points and control-polygon previews.".to_owned(),
                    }
                } else {
                    OutputTargetNegotiation {
                        target,
                        mapping: OutputCapabilityMapping::PreparedRepresentation,
                        reason: "Generic output can be adapted to Rerun through a declared prepared scene representation.".to_owned(),
                    }
                }
            }
            OutputTargetId::DebugPreparedPolyline => OutputTargetNegotiation {
                target,
                mapping: OutputCapabilityMapping::PreparedRepresentation,
                reason: "Dense polyline data is a declared debug/export representation at the output boundary.".to_owned(),
            },
            OutputTargetId::UnsupportedExternal => OutputTargetNegotiation {
                target,
                mapping: OutputCapabilityMapping::Unsupported,
                reason: "No output target adapter has declared support for this target.".to_owned(),
            },
        }
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

#[cfg(not(target_arch = "wasm32"))]
const PYTHON_GEOMETRY_EXCHANGE_VERSION: u32 = 1;

#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Deserialize, serde::Serialize)]
struct PythonGeometryExchange {
    schema_version: u32,
    records: Vec<PythonGeometryExchangeRecord>,
}

#[cfg(not(target_arch = "wasm32"))]
impl PythonGeometryExchange {
    fn from_geometry(geometry: &[Geometry]) -> Self {
        Self {
            schema_version: PYTHON_GEOMETRY_EXCHANGE_VERSION,
            records: geometry
                .iter()
                .map(|geometry| PythonGeometryExchangeRecord {
                    kind: geometry.kind(),
                    layer: geometry.layer(),
                    score: geometry.score(),
                    geometry: geometry.clone(),
                })
                .collect(),
        }
    }

    fn into_geometry(self) -> anyhow::Result<Vec<Geometry>> {
        if self.schema_version != PYTHON_GEOMETRY_EXCHANGE_VERSION {
            anyhow::bail!(
                "unsupported Houdini Python geometry exchange version {}; expected {}",
                self.schema_version,
                PYTHON_GEOMETRY_EXCHANGE_VERSION
            );
        }

        self.records
            .into_iter()
            .map(PythonGeometryExchangeRecord::into_geometry)
            .collect()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Deserialize, serde::Serialize)]
struct PythonGeometryExchangeRecord {
    kind: GeometryKind,
    layer: LayerKind,
    score: f32,
    geometry: Geometry,
}

#[cfg(not(target_arch = "wasm32"))]
impl PythonGeometryExchangeRecord {
    fn into_geometry(self) -> anyhow::Result<Geometry> {
        match (&self.kind, &self.geometry) {
            (GeometryKind::Polygon, Geometry::Polygon(_))
            | (GeometryKind::CubicBezier, Geometry::CubicBezier(_)) => Ok(self.geometry),
            _ => {
                anyhow::bail!("Houdini Python geometry record kind did not match geometry payload")
            }
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
    PythonOperator,
}

impl SourceProvenance {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DemoFallback => "demo fallback",
            Self::ParquetImport => "parquet import",
            Self::RecordingQuery => "recording query",
            Self::SyntheticBenchmark => "synthetic benchmark",
            Self::PythonOperator => "python operator",
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
    #[serde(default)]
    annotations: Vec<GraphAnnotation>,
    #[serde(default)]
    network_view: NetworkViewDisplayOptions,
    layers: Vec<LayerSidecar>,
    #[serde(default)]
    style: GraphStyle,
    demo_geometry: Vec<Geometry>,
    recording_geometry: Vec<Geometry>,
    #[serde(default)]
    python_operator_declarations: Vec<PythonOperatorDeclaration>,
    #[serde(default)]
    procedural_asset_declarations: Vec<ProceduralAssetDeclaration>,
    #[serde(default)]
    native_operator_declarations: Vec<NativeOperatorDeclaration>,
    #[serde(default)]
    native_operator_trust: NativeOperatorTrustPolicy,
    #[serde(default)]
    python_environment: PythonEnvironmentDescriptor,
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
                    node_id: node.node_id.clone(),
                    name: node.name.clone(),
                    kind: node.kind,
                    layout_position: node.layout_position,
                    parameter_value: node.parameter.value,
                    parameter_rule: node.parameter.rule_spec.clone(),
                    generated: node.generated,
                    coordinate_contract: Some(node.coordinate_contract.clone()),
                    output_operator: node.output_operator.clone(),
                    null_operator: node.null_operator.clone(),
                    reference_input: node.reference_input.clone(),
                    substrate_projection: node.substrate_projection.clone(),
                    python_operator: node.python_operator.clone(),
                    procedural_asset: node.procedural_asset.clone(),
                    native_operator: node.native_operator.clone(),
                    comment: node.comment.clone(),
                    show_comment_in_network: node.show_comment_in_network,
                })
                .collect(),
            annotations: graph.annotations.clone(),
            network_view: graph.network_view,
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
            python_operator_declarations: graph.python_operator_declarations.clone(),
            procedural_asset_declarations: graph.procedural_asset_declarations.clone(),
            native_operator_declarations: graph.native_operator_declarations.clone(),
            native_operator_trust: graph.native_operator_trust.clone(),
            python_environment: graph.python_environment.clone(),
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
        graph.annotations = self.annotations;
        graph.network_view = self.network_view;
        graph.style = self.style;
        graph.python_operator_declarations = self.python_operator_declarations;
        graph.procedural_asset_declarations = self.procedural_asset_declarations;
        graph.native_operator_declarations = self.native_operator_declarations;
        graph.native_operator_trust = self.native_operator_trust;
        graph.python_environment = self.python_environment;

        for (snapshot_index, node_snapshot) in self.nodes.into_iter().enumerate() {
            let matching_node = graph.nodes.iter_mut().find(|node| {
                node.kind == node_snapshot.kind
                    && node_matches_snapshot_identity(node, &node_snapshot)
            });
            if let Some(node) = matching_node {
                if !node_snapshot.name.is_empty() {
                    node.name = node_snapshot.name;
                }
                node.layout_position = node_snapshot.layout_position;
                node.parameter.value = node_snapshot
                    .parameter_value
                    .clamp(*node.parameter.range.start(), *node.parameter.range.end());
                if let Some(parameter_rule) = node_snapshot.parameter_rule {
                    node.parameter.rule_spec = Some(parameter_rule);
                }
                if !node_snapshot.node_id.is_empty() {
                    node.node_id = node_snapshot.node_id;
                }
                node.generated = node_snapshot.generated;
                node.coordinate_contract = node_snapshot.coordinate_contract.unwrap_or_else(|| {
                    GraphDocument::default_coordinate_contract_for_kind(node.kind)
                });
                node.output_operator = node_snapshot.output_operator.or_else(|| {
                    (node.kind == NodeKind::Output).then(OutputOperatorNode::rerun_scene)
                });
                node.python_operator = node_snapshot.python_operator;
                node.reference_input = node_snapshot.reference_input;
                node.substrate_projection = node_snapshot.substrate_projection;
                node.procedural_asset = node_snapshot.procedural_asset;
                node.native_operator = node_snapshot.native_operator;
                node.comment = node_snapshot.comment;
                node.show_comment_in_network = node_snapshot.show_comment_in_network;
            } else if node_snapshot.is_instance_node() {
                let insert_index = snapshot_index.min(graph.nodes.len());
                graph
                    .nodes
                    .insert(insert_index, node_snapshot.into_instance_node());
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
    #[serde(default)]
    node_id: String,
    #[serde(default)]
    name: String,
    kind: NodeKind,
    layout_position: GraphPoint,
    parameter_value: f32,
    #[serde(default)]
    parameter_rule: Option<AttributeFilterRuleSpec>,
    #[serde(default)]
    generated: Option<GeneratedNodeInfo>,
    #[serde(default)]
    coordinate_contract: Option<Option<SubstrateCoordinateContract>>,
    #[serde(default)]
    output_operator: Option<OutputOperatorNode>,
    #[serde(default)]
    null_operator: Option<NullOperatorNode>,
    #[serde(default)]
    reference_input: Option<ReferenceInputNode>,
    #[serde(default)]
    substrate_projection: Option<SubstrateProjectionNode>,
    #[serde(default)]
    python_operator: Option<PythonOperatorNode>,
    #[serde(default)]
    procedural_asset: Option<ProceduralAssetInstanceNode>,
    #[serde(default)]
    native_operator: Option<NativeOperatorNode>,
    #[serde(default)]
    comment: String,
    #[serde(default)]
    show_comment_in_network: bool,
}

impl NodeSidecar {
    fn is_instance_node(&self) -> bool {
        matches!(
            self.kind,
            NodeKind::Null
                | NodeKind::ReferenceInput
                | NodeKind::SubstrateProjection
                | NodeKind::PythonOperator
                | NodeKind::ProceduralAsset
                | NodeKind::NativeOperator
        )
    }

    fn into_instance_node(self) -> GraphNode {
        let (name, info) = match self.kind {
            NodeKind::PythonOperator => (
                "Python Operator",
                "Runs trusted project Python against typed graph inputs once execution is enabled.",
            ),
            NodeKind::ProceduralAsset => (
                "Asset",
                "Runs a graph-backed procedural asset without calling viewer APIs.",
            ),
            NodeKind::NativeOperator => (
                "Native Operator",
                "Runs a trusted native operator once a loader is available.",
            ),
            NodeKind::Null => (
                "Null",
                "Passes typed geometry through unchanged as a visible graph anchor.",
            ),
            NodeKind::ReferenceInput => (
                "Reference Input",
                "Imports one compatible graph output as a live one-way dependency.",
            ),
            NodeKind::SubstrateProjection => (
                "Substrate Projection",
                "Converts substrate coordinates as a visible graph operator.",
            ),
            _ => ("Graph Node", "Restored graph node."),
        };
        GraphNode {
            node_id: if self.node_id.is_empty() {
                stable_digest(&serde_json::json!({
                    "kind": self.kind,
                    "name": &self.name,
                    "position": self.layout_position,
                }))
            } else {
                self.node_id
            },
            name: if self.name.is_empty() {
                name.to_owned()
            } else {
                self.name
            },
            kind: self.kind,
            layout_position: self.layout_position,
            generated: self.generated,
            coordinate_contract: self
                .coordinate_contract
                .unwrap_or_else(|| GraphDocument::default_coordinate_contract_for_kind(self.kind)),
            output_operator: self.output_operator,
            null_operator: self.null_operator,
            reference_input: self.reference_input,
            substrate_projection: self.substrate_projection,
            python_operator: self.python_operator,
            procedural_asset: self.procedural_asset,
            native_operator: self.native_operator,
            evaluation: NodeEvaluation::clean(),
            participates_in_output: true,
            comment: self.comment,
            show_comment_in_network: self.show_comment_in_network,
            parameter: NodeParameter::scalar(
                "Run",
                self.parameter_value.clamp(0.0, 1.0),
                0.0..=1.0,
                "Manual readiness placeholder for a graph-visible Python operator.",
            ),
            info,
        }
    }
}

fn node_matches_snapshot_identity(node: &GraphNode, snapshot: &NodeSidecar) -> bool {
    match node.kind {
        NodeKind::Null | NodeKind::ReferenceInput | NodeKind::SubstrateProjection => {
            if snapshot.node_id.is_empty() {
                node.name == snapshot.name
            } else {
                node.node_id == snapshot.node_id
            }
        }
        NodeKind::PythonOperator => {
            node.python_operator.as_ref().and_then(|python_operator| {
                snapshot
                    .python_operator
                    .as_ref()
                    .map(|snapshot| python_operator.instance_id == snapshot.instance_id)
            }) == Some(true)
        }
        NodeKind::ProceduralAsset => {
            node.procedural_asset.as_ref().and_then(|asset| {
                snapshot
                    .procedural_asset
                    .as_ref()
                    .map(|snapshot| asset.instance_id == snapshot.instance_id)
            }) == Some(true)
        }
        NodeKind::NativeOperator => {
            node.native_operator.as_ref().and_then(|native| {
                snapshot
                    .native_operator
                    .as_ref()
                    .map(|snapshot| native.instance_id == snapshot.instance_id)
            }) == Some(true)
        }
        _ => true,
    }
}

fn readable_reference_path(node: &GraphNode, output_name: &str) -> String {
    format!("{MAIN_GRAPH_ID}/{}:{output_name}", node.name)
}

fn unlocked_asset_graph_id(instance_id: &str) -> String {
    format!("{instance_id}.local_edit_graph")
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
    pub node_id: String,
    pub name: String,
    pub kind: NodeKind,
    pub layout_position: GraphPoint,
    pub generated: Option<GeneratedNodeInfo>,
    pub coordinate_contract: Option<SubstrateCoordinateContract>,
    pub output_operator: Option<OutputOperatorNode>,
    pub null_operator: Option<NullOperatorNode>,
    pub reference_input: Option<ReferenceInputNode>,
    pub substrate_projection: Option<SubstrateProjectionNode>,
    pub python_operator: Option<PythonOperatorNode>,
    pub procedural_asset: Option<ProceduralAssetInstanceNode>,
    pub native_operator: Option<NativeOperatorNode>,
    pub evaluation: NodeEvaluation,
    pub participates_in_output: bool,
    pub comment: String,
    pub show_comment_in_network: bool,
    pub parameter: NodeParameter,
    pub info: &'static str,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GraphAnnotation {
    pub annotation_id: String,
    pub kind: GraphAnnotationKind,
    pub title: String,
    pub text: String,
    pub position: GraphPoint,
    pub size: GraphPoint,
    pub collapsed: bool,
    pub member_node_ids: Vec<String>,
}

impl GraphAnnotation {
    fn network_box(
        annotation_id: String,
        title: String,
        position: GraphPoint,
        size: GraphPoint,
        member_node_ids: Vec<String>,
    ) -> Self {
        Self {
            annotation_id,
            kind: GraphAnnotationKind::NetworkBox,
            title,
            text: String::new(),
            position,
            size,
            collapsed: false,
            member_node_ids,
        }
    }

    fn sticky_note(
        annotation_id: String,
        title: String,
        text: String,
        position: GraphPoint,
        size: GraphPoint,
    ) -> Self {
        Self {
            annotation_id,
            kind: GraphAnnotationKind::StickyNote,
            title,
            text,
            position,
            size,
            collapsed: false,
            member_node_ids: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum GraphAnnotationKind {
    NetworkBox,
    StickyNote,
}

impl GraphAnnotationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NetworkBox => "Network Box",
            Self::StickyNote => "Sticky Note",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct NetworkViewDisplayOptions {
    #[serde(default)]
    pub node_ring_visibility: NetworkNodeRingVisibility,
    #[serde(default = "default_max_node_name_width")]
    pub max_node_name_width: f32,
    #[serde(default = "default_long_wire_fading")]
    pub long_wire_fading: f32,
    #[serde(default = "default_grid_spacing")]
    pub grid_spacing: f32,
    #[serde(default = "default_background_brightness")]
    pub background_brightness: f32,
    #[serde(default)]
    pub error_badge: NetworkBadgeVisibility,
    #[serde(default)]
    pub warning_badge: NetworkBadgeVisibility,
    #[serde(default)]
    pub comment_badge: NetworkBadgeVisibility,
    #[serde(default)]
    pub time_dependent_badge: NetworkBadgeVisibility,
    #[serde(default)]
    pub lock_badge: NetworkBadgeVisibility,
    #[serde(default)]
    pub has_data_badge: NetworkBadgeVisibility,
    #[serde(default)]
    pub cached_code_badge: NetworkBadgeVisibility,
    #[serde(default)]
    pub constraint_badge: NetworkBadgeVisibility,
    #[serde(default)]
    pub compilable_badge: NetworkBadgeVisibility,
}

impl Default for NetworkViewDisplayOptions {
    fn default() -> Self {
        Self {
            node_ring_visibility: NetworkNodeRingVisibility::Selected,
            max_node_name_width: default_max_node_name_width(),
            long_wire_fading: default_long_wire_fading(),
            grid_spacing: default_grid_spacing(),
            background_brightness: default_background_brightness(),
            error_badge: NetworkBadgeVisibility::Large,
            warning_badge: NetworkBadgeVisibility::Normal,
            comment_badge: NetworkBadgeVisibility::Large,
            time_dependent_badge: NetworkBadgeVisibility::Normal,
            lock_badge: NetworkBadgeVisibility::Normal,
            has_data_badge: NetworkBadgeVisibility::Normal,
            cached_code_badge: NetworkBadgeVisibility::Normal,
            constraint_badge: NetworkBadgeVisibility::Normal,
            compilable_badge: NetworkBadgeVisibility::Normal,
        }
    }
}

fn default_max_node_name_width() -> f32 {
    96.0
}

fn default_long_wire_fading() -> f32 {
    0.7
}

fn default_grid_spacing() -> f32 {
    2.0
}

fn default_background_brightness() -> f32 {
    0.12
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum NetworkNodeRingVisibility {
    Hidden,
    #[default]
    Selected,
    Always,
}

impl NetworkNodeRingVisibility {
    pub const ALL: [Self; 3] = [Self::Selected, Self::Always, Self::Hidden];

    pub fn label(self) -> &'static str {
        match self {
            Self::Hidden => "Hidden",
            Self::Selected => "Selected",
            Self::Always => "Always",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum NetworkBadgeVisibility {
    Hide,
    #[default]
    Normal,
    Large,
}

impl NetworkBadgeVisibility {
    pub const ALL: [Self; 3] = [Self::Large, Self::Normal, Self::Hide];

    pub fn label(self) -> &'static str {
        match self {
            Self::Hide => "Hide",
            Self::Normal => "Normal",
            Self::Large => "Large",
        }
    }

    pub fn radius(self) -> Option<f32> {
        match self {
            Self::Hide => None,
            Self::Normal => Some(4.0),
            Self::Large => Some(5.5),
        }
    }
}

fn network_box_contains_position(annotation: &GraphAnnotation, point: GraphPoint) -> bool {
    point.x >= annotation.position.x
        && point.x <= annotation.position.x + annotation.size.x
        && point.y >= annotation.position.y
        && point.y <= annotation.position.y + annotation.size.y
}

fn expand_network_box_to_include_position(annotation: &mut GraphAnnotation, point: GraphPoint) {
    let (position, size) = network_box_bounds_for_positions(&[point])
        .unwrap_or((annotation.position, annotation.size));
    let min_x = annotation.position.x.min(position.x);
    let min_y = annotation.position.y.min(position.y);
    let max_x = (annotation.position.x + annotation.size.x)
        .max(position.x + size.x)
        .max(min_x + 0.08);
    let max_y = (annotation.position.y + annotation.size.y)
        .max(position.y + size.y)
        .max(min_y + 0.08);

    annotation.position = GraphPoint::new(min_x, min_y);
    annotation.size = GraphPoint::new(max_x - min_x, max_y - min_y);
}

fn network_box_bounds_for_positions(positions: &[GraphPoint]) -> Option<(GraphPoint, GraphPoint)> {
    let first = positions.first()?;
    let minimum_size = 0.08;
    let padding = GraphPoint::new(0.08, 0.14);
    let mut min_x = first.x - padding.x;
    let mut min_y = first.y - padding.y;
    let mut max_x = first.x + padding.x;
    let mut max_y = first.y + padding.y;

    for position in &positions[1..] {
        min_x = min_x.min(position.x - padding.x);
        min_y = min_y.min(position.y - padding.y);
        max_x = max_x.max(position.x + padding.x);
        max_y = max_y.max(position.y + padding.y);
    }

    let max_x = max_x.max(min_x + minimum_size);
    let max_y = max_y.max(min_y + minimum_size);

    Some((
        GraphPoint::new(min_x, min_y),
        GraphPoint::new(max_x - min_x, max_y - min_y),
    ))
}

impl GraphNode {
    fn null_operator(name: String) -> Self {
        Self {
            node_id: String::new(),
            name,
            kind: NodeKind::Null,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
            output_operator: None,
            null_operator: Some(NullOperatorNode {
                input_kind: HoudiniDataKind::GeometryTable,
                output_kind: HoudiniDataKind::GeometryTable,
            }),
            reference_input: None,
            substrate_projection: None,
            python_operator: None,
            procedural_asset: None,
            native_operator: None,
            evaluation: NodeEvaluation::clean(),
            participates_in_output: true,
            comment: String::new(),
            show_comment_in_network: false,
            parameter: NodeParameter::scalar(
                "Pass-through",
                1.0,
                0.0..=1.0,
                "Typed pass-through anchor; the parameter is inspect-only in this spike.",
            ),
            info: "Passes compatible typed geometry through unchanged as a visible named graph anchor.",
        }
    }

    fn reference_input(node_id: String, target: ReferenceTargetEntry) -> Self {
        Self {
            node_id,
            name: "Reference Input".to_owned(),
            kind: NodeKind::ReferenceInput,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
            output_operator: None,
            null_operator: None,
            reference_input: Some(ReferenceInputNode {
                targets: vec![target],
            }),
            substrate_projection: None,
            python_operator: None,
            procedural_asset: None,
            native_operator: None,
            evaluation: NodeEvaluation::clean(),
            participates_in_output: true,
            comment: String::new(),
            show_comment_in_network: false,
            parameter: NodeParameter::scalar(
                "Live reference",
                1.0,
                0.0..=1.0,
                "Live one-way reference; does not copy source data or apply hidden transforms.",
            ),
            info: "Imports one compatible graph output by stable identity while showing a readable path.",
        }
    }

    fn substrate_projection(instance_id: String, projection: SubstrateProjectionNode) -> Self {
        Self {
            node_id: instance_id,
            name: "Substrate Projection".to_owned(),
            kind: NodeKind::SubstrateProjection,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: Some(projection.to_contract.clone()),
            output_operator: None,
            null_operator: None,
            reference_input: None,
            substrate_projection: Some(projection),
            python_operator: None,
            procedural_asset: None,
            native_operator: None,
            evaluation: NodeEvaluation::clean(),
            participates_in_output: true,
            comment: String::new(),
            show_comment_in_network: false,
            parameter: NodeParameter::scalar(
                "Projected",
                1.0,
                0.0..=1.0,
                "Visible assisted substrate projection; no reference input transform is hidden.",
            ),
            info: "Converts a referenced substrate coordinate contract through a visible graph node.",
        }
    }

    fn python_operator(instance_id: String, declaration_id: String) -> Self {
        Self {
            node_id: instance_id.clone(),
            name: "Python Operator".to_owned(),
            kind: NodeKind::PythonOperator,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
            output_operator: None,
            null_operator: None,
            reference_input: None,
            substrate_projection: None,
            python_operator: Some(PythonOperatorNode {
                instance_id,
                declaration_id,
                provenance_summary: None,
                cache_key: None,
                provenance: None,
                last_failure_summary: None,
            }),
            procedural_asset: None,
            native_operator: None,
            evaluation: NodeEvaluation {
                state: EvaluationState::Manual,
                manual: true,
                message: Some("Python execution is not enabled for this node yet.".to_owned()),
            },
            participates_in_output: true,
            comment: String::new(),
            show_comment_in_network: false,
            parameter: NodeParameter::scalar(
                "Run",
                0.0,
                0.0..=1.0,
                "Manual readiness placeholder for a graph-visible Python operator.",
            ),
            info: "Runs trusted project Python against typed graph inputs once execution is enabled.",
        }
    }

    fn procedural_asset(instance_id: String, asset_id: String, instance_version: String) -> Self {
        Self {
            node_id: instance_id.clone(),
            name: "Asset".to_owned(),
            kind: NodeKind::ProceduralAsset,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
            output_operator: None,
            null_operator: None,
            reference_input: None,
            substrate_projection: None,
            python_operator: None,
            procedural_asset: Some(ProceduralAssetInstanceNode {
                instance_id,
                asset_id,
                instance_version,
                contents_unlocked: false,
                input_bindings: vec![HoudiniNodeBinding {
                    port_name: "geometry".to_owned(),
                    source_summary: "previous output".to_owned(),
                }],
                output_summary: None,
                version_status: OperatorVersionStatus::Current,
            }),
            native_operator: None,
            evaluation: NodeEvaluation::clean(),
            participates_in_output: true,
            comment: String::new(),
            show_comment_in_network: false,
            parameter: NodeParameter::scalar(
                "Bypass",
                0.0,
                0.0..=1.0,
                "Asset node readiness placeholder.",
            ),
            info: "Runs a graph-backed procedural asset without calling viewer APIs.",
        }
    }

    fn native_operator(instance_id: String, operator_id: String) -> Self {
        Self {
            node_id: instance_id.clone(),
            name: "Native Operator".to_owned(),
            kind: NodeKind::NativeOperator,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
            output_operator: None,
            null_operator: None,
            reference_input: None,
            substrate_projection: None,
            python_operator: None,
            procedural_asset: None,
            native_operator: Some(NativeOperatorNode {
                instance_id,
                operator_id,
                version_status: OperatorVersionStatus::Current,
                cache_key: None,
                provenance: None,
                provenance_summary: None,
                last_valid_cache_key: None,
                last_failure_summary: None,
            }),
            evaluation: NodeEvaluation::clean(),
            participates_in_output: true,
            comment: String::new(),
            show_comment_in_network: false,
            parameter: NodeParameter::scalar(
                "Bypass",
                0.0,
                0.0..=1.0,
                "Native operator readiness placeholder.",
            ),
            info: "Runs a trusted native operator once a loader is available.",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct NullOperatorNode {
    pub input_kind: HoudiniDataKind,
    pub output_kind: HoudiniDataKind,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct OutputOperatorNode {
    pub kind: OutputOperatorKind,
    pub contract: OutputTargetContract,
    pub rerun_options: Option<RerunOutputTargetOptions>,
}

impl OutputOperatorNode {
    fn rerun_scene() -> Self {
        Self {
            kind: OutputOperatorKind::RerunSpecialized,
            contract: OutputTargetContract {
                semantic_payload: OutputSemanticPayload::LayeredGeometry,
                command: OutputCommand::ComposeScene,
                preferred_target: Some(OutputTargetId::Rerun),
            },
            rerun_options: Some(RerunOutputTargetOptions {
                include_debug_items: true,
                preserve_native_cubic_metadata: true,
            }),
        }
    }

    #[allow(dead_code)]
    fn generic_scene() -> Self {
        Self {
            kind: OutputOperatorKind::Generic,
            contract: OutputTargetContract {
                semantic_payload: OutputSemanticPayload::LayeredGeometry,
                command: OutputCommand::ComposeScene,
                preferred_target: None,
            },
            rerun_options: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum OutputOperatorKind {
    Generic,
    RerunSpecialized,
}

impl OutputOperatorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Generic => "Generic",
            Self::RerunSpecialized => "Rerun specialized",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct OutputTargetContract {
    pub semantic_payload: OutputSemanticPayload,
    pub command: OutputCommand,
    pub preferred_target: Option<OutputTargetId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum OutputSemanticPayload {
    LayeredGeometry,
}

impl OutputSemanticPayload {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LayeredGeometry => "Layered geometry",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum OutputCommand {
    ComposeScene,
    SaveRecording,
}

impl OutputCommand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ComposeScene => "Compose scene",
            Self::SaveRecording => "Save recording",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum OutputTargetId {
    GenericGraph,
    Rerun,
    DebugPreparedPolyline,
    UnsupportedExternal,
}

impl OutputTargetId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GenericGraph => "Generic graph",
            Self::Rerun => "Rerun",
            Self::DebugPreparedPolyline => "Debug prepared polyline",
            Self::UnsupportedExternal => "Unsupported external",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct RerunOutputTargetOptions {
    pub include_debug_items: bool,
    pub preserve_native_cubic_metadata: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OutputTargetNegotiation {
    pub target: OutputTargetId,
    pub mapping: OutputCapabilityMapping,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OutputCapabilityMapping {
    NativeMapping,
    PreparedRepresentation,
    LowerFidelityWithWarning,
    Unsupported,
}

impl OutputCapabilityMapping {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NativeMapping => "Native mapping",
            Self::PreparedRepresentation => "Prepared representation",
            Self::LowerFidelityWithWarning => "Lower fidelity with warning",
            Self::Unsupported => "Unsupported",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct SubstrateProjectionNode {
    pub source_target: ReferenceTargetIdentity,
    pub from_contract: SubstrateCoordinateContract,
    pub to_contract: SubstrateCoordinateContract,
    pub repair_summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct SubstrateCoordinateContract {
    pub substrate_id: String,
    pub width: u32,
    pub height: u32,
    pub origin: SubstrateOrigin,
    pub y_axis: SubstrateYAxis,
}

impl SubstrateCoordinateContract {
    fn demo_byteplot() -> Self {
        Self {
            substrate_id: "demo-byteplot-pixel-space".to_owned(),
            width: 1024,
            height: 1024,
            origin: SubstrateOrigin::TopLeft,
            y_axis: SubstrateYAxis::Down,
        }
    }

    fn repair_summary_to(&self, to_contract: &Self) -> String {
        format!(
            "{} {}x{} {:?}/{:?} -> {} {}x{} {:?}/{:?}",
            self.substrate_id,
            self.width,
            self.height,
            self.origin,
            self.y_axis,
            to_contract.substrate_id,
            to_contract.width,
            to_contract.height,
            to_contract.origin,
            to_contract.y_axis
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SubstrateOrigin {
    TopLeft,
    BottomLeft,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SubstrateYAxis {
    Down,
    Up,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct ReferenceInputNode {
    pub targets: Vec<ReferenceTargetEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct ReferenceTargetEntry {
    pub target: ReferenceTargetIdentity,
    pub enabled: bool,
    pub provenance: ReferenceTargetProvenance,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct ReferenceTargetIdentity {
    pub graph_id: String,
    pub node_id: String,
    pub output_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct ReferenceTargetProvenance {
    pub source_graph_id: String,
    pub source_node_id: String,
    pub source_node_name: String,
    pub source_output_name: String,
    pub source_data_kind: HoudiniDataKind,
}

impl ReferenceTargetProvenance {
    fn from_node(node: &GraphNode, target: &ReferenceTargetIdentity) -> Self {
        Self {
            source_graph_id: target.graph_id.clone(),
            source_node_id: node.node_id.clone(),
            source_node_name: node.name.clone(),
            source_output_name: target.output_name.clone(),
            source_data_kind: HoudiniDataKind::GeometryTable,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReferenceTargetEntryResolution {
    pub enabled: bool,
    pub provenance: ReferenceTargetProvenance,
    pub resolution: ReferenceTargetResolution,
    pub expected_coordinate_contract: Option<SubstrateCoordinateContract>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReferenceTargetResolution {
    pub target: ReferenceTargetIdentity,
    pub status: ReferenceDiagnosticStatus,
    pub readable_path: String,
    pub target_node_index: Option<usize>,
    pub output_kind: Option<HoudiniDataKind>,
    pub coordinate_contract: Option<SubstrateCoordinateContract>,
    pub record_count: usize,
    pub source_provenance: Option<SourceProvenance>,
    pub diagnostic: Option<String>,
}

impl ReferenceTargetResolution {
    fn diagnostic(
        target: &ReferenceTargetIdentity,
        status: ReferenceDiagnosticStatus,
        message: &'static str,
    ) -> Self {
        Self {
            target: target.clone(),
            status,
            readable_path: format!(
                "{}/{}:{}",
                target.graph_id, target.node_id, target.output_name
            ),
            target_node_index: None,
            output_kind: None,
            coordinate_contract: None,
            record_count: 0,
            source_provenance: None,
            diagnostic: Some(message.to_owned()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReferenceDiagnosticStatus {
    Resolved,
    MissingNode,
    MissingOutput,
    DisallowedBoundary,
    AssetPrivateInternal,
    CoordinateContractMissing,
    CoordinateIncompatibleRepairable,
}

impl ReferenceDiagnosticStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "Resolved",
            Self::MissingNode => "Missing node",
            Self::MissingOutput => "Missing output",
            Self::DisallowedBoundary => "Disallowed boundary",
            Self::AssetPrivateInternal => "Asset private internal",
            Self::CoordinateContractMissing => "Coordinate contract missing",
            Self::CoordinateIncompatibleRepairable => "Coordinate repair available",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NullOperatorContract {
    pub node_name: String,
    pub convention: NullNameConvention,
    pub input_kind: HoudiniDataKind,
    pub output_kind: HoudiniDataKind,
    pub input_record_count: usize,
    pub output_record_count: usize,
    pub source_provenance: SourceProvenance,
    pub preserves_record_identity: bool,
    pub preserves_source_provenance: bool,
    pub preserves_evaluation_state: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NullNameConvention {
    InputAnchor,
    OutputAnchor,
    Ordinary,
}

impl NullNameConvention {
    fn from_name(name: &str) -> Self {
        let upper_name = name.to_ascii_uppercase();
        if upper_name.starts_with("IN_") {
            Self::InputAnchor
        } else if upper_name.starts_with("OUT_") {
            Self::OutputAnchor
        } else {
            Self::Ordinary
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::InputAnchor => "IN_* convention",
            Self::OutputAnchor => "OUT_* convention",
            Self::Ordinary => "ordinary null",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonOperatorNode {
    #[serde(default)]
    pub instance_id: String,
    pub declaration_id: String,
    pub provenance_summary: Option<String>,
    #[serde(default)]
    pub cache_key: Option<PythonOperatorCacheKey>,
    #[serde(default)]
    pub provenance: Option<PythonOperatorProvenanceRecord>,
    #[serde(default)]
    pub last_failure_summary: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct ProceduralAssetInstanceNode {
    #[serde(default)]
    pub instance_id: String,
    pub asset_id: String,
    #[serde(default)]
    pub instance_version: String,
    #[serde(default)]
    pub contents_unlocked: bool,
    #[serde(default)]
    pub input_bindings: Vec<HoudiniNodeBinding>,
    pub output_summary: Option<String>,
    #[serde(default)]
    pub version_status: OperatorVersionStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct NativeOperatorNode {
    #[serde(default)]
    pub instance_id: String,
    pub operator_id: String,
    #[serde(default)]
    pub version_status: OperatorVersionStatus,
    #[serde(default)]
    pub cache_key: Option<NativeOperatorCacheKey>,
    #[serde(default)]
    pub provenance: Option<NativeOperatorProvenanceRecord>,
    #[serde(default)]
    pub provenance_summary: Option<String>,
    pub last_valid_cache_key: Option<String>,
    pub last_failure_summary: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct HoudiniNodeBinding {
    pub port_name: String,
    pub source_summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum OperatorVersionStatus {
    #[default]
    Current,
    NewerAvailable,
    MissingDeclaration,
    Incompatible,
}

impl OperatorVersionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "Current",
            Self::NewerAvailable => "Newer available",
            Self::MissingDeclaration => "Declaration missing",
            Self::Incompatible => "Incompatible",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonOperatorCacheKey {
    pub key_digest: String,
    pub operator_id: String,
    pub node_instance_id: String,
    pub source_digest: String,
    pub declaration_version: String,
    pub parameter_digest: String,
    pub input_cache_keys: Vec<String>,
    pub dependency_lock_digest: Option<String>,
    pub capability_digest: String,
}

impl PythonOperatorCacheKey {
    fn summary(&self) -> String {
        format!(
            "{} {} with {} input key(s)",
            self.operator_id,
            self.key_digest,
            self.input_cache_keys.len()
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonOperatorProvenanceRecord {
    pub operator_id: String,
    pub version: String,
    pub node_instance_id: String,
    pub source_path: Option<String>,
    pub source_digest: String,
    pub parameter_digest: String,
    pub input_cache_keys: Vec<String>,
    pub dependency_identity: PythonDependencyIdentity,
    pub timestamp: u128,
    pub output_counts: PythonOperatorOutputCounts,
}

impl PythonOperatorProvenanceRecord {
    fn summary(&self) -> String {
        format!(
            "{} {} produced {} geometry record(s)",
            self.operator_id, self.version, self.output_counts.geometry_records
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct NativeOperatorCacheKey {
    pub key_digest: String,
    pub operator_id: String,
    pub node_instance_id: String,
    pub implementation_digest: String,
    pub declaration_version: String,
    pub parameter_digest: String,
    pub input_cache_keys: Vec<String>,
    pub host_compatibility_version: String,
    pub capability_digest: String,
}

impl NativeOperatorCacheKey {
    fn summary(&self) -> String {
        format!(
            "{} {} on host {} with {} input key(s)",
            self.operator_id,
            self.key_digest,
            self.host_compatibility_version,
            self.input_cache_keys.len()
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct NativeOperatorProvenanceRecord {
    pub operator_id: String,
    pub version: String,
    pub node_instance_id: String,
    pub implementation_digest: String,
    pub host_compatibility_version: String,
    pub parameter_digest: String,
    pub input_cache_keys: Vec<String>,
    pub timestamp: u128,
    pub output_counts: NativeOperatorOutputCounts,
}

impl NativeOperatorProvenanceRecord {
    fn summary(&self) -> String {
        format!(
            "{} {} produced {} geometry record(s) with {}",
            self.operator_id,
            self.version,
            self.output_counts.geometry_records,
            self.implementation_digest
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct NativeOperatorOutputCounts {
    pub geometry_records: usize,
    pub attribute_records: usize,
    pub layer_records: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonDependencyIdentity {
    pub environment_id: String,
    pub lock_digest: Option<String>,
    pub resolver_tool: String,
    pub resolver_version: Option<String>,
    pub resolver_executable_path: Option<String>,
    pub interpreter_path: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonOperatorOutputCounts {
    pub geometry_records: usize,
    pub attribute_records: usize,
    pub layer_records: usize,
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub(crate) struct PythonProcessRunReport {
    pub entry_point: String,
    pub interpreter_path: String,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub stdout: String,
    pub stderr: String,
    pub exit_status: Option<i32>,
    pub timed_out: bool,
    pub traceback_summary: Option<String>,
    pub output_record_count: usize,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct ProceduralAssetDeclaration {
    pub asset_id: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub labels: Vec<String>,
    pub help: String,
    pub source: ProceduralAssetSource,
    pub inputs: Vec<HoudiniOperatorPort>,
    pub outputs: Vec<HoudiniOperatorPort>,
    pub promoted_parameters: Vec<HoudiniParameterDeclaration>,
    pub wrapped_subgraph: ProceduralAssetSubgraphReference,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct ProceduralAssetSource {
    pub project_path: String,
    pub author: Option<String>,
    pub created_at: Option<String>,
    pub source_digest: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct ProceduralAssetSubgraphReference {
    pub graph_id: String,
    pub output_node_id: String,
    pub captures_native_cubic_bezier: bool,
    pub graph_snapshot: Option<ProceduralAssetGraphSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct ProceduralAssetGraphSnapshot {
    pub node_count: usize,
    pub edge_count: usize,
    pub layer_count: usize,
    pub geometry_contract: String,
}

pub(crate) struct CreateAssetDraft {
    pub asset_id: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub help: String,
    pub inputs: Vec<HoudiniOperatorPort>,
    pub outputs: Vec<HoudiniOperatorPort>,
    pub promoted_parameters: Vec<HoudiniParameterDeclaration>,
    pub graph_snapshot: ProceduralAssetGraphSnapshot,
}

impl CreateAssetDraft {
    fn into_declaration(self) -> ProceduralAssetDeclaration {
        ProceduralAssetDeclaration {
            asset_id: self.asset_id.clone(),
            display_name: self.display_name,
            version: self.version,
            description: self.description,
            labels: vec!["project-local".to_owned()],
            help: self.help,
            source: ProceduralAssetSource {
                project_path: format!("assets/{}.houdini_graph.json", self.asset_id),
                author: None,
                created_at: Some(current_timestamp_millis().to_string()),
                source_digest: Some(stable_digest(&serde_json::json!({
                    "asset_id": &self.asset_id,
                    "snapshot": &self.graph_snapshot,
                }))),
            },
            inputs: self.inputs,
            outputs: self.outputs,
            promoted_parameters: self.promoted_parameters,
            wrapped_subgraph: ProceduralAssetSubgraphReference {
                graph_id: format!("{}.graph", self.asset_id),
                output_node_id: "output.main".to_owned(),
                captures_native_cubic_bezier: true,
                graph_snapshot: Some(self.graph_snapshot),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct NativeOperatorDeclaration {
    pub operator_id: String,
    pub display_name: String,
    pub version: String,
    pub host_compatibility_version: String,
    pub implementation: NativeOperatorImplementation,
    pub inputs: Vec<HoudiniOperatorPort>,
    pub outputs: Vec<HoudiniOperatorPort>,
    pub parameters: Vec<HoudiniParameterDeclaration>,
    pub capabilities: Vec<NativeOperatorCapability>,
    pub provenance: NativeOperatorProvenance,
    pub failure_modes: Vec<NativeOperatorFailureMode>,
    pub documentation: String,
}

impl NativeOperatorDeclaration {
    fn implementation_digest(&self) -> String {
        self.provenance.build_digest.clone().unwrap_or_else(|| {
            stable_digest(&serde_json::json!({ "implementation": &self.implementation }))
        })
    }

    fn parameter_digest(&self, node_parameter_value: f32) -> String {
        stable_digest(&serde_json::json!({
            "node_parameter_value": node_parameter_value,
            "parameters": &self.parameters,
        }))
    }

    fn capability_digest(&self, granted_capabilities: &[NativeOperatorCapability]) -> String {
        stable_digest(&serde_json::json!({
            "declared_capabilities": &self.capabilities,
            "granted_capabilities": granted_capabilities,
        }))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum NativeOperatorImplementation {
    DynamicLibrary { path: String, symbol: String },
    Builtin { name: String },
    WasmComponent { path: String, export: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum NativeOperatorCapability {
    GeometryRead,
    GeometryWrite,
    FileRead,
    FileWrite,
    Network,
    Gpu,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct NativeOperatorProvenance {
    pub source_repository: Option<String>,
    pub source_revision: Option<String>,
    pub build_digest: Option<String>,
    pub vendor: Option<String>,
}

impl NativeOperatorProvenance {
    fn summary(&self) -> String {
        format!(
            "repo {}, rev {}, build {}",
            self.source_repository.as_deref().unwrap_or("unknown"),
            self.source_revision.as_deref().unwrap_or("unknown"),
            self.build_digest.as_deref().unwrap_or("unknown")
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct NativeOperatorFailureMode {
    pub code: String,
    pub summary: String,
    pub recoverable: bool,
}

impl NativeOperatorFailureMode {
    fn summary(&self) -> String {
        format!(
            "{}: {} ({})",
            self.code,
            self.summary,
            if self.recoverable {
                "recoverable"
            } else {
                "fatal"
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct HoudiniOperatorPort {
    pub name: String,
    pub data_kind: HoudiniDataKind,
    pub required: bool,
    pub help: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum HoudiniDataKind {
    GeometryTable,
    AttributeTable,
    Scalar,
    String,
    LayerStyle,
}

impl HoudiniDataKind {
    #[allow(dead_code)]
    pub fn preserves_native_cubic_bezier(self) -> bool {
        matches!(self, Self::GeometryTable)
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct HoudiniParameterDeclaration {
    pub name: String,
    pub kind: HoudiniParameterKind,
    pub default_value: HoudiniParameterValue,
    pub range: Option<HoudiniNumericRange>,
    pub allowed_values: Vec<String>,
    pub help: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum HoudiniParameterKind {
    Float,
    Bool,
    String,
    Enum,
    FilePath,
    AttributeSelector,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) enum HoudiniParameterValue {
    Float(f32),
    Bool(bool),
    String(String),
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct HoudiniNumericRange {
    pub min: f32,
    pub max: f32,
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

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonOperatorDeclaration {
    pub operator_id: String,
    pub display_name: String,
    pub version: String,
    pub entry_point: PythonOperatorEntryPoint,
    pub inputs: Vec<PythonOperatorPort>,
    pub outputs: Vec<PythonOperatorPort>,
    pub parameters: Vec<PythonOperatorParameterDeclaration>,
    pub dependencies: PythonOperatorDependencies,
    pub capabilities: Vec<PythonOperatorCapability>,
    pub help: String,
}

impl PythonOperatorDeclaration {
    #[allow(dead_code)]
    pub fn cache_key_material(&self) -> String {
        serde_json::to_string(&serde_json::json!({
            "operator_id": &self.operator_id,
            "version": &self.version,
            "entry_point": &self.entry_point,
            "inputs": &self.inputs,
            "outputs": &self.outputs,
            "parameters": &self.parameters,
            "dependencies": &self.dependencies,
            "capabilities": &self.capabilities,
        }))
        .unwrap_or_else(|err| format!("invalid-python-operator:{err}"))
    }

    #[allow(dead_code)]
    fn source_digest(&self) -> String {
        stable_digest(&serde_json::json!({
            "source": &self.entry_point.source,
            "callable": &self.entry_point.callable,
        }))
    }

    #[allow(dead_code)]
    fn parameter_digest(&self, node_parameter_value: f32) -> String {
        stable_digest(&serde_json::json!({
            "node_parameter_value": node_parameter_value,
            "parameters": &self.parameters
                .iter()
                .filter(|parameter| parameter.invalidates_cache)
                .collect::<Vec<_>>(),
        }))
    }

    #[allow(dead_code)]
    fn capability_digest(&self) -> String {
        stable_digest(&serde_json::json!({
            "capabilities": &self.capabilities,
        }))
    }

    #[allow(dead_code)]
    fn source_path(&self) -> Option<String> {
        match &self.entry_point.source {
            PythonOperatorSource::File { path } => Some(path.clone()),
            PythonOperatorSource::Module { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonOperatorEntryPoint {
    pub source: PythonOperatorSource,
    pub callable: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum PythonOperatorSource {
    File { path: String },
    Module { module: String },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonOperatorPort {
    pub name: String,
    pub data_kind: PythonOperatorDataKind,
    pub help: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum PythonOperatorDataKind {
    GeometryTable,
    AttributeTable,
    Scalar,
    String,
    LayerStyle,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonOperatorParameterDeclaration {
    pub name: String,
    pub kind: PythonOperatorParameterKind,
    pub default_value: PythonOperatorParameterValue,
    pub range: Option<PythonOperatorNumericRange>,
    pub allowed_values: Vec<String>,
    pub invalidates_cache: bool,
    pub help: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum PythonOperatorParameterKind {
    Float,
    Bool,
    String,
    Enum,
    FilePath,
    AttributeSelector,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) enum PythonOperatorParameterValue {
    Float(f32),
    Bool(bool),
    String(String),
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonOperatorNumericRange {
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonOperatorDependencies {
    pub python_version: Option<String>,
    pub requirements: Vec<String>,
    pub extras: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum PythonOperatorCapability {
    FileRead,
    FileWrite,
    Network,
    Subprocess,
    Gpu,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonEnvironmentDescriptor {
    pub environment_id: String,
    pub python_version_requirement: String,
    pub requirements_source: PythonRequirementsSource,
    pub project_requirements: PythonProjectRequirements,
    pub lock_status: PythonEnvironmentStatus,
    pub lock_digest: Option<String>,
    pub environment_path: Option<String>,
    pub resolver: PythonEnvironmentResolver,
    #[serde(default)]
    pub paths: PythonEnvironmentPaths,
    pub last_health_check: Option<String>,
    pub last_failure_summary: Option<String>,
    pub dependency_health: PythonDependencyHealth,
    pub resolve_state: PythonEnvironmentResolveState,
}

impl Default for PythonEnvironmentDescriptor {
    fn default() -> Self {
        Self {
            environment_id: "project-python".to_owned(),
            python_version_requirement: ">=3.11,<3.13".to_owned(),
            requirements_source: PythonRequirementsSource::ProjectLocal,
            project_requirements: PythonProjectRequirements::default(),
            lock_status: PythonEnvironmentStatus::Missing,
            lock_digest: None,
            environment_path: Some(".houdini/python/envs/project-python".to_owned()),
            resolver: PythonEnvironmentResolver {
                tool: "uv".to_owned(),
                version: None,
                executable_path: Some(".houdini/tools/uv".to_owned()),
            },
            paths: PythonEnvironmentPaths::default(),
            last_health_check: None,
            last_failure_summary: None,
            dependency_health: PythonDependencyHealth::default(),
            resolve_state: PythonEnvironmentResolveState::default(),
        }
    }
}

impl PythonEnvironmentDescriptor {
    pub fn status_summary(&self) -> String {
        match self.lock_status {
            PythonEnvironmentStatus::Missing => {
                "Project Python environment is not configured.".to_owned()
            }
            PythonEnvironmentStatus::Unlocked => {
                "Python requirements exist but no lock digest has been recorded.".to_owned()
            }
            PythonEnvironmentStatus::Resolving => {
                "uv is resolving the project Python environment.".to_owned()
            }
            PythonEnvironmentStatus::Locked => {
                "Python lock exists but the environment has not been verified.".to_owned()
            }
            PythonEnvironmentStatus::Ready => {
                "Project Python environment is ready for trusted operator execution.".to_owned()
            }
            PythonEnvironmentStatus::Stale => {
                "Python environment is stale because requirements or lock inputs changed."
                    .to_owned()
            }
            PythonEnvironmentStatus::Failed => self
                .last_failure_summary
                .clone()
                .unwrap_or_else(|| "Python environment validation failed.".to_owned()),
            PythonEnvironmentStatus::Disabled => "Project Python execution is disabled.".to_owned(),
        }
    }

    #[allow(dead_code)]
    fn dependency_identity(&self) -> PythonDependencyIdentity {
        PythonDependencyIdentity {
            environment_id: self.environment_id.clone(),
            lock_digest: self.lock_digest.clone(),
            resolver_tool: self.resolver.tool.clone(),
            resolver_version: self.resolver.version.clone(),
            resolver_executable_path: self.resolver.executable_path.clone(),
            interpreter_path: self.environment_path.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonProjectRequirements {
    pub requirements: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum PythonEnvironmentStatus {
    Missing,
    Unlocked,
    Resolving,
    Locked,
    Ready,
    Stale,
    Failed,
    Disabled,
}

impl PythonEnvironmentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "Missing",
            Self::Unlocked => "Unlocked",
            Self::Resolving => "Resolving",
            Self::Locked => "Locked",
            Self::Ready => "Ready",
            Self::Stale => "Stale",
            Self::Failed => "Failed",
            Self::Disabled => "Disabled",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum PythonRequirementsSource {
    ProjectLocal,
    GeneratedFromOperators,
    PyprojectFragment { path: String },
}

impl PythonRequirementsSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProjectLocal => "Project-local",
            Self::GeneratedFromOperators => "Generated from operators",
            Self::PyprojectFragment { .. } => "pyproject fragment",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonEnvironmentResolver {
    pub tool: String,
    pub version: Option<String>,
    #[serde(default)]
    pub executable_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonEnvironmentPaths {
    pub mode: PythonEnvironmentPathMode,
    pub existing_environment_path: Option<String>,
    pub create_environment_path: String,
}

impl Default for PythonEnvironmentPaths {
    fn default() -> Self {
        Self {
            mode: PythonEnvironmentPathMode::CreateProjectLocal,
            existing_environment_path: None,
            create_environment_path: ".houdini/python/envs/project-python".to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum PythonEnvironmentPathMode {
    ExistingEnvironment,
    #[default]
    CreateProjectLocal,
}

impl PythonEnvironmentPathMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExistingEnvironment => "existing environment",
            Self::CreateProjectLocal => "create project-local environment",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonEnvironmentResolveState {
    pub last_plan: Option<PythonEnvironmentResolvePlan>,
    pub in_progress: Option<PythonEnvironmentResolveRun>,
    pub previous_ready: Option<PythonEnvironmentReadySnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonEnvironmentResolvePlan {
    pub trigger: PythonEnvironmentResolveTrigger,
    pub requirements: Vec<PythonRequirementContribution>,
    pub conflicts: Vec<PythonDependencyConflict>,
}

impl PythonEnvironmentResolvePlan {
    pub fn unique_requirement_count(&self) -> usize {
        self.requirements.len()
    }

    pub fn conflict_summary(&self) -> String {
        if self.conflicts.is_empty() {
            "No dependency conflicts detected in declared requirements.".to_owned()
        } else {
            format!("{} dependency conflict(s) detected.", self.conflicts.len())
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum PythonEnvironmentResolveTrigger {
    ExplicitUserAction,
    TrustedProjectOpen,
}

impl PythonEnvironmentResolveTrigger {
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitUserAction => "Explicit user action",
            Self::TrustedProjectOpen => "Trusted project open",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonEnvironmentResolveRun {
    pub trigger: PythonEnvironmentResolveTrigger,
    pub resolver_tool: String,
    pub resolver_executable_path: Option<String>,
    pub started_at: u128,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonEnvironmentReadySnapshot {
    pub lock_digest: Option<String>,
    pub resolver_version: Option<String>,
    pub resolver_executable_path: Option<String>,
    pub environment_path: Option<String>,
    pub paths: PythonEnvironmentPaths,
    pub dependency_health: PythonDependencyHealth,
    pub last_health_check: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonRequirementContribution {
    pub requirement: String,
    pub source: PythonRequirementSource,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum PythonRequirementSource {
    Project,
    Operator { operator_id: String },
}

impl PythonRequirementSource {
    #[allow(dead_code)]
    pub fn as_str(&self) -> String {
        match self {
            Self::Project => "project".to_owned(),
            Self::Operator { operator_id } => format!("operator {operator_id}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonDependencyConflict {
    pub package: String,
    pub requirements: Vec<PythonRequirementContribution>,
}

impl PythonDependencyConflict {
    pub fn summary(&self) -> String {
        format!(
            "{} has {} incompatible requirement declaration(s)",
            self.package,
            self.requirements.len()
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct PythonDependencyHealth {
    pub package_count: usize,
    pub missing_packages: Vec<String>,
    pub conflicts: Vec<String>,
    pub failed_imports: Vec<String>,
}

impl PythonDependencyHealth {
    pub fn is_healthy(&self) -> bool {
        self.missing_packages.is_empty()
            && self.conflicts.is_empty()
            && self.failed_imports.is_empty()
    }
}

#[allow(dead_code)]
fn stable_digest(value: &serde_json::Value) -> String {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[allow(dead_code)]
fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

#[allow(dead_code)]
fn push_unique_requirement(
    requirements: &mut Vec<PythonRequirementContribution>,
    contribution: PythonRequirementContribution,
) {
    if !requirements.iter().any(|existing| {
        existing.requirement == contribution.requirement && existing.source == contribution.source
    }) {
        requirements.push(contribution);
    }
}

#[allow(dead_code)]
fn dependency_conflicts(
    requirements: &[PythonRequirementContribution],
) -> Vec<PythonDependencyConflict> {
    let mut conflicts = Vec::new();
    for contribution in requirements {
        let package = requirement_package_name(&contribution.requirement);
        if conflicts
            .iter()
            .any(|conflict: &PythonDependencyConflict| conflict.package == package)
        {
            continue;
        }
        let package_requirements = requirements
            .iter()
            .filter(|other| requirement_package_name(&other.requirement) == package)
            .cloned()
            .collect::<Vec<_>>();
        let distinct_specs = package_requirements
            .iter()
            .map(|other| other.requirement.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        if distinct_specs.len() > 1 {
            conflicts.push(PythonDependencyConflict {
                package,
                requirements: package_requirements,
            });
        }
    }
    conflicts
}

#[allow(dead_code)]
fn requirement_package_name(requirement: &str) -> String {
    requirement
        .split(['=', '<', '>', '!', '~', '[', ';', ' '])
        .next()
        .unwrap_or(requirement)
        .trim()
        .to_ascii_lowercase()
}

#[cfg(not(target_arch = "wasm32"))]
struct PythonProcessOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_status: Option<i32>,
    timed_out: bool,
}

#[cfg(not(target_arch = "wasm32"))]
fn run_python_process(
    interpreter_path: PathBuf,
    entry_point_path: PathBuf,
    input_path: &Path,
    output_path: &Path,
    timeout: Duration,
) -> anyhow::Result<PythonProcessOutput> {
    let mut child = std::process::Command::new(interpreter_path)
        .arg(entry_point_path)
        .arg("--houdini-input")
        .arg(input_path)
        .arg("--houdini-output")
        .arg(output_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let deadline = Instant::now() + timeout;
    loop {
        if child.try_wait()?.is_some() {
            let output = child.wait_with_output()?;
            return Ok(PythonProcessOutput {
                stdout: output.stdout,
                stderr: output.stderr,
                exit_status: output.status.code(),
                timed_out: false,
            });
        }

        if Instant::now() >= deadline {
            child.kill()?;
            let output = child.wait_with_output()?;
            return Ok(PythonProcessOutput {
                stdout: output.stdout,
                stderr: output.stderr,
                exit_status: output.status.code(),
                timed_out: true,
            });
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn traceback_summary(stderr: &str) -> Option<String> {
    stderr
        .lines()
        .rev()
        .find(|line| {
            line.contains("Traceback")
                || line.contains("Error:")
                || line.contains("Exception:")
                || line.starts_with("  File ")
        })
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
}

fn sanitize_asset_id_part(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_owned();
    if sanitized.is_empty() {
        "asset".to_owned()
    } else {
        sanitized
    }
}

fn port_names(ports: &[HoudiniOperatorPort]) -> Vec<String> {
    ports
        .iter()
        .map(|port| format!("{} ({:?})", port.name, port.data_kind))
        .collect()
}

fn native_operator_node_status(
    version_status: OperatorVersionStatus,
    load_status: NativeOperatorLoadStatus,
) -> NodeStatus {
    match load_status {
        NativeOperatorLoadStatus::Ready => match version_status {
            OperatorVersionStatus::Current => NodeStatus::Healthy,
            OperatorVersionStatus::NewerAvailable => NodeStatus::Warning,
            OperatorVersionStatus::MissingDeclaration | OperatorVersionStatus::Incompatible => {
                NodeStatus::Failed
            }
        },
        NativeOperatorLoadStatus::TrustRequired
        | NativeOperatorLoadStatus::MissingCapabilityGrant => NodeStatus::Warning,
        NativeOperatorLoadStatus::DeclarationMissing
        | NativeOperatorLoadStatus::HostIncompatible
        | NativeOperatorLoadStatus::ImplementationDigestMissing
        | NativeOperatorLoadStatus::LoadFailed
        | NativeOperatorLoadStatus::RuntimeFailed
        | NativeOperatorLoadStatus::TimedOut
        | NativeOperatorLoadStatus::OutputSchemaMismatch => NodeStatus::Failed,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GeneratedNodeInfo {
    pub source: GeneratedNodeSource,
    #[serde(default)]
    pub binding_state: GeneratedNodeBindingState,
}

impl GeneratedNodeInfo {
    fn managed(source: GeneratedNodeSource) -> Self {
        Self {
            source,
            binding_state: GeneratedNodeBindingState::Managed,
        }
    }

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum GeneratedNodeBindingState {
    #[default]
    Managed,
    Adopted,
    Unbound,
}

impl GeneratedNodeBindingState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Managed => "Managed layer binding",
            Self::Adopted => "Adopted graph node",
            Self::Unbound => "Unbound generated node",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Self::Managed => "mgd",
            Self::Adopted => "adp",
            Self::Unbound => "gen",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Managed => {
                "Layer-facing controls may still update compatible parameters on this graph node."
            }
            Self::Adopted => {
                "The node began as generated graph material, but structural graph edits made it user-owned."
            }
            Self::Unbound => {
                "The node is generated graph material without an active layer-facing control."
            }
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
    pub name: String,
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
    Null,
    ReferenceInput,
    SubstrateProjection,
    PythonOperator,
    ProceduralAsset,
    NativeOperator,
    Output,
}

impl NodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Source => "Source",
            Self::Filter => "Filter",
            Self::Style => "Style",
            Self::Null => "Null",
            Self::ReferenceInput => "Reference Input",
            Self::SubstrateProjection => "Substrate Projection",
            Self::PythonOperator => "Python Operator",
            Self::ProceduralAsset => "Asset",
            Self::NativeOperator => "Native Operator",
            Self::Output => "Output",
        }
    }

    pub fn role(self) -> &'static str {
        match self {
            Self::Source => "Read",
            Self::Filter => "Cull",
            Self::Style => "Style",
            Self::Null => "Anchor",
            Self::ReferenceInput => "Reference",
            Self::SubstrateProjection => "Project",
            Self::PythonOperator => "Compute",
            Self::ProceduralAsset => "Asset",
            Self::NativeOperator => "Native",
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
    pub reference_consumers: Vec<ReferenceConsumerInfo>,
    pub reference_output_warning: Option<ReferenceOutputChangeWarning>,
    pub output_operator: Option<OutputOperatorNodeInfo>,
    pub null_operator: Option<NullOperatorNodeInfo>,
    pub reference_input: Option<ReferenceInputNodeInfo>,
    pub python_operator: Option<PythonOperatorNodeInfo>,
    pub procedural_asset: Option<ProceduralAssetNodeInfo>,
    pub native_operator: Option<NativeOperatorNodeInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReferenceConsumerInfo {
    pub reference_node_index: usize,
    pub reference_node_id: String,
    pub reference_node_name: String,
    pub target_output_name: String,
    pub readable_source_path: String,
    pub enabled: bool,
    pub status: ReferenceDiagnosticStatus,
    pub diagnostic: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReferenceOutputChangeWarning {
    pub target_node_index: usize,
    pub target_node_id: String,
    pub target_node_name: String,
    pub output_name: String,
    pub affected_references: Vec<ReferenceConsumerInfo>,
}

pub(crate) struct OutputOperatorNodeInfo {
    pub kind: OutputOperatorKind,
    pub semantic_payload: OutputSemanticPayload,
    pub command: OutputCommand,
    pub preferred_target: Option<OutputTargetId>,
    pub negotiations: Vec<OutputTargetNegotiation>,
    pub rerun_options: Option<RerunOutputTargetOptions>,
    pub graph_viewport_state_separate: bool,
}

pub(crate) struct NullOperatorNodeInfo {
    pub convention: NullNameConvention,
    pub input_kind: HoudiniDataKind,
    pub output_kind: HoudiniDataKind,
    pub preserves_record_identity: bool,
    pub preserves_source_provenance: bool,
}

pub(crate) struct ReferenceInputNodeInfo {
    pub target: ReferenceTargetIdentity,
    pub readable_path: String,
    pub status: ReferenceDiagnosticStatus,
    pub output_kind: Option<HoudiniDataKind>,
    pub coordinate_contract: Option<SubstrateCoordinateContract>,
    pub source_provenance: Option<SourceProvenance>,
    pub targets: Vec<ReferenceTargetNodeInfo>,
    pub preserves_source_data: bool,
    pub applies_hidden_transform: bool,
}

pub(crate) struct ReferenceTargetNodeInfo {
    pub target: ReferenceTargetIdentity,
    pub readable_path: String,
    pub status: ReferenceDiagnosticStatus,
    pub enabled: bool,
    pub target_node_index: Option<usize>,
    pub output_kind: Option<HoudiniDataKind>,
    pub coordinate_contract: Option<SubstrateCoordinateContract>,
    pub expected_coordinate_contract: Option<SubstrateCoordinateContract>,
    pub record_count: usize,
    pub source_provenance: Option<SourceProvenance>,
    pub diagnostic: Option<String>,
    pub provenance: ReferenceTargetProvenance,
}

pub(crate) struct PythonOperatorNodeInfo {
    pub declaration_id: String,
    pub display_name: String,
    pub version: String,
    pub dependency_status: PythonOperatorDependencyStatus,
    pub dependency_summary: String,
    pub requirements: Vec<String>,
    pub provenance_summary: Option<String>,
    pub cache_key_summary: Option<String>,
    pub last_failure_summary: Option<String>,
}

pub(crate) struct ProceduralAssetNodeInfo {
    pub asset_id: String,
    pub display_name: String,
    pub instance_version: String,
    pub current_version: Option<String>,
    pub contents_unlocked: bool,
    pub local_graph_id: Option<String>,
    pub description: String,
    pub labels: Vec<String>,
    pub promoted_parameters: Vec<String>,
    pub input_bindings: Vec<HoudiniNodeBinding>,
    pub output_summary: Option<String>,
    pub version_status: OperatorVersionStatus,
}

pub(crate) struct NativeOperatorNodeInfo {
    pub operator_id: String,
    pub display_name: String,
    pub version: String,
    pub host_compatibility_version: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub parameters: Vec<String>,
    pub capabilities: Vec<String>,
    pub provenance_summary: String,
    pub output_provenance_summary: Option<String>,
    pub cache_key_summary: Option<String>,
    pub failure_modes: Vec<String>,
    pub version_status: OperatorVersionStatus,
    pub load_status: NativeOperatorLoadStatus,
    pub last_valid_cache_key: Option<String>,
    pub last_failure_summary: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeOperatorLoadStatus {
    DeclarationMissing,
    TrustRequired,
    HostIncompatible,
    ImplementationDigestMissing,
    MissingCapabilityGrant,
    Ready,
    #[allow(dead_code)]
    LoadFailed,
    #[allow(dead_code)]
    RuntimeFailed,
    #[allow(dead_code)]
    TimedOut,
    #[allow(dead_code)]
    OutputSchemaMismatch,
}

impl NativeOperatorLoadStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeclarationMissing => "Declaration missing",
            Self::TrustRequired => "Trust required",
            Self::HostIncompatible => "Host incompatible",
            Self::ImplementationDigestMissing => "Implementation digest missing",
            Self::MissingCapabilityGrant => "Missing capability grant",
            Self::Ready => "Ready",
            Self::LoadFailed => "Load failed",
            Self::RuntimeFailed => "Runtime failed",
            Self::TimedOut => "Timed out",
            Self::OutputSchemaMismatch => "Output schema mismatch",
        }
    }

    fn summary(self) -> &'static str {
        match self {
            Self::DeclarationMissing => "Native operator declaration is missing.",
            Self::TrustRequired => {
                "Project trust or explicit operator enablement is required before loading native code."
            }
            Self::HostIncompatible => {
                "Native operator host compatibility version does not match this viewer."
            }
            Self::ImplementationDigestMissing => {
                "Native operator implementation digest is missing."
            }
            Self::MissingCapabilityGrant => {
                "Native operator declares capabilities that have not been granted."
            }
            Self::Ready => "Native operator is trusted and ready to load.",
            Self::LoadFailed => "Native operator load failed.",
            Self::RuntimeFailed => "Native operator runtime failed.",
            Self::TimedOut => "Native operator execution timed out.",
            Self::OutputSchemaMismatch => {
                "Native operator output schema did not match Houdini geometry."
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct NativeOperatorTrustPolicy {
    pub project_trusted: bool,
    pub enabled_operator_ids: Vec<String>,
    pub granted_capabilities: Vec<NativeOperatorCapability>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PythonOperatorDependencyStatus {
    DeclarationMissing,
    MissingEnvironment,
    ResolvingEnvironment,
    Ready,
    StaleEnvironment,
    FailedEnvironment,
    DisabledEnvironment,
}

impl PythonOperatorDependencyStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeclarationMissing => "Declaration missing",
            Self::MissingEnvironment => "Environment missing",
            Self::ResolvingEnvironment => "Environment resolving",
            Self::Ready => "Ready",
            Self::StaleEnvironment => "Environment stale",
            Self::FailedEnvironment => "Environment failed",
            Self::DisabledEnvironment => "Environment disabled",
        }
    }

    fn summary(self) -> &'static str {
        match self {
            Self::DeclarationMissing => "Python operator declaration is missing.",
            Self::MissingEnvironment => "Project Python environment is not configured.",
            Self::ResolvingEnvironment => {
                "Project Python environment is resolving or locked but unverified."
            }
            Self::Ready => "Python operator dependencies are ready.",
            Self::StaleEnvironment => {
                "Project Python environment must be resolved before execution."
            }
            Self::FailedEnvironment => {
                "Project Python environment has dependency or validation failures."
            }
            Self::DisabledEnvironment => "Project Python execution is disabled.",
        }
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PipelineStage {
    pub name: String,
    pub input_count: usize,
    pub output_count: usize,
    pub note: String,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
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
        GeneratedNodeBindingState, GeneratedNodeSource, Geometry, GeometryKind,
        GraphAnnotationKind, GraphColor, GraphDocument, GraphNode, GraphPoint, GraphStyle,
        HoudiniCubicBezierParquetSchema, HoudiniDataKind, HoudiniGeometryRecord,
        HoudiniGeometrySchema, HoudiniNumericRange, HoudiniOperatorPort,
        HoudiniParameterDeclaration, HoudiniParameterKind, HoudiniParameterValue, LayerKind,
        NativeOperatorCapability, NativeOperatorDeclaration, NativeOperatorFailureMode,
        NativeOperatorImplementation, NativeOperatorLoadStatus, NativeOperatorOutputCounts,
        NativeOperatorProvenance, NetworkBadgeVisibility, NetworkNodeRingVisibility,
        NodeEvaluation, NodeKind, NodeParameter, NodeParameterKind, NodeStatus,
        OperatorVersionStatus, OutputCapabilityMapping, OutputOperatorKind, OutputOperatorNode,
        OutputTargetId, PRIMARY_GEOMETRY_OUTPUT, ProceduralAssetDeclaration,
        ProceduralAssetGraphSnapshot, ProceduralAssetSource, ProceduralAssetSubgraphReference,
        PythonDependencyHealth, PythonEnvironmentDescriptor, PythonEnvironmentPathMode,
        PythonEnvironmentPaths, PythonEnvironmentResolveState, PythonEnvironmentResolveTrigger,
        PythonEnvironmentResolver, PythonEnvironmentStatus, PythonOperatorCapability,
        PythonOperatorDataKind, PythonOperatorDeclaration, PythonOperatorDependencies,
        PythonOperatorDependencyStatus, PythonOperatorEntryPoint, PythonOperatorNumericRange,
        PythonOperatorOutputCounts, PythonOperatorParameterDeclaration,
        PythonOperatorParameterKind, PythonOperatorParameterValue, PythonOperatorPort,
        PythonOperatorSource, PythonProjectRequirements, PythonRequirementSource,
        PythonRequirementsSource, ReferenceDiagnosticStatus, ReferenceTargetEntry,
        ReferenceTargetIdentity, ReferenceTargetProvenance, RerunSceneDebugItem, RerunSceneItem,
        SourceProvenance, SubstrateCoordinateContract, SubstrateOrigin, SubstrateYAxis,
        ViewerGeometry, load_cubic_bezier_parquet, load_cubic_bezier_parquet_with_metadata,
    };
    use std::sync::Arc;

    use arrow::array::Float64Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use parquet::arrow::ArrowWriter;

    fn alternate_substrate_contract() -> SubstrateCoordinateContract {
        SubstrateCoordinateContract {
            substrate_id: "alternate-markov-pixel-space".to_owned(),
            width: 1024,
            height: 1024,
            origin: SubstrateOrigin::BottomLeft,
            y_axis: SubstrateYAxis::Up,
        }
    }

    fn add_reference_node_for_target(
        graph: &mut GraphDocument,
        target: ReferenceTargetIdentity,
    ) -> usize {
        let provenance = ReferenceTargetProvenance {
            source_graph_id: target.graph_id.clone(),
            source_node_id: target.node_id.clone(),
            source_node_name: target.node_id.clone(),
            source_output_name: target.output_name.clone(),
            source_data_kind: HoudiniDataKind::GeometryTable,
        };
        let mut node = GraphNode::reference_input(
            graph.unique_node_id("reference_input"),
            ReferenceTargetEntry {
                target,
                enabled: true,
                provenance,
            },
        );
        node.layout_position = GraphPoint::new(0.88, 0.62);
        let insert_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .unwrap_or(graph.nodes.len());
        graph.nodes.insert(insert_index, node);
        insert_index
    }

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
    fn generic_output_operator_expresses_viewer_agnostic_contract() {
        let mut graph = GraphDocument::sample();
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .expect("output node should exist");
        graph.nodes[output_index].output_operator = Some(OutputOperatorNode::generic_scene());

        let contract = graph
            .output_target_contract_for_node(output_index)
            .expect("generic output should expose a contract");
        let generic_negotiation = graph
            .negotiate_output_target_for_node(output_index, OutputTargetId::GenericGraph)
            .expect("generic target should negotiate");
        let rerun_negotiation = graph
            .negotiate_output_target_for_node(output_index, OutputTargetId::Rerun)
            .expect("rerun target should negotiate through adapter");

        assert_eq!(contract.preferred_target, None);
        assert_eq!(
            generic_negotiation.mapping,
            OutputCapabilityMapping::NativeMapping
        );
        assert_eq!(
            rerun_negotiation.mapping,
            OutputCapabilityMapping::PreparedRepresentation
        );
    }

    #[test]
    fn rerun_output_operator_negotiates_lower_fidelity_for_native_cubics() {
        let graph = GraphDocument::sample();
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .expect("output node should exist");

        let info = graph
            .selected_node_info(output_index)
            .expect("output node should expose info")
            .output_operator
            .expect("output operator info should exist");
        let rerun_negotiation = info
            .negotiations
            .iter()
            .find(|negotiation| negotiation.target == OutputTargetId::Rerun)
            .expect("rerun negotiation should be present");

        assert_eq!(info.kind, OutputOperatorKind::RerunSpecialized);
        assert_eq!(info.preferred_target, Some(OutputTargetId::Rerun));
        assert_eq!(
            rerun_negotiation.mapping,
            OutputCapabilityMapping::LowerFidelityWithWarning
        );
        assert!(
            rerun_negotiation
                .reason
                .contains("preserves cubic control points")
        );
        assert!(
            info.rerun_options
                .expect("rerun output should expose adapter options")
                .preserve_native_cubic_metadata
        );
        assert!(info.graph_viewport_state_separate);
    }

    #[test]
    fn output_target_negotiation_reports_native_prepared_and_unsupported_paths() {
        let mut graph = GraphDocument::sample();
        graph
            .geometry
            .retain(|geometry| matches!(geometry, Geometry::Polygon(_)));
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .expect("output node should exist");

        assert_eq!(
            graph
                .negotiate_output_target_for_node(output_index, OutputTargetId::Rerun)
                .expect("rerun target should negotiate")
                .mapping,
            OutputCapabilityMapping::NativeMapping
        );
        assert_eq!(
            graph
                .negotiate_output_target_for_node(
                    output_index,
                    OutputTargetId::DebugPreparedPolyline
                )
                .expect("debug target should negotiate")
                .mapping,
            OutputCapabilityMapping::PreparedRepresentation
        );
        assert_eq!(
            graph
                .negotiate_output_target_for_node(output_index, OutputTargetId::UnsupportedExternal)
                .expect("unsupported target should negotiate")
                .mapping,
            OutputCapabilityMapping::Unsupported
        );
    }

    #[test]
    fn output_operator_round_trips_without_target_owned_runtime_state() {
        let graph = GraphDocument::sample();
        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();
        let output_index = restored
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .expect("output node should exist");

        assert!(json.contains("output_operator"));
        assert!(json.contains("RerunSpecialized"));
        assert!(!json.contains("entity_path"));
        assert!(!json.contains("timeline"));
        assert!(!json.contains("session"));
        assert_eq!(
            restored
                .output_target_contract_for_node(output_index)
                .expect("output contract should round-trip")
                .preferred_target,
            Some(OutputTargetId::Rerun)
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
    fn null_operator_inserts_as_visible_typed_pass_through_anchor() {
        let mut graph = GraphDocument::sample();
        let before_rows = graph.attribute_table_rows(&AttributeTableQuery::default());
        let before_scene = graph.rerun_scene_output();

        let null_index = graph.add_null_operator_node("OUT_FILTERED");

        assert_eq!(graph.nodes[null_index].kind, NodeKind::Null);
        assert_eq!(graph.nodes[null_index].name, "OUT_FILTERED");
        assert_eq!(graph.nodes[null_index + 1].kind, NodeKind::Output);
        assert_eq!(graph.graph_layout().edges.len(), graph.nodes.len() - 1);

        let contract = graph
            .null_operator_contract(null_index)
            .expect("null node should expose a pass-through contract");
        assert_eq!(contract.convention, super::NullNameConvention::OutputAnchor);
        assert_eq!(contract.input_kind, HoudiniDataKind::GeometryTable);
        assert_eq!(contract.output_kind, HoudiniDataKind::GeometryTable);
        assert_eq!(contract.input_record_count, before_rows.len());
        assert_eq!(contract.output_record_count, before_rows.len());
        assert_eq!(contract.source_provenance, SourceProvenance::DemoFallback);
        assert!(contract.preserves_record_identity);
        assert!(contract.preserves_source_provenance);
        assert!(contract.preserves_evaluation_state);

        let after_rows = graph.attribute_table_rows(&AttributeTableQuery::default());
        let after_scene = graph.rerun_scene_output();
        assert_eq!(
            after_rows
                .iter()
                .map(|row| (row.record_index, row.geometry_kind, row.provenance))
                .collect::<Vec<_>>(),
            before_rows
                .iter()
                .map(|row| (row.record_index, row.geometry_kind, row.provenance))
                .collect::<Vec<_>>()
        );
        assert_eq!(after_scene.items.len(), before_scene.items.len());
        assert_eq!(after_scene.polygon_count(), before_scene.polygon_count());
        assert_eq!(
            after_scene.native_cubic_bezier_count(),
            before_scene.native_cubic_bezier_count()
        );
    }

    #[test]
    fn null_operator_node_info_exposes_convention_without_type_magic() {
        let mut graph = GraphDocument::sample();
        let null_index = graph.add_null_operator_node("IN_GEO");

        let info = graph
            .selected_node_info(null_index)
            .expect("null node should report node info");
        let null_info = info
            .null_operator
            .expect("null node info should expose pass-through metadata");

        assert_eq!(info.kind, NodeKind::Null);
        assert_eq!(info.role, "Anchor");
        assert_eq!(info.input_count, info.output_count);
        assert_eq!(info.record_count, graph.visible_output_count());
        assert_eq!(info.data_kind, "Geometry table pass-through");
        assert_eq!(null_info.convention, super::NullNameConvention::InputAnchor);
        assert_eq!(null_info.input_kind, HoudiniDataKind::GeometryTable);
        assert_eq!(null_info.output_kind, HoudiniDataKind::GeometryTable);
        assert!(null_info.preserves_record_identity);
        assert!(null_info.preserves_source_provenance);
    }

    #[test]
    fn null_operator_round_trips_through_sidecar_with_anchor_name() {
        let mut graph = GraphDocument::sample();
        graph.add_null_operator_node("OUT_CURVES");

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        let null_index = restored
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Null && node.name == "OUT_CURVES")
            .expect("restored graph should include named null anchor");
        let contract = restored
            .null_operator_contract(null_index)
            .expect("restored null should expose a contract");

        assert_eq!(contract.convention, super::NullNameConvention::OutputAnchor);
        assert_eq!(contract.input_kind, contract.output_kind);
        assert_eq!(restored.nodes[null_index + 1].kind, NodeKind::Output);
    }

    #[test]
    fn reference_input_targets_null_by_stable_identity_and_readable_path() {
        let mut graph = GraphDocument::sample();
        let null_index = graph.add_null_operator_node("OUT_CURVES");
        let target_node_id = graph.nodes[null_index].node_id.clone();

        let reference_index = graph
            .add_reference_input_node(null_index)
            .expect("null output should be a compatible reference target");
        let resolution = graph
            .reference_input_resolution(reference_index)
            .expect("reference node should resolve its target");

        assert_eq!(graph.nodes[reference_index].kind, NodeKind::ReferenceInput);
        assert_eq!(resolution.status, ReferenceDiagnosticStatus::Resolved);
        assert_eq!(resolution.target.node_id, target_node_id);
        assert_eq!(
            resolution.target.output_name,
            super::PRIMARY_GEOMETRY_OUTPUT
        );
        assert_eq!(resolution.readable_path, "main/OUT_CURVES:geometry");
        assert_eq!(resolution.output_kind, Some(HoudiniDataKind::GeometryTable));
        assert_eq!(resolution.record_count, graph.visible_output_count());
        assert_eq!(
            resolution.source_provenance,
            Some(SourceProvenance::DemoFallback)
        );

        let info = graph
            .selected_node_info(reference_index)
            .expect("reference node should have node info");
        let reference_info = info
            .reference_input
            .expect("reference node info should expose the target");
        assert_eq!(info.kind, NodeKind::ReferenceInput);
        assert_eq!(info.status, NodeStatus::Healthy);
        assert_eq!(reference_info.status, ReferenceDiagnosticStatus::Resolved);
        assert!(reference_info.preserves_source_data);
        assert!(!reference_info.applies_hidden_transform);
    }

    #[test]
    fn reference_input_survives_target_rename_and_move_by_identity() {
        let mut graph = GraphDocument::sample();
        let null_index = graph.add_null_operator_node("OUT_ORIGINAL");
        let reference_index = graph
            .add_reference_input_node(null_index)
            .expect("null output should be a compatible reference target");
        let target = graph.nodes[reference_index]
            .reference_input
            .as_ref()
            .expect("reference input should exist")
            .targets
            .first()
            .expect("reference input should have a target")
            .target
            .clone();

        graph.nodes[null_index].name = "OUT_RENAMED".to_owned();
        graph.set_node_layout_position(null_index, GraphPoint::new(0.25, 0.25));

        let resolution = graph.resolve_reference_target(&target);

        assert_eq!(resolution.status, ReferenceDiagnosticStatus::Resolved);
        assert_eq!(resolution.target.node_id, target.node_id);
        assert_eq!(resolution.readable_path, "main/OUT_RENAMED:geometry");
    }

    #[test]
    fn node_rename_stays_unique_and_keeps_reference_identity() {
        let mut graph = GraphDocument::sample();
        let null_index = graph.add_null_operator_node("OUT_ORIGINAL");
        let reference_index = graph
            .add_reference_input_node(null_index)
            .expect("null output should be a compatible reference target");
        let target = graph.nodes[reference_index]
            .reference_input
            .as_ref()
            .expect("reference input should exist")
            .targets
            .first()
            .expect("reference input should have a target")
            .target
            .clone();

        assert!(!graph.set_node_name(null_index, ""));
        assert!(graph.set_node_name(null_index, "Source"));

        assert_eq!(graph.nodes[null_index].name, "Source_2");
        let resolution = graph.resolve_reference_target(&target);
        assert_eq!(resolution.status, ReferenceDiagnosticStatus::Resolved);
        assert_eq!(resolution.target.node_id, target.node_id);
        assert_eq!(resolution.readable_path, "main/Source_2:geometry");
    }

    #[test]
    fn reference_input_reports_missing_target_without_rebinding_by_name() {
        let mut graph = GraphDocument::sample();
        let null_index = graph.add_null_operator_node("OUT_GEO");
        let reference_index = graph
            .add_reference_input_node(null_index)
            .expect("null output should be a compatible reference target");
        let target = graph.nodes[reference_index]
            .reference_input
            .as_ref()
            .expect("reference input should exist")
            .targets
            .first()
            .expect("reference input should have a target")
            .target
            .clone();

        graph
            .remove_node(null_index)
            .expect("test null should be removable");
        graph.add_null_operator_node("OUT_GEO");

        let resolution = graph.resolve_reference_target(&target);

        assert_eq!(resolution.status, ReferenceDiagnosticStatus::MissingNode);
        assert_eq!(resolution.record_count, 0);
        assert!(resolution.diagnostic.is_some());
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| node.kind == NodeKind::Null && node.name == "OUT_GEO")
        );
    }

    #[test]
    fn reference_input_reports_missing_output_and_disallowed_boundary() {
        let mut graph = GraphDocument::sample();
        let null_index = graph.add_null_operator_node("OUT_GEO");
        let reference_index = graph
            .add_reference_input_node(null_index)
            .expect("null output should be a compatible reference target");
        let mut target = graph.nodes[reference_index]
            .reference_input
            .as_ref()
            .expect("reference input should exist")
            .targets
            .first()
            .expect("reference input should have a target")
            .target
            .clone();

        target.output_name = "missing".to_owned();
        assert_eq!(
            graph.resolve_reference_target(&target).status,
            ReferenceDiagnosticStatus::MissingOutput
        );

        target.output_name = super::PRIMARY_GEOMETRY_OUTPUT.to_owned();
        target.graph_id = "other_project".to_owned();
        assert_eq!(
            graph.resolve_reference_target(&target).status,
            ReferenceDiagnosticStatus::DisallowedBoundary
        );
    }

    #[test]
    fn reference_input_becomes_stale_when_upstream_target_changes() {
        let mut graph = GraphDocument::sample();
        let null_index = graph.add_null_operator_node("OUT_GEO");
        let reference_index = graph
            .add_reference_input_node(null_index)
            .expect("null output should be a compatible reference target");

        graph.mark_node_stale(null_index);

        assert_eq!(
            graph.nodes[reference_index].evaluation.state,
            EvaluationState::Stale
        );
        assert!(
            graph.nodes[reference_index]
                .evaluation
                .message
                .as_deref()
                .is_some_and(|message| message.contains("Referenced output changed"))
        );
    }

    #[test]
    fn reference_input_round_trips_stable_target_identity_through_sidecar() {
        let mut graph = GraphDocument::sample();
        let null_index = graph.add_null_operator_node("OUT_SERIALIZED");
        let reference_index = graph
            .add_reference_input_node(null_index)
            .expect("null output should be a compatible reference target");
        let target = graph.nodes[reference_index]
            .reference_input
            .as_ref()
            .expect("reference input should exist")
            .targets
            .first()
            .expect("reference input should have a target")
            .target
            .clone();

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();
        let restored_reference_index = restored
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::ReferenceInput)
            .expect("restored graph should include reference node");
        let restored_resolution = restored
            .reference_input_resolution(restored_reference_index)
            .expect("restored reference should resolve");

        assert_eq!(
            restored_resolution.status,
            ReferenceDiagnosticStatus::Resolved
        );
        assert_eq!(restored_resolution.target, target);
        assert_eq!(
            restored_resolution.readable_path,
            "main/OUT_SERIALIZED:geometry"
        );
    }

    #[test]
    fn reference_input_supports_visible_multi_source_target_set() {
        let mut graph = GraphDocument::sample();
        let first_null_index = graph.add_null_operator_node("OUT_A");
        let second_null_index = graph.add_null_operator_node("OUT_B");
        let reference_index = graph
            .add_reference_input_node(first_null_index)
            .expect("first null output should be referenceable");
        assert!(graph.add_reference_target_to_node(reference_index, second_null_index));

        let entries = graph
            .reference_input_resolutions(reference_index)
            .expect("reference node should expose target entries");
        let info = graph
            .selected_node_info(reference_index)
            .expect("reference node should expose node info");

        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|entry| entry.enabled));
        assert!(
            entries
                .iter()
                .all(|entry| entry.resolution.status == ReferenceDiagnosticStatus::Resolved)
        );
        assert_eq!(info.input_count, 2);
        assert_eq!(info.output_count, graph.visible_output_count() * 2);
        assert_eq!(
            info.reference_input
                .expect("reference info should exist")
                .targets
                .len(),
            2
        );
    }

    #[test]
    fn referenced_outputs_expose_consumers_and_delete_warnings() {
        let mut graph = GraphDocument::sample();
        let null_index = graph.add_null_operator_node("OUT_A");
        let reference_index = graph
            .add_reference_input_node(null_index)
            .expect("null output should be referenceable");

        let source_info = graph
            .selected_node_info(null_index)
            .expect("referenced output should have node info");
        let warning = graph
            .reference_output_change_warning_for_node(null_index)
            .expect("referenced output should warn before output changes");

        assert_eq!(source_info.reference_consumers.len(), 1);
        assert_eq!(
            source_info.reference_consumers[0].reference_node_index,
            reference_index
        );
        assert_eq!(
            source_info.reference_consumers[0].status,
            ReferenceDiagnosticStatus::Resolved
        );
        assert_eq!(warning.target_node_name, "OUT_A");
        assert_eq!(warning.affected_references.len(), 1);
        assert_eq!(
            warning.affected_references[0].reference_node_index,
            reference_index
        );
    }

    #[test]
    fn reference_targets_expose_navigation_indices_and_diagnostics() {
        let mut graph = GraphDocument::sample();
        let first_null_index = graph.add_null_operator_node("OUT_A");
        let second_null_index = graph.add_null_operator_node("OUT_B");
        let second_target_node_id = graph.nodes[second_null_index].node_id.clone();
        let reference_index = graph
            .add_reference_input_node(first_null_index)
            .expect("first null output should be referenceable");
        assert!(graph.add_reference_target_to_node(reference_index, second_null_index));
        assert!(
            graph.set_reference_target_enabled(reference_index, &second_target_node_id, false,)
        );
        graph
            .remove_node(second_null_index)
            .expect("disabled target node should be removable");
        let reference_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::ReferenceInput)
            .expect("reference node should still exist");

        let info = graph
            .selected_node_info(reference_index)
            .expect("reference node should have info")
            .reference_input
            .expect("reference info should exist");

        assert_eq!(info.targets[0].target_node_index, Some(first_null_index));
        assert!(info.targets.iter().any(|target| {
            !target.enabled
                && target.status == ReferenceDiagnosticStatus::MissingNode
                && target.diagnostic.as_deref() == Some("Reference target node is missing.")
        }));
    }

    #[test]
    fn reference_input_disable_retains_target_but_excludes_output() {
        let mut graph = GraphDocument::sample();
        let first_null_index = graph.add_null_operator_node("OUT_A");
        let second_null_index = graph.add_null_operator_node("OUT_B");
        let second_target_node_id = graph.nodes[second_null_index].node_id.clone();
        let reference_index = graph
            .add_reference_input_node(first_null_index)
            .expect("first null output should be referenceable");
        assert!(graph.add_reference_target_to_node(reference_index, second_null_index));

        assert!(
            graph.set_reference_target_enabled(reference_index, &second_target_node_id, false,)
        );

        let entries = graph
            .reference_input_resolutions(reference_index)
            .expect("reference node should expose target entries");
        let disabled_entry = entries
            .iter()
            .find(|entry| entry.resolution.target.node_id == second_target_node_id)
            .expect("disabled target should remain visible");

        assert!(!disabled_entry.enabled);
        assert_eq!(entries.len(), 2);
        assert_eq!(
            graph.node_output_record_count_for_index(reference_index),
            graph.visible_output_count()
        );
        assert_eq!(
            graph.nodes[reference_index].evaluation.state,
            EvaluationState::Stale
        );
    }

    #[test]
    fn reference_input_remove_target_deletes_entry_but_disable_does_not() {
        let mut graph = GraphDocument::sample();
        let first_null_index = graph.add_null_operator_node("OUT_A");
        let second_null_index = graph.add_null_operator_node("OUT_B");
        let second_target_node_id = graph.nodes[second_null_index].node_id.clone();
        let reference_index = graph
            .add_reference_input_node(first_null_index)
            .expect("first null output should be referenceable");
        assert!(graph.add_reference_target_to_node(reference_index, second_null_index));

        assert!(
            graph.set_reference_target_enabled(reference_index, &second_target_node_id, false,)
        );
        assert_eq!(
            graph
                .reference_input_resolutions(reference_index)
                .expect("reference entries should exist")
                .len(),
            2
        );

        assert!(graph.remove_reference_target_from_node(reference_index, &second_target_node_id,));
        assert_eq!(
            graph
                .reference_input_resolutions(reference_index)
                .expect("reference entries should exist")
                .len(),
            1
        );
    }

    #[test]
    fn disabled_missing_reference_target_is_retained_without_blocking_evaluation() {
        let mut graph = GraphDocument::sample();
        let first_null_index = graph.add_null_operator_node("OUT_A");
        let second_null_index = graph.add_null_operator_node("OUT_B");
        let second_target_node_id = graph.nodes[second_null_index].node_id.clone();
        let reference_index = graph
            .add_reference_input_node(first_null_index)
            .expect("first null output should be referenceable");
        assert!(graph.add_reference_target_to_node(reference_index, second_null_index));
        assert!(
            graph.set_reference_target_enabled(reference_index, &second_target_node_id, false,)
        );

        graph
            .remove_node(second_null_index)
            .expect("disabled target node should be removable");
        let reference_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::ReferenceInput)
            .expect("reference node should still exist");
        graph.demand_output_evaluation();
        let info = graph
            .selected_node_info(reference_index)
            .expect("reference node should still have info");
        let target_info = info.reference_input.expect("reference info should exist");

        assert_eq!(info.status, NodeStatus::Healthy);
        assert!(target_info.targets.iter().any(
            |target| !target.enabled && target.status == ReferenceDiagnosticStatus::MissingNode
        ));
    }

    #[test]
    fn enabled_coordinate_incompatible_reference_target_blocks_output() {
        let mut graph = GraphDocument::sample();
        let first_null_index = graph.add_null_operator_node("OUT_A");
        let second_null_index = graph.add_null_operator_node("OUT_B");
        graph.nodes[second_null_index].coordinate_contract = Some(alternate_substrate_contract());
        let reference_index = graph
            .add_reference_input_node(first_null_index)
            .expect("first null output should be referenceable");
        assert!(graph.add_reference_target_to_node(reference_index, second_null_index));

        let info = graph
            .selected_node_info(reference_index)
            .expect("reference node should have info");
        let reference_info = info.reference_input.expect("reference info should exist");

        assert_eq!(info.status, NodeStatus::Failed);
        assert_eq!(info.output_count, 0);
        assert!(!reference_info.applies_hidden_transform);
        assert!(reference_info.targets.iter().any(|target| {
            target.enabled
                && target.status == ReferenceDiagnosticStatus::CoordinateIncompatibleRepairable
        }));
        assert!(
            graph
                .reference_coordinate_repair_summary(reference_index)
                .expect("repair should be offered")
                .contains("visible substrate projection")
        );
    }

    #[test]
    fn disabled_coordinate_incompatible_reference_target_stays_visible() {
        let mut graph = GraphDocument::sample();
        let first_null_index = graph.add_null_operator_node("OUT_A");
        let second_null_index = graph.add_null_operator_node("OUT_B");
        graph.nodes[second_null_index].coordinate_contract = Some(alternate_substrate_contract());
        let second_target_node_id = graph.nodes[second_null_index].node_id.clone();
        let reference_index = graph
            .add_reference_input_node(first_null_index)
            .expect("first null output should be referenceable");
        assert!(graph.add_reference_target_to_node(reference_index, second_null_index));
        assert!(
            graph.set_reference_target_enabled(reference_index, &second_target_node_id, false,)
        );

        let info = graph
            .selected_node_info(reference_index)
            .expect("reference node should have info");
        let reference_info = info.reference_input.expect("reference info should exist");

        assert_eq!(info.status, NodeStatus::Healthy);
        assert_eq!(info.output_count, graph.visible_output_count());
        assert!(reference_info.targets.iter().any(|target| {
            !target.enabled
                && target.status == ReferenceDiagnosticStatus::CoordinateIncompatibleRepairable
        }));
    }

    #[test]
    fn missing_coordinate_contract_is_diagnostic_until_disabled_or_fixed() {
        let mut graph = GraphDocument::sample();
        let first_null_index = graph.add_null_operator_node("OUT_A");
        let second_null_index = graph.add_null_operator_node("OUT_B");
        graph.nodes[second_null_index].coordinate_contract = None;
        let second_target_node_id = graph.nodes[second_null_index].node_id.clone();
        let reference_index = graph
            .add_reference_input_node(first_null_index)
            .expect("first null output should be referenceable");
        assert!(graph.add_reference_target_to_node(reference_index, second_null_index));

        let blocked = graph
            .selected_node_info(reference_index)
            .expect("reference node should have info");
        assert_eq!(blocked.status, NodeStatus::Failed);
        assert_eq!(blocked.output_count, 0);
        assert!(
            graph
                .reference_coordinate_repair_summary(reference_index)
                .is_none()
        );

        assert!(
            graph.set_reference_target_enabled(reference_index, &second_target_node_id, false,)
        );
        let disabled = graph
            .selected_node_info(reference_index)
            .expect("reference node should have info");
        let reference_info = disabled
            .reference_input
            .expect("reference info should exist");

        assert_eq!(disabled.status, NodeStatus::Healthy);
        assert!(reference_info.targets.iter().any(|target| {
            !target.enabled && target.status == ReferenceDiagnosticStatus::CoordinateContractMissing
        }));
    }

    #[test]
    fn assisted_projection_repair_creates_visible_node_and_retargets_reference() {
        let mut graph = GraphDocument::sample();
        let first_null_index = graph.add_null_operator_node("OUT_A");
        let second_null_index = graph.add_null_operator_node("OUT_B");
        graph.nodes[second_null_index].coordinate_contract = Some(alternate_substrate_contract());
        let reference_index = graph
            .add_reference_input_node(first_null_index)
            .expect("first null output should be referenceable");
        assert!(graph.add_reference_target_to_node(reference_index, second_null_index));
        for node in &mut graph.nodes {
            node.layout_position = GraphPoint::new(-1.25, 1.80);
        }

        let projection_index = graph
            .create_assisted_projection_for_first_repairable_reference_target(reference_index)
            .expect("repair should create a visible projection node");
        let projection_node_id = graph.nodes[projection_index].node_id.clone();
        let reference_index = projection_index + 1;
        let info = graph
            .selected_node_info(reference_index)
            .expect("reference node should have info");
        let reference_info = info.reference_input.expect("reference info should exist");

        assert_eq!(
            graph.nodes[projection_index].kind,
            NodeKind::SubstrateProjection
        );
        assert!(
            (graph.nodes[projection_index].layout_position.x - -1.17).abs() < 0.0001,
            "{:?}",
            graph.nodes[projection_index].layout_position
        );
        assert!(
            (graph.nodes[projection_index].layout_position.y - 1.92).abs() < 0.0001,
            "{:?}",
            graph.nodes[projection_index].layout_position
        );
        assert!(
            graph.nodes[projection_index]
                .substrate_projection
                .as_ref()
                .expect("projection node should carry repair contract")
                .repair_summary
                .contains("alternate-markov-pixel-space")
        );
        assert_eq!(info.status, NodeStatus::Healthy);
        assert_eq!(info.output_count, graph.visible_output_count() * 2);
        assert!(reference_info.targets.iter().any(|target| {
            target.target.node_id == projection_node_id
                && target.status == ReferenceDiagnosticStatus::Resolved
        }));

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();
        let restored_projection = restored
            .nodes
            .iter()
            .find(|node| node.node_id == projection_node_id)
            .expect("projection node should round-trip as durable graph state");
        assert_eq!(restored_projection.kind, NodeKind::SubstrateProjection);
        assert_eq!(
            restored_projection
                .coordinate_contract
                .as_ref()
                .expect("projection should keep target coordinate contract"),
            &SubstrateCoordinateContract::demo_byteplot()
        );
    }

    #[test]
    fn reference_target_set_provenance_and_enablement_affect_fingerprint() {
        let mut graph = GraphDocument::sample();
        let first_null_index = graph.add_null_operator_node("OUT_A");
        let second_null_index = graph.add_null_operator_node("OUT_B");
        let second_target_node_id = graph.nodes[second_null_index].node_id.clone();
        let reference_index = graph
            .add_reference_input_node(first_null_index)
            .expect("first null output should be referenceable");
        assert!(graph.add_reference_target_to_node(reference_index, second_null_index));
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .expect("output node should exist");
        let before = graph.input_cache_keys_for_node(output_index);

        assert!(
            graph.set_reference_target_enabled(reference_index, &second_target_node_id, false,)
        );
        let after = graph.input_cache_keys_for_node(output_index);
        let info = graph
            .selected_node_info(reference_index)
            .expect("reference node should have info")
            .reference_input
            .expect("reference info should exist");

        assert_ne!(before, after);
        assert!(
            info.targets
                .iter()
                .any(|target| target.provenance.source_node_name == "OUT_B")
        );
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
    fn attribute_table_preview_rows_caps_large_outputs() {
        let mut graph = GraphDocument::sample();
        graph.load_synthetic_render_benchmark(10_000, 0);

        let rows = graph.attribute_table_preview_rows(&AttributeTableQuery::default(), 200);

        assert_eq!(rows.len(), 200);
        assert_eq!(rows.first().map(|row| row.record_index), Some(0));
        assert_eq!(rows.last().map(|row| row.record_index), Some(199));
        assert!(rows.iter().all(|row| row.is_native_cubic_bezier));
        assert_eq!(graph.visible_output_count(), 10_000);
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
        let generated = filter_node.generated.expect("filter should be generated");
        assert_eq!(generated.source, GeneratedNodeSource::AttributeTableCommit);
        assert_eq!(generated.binding_state, GeneratedNodeBindingState::Managed);
        assert!(filter_node.layout_position.y >= 0.8);
        let generated_info = graph
            .selected_node_info(1)
            .expect("filter node info should exist")
            .generated
            .expect("filter node info should expose generated metadata");
        assert_eq!(
            generated_info.source,
            GeneratedNodeSource::AttributeTableCommit
        );
        assert_eq!(
            generated_info.binding_state,
            GeneratedNodeBindingState::Managed
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
        let restored_generated = restored_filter
            .generated
            .expect("generated filter metadata should round trip");
        assert_eq!(
            restored_generated.source,
            GeneratedNodeSource::AttributeTableCommit
        );
        assert_eq!(
            restored_generated.binding_state,
            GeneratedNodeBindingState::Managed
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
            node_id: "scratch.filter".to_owned(),
            name: "Scratch Filter".to_owned(),
            kind: NodeKind::Filter,
            layout_position: GraphPoint::new(0.5, 0.1),
            generated: None,
            coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
            output_operator: None,
            null_operator: None,
            reference_input: None,
            substrate_projection: None,
            python_operator: None,
            procedural_asset: None,
            native_operator: None,
            evaluation: NodeEvaluation {
                state: EvaluationState::Stale,
                manual: false,
                message: None,
            },
            participates_in_output: false,
            comment: String::new(),
            show_comment_in_network: false,
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
    fn python_operator_node_reports_declaration_and_dependency_status() {
        let mut graph = GraphDocument::sample();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        graph.python_environment = sample_python_environment_descriptor();
        let node_index = graph.add_python_operator_node("vy.blur_curves");

        let info = graph
            .selected_node_info(node_index)
            .expect("python node info should exist");

        assert_eq!(info.kind, NodeKind::PythonOperator);
        assert_eq!(info.role, "Compute");
        assert_eq!(info.input_count, 1);
        assert_eq!(info.output_count, 1);
        assert_eq!(info.status, NodeStatus::Healthy);
        assert_eq!(info.evaluation.state, EvaluationState::Manual);
        let python = info
            .python_operator
            .expect("python operator info should exist");
        assert_eq!(python.declaration_id, "vy.blur_curves");
        assert_eq!(python.display_name, "Blur curves");
        assert_eq!(python.version, "0.1.0");
        assert_eq!(
            python.dependency_status,
            PythonOperatorDependencyStatus::Ready
        );
        assert_eq!(python.requirements, vec!["numpy==2.0.0".to_owned()]);
    }

    #[test]
    fn python_operator_node_dependency_status_tracks_environment_health() {
        let mut graph = GraphDocument::sample();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        let node_index = graph.add_python_operator_node("vy.blur_curves");

        let missing = graph
            .selected_node_info(node_index)
            .expect("python node info should exist")
            .python_operator
            .expect("python operator info should exist")
            .dependency_status;
        assert_eq!(missing, PythonOperatorDependencyStatus::MissingEnvironment);

        graph.python_environment = sample_python_environment_descriptor();
        graph
            .python_environment
            .dependency_health
            .failed_imports
            .push("numpy".to_owned());
        let failed = graph
            .selected_node_info(node_index)
            .expect("python node info should exist")
            .python_operator
            .expect("python operator info should exist")
            .dependency_status;

        assert_eq!(failed, PythonOperatorDependencyStatus::FailedEnvironment);
    }

    #[test]
    fn python_environment_resolve_plan_unions_project_and_enabled_operator_requirements() {
        let mut graph = GraphDocument::sample();
        graph.python_environment.project_requirements = PythonProjectRequirements {
            requirements: vec!["pyarrow==16.1.0".to_owned()],
        };
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        graph.add_python_operator_node("vy.blur_curves");

        let plan = graph
            .python_environment_resolve_plan(PythonEnvironmentResolveTrigger::ExplicitUserAction);

        assert_eq!(
            plan.trigger,
            PythonEnvironmentResolveTrigger::ExplicitUserAction
        );
        assert_eq!(plan.unique_requirement_count(), 2);
        assert!(plan.requirements.iter().any(|contribution| {
            contribution.requirement == "pyarrow==16.1.0"
                && contribution.source == PythonRequirementSource::Project
        }));
        assert!(plan.requirements.iter().any(|contribution| {
            contribution.requirement == "numpy==2.0.0"
                && contribution.source
                    == PythonRequirementSource::Operator {
                        operator_id: "vy.blur_curves".to_owned(),
                    }
        }));
        assert!(plan.conflicts.is_empty());
    }

    #[test]
    fn python_environment_resolve_plan_reports_operator_conflicts() {
        let mut graph = GraphDocument::sample();
        graph.python_environment.project_requirements = PythonProjectRequirements {
            requirements: vec!["numpy==1.26.4".to_owned()],
        };
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        graph.add_python_operator_node("vy.blur_curves");

        let plan = graph
            .python_environment_resolve_plan(PythonEnvironmentResolveTrigger::ExplicitUserAction);

        assert_eq!(plan.conflicts.len(), 1);
        assert_eq!(plan.conflicts[0].package, "numpy");
        assert_eq!(plan.conflicts[0].requirements.len(), 2);
        assert_eq!(
            plan.conflict_summary(),
            "1 dependency conflict(s) detected."
        );
    }

    #[test]
    fn python_environment_resolve_lifecycle_records_ready_identity() {
        let mut graph = GraphDocument::sample();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        graph.add_python_operator_node("vy.blur_curves");

        let plan = graph
            .begin_python_environment_resolve(PythonEnvironmentResolveTrigger::ExplicitUserAction);
        assert_eq!(
            graph.python_environment.lock_status,
            PythonEnvironmentStatus::Resolving
        );
        assert_eq!(plan.trigger.as_str(), "Explicit user action");
        assert!(
            graph
                .python_environment
                .resolve_state
                .in_progress
                .as_ref()
                .is_some_and(|run| run.resolver_tool == "uv")
        );

        graph.complete_python_environment_resolve(
            "lock:resolved",
            "0.7.1",
            ".houdini/python/envs/project-python",
            8,
        );

        assert_eq!(
            graph.python_environment.lock_status,
            PythonEnvironmentStatus::Ready
        );
        assert_eq!(
            graph.python_environment.lock_digest.as_deref(),
            Some("lock:resolved")
        );
        assert_eq!(
            graph.python_environment.resolver.version.as_deref(),
            Some("0.7.1")
        );
        assert_eq!(
            graph.python_environment.environment_path.as_deref(),
            Some(".houdini/python/envs/project-python")
        );
        assert_eq!(graph.python_environment.dependency_health.package_count, 8);
        assert!(graph.python_environment.resolve_state.in_progress.is_none());
    }

    #[test]
    fn python_environment_cancel_or_failure_preserves_previous_ready_snapshot() {
        let mut graph = GraphDocument::sample();
        graph.python_environment = sample_python_environment_descriptor();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        graph.add_python_operator_node("vy.blur_curves");

        graph.begin_python_environment_resolve(PythonEnvironmentResolveTrigger::ExplicitUserAction);
        graph.cancel_python_environment_resolve();
        assert_eq!(
            graph.python_environment.lock_status,
            PythonEnvironmentStatus::Ready
        );
        assert_eq!(
            graph.python_environment.lock_digest.as_deref(),
            Some("lock:abc123")
        );
        assert_eq!(graph.python_environment.dependency_health.package_count, 3);

        graph.begin_python_environment_resolve(PythonEnvironmentResolveTrigger::ExplicitUserAction);
        graph.fail_python_environment_resolve("uv resolve failed");
        assert_eq!(
            graph.python_environment.lock_status,
            PythonEnvironmentStatus::Failed
        );
        assert_eq!(
            graph.python_environment.last_failure_summary.as_deref(),
            Some("uv resolve failed")
        );
        assert_eq!(
            graph
                .python_environment
                .resolve_state
                .previous_ready
                .as_ref()
                .and_then(|snapshot| snapshot.lock_digest.as_deref()),
            Some("lock:abc123")
        );
    }

    #[test]
    fn python_operator_cache_key_invalidates_on_explicit_inputs() {
        let mut graph = GraphDocument::sample();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        graph.python_environment = sample_python_environment_descriptor();
        let node_index = graph.add_python_operator_node("vy.blur_curves");

        let base_key = graph
            .python_operator_cache_key(node_index)
            .expect("cache key should derive from declaration and environment");
        assert_eq!(base_key.operator_id, "vy.blur_curves");
        assert_eq!(base_key.declaration_version, "0.1.0");
        assert_eq!(
            base_key.dependency_lock_digest.as_deref(),
            Some("lock:abc123")
        );
        assert_eq!(base_key.input_cache_keys.len(), node_index);

        graph.nodes[node_index].parameter.value = 0.75;
        let changed_parameter = graph
            .python_operator_cache_key(node_index)
            .expect("cache key should update after parameter changes");
        assert_ne!(changed_parameter.key_digest, base_key.key_digest);
        assert_ne!(
            changed_parameter.parameter_digest,
            base_key.parameter_digest
        );

        graph.nodes[node_index].parameter.value = 0.0;
        graph.python_environment.lock_digest = Some("lock:def456".to_owned());
        let changed_lock = graph
            .python_operator_cache_key(node_index)
            .expect("cache key should update after lock changes");
        assert_ne!(changed_lock.key_digest, base_key.key_digest);
        assert_ne!(
            changed_lock.dependency_lock_digest,
            base_key.dependency_lock_digest
        );

        graph.python_environment.lock_digest = Some("lock:abc123".to_owned());
        graph.python_operator_declarations[0].entry_point.source = PythonOperatorSource::File {
            path: "operators/blur_curves_v2.py".to_owned(),
        };
        let changed_source = graph
            .python_operator_cache_key(node_index)
            .expect("cache key should update after source changes");
        assert_ne!(changed_source.key_digest, base_key.key_digest);
        assert_ne!(changed_source.source_digest, base_key.source_digest);
    }

    #[test]
    fn python_operator_cache_key_tracks_upstream_input_keys() {
        let mut graph = GraphDocument::sample();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        graph.python_environment = sample_python_environment_descriptor();
        let node_index = graph.add_python_operator_node("vy.blur_curves");

        let base_key = graph
            .python_operator_cache_key(node_index)
            .expect("cache key should exist");
        graph.nodes[1].parameter.value = 0.25;
        let changed_input = graph
            .python_operator_cache_key(node_index)
            .expect("cache key should update after upstream graph input changes");

        assert_ne!(changed_input.key_digest, base_key.key_digest);
        assert_ne!(changed_input.input_cache_keys, base_key.input_cache_keys);
    }

    #[test]
    fn python_operator_provenance_persists_typed_output_metadata() {
        let mut graph = GraphDocument::sample();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        graph.python_environment = sample_python_environment_descriptor();
        let node_index = graph.add_python_operator_node("vy.blur_curves");

        let record = graph
            .record_python_operator_output(
                node_index,
                PythonOperatorOutputCounts {
                    geometry_records: 4,
                    attribute_records: 2,
                    layer_records: 1,
                },
            )
            .expect("provenance should be recorded for python operator");

        assert_eq!(record.operator_id, "vy.blur_curves");
        assert_eq!(
            record.source_path.as_deref(),
            Some("operators/blur_curves.py")
        );
        assert_eq!(
            record.dependency_identity.interpreter_path.as_deref(),
            Some(".houdini/python/envs/project-python")
        );
        assert_eq!(record.output_counts.geometry_records, 4);

        let json = graph.to_sidecar_json().unwrap();
        assert!(json.contains("cache_key"));
        assert!(json.contains("provenance"));
        assert!(!json.contains("pickle"));

        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();
        let restored_python = restored
            .nodes
            .iter()
            .find(|node| node.kind == NodeKind::PythonOperator)
            .expect("python node should restore")
            .python_operator
            .as_ref()
            .expect("python node should restore");
        assert_eq!(
            restored_python
                .cache_key
                .as_ref()
                .expect("cache key should restore")
                .operator_id,
            "vy.blur_curves"
        );
        assert_eq!(
            restored_python
                .provenance
                .as_ref()
                .expect("provenance should restore")
                .output_counts
                .geometry_records,
            4
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn python_operator_process_executes_typed_geometry_boundary() {
        let project = tempfile::tempdir().unwrap();
        let operators_dir = project.path().join("operators");
        std::fs::create_dir_all(&operators_dir).unwrap();
        std::fs::write(
            operators_dir.join("blur_curves.py"),
            r#"
import argparse
import json

parser = argparse.ArgumentParser()
parser.add_argument("--houdini-input", required=True)
parser.add_argument("--houdini-output", required=True)
args = parser.parse_args()

with open(args.houdini_input, "r", encoding="utf-8") as handle:
    payload = json.load(handle)

print(f"records={len(payload['records'])}")

with open(args.houdini_output, "w", encoding="utf-8") as handle:
    json.dump(payload, handle)
"#,
        )
        .unwrap();

        let mut graph = GraphDocument::sample();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        graph.python_environment = sample_python_environment_descriptor();
        graph.python_environment.environment_path = Some(configured_python_interpreter());
        let node_index = graph.add_python_operator_node("vy.blur_curves");

        let report = graph
            .execute_python_operator_process(
                node_index,
                project.path(),
                std::time::Duration::from_secs(5),
            )
            .expect("python process should execute");

        assert_eq!(report.exit_status, Some(0));
        assert!(!report.timed_out);
        assert!(report.stdout.contains("records=4"));
        assert_eq!(report.output_record_count, 4);
        assert_eq!(
            graph.source.metadata.provenance,
            SourceProvenance::PythonOperator
        );
        assert_eq!(graph.cubic_bezier_count(), 2);
        assert!(
            graph
                .recording_geometry
                .iter()
                .any(|geometry| matches!(geometry, Geometry::CubicBezier(_)))
        );
        assert!(
            graph
                .nodes
                .get(node_index)
                .and_then(|node| node.python_operator.as_ref())
                .and_then(|python| python.provenance.as_ref())
                .is_some_and(|record| record.output_counts.geometry_records == 4)
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn python_operator_process_captures_traceback_summary() {
        let project = tempfile::tempdir().unwrap();
        let operators_dir = project.path().join("operators");
        std::fs::create_dir_all(&operators_dir).unwrap();
        std::fs::write(
            operators_dir.join("blur_curves.py"),
            "raise RuntimeError('boom from trusted operator')\n",
        )
        .unwrap();

        let mut graph = GraphDocument::sample();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        graph.python_environment = sample_python_environment_descriptor();
        graph.python_environment.environment_path = Some(configured_python_interpreter());
        let node_index = graph.add_python_operator_node("vy.blur_curves");

        let report = graph
            .execute_python_operator_process(
                node_index,
                project.path(),
                std::time::Duration::from_secs(5),
            )
            .expect("python process failure should still return a report");

        assert_ne!(report.exit_status, Some(0));
        assert!(report.stderr.contains("RuntimeError"));
        assert!(
            report
                .traceback_summary
                .as_deref()
                .is_some_and(|summary| summary.contains("RuntimeError"))
        );
        assert_eq!(
            graph.nodes[node_index].evaluation.state,
            EvaluationState::Failed
        );
        assert!(
            graph.nodes[node_index]
                .evaluation
                .message
                .as_deref()
                .is_some_and(|message| message.contains("RuntimeError"))
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn python_operator_process_times_out_without_global_default_python() {
        let project = tempfile::tempdir().unwrap();
        let operators_dir = project.path().join("operators");
        std::fs::create_dir_all(&operators_dir).unwrap();
        std::fs::write(
            operators_dir.join("blur_curves.py"),
            "import time\ntime.sleep(2)\n",
        )
        .unwrap();

        let mut graph = GraphDocument::sample();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        graph.python_environment = sample_python_environment_descriptor();
        let node_index = graph.add_python_operator_node("vy.blur_curves");
        assert!(
            graph
                .execute_python_operator_process(
                    node_index,
                    project.path(),
                    std::time::Duration::from_millis(20),
                )
                .is_err()
        );

        graph.python_environment.environment_path = Some(configured_python_interpreter());
        let report = graph
            .execute_python_operator_process(
                node_index,
                project.path(),
                std::time::Duration::from_millis(20),
            )
            .expect("timeout should return process report");

        assert!(report.timed_out);
        assert_ne!(report.exit_status, Some(0));
        assert_eq!(
            graph.nodes[node_index].evaluation.state,
            EvaluationState::Failed
        );
    }

    #[test]
    fn python_operator_environment_health_blocks_or_allows_run_requests() {
        let mut graph = GraphDocument::sample();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        let node_index = graph.add_python_operator_node("vy.blur_curves");

        graph.request_node_run(node_index);
        assert_eq!(
            graph.nodes[node_index].evaluation.state,
            EvaluationState::Manual
        );
        assert_eq!(
            graph.nodes[node_index].evaluation.message.as_deref(),
            Some("Project Python environment is not configured.")
        );

        graph.python_environment = sample_python_environment_descriptor();
        graph.python_environment.lock_status = PythonEnvironmentStatus::Stale;
        graph.request_node_run(node_index);
        assert_eq!(
            graph.nodes[node_index].evaluation.state,
            EvaluationState::Stale
        );
        assert!(!graph.nodes[node_index].evaluation.manual);

        graph.python_environment = sample_python_environment_descriptor();
        graph.python_environment.lock_status = PythonEnvironmentStatus::Failed;
        graph.python_environment.last_failure_summary = Some("uv lock conflict".to_owned());
        graph.request_node_run(node_index);
        assert_eq!(
            graph.nodes[node_index].evaluation.state,
            EvaluationState::Manual
        );
        assert_eq!(
            graph.nodes[node_index]
                .python_operator
                .as_ref()
                .expect("python operator should exist")
                .last_failure_summary
                .as_deref(),
            Some("Project Python environment has dependency or validation failures.")
        );

        graph.python_environment = sample_python_environment_descriptor();
        graph.request_node_run(node_index);
        assert_eq!(
            graph.nodes[node_index].evaluation.state,
            EvaluationState::Running
        );
        assert!(graph.nodes[node_index].evaluation.message.is_none());
    }

    #[test]
    fn stale_python_environment_marks_python_nodes_stale_on_demand() {
        let mut graph = GraphDocument::sample();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        graph.python_environment = sample_python_environment_descriptor();
        graph.python_environment.lock_status = PythonEnvironmentStatus::Stale;
        let node_index = graph.add_python_operator_node("vy.blur_curves");

        graph.demand_output_evaluation();

        assert_eq!(
            graph.nodes[node_index].evaluation.state,
            EvaluationState::Stale
        );
        assert_eq!(
            graph.nodes[node_index].evaluation.message.as_deref(),
            Some("Project Python environment must be resolved before execution.")
        );
    }

    #[test]
    fn python_operator_nodes_round_trip_through_sidecar() {
        let mut graph = GraphDocument::sample();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());
        let node_index = graph.add_python_operator_node("vy.blur_curves");
        let second_node_index = graph.add_python_operator_node("vy.blur_curves");
        graph.set_node_layout_position(node_index, GraphPoint::new(0.52, 0.42));
        graph.set_node_layout_position(second_node_index, GraphPoint::new(0.62, 0.43));

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        let restored_python_nodes = restored
            .nodes
            .iter()
            .filter(|node| node.kind == NodeKind::PythonOperator)
            .collect::<Vec<_>>();
        assert_eq!(restored_python_nodes.len(), 2);
        assert_eq!(
            restored_python_nodes[0]
                .python_operator
                .as_ref()
                .expect("python operator payload should restore")
                .declaration_id,
            "vy.blur_curves"
        );
        assert_eq!(
            restored_python_nodes[0]
                .python_operator
                .as_ref()
                .expect("python operator payload should restore")
                .instance_id,
            "python_operator_1"
        );
        assert_eq!(
            restored_python_nodes[1]
                .python_operator
                .as_ref()
                .expect("python operator payload should restore")
                .instance_id,
            "python_operator_2"
        );
        assert_eq!(
            restored_python_nodes[0].layout_position,
            GraphPoint::new(0.52, 0.42)
        );
        assert_eq!(
            restored_python_nodes[1].layout_position,
            GraphPoint::new(0.62, 0.43)
        );
        assert_eq!(
            restored.graph_layout().edges.len(),
            restored.nodes.len() - 1
        );
    }

    #[test]
    fn graph_layout_node_positions_can_leave_initial_viewport() {
        let mut graph = GraphDocument::sample();

        graph.set_node_layout_position(1, GraphPoint::new(0.25, 0.75));
        let layout = graph.graph_layout();
        assert_eq!(layout.nodes[1].position, GraphPoint::new(0.25, 0.75));

        graph.set_node_layout_position(1, GraphPoint::new(-1.0, 2.0));
        let layout = graph.graph_layout();
        assert_eq!(layout.nodes[1].position, GraphPoint::new(-1.0, 2.0));

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();
        assert_eq!(
            restored.graph_layout().nodes[1].position,
            GraphPoint::new(-1.0, 2.0)
        );
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
    fn graph_annotations_round_trip_through_sidecar() {
        let mut graph = GraphDocument::sample();
        graph.annotations.clear();
        let selected_node_index = 1;
        graph
            .add_network_box_for_node(selected_node_index)
            .expect("network box should be created for selected node");
        graph
            .add_sticky_note_near_node(selected_node_index)
            .expect("sticky note should be created near selected node");

        graph.annotations[0].title = "Filter Prep".to_owned();
        graph.annotations[0].collapsed = true;
        graph.annotations[1].title = "Publish Note".to_owned();
        graph.annotations[1].text = "Raise threshold before output.".to_owned();

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(restored.annotations.len(), 2);
        assert_eq!(
            restored.annotations[0].kind,
            GraphAnnotationKind::NetworkBox
        );
        assert_eq!(restored.annotations[0].title, "Filter Prep");
        assert!(restored.annotations[0].collapsed);
        assert_eq!(
            restored.annotations[0].member_node_ids,
            vec![graph.nodes[selected_node_index].node_id.clone()]
        );
        assert_eq!(
            restored.annotations[1].kind,
            GraphAnnotationKind::StickyNote
        );
        assert_eq!(restored.annotations[1].title, "Publish Note");
        assert_eq!(
            restored.annotations[1].text,
            "Raise threshold before output."
        );
    }

    #[test]
    fn network_view_display_options_round_trip_through_sidecar() {
        let mut graph = GraphDocument::sample();
        graph.network_view.node_ring_visibility = NetworkNodeRingVisibility::Always;
        graph.network_view.max_node_name_width = 132.0;
        graph.network_view.long_wire_fading = 0.25;
        graph.network_view.grid_spacing = 3.0;
        graph.network_view.background_brightness = 0.68;
        graph.network_view.error_badge = NetworkBadgeVisibility::Hide;
        graph.network_view.warning_badge = NetworkBadgeVisibility::Large;
        graph.network_view.comment_badge = NetworkBadgeVisibility::Normal;
        graph.network_view.time_dependent_badge = NetworkBadgeVisibility::Hide;
        graph.network_view.lock_badge = NetworkBadgeVisibility::Large;
        graph.network_view.has_data_badge = NetworkBadgeVisibility::Hide;
        graph.network_view.cached_code_badge = NetworkBadgeVisibility::Large;
        graph.network_view.constraint_badge = NetworkBadgeVisibility::Normal;
        graph.network_view.compilable_badge = NetworkBadgeVisibility::Hide;

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(restored.network_view, graph.network_view);
    }

    #[test]
    fn sidecar_without_network_view_uses_display_defaults() {
        let graph = GraphDocument::sample();
        let mut value =
            serde_json::from_str::<serde_json::Value>(&graph.to_sidecar_json().unwrap())
                .expect("sidecar should be valid json");
        value
            .as_object_mut()
            .expect("sidecar should be an object")
            .remove("network_view");
        let json = serde_json::to_string_pretty(&value).unwrap();

        let mut restored = GraphDocument::sample();
        restored.network_view.node_ring_visibility = NetworkNodeRingVisibility::Hidden;
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(
            restored.network_view.node_ring_visibility,
            NetworkNodeRingVisibility::Selected
        );
        assert_eq!(
            restored.network_view.error_badge,
            NetworkBadgeVisibility::Large
        );
        assert_eq!(
            restored.network_view.comment_badge,
            NetworkBadgeVisibility::Large
        );
        assert_eq!(
            restored.network_view.lock_badge,
            NetworkBadgeVisibility::Normal
        );
        assert_eq!(
            restored.network_view.has_data_badge,
            NetworkBadgeVisibility::Normal
        );
        assert_eq!(
            restored.network_view.cached_code_badge,
            NetworkBadgeVisibility::Normal
        );
        assert_eq!(
            restored.network_view.constraint_badge,
            NetworkBadgeVisibility::Normal
        );
        assert_eq!(
            restored.network_view.compilable_badge,
            NetworkBadgeVisibility::Normal
        );
    }

    #[test]
    fn network_box_membership_settles_after_node_drag() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "filter.main")
            .expect("sample filter node should exist");
        let box_index = graph
            .annotations
            .iter()
            .position(|annotation| annotation.kind == GraphAnnotationKind::NetworkBox)
            .expect("sample network box should exist");

        graph.set_node_layout_position(filter_index, GraphPoint::new(0.93, 0.90));
        assert!(graph.settle_node_drag_for_network_boxes(filter_index, false));
        assert!(
            graph.annotations[box_index]
                .member_node_ids
                .contains(&"filter.main".to_owned())
        );
        assert!(graph.annotations[box_index].position.x <= 0.85);
        assert!(
            graph.annotations[box_index].position.x + graph.annotations[box_index].size.x >= 1.0
        );

        graph.set_node_layout_position(filter_index, GraphPoint::new(0.0, 0.0));
        assert!(graph.settle_node_drag_for_network_boxes(filter_index, true));
        assert!(
            !graph.annotations[box_index]
                .member_node_ids
                .contains(&"filter.main".to_owned())
        );
    }

    #[test]
    fn dragging_network_box_translates_member_nodes() {
        let mut graph = GraphDocument::sample();
        let box_index = graph
            .annotations
            .iter()
            .position(|annotation| annotation.kind == GraphAnnotationKind::NetworkBox)
            .expect("sample network box should exist");
        let source_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "source.main")
            .expect("source node should exist");
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "filter.main")
            .expect("filter node should exist");
        let style_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "style.main")
            .expect("style node should exist");

        let original_box_position = graph.annotations[box_index].position;
        let original_source_position = graph.nodes[source_index].layout_position;
        let original_filter_position = graph.nodes[filter_index].layout_position;
        let original_style_position = graph.nodes[style_index].layout_position;
        let delta = GraphPoint::new(-0.45, 1.2);

        assert!(graph.translate_annotation(box_index, delta));

        assert_eq!(
            graph.annotations[box_index].position,
            GraphPoint::new(
                original_box_position.x + delta.x,
                original_box_position.y + delta.y
            )
        );
        assert_eq!(
            graph.nodes[source_index].layout_position,
            GraphPoint::new(
                original_source_position.x + delta.x,
                original_source_position.y + delta.y
            )
        );
        assert_eq!(
            graph.nodes[filter_index].layout_position,
            GraphPoint::new(
                original_filter_position.x + delta.x,
                original_filter_position.y + delta.y
            )
        );
        assert_eq!(
            graph.nodes[style_index].layout_position,
            original_style_position
        );
    }

    #[test]
    fn network_box_resize_to_contents_uses_member_node_bounds() {
        let mut graph = GraphDocument::sample();
        graph.annotations.clear();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "filter.main")
            .expect("sample filter node should exist");
        graph.set_node_layout_position(filter_index, GraphPoint::new(0.50, 0.50));
        let box_index = graph
            .add_network_box_for_node(filter_index)
            .expect("network box should be created for selected node");
        graph.annotations[box_index].position = GraphPoint::new(0.0, 0.0);
        graph.annotations[box_index].size = GraphPoint::new(0.90, 0.90);

        assert!(graph.resize_network_box_to_contents(box_index));

        assert!((graph.annotations[box_index].position.x - 0.42).abs() < 0.0001);
        assert!((graph.annotations[box_index].position.y - 0.36).abs() < 0.0001);
        assert!((graph.annotations[box_index].size.x - 0.16).abs() < 0.0001);
        assert!((graph.annotations[box_index].size.y - 0.28).abs() < 0.0001);
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
        assert!(graph.set_node_name(1, "FILTER_LOW"));
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
        assert_eq!(restored.nodes[1].name, "FILTER_LOW");
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
    fn python_operator_declarations_round_trip_through_sidecar() {
        let mut graph = GraphDocument::sample();
        graph
            .python_operator_declarations
            .push(sample_python_operator_declaration());

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(restored.python_operator_declarations.len(), 1);
        assert_eq!(
            restored.python_operator_declarations[0],
            graph.python_operator_declarations[0]
        );
        assert!(json.contains("python_operator_declarations"));
        assert!(json.contains("vy.blur_curves"));
    }

    #[test]
    fn python_operator_declaration_cache_material_tracks_relevant_fields() {
        let declaration = sample_python_operator_declaration();
        let original_material = declaration.cache_key_material();
        let mut renamed = declaration.clone();
        renamed.display_name = "Blur curves harder".to_owned();
        renamed.help = "Updated operator help text.".to_owned();
        let mut changed_dependency = declaration.clone();
        changed_dependency
            .dependencies
            .requirements
            .push("scipy==1.13.0".to_owned());
        let mut changed_parameter = declaration.clone();
        changed_parameter.parameters[0].default_value = PythonOperatorParameterValue::Float(2.0);

        assert_eq!(original_material, renamed.cache_key_material());
        assert_ne!(original_material, changed_dependency.cache_key_material());
        assert_ne!(original_material, changed_parameter.cache_key_material());
    }

    #[test]
    fn sidecar_without_python_operator_declarations_still_loads() {
        let graph = GraphDocument::sample();
        let mut value =
            serde_json::from_str::<serde_json::Value>(&graph.to_sidecar_json().unwrap())
                .expect("sidecar should be valid json");
        value
            .as_object_mut()
            .expect("sidecar should be an object")
            .remove("python_operator_declarations");
        let json = serde_json::to_string_pretty(&value).unwrap();

        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert!(restored.python_operator_declarations.is_empty());
    }

    #[test]
    fn procedural_asset_declarations_round_trip_through_sidecar() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(restored.procedural_asset_declarations.len(), 1);
        assert_eq!(
            restored.procedural_asset_declarations[0],
            graph.procedural_asset_declarations[0]
        );
        assert!(json.contains("procedural_asset_declarations"));
        assert!(json.contains("vy.asset.curve_cleanup"));
        assert!(
            restored.procedural_asset_declarations[0].outputs[0]
                .data_kind
                .preserves_native_cubic_bezier()
        );
    }

    #[test]
    fn native_operator_declarations_round_trip_through_sidecar() {
        let mut graph = GraphDocument::sample();
        graph
            .native_operator_declarations
            .push(sample_native_operator_declaration());

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(restored.native_operator_declarations.len(), 1);
        assert_eq!(
            restored.native_operator_declarations[0],
            graph.native_operator_declarations[0]
        );
        assert!(json.contains("native_operator_declarations"));
        assert!(json.contains("vy.native.simplify_curves"));
        assert!(
            restored.native_operator_declarations[0].outputs[0]
                .data_kind
                .preserves_native_cubic_bezier()
        );
    }

    #[test]
    fn sidecar_without_asset_or_native_declarations_still_loads() {
        let graph = GraphDocument::sample();
        let mut value =
            serde_json::from_str::<serde_json::Value>(&graph.to_sidecar_json().unwrap())
                .expect("sidecar should be valid json");
        let object = value.as_object_mut().expect("sidecar should be an object");
        object.remove("procedural_asset_declarations");
        object.remove("native_operator_declarations");
        let json = serde_json::to_string_pretty(&value).unwrap();

        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert!(restored.procedural_asset_declarations.is_empty());
        assert!(restored.native_operator_declarations.is_empty());
    }

    #[test]
    fn asset_and_native_geometry_contracts_keep_native_cubic_bezier() {
        let asset = sample_procedural_asset_declaration();
        let native = sample_native_operator_declaration();

        assert!(
            asset
                .outputs
                .iter()
                .any(|port| port.data_kind.preserves_native_cubic_bezier())
        );
        assert!(
            native
                .inputs
                .iter()
                .any(|port| port.data_kind.preserves_native_cubic_bezier())
        );
        assert!(
            native
                .outputs
                .iter()
                .any(|port| port.data_kind.preserves_native_cubic_bezier())
        );
        assert!(asset.wrapped_subgraph.captures_native_cubic_bezier);
    }

    #[test]
    fn procedural_asset_instance_node_reports_asset_inspection_data() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        let node_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");
        graph.mark_node_stale(node_index);
        graph.demand_output_evaluation();

        let info = graph
            .selected_node_info(node_index)
            .expect("asset node info should exist");

        assert_eq!(info.kind, NodeKind::ProceduralAsset);
        assert_eq!(info.role, "Asset");
        assert_eq!(info.status, NodeStatus::Healthy);
        assert_eq!(info.evaluation.state, EvaluationState::Cached);
        let asset = info
            .procedural_asset
            .expect("asset inspector info should exist");
        assert_eq!(asset.asset_id, "vy.asset.curve_cleanup");
        assert_eq!(asset.display_name, "Curve cleanup");
        assert_eq!(asset.instance_version, "0.1.0");
        assert_eq!(asset.current_version.as_deref(), Some("0.1.0"));
        assert_eq!(asset.version_status, OperatorVersionStatus::Current);
        assert_eq!(
            asset.promoted_parameters,
            vec!["minimum_score", "layer_name"]
        );
        assert_eq!(asset.input_bindings[0].port_name, "geometry");
        assert!(info.python_operator.is_none());
        assert!(info.native_operator.is_none());
    }

    #[test]
    fn matched_asset_private_internals_are_not_external_reference_targets() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        graph.add_procedural_asset_node("vy.asset.curve_cleanup");
        let private_graph_id = graph.procedural_asset_declarations[0]
            .wrapped_subgraph
            .graph_id
            .clone();
        let reference_index = add_reference_node_for_target(
            &mut graph,
            ReferenceTargetIdentity {
                graph_id: private_graph_id,
                node_id: "OUT_INTERNAL".to_owned(),
                output_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
            },
        );

        let info = graph
            .selected_node_info(reference_index)
            .expect("reference info should exist");
        let reference_info = info.reference_input.expect("reference input should exist");

        assert_eq!(info.status, NodeStatus::Failed);
        assert_eq!(
            reference_info.status,
            ReferenceDiagnosticStatus::AssetPrivateInternal
        );
        assert!(
            info.warnings[0].contains("private")
                && info.warnings[0].contains("asset boundary output")
        );
    }

    #[test]
    fn procedural_asset_boundary_output_is_referenceable() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        let asset_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");
        let reference_index = graph
            .add_reference_input_node(asset_index)
            .expect("asset boundary output should be referenceable");

        let info = graph
            .selected_node_info(reference_index)
            .expect("reference info should exist");
        let reference_info = info.reference_input.expect("reference input should exist");

        assert_eq!(info.status, NodeStatus::Healthy);
        assert_eq!(reference_info.status, ReferenceDiagnosticStatus::Resolved);
        assert_eq!(
            reference_info.target.node_id,
            graph.nodes[asset_index].node_id
        );
        assert_eq!(
            reference_info.output_kind,
            Some(HoudiniDataKind::GeometryTable)
        );
    }

    #[test]
    fn internal_out_nulls_require_asset_interface_exposure() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        let private_graph_id = graph.procedural_asset_declarations[0]
            .wrapped_subgraph
            .graph_id
            .clone();
        let reference_index = add_reference_node_for_target(
            &mut graph,
            ReferenceTargetIdentity {
                graph_id: private_graph_id,
                node_id: "OUT_MAIN".to_owned(),
                output_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
            },
        );

        let info = graph
            .selected_node_info(reference_index)
            .expect("reference info should exist");

        assert_eq!(info.status, NodeStatus::Failed);
        assert!(
            info.reference_input
                .expect("reference input should exist")
                .targets
                .iter()
                .any(|target| target.status == ReferenceDiagnosticStatus::AssetPrivateInternal)
        );
    }

    #[test]
    fn unlocked_asset_instance_allows_local_internal_references() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        let asset_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");
        assert!(graph.set_procedural_asset_contents_unlocked(asset_index, true));
        let local_graph_id = graph
            .unlocked_asset_graph_id_for_node(asset_index)
            .expect("unlocked asset should expose local graph id");
        let reference_index = add_reference_node_for_target(
            &mut graph,
            ReferenceTargetIdentity {
                graph_id: local_graph_id.clone(),
                node_id: "OUT_INTERNAL".to_owned(),
                output_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
            },
        );

        let asset_info = graph
            .selected_node_info(asset_index)
            .expect("asset info should exist")
            .procedural_asset
            .expect("asset node info should exist");
        let reference_info = graph
            .selected_node_info(reference_index)
            .expect("reference info should exist");

        assert!(asset_info.contents_unlocked);
        assert_eq!(
            asset_info.local_graph_id.as_deref(),
            Some(local_graph_id.as_str())
        );
        assert_eq!(reference_info.status, NodeStatus::Healthy);
        assert_eq!(
            reference_info
                .reference_input
                .expect("reference input should exist")
                .status,
            ReferenceDiagnosticStatus::Resolved
        );
    }

    #[test]
    fn procedural_asset_missing_declaration_reports_failed_version_status() {
        let mut graph = GraphDocument::sample();
        let node_index = graph.add_procedural_asset_node("vy.asset.missing");

        let info = graph
            .selected_node_info(node_index)
            .expect("asset node info should exist");

        assert_eq!(info.status, NodeStatus::Failed);
        assert_eq!(
            info.procedural_asset
                .expect("asset info should exist")
                .version_status,
            OperatorVersionStatus::MissingDeclaration
        );
    }

    #[test]
    fn procedural_asset_version_drift_marks_instance_stale() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        let node_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");
        graph.procedural_asset_declarations[0].version = "0.2.0".to_owned();

        graph.refresh_asset_version_statuses();

        let info = graph
            .selected_node_info(node_index)
            .expect("asset node info should exist");
        let asset = info
            .procedural_asset
            .expect("asset inspector info should exist");
        assert_eq!(asset.instance_version, "0.1.0");
        assert_eq!(asset.current_version.as_deref(), Some("0.2.0"));
        assert_eq!(asset.version_status, OperatorVersionStatus::NewerAvailable);
        assert_eq!(info.status, NodeStatus::Warning);
        assert_eq!(info.evaluation.state, EvaluationState::Stale);
        assert_eq!(
            info.evaluation.message.as_deref(),
            Some("Asset declaration version changed after this instance was created.")
        );
    }

    #[test]
    fn create_asset_draft_from_graph_commits_project_local_declaration() {
        let mut graph = GraphDocument::sample();
        let draft = graph.create_asset_draft_from_graph(
            "My Cleanup Asset",
            "Cleans the current graph.",
            "Use inside this project.",
        );

        assert_eq!(draft.asset_id, "project.asset.my_cleanup_asset");
        assert_eq!(draft.inputs[0].data_kind, HoudiniDataKind::GeometryTable);
        assert_eq!(draft.outputs[0].data_kind, HoudiniDataKind::GeometryTable);
        assert_eq!(draft.graph_snapshot.node_count, graph.nodes.len());
        assert_eq!(
            draft.graph_snapshot.edge_count,
            graph.graph_layout().edges.len()
        );
        assert!(
            draft
                .promoted_parameters
                .iter()
                .any(|parameter| parameter.name == "minimum_score")
        );

        let asset_id = graph.commit_asset_draft(draft);

        assert_eq!(asset_id, "project.asset.my_cleanup_asset");
        assert_eq!(graph.procedural_asset_declarations.len(), 1);
        let declaration = &graph.procedural_asset_declarations[0];
        assert_eq!(declaration.display_name, "My Cleanup Asset");
        assert_eq!(declaration.description, "Cleans the current graph.");
        assert_eq!(declaration.help, "Use inside this project.");
        assert!(
            declaration
                .source
                .project_path
                .starts_with("assets/project.asset.")
        );
        assert!(
            declaration
                .wrapped_subgraph
                .graph_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.node_count == 4)
        );
        assert!(!graph.to_sidecar_json().unwrap().contains("cached_output"));
    }

    #[test]
    fn native_operator_node_reports_native_inspection_data() {
        let mut graph = GraphDocument::sample();
        graph
            .native_operator_declarations
            .push(sample_native_operator_declaration());
        trust_sample_native_operator(&mut graph);
        let node_index = graph.add_native_operator_node("vy.native.simplify_curves");
        graph.set_node_manual(node_index, true);
        graph.request_node_run(node_index);

        let info = graph
            .selected_node_info(node_index)
            .expect("native node info should exist");

        assert_eq!(info.kind, NodeKind::NativeOperator);
        assert_eq!(info.role, "Native");
        assert_eq!(info.status, NodeStatus::Healthy);
        assert_eq!(info.evaluation.state, EvaluationState::Running);
        let native = info
            .native_operator
            .expect("native inspector info should exist");
        assert_eq!(native.operator_id, "vy.native.simplify_curves");
        assert_eq!(native.display_name, "Simplify curves");
        assert_eq!(native.version_status, OperatorVersionStatus::Current);
        assert_eq!(native.load_status, NativeOperatorLoadStatus::Ready);
        assert_eq!(native.inputs, vec!["geometry (GeometryTable)".to_owned()]);
        assert!(native.capabilities.contains(&"GeometryRead".to_owned()));
        assert!(native.provenance_summary.contains("vycorporation/rerun"));
        assert!(native.failure_modes[0].contains("invalid_geometry"));
        assert!(info.procedural_asset.is_none());
        assert!(info.python_operator.is_none());
    }

    #[test]
    fn native_operator_failure_preserves_last_valid_cache_key() {
        let mut graph = GraphDocument::sample();
        graph
            .native_operator_declarations
            .push(sample_native_operator_declaration());
        let node_index = graph.add_native_operator_node("vy.native.simplify_curves");
        graph.nodes[node_index]
            .native_operator
            .as_mut()
            .expect("native payload should exist")
            .last_valid_cache_key = Some("native-cache:ok".to_owned());

        graph.fail_node_run(node_index, "native operator crashed");

        let native = graph
            .selected_node_info(node_index)
            .expect("native node info should exist")
            .native_operator
            .expect("native info should exist");
        assert_eq!(
            native.last_valid_cache_key.as_deref(),
            Some("native-cache:ok")
        );
        assert_eq!(native.last_failure_summary, None);
    }

    #[test]
    fn native_operator_requires_trust_before_run() {
        let mut graph = GraphDocument::sample();
        graph
            .native_operator_declarations
            .push(sample_native_operator_declaration());
        let node_index = graph.add_native_operator_node("vy.native.simplify_curves");

        graph.request_node_run(node_index);

        let info = graph
            .selected_node_info(node_index)
            .expect("native node info should exist");
        let native = info.native_operator.expect("native info should exist");
        assert_eq!(info.status, NodeStatus::Warning);
        assert_eq!(native.load_status, NativeOperatorLoadStatus::TrustRequired);
        assert_eq!(info.evaluation.state, EvaluationState::Manual);
        assert_eq!(
            info.evaluation.message.as_deref(),
            Some(
                "Project trust or explicit operator enablement is required before loading native code."
            )
        );
    }

    #[test]
    fn native_operator_checks_capability_grants_and_host_compatibility() {
        let mut graph = GraphDocument::sample();
        graph
            .native_operator_declarations
            .push(sample_native_operator_declaration());
        graph.native_operator_trust.project_trusted = true;
        graph.native_operator_trust.granted_capabilities =
            vec![NativeOperatorCapability::GeometryRead];
        let node_index = graph.add_native_operator_node("vy.native.simplify_curves");
        let missing_grant = graph
            .selected_node_info(node_index)
            .expect("native node info should exist")
            .native_operator
            .expect("native info should exist")
            .load_status;
        assert_eq!(
            missing_grant,
            NativeOperatorLoadStatus::MissingCapabilityGrant
        );

        graph.native_operator_trust.granted_capabilities =
            sample_native_operator_declaration().capabilities;
        graph.native_operator_declarations[0].host_compatibility_version = "old-host".to_owned();
        let incompatible = graph
            .selected_node_info(node_index)
            .expect("native node info should exist")
            .native_operator
            .expect("native info should exist")
            .load_status;
        assert_eq!(incompatible, NativeOperatorLoadStatus::HostIncompatible);
    }

    #[test]
    fn native_operator_requires_implementation_digest() {
        let mut graph = GraphDocument::sample();
        let mut declaration = sample_native_operator_declaration();
        declaration.provenance.build_digest = None;
        graph.native_operator_declarations.push(declaration);
        trust_sample_native_operator(&mut graph);
        let node_index = graph.add_native_operator_node("vy.native.simplify_curves");

        let load_status = graph
            .selected_node_info(node_index)
            .expect("native node info should exist")
            .native_operator
            .expect("native info should exist")
            .load_status;

        assert_eq!(
            load_status,
            NativeOperatorLoadStatus::ImplementationDigestMissing
        );
    }

    #[test]
    fn native_operator_cache_key_invalidates_on_explicit_inputs() {
        let mut graph = GraphDocument::sample();
        graph
            .native_operator_declarations
            .push(sample_native_operator_declaration());
        trust_sample_native_operator(&mut graph);
        let node_index = graph.add_native_operator_node("vy.native.simplify_curves");

        let base_key = graph
            .native_operator_cache_key(node_index)
            .expect("cache key should derive from declaration and trust settings");
        assert_eq!(base_key.operator_id, "vy.native.simplify_curves");
        assert_eq!(base_key.declaration_version, "0.1.0");
        assert_eq!(base_key.implementation_digest, "native:simplify-curves:001");
        assert_eq!(
            base_key.host_compatibility_version,
            "re_viewer-houdini-graph-0.1"
        );
        assert_eq!(base_key.input_cache_keys.len(), node_index);

        graph.nodes[node_index].parameter.value = 0.75;
        let changed_parameter = graph
            .native_operator_cache_key(node_index)
            .expect("cache key should update after parameter changes");
        assert_ne!(changed_parameter.key_digest, base_key.key_digest);
        assert_ne!(
            changed_parameter.parameter_digest,
            base_key.parameter_digest
        );

        graph.nodes[node_index].parameter.value = 0.0;
        graph.native_operator_trust.granted_capabilities =
            vec![NativeOperatorCapability::GeometryRead];
        let changed_capabilities = graph
            .native_operator_cache_key(node_index)
            .expect("cache key should update after capability grants change");
        assert_ne!(changed_capabilities.key_digest, base_key.key_digest);
        assert_ne!(
            changed_capabilities.capability_digest,
            base_key.capability_digest
        );

        graph.native_operator_trust.granted_capabilities =
            sample_native_operator_declaration().capabilities;
        graph.native_operator_declarations[0]
            .provenance
            .build_digest = Some("native:simplify-curves:002".to_owned());
        let changed_implementation = graph
            .native_operator_cache_key(node_index)
            .expect("cache key should update after implementation digest changes");
        assert_ne!(changed_implementation.key_digest, base_key.key_digest);
        assert_ne!(
            changed_implementation.implementation_digest,
            base_key.implementation_digest
        );
    }

    #[test]
    fn native_operator_provenance_persists_typed_output_metadata() {
        let mut graph = GraphDocument::sample();
        graph
            .native_operator_declarations
            .push(sample_native_operator_declaration());
        trust_sample_native_operator(&mut graph);
        let node_index = graph.add_native_operator_node("vy.native.simplify_curves");

        let record = graph
            .record_native_operator_output(
                node_index,
                NativeOperatorOutputCounts {
                    geometry_records: 4,
                    attribute_records: 2,
                    layer_records: 1,
                },
            )
            .expect("provenance should be recorded for native operator");

        assert_eq!(record.operator_id, "vy.native.simplify_curves");
        assert_eq!(record.implementation_digest, "native:simplify-curves:001");
        assert_eq!(
            record.host_compatibility_version,
            "re_viewer-houdini-graph-0.1"
        );
        assert_eq!(record.output_counts.geometry_records, 4);

        let info = graph
            .selected_node_info(node_index)
            .expect("native node info should exist")
            .native_operator
            .expect("native info should exist");
        assert!(
            info.cache_key_summary
                .as_deref()
                .is_some_and(|summary| summary.contains("vy.native.simplify_curves"))
        );
        assert!(
            info.output_provenance_summary
                .as_deref()
                .is_some_and(|summary| summary.contains("produced 4 geometry"))
        );

        let json = graph.to_sidecar_json().unwrap();
        assert!(json.contains("cache_key"));
        assert!(json.contains("provenance"));
        assert!(json.contains("native:simplify-curves:001"));
        assert!(!json.contains("cached_output"));

        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();
        let restored_native = restored
            .nodes
            .iter()
            .find(|node| node.kind == NodeKind::NativeOperator)
            .expect("native node should restore")
            .native_operator
            .as_ref()
            .expect("native payload should restore");
        assert_eq!(
            restored_native
                .cache_key
                .as_ref()
                .expect("cache key should restore")
                .implementation_digest,
            "native:simplify-curves:001"
        );
        assert_eq!(
            restored_native
                .provenance
                .as_ref()
                .expect("provenance should restore")
                .output_counts
                .geometry_records,
            4
        );
    }

    #[test]
    fn native_operator_host_change_blocks_run_and_marks_cached_output_stale() {
        let mut graph = GraphDocument::sample();
        graph
            .native_operator_declarations
            .push(sample_native_operator_declaration());
        trust_sample_native_operator(&mut graph);
        let node_index = graph.add_native_operator_node("vy.native.simplify_curves");
        graph
            .record_native_operator_output(
                node_index,
                NativeOperatorOutputCounts {
                    geometry_records: 4,
                    attribute_records: 0,
                    layer_records: 0,
                },
            )
            .expect("native output should be recorded");

        graph.native_operator_declarations[0].host_compatibility_version = "old-host".to_owned();
        graph.refresh_native_operator_cache_statuses();
        graph.request_node_run(node_index);

        let info = graph
            .selected_node_info(node_index)
            .expect("native node info should exist");
        let native = info.native_operator.expect("native info should exist");
        assert_eq!(
            native.load_status,
            NativeOperatorLoadStatus::HostIncompatible
        );
        assert_eq!(info.status, NodeStatus::Failed);
        assert_eq!(info.evaluation.state, EvaluationState::Manual);
        assert_eq!(
            info.evaluation.message.as_deref(),
            Some("Native operator host compatibility version does not match this viewer.")
        );
        assert!(
            native
                .cache_key_summary
                .as_deref()
                .is_some_and(|summary| summary.contains("re_viewer-houdini-graph-0.1"))
        );
    }

    #[test]
    fn native_operator_cache_status_marks_parameter_and_upstream_changes_stale() {
        let mut graph = GraphDocument::sample();
        graph
            .native_operator_declarations
            .push(sample_native_operator_declaration());
        trust_sample_native_operator(&mut graph);
        let node_index = graph.add_native_operator_node("vy.native.simplify_curves");
        graph
            .record_native_operator_output(
                node_index,
                NativeOperatorOutputCounts {
                    geometry_records: 4,
                    attribute_records: 0,
                    layer_records: 0,
                },
            )
            .expect("native output should be recorded");

        graph.nodes[1].parameter.value = 0.25;
        graph.refresh_native_operator_cache_statuses();

        assert_eq!(
            graph.nodes[node_index].evaluation.state,
            EvaluationState::Stale
        );
        assert_eq!(
            graph.nodes[node_index].evaluation.message.as_deref(),
            Some("Native operator cache key changed after the last run.")
        );
    }

    #[test]
    fn asset_and_native_nodes_round_trip_through_sidecar() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        graph
            .native_operator_declarations
            .push(sample_native_operator_declaration());
        let asset_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");
        let native_index = graph.add_native_operator_node("vy.native.simplify_curves");
        graph.set_node_layout_position(asset_index, GraphPoint::new(0.48, 0.41));
        graph.set_node_layout_position(native_index, GraphPoint::new(0.58, 0.43));

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        let asset = restored
            .nodes
            .iter()
            .find(|node| node.kind == NodeKind::ProceduralAsset)
            .expect("asset node should restore");
        let native = restored
            .nodes
            .iter()
            .find(|node| node.kind == NodeKind::NativeOperator)
            .expect("native node should restore");
        assert_eq!(
            asset
                .procedural_asset
                .as_ref()
                .expect("asset payload should restore")
                .instance_id,
            "asset_1"
        );
        assert_eq!(
            native
                .native_operator
                .as_ref()
                .expect("native payload should restore")
                .instance_id,
            "native_operator_1"
        );
        assert_eq!(asset.layout_position, GraphPoint::new(0.48, 0.41));
        assert_eq!(native.layout_position, GraphPoint::new(0.58, 0.43));
    }

    #[test]
    fn python_environment_descriptor_round_trips_through_sidecar() {
        let mut graph = GraphDocument::sample();
        graph.python_environment = sample_python_environment_descriptor();

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(restored.python_environment, graph.python_environment);
        assert!(json.contains("python_environment"));
        assert!(json.contains("uv"));
        assert!(json.contains(".houdini/tools/uv"));
        assert!(json.contains("lock:abc123"));
    }

    #[test]
    fn sidecar_without_python_environment_still_uses_missing_project_default() {
        let graph = GraphDocument::sample();
        let mut value =
            serde_json::from_str::<serde_json::Value>(&graph.to_sidecar_json().unwrap())
                .expect("sidecar should be valid json");
        value
            .as_object_mut()
            .expect("sidecar should be an object")
            .remove("python_environment");
        let json = serde_json::to_string_pretty(&value).unwrap();

        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(
            restored.python_environment.lock_status,
            PythonEnvironmentStatus::Missing
        );
        assert_eq!(
            restored.python_environment.environment_path.as_deref(),
            Some(".houdini/python/envs/project-python")
        );
        assert_eq!(restored.python_environment.resolver.tool, "uv");
        assert_eq!(
            restored
                .python_environment
                .resolver
                .executable_path
                .as_deref(),
            Some(".houdini/tools/uv")
        );
        assert_eq!(
            restored.python_environment.paths.create_environment_path,
            ".houdini/python/envs/project-python"
        );
    }

    #[test]
    fn python_environment_descriptor_does_not_default_to_global_python() {
        let graph = GraphDocument::sample();

        let environment_path = graph
            .python_environment
            .environment_path
            .as_deref()
            .expect("default environment should have an app-managed project path");
        assert!(environment_path.starts_with(".houdini/python/envs/"));
        assert!(!environment_path.starts_with("/usr/bin"));
        assert_eq!(
            graph.python_environment.resolver.executable_path.as_deref(),
            Some(".houdini/tools/uv")
        );
        assert_eq!(
            graph.python_environment.paths.mode,
            PythonEnvironmentPathMode::CreateProjectLocal
        );
        assert_eq!(
            graph.python_environment.paths.create_environment_path,
            ".houdini/python/envs/project-python"
        );
        assert_eq!(
            graph.python_environment.status_summary(),
            "Project Python environment is not configured."
        );
        assert_eq!(
            graph.python_environment.requirements_source,
            PythonRequirementsSource::ProjectLocal
        );
    }

    #[test]
    fn python_environment_paths_can_select_existing_environment() {
        let mut graph = GraphDocument::sample();

        graph.configure_python_uv_executable_path("/opt/uv/bin/uv");
        graph.select_existing_python_environment(".venv");

        assert_eq!(
            graph.python_environment.resolver.executable_path.as_deref(),
            Some("/opt/uv/bin/uv")
        );
        assert_eq!(
            graph.python_environment.paths.mode,
            PythonEnvironmentPathMode::ExistingEnvironment
        );
        assert_eq!(
            graph
                .python_environment
                .paths
                .existing_environment_path
                .as_deref(),
            Some(".venv")
        );
        assert_eq!(
            graph.python_environment.environment_path.as_deref(),
            Some(".venv")
        );
        assert_eq!(
            graph.python_environment.lock_status,
            PythonEnvironmentStatus::Locked
        );
    }

    #[test]
    fn python_environment_paths_can_choose_create_target() {
        let mut graph = GraphDocument::sample();

        graph.select_python_environment_create_path(".houdini/python/envs/experiment");

        assert_eq!(
            graph.python_environment.paths.mode,
            PythonEnvironmentPathMode::CreateProjectLocal
        );
        assert_eq!(
            graph.python_environment.paths.create_environment_path,
            ".houdini/python/envs/experiment"
        );
        assert_eq!(
            graph.python_environment.environment_path.as_deref(),
            Some(".houdini/python/envs/experiment")
        );
        assert_eq!(
            graph.python_environment.lock_status,
            PythonEnvironmentStatus::Unlocked
        );
        assert!(
            !graph
                .python_environment
                .environment_path
                .as_deref()
                .is_some_and(|path| path.starts_with("/usr/bin"))
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

    fn sample_python_operator_declaration() -> PythonOperatorDeclaration {
        PythonOperatorDeclaration {
            operator_id: "vy.blur_curves".to_owned(),
            display_name: "Blur curves".to_owned(),
            version: "0.1.0".to_owned(),
            entry_point: PythonOperatorEntryPoint {
                source: PythonOperatorSource::File {
                    path: "operators/blur_curves.py".to_owned(),
                },
                callable: "run".to_owned(),
            },
            inputs: vec![PythonOperatorPort {
                name: "geometry".to_owned(),
                data_kind: PythonOperatorDataKind::GeometryTable,
                help: "Input polygons and native cubic Beziers.".to_owned(),
            }],
            outputs: vec![PythonOperatorPort {
                name: "geometry".to_owned(),
                data_kind: PythonOperatorDataKind::GeometryTable,
                help: "Output polygons and native cubic Beziers.".to_owned(),
            }],
            parameters: vec![PythonOperatorParameterDeclaration {
                name: "radius".to_owned(),
                kind: PythonOperatorParameterKind::Float,
                default_value: PythonOperatorParameterValue::Float(1.5),
                range: Some(PythonOperatorNumericRange {
                    min: 0.0,
                    max: 10.0,
                }),
                allowed_values: Vec::new(),
                invalidates_cache: true,
                help: "Blur radius in graph units.".to_owned(),
            }],
            dependencies: PythonOperatorDependencies {
                python_version: Some(">=3.11,<3.13".to_owned()),
                requirements: vec!["numpy==2.0.0".to_owned()],
                extras: vec!["cv".to_owned()],
            },
            capabilities: vec![PythonOperatorCapability::FileRead],
            help: "Smooths curve control points without mutating viewer state.".to_owned(),
        }
    }

    fn sample_procedural_asset_declaration() -> ProceduralAssetDeclaration {
        ProceduralAssetDeclaration {
            asset_id: "vy.asset.curve_cleanup".to_owned(),
            display_name: "Curve cleanup".to_owned(),
            version: "0.1.0".to_owned(),
            description: "Reusable graph asset for cleaning polygon and native cubic curve layers."
                .to_owned(),
            labels: vec!["curves".to_owned(), "cleanup".to_owned()],
            help: "Promotes the score threshold and stroke scale from a wrapped Houdini graph."
                .to_owned(),
            source: ProceduralAssetSource {
                project_path: "assets/curve_cleanup.houdini_graph.json".to_owned(),
                author: Some("vy".to_owned()),
                created_at: Some("2026-06-29T00:00:00Z".to_owned()),
                source_digest: Some("asset:curve-cleanup:001".to_owned()),
            },
            inputs: vec![geometry_port("geometry", "Input graph geometry.")],
            outputs: vec![geometry_port(
                "geometry",
                "Output geometry preserving polygons and native cubic Beziers.",
            )],
            promoted_parameters: vec![
                HoudiniParameterDeclaration {
                    name: "minimum_score".to_owned(),
                    kind: HoudiniParameterKind::Float,
                    default_value: HoudiniParameterValue::Float(0.55),
                    range: Some(HoudiniNumericRange { min: 0.0, max: 1.0 }),
                    allowed_values: Vec::new(),
                    help: "Promoted filter threshold.".to_owned(),
                },
                HoudiniParameterDeclaration {
                    name: "layer_name".to_owned(),
                    kind: HoudiniParameterKind::String,
                    default_value: HoudiniParameterValue::String("Clean curves".to_owned()),
                    range: None,
                    allowed_values: Vec::new(),
                    help: "Output layer label.".to_owned(),
                },
            ],
            wrapped_subgraph: ProceduralAssetSubgraphReference {
                graph_id: "graph.curve_cleanup".to_owned(),
                output_node_id: "output.main".to_owned(),
                captures_native_cubic_bezier: true,
                graph_snapshot: Some(ProceduralAssetGraphSnapshot {
                    node_count: 4,
                    edge_count: 3,
                    layer_count: 3,
                    geometry_contract: "HoudiniGeometryRecord polygons and native cubic Beziers"
                        .to_owned(),
                }),
            },
        }
    }

    fn sample_native_operator_declaration() -> NativeOperatorDeclaration {
        NativeOperatorDeclaration {
            operator_id: "vy.native.simplify_curves".to_owned(),
            display_name: "Simplify curves".to_owned(),
            version: "0.1.0".to_owned(),
            host_compatibility_version: "re_viewer-houdini-graph-0.1".to_owned(),
            implementation: NativeOperatorImplementation::DynamicLibrary {
                path: "plugins/native/libvy_simplify_curves.dylib".to_owned(),
                symbol: "vy_houdini_operator_entry".to_owned(),
            },
            inputs: vec![geometry_port(
                "geometry",
                "Input geometry table with polygons and native cubic Beziers.",
            )],
            outputs: vec![geometry_port(
                "geometry",
                "Simplified geometry table preserving native cubic Bezier records.",
            )],
            parameters: vec![HoudiniParameterDeclaration {
                name: "tolerance".to_owned(),
                kind: HoudiniParameterKind::Float,
                default_value: HoudiniParameterValue::Float(0.1),
                range: Some(HoudiniNumericRange {
                    min: 0.0,
                    max: 10.0,
                }),
                allowed_values: Vec::new(),
                help: "Simplification tolerance in graph units.".to_owned(),
            }],
            capabilities: vec![
                NativeOperatorCapability::GeometryRead,
                NativeOperatorCapability::GeometryWrite,
            ],
            provenance: NativeOperatorProvenance {
                source_repository: Some("vycorporation/rerun".to_owned()),
                source_revision: Some("native-plugin-spike".to_owned()),
                build_digest: Some("native:simplify-curves:001".to_owned()),
                vendor: Some("vy".to_owned()),
            },
            failure_modes: vec![NativeOperatorFailureMode {
                code: "invalid_geometry".to_owned(),
                summary: "Input geometry table did not match the Houdini geometry schema."
                    .to_owned(),
                recoverable: true,
            }],
            documentation:
                "Trusted native operator declaration only; loading happens in a later issue."
                    .to_owned(),
        }
    }

    fn trust_sample_native_operator(graph: &mut GraphDocument) {
        graph.native_operator_trust.project_trusted = true;
        graph.native_operator_trust.enabled_operator_ids =
            vec!["vy.native.simplify_curves".to_owned()];
        graph.native_operator_trust.granted_capabilities =
            sample_native_operator_declaration().capabilities;
    }

    fn geometry_port(name: &str, help: &str) -> HoudiniOperatorPort {
        HoudiniOperatorPort {
            name: name.to_owned(),
            data_kind: HoudiniDataKind::GeometryTable,
            required: true,
            help: help.to_owned(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn configured_python_interpreter() -> String {
        if let Ok(path) = std::env::var("PYTHON3")
            && std::path::Path::new(&path).is_absolute()
        {
            return path;
        }

        std::process::Command::new("python3")
            .arg("-c")
            .arg("import sys; print(sys.executable)")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|path| path.trim().to_owned())
            .filter(|path| std::path::Path::new(path).is_absolute())
            .unwrap_or_else(|| "/usr/bin/python3".to_owned())
    }

    fn sample_python_environment_descriptor() -> PythonEnvironmentDescriptor {
        PythonEnvironmentDescriptor {
            environment_id: "project-python".to_owned(),
            python_version_requirement: ">=3.11,<3.13".to_owned(),
            requirements_source: PythonRequirementsSource::GeneratedFromOperators,
            project_requirements: PythonProjectRequirements {
                requirements: vec!["pyarrow==16.1.0".to_owned()],
            },
            lock_status: PythonEnvironmentStatus::Ready,
            lock_digest: Some("lock:abc123".to_owned()),
            environment_path: Some(".houdini/python/envs/project-python".to_owned()),
            resolver: PythonEnvironmentResolver {
                tool: "uv".to_owned(),
                version: Some("0.7.0".to_owned()),
                executable_path: Some(".houdini/tools/uv".to_owned()),
            },
            paths: PythonEnvironmentPaths::default(),
            last_health_check: Some("2026-06-28T23:30:00Z".to_owned()),
            last_failure_summary: None,
            dependency_health: PythonDependencyHealth {
                package_count: 3,
                missing_packages: Vec::new(),
                conflicts: Vec::new(),
                failed_imports: Vec::new(),
            },
            resolve_state: PythonEnvironmentResolveState::default(),
        }
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
