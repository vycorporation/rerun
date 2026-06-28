#[derive(Clone, Copy, Debug, PartialEq)]
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
    pub nodes: Vec<GraphNode>,
    pub layers: Vec<Layer>,
    pub geometry: Vec<Geometry>,
}

impl GraphDocument {
    pub fn sample() -> Self {
        Self {
            nodes: vec![
                GraphNode {
                    name: "Source",
                    kind: NodeKind::Source,
                    layout_position: GraphPoint::new(0.0, 0.5),
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
                    parameter: NodeParameter::scalar(
                        "Minimum score",
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
                    name: "Polygons",
                    kind: LayerKind::Polygons,
                    visible: true,
                },
                Layer {
                    name: "Curves",
                    kind: LayerKind::Curves,
                    visible: true,
                },
                Layer {
                    name: "Debug Output",
                    kind: LayerKind::Debug,
                    visible: false,
                },
            ],
            geometry: vec![
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
            ],
        }
    }

    pub fn polygon_count(&self) -> usize {
        self.geometry
            .iter()
            .filter(|geometry| matches!(geometry, Geometry::Polygon(_)))
            .count()
    }

    pub fn cubic_bezier_count(&self) -> usize {
        self.geometry
            .iter()
            .filter(|geometry| matches!(geometry, Geometry::CubicBezier(_)))
            .count()
    }

    pub fn polygon_vertex_count(&self) -> usize {
        self.geometry
            .iter()
            .map(|geometry| match geometry {
                Geometry::Polygon(polygon) => polygon.points.len(),
                Geometry::CubicBezier(_) => 0,
            })
            .sum()
    }

    pub fn cubic_control_point_count(&self) -> usize {
        self.geometry
            .iter()
            .map(|geometry| match geometry {
                Geometry::Polygon(_) => 0,
                Geometry::CubicBezier(curve) => curve.control_points().len(),
            })
            .sum()
    }

    pub fn filter_minimum_score(&self) -> f32 {
        self.nodes
            .iter()
            .find(|node| node.kind == NodeKind::Filter)
            .map_or(0.0, |node| node.parameter.value)
    }

    pub fn style_scale(&self) -> f32 {
        self.nodes
            .iter()
            .find(|node| node.kind == NodeKind::Style)
            .map_or(0.5, |node| node.parameter.value)
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
            .find(|layer| layer.kind == kind)
            .is_some_and(|layer| layer.visible)
    }

    pub fn emits(&self, geometry: &Geometry) -> bool {
        let layer_visible = match geometry {
            Geometry::Polygon(_) => self.layer_visible(LayerKind::Polygons),
            Geometry::CubicBezier(_) => self.layer_visible(LayerKind::Curves),
        };

        layer_visible && geometry.score() >= self.filter_minimum_score()
    }

    pub fn visible_output_count(&self) -> usize {
        self.viewer_output().items.len()
    }

    pub fn pipeline_stages(&self) -> Vec<PipelineStage> {
        let source_count = self.geometry.len();
        let filtered_count = self
            .geometry
            .iter()
            .filter(|geometry| geometry.score() >= self.filter_minimum_score())
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

        let edges = (0..self.nodes.len().saturating_sub(1))
            .map(|index| GraphEdge {
                from_node: index,
                to_node: index + 1,
            })
            .collect();

        GraphLayout { nodes, edges }
    }

    pub fn set_node_layout_position(&mut self, index: usize, position: GraphPoint) {
        if let Some(node) = self.nodes.get_mut(index) {
            node.layout_position = position.clamped_to_unit();
        }
    }

    pub fn selected_node_info(&self, index: usize) -> Option<NodeInfo> {
        let node = self.nodes.get(index)?;
        let stages = self.pipeline_stages();

        Some(match node.kind {
            NodeKind::Source => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                input_count: stages[0].input_count,
                output_count: stages[0].output_count,
                parameter: node.parameter.clone(),
                summary: "Source geometry lives in the graph model before any viewer adaptation.",
            },
            NodeKind::Filter => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                input_count: stages[1].input_count,
                output_count: stages[1].output_count,
                parameter: node.parameter.clone(),
                summary: "Filter removes geometry below the minimum sample score.",
            },
            NodeKind::Style => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                input_count: stages[2].input_count,
                output_count: stages[2].output_count,
                parameter: node.parameter.clone(),
                summary: "Style changes viewer presentation without mutating graph geometry.",
            },
            NodeKind::Output => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                input_count: stages[3].input_count,
                output_count: stages[3].output_count,
                parameter: node.parameter.clone(),
                summary: "Output prepares boundary data while preserving native graph geometry.",
            },
        })
    }

    pub fn viewer_output(&self) -> ViewerOutput {
        ViewerOutput {
            stroke_scale: self.style_scale(),
            items: self
                .geometry
                .iter()
                .filter(|geometry| self.emits(geometry))
                .map(|geometry| match geometry {
                    Geometry::Polygon(polygon) => ViewerGeometry::Polygon(polygon.clone()),
                    Geometry::CubicBezier(curve) => ViewerGeometry::CubicBezier(*curve),
                })
                .collect(),
        }
    }

    pub fn adaptive_export_output(&self) -> ExportOutput {
        let curve_segments = self.export_segments().max(1);
        ExportOutput {
            items: self
                .geometry
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
}

pub(crate) struct GraphNode {
    pub name: &'static str,
    pub kind: NodeKind,
    pub layout_position: GraphPoint,
    pub parameter: NodeParameter,
    pub info: &'static str,
}

#[derive(Clone)]
pub(crate) struct NodeParameter {
    pub name: &'static str,
    pub kind: NodeParameterKind,
    pub value: f32,
    pub range: std::ops::RangeInclusive<f32>,
    pub help: &'static str,
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
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NodeParameterKind {
    Scalar,
}

impl NodeParameterKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scalar => "Scalar",
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub parameter: NodeParameter,
    pub summary: &'static str,
}

pub(crate) struct PipelineStage {
    pub name: &'static str,
    pub input_count: usize,
    pub output_count: usize,
    pub note: &'static str,
}

pub(crate) struct Layer {
    pub name: &'static str,
    pub kind: LayerKind,
    pub visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LayerKind {
    Polygons,
    Curves,
    Debug,
}

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
}

#[derive(Clone)]
pub(crate) struct Polygon {
    pub points: Vec<GraphPoint>,
    pub score: f32,
}

#[derive(Clone, Copy)]
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
    pub items: Vec<ViewerGeometry>,
    pub stroke_scale: f32,
}

pub(crate) enum ViewerGeometry {
    Polygon(Polygon),
    CubicBezier(CubicBezier),
}

pub(crate) struct ExportOutput {
    pub items: Vec<ExportGeometry>,
}

pub(crate) enum ExportGeometry {
    Polygon(Vec<GraphPoint>),
    Polyline(Vec<GraphPoint>),
}

#[cfg(test)]
mod tests {
    use super::{
        ExportGeometry, Geometry, GraphDocument, GraphPoint, LayerKind, NodeParameterKind,
        ViewerGeometry,
    };

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
                .any(|geometry| matches!(geometry, ViewerGeometry::CubicBezier(_)))
        );
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
    fn selected_node_info_reports_pipeline_counts() {
        let graph = GraphDocument::sample();

        let source = graph
            .selected_node_info(0)
            .expect("sample graph should include source node");
        assert_eq!(source.input_count, 0);
        assert_eq!(source.output_count, 4);

        let filter = graph
            .selected_node_info(1)
            .expect("sample graph should include filter node");
        assert_eq!(filter.input_count, 4);
        assert_eq!(filter.output_count, 2);
        assert_eq!(filter.role, "Cull");
        assert_eq!(filter.parameter.name, "Minimum score");
        assert_eq!(filter.parameter.kind, NodeParameterKind::Scalar);
        assert_eq!(filter.parameter.value, 0.55);

        let output = graph
            .selected_node_info(3)
            .expect("sample graph should include output node");
        assert_eq!(output.output_count, 2);
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
}
