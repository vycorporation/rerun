use std::path::{Path, PathBuf};
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
    pub graph_registry: ProjectGraphRegistry,
    pub graph_containers: Vec<GraphContainerMetadata>,
    pub data_flow_edges: Vec<GraphDataFlowEdge>,
    pub nodes: Vec<GraphNode>,
    pub annotations: Vec<GraphAnnotation>,
    pub network_view: NetworkViewDisplayOptions,
    pub layers: Vec<Layer>,
    pub style: GraphStyle,
    pub substrate_raster: Option<SubstrateRaster>,
    pub geometry: Vec<Geometry>,
    pub recording_geometry: Vec<Geometry>,
    pub python_operator_declarations: Vec<PythonOperatorDeclaration>,
    pub procedural_asset_declarations: Vec<ProceduralAssetDeclaration>,
    pub native_operator_declarations: Vec<NativeOperatorDeclaration>,
    pub native_operator_trust: NativeOperatorTrustPolicy,
    pub python_environment: PythonEnvironmentDescriptor,
    pub evaluation_mode: GraphEvaluationMode,
    pub command_history: ProjectCommandHistory,
    pub work_items: Vec<GraphWorkItem>,
}

const GENERATED_NODE_LANE_Y: f32 = 0.82;
const NATIVE_OPERATOR_HOST_COMPATIBILITY_VERSION: &str = "re_viewer-houdini-graph-0.1";
const MAIN_GRAPH_ID: &str = "main";
pub(crate) const PRIMARY_GEOMETRY_OUTPUT: &str = "geometry";

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct ProjectGraphRegistry {
    pub selected_graph_id: String,
    pub graphs: Vec<ProjectGraphMetadata>,
}

impl ProjectGraphRegistry {
    fn main_graph() -> ProjectGraphMetadata {
        ProjectGraphMetadata {
            graph_id: MAIN_GRAPH_ID.to_owned(),
            name: "Main".to_owned(),
            path: "/obj/main".to_owned(),
            role: ProjectGraphRole::Main,
        }
    }

    fn normalize(mut self) -> Self {
        if self.graphs.is_empty() {
            self.graphs.push(Self::main_graph());
        }
        if !self
            .graphs
            .iter()
            .any(|graph| graph.graph_id == MAIN_GRAPH_ID)
        {
            self.graphs.insert(0, Self::main_graph());
        }
        if self.selected_graph_id.is_empty()
            || !self
                .graphs
                .iter()
                .any(|graph| graph.graph_id == self.selected_graph_id)
        {
            self.selected_graph_id = MAIN_GRAPH_ID.to_owned();
        }
        self
    }

    pub fn selected_graph(&self) -> Option<&ProjectGraphMetadata> {
        self.graphs
            .iter()
            .find(|graph| graph.graph_id == self.selected_graph_id)
    }

    pub fn graph(&self, graph_id: &str) -> Option<&ProjectGraphMetadata> {
        self.graphs.iter().find(|graph| graph.graph_id == graph_id)
    }
}

impl Default for ProjectGraphRegistry {
    fn default() -> Self {
        Self {
            selected_graph_id: MAIN_GRAPH_ID.to_owned(),
            graphs: vec![Self::main_graph()],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct ProjectGraphMetadata {
    pub graph_id: String,
    pub name: String,
    pub path: String,
    pub role: ProjectGraphRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum ProjectGraphRole {
    Main,
    Subgraph,
    AssetInternal,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GraphContainerMetadata {
    pub container_node_id: String,
    pub internal_graph_id: String,
    pub kind: GraphContainerKind,
    #[serde(default = "GraphBoundaryDeclaration::geometry_passthrough")]
    pub boundary: GraphBoundaryDeclaration,
    #[serde(default)]
    pub collapse_manifest: Option<GraphContainerCollapseManifest>,
    pub navigable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GraphContainerCollapseManifest {
    pub source_graph_id: String,
    pub captured_node_ids: Vec<String>,
    #[serde(default)]
    pub external_edges: Vec<GraphContainerExternalEdge>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GraphContainerExternalEdge {
    pub direction: GraphBoundaryMappingDirection,
    pub edge_id: String,
    pub external_node_id: String,
    pub external_port_name: String,
    pub internal_node_id: String,
    pub internal_port_name: String,
    pub public_port_name: String,
    pub data_kind: HoudiniDataKind,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GraphBoundaryDeclaration {
    pub inputs: Vec<HoudiniOperatorPort>,
    pub outputs: Vec<HoudiniOperatorPort>,
    #[serde(default)]
    pub mappings: Vec<GraphBoundaryMapping>,
}

impl GraphBoundaryDeclaration {
    fn geometry_passthrough() -> Self {
        Self {
            inputs: vec![HoudiniOperatorPort::geometry(
                PRIMARY_GEOMETRY_OUTPUT,
                "Geometry table entering the graph container boundary.",
            )],
            outputs: vec![HoudiniOperatorPort::geometry(
                PRIMARY_GEOMETRY_OUTPUT,
                "Geometry table exposed by the graph container boundary.",
            )],
            mappings: Vec::new(),
        }
    }

    fn output_kind(&self, output_name: &str) -> Option<HoudiniDataKind> {
        self.outputs
            .iter()
            .find(|port| port.name == output_name)
            .map(|port| port.data_kind)
    }

    fn primary_output(&self) -> Option<&HoudiniOperatorPort> {
        self.outputs.first()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GraphBoundaryMapping {
    pub direction: GraphBoundaryMappingDirection,
    pub public_port_name: String,
    pub internal_node_id: String,
    pub internal_port_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum GraphBoundaryMappingDirection {
    Input,
    Output,
}

impl GraphBoundaryMappingDirection {
    fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum GraphContainerKind {
    Subnet,
}

impl GraphContainerKind {
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Subnet => "Subnet",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GraphDataFlowEdge {
    pub edge_id: String,
    pub from_node_id: String,
    pub from_output: String,
    pub to_node_id: String,
    pub to_input: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GraphDataFlowEdgeDiagnostic {
    pub edge_id: String,
    pub status: GraphDataFlowEdgeDiagnosticStatus,
    pub readable_path: String,
    pub message: String,
}

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct ReconnectNodeDeleteResult {
    pub deleted_node: GraphNode,
    pub added_edges: Vec<GraphDataFlowEdge>,
    pub skipped_diagnostics: Vec<GraphDataFlowEdgeDiagnostic>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct InsertNodeOnConnectionResult {
    pub removed_edge: GraphDataFlowEdge,
    pub added_edges: Vec<GraphDataFlowEdge>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GraphDataFlowEdgeDiagnosticStatus {
    MissingSourceNode,
    MissingTargetNode,
    MissingSourcePort,
    MissingTargetPort,
    IncompatibleDataKind,
    DuplicateConnection,
    Cycle,
}

impl GraphDataFlowEdgeDiagnosticStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingSourceNode => "missing source node",
            Self::MissingTargetNode => "missing target node",
            Self::MissingSourcePort => "missing source port",
            Self::MissingTargetPort => "missing target port",
            Self::IncompatibleDataKind => "incompatible data kind",
            Self::DuplicateConnection => "duplicate connection",
            Self::Cycle => "cyclic data flow",
        }
    }
}

impl GraphDocument {
    pub fn current_graph_id(&self) -> &str {
        self.graph_registry
            .selected_graph()
            .map(|graph| graph.graph_id.as_str())
            .unwrap_or(MAIN_GRAPH_ID)
    }

    pub fn current_graph_path(&self) -> &str {
        self.graph_registry
            .selected_graph()
            .map(|graph| graph.path.as_str())
            .unwrap_or("/obj/main")
    }

    #[allow(dead_code)]
    pub fn graph_navigation_targets(&self) -> Vec<GraphNavigationTarget> {
        self.graph_registry
            .graphs
            .iter()
            .map(GraphNavigationTarget::from_metadata)
            .collect()
    }

    #[allow(dead_code)]
    pub fn select_graph_by_id(
        &mut self,
        graph_id: &str,
    ) -> Result<GraphNavigationChange, GraphNavigationError> {
        let target_graph_id = graph_id.trim();
        let Some(selected_graph) = self.graph_registry.graph(target_graph_id).cloned() else {
            return Err(GraphNavigationError::MissingGraph {
                graph_id: graph_id.to_owned(),
            });
        };
        let previous_graph = self
            .graph_registry
            .selected_graph()
            .cloned()
            .unwrap_or_else(ProjectGraphRegistry::main_graph);
        let changed = previous_graph.graph_id != selected_graph.graph_id;

        if changed {
            self.graph_registry.selected_graph_id = selected_graph.graph_id.clone();
        }

        Ok(GraphNavigationChange {
            previous_graph: GraphNavigationTarget::from_metadata(&previous_graph),
            selected_graph: GraphNavigationTarget::from_metadata(&selected_graph),
            changed,
        })
    }

    #[allow(dead_code)]
    pub fn enter_graph_container_node(
        &mut self,
        node_index: usize,
    ) -> Result<GraphNavigationChange, GraphNavigationError> {
        let Some(node) = self.nodes.get(node_index) else {
            return Err(GraphNavigationError::MissingNodeIndex(node_index));
        };
        if node.kind != NodeKind::GraphContainer {
            return Err(GraphNavigationError::NodeIsNotGraphContainer {
                node_id: node.node_id.clone(),
                node_name: node.name.clone(),
            });
        }
        let Some(container) = self.graph_container_metadata_for_node(&node.node_id) else {
            return Err(GraphNavigationError::MissingContainerMetadata {
                node_id: node.node_id.clone(),
            });
        };
        if self
            .graph_registry
            .graph(&container.internal_graph_id)
            .is_none()
        {
            return Err(GraphNavigationError::MissingInternalGraph {
                graph_id: container.internal_graph_id.clone(),
            });
        }
        if !container.navigable {
            return Err(GraphNavigationError::ContainerNotNavigable {
                node_id: node.node_id.clone(),
                internal_graph_id: container.internal_graph_id.clone(),
            });
        }

        let internal_graph_id = container.internal_graph_id.clone();
        self.select_graph_by_id(&internal_graph_id)
    }

    pub fn exit_current_graph_to_parent_container(
        &mut self,
    ) -> Option<GraphParentNavigationChange> {
        let current_graph_id = self.current_graph_id().to_owned();
        if current_graph_id == MAIN_GRAPH_ID {
            return None;
        }
        let container_node_id = self
            .graph_containers
            .iter()
            .find(|container| container.internal_graph_id == current_graph_id)?
            .container_node_id
            .clone();
        let container_node_index = self
            .nodes
            .iter()
            .position(|node| node.node_id == container_node_id)?;
        let parent_graph_id = self
            .nodes
            .get(container_node_index)
            .map(|node| self.node_parent_graph_id(node).to_owned())?;
        let navigation = self.select_graph_by_id(&parent_graph_id).ok()?;

        Some(GraphParentNavigationChange {
            navigation,
            container_node_index,
        })
    }

    pub fn current_graph_parent_container_node_index(&self) -> Option<usize> {
        let current_graph_id = self.current_graph_id();
        if current_graph_id == MAIN_GRAPH_ID {
            return None;
        }
        let container_node_id = self
            .graph_containers
            .iter()
            .find(|container| container.internal_graph_id == current_graph_id)?
            .container_node_id
            .as_str();
        self.nodes
            .iter()
            .position(|node| node.node_id == container_node_id)
    }

    #[allow(dead_code)]
    pub fn graph_local_node_indices(&self, graph_id: &str) -> Vec<usize> {
        let graph_id = if graph_id.is_empty() {
            MAIN_GRAPH_ID
        } else {
            graph_id
        };
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                (self.node_parent_graph_id(node) == graph_id).then_some(index)
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn current_graph_node_indices(&self) -> Vec<usize> {
        self.graph_local_node_indices(self.current_graph_id())
    }

    pub fn graph_local_annotation_indices(&self, graph_id: &str) -> Vec<usize> {
        let graph_id = if graph_id.is_empty() {
            MAIN_GRAPH_ID
        } else {
            graph_id
        };
        self.annotations
            .iter()
            .enumerate()
            .filter_map(|(index, annotation)| {
                (self.annotation_parent_graph_id(annotation) == graph_id).then_some(index)
            })
            .collect()
    }

    pub fn current_graph_annotation_indices(&self) -> Vec<usize> {
        self.graph_local_annotation_indices(self.current_graph_id())
    }

    pub fn annotation_belongs_to_current_graph(&self, annotation_index: usize) -> bool {
        self.annotations
            .get(annotation_index)
            .is_some_and(|annotation| {
                self.annotation_parent_graph_id(annotation) == self.current_graph_id()
            })
    }

    pub fn procedural_asset_gallery_entries(&self) -> Vec<ProceduralAssetGalleryEntry> {
        let mut entries = self
            .procedural_asset_declarations
            .iter()
            .map(|declaration| ProceduralAssetGalleryEntry {
                asset_id: declaration.asset_id.clone(),
                display_name: declaration.display_name.clone(),
                version: Some(declaration.version.clone()),
                description: declaration.description.clone(),
                labels: declaration.labels.clone(),
                input_count: declaration.inputs.len(),
                output_count: declaration.outputs.len(),
                promoted_parameter_count: declaration.promoted_parameters.len(),
                wrapped_graph_id: Some(declaration.wrapped_subgraph.graph_id.clone()),
                missing_declaration: false,
                usages: Vec::new(),
            })
            .collect::<Vec<_>>();

        for (node_index, node) in self.nodes.iter().enumerate() {
            let Some(asset_node) = node.procedural_asset.as_ref() else {
                continue;
            };
            let graph_location = self.graph_location_for_node(node);
            let version_status = self.procedural_asset_version_status_for_instance(asset_node);
            let declaration = self
                .procedural_asset_declarations
                .iter()
                .find(|declaration| declaration.asset_id == asset_node.asset_id);
            let usage = ProceduralAssetUsageInfo {
                node_index,
                node_id: node.node_id.clone(),
                node_name: node.name.clone(),
                graph_id: graph_location.graph_id,
                graph_path: graph_location.graph_path,
                node_path: graph_location.node_path,
                instance_version: asset_node.instance_version.clone(),
                contents_unlocked: asset_node.contents_unlocked,
                can_match_definition: asset_node.contents_unlocked,
                can_upgrade_to_current_definition: declaration
                    .is_some_and(|declaration| declaration.version != asset_node.instance_version),
                version_status,
            };

            if let Some(entry) = entries
                .iter_mut()
                .find(|entry| entry.asset_id == asset_node.asset_id)
            {
                entry.usages.push(usage);
            } else {
                entries.push(ProceduralAssetGalleryEntry {
                    asset_id: asset_node.asset_id.clone(),
                    display_name: asset_node.asset_id.clone(),
                    version: None,
                    description: "Asset definition is missing from this project.".to_owned(),
                    labels: Vec::new(),
                    input_count: 0,
                    output_count: 0,
                    promoted_parameter_count: 0,
                    wrapped_graph_id: None,
                    missing_declaration: true,
                    usages: vec![usage],
                });
            }
        }

        entries.sort_by(|left, right| {
            left.display_name
                .to_lowercase()
                .cmp(&right.display_name.to_lowercase())
                .then_with(|| left.asset_id.cmp(&right.asset_id))
        });
        for entry in &mut entries {
            entry.usages.sort_by(|left, right| {
                left.node_path
                    .to_lowercase()
                    .cmp(&right.node_path.to_lowercase())
                    .then_with(|| left.node_id.cmp(&right.node_id))
            });
        }
        entries
    }

    fn graph_container_metadata_for_node(&self, node_id: &str) -> Option<&GraphContainerMetadata> {
        self.graph_containers
            .iter()
            .find(|container| container.container_node_id == node_id)
    }

    fn graph_container_info_for_node(&self, node: &GraphNode) -> GraphContainerNodeInfo {
        let Some(container) = self.graph_container_metadata_for_node(&node.node_id) else {
            return GraphContainerNodeInfo {
                kind: GraphContainerKind::Subnet,
                internal_graph_id: String::new(),
                internal_graph_name: None,
                internal_graph_path: None,
                inputs: Vec::new(),
                outputs: Vec::new(),
                mappings: Vec::new(),
                collapse_manifest: None,
                navigable: false,
                status: GraphContainerStatus::MissingContainerMetadata,
            };
        };
        let graph = self.graph_registry.graph(&container.internal_graph_id);
        let internal_graph_exists = graph.is_some();
        GraphContainerNodeInfo {
            kind: container.kind,
            internal_graph_id: container.internal_graph_id.clone(),
            internal_graph_name: graph.map(|graph| graph.name.clone()),
            internal_graph_path: graph.map(|graph| graph.path.clone()),
            inputs: container.boundary.inputs.clone(),
            outputs: container.boundary.outputs.clone(),
            mappings: self.graph_boundary_mapping_info(container, internal_graph_exists),
            collapse_manifest: container.collapse_manifest.clone(),
            navigable: container.navigable && internal_graph_exists,
            status: if internal_graph_exists {
                GraphContainerStatus::Resolved
            } else {
                GraphContainerStatus::MissingInternalGraph
            },
        }
    }

    fn graph_boundary_mapping_info(
        &self,
        container: &GraphContainerMetadata,
        internal_graph_exists: bool,
    ) -> Vec<GraphBoundaryMappingInfo> {
        container
            .boundary
            .mappings
            .iter()
            .map(|mapping| {
                let public_port_exists = match mapping.direction {
                    GraphBoundaryMappingDirection::Input => container
                        .boundary
                        .inputs
                        .iter()
                        .any(|port| port.name == mapping.public_port_name),
                    GraphBoundaryMappingDirection::Output => container
                        .boundary
                        .outputs
                        .iter()
                        .any(|port| port.name == mapping.public_port_name),
                };
                let status = if !internal_graph_exists {
                    GraphBoundaryMappingStatus::MissingInternalGraph
                } else if !public_port_exists {
                    GraphBoundaryMappingStatus::MissingPublicPort
                } else if mapping.internal_node_id.trim().is_empty()
                    || mapping.internal_port_name.trim().is_empty()
                {
                    GraphBoundaryMappingStatus::MissingInternalAnchor
                } else {
                    GraphBoundaryMappingStatus::Resolved
                };
                GraphBoundaryMappingInfo {
                    direction: mapping.direction,
                    public_port_name: mapping.public_port_name.clone(),
                    internal_node_id: mapping.internal_node_id.clone(),
                    internal_port_name: mapping.internal_port_name.clone(),
                    status,
                }
            })
            .collect()
    }

    fn normalize_graph_containers(&mut self) {
        let graph_container_node_ids = self
            .nodes
            .iter()
            .filter(|node| node.kind == NodeKind::GraphContainer)
            .map(|node| node.node_id.clone())
            .collect::<Vec<_>>();
        let mut seen_container_node_ids = Vec::new();
        self.graph_containers.retain(|container| {
            if container.container_node_id.is_empty()
                || seen_container_node_ids
                    .iter()
                    .any(|node_id| node_id == &container.container_node_id)
            {
                return false;
            }
            let node_exists = graph_container_node_ids
                .iter()
                .any(|node_id| node_id == &container.container_node_id);
            if node_exists {
                seen_container_node_ids.push(container.container_node_id.clone());
            }
            node_exists
        });
    }

    pub fn readable_node_path(&self, node_index: usize) -> Option<String> {
        self.nodes
            .get(node_index)
            .map(|node| self.readable_node_path_for_node(node))
    }

    fn graph_location_for_node(&self, node: &GraphNode) -> GraphLocationInfo {
        let parent_graph_id = self.node_parent_graph_id(node);
        let name_collision_count = self
            .nodes
            .iter()
            .filter(|candidate| {
                self.node_parent_graph_id(candidate) == parent_graph_id
                    && candidate.name == node.name
            })
            .count();
        GraphLocationInfo {
            graph_id: parent_graph_id.to_owned(),
            graph_path: self.graph_path_for_id(parent_graph_id).to_owned(),
            node_name: node.name.clone(),
            node_path: self.readable_node_path_for_node(node),
            name_collision_count,
        }
    }

    fn node_parent_graph_id<'a>(&'a self, node: &'a GraphNode) -> &'a str {
        if node.parent_graph_id.is_empty() {
            MAIN_GRAPH_ID
        } else {
            node.parent_graph_id.as_str()
        }
    }

    fn annotation_parent_graph_id<'a>(&'a self, annotation: &'a GraphAnnotation) -> &'a str {
        if annotation.parent_graph_id.is_empty() {
            MAIN_GRAPH_ID
        } else {
            annotation.parent_graph_id.as_str()
        }
    }

    fn graph_path_for_id(&self, graph_id: &str) -> &str {
        self.graph_registry
            .graph(graph_id)
            .map(|graph| graph.path.as_str())
            .unwrap_or("/obj/main")
    }

    fn readable_node_path_for_node(&self, node: &GraphNode) -> String {
        self.readable_node_path_for_graph_and_name(self.node_parent_graph_id(node), &node.name)
    }

    fn readable_node_path_for_graph_and_name(&self, graph_id: &str, node_name: &str) -> String {
        format!(
            "{}/{}",
            self.graph_path_for_id(graph_id).trim_end_matches('/'),
            node_name
        )
    }

    fn with_default_data_flow_edges(mut self) -> Self {
        self.rebuild_default_data_flow_edges();
        self
    }

    fn rebuild_default_data_flow_edges(&mut self) {
        self.data_flow_edges = Self::default_data_flow_edges_for_nodes(&self.nodes);
    }

    fn default_data_flow_edges_for_nodes(nodes: &[GraphNode]) -> Vec<GraphDataFlowEdge> {
        let mut nodes_by_graph = std::collections::BTreeMap::<&str, Vec<&GraphNode>>::new();
        for node in nodes.iter().filter(|node| node.participates_in_output) {
            let graph_id = if node.parent_graph_id.is_empty() {
                MAIN_GRAPH_ID
            } else {
                node.parent_graph_id.as_str()
            };
            nodes_by_graph.entry(graph_id).or_default().push(node);
        }

        nodes_by_graph
            .values()
            .flat_map(|nodes| {
                nodes.windows(2).map(|nodes| {
                    let from_node = nodes[0];
                    let to_node = nodes[1];
                    GraphDataFlowEdge {
                        edge_id: Self::data_flow_edge_id(
                            &from_node.node_id,
                            PRIMARY_GEOMETRY_OUTPUT,
                            &to_node.node_id,
                            PRIMARY_GEOMETRY_OUTPUT,
                        ),
                        from_node_id: from_node.node_id.clone(),
                        from_output: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
                        to_node_id: to_node.node_id.clone(),
                        to_input: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
                    }
                })
            })
            .collect()
    }

    fn data_flow_edge_id(
        from_node_id: &str,
        from_output: &str,
        to_node_id: &str,
        to_input: &str,
    ) -> String {
        format!("{from_node_id}:{from_output}->{to_node_id}:{to_input}")
    }

    #[allow(dead_code)]
    pub fn can_add_data_flow_edge(&self, edge: &GraphDataFlowEdge) -> bool {
        self.data_flow_edge_addition_diagnostic(edge).is_none()
    }

    #[allow(dead_code)]
    pub fn preview_add_data_flow_edge(
        &self,
        from_node_id: &str,
        from_output: &str,
        to_node_id: &str,
        to_input: &str,
    ) -> Result<String, GraphDataFlowEdgeDiagnostic> {
        let edge = GraphDataFlowEdge {
            edge_id: Self::data_flow_edge_id(from_node_id, from_output, to_node_id, to_input),
            from_node_id: from_node_id.to_owned(),
            from_output: from_output.to_owned(),
            to_node_id: to_node_id.to_owned(),
            to_input: to_input.to_owned(),
        };
        if let Some(diagnostic) = self.data_flow_edge_addition_diagnostic(&edge) {
            return Err(diagnostic);
        }
        Ok(edge.edge_id)
    }

    #[allow(dead_code)]
    pub fn add_data_flow_edge(
        &mut self,
        from_node_id: &str,
        from_output: &str,
        to_node_id: &str,
        to_input: &str,
    ) -> Result<String, GraphDataFlowEdgeDiagnostic> {
        self.preview_add_data_flow_edge(from_node_id, from_output, to_node_id, to_input)?;

        let edge = GraphDataFlowEdge {
            edge_id: Self::data_flow_edge_id(from_node_id, from_output, to_node_id, to_input),
            from_node_id: from_node_id.to_owned(),
            from_output: from_output.to_owned(),
            to_node_id: to_node_id.to_owned(),
            to_input: to_input.to_owned(),
        };
        let edge_id = edge.edge_id.clone();
        let readable_path = self.readable_data_flow_edge_path(&edge);
        self.data_flow_edges.push(edge.clone());
        if let Some(target_index) = self
            .nodes
            .iter()
            .position(|node| node.node_id == edge.to_node_id)
        {
            self.mark_node_stale(target_index);
        }
        self.record_project_command(ProjectCommand::DataFlowEdgeAdd {
            readable_path,
            edge,
        });
        Ok(edge_id)
    }

    #[allow(dead_code)]
    pub fn remove_data_flow_edge(&mut self, edge_id: &str) -> Option<GraphDataFlowEdge> {
        let edge = self
            .data_flow_edges
            .iter()
            .find(|edge| edge.edge_id == edge_id)?
            .clone();
        let readable_path = self.readable_data_flow_edge_path(&edge);
        if !self.remove_data_flow_edge_without_history(edge_id) {
            return None;
        }
        self.record_project_command(ProjectCommand::DataFlowEdgeRemove {
            readable_path,
            edge: edge.clone(),
        });
        Some(edge)
    }

    #[allow(dead_code)]
    pub fn data_flow_edge_readable_path(&self, edge_id: &str) -> Option<String> {
        self.data_flow_edges
            .iter()
            .find(|edge| edge.edge_id == edge_id)
            .map(|edge| self.readable_data_flow_edge_path(edge))
    }

    #[allow(dead_code)]
    pub fn node_has_primary_geometry_output(&self, node_index: usize) -> bool {
        self.nodes.get(node_index).is_some_and(|node| {
            self.node_output_kind_for_name(node, PRIMARY_GEOMETRY_OUTPUT)
                .is_some()
        })
    }

    #[allow(dead_code)]
    pub fn node_has_primary_geometry_input(&self, node_index: usize) -> bool {
        self.nodes.get(node_index).is_some_and(|node| {
            if node.kind == NodeKind::Source {
                return false;
            }
            self.node_input_kind_for_name(node, PRIMARY_GEOMETRY_OUTPUT)
                .is_some()
        })
    }

    #[allow(dead_code)]
    pub fn insert_node_on_data_flow_edge(
        &mut self,
        edge_id: &str,
        inserted_node_id: &str,
        inserted_input: &str,
        inserted_output: &str,
    ) -> Option<Result<InsertNodeOnConnectionResult, Vec<GraphDataFlowEdgeDiagnostic>>> {
        let removed_edge = self
            .data_flow_edges
            .iter()
            .find(|edge| edge.edge_id == edge_id)?
            .clone();
        let inserted_node_name = self
            .nodes
            .iter()
            .find(|node| node.node_id == inserted_node_id)?
            .name
            .clone();
        let incoming_edge = GraphDataFlowEdge {
            edge_id: Self::data_flow_edge_id(
                &removed_edge.from_node_id,
                &removed_edge.from_output,
                inserted_node_id,
                inserted_input,
            ),
            from_node_id: removed_edge.from_node_id.clone(),
            from_output: removed_edge.from_output.clone(),
            to_node_id: inserted_node_id.to_owned(),
            to_input: inserted_input.to_owned(),
        };
        let outgoing_edge = GraphDataFlowEdge {
            edge_id: Self::data_flow_edge_id(
                inserted_node_id,
                inserted_output,
                &removed_edge.to_node_id,
                &removed_edge.to_input,
            ),
            from_node_id: inserted_node_id.to_owned(),
            from_output: inserted_output.to_owned(),
            to_node_id: removed_edge.to_node_id.clone(),
            to_input: removed_edge.to_input.clone(),
        };

        let mut candidate_edges = self
            .data_flow_edges
            .iter()
            .filter(|edge| edge.edge_id != removed_edge.edge_id)
            .cloned()
            .collect::<Vec<_>>();
        let mut diagnostics = Vec::new();
        if let Some(diagnostic) =
            self.data_flow_edge_addition_diagnostic_with_edges(&incoming_edge, &candidate_edges)
        {
            diagnostics.push(diagnostic);
        } else {
            candidate_edges.push(incoming_edge.clone());
        }
        if let Some(diagnostic) =
            self.data_flow_edge_addition_diagnostic_with_edges(&outgoing_edge, &candidate_edges)
        {
            diagnostics.push(diagnostic);
        } else {
            candidate_edges.push(outgoing_edge.clone());
        }
        if !diagnostics.is_empty() {
            return Some(Err(diagnostics));
        }

        if !self.remove_data_flow_edge_without_history(&removed_edge.edge_id) {
            return None;
        }
        let added_edges = vec![incoming_edge, outgoing_edge];
        for edge in &added_edges {
            if !self.add_data_flow_edge_without_history(edge.clone()) {
                return None;
            }
        }
        let readable_path = self.readable_data_flow_edge_path(&removed_edge);
        self.record_project_command(ProjectCommand::DataFlowEdgeInsertNode {
            readable_path,
            inserted_node_name,
            removed_edge: removed_edge.clone(),
            added_edges: added_edges.clone(),
        });
        Some(Ok(InsertNodeOnConnectionResult {
            removed_edge,
            added_edges,
        }))
    }

    pub fn data_flow_edge_diagnostics(&self) -> Vec<GraphDataFlowEdgeDiagnostic> {
        self.data_flow_edges
            .iter()
            .filter_map(|edge| self.data_flow_edge_diagnostic(edge))
            .collect()
    }

    #[allow(dead_code)]
    pub fn current_graph_data_flow_edge_diagnostics(&self) -> Vec<GraphDataFlowEdgeDiagnostic> {
        self.graph_data_flow_edge_diagnostics(self.current_graph_id())
    }

    #[allow(dead_code)]
    pub fn graph_data_flow_edge_diagnostics(
        &self,
        graph_id: &str,
    ) -> Vec<GraphDataFlowEdgeDiagnostic> {
        let graph_id = if graph_id.is_empty() {
            MAIN_GRAPH_ID
        } else {
            graph_id
        };
        self.data_flow_edges
            .iter()
            .filter(|edge| self.data_flow_edge_touches_graph(edge, graph_id))
            .filter_map(|edge| self.data_flow_edge_diagnostic(edge))
            .collect()
    }

    fn data_flow_edge_touches_graph(&self, edge: &GraphDataFlowEdge, graph_id: &str) -> bool {
        [edge.from_node_id.as_str(), edge.to_node_id.as_str()]
            .into_iter()
            .filter_map(|node_id| self.nodes.iter().find(|node| node.node_id == node_id))
            .any(|node| self.node_parent_graph_id(node) == graph_id)
    }

    #[allow(dead_code)]
    fn data_flow_edge_addition_diagnostic(
        &self,
        edge: &GraphDataFlowEdge,
    ) -> Option<GraphDataFlowEdgeDiagnostic> {
        self.data_flow_edge_addition_diagnostic_with_edges(edge, &self.data_flow_edges)
    }

    fn data_flow_edge_addition_diagnostic_with_edges(
        &self,
        edge: &GraphDataFlowEdge,
        data_flow_edges: &[GraphDataFlowEdge],
    ) -> Option<GraphDataFlowEdgeDiagnostic> {
        if data_flow_edges
            .iter()
            .any(|existing_edge| existing_edge.edge_id == edge.edge_id)
        {
            return Some(GraphDataFlowEdgeDiagnostic {
                edge_id: edge.edge_id.clone(),
                status: GraphDataFlowEdgeDiagnosticStatus::DuplicateConnection,
                readable_path: self.readable_data_flow_edge_path(edge),
                message: "Connection already exists.".to_owned(),
            });
        }

        self.data_flow_edge_endpoint_diagnostic(edge).or_else(|| {
            self.edge_would_create_cycle_in_edges(
                &edge.from_node_id,
                &edge.to_node_id,
                None,
                data_flow_edges,
            )
            .then(|| GraphDataFlowEdgeDiagnostic {
                edge_id: edge.edge_id.clone(),
                status: GraphDataFlowEdgeDiagnosticStatus::Cycle,
                readable_path: self.readable_data_flow_edge_path(edge),
                message: "Connection would create cyclic v1 data flow.".to_owned(),
            })
        })
    }

    fn add_data_flow_edge_without_history(&mut self, edge: GraphDataFlowEdge) -> bool {
        if self.data_flow_edge_addition_diagnostic(&edge).is_some() {
            return false;
        }
        self.data_flow_edges.push(edge.clone());
        if let Some(target_index) = self
            .nodes
            .iter()
            .position(|node| node.node_id == edge.to_node_id)
        {
            self.mark_node_stale(target_index);
        }
        true
    }

    fn remove_data_flow_edge_without_history(&mut self, edge_id: &str) -> bool {
        let Some(edge_index) = self
            .data_flow_edges
            .iter()
            .position(|edge| edge.edge_id == edge_id)
        else {
            return false;
        };
        let edge = self.data_flow_edges.remove(edge_index);
        if let Some(target_index) = self
            .nodes
            .iter()
            .position(|node| node.node_id == edge.to_node_id)
        {
            self.mark_node_stale(target_index);
        }
        true
    }

    #[allow(dead_code)]
    fn data_flow_edge_diagnostic(
        &self,
        edge: &GraphDataFlowEdge,
    ) -> Option<GraphDataFlowEdgeDiagnostic> {
        self.data_flow_edge_endpoint_diagnostic(edge).or_else(|| {
            self.edge_would_create_cycle(&edge.from_node_id, &edge.to_node_id, Some(&edge.edge_id))
                .then(|| GraphDataFlowEdgeDiagnostic {
                    edge_id: edge.edge_id.clone(),
                    status: GraphDataFlowEdgeDiagnosticStatus::Cycle,
                    readable_path: self.readable_data_flow_edge_path(edge),
                    message: "Loaded edge participates in cyclic v1 data flow.".to_owned(),
                })
        })
    }

    #[allow(dead_code)]
    fn data_flow_edge_endpoint_diagnostic(
        &self,
        edge: &GraphDataFlowEdge,
    ) -> Option<GraphDataFlowEdgeDiagnostic> {
        let Some(source_node) = self
            .nodes
            .iter()
            .find(|node| node.node_id == edge.from_node_id)
        else {
            return Some(GraphDataFlowEdgeDiagnostic {
                edge_id: edge.edge_id.clone(),
                status: GraphDataFlowEdgeDiagnosticStatus::MissingSourceNode,
                readable_path: self.readable_data_flow_edge_path(edge),
                message: format!("Source node id '{}' is missing.", edge.from_node_id),
            });
        };
        let Some(source_output_kind) =
            self.node_output_kind_for_name(source_node, &edge.from_output)
        else {
            if self.node_primary_output_name(source_node).is_some() {
                return Some(GraphDataFlowEdgeDiagnostic {
                    edge_id: edge.edge_id.clone(),
                    status: GraphDataFlowEdgeDiagnosticStatus::MissingSourcePort,
                    readable_path: self.readable_data_flow_edge_path(edge),
                    message: format!("Source port '{}' is not available.", edge.from_output),
                });
            }
            return Some(GraphDataFlowEdgeDiagnostic {
                edge_id: edge.edge_id.clone(),
                status: GraphDataFlowEdgeDiagnosticStatus::IncompatibleDataKind,
                readable_path: self.readable_data_flow_edge_path(edge),
                message: format!(
                    "{} cannot produce a geometry-table output.",
                    source_node.kind.as_str()
                ),
            });
        };
        if source_output_kind != HoudiniDataKind::GeometryTable {
            return Some(GraphDataFlowEdgeDiagnostic {
                edge_id: edge.edge_id.clone(),
                status: GraphDataFlowEdgeDiagnosticStatus::IncompatibleDataKind,
                readable_path: self.readable_data_flow_edge_path(edge),
                message: format!(
                    "Source port '{}' is not geometry-table data.",
                    edge.from_output
                ),
            });
        }
        let Some(target_node) = self
            .nodes
            .iter()
            .find(|node| node.node_id == edge.to_node_id)
        else {
            return Some(GraphDataFlowEdgeDiagnostic {
                edge_id: edge.edge_id.clone(),
                status: GraphDataFlowEdgeDiagnosticStatus::MissingTargetNode,
                readable_path: self.readable_data_flow_edge_path(edge),
                message: format!("Target node id '{}' is missing.", edge.to_node_id),
            });
        };
        let Some(target_input_kind) = self.node_input_kind_for_name(target_node, &edge.to_input)
        else {
            return Some(GraphDataFlowEdgeDiagnostic {
                edge_id: edge.edge_id.clone(),
                status: GraphDataFlowEdgeDiagnosticStatus::MissingTargetPort,
                readable_path: self.readable_data_flow_edge_path(edge),
                message: format!("Target port '{}' is not available.", edge.to_input),
            });
        };
        if target_input_kind != HoudiniDataKind::GeometryTable {
            return Some(GraphDataFlowEdgeDiagnostic {
                edge_id: edge.edge_id.clone(),
                status: GraphDataFlowEdgeDiagnosticStatus::IncompatibleDataKind,
                readable_path: self.readable_data_flow_edge_path(edge),
                message: format!(
                    "Target port '{}' is not geometry-table data.",
                    edge.to_input
                ),
            });
        }
        None
    }

    #[allow(dead_code)]
    fn edge_would_create_cycle(
        &self,
        from_node_id: &str,
        to_node_id: &str,
        ignored_edge_id: Option<&str>,
    ) -> bool {
        self.edge_would_create_cycle_in_edges(
            from_node_id,
            to_node_id,
            ignored_edge_id,
            &self.data_flow_edges,
        )
    }

    fn edge_would_create_cycle_in_edges(
        &self,
        from_node_id: &str,
        to_node_id: &str,
        ignored_edge_id: Option<&str>,
        data_flow_edges: &[GraphDataFlowEdge],
    ) -> bool {
        let mut adjacency = std::collections::BTreeMap::<&str, Vec<&str>>::new();
        for edge in data_flow_edges {
            if ignored_edge_id.is_some_and(|edge_id| edge.edge_id == edge_id) {
                continue;
            }
            adjacency
                .entry(edge.from_node_id.as_str())
                .or_default()
                .push(edge.to_node_id.as_str());
        }
        path_exists(to_node_id, from_node_id, &adjacency)
    }

    fn data_flow_info_for_node(&self, node: &GraphNode) -> NodeDataFlowInfo {
        let incoming_edge_count = self
            .data_flow_edges
            .iter()
            .filter(|edge| edge.to_node_id == node.node_id)
            .count();
        let outgoing_edge_count = self
            .data_flow_edges
            .iter()
            .filter(|edge| edge.from_node_id == node.node_id)
            .count();
        let diagnostics = self
            .data_flow_edge_diagnostics()
            .into_iter()
            .filter(|diagnostic| {
                self.data_flow_edges
                    .iter()
                    .find(|edge| edge.edge_id == diagnostic.edge_id)
                    .is_some_and(|edge| {
                        edge.from_node_id == node.node_id || edge.to_node_id == node.node_id
                    })
            })
            .collect();
        NodeDataFlowInfo {
            incoming_edge_count,
            outgoing_edge_count,
            diagnostics,
        }
    }

    fn readable_data_flow_edge_path(&self, edge: &GraphDataFlowEdge) -> String {
        format!(
            "{}:{} -> {}:{}",
            self.readable_node_path_for_id(&edge.from_node_id),
            edge.from_output,
            self.readable_node_path_for_id(&edge.to_node_id),
            edge.to_input
        )
    }

    fn readable_node_path_for_id(&self, node_id: &str) -> String {
        self.nodes
            .iter()
            .find(|node| node.node_id == node_id)
            .map(|node| self.readable_node_path_for_node(node))
            .unwrap_or_else(|| format!("{}/<missing:{node_id}>", self.current_graph_path()))
    }

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
            graph_registry: ProjectGraphRegistry::default(),
            graph_containers: Vec::new(),
            data_flow_edges: Vec::new(),
            nodes: vec![
                GraphNode {
                    node_id: "source.main".to_owned(),
                    parent_graph_id: MAIN_GRAPH_ID.to_owned(),
                    name: "Source".to_owned(),
                    kind: NodeKind::Source,
                    layout_position: GraphPoint::new(0.0, 0.5),
                    generated: None,
                    coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
                    source_node: None,
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
                    parent_graph_id: MAIN_GRAPH_ID.to_owned(),
                    name: "Filter".to_owned(),
                    kind: NodeKind::Filter,
                    layout_position: GraphPoint::new(0.33, 0.5),
                    generated: None,
                    coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
                    source_node: None,
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
                    parent_graph_id: MAIN_GRAPH_ID.to_owned(),
                    name: "Style".to_owned(),
                    kind: NodeKind::Style,
                    layout_position: GraphPoint::new(0.66, 0.5),
                    generated: None,
                    coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
                    source_node: None,
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
                    parent_graph_id: MAIN_GRAPH_ID.to_owned(),
                    name: "Rerun Output".to_owned(),
                    kind: NodeKind::Output,
                    layout_position: GraphPoint::new(1.0, 0.5),
                    generated: None,
                    coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
                    source_node: None,
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
                    MAIN_GRAPH_ID.to_owned(),
                    "Prep".to_owned(),
                    GraphPoint::new(0.03, 0.24),
                    GraphPoint::new(0.62, 0.48),
                    vec!["source.main".to_owned(), "filter.main".to_owned()],
                ),
                GraphAnnotation::sticky_note(
                    "note.review".to_owned(),
                    MAIN_GRAPH_ID.to_owned(),
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
            substrate_raster: None,
            geometry,
            recording_geometry: Vec::new(),
            python_operator_declarations: Vec::new(),
            procedural_asset_declarations: Vec::new(),
            native_operator_declarations: Vec::new(),
            native_operator_trust: NativeOperatorTrustPolicy::default(),
            python_environment: PythonEnvironmentDescriptor::default(),
            evaluation_mode: GraphEvaluationMode::default(),
            command_history: ProjectCommandHistory::default(),
            work_items: Vec::new(),
        }
        .with_default_data_flow_edges()
    }

    pub fn malware_starter() -> Self {
        let geometry = vec![
            Geometry::Polygon(Polygon {
                points: vec![
                    GraphPoint::new(20.0, 24.0),
                    GraphPoint::new(86.0, 18.0),
                    GraphPoint::new(112.0, 54.0),
                    GraphPoint::new(92.0, 116.0),
                    GraphPoint::new(34.0, 104.0),
                ],
                score: 0.91,
            }),
            Geometry::Polygon(Polygon {
                points: vec![
                    GraphPoint::new(132.0, 36.0),
                    GraphPoint::new(214.0, 42.0),
                    GraphPoint::new(226.0, 88.0),
                    GraphPoint::new(192.0, 126.0),
                    GraphPoint::new(144.0, 104.0),
                ],
                score: 0.84,
            }),
            Geometry::Polygon(Polygon {
                points: vec![
                    GraphPoint::new(46.0, 152.0),
                    GraphPoint::new(104.0, 136.0),
                    GraphPoint::new(142.0, 164.0),
                    GraphPoint::new(118.0, 202.0),
                    GraphPoint::new(76.0, 190.0),
                    GraphPoint::new(58.0, 220.0),
                    GraphPoint::new(28.0, 190.0),
                ],
                score: 0.76,
            }),
            Geometry::Polygon(Polygon {
                points: vec![
                    GraphPoint::new(156.0, 148.0),
                    GraphPoint::new(228.0, 136.0),
                    GraphPoint::new(236.0, 216.0),
                    GraphPoint::new(178.0, 232.0),
                    GraphPoint::new(166.0, 196.0),
                    GraphPoint::new(130.0, 184.0),
                ],
                score: 0.68,
            }),
        ];
        let coordinate_contract = SubstrateCoordinateContract::malware_byteplot();
        let mut metadata = SourceMetadata::from_geometry(
            SourceProvenance::SyntheticMalware,
            Some("examples/malware_byteplot/mock-byteplot.png".to_owned()),
            &geometry,
            Vec::new(),
        );
        metadata.attribute_names = vec![
            "score".to_owned(),
            "region_family".to_owned(),
            "hull_kind".to_owned(),
            "record_id".to_owned(),
        ];

        Self {
            source: GraphSource::malware_starter(metadata),
            graph_registry: ProjectGraphRegistry::default(),
            graph_containers: Vec::new(),
            data_flow_edges: Vec::new(),
            nodes: vec![
                GraphNode {
                    node_id: "source.byteplot".to_owned(),
                    parent_graph_id: MAIN_GRAPH_ID.to_owned(),
                    name: "Byteplot Substrate".to_owned(),
                    kind: NodeKind::Source,
                    layout_position: GraphPoint::new(0.0, 0.28),
                    generated: None,
                    coordinate_contract: Some(coordinate_contract.clone()),
                    source_node: None,
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    substrate_projection: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    comment: "Synthetic 256x256 byteplot image substrate; polygons use this pixel space.".to_owned(),
                    show_comment_in_network: true,
                    parameter: NodeParameter::scalar(
                        "Image ready",
                        1.0,
                        0.0..=1.0,
                        "Synthetic byteplot substrate readiness for the malware starter graph.",
                    ),
                    info: "Loads the byteplot image substrate that region polygons annotate.",
                },
                GraphNode {
                    node_id: "source.convex_regions".to_owned(),
                    parent_graph_id: MAIN_GRAPH_ID.to_owned(),
                    name: "Convex Hull Regions".to_owned(),
                    kind: NodeKind::Source,
                    layout_position: GraphPoint::new(0.0, 0.56),
                    generated: None,
                    coordinate_contract: Some(coordinate_contract.clone()),
                    source_node: None,
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    substrate_projection: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    comment: "Mock upstream ML detections represented as convex polygon hull records.".to_owned(),
                    show_comment_in_network: true,
                    parameter: NodeParameter::scalar(
                        "Layer enabled",
                        1.0,
                        0.0..=1.0,
                        "Convex hull polygon source visibility placeholder.",
                    ),
                    info: "Loads one independent polygon source layer for convex malware regions.",
                },
                GraphNode {
                    node_id: "source.concave_regions".to_owned(),
                    parent_graph_id: MAIN_GRAPH_ID.to_owned(),
                    name: "Concave Hull Regions".to_owned(),
                    kind: NodeKind::Source,
                    layout_position: GraphPoint::new(0.0, 0.84),
                    generated: None,
                    coordinate_contract: Some(coordinate_contract.clone()),
                    source_node: None,
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    substrate_projection: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    comment: "Mock upstream ML detections represented as concave polygon hull records.".to_owned(),
                    show_comment_in_network: true,
                    parameter: NodeParameter::scalar(
                        "Layer enabled",
                        1.0,
                        0.0..=1.0,
                        "Concave hull polygon source visibility placeholder.",
                    ),
                    info: "Loads one independent polygon source layer for concave malware regions.",
                },
                GraphNode {
                    node_id: "filter.high_confidence".to_owned(),
                    parent_graph_id: MAIN_GRAPH_ID.to_owned(),
                    name: "High Confidence Filter".to_owned(),
                    kind: NodeKind::Filter,
                    layout_position: GraphPoint::new(0.36, 0.56),
                    generated: Some(GeneratedNodeInfo::managed(
                        GeneratedNodeSource::AttributeTableCommit,
                    )),
                    coordinate_contract: Some(coordinate_contract.clone()),
                    source_node: None,
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    substrate_projection: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    comment: "Logical filtered view; records stay graph-owned and identity-preserving.".to_owned(),
                    show_comment_in_network: true,
                    parameter: NodeParameter::attribute_rule(
                        "Minimum ML score",
                        "score",
                        FilterComparison::GreaterOrEqual,
                        0.70,
                        0.0..=1.0,
                        "Filters mock malware regions by upstream ML confidence.",
                    ),
                    info: "Filters byteplot region polygons as a logical view rather than copying them.",
                },
                GraphNode {
                    node_id: "style.region_overlay".to_owned(),
                    parent_graph_id: MAIN_GRAPH_ID.to_owned(),
                    name: "Region Overlay Style".to_owned(),
                    kind: NodeKind::Style,
                    layout_position: GraphPoint::new(0.64, 0.56),
                    generated: None,
                    coordinate_contract: Some(coordinate_contract.clone()),
                    source_node: None,
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    substrate_projection: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    comment: "Applies overlay styling while preserving substrate pixel-space geometry.".to_owned(),
                    show_comment_in_network: false,
                    parameter: NodeParameter::scalar(
                        "Overlay stroke",
                        0.68,
                        0.0..=1.0,
                        "Controls polygon overlay stroke scale.",
                    ),
                    info: "Styles malware region overlays before Rerun output mapping.",
                },
                GraphNode {
                    node_id: "output.rerun_malware".to_owned(),
                    parent_graph_id: MAIN_GRAPH_ID.to_owned(),
                    name: "Rerun Malware Output".to_owned(),
                    kind: NodeKind::Output,
                    layout_position: GraphPoint::new(1.0, 0.56),
                    generated: None,
                    coordinate_contract: Some(coordinate_contract),
                    source_node: None,
                    output_operator: Some(OutputOperatorNode::rerun_scene()),
                    null_operator: None,
                    reference_input: None,
                    substrate_projection: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                    evaluation: NodeEvaluation::clean(),
                    participates_in_output: true,
                    comment: "Composes the byteplot substrate and polygon overlays for Rerun inspection.".to_owned(),
                    show_comment_in_network: true,
                    parameter: NodeParameter::scalar(
                        "Overlay fidelity",
                        1.0,
                        0.0..=1.0,
                        "Controls prepared overlay output fidelity for the Rerun target.",
                    ),
                    info: "Maps graph-owned malware substrate overlays to a Rerun viewer output.",
                },
            ],
            annotations: vec![
                GraphAnnotation::network_box(
                    "box.malware_sources".to_owned(),
                    MAIN_GRAPH_ID.to_owned(),
                    "Independent Sources".to_owned(),
                    GraphPoint::new(-0.08, 0.08),
                    GraphPoint::new(0.28, 0.90),
                    vec![
                        "source.byteplot".to_owned(),
                        "source.convex_regions".to_owned(),
                        "source.concave_regions".to_owned(),
                    ],
                ),
                GraphAnnotation::network_box(
                    "box.malware_output".to_owned(),
                    MAIN_GRAPH_ID.to_owned(),
                    "Filter Style Output".to_owned(),
                    GraphPoint::new(0.28, 0.34),
                    GraphPoint::new(0.82, 0.44),
                    vec![
                        "filter.high_confidence".to_owned(),
                        "style.region_overlay".to_owned(),
                        "output.rerun_malware".to_owned(),
                    ],
                ),
                GraphAnnotation::sticky_note(
                    "note.pixel_space".to_owned(),
                    MAIN_GRAPH_ID.to_owned(),
                    "Pixel Space".to_owned(),
                    "All mock region polygons are authored in the byteplot image pixel coordinate space.".to_owned(),
                    GraphPoint::new(0.30, 0.08),
                    GraphPoint::new(0.44, 0.22),
                ),
            ],
            network_view: NetworkViewDisplayOptions::default(),
            layers: vec![
                Layer {
                    name: "Malware hull overlays".to_owned(),
                    kind: LayerKind::Polygons,
                    visible: true,
                    order: 0,
                    style: GraphStyle {
                        color: GraphColor {
                            r: 239,
                            g: 96,
                            b: 121,
                        },
                        opacity: 0.78,
                        stroke_scale: 0.68,
                    },
                },
                Layer {
                    name: "Debug validation".to_owned(),
                    kind: LayerKind::Debug,
                    visible: false,
                    order: 1,
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
            style: GraphStyle {
                color: GraphColor {
                    r: 239,
                    g: 96,
                    b: 121,
                },
                opacity: 0.78,
                stroke_scale: 0.68,
            },
            substrate_raster: Some(SubstrateRaster::mock_malware_byteplot()),
            geometry,
            recording_geometry: Vec::new(),
            python_operator_declarations: Vec::new(),
            procedural_asset_declarations: Vec::new(),
            native_operator_declarations: Vec::new(),
            native_operator_trust: NativeOperatorTrustPolicy::default(),
            python_environment: PythonEnvironmentDescriptor::default(),
            evaluation_mode: GraphEvaluationMode::default(),
            command_history: ProjectCommandHistory::default(),
            work_items: Vec::new(),
        }
        .with_default_data_flow_edges()
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

    #[allow(dead_code)]
    pub fn source_format_capabilities(&self) -> Vec<SourceFormatCapability> {
        source_format_capabilities()
    }

    #[allow(dead_code)]
    pub fn source_format_capabilities_with_status(
        &self,
        status: SourceFormatSupportStatus,
    ) -> Vec<SourceFormatCapability> {
        source_format_capabilities()
            .into_iter()
            .filter(|capability| capability.status == status)
            .collect()
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

    pub fn set_layer_visibility(&mut self, layer_index: usize, visible: bool) -> bool {
        let Some(command) = self.set_layer_visibility_without_history(layer_index, visible) else {
            return false;
        };
        self.record_project_command(command);
        true
    }

    fn set_layer_visibility_without_history(
        &mut self,
        layer_index: usize,
        visible: bool,
    ) -> Option<ProjectCommand> {
        let layer = self.layers.get_mut(layer_index)?;
        if layer.visible == visible {
            return None;
        }
        let old_visible = layer.visible;
        layer.visible = visible;
        Some(ProjectCommand::LayerVisibilityEdit {
            layer_index,
            layer_name: layer.name.clone(),
            layer_kind: layer.kind,
            old_visible,
            new_visible: visible,
        })
    }

    pub fn set_layer_order(&mut self, layer_index: usize, order: i32) -> bool {
        let Some(command) = self.set_layer_order_without_history(layer_index, order) else {
            return false;
        };
        self.record_project_command(command);
        true
    }

    fn set_layer_order_without_history(
        &mut self,
        layer_index: usize,
        order: i32,
    ) -> Option<ProjectCommand> {
        let layer = self.layers.get_mut(layer_index)?;
        if layer.order == order {
            return None;
        }
        let old_order = layer.order;
        layer.order = order;
        Some(ProjectCommand::LayerOrderEdit {
            layer_index,
            layer_name: layer.name.clone(),
            layer_kind: layer.kind,
            old_order,
            new_order: order,
        })
    }

    pub fn add_network_box_for_node(&mut self, node_index: usize) -> Option<usize> {
        let node = self.nodes.get(node_index)?;
        let position =
            GraphPoint::new(node.layout_position.x - 0.08, node.layout_position.y - 0.16);
        let annotation = GraphAnnotation::network_box(
            self.unique_annotation_id("box"),
            self.current_graph_id().to_owned(),
            self.unique_annotation_title("Network Box"),
            position,
            GraphPoint::new(0.22, 0.24),
            vec![node.node_id.clone()],
        );
        let insert_index = self.annotations.len();
        self.annotations.push(annotation.clone());
        self.record_project_command(ProjectCommand::AnnotationCreate {
            annotation,
            insert_index,
        });
        Some(insert_index)
    }

    pub fn add_sticky_note_near_node(&mut self, node_index: usize) -> Option<usize> {
        let node = self.nodes.get(node_index)?;
        let position =
            GraphPoint::new(node.layout_position.x + 0.08, node.layout_position.y - 0.18);
        let annotation = GraphAnnotation::sticky_note(
            self.unique_annotation_id("note"),
            self.current_graph_id().to_owned(),
            self.unique_annotation_title("Sticky Note"),
            String::new(),
            position,
            GraphPoint::new(0.22, 0.20),
        );
        let insert_index = self.annotations.len();
        self.annotations.push(annotation.clone());
        self.record_project_command(ProjectCommand::AnnotationCreate {
            annotation,
            insert_index,
        });
        Some(insert_index)
    }

    pub fn remove_annotation(&mut self, annotation_index: usize) -> Option<GraphAnnotation> {
        if annotation_index >= self.annotations.len() {
            return None;
        }
        let annotation = self.annotations.remove(annotation_index);
        self.record_project_command(ProjectCommand::AnnotationDelete {
            annotation: annotation.clone(),
            remove_index: annotation_index,
        });
        Some(annotation)
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
        let node_parent_graph_id = if node.parent_graph_id.is_empty() {
            MAIN_GRAPH_ID.to_owned()
        } else {
            node.parent_graph_id.clone()
        };
        let node_position = node.layout_position;
        let mut changed = false;

        for annotation in &mut self.annotations {
            if annotation.kind != GraphAnnotationKind::NetworkBox {
                continue;
            }
            let annotation_parent_graph_id = if annotation.parent_graph_id.is_empty() {
                MAIN_GRAPH_ID
            } else {
                annotation.parent_graph_id.as_str()
            };
            if annotation_parent_graph_id != node_parent_graph_id {
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

    pub fn set_annotation_size(&mut self, annotation_index: usize, size: GraphPoint) -> bool {
        let Some(annotation) = self.annotations.get_mut(annotation_index) else {
            return false;
        };
        annotation.size = size;
        true
    }

    pub fn annotation_member_layout_positions(
        &self,
        annotation_index: usize,
    ) -> Vec<(String, GraphPoint)> {
        let Some(annotation) = self.annotations.get(annotation_index) else {
            return Vec::new();
        };
        if annotation.kind != GraphAnnotationKind::NetworkBox {
            return Vec::new();
        }

        self.nodes
            .iter()
            .filter(|node| {
                annotation
                    .member_node_ids
                    .iter()
                    .any(|member_node_id| member_node_id == &node.node_id)
            })
            .map(|node| (node.node_id.clone(), node.layout_position))
            .collect()
    }

    pub fn network_box_organization_snapshots(&self) -> Vec<NetworkBoxOrganizationSnapshot> {
        self.annotations
            .iter()
            .filter(|annotation| {
                annotation.kind == GraphAnnotationKind::NetworkBox
                    && self.annotation_parent_graph_id(annotation) == self.current_graph_id()
            })
            .map(NetworkBoxOrganizationSnapshot::from_annotation)
            .collect()
    }

    pub fn finish_annotation_drag(
        &mut self,
        annotation_index: usize,
        old_position: GraphPoint,
        old_member_positions: &[(String, GraphPoint)],
    ) -> bool {
        let Some(annotation) = self.annotations.get(annotation_index) else {
            return false;
        };
        let new_position = annotation.position;
        let moved_nodes = old_member_positions
            .iter()
            .filter_map(|(node_id, old_node_position)| {
                let node = self.nodes.iter().find(|node| node.node_id == *node_id)?;
                (node.layout_position != *old_node_position).then(|| NodeLayoutCommandSnapshot {
                    node_id: node.node_id.clone(),
                    old_position: *old_node_position,
                    new_position: node.layout_position,
                })
            })
            .collect::<Vec<_>>();
        if old_position == new_position && moved_nodes.is_empty() {
            return false;
        }
        let annotation_id = annotation.annotation_id.clone();
        let annotation_title = annotation.title.clone();
        self.record_project_command(ProjectCommand::AnnotationMoveEdit {
            annotation_id,
            annotation_title,
            old_position,
            new_position,
            moved_nodes,
        });
        true
    }

    pub fn finish_annotation_resize(
        &mut self,
        annotation_index: usize,
        old_size: GraphPoint,
    ) -> bool {
        let Some(annotation) = self.annotations.get(annotation_index) else {
            return false;
        };
        let new_size = annotation.size;
        if old_size == new_size {
            return false;
        }
        let annotation_id = annotation.annotation_id.clone();
        let annotation_title = annotation.title.clone();
        self.record_project_command(ProjectCommand::AnnotationResizeEdit {
            annotation_id,
            annotation_title,
            old_size,
            new_size,
        });
        true
    }

    pub fn set_annotation_title(&mut self, annotation_index: usize, title: String) -> bool {
        let Some(annotation) = self.annotations.get_mut(annotation_index) else {
            return false;
        };
        if annotation.title == title {
            return false;
        }
        let old_title = annotation.title.clone();
        annotation.title = title.clone();
        let annotation_id = annotation.annotation_id.clone();
        self.record_project_command(ProjectCommand::AnnotationTitleEdit {
            annotation_id,
            annotation_title: old_title.clone(),
            old_title,
            new_title: title,
        });
        true
    }

    pub fn set_annotation_text(&mut self, annotation_index: usize, text: String) -> bool {
        let Some(annotation) = self.annotations.get_mut(annotation_index) else {
            return false;
        };
        if annotation.kind != GraphAnnotationKind::StickyNote || annotation.text == text {
            return false;
        }
        let old_text = annotation.text.clone();
        annotation.text = text.clone();
        let annotation_id = annotation.annotation_id.clone();
        let annotation_title = annotation.title.clone();
        self.record_project_command(ProjectCommand::AnnotationTextEdit {
            annotation_id,
            annotation_title,
            old_text,
            new_text: text,
        });
        true
    }

    pub fn set_annotation_collapsed(&mut self, annotation_index: usize, collapsed: bool) -> bool {
        let Some(annotation) = self.annotations.get_mut(annotation_index) else {
            return false;
        };
        if annotation.collapsed == collapsed {
            return false;
        }
        let old_collapsed = annotation.collapsed;
        annotation.collapsed = collapsed;
        let annotation_id = annotation.annotation_id.clone();
        let annotation_title = annotation.title.clone();
        self.record_project_command(ProjectCommand::AnnotationCollapsedEdit {
            annotation_id,
            annotation_title,
            old_collapsed,
            new_collapsed: collapsed,
        });
        true
    }

    pub fn set_all_annotations_collapsed(&mut self, collapsed: bool) -> bool {
        let current_graph_id = self.current_graph_id().to_owned();
        let collapsed_annotations = self
            .annotations
            .iter_mut()
            .filter_map(|annotation| {
                if (annotation.parent_graph_id.as_str() != current_graph_id
                    && !(annotation.parent_graph_id.is_empty()
                        && current_graph_id == MAIN_GRAPH_ID))
                    || annotation.collapsed == collapsed
                {
                    return None;
                }
                let old_collapsed = annotation.collapsed;
                annotation.collapsed = collapsed;
                Some(AnnotationCollapsedCommandSnapshot {
                    annotation_id: annotation.annotation_id.clone(),
                    old_collapsed,
                    new_collapsed: collapsed,
                })
            })
            .collect::<Vec<_>>();
        if collapsed_annotations.is_empty() {
            return false;
        }

        self.record_project_command(ProjectCommand::AnnotationsCollapsedEdit {
            collapsed,
            annotations: collapsed_annotations,
        });
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
        if annotation.position == position && annotation.size == size {
            return false;
        }
        let old_position = annotation.position;
        let old_size = annotation.size;
        let annotation_id = annotation.annotation_id.clone();
        let annotation_title = annotation.title.clone();
        annotation.position = position;
        annotation.size = size;
        self.record_project_command(ProjectCommand::AnnotationBoundsEdit {
            annotation_id,
            annotation_title,
            old_position,
            new_position: position,
            old_size,
            new_size: size,
        });
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

        let Some(filter_node_index) = self
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Filter)
        else {
            return false;
        };
        if !self.node_accepts_layer_managed_update(filter_node_index) {
            return false;
        }
        let filter_node = &mut self.nodes[filter_node_index];

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

    pub fn set_node_parameter_value(&mut self, node_index: usize, value: f32) -> bool {
        let Some(command) = self.set_node_parameter_value_without_history(node_index, value) else {
            return false;
        };
        self.record_project_command(command);
        self.mark_node_stale(node_index);
        true
    }

    fn set_node_parameter_value_without_history(
        &mut self,
        node_index: usize,
        value: f32,
    ) -> Option<ProjectCommand> {
        let node = self.nodes.get_mut(node_index)?;
        let clamped_value = value.clamp(*node.parameter.range.start(), *node.parameter.range.end());
        if (node.parameter.value - clamped_value).abs() <= f32::EPSILON {
            return None;
        }
        let old_value = node.parameter.value;
        node.parameter.value = clamped_value;
        Some(ProjectCommand::NodeParameterEdit {
            node_id: node.node_id.clone(),
            node_name: node.name.clone(),
            parameter_name: node.parameter.name.to_owned(),
            old_value,
            new_value: clamped_value,
        })
    }

    fn record_project_command(&mut self, command: ProjectCommand) {
        if let Some(last_command) = self.command_history.undo_stack.last_mut()
            && last_command.coalesce_with(&command)
        {
            self.command_history.redo_stack.clear();
            return;
        }
        self.command_history.undo_stack.push(command);
        self.command_history.redo_stack.clear();
    }

    pub fn undo_project_command(&mut self) -> bool {
        let Some(command) = self.command_history.undo_stack.pop() else {
            return false;
        };
        if self.apply_project_command(&command, ProjectCommandDirection::Undo) {
            if command.rebuilds_default_data_flow_edges_after_apply() {
                self.rebuild_default_data_flow_edges();
            }
            self.command_history.redo_stack.push(command);
            true
        } else {
            self.command_history.undo_stack.push(command);
            false
        }
    }

    pub fn redo_project_command(&mut self) -> bool {
        let Some(command) = self.command_history.redo_stack.pop() else {
            return false;
        };
        if self.apply_project_command(&command, ProjectCommandDirection::Redo) {
            if command.rebuilds_default_data_flow_edges_after_apply() {
                self.rebuild_default_data_flow_edges();
            }
            self.command_history.undo_stack.push(command);
            true
        } else {
            self.command_history.redo_stack.push(command);
            false
        }
    }

    pub fn undo_project_command_label(&self) -> Option<String> {
        self.command_history
            .undo_stack
            .last()
            .map(ProjectCommand::summary)
    }

    pub fn redo_project_command_label(&self) -> Option<String> {
        self.command_history
            .redo_stack
            .last()
            .map(ProjectCommand::summary)
    }

    fn apply_project_command(
        &mut self,
        command: &ProjectCommand,
        direction: ProjectCommandDirection,
    ) -> bool {
        match command {
            ProjectCommand::NodeRename {
                node_id,
                old_name,
                new_name,
            } => {
                let Some(node) = self.nodes.iter_mut().find(|node| node.node_id == *node_id) else {
                    return false;
                };
                node.name = match direction {
                    ProjectCommandDirection::Undo => old_name.clone(),
                    ProjectCommandDirection::Redo => new_name.clone(),
                };
                true
            }
            ProjectCommand::NodeDuplicate {
                duplicated_node,
                insert_index,
                ..
            } => match direction {
                ProjectCommandDirection::Undo => {
                    let Some(node_index) = self
                        .nodes
                        .iter()
                        .position(|node| node.node_id == duplicated_node.node_id)
                    else {
                        return false;
                    };
                    self.nodes.remove(node_index);
                    true
                }
                ProjectCommandDirection::Redo => {
                    if self
                        .nodes
                        .iter()
                        .any(|node| node.node_id == duplicated_node.node_id)
                    {
                        return false;
                    }
                    let insert_index = (*insert_index).min(self.nodes.len());
                    self.nodes.insert(insert_index, (**duplicated_node).clone());
                    true
                }
            },
            ProjectCommand::SourceNodeCreate {
                source_node,
                insert_index,
            } => match direction {
                ProjectCommandDirection::Undo => {
                    let Some(node_index) = self
                        .nodes
                        .iter()
                        .position(|node| node.node_id == source_node.node_id)
                    else {
                        return false;
                    };
                    self.nodes.remove(node_index);
                    true
                }
                ProjectCommandDirection::Redo => {
                    if self
                        .nodes
                        .iter()
                        .any(|node| node.node_id == source_node.node_id)
                    {
                        return false;
                    }
                    let insert_index = (*insert_index).min(self.nodes.len());
                    self.nodes.insert(insert_index, (**source_node).clone());
                    true
                }
            },
            ProjectCommand::NodeDelete {
                deleted_node,
                remove_index,
                data_flow_edges_before,
                data_flow_edges_after,
            } => match direction {
                ProjectCommandDirection::Undo => {
                    if self
                        .nodes
                        .iter()
                        .any(|node| node.node_id == deleted_node.node_id)
                    {
                        return false;
                    }
                    let insert_index = (*remove_index).min(self.nodes.len());
                    self.nodes.insert(insert_index, (**deleted_node).clone());
                    self.data_flow_edges = data_flow_edges_before.clone();
                    true
                }
                ProjectCommandDirection::Redo => {
                    let Some(node_index) = self
                        .nodes
                        .iter()
                        .position(|node| node.node_id == deleted_node.node_id)
                    else {
                        return false;
                    };
                    self.nodes.remove(node_index);
                    self.data_flow_edges = data_flow_edges_after.clone();
                    true
                }
            },
            ProjectCommand::ReferenceInputCreate {
                reference_node,
                insert_index,
            } => match direction {
                ProjectCommandDirection::Undo => {
                    let Some(node_index) = self
                        .nodes
                        .iter()
                        .position(|node| node.node_id == reference_node.node_id)
                    else {
                        return false;
                    };
                    self.nodes.remove(node_index);
                    true
                }
                ProjectCommandDirection::Redo => {
                    if self
                        .nodes
                        .iter()
                        .any(|node| node.node_id == reference_node.node_id)
                    {
                        return false;
                    }
                    let insert_index = (*insert_index).min(self.nodes.len());
                    self.nodes.insert(insert_index, (**reference_node).clone());
                    true
                }
            },
            ProjectCommand::ReferenceTargetAdd {
                reference_node_id,
                added_entry,
                target_index,
                ..
            } => match direction {
                ProjectCommandDirection::Undo => {
                    self.apply_reference_target_remove(reference_node_id, &added_entry.target)
                }
                ProjectCommandDirection::Redo => {
                    let Some(node) = self
                        .nodes
                        .iter_mut()
                        .find(|node| node.node_id == *reference_node_id)
                    else {
                        return false;
                    };
                    let Some(reference_input) = node.reference_input.as_mut() else {
                        return false;
                    };
                    if reference_input
                        .targets
                        .iter()
                        .any(|entry| entry.target == added_entry.target)
                    {
                        return false;
                    }
                    let insert_index = (*target_index).min(reference_input.targets.len());
                    reference_input
                        .targets
                        .insert(insert_index, added_entry.clone());
                    node.evaluation.state = EvaluationState::Stale;
                    node.evaluation.message = Some("Reference target set changed.".to_owned());
                    true
                }
            },
            ProjectCommand::DataFlowEdgeAdd { edge, .. } => match direction {
                ProjectCommandDirection::Undo => {
                    self.remove_data_flow_edge_without_history(&edge.edge_id)
                }
                ProjectCommandDirection::Redo => {
                    self.add_data_flow_edge_without_history(edge.clone())
                }
            },
            ProjectCommand::DataFlowEdgeRemove { edge, .. } => match direction {
                ProjectCommandDirection::Undo => {
                    self.add_data_flow_edge_without_history(edge.clone())
                }
                ProjectCommandDirection::Redo => {
                    self.remove_data_flow_edge_without_history(&edge.edge_id)
                }
            },
            ProjectCommand::DataFlowEdgeInsertNode {
                removed_edge,
                added_edges,
                ..
            } => match direction {
                ProjectCommandDirection::Undo => {
                    for edge in added_edges.iter().rev() {
                        if !self.remove_data_flow_edge_without_history(&edge.edge_id) {
                            return false;
                        }
                    }
                    self.add_data_flow_edge_without_history(removed_edge.clone())
                }
                ProjectCommandDirection::Redo => {
                    if !self.remove_data_flow_edge_without_history(&removed_edge.edge_id) {
                        return false;
                    }
                    for edge in added_edges {
                        if !self.add_data_flow_edge_without_history(edge.clone()) {
                            return false;
                        }
                    }
                    true
                }
            },
            ProjectCommand::NodeParameterEdit {
                node_id,
                old_value,
                new_value,
                ..
            } => {
                let Some(node_index) = self.nodes.iter().position(|node| node.node_id == *node_id)
                else {
                    return false;
                };
                let value = match direction {
                    ProjectCommandDirection::Undo => *old_value,
                    ProjectCommandDirection::Redo => *new_value,
                };
                let Some(node) = self.nodes.get_mut(node_index) else {
                    return false;
                };
                node.parameter.value =
                    value.clamp(*node.parameter.range.start(), *node.parameter.range.end());
                self.mark_node_stale(node_index);
                true
            }
            ProjectCommand::NodeOutputParticipationEdit {
                node_id,
                old_participates,
                new_participates,
                ..
            } => {
                let Some(node) = self.nodes.iter_mut().find(|node| node.node_id == *node_id) else {
                    return false;
                };
                node.participates_in_output = match direction {
                    ProjectCommandDirection::Undo => *old_participates,
                    ProjectCommandDirection::Redo => *new_participates,
                };
                true
            }
            ProjectCommand::NodeCommentVisibilityEdit {
                node_id,
                old_show_comment,
                new_show_comment,
                ..
            } => {
                let Some(node) = self.nodes.iter_mut().find(|node| node.node_id == *node_id) else {
                    return false;
                };
                node.show_comment_in_network = match direction {
                    ProjectCommandDirection::Undo => *old_show_comment,
                    ProjectCommandDirection::Redo => *new_show_comment,
                };
                true
            }
            ProjectCommand::NodeManualCookEdit {
                node_id,
                old_manual,
                new_manual,
                ..
            } => {
                let Some(node) = self.nodes.iter_mut().find(|node| node.node_id == *node_id) else {
                    return false;
                };
                Self::apply_node_manual_state(
                    node,
                    match direction {
                        ProjectCommandDirection::Undo => *old_manual,
                        ProjectCommandDirection::Redo => *new_manual,
                    },
                );
                true
            }
            ProjectCommand::NodeLayoutEdit {
                node_id,
                old_position,
                new_position,
                network_box_changes,
                ..
            } => {
                let Some(node) = self.nodes.iter_mut().find(|node| node.node_id == *node_id) else {
                    return false;
                };
                node.layout_position = match direction {
                    ProjectCommandDirection::Undo => *old_position,
                    ProjectCommandDirection::Redo => *new_position,
                };
                for change in network_box_changes {
                    if let Some(annotation) = self
                        .annotations
                        .iter_mut()
                        .find(|annotation| annotation.annotation_id == change.annotation_id)
                    {
                        let snapshot = match direction {
                            ProjectCommandDirection::Undo => &change.old_state,
                            ProjectCommandDirection::Redo => &change.new_state,
                        };
                        snapshot.apply_to_annotation(annotation);
                    }
                }
                true
            }
            ProjectCommand::ReferenceTargetEnablementEdit {
                reference_node_id,
                target,
                old_enabled,
                new_enabled,
                ..
            } => self.apply_reference_target_enabled(
                reference_node_id,
                target,
                match direction {
                    ProjectCommandDirection::Undo => *old_enabled,
                    ProjectCommandDirection::Redo => *new_enabled,
                },
            ),
            ProjectCommand::ReferenceTargetRemove {
                reference_node_id,
                removed_entry,
                target_index,
                ..
            } => match direction {
                ProjectCommandDirection::Undo => {
                    let Some(node) = self
                        .nodes
                        .iter_mut()
                        .find(|node| node.node_id == *reference_node_id)
                    else {
                        return false;
                    };
                    let Some(reference_input) = node.reference_input.as_mut() else {
                        return false;
                    };
                    if reference_input
                        .targets
                        .iter()
                        .any(|entry| entry.target == removed_entry.target)
                    {
                        return false;
                    }
                    let insert_index = (*target_index).min(reference_input.targets.len());
                    reference_input
                        .targets
                        .insert(insert_index, removed_entry.clone());
                    node.evaluation.state = EvaluationState::Stale;
                    node.evaluation.message = Some("Reference target restored.".to_owned());
                    true
                }
                ProjectCommandDirection::Redo => {
                    self.apply_reference_target_remove(reference_node_id, &removed_entry.target)
                }
            },
            ProjectCommand::AnnotationMoveEdit {
                annotation_id,
                old_position,
                new_position,
                moved_nodes,
                ..
            } => {
                let Some(annotation) = self
                    .annotations
                    .iter_mut()
                    .find(|annotation| annotation.annotation_id == *annotation_id)
                else {
                    return false;
                };
                annotation.position = match direction {
                    ProjectCommandDirection::Undo => *old_position,
                    ProjectCommandDirection::Redo => *new_position,
                };
                for moved_node in moved_nodes {
                    if let Some(node) = self
                        .nodes
                        .iter_mut()
                        .find(|node| node.node_id == moved_node.node_id)
                    {
                        node.layout_position = match direction {
                            ProjectCommandDirection::Undo => moved_node.old_position,
                            ProjectCommandDirection::Redo => moved_node.new_position,
                        };
                    }
                }
                true
            }
            ProjectCommand::AnnotationCreate {
                annotation,
                insert_index,
            } => {
                match direction {
                    ProjectCommandDirection::Undo => {
                        let Some(annotation_index) = self.annotations.iter().position(|existing| {
                            existing.annotation_id == annotation.annotation_id
                        }) else {
                            return false;
                        };
                        self.annotations.remove(annotation_index);
                        true
                    }
                    ProjectCommandDirection::Redo => {
                        if self
                            .annotations
                            .iter()
                            .any(|existing| existing.annotation_id == annotation.annotation_id)
                        {
                            return false;
                        }
                        let insert_index = (*insert_index).min(self.annotations.len());
                        self.annotations.insert(insert_index, annotation.clone());
                        true
                    }
                }
            }
            ProjectCommand::AnnotationDelete {
                annotation,
                remove_index,
            } => {
                match direction {
                    ProjectCommandDirection::Undo => {
                        if self
                            .annotations
                            .iter()
                            .any(|existing| existing.annotation_id == annotation.annotation_id)
                        {
                            return false;
                        }
                        let insert_index = (*remove_index).min(self.annotations.len());
                        self.annotations.insert(insert_index, annotation.clone());
                        true
                    }
                    ProjectCommandDirection::Redo => {
                        let Some(annotation_index) = self.annotations.iter().position(|existing| {
                            existing.annotation_id == annotation.annotation_id
                        }) else {
                            return false;
                        };
                        self.annotations.remove(annotation_index);
                        true
                    }
                }
            }
            ProjectCommand::AnnotationResizeEdit {
                annotation_id,
                old_size,
                new_size,
                ..
            } => {
                let Some(annotation) = self
                    .annotations
                    .iter_mut()
                    .find(|annotation| annotation.annotation_id == *annotation_id)
                else {
                    return false;
                };
                annotation.size = match direction {
                    ProjectCommandDirection::Undo => *old_size,
                    ProjectCommandDirection::Redo => *new_size,
                };
                true
            }
            ProjectCommand::AnnotationBoundsEdit {
                annotation_id,
                old_position,
                new_position,
                old_size,
                new_size,
                ..
            } => {
                let Some(annotation) = self
                    .annotations
                    .iter_mut()
                    .find(|annotation| annotation.annotation_id == *annotation_id)
                else {
                    return false;
                };
                annotation.position = match direction {
                    ProjectCommandDirection::Undo => *old_position,
                    ProjectCommandDirection::Redo => *new_position,
                };
                annotation.size = match direction {
                    ProjectCommandDirection::Undo => *old_size,
                    ProjectCommandDirection::Redo => *new_size,
                };
                true
            }
            ProjectCommand::AnnotationTitleEdit {
                annotation_id,
                old_title,
                new_title,
                ..
            } => {
                let Some(annotation) = self
                    .annotations
                    .iter_mut()
                    .find(|annotation| annotation.annotation_id == *annotation_id)
                else {
                    return false;
                };
                annotation.title = match direction {
                    ProjectCommandDirection::Undo => old_title.clone(),
                    ProjectCommandDirection::Redo => new_title.clone(),
                };
                true
            }
            ProjectCommand::AnnotationTextEdit {
                annotation_id,
                old_text,
                new_text,
                ..
            } => {
                let Some(annotation) = self
                    .annotations
                    .iter_mut()
                    .find(|annotation| annotation.annotation_id == *annotation_id)
                else {
                    return false;
                };
                annotation.text = match direction {
                    ProjectCommandDirection::Undo => old_text.clone(),
                    ProjectCommandDirection::Redo => new_text.clone(),
                };
                true
            }
            ProjectCommand::AnnotationCollapsedEdit {
                annotation_id,
                old_collapsed,
                new_collapsed,
                ..
            } => {
                let Some(annotation) = self
                    .annotations
                    .iter_mut()
                    .find(|annotation| annotation.annotation_id == *annotation_id)
                else {
                    return false;
                };
                annotation.collapsed = match direction {
                    ProjectCommandDirection::Undo => *old_collapsed,
                    ProjectCommandDirection::Redo => *new_collapsed,
                };
                true
            }
            ProjectCommand::AnnotationsCollapsedEdit { annotations, .. } => {
                for collapsed_annotation in annotations {
                    if let Some(annotation) = self.annotations.iter_mut().find(|annotation| {
                        annotation.annotation_id == collapsed_annotation.annotation_id
                    }) {
                        annotation.collapsed = match direction {
                            ProjectCommandDirection::Undo => collapsed_annotation.old_collapsed,
                            ProjectCommandDirection::Redo => collapsed_annotation.new_collapsed,
                        };
                    }
                }
                true
            }
            ProjectCommand::LayerVisibilityEdit {
                layer_index,
                old_visible,
                new_visible,
                ..
            } => {
                let visible = match direction {
                    ProjectCommandDirection::Undo => *old_visible,
                    ProjectCommandDirection::Redo => *new_visible,
                };
                let Some(layer) = self.layers.get_mut(*layer_index) else {
                    return false;
                };
                layer.visible = visible;
                true
            }
            ProjectCommand::LayerOrderEdit {
                layer_index,
                old_order,
                new_order,
                ..
            } => {
                let order = match direction {
                    ProjectCommandDirection::Undo => *old_order,
                    ProjectCommandDirection::Redo => *new_order,
                };
                let Some(layer) = self.layers.get_mut(*layer_index) else {
                    return false;
                };
                layer.order = order;
                true
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_output_operator_for_node(
        &mut self,
        node_index: usize,
        output_operator: OutputOperatorNode,
    ) -> bool {
        let Some(node) = self.nodes.get_mut(node_index) else {
            return false;
        };
        if node.kind != NodeKind::Output {
            return false;
        }
        if node.output_operator.as_ref() == Some(&output_operator) {
            return false;
        }
        node.output_operator = Some(output_operator);
        self.adopt_generated_node_for_structural_edit(node_index);
        true
    }

    pub fn set_generated_node_binding_state(
        &mut self,
        node_index: usize,
        binding_state: GeneratedNodeBindingState,
    ) -> bool {
        let Some(generated) = self
            .nodes
            .get_mut(node_index)
            .and_then(|node| node.generated.as_mut())
        else {
            return false;
        };
        if generated.binding_state == binding_state {
            return false;
        }
        generated.binding_state = binding_state;
        true
    }

    #[allow(dead_code)]
    pub fn adopt_generated_node_for_structural_edit(&mut self, node_index: usize) -> bool {
        let Some(generated) = self
            .nodes
            .get_mut(node_index)
            .and_then(|node| node.generated.as_mut())
        else {
            return false;
        };
        if generated.binding_state != GeneratedNodeBindingState::Managed {
            return false;
        }
        generated.binding_state = GeneratedNodeBindingState::Adopted;
        true
    }

    fn node_accepts_layer_managed_update(&self, node_index: usize) -> bool {
        self.nodes
            .get(node_index)
            .is_some_and(|node| match node.generated {
                None => true,
                Some(generated) => generated.binding_state == GeneratedNodeBindingState::Managed,
            })
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
        name = self.unique_node_name_in_graph(&name, self.current_graph_id(), None);

        let insert_index = self
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .unwrap_or(self.nodes.len());
        let mut node = GraphNode::null_operator(name);
        node.node_id = self.unique_node_id("null");
        node.parent_graph_id = self.current_graph_id().to_owned();
        node.layout_position = GraphPoint::new(0.82, 0.5);
        self.nodes.insert(insert_index, node);
        self.rebuild_default_data_flow_edges();
        insert_index
    }

    pub fn add_source_gallery_item_node(&mut self, item: &SourceGalleryItem) -> usize {
        let name = self.unique_node_name_in_graph(
            &format!("Source {}", item.display_name),
            self.current_graph_id(),
            None,
        );
        let insert_index = self
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .unwrap_or(self.nodes.len());
        let mut node = GraphNode::source_node(
            self.unique_node_id("source"),
            name,
            SourceNode::from_gallery_item(item),
        );
        node.parent_graph_id = self.current_graph_id().to_owned();
        node.layout_position = GraphPoint::new(0.12, 0.7);
        let source_node = node.clone();
        self.nodes.insert(insert_index, node);
        self.record_project_command(ProjectCommand::SourceNodeCreate {
            source_node: Box::new(source_node),
            insert_index,
        });
        insert_index
    }

    pub fn add_source_gallery_collection_node(
        &mut self,
        items: &[SourceGalleryItem],
    ) -> Option<usize> {
        let source_node = SourceNode::from_gallery_items(items)?;
        let name = self.unique_node_name_in_graph(
            &format!("Source Collection {}", source_node.source_count()),
            self.current_graph_id(),
            None,
        );
        let insert_index = self
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .unwrap_or(self.nodes.len());
        let mut node =
            GraphNode::source_node(self.unique_node_id("source_collection"), name, source_node);
        node.parent_graph_id = self.current_graph_id().to_owned();
        node.layout_position = GraphPoint::new(0.12, 0.82);
        let created_node = node.clone();
        self.nodes.insert(insert_index, node);
        self.record_project_command(ProjectCommand::SourceNodeCreate {
            source_node: Box::new(created_node),
            insert_index,
        });
        Some(insert_index)
    }

    #[allow(dead_code)]
    fn unique_node_name(&self, candidate: &str) -> String {
        self.unique_node_name_in_graph(candidate, self.current_graph_id(), None)
    }

    fn unique_node_name_in_graph(
        &self,
        candidate: &str,
        parent_graph_id: &str,
        ignored_node_index: Option<usize>,
    ) -> String {
        if !self.node_name_exists_in_graph(candidate, parent_graph_id, ignored_node_index) {
            return candidate.to_owned();
        }

        let mut suffix = 2;
        loop {
            let name = format!("{candidate}_{suffix}");
            if !self.node_name_exists_in_graph(&name, parent_graph_id, ignored_node_index) {
                return name;
            }
            suffix += 1;
        }
    }

    fn node_name_exists_in_graph(
        &self,
        name: &str,
        parent_graph_id: &str,
        ignored_node_index: Option<usize>,
    ) -> bool {
        self.nodes.iter().enumerate().any(|(index, node)| {
            Some(index) != ignored_node_index
                && self.node_parent_graph_id(node) == parent_graph_id
                && node.name == name
        })
    }

    pub fn set_node_name(&mut self, node_index: usize, candidate: impl Into<String>) -> bool {
        let Some(command) = self.set_node_name_without_history(node_index, candidate) else {
            return false;
        };
        self.record_project_command(command);
        true
    }

    fn set_node_name_without_history(
        &mut self,
        node_index: usize,
        candidate: impl Into<String>,
    ) -> Option<ProjectCommand> {
        let candidate = candidate.into().trim().to_owned();
        if candidate.is_empty() {
            return None;
        }
        let Some(current_name) = self.nodes.get(node_index).map(|node| node.name.clone()) else {
            return None;
        };
        if current_name == candidate {
            return None;
        }

        let parent_graph_id = self
            .nodes
            .get(node_index)
            .map(|node| self.node_parent_graph_id(node).to_owned())?;
        let name = self.unique_node_name_in_graph(&candidate, &parent_graph_id, Some(node_index));

        if let Some(node) = self.nodes.get_mut(node_index) {
            node.name = name.clone();
            Some(ProjectCommand::NodeRename {
                node_id: node.node_id.clone(),
                old_name: current_name,
                new_name: name,
            })
        } else {
            None
        }
    }

    pub fn set_node_output_participation(&mut self, node_index: usize, participates: bool) -> bool {
        let Some(command) =
            self.set_node_output_participation_without_history(node_index, participates)
        else {
            return false;
        };
        self.record_project_command(command);
        true
    }

    fn set_node_output_participation_without_history(
        &mut self,
        node_index: usize,
        participates: bool,
    ) -> Option<ProjectCommand> {
        let node = self.nodes.get_mut(node_index)?;
        if node.participates_in_output == participates {
            return None;
        }
        let old_participates = node.participates_in_output;
        node.participates_in_output = participates;
        let node_id = node.node_id.clone();
        let node_name = node.name.clone();
        self.rebuild_default_data_flow_edges();
        Some(ProjectCommand::NodeOutputParticipationEdit {
            node_id,
            node_name,
            old_participates,
            new_participates: participates,
        })
    }

    pub fn set_node_comment_visibility(&mut self, node_index: usize, show_comment: bool) -> bool {
        let Some(command) =
            self.set_node_comment_visibility_without_history(node_index, show_comment)
        else {
            return false;
        };
        self.record_project_command(command);
        true
    }

    fn set_node_comment_visibility_without_history(
        &mut self,
        node_index: usize,
        show_comment: bool,
    ) -> Option<ProjectCommand> {
        let node = self.nodes.get_mut(node_index)?;
        if node.show_comment_in_network == show_comment {
            return None;
        }
        let old_show_comment = node.show_comment_in_network;
        node.show_comment_in_network = show_comment;
        Some(ProjectCommand::NodeCommentVisibilityEdit {
            node_id: node.node_id.clone(),
            node_name: node.name.clone(),
            old_show_comment,
            new_show_comment: show_comment,
        })
    }

    pub fn duplicate_node(&mut self, node_index: usize) -> Option<usize> {
        let source = self.nodes.get(node_index)?;
        let source_node_id = source.node_id.clone();
        let source_name = source.name.clone();
        let mut node = source.clone();

        node.node_id = self.unique_node_id(node.kind.duplicate_node_id_prefix());
        let parent_graph_id = self.node_parent_graph_id(source).to_owned();
        node.name = self.unique_node_name_in_graph(&source_name, &parent_graph_id, None);
        node.parent_graph_id = parent_graph_id;
        node.layout_position = GraphPoint::new(
            source.layout_position.x + 0.12,
            source.layout_position.y + 0.08,
        );
        node.generated = None;
        node.output_operator = None;
        node.evaluation = NodeEvaluation::clean();
        node.participates_in_output = false;

        if let Some(reference_input) = node.reference_input.as_mut() {
            reference_input.targets.clear();
        }
        if let Some(python_operator) = node.python_operator.as_mut() {
            python_operator.instance_id = node.node_id.clone();
            python_operator.cache_key = None;
            python_operator.provenance = None;
            python_operator.provenance_summary = None;
            python_operator.last_failure_summary = None;
        }
        if let Some(procedural_asset) = node.procedural_asset.as_mut() {
            procedural_asset.instance_id = node.node_id.clone();
            procedural_asset.output_summary = None;
        }
        if let Some(native_operator) = node.native_operator.as_mut() {
            native_operator.instance_id = node.node_id.clone();
            native_operator.cache_key = None;
            native_operator.provenance = None;
            native_operator.provenance_summary = None;
            native_operator.last_valid_cache_key = None;
            native_operator.last_failure_summary = None;
        }

        let insert_index = (node_index + 1).min(self.nodes.len());
        let duplicated_node = node.clone();
        self.nodes.insert(insert_index, node);
        self.rebuild_default_data_flow_edges();
        self.record_project_command(ProjectCommand::NodeDuplicate {
            source_node_id,
            source_node_name: source_name,
            duplicated_node: Box::new(duplicated_node),
            insert_index,
        });
        Some(insert_index)
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
        node.parent_graph_id = self.current_graph_id().to_owned();
        node.layout_position = GraphPoint::new(0.88, 0.5);
        let reference_node = node.clone();
        self.nodes.insert(insert_index, node);
        self.rebuild_default_data_flow_edges();
        self.record_project_command(ProjectCommand::ReferenceInputCreate {
            reference_node: Box::new(reference_node),
            insert_index,
        });
        Some(insert_index)
    }

    #[allow(dead_code)]
    pub fn add_reference_target_to_node(
        &mut self,
        reference_node_index: usize,
        target_node_index: usize,
    ) -> bool {
        let Some(command) = self
            .add_reference_target_to_node_without_history(reference_node_index, target_node_index)
        else {
            return false;
        };
        self.record_project_command(command);
        true
    }

    fn add_reference_target_to_node_without_history(
        &mut self,
        reference_node_index: usize,
        target_node_index: usize,
    ) -> Option<ProjectCommand> {
        let Some(target) = self.reference_target_for_node(target_node_index) else {
            return None;
        };
        let Some(source_node) = self.nodes.get(target_node_index) else {
            return None;
        };
        let provenance = ReferenceTargetProvenance::from_node(source_node, &target);
        let reference_node = self.nodes.get_mut(reference_node_index)?;
        let reference_input = reference_node.reference_input.as_mut()?;
        if reference_input
            .targets
            .iter()
            .any(|entry| entry.target == target)
        {
            return None;
        }

        let target_index = reference_input.targets.len();
        let added_entry = ReferenceTargetEntry {
            target,
            enabled: true,
            provenance,
        };
        reference_input.targets.push(added_entry.clone());
        reference_node.evaluation.state = EvaluationState::Stale;
        reference_node.evaluation.message = Some("Reference target set changed.".to_owned());
        Some(ProjectCommand::ReferenceTargetAdd {
            reference_node_id: reference_node.node_id.clone(),
            reference_node_name: reference_node.name.clone(),
            added_entry,
            target_index,
        })
    }

    #[allow(dead_code)]
    pub fn set_reference_target_enabled(
        &mut self,
        reference_node_index: usize,
        target_node_id: &str,
        enabled: bool,
    ) -> bool {
        let Some(command) = self.set_reference_target_enabled_without_history(
            reference_node_index,
            target_node_id,
            enabled,
        ) else {
            return false;
        };
        self.record_project_command(command);
        true
    }

    fn set_reference_target_enabled_without_history(
        &mut self,
        reference_node_index: usize,
        target_node_id: &str,
        enabled: bool,
    ) -> Option<ProjectCommand> {
        let node = self.nodes.get_mut(reference_node_index)?;
        let reference_input = node.reference_input.as_mut()?;
        let entry = reference_input
            .targets
            .iter_mut()
            .find(|entry| entry.target.node_id == target_node_id)?;
        if entry.enabled == enabled {
            return None;
        }

        let old_enabled = entry.enabled;
        let target = entry.target.clone();
        let target_node_name = entry.provenance.source_node_name.clone();
        entry.enabled = enabled;
        node.evaluation.state = EvaluationState::Stale;
        node.evaluation.message = Some("Reference target enablement changed.".to_owned());
        Some(ProjectCommand::ReferenceTargetEnablementEdit {
            reference_node_id: node.node_id.clone(),
            reference_node_name: node.name.clone(),
            target,
            target_node_name,
            old_enabled,
            new_enabled: enabled,
        })
    }

    #[allow(dead_code)]
    pub fn remove_reference_target_from_node(
        &mut self,
        reference_node_index: usize,
        target_node_id: &str,
    ) -> bool {
        let Some(command) = self.remove_reference_target_from_node_without_history(
            reference_node_index,
            target_node_id,
        ) else {
            return false;
        };
        self.record_project_command(command);
        true
    }

    fn remove_reference_target_from_node_without_history(
        &mut self,
        reference_node_index: usize,
        target_node_id: &str,
    ) -> Option<ProjectCommand> {
        let node = self.nodes.get_mut(reference_node_index)?;
        let reference_input = node.reference_input.as_mut()?;
        let target_index = reference_input
            .targets
            .iter()
            .position(|entry| entry.target.node_id == target_node_id)?;
        let removed_entry = reference_input.targets.remove(target_index);
        node.evaluation.state = EvaluationState::Stale;
        node.evaluation.message = Some("Reference target removed.".to_owned());
        Some(ProjectCommand::ReferenceTargetRemove {
            reference_node_id: node.node_id.clone(),
            reference_node_name: node.name.clone(),
            removed_entry,
            target_index,
        })
    }

    fn apply_reference_target_enabled(
        &mut self,
        reference_node_id: &str,
        target: &ReferenceTargetIdentity,
        enabled: bool,
    ) -> bool {
        let Some(node) = self
            .nodes
            .iter_mut()
            .find(|node| node.node_id == reference_node_id)
        else {
            return false;
        };
        let Some(reference_input) = node.reference_input.as_mut() else {
            return false;
        };
        let Some(entry) = reference_input
            .targets
            .iter_mut()
            .find(|entry| entry.target == *target)
        else {
            return false;
        };
        entry.enabled = enabled;
        node.evaluation.state = EvaluationState::Stale;
        node.evaluation.message = Some("Reference target enablement changed.".to_owned());
        true
    }

    fn apply_reference_target_remove(
        &mut self,
        reference_node_id: &str,
        target: &ReferenceTargetIdentity,
    ) -> bool {
        let Some(node) = self
            .nodes
            .iter_mut()
            .find(|node| node.node_id == reference_node_id)
        else {
            return false;
        };
        let Some(reference_input) = node.reference_input.as_mut() else {
            return false;
        };
        let Some(target_index) = reference_input
            .targets
            .iter()
            .position(|entry| entry.target == *target)
        else {
            return false;
        };
        reference_input.targets.remove(target_index);
        node.evaluation.state = EvaluationState::Stale;
        node.evaluation.message = Some("Reference target removed.".to_owned());
        true
    }

    pub fn reference_target_for_node(
        &self,
        target_node_index: usize,
    ) -> Option<ReferenceTargetIdentity> {
        let node = self.nodes.get(target_node_index)?;
        let output_name = self.node_primary_output_name(node)?;
        Some(ReferenceTargetIdentity {
            graph_id: self.node_parent_graph_id(node).to_owned(),
            node_id: node.node_id.clone(),
            output_name,
        })
    }

    fn node_primary_output_name(&self, node: &GraphNode) -> Option<String> {
        if node.kind == NodeKind::GraphContainer {
            return self
                .graph_container_metadata_for_node(&node.node_id)?
                .boundary
                .primary_output()
                .map(|port| port.name.clone());
        }
        self.node_primary_output_kind(node.kind)?;
        Some(PRIMARY_GEOMETRY_OUTPUT.to_owned())
    }

    fn node_output_kind_for_name(
        &self,
        node: &GraphNode,
        output_name: &str,
    ) -> Option<HoudiniDataKind> {
        if node.kind == NodeKind::GraphContainer {
            return self
                .graph_container_metadata_for_node(&node.node_id)?
                .boundary
                .output_kind(output_name);
        }
        (output_name == PRIMARY_GEOMETRY_OUTPUT)
            .then(|| self.node_primary_output_kind(node.kind))
            .flatten()
    }

    fn node_input_kind_for_name(
        &self,
        node: &GraphNode,
        input_name: &str,
    ) -> Option<HoudiniDataKind> {
        if node.kind == NodeKind::GraphContainer {
            return self
                .graph_container_metadata_for_node(&node.node_id)?
                .boundary
                .inputs
                .iter()
                .find(|port| port.name == input_name)
                .map(|port| port.data_kind);
        }
        (input_name == PRIMARY_GEOMETRY_OUTPUT).then_some(HoudiniDataKind::GeometryTable)
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
            NodeKind::ReferenceInput | NodeKind::GraphContainer | NodeKind::Output => None,
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

        let Some((target_node_index, target_node)) =
            self.nodes.iter().enumerate().find(|(_, node)| {
                node.node_id == target.node_id && self.node_parent_graph_id(node) == target.graph_id
            })
        else {
            if self.graph_registry.graph(&target.graph_id).is_none()
                && target.graph_id != MAIN_GRAPH_ID
            {
                return ReferenceTargetResolution::diagnostic(
                    target,
                    ReferenceDiagnosticStatus::DisallowedBoundary,
                    "Reference target is outside the current project graph.",
                );
            }
            return ReferenceTargetResolution::diagnostic(
                target,
                ReferenceDiagnosticStatus::MissingNode,
                "Reference target node is missing.",
            );
        };

        let Some(output_kind) = self.node_output_kind_for_name(target_node, &target.output_name)
        else {
            return ReferenceTargetResolution::diagnostic(
                target,
                ReferenceDiagnosticStatus::MissingOutput,
                "Reference target node does not expose a compatible geometry output.",
            );
        };

        ReferenceTargetResolution {
            target: target.clone(),
            status: ReferenceDiagnosticStatus::Resolved,
            readable_path: readable_reference_path(
                &target.graph_id,
                target_node,
                &target.output_name,
            ),
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
        let target_parent_graph_id = self.node_parent_graph_id(target_node).to_owned();
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.reference_input.is_some())
            .flat_map(|(reference_node_index, reference_node)| {
                let target_parent_graph_id = target_parent_graph_id.clone();
                self.reference_input_resolutions(reference_node_index)
                    .unwrap_or_default()
                    .into_iter()
                    .filter(move |entry| {
                        entry.resolution.target.graph_id == target_parent_graph_id
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
            target_node_path: self.readable_node_path_for_node(target_node),
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
            graph_id: self.current_graph_id().to_owned(),
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
        projection_node.parent_graph_id = self.current_graph_id().to_owned();
        let source_position = self
            .nodes
            .get(source_node_index)
            .map(|node| node.layout_position)
            .unwrap_or(GraphPoint::new(0.5, 0.5));
        projection_node.layout_position =
            GraphPoint::new(source_position.x + 0.08, source_position.y + 0.12);

        let insert_index = reference_node_index.min(self.nodes.len());
        self.nodes.insert(insert_index, projection_node);
        self.rebuild_default_data_flow_edges();
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
        let data_flow_edges_before = self.data_flow_edges.clone();
        let deleted_node = self.nodes.remove(index);
        self.data_flow_edges.retain(|edge| {
            edge.from_node_id != deleted_node.node_id && edge.to_node_id != deleted_node.node_id
        });
        let data_flow_edges_after = self.data_flow_edges.clone();
        self.record_project_command(ProjectCommand::NodeDelete {
            deleted_node: Box::new(deleted_node.clone()),
            remove_index: index,
            data_flow_edges_before,
            data_flow_edges_after,
        });
        Some(deleted_node)
    }

    #[allow(dead_code)]
    pub fn remove_node_reconnecting_data_flow(
        &mut self,
        index: usize,
    ) -> Option<ReconnectNodeDeleteResult> {
        if self.nodes.get(index)?.kind == NodeKind::Output {
            return None;
        }
        let data_flow_edges_before = self.data_flow_edges.clone();
        let deleted_node_id = self.nodes.get(index)?.node_id.clone();
        let incoming_edges = data_flow_edges_before
            .iter()
            .filter(|edge| edge.to_node_id == deleted_node_id)
            .cloned()
            .collect::<Vec<_>>();
        let outgoing_edges = data_flow_edges_before
            .iter()
            .filter(|edge| edge.from_node_id == deleted_node_id)
            .cloned()
            .collect::<Vec<_>>();

        let deleted_node = self.nodes.remove(index);
        self.data_flow_edges.retain(|edge| {
            edge.from_node_id != deleted_node.node_id && edge.to_node_id != deleted_node.node_id
        });

        let mut added_edges = Vec::new();
        let mut skipped_diagnostics = Vec::new();
        for incoming_edge in &incoming_edges {
            for outgoing_edge in &outgoing_edges {
                let edge = GraphDataFlowEdge {
                    edge_id: Self::data_flow_edge_id(
                        &incoming_edge.from_node_id,
                        &incoming_edge.from_output,
                        &outgoing_edge.to_node_id,
                        &outgoing_edge.to_input,
                    ),
                    from_node_id: incoming_edge.from_node_id.clone(),
                    from_output: incoming_edge.from_output.clone(),
                    to_node_id: outgoing_edge.to_node_id.clone(),
                    to_input: outgoing_edge.to_input.clone(),
                };
                if let Some(diagnostic) = self.data_flow_edge_addition_diagnostic(&edge) {
                    skipped_diagnostics.push(diagnostic);
                    continue;
                }
                if self.add_data_flow_edge_without_history(edge.clone()) {
                    added_edges.push(edge);
                }
            }
        }

        let data_flow_edges_after = self.data_flow_edges.clone();
        self.record_project_command(ProjectCommand::NodeDelete {
            deleted_node: Box::new(deleted_node.clone()),
            remove_index: index,
            data_flow_edges_before,
            data_flow_edges_after,
        });
        Some(ReconnectNodeDeleteResult {
            deleted_node,
            added_edges,
            skipped_diagnostics,
        })
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
            NodeKind::ReferenceInput | NodeKind::GraphContainer => 0,
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
        let mut node = GraphNode::python_operator(instance_id, declaration_id);
        node.parent_graph_id = self.current_graph_id().to_owned();
        self.nodes.insert(insert_index, node);
        self.rebuild_default_data_flow_edges();
        insert_index
    }

    #[allow(dead_code)]
    pub fn add_graph_container_node(
        &mut self,
        name: impl Into<String>,
        internal_graph: ProjectGraphMetadata,
    ) -> usize {
        let internal_graph_id = internal_graph.graph_id.clone();
        if self.graph_registry.graph(&internal_graph_id).is_none() {
            self.graph_registry.graphs.push(internal_graph);
        }

        let instance_id = self.unique_node_id("graph_container");
        let insert_index = self
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .unwrap_or(self.nodes.len());
        let parent_graph_id = self.current_graph_id().to_owned();
        let mut node = GraphNode::graph_container(
            instance_id.clone(),
            self.unique_node_name_in_graph(&name.into(), &parent_graph_id, None),
        );
        node.parent_graph_id = parent_graph_id;
        node.layout_position = GraphPoint::new(0.74, 0.24);
        self.nodes.insert(insert_index, node);
        self.graph_containers.push(GraphContainerMetadata {
            container_node_id: instance_id,
            internal_graph_id,
            kind: GraphContainerKind::Subnet,
            boundary: GraphBoundaryDeclaration::geometry_passthrough(),
            collapse_manifest: None,
            navigable: true,
        });
        self.rebuild_default_data_flow_edges();
        insert_index
    }

    #[allow(dead_code)]
    pub fn add_graph_container_collapse_manifest_for_node_set(
        &mut self,
        name: impl Into<String>,
        node_indices: &[usize],
    ) -> Result<usize, GraphContainerCollapseError> {
        let captured_node_ids = self.captured_node_ids_for_indices(node_indices)?;
        if !self.node_set_is_connected(&captured_node_ids) {
            return Err(GraphContainerCollapseError::DisconnectedSelection);
        }

        let source_graph_id = self.current_graph_id().to_owned();
        let source_graph_path = self.current_graph_path().to_owned();
        let name = name.into();
        let internal_graph = ProjectGraphMetadata {
            graph_id: self.unique_graph_id(&name),
            name: name.clone(),
            path: format!(
                "{}/{}",
                source_graph_path.trim_end_matches('/'),
                sanitize_asset_id_part(&name)
            ),
            role: ProjectGraphRole::Subgraph,
        };
        let internal_graph_id = internal_graph.graph_id.clone();
        let original_data_flow_edges = self.data_flow_edges.clone();
        let (boundary, external_edges) = self.collapse_boundary_for_node_set(&captured_node_ids)?;
        let selection_center = self.node_set_layout_center(&captured_node_ids);

        let container_index = self.add_graph_container_node(name, internal_graph);
        let container_node_id = self.nodes[container_index].node_id.clone();
        if let Some(center) = selection_center
            && let Some(node) = self.nodes.get_mut(container_index)
        {
            node.layout_position = center;
        }
        let Some(container) = self
            .graph_containers
            .iter_mut()
            .find(|container| container.container_node_id == self.nodes[container_index].node_id)
        else {
            return Ok(container_index);
        };
        let captured_node_ids_for_move = captured_node_ids.clone();
        let external_edges_for_rewire = external_edges.clone();
        container.boundary = boundary;
        container.collapse_manifest = Some(GraphContainerCollapseManifest {
            source_graph_id,
            captured_node_ids,
            external_edges,
        });
        self.move_nodes_to_graph(&captured_node_ids_for_move, &internal_graph_id);
        self.data_flow_edges = Self::rewired_collapse_edges(
            &original_data_flow_edges,
            &container_node_id,
            &external_edges_for_rewire,
        );
        Ok(container_index)
    }

    fn move_nodes_to_graph(&mut self, node_ids: &[String], graph_id: &str) {
        let node_ids = node_ids
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        for node in &mut self.nodes {
            if node_ids.contains(node.node_id.as_str()) {
                node.parent_graph_id = graph_id.to_owned();
            }
        }
    }

    fn rewired_collapse_edges(
        original_edges: &[GraphDataFlowEdge],
        container_node_id: &str,
        external_edges: &[GraphContainerExternalEdge],
    ) -> Vec<GraphDataFlowEdge> {
        let external_edges_by_id = external_edges
            .iter()
            .map(|edge| (edge.edge_id.as_str(), edge))
            .collect::<std::collections::BTreeMap<_, _>>();

        original_edges
            .iter()
            .map(|edge| {
                let Some(external_edge) = external_edges_by_id.get(edge.edge_id.as_str()) else {
                    return edge.clone();
                };
                match external_edge.direction {
                    GraphBoundaryMappingDirection::Input => GraphDataFlowEdge {
                        edge_id: Self::data_flow_edge_id(
                            &external_edge.external_node_id,
                            &external_edge.external_port_name,
                            container_node_id,
                            &external_edge.public_port_name,
                        ),
                        from_node_id: external_edge.external_node_id.clone(),
                        from_output: external_edge.external_port_name.clone(),
                        to_node_id: container_node_id.to_owned(),
                        to_input: external_edge.public_port_name.clone(),
                    },
                    GraphBoundaryMappingDirection::Output => GraphDataFlowEdge {
                        edge_id: Self::data_flow_edge_id(
                            container_node_id,
                            &external_edge.public_port_name,
                            &external_edge.external_node_id,
                            &external_edge.external_port_name,
                        ),
                        from_node_id: container_node_id.to_owned(),
                        from_output: external_edge.public_port_name.clone(),
                        to_node_id: external_edge.external_node_id.clone(),
                        to_input: external_edge.external_port_name.clone(),
                    },
                }
            })
            .collect()
    }

    fn captured_node_ids_for_indices(
        &self,
        node_indices: &[usize],
    ) -> Result<Vec<String>, GraphContainerCollapseError> {
        if node_indices.is_empty() {
            return Err(GraphContainerCollapseError::EmptySelection);
        }
        let mut captured_node_ids = Vec::new();
        for &node_index in node_indices {
            let Some(node) = self.nodes.get(node_index) else {
                return Err(GraphContainerCollapseError::MissingNodeIndex(node_index));
            };
            if !captured_node_ids
                .iter()
                .any(|node_id| node_id == &node.node_id)
            {
                captured_node_ids.push(node.node_id.clone());
            }
        }
        if captured_node_ids.is_empty() {
            return Err(GraphContainerCollapseError::EmptySelection);
        }
        Ok(captured_node_ids)
    }

    fn node_set_is_connected(&self, node_ids: &[String]) -> bool {
        if node_ids.len() <= 1 {
            return true;
        }
        let selected = node_ids
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        let mut stack = vec![node_ids[0].as_str()];
        let mut visited = std::collections::BTreeSet::new();
        while let Some(node_id) = stack.pop() {
            if !visited.insert(node_id) {
                continue;
            }
            for edge in &self.data_flow_edges {
                let from_selected = selected.contains(edge.from_node_id.as_str());
                let to_selected = selected.contains(edge.to_node_id.as_str());
                if !(from_selected && to_selected) {
                    continue;
                }
                if edge.from_node_id == node_id {
                    stack.push(edge.to_node_id.as_str());
                } else if edge.to_node_id == node_id {
                    stack.push(edge.from_node_id.as_str());
                }
            }
        }
        visited.len() == selected.len()
    }

    fn collapse_boundary_for_node_set(
        &self,
        captured_node_ids: &[String],
    ) -> Result<
        (GraphBoundaryDeclaration, Vec<GraphContainerExternalEdge>),
        GraphContainerCollapseError,
    > {
        let captured = captured_node_ids
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut mappings = Vec::new();
        let mut external_edges = Vec::new();
        let mut input_names = Vec::new();
        let mut output_names = Vec::new();

        for edge in &self.data_flow_edges {
            let from_captured = captured.contains(edge.from_node_id.as_str());
            let to_captured = captured.contains(edge.to_node_id.as_str());
            match (from_captured, to_captured) {
                (false, true) => {
                    let data_kind = self.data_kind_for_edge(edge).ok_or_else(|| {
                        GraphContainerCollapseError::UntypedExternalEdge(edge.edge_id.clone())
                    })?;
                    let public_port_name =
                        unique_boundary_port_name(&edge.to_input, &mut input_names);
                    inputs.push(HoudiniOperatorPort {
                        name: public_port_name.clone(),
                        data_kind,
                        required: true,
                        help: format!("Collapse input for external edge {}.", edge.edge_id),
                    });
                    mappings.push(GraphBoundaryMapping {
                        direction: GraphBoundaryMappingDirection::Input,
                        public_port_name: public_port_name.clone(),
                        internal_node_id: edge.to_node_id.clone(),
                        internal_port_name: edge.to_input.clone(),
                    });
                    external_edges.push(GraphContainerExternalEdge {
                        direction: GraphBoundaryMappingDirection::Input,
                        edge_id: edge.edge_id.clone(),
                        external_node_id: edge.from_node_id.clone(),
                        external_port_name: edge.from_output.clone(),
                        internal_node_id: edge.to_node_id.clone(),
                        internal_port_name: edge.to_input.clone(),
                        public_port_name,
                        data_kind,
                    });
                }
                (true, false) => {
                    let data_kind = self.data_kind_for_edge(edge).ok_or_else(|| {
                        GraphContainerCollapseError::UntypedExternalEdge(edge.edge_id.clone())
                    })?;
                    let public_port_name =
                        unique_boundary_port_name(&edge.from_output, &mut output_names);
                    outputs.push(HoudiniOperatorPort {
                        name: public_port_name.clone(),
                        data_kind,
                        required: true,
                        help: format!("Collapse output for external edge {}.", edge.edge_id),
                    });
                    mappings.push(GraphBoundaryMapping {
                        direction: GraphBoundaryMappingDirection::Output,
                        public_port_name: public_port_name.clone(),
                        internal_node_id: edge.from_node_id.clone(),
                        internal_port_name: edge.from_output.clone(),
                    });
                    external_edges.push(GraphContainerExternalEdge {
                        direction: GraphBoundaryMappingDirection::Output,
                        edge_id: edge.edge_id.clone(),
                        external_node_id: edge.to_node_id.clone(),
                        external_port_name: edge.to_input.clone(),
                        internal_node_id: edge.from_node_id.clone(),
                        internal_port_name: edge.from_output.clone(),
                        public_port_name,
                        data_kind,
                    });
                }
                _ => {}
            }
        }

        Ok((
            GraphBoundaryDeclaration {
                inputs,
                outputs,
                mappings,
            },
            external_edges,
        ))
    }

    fn data_kind_for_edge(&self, edge: &GraphDataFlowEdge) -> Option<HoudiniDataKind> {
        let source_kind = self
            .nodes
            .iter()
            .find(|node| node.node_id == edge.from_node_id)
            .and_then(|node| self.node_output_kind_for_name(node, &edge.from_output));
        let target_kind = self
            .nodes
            .iter()
            .find(|node| node.node_id == edge.to_node_id)
            .and_then(|node| self.node_input_kind_for_name(node, &edge.to_input));
        match (source_kind, target_kind) {
            (Some(source_kind), Some(target_kind)) if source_kind == target_kind => {
                Some(source_kind)
            }
            (Some(data_kind), None) | (None, Some(data_kind)) => Some(data_kind),
            _ => None,
        }
    }

    fn node_set_layout_center(&self, node_ids: &[String]) -> Option<GraphPoint> {
        let mut count = 0.0;
        let mut total = GraphPoint::new(0.0, 0.0);
        for node in &self.nodes {
            if node_ids.iter().any(|node_id| node_id == &node.node_id) {
                count += 1.0;
                total.x += node.layout_position.x;
                total.y += node.layout_position.y;
            }
        }
        (count > 0.0).then(|| GraphPoint::new(total.x / count, total.y / count))
    }

    fn unique_graph_id(&self, name: &str) -> String {
        let slug = sanitize_asset_id_part(name);
        let base = format!("graph.{slug}");
        if self.graph_registry.graph(&base).is_none() {
            return base;
        }

        let mut suffix = 2;
        loop {
            let graph_id = format!("{base}_{suffix}");
            if self.graph_registry.graph(&graph_id).is_none() {
                return graph_id;
            }
            suffix += 1;
        }
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
        let mut node = GraphNode::procedural_asset(instance_id, asset_id, instance_version);
        node.parent_graph_id = self.current_graph_id().to_owned();
        self.nodes.insert(insert_index, node);
        self.rebuild_default_data_flow_edges();
        insert_index
    }

    #[allow(dead_code)]
    pub fn create_asset_draft_from_graph(
        &self,
        display_name: impl Into<String>,
        description: impl Into<String>,
        help: impl Into<String>,
    ) -> CreateAssetDraft {
        let display_name = normalized_asset_display_name(display_name, "Project Asset");
        let asset_id = self.next_project_asset_id_for_display_name(&display_name);
        CreateAssetDraft {
            asset_id: asset_id.clone(),
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
            external_artifacts: Vec::new(),
            graph_snapshot: self.procedural_asset_graph_snapshot(),
            wrapped_subgraph: ProceduralAssetSubgraphReference {
                graph_id: format!("{asset_id}.graph"),
                output_node_id: "output.main".to_owned(),
                captures_native_cubic_bezier: true,
                graph_snapshot: None,
            },
        }
    }

    #[allow(dead_code)]
    pub fn create_asset_draft_from_graph_container(
        &self,
        node_index: usize,
        display_name: impl Into<String>,
        description: impl Into<String>,
        help: impl Into<String>,
    ) -> Result<CreateAssetDraft, GraphContainerAssetDraftError> {
        let node = self
            .nodes
            .get(node_index)
            .ok_or(GraphContainerAssetDraftError::MissingNodeIndex(node_index))?;
        if node.kind != NodeKind::GraphContainer {
            return Err(GraphContainerAssetDraftError::NotGraphContainer);
        }
        let container = self
            .graph_container_metadata_for_node(&node.node_id)
            .ok_or(GraphContainerAssetDraftError::MissingContainerMetadata)?;
        if self
            .graph_registry
            .graph(&container.internal_graph_id)
            .is_none()
        {
            return Err(GraphContainerAssetDraftError::MissingInternalGraph);
        }
        if container.boundary.outputs.is_empty() {
            return Err(GraphContainerAssetDraftError::MissingOutputBoundary);
        }

        let display_name = normalized_asset_display_name(display_name, &node.name);
        let asset_id = self.next_project_asset_id_for_display_name(&display_name);
        let graph_snapshot =
            self.procedural_asset_graph_snapshot_for_graph(&container.internal_graph_id);
        let output_node_id = container
            .boundary
            .mappings
            .iter()
            .find(|mapping| mapping.direction == GraphBoundaryMappingDirection::Output)
            .map(|mapping| mapping.internal_node_id.clone())
            .unwrap_or_else(|| "output.main".to_owned());

        Ok(CreateAssetDraft {
            asset_id,
            display_name,
            version: "0.1.0".to_owned(),
            description: description.into(),
            help: help.into(),
            inputs: container.boundary.inputs.clone(),
            outputs: container.boundary.outputs.clone(),
            promoted_parameters: self
                .promotable_asset_parameters_for_graph(&container.internal_graph_id),
            external_artifacts: Vec::new(),
            graph_snapshot: graph_snapshot.clone(),
            wrapped_subgraph: ProceduralAssetSubgraphReference {
                graph_id: container.internal_graph_id.clone(),
                output_node_id,
                captures_native_cubic_bezier: container
                    .boundary
                    .outputs
                    .iter()
                    .any(|output| output.data_kind.preserves_native_cubic_bezier()),
                graph_snapshot: Some(graph_snapshot),
            },
        })
    }

    fn next_project_asset_id_for_display_name(&self, display_name: &str) -> String {
        let asset_slug = sanitize_asset_id_part(display_name);
        let base_asset_id = format!("project.asset.{asset_slug}");
        if !self
            .procedural_asset_declarations
            .iter()
            .any(|declaration| declaration.asset_id == base_asset_id)
        {
            return base_asset_id;
        }

        let mut suffix = 2;
        loop {
            let asset_id = format!("{base_asset_id}_{suffix}");
            if !self
                .procedural_asset_declarations
                .iter()
                .any(|declaration| declaration.asset_id == asset_id)
            {
                return asset_id;
            }
            suffix += 1;
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

    pub fn create_asset_instance_from_graph(
        &mut self,
        display_name: impl Into<String>,
        description: impl Into<String>,
        help: impl Into<String>,
    ) -> (String, usize) {
        let draft = self.create_asset_draft_from_graph(display_name, description, help);
        let asset_id = self.commit_asset_draft(draft);
        let node_index = self.add_procedural_asset_node(asset_id.clone());
        (asset_id, node_index)
    }

    #[allow(dead_code)]
    pub fn save_procedural_asset_definition(
        &mut self,
        node_index: usize,
    ) -> Option<ProceduralAssetDefinitionSaveResult> {
        let asset_node = self
            .nodes
            .get(node_index)?
            .procedural_asset
            .as_ref()?
            .clone();
        if !asset_node.contents_unlocked {
            return None;
        }

        let declaration_index = self
            .procedural_asset_declarations
            .iter()
            .position(|declaration| declaration.asset_id == asset_node.asset_id)?;
        let previous_version = self.procedural_asset_declarations[declaration_index]
            .version
            .clone();
        let new_version = next_asset_definition_version(&previous_version);
        let graph_snapshot = self.procedural_asset_graph_snapshot();
        let promoted_parameters = self.promotable_asset_parameters();
        let update_available_instance_count = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(index, node)| {
                *index != node_index
                    && node.procedural_asset.as_ref().is_some_and(|candidate| {
                        candidate.asset_id == asset_node.asset_id
                            && candidate.instance_version == previous_version
                            && !candidate.contents_unlocked
                    })
            })
            .count();

        {
            let declaration = &mut self.procedural_asset_declarations[declaration_index];
            declaration.version = new_version.clone();
            declaration.promoted_parameters = promoted_parameters;
            declaration.source.source_digest = Some(stable_digest(&serde_json::json!({
                "asset_id": &declaration.asset_id,
                "version": &declaration.version,
                "snapshot": &graph_snapshot,
            })));
            declaration.wrapped_subgraph.graph_snapshot = Some(graph_snapshot);
        }

        self.refresh_asset_version_statuses();

        let node = self.nodes.get_mut(node_index)?;
        let saved_asset_node = node.procedural_asset.as_mut()?;
        saved_asset_node.instance_version = new_version.clone();
        saved_asset_node.version_status = OperatorVersionStatus::Current;
        saved_asset_node.contents_unlocked = false;
        node.evaluation.state = EvaluationState::Stale;
        node.evaluation.message = Some(format!(
            "Saved asset definition {} as version {}.",
            asset_node.asset_id, new_version
        ));

        Some(ProceduralAssetDefinitionSaveResult {
            asset_id: asset_node.asset_id,
            previous_version,
            new_version,
            update_available_instance_count,
        })
    }

    #[allow(dead_code)]
    pub fn refresh_asset_version_statuses(&mut self) {
        for node in &mut self.nodes {
            let Some(asset_node) = node.procedural_asset.as_mut() else {
                continue;
            };
            asset_node.version_status =
                procedural_asset_version_status(&self.procedural_asset_declarations, asset_node);
            if asset_node.version_status == OperatorVersionStatus::NewerAvailable {
                node.evaluation.state = EvaluationState::Stale;
                node.evaluation.message = Some(
                    "Asset declaration version changed after this instance was created.".to_owned(),
                );
            }
        }
    }

    fn procedural_asset_version_status_for_instance(
        &self,
        asset_node: &ProceduralAssetInstanceNode,
    ) -> OperatorVersionStatus {
        procedural_asset_version_status(&self.procedural_asset_declarations, asset_node)
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
    pub fn match_procedural_asset_definition(&mut self, node_index: usize) -> bool {
        let Some(node) = self.nodes.get_mut(node_index) else {
            return false;
        };
        let Some(asset_node) = node.procedural_asset.as_mut() else {
            return false;
        };
        if !asset_node.contents_unlocked {
            return false;
        }

        asset_node.contents_unlocked = false;
        node.evaluation.state = EvaluationState::Stale;
        node.evaluation.message = Some(
            "Asset contents matched to the pinned definition without changing the pinned version."
                .to_owned(),
        );
        true
    }

    #[allow(dead_code)]
    pub fn upgrade_procedural_asset_to_current_definition(&mut self, node_index: usize) -> bool {
        let Some(node) = self.nodes.get_mut(node_index) else {
            return false;
        };
        let Some(asset_node) = node.procedural_asset.as_mut() else {
            return false;
        };
        let Some(declaration) = self
            .procedural_asset_declarations
            .iter()
            .find(|declaration| declaration.asset_id == asset_node.asset_id)
        else {
            return false;
        };
        if asset_node.instance_version == declaration.version {
            return false;
        }

        asset_node.instance_version = declaration.version.clone();
        asset_node.version_status = OperatorVersionStatus::Current;
        asset_node.contents_unlocked = false;
        node.evaluation.state = EvaluationState::Stale;
        node.evaluation.message = Some(format!(
            "Asset instance upgraded to definition version {}.",
            asset_node.instance_version
        ));
        true
    }

    #[allow(dead_code)]
    pub fn add_procedural_asset_boundary_port(
        &mut self,
        asset_id: &str,
        direction: ProceduralAssetBoundaryDirection,
        port: HoudiniOperatorPort,
    ) -> bool {
        let Some(port) = normalized_asset_boundary_port(port) else {
            return false;
        };
        let changed = self.edit_procedural_asset_boundary_ports(asset_id, direction, |ports| {
            if ports.iter().any(|existing| existing.name == port.name) {
                return false;
            }
            ports.push(port);
            true
        });
        if changed {
            self.mark_procedural_asset_boundary_changed(asset_id, direction);
        }
        changed
    }

    #[allow(dead_code)]
    pub fn replace_procedural_asset_boundary_port(
        &mut self,
        asset_id: &str,
        direction: ProceduralAssetBoundaryDirection,
        existing_port_name: &str,
        port: HoudiniOperatorPort,
    ) -> bool {
        let existing_port_name = existing_port_name.trim();
        if existing_port_name.is_empty() {
            return false;
        }
        let Some(port) = normalized_asset_boundary_port(port) else {
            return false;
        };
        let changed = self.edit_procedural_asset_boundary_ports(asset_id, direction, |ports| {
            let Some(index) = ports
                .iter()
                .position(|existing| existing.name == existing_port_name)
            else {
                return false;
            };
            if port.name != existing_port_name
                && ports.iter().any(|existing| existing.name == port.name)
            {
                return false;
            }
            if ports[index] == port {
                return false;
            }
            ports[index] = port;
            true
        });
        if changed {
            self.mark_procedural_asset_boundary_changed(asset_id, direction);
        }
        changed
    }

    #[allow(dead_code)]
    pub fn remove_procedural_asset_boundary_port(
        &mut self,
        asset_id: &str,
        direction: ProceduralAssetBoundaryDirection,
        port_name: &str,
    ) -> bool {
        let port_name = port_name.trim();
        if port_name.is_empty() {
            return false;
        }
        let changed = self.edit_procedural_asset_boundary_ports(asset_id, direction, |ports| {
            if direction == ProceduralAssetBoundaryDirection::Output && ports.len() <= 1 {
                return false;
            }
            let Some(index) = ports.iter().position(|port| port.name == port_name) else {
                return false;
            };
            ports.remove(index);
            true
        });
        if changed {
            self.mark_procedural_asset_boundary_changed(asset_id, direction);
        }
        changed
    }

    fn edit_procedural_asset_boundary_ports(
        &mut self,
        asset_id: &str,
        direction: ProceduralAssetBoundaryDirection,
        edit: impl FnOnce(&mut Vec<HoudiniOperatorPort>) -> bool,
    ) -> bool {
        let Some(declaration) = self
            .procedural_asset_declarations
            .iter_mut()
            .find(|declaration| declaration.asset_id == asset_id)
        else {
            return false;
        };
        edit(direction.ports_mut(declaration))
    }

    fn mark_procedural_asset_boundary_changed(
        &mut self,
        asset_id: &str,
        direction: ProceduralAssetBoundaryDirection,
    ) {
        for node in &mut self.nodes {
            let Some(asset_node) = node.procedural_asset.as_ref() else {
                continue;
            };
            if asset_node.asset_id != asset_id {
                continue;
            }
            node.evaluation.state = EvaluationState::Stale;
            node.evaluation.message = Some(format!(
                "Asset {} boundary changed; review instance bindings before running.",
                direction.as_str()
            ));
        }
    }

    #[allow(dead_code)]
    pub fn procedural_asset_bundle_preview(
        &self,
        asset_id: &str,
        inclusion_choices: &[ProceduralAssetArtifactInclusionChoice],
    ) -> Option<ProceduralAssetBundlePreview> {
        let declaration = self
            .procedural_asset_declarations
            .iter()
            .find(|declaration| declaration.asset_id == asset_id)?;

        let artifacts = declaration
            .external_artifacts
            .iter()
            .map(|artifact| {
                let choice = inclusion_choices
                    .iter()
                    .find(|choice| choice.locator == artifact.locator);
                ProceduralAssetArtifactBundlePreview::from_reference(
                    &declaration.asset_id,
                    artifact,
                    choice,
                )
            })
            .collect::<Vec<_>>();
        Some(ProceduralAssetBundlePreview::new(declaration, artifacts))
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
        self.promotable_asset_parameters_for_graph(self.current_graph_id())
    }

    fn promotable_asset_parameters_for_graph(
        &self,
        graph_id: &str,
    ) -> Vec<HoudiniParameterDeclaration> {
        let mut parameters = Vec::new();
        if let Some(filter_node) = self.nodes.iter().find(|node| {
            node.kind == NodeKind::Filter && self.node_parent_graph_id(node) == graph_id
        }) {
            parameters.push(HoudiniParameterDeclaration {
                name: "minimum_score".to_owned(),
                label: Some(filter_node.parameter.name.to_owned()),
                kind: HoudiniParameterKind::Float,
                default_value: HoudiniParameterValue::Float(filter_node.parameter.value),
                current_value: Some(HoudiniParameterValue::Float(filter_node.parameter.value)),
                range: Some(HoudiniNumericRange { min: 0.0, max: 1.0 }),
                allowed_values: Vec::new(),
                group: Some("Filter".to_owned()),
                binding: Some(HoudiniParameterBinding {
                    internal_node_id: filter_node.node_id.clone(),
                    internal_parameter_name: "score_threshold".to_owned(),
                }),
                help: "Promoted graph filter threshold.".to_owned(),
            });
        }
        if let Some(style_node) = self.nodes.iter().find(|node| {
            node.kind == NodeKind::Style && self.node_parent_graph_id(node) == graph_id
        }) {
            parameters.push(HoudiniParameterDeclaration {
                name: "stroke_scale".to_owned(),
                label: Some(style_node.parameter.name.to_owned()),
                kind: HoudiniParameterKind::Float,
                default_value: HoudiniParameterValue::Float(style_node.parameter.value),
                current_value: Some(HoudiniParameterValue::Float(style_node.parameter.value)),
                range: Some(HoudiniNumericRange { min: 0.0, max: 1.0 }),
                allowed_values: Vec::new(),
                group: Some("Style".to_owned()),
                binding: Some(HoudiniParameterBinding {
                    internal_node_id: style_node.node_id.clone(),
                    internal_parameter_name: "stroke_scale".to_owned(),
                }),
                help: "Promoted graph style stroke scale.".to_owned(),
            });
        }
        parameters
    }

    fn procedural_asset_graph_snapshot(&self) -> ProceduralAssetGraphSnapshot {
        self.procedural_asset_graph_snapshot_for_graph(self.current_graph_id())
    }

    fn procedural_asset_graph_snapshot_for_graph(
        &self,
        graph_id: &str,
    ) -> ProceduralAssetGraphSnapshot {
        let layout = self.graph_layout_for_graph(graph_id);
        ProceduralAssetGraphSnapshot {
            node_count: layout.nodes.len(),
            edge_count: layout.edges.len(),
            layer_count: self.layers.len(),
            geometry_contract: "HoudiniGeometryRecord polygons and native cubic Beziers".to_owned(),
        }
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
        let mut node = GraphNode::native_operator(instance_id, operator_id);
        node.parent_graph_id = self.current_graph_id().to_owned();
        self.nodes.insert(insert_index, node);
        self.rebuild_default_data_flow_edges();
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
                NodeKind::GraphContainer => {
                    "Subnet-like container points to an internal named graph.".to_owned()
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
        self.graph_layout_for_graph(self.current_graph_id())
    }

    #[allow(dead_code)]
    pub fn graph_layout_for_graph(&self, graph_id: &str) -> GraphLayout {
        let graph_node_indices = self.graph_local_node_indices(graph_id);
        let nodes = graph_node_indices
            .iter()
            .filter_map(|index| {
                let node = self.nodes.get(*index)?;
                Some(GraphLayoutNode {
                    node_index: *index,
                    name: node.name.clone(),
                    position: node.layout_position,
                })
            })
            .collect();

        let node_indices_by_id = graph_node_indices
            .into_iter()
            .filter_map(|index| {
                let node = self.nodes.get(index)?;
                Some((node.node_id.as_str(), index))
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let edges = self
            .data_flow_edges
            .iter()
            .filter_map(|edge| {
                Some(GraphEdge {
                    edge_id: edge.edge_id.clone(),
                    from_node: *node_indices_by_id.get(edge.from_node_id.as_str())?,
                    to_node: *node_indices_by_id.get(edge.to_node_id.as_str())?,
                })
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
    pub fn finish_node_layout_drag(&mut self, index: usize, old_position: GraphPoint) -> bool {
        self.finish_node_layout_drag_with_network_box_snapshots(index, old_position, &[])
    }

    pub fn finish_node_layout_drag_with_network_box_snapshots(
        &mut self,
        index: usize,
        old_position: GraphPoint,
        old_network_box_states: &[NetworkBoxOrganizationSnapshot],
    ) -> bool {
        let Some(node) = self.nodes.get(index) else {
            return false;
        };
        let new_position = node.layout_position;
        let network_box_changes = old_network_box_states
            .iter()
            .filter_map(|old_state| {
                let annotation = self
                    .annotations
                    .iter()
                    .find(|annotation| annotation.annotation_id == old_state.annotation_id)?;
                let new_state = NetworkBoxOrganizationSnapshot::from_annotation(annotation);
                (*old_state != new_state).then(|| NetworkBoxOrganizationCommandSnapshot {
                    annotation_id: old_state.annotation_id.clone(),
                    old_state: old_state.clone(),
                    new_state,
                })
            })
            .collect::<Vec<_>>();
        if old_position == new_position && network_box_changes.is_empty() {
            return false;
        }
        let node_id = node.node_id.clone();
        let node_name = node.name.clone();
        self.record_project_command(ProjectCommand::NodeLayoutEdit {
            node_id,
            node_name,
            old_position,
            new_position,
            network_box_changes,
        });
        true
    }

    #[allow(dead_code)]
    pub fn mark_node_stale(&mut self, index: usize) {
        if let Some(node) = self.nodes.get_mut(index) {
            node.evaluation.state = EvaluationState::Stale;
            node.evaluation.message = None;
        }
        self.mark_reference_inputs_stale_for_target_index(index);
    }

    pub fn set_node_manual(&mut self, index: usize, manual: bool) -> bool {
        let Some(node) = self.nodes.get_mut(index) else {
            return false;
        };
        if node.evaluation.manual == manual {
            return false;
        }
        let old_manual = node.evaluation.manual;
        Self::apply_node_manual_state(node, manual);
        let node_id = node.node_id.clone();
        let node_name = node.name.clone();
        self.record_project_command(ProjectCommand::NodeManualCookEdit {
            node_id,
            node_name,
            old_manual,
            new_manual: manual,
        });
        true
    }

    fn apply_node_manual_state(node: &mut GraphNode, manual: bool) {
        node.evaluation.manual = manual;
        node.evaluation.state = if manual {
            EvaluationState::Manual
        } else {
            EvaluationState::Stale
        };
        node.evaluation.message = None;
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
                let work_item_status = if self.evaluation_mode == GraphEvaluationMode::Manual {
                    node.evaluation.state = EvaluationState::Manual;
                    node.evaluation.manual = true;
                    node.evaluation.message = Some("Waiting for manual evaluation".to_owned());
                    Some((
                        GraphWorkItemStatus::Waiting,
                        "Manual evaluation mode is waiting for an explicit run",
                    ))
                } else {
                    node.evaluation.state = EvaluationState::Cached;
                    node.evaluation.message = None;
                    Some((GraphWorkItemStatus::Cached, "Cached output reused"))
                };
                if let Some((status, summary)) = work_item_status {
                    self.record_work_item(index, status, summary);
                }
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
            self.record_work_item(index, GraphWorkItemStatus::Running, "Manual run requested");
        }
    }

    pub fn complete_node_run(&mut self, index: usize) {
        if let Some(node) = self.nodes.get_mut(index) {
            node.evaluation.state = EvaluationState::Clean;
            node.evaluation.manual = false;
            node.evaluation.message = None;
            self.record_work_item(index, GraphWorkItemStatus::Complete, "Run complete");
        }
    }

    pub fn cancel_node_run(&mut self, index: usize) {
        if let Some(node) = self.nodes.get_mut(index)
            && node.evaluation.state == EvaluationState::Running
        {
            node.evaluation.state = EvaluationState::Manual;
            node.evaluation.manual = true;
            node.evaluation.message = Some("Run cancelled".to_owned());
            self.record_work_item(index, GraphWorkItemStatus::Canceled, "Run canceled");
        }
    }

    #[allow(dead_code)]
    pub fn fail_node_run(&mut self, index: usize, message: impl Into<String>) {
        let message = message.into();
        if let Some(node) = self.nodes.get_mut(index) {
            node.evaluation.state = EvaluationState::Failed;
            node.evaluation.message = Some(message.clone());
            self.record_work_item(index, GraphWorkItemStatus::Failed, message);
        }
    }

    pub fn queue_node_evaluation(&mut self, index: usize) {
        self.supersede_running_work_items_for_node(
            index,
            "Queued evaluation superseded previous running work",
        );
        if let Some(node) = self.nodes.get_mut(index) {
            node.evaluation.state = EvaluationState::Stale;
            node.evaluation.message = Some("Evaluation queued".to_owned());
            self.record_work_item(
                index,
                GraphWorkItemStatus::Waiting,
                "Waiting for evaluation",
            );
        }
    }

    pub fn retry_work_item_for_node(&mut self, index: usize) {
        self.request_node_run(index);
        if let Some(item) = self.latest_work_item_for_node_mut(index) {
            item.summary = "Retry requested for current graph request".to_owned();
        }
    }

    fn record_work_item(
        &mut self,
        node_index: usize,
        status: GraphWorkItemStatus,
        summary: impl Into<String>,
    ) {
        let Some(node) = self.nodes.get(node_index) else {
            return;
        };
        let fingerprint = self.evaluation_fingerprint_for_node(node_index);
        let work_item = GraphWorkItem {
            work_item_id: format!("work_item_{}", self.work_items.len() + 1),
            node_index,
            node_id: node.node_id.clone(),
            node_name: node.name.clone(),
            output_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
            status,
            fingerprint,
            summary: summary.into(),
            diagnostic: node.evaluation.message.clone(),
            progress: match status {
                GraphWorkItemStatus::Waiting => 0.0,
                GraphWorkItemStatus::Running => 0.5,
                GraphWorkItemStatus::Cached | GraphWorkItemStatus::Complete => 1.0,
                GraphWorkItemStatus::Canceled
                | GraphWorkItemStatus::Superseded
                | GraphWorkItemStatus::Failed => 0.0,
            },
            created_at_millis: current_timestamp_millis(),
        };
        self.work_items.push(work_item);
    }

    pub fn evaluation_fingerprint_for_node(&self, node_index: usize) -> String {
        self.nodes.get(node_index).map_or_else(
            || "missing-node".to_owned(),
            |node| {
                format!(
                    "{}:{}:{:.6}:{}",
                    node.node_id,
                    PRIMARY_GEOMETRY_OUTPUT,
                    node.parameter.value,
                    node.kind.as_str()
                )
            },
        )
    }

    fn latest_work_item_for_node_mut(&mut self, node_index: usize) -> Option<&mut GraphWorkItem> {
        self.work_items
            .iter_mut()
            .rev()
            .find(|item| item.node_index == node_index)
    }

    fn supersede_running_work_items_for_node(&mut self, node_index: usize, summary: &str) {
        for item in &mut self.work_items {
            if item.node_index == node_index
                && matches!(
                    item.status,
                    GraphWorkItemStatus::Waiting | GraphWorkItemStatus::Running
                )
            {
                item.status = GraphWorkItemStatus::Superseded;
                item.summary = summary.to_owned();
                item.progress = 0.0;
            }
        }
    }

    pub fn selected_node_info(&self, index: usize) -> Option<NodeInfo> {
        let node = self.nodes.get(index)?;
        let stage = self.pipeline_stage_for_node(index, node);
        let source_metadata = node
            .source_node
            .as_ref()
            .and_then(SourceNode::primary_metadata)
            .unwrap_or_else(|| self.source.metadata.clone());
        let filter_warnings = self.filter_rule_warning().into_iter().collect::<Vec<_>>();
        let style_warnings = self.style_warnings();
        let reference_consumers = self.reference_consumers_for_node(index);
        let reference_output_warning = self.reference_output_change_warning_for_node(index);
        let coordinate_warnings = self.coordinate_contract_warnings(node);
        let substrate_raster = self.substrate_raster_for_node(node);
        let graph_location = self.graph_location_for_node(node);
        let data_flow = self.data_flow_info_for_node(node);

        Some(match node.kind {
            NodeKind::Source => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                graph_location: graph_location.clone(),
                data_flow: data_flow.clone(),
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
                attributes: source_metadata.attribute_names.clone(),
                parameter: node.parameter.clone(),
                summary: "Source geometry lives in the graph model before any viewer adaptation.",
                source_metadata: Some(source_metadata.clone()),
                coordinate_contract: node.coordinate_contract.clone(),
                substrate_raster: substrate_raster.clone(),
                source_error: self.source.import_error.clone(),
                style: None,
                generated: node.generated,
                evaluation: node.evaluation.clone(),
                warnings: with_coordinate_warnings(Vec::new(), &coordinate_warnings),
                reference_consumers: reference_consumers.clone(),
                reference_output_warning: reference_output_warning.clone(),
                output_operator: None,
                null_operator: None,
                reference_input: None,
                graph_container: None,
                python_operator: None,
                procedural_asset: None,
                native_operator: None,
            },
            NodeKind::Filter => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                graph_location: graph_location.clone(),
                data_flow: data_flow.clone(),
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
                coordinate_contract: node.coordinate_contract.clone(),
                substrate_raster: substrate_raster.clone(),
                source_error: None,
                style: None,
                generated: node.generated,
                evaluation: node.evaluation.clone(),
                warnings: with_coordinate_warnings(filter_warnings, &coordinate_warnings),
                reference_consumers: reference_consumers.clone(),
                reference_output_warning: reference_output_warning.clone(),
                output_operator: None,
                null_operator: None,
                reference_input: None,
                graph_container: None,
                python_operator: None,
                procedural_asset: None,
                native_operator: None,
            },
            NodeKind::Style => NodeInfo {
                kind: node.kind,
                role: node.kind.role(),
                graph_location: graph_location.clone(),
                data_flow: data_flow.clone(),
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
                coordinate_contract: node.coordinate_contract.clone(),
                substrate_raster: substrate_raster.clone(),
                source_error: None,
                style: Some(self.resolved_style()),
                generated: node.generated,
                evaluation: node.evaluation.clone(),
                warnings: with_coordinate_warnings(style_warnings, &coordinate_warnings),
                reference_consumers: reference_consumers.clone(),
                reference_output_warning: reference_output_warning.clone(),
                output_operator: None,
                null_operator: None,
                reference_input: None,
                graph_container: None,
                python_operator: None,
                procedural_asset: None,
                native_operator: None,
            },
            NodeKind::Null => {
                let contract = self.null_operator_contract(index)?;
                NodeInfo {
                    kind: node.kind,
                    role: node.kind.role(),
                    graph_location: graph_location.clone(),
                    data_flow: data_flow.clone(),
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
                    coordinate_contract: node.coordinate_contract.clone(),
                    substrate_raster: substrate_raster.clone(),
                    source_error: None,
                    style: Some(self.resolved_style()),
                    generated: node.generated,
                    evaluation: node.evaluation.clone(),
                    warnings: with_coordinate_warnings(Vec::new(), &coordinate_warnings),
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
                    graph_container: None,
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
                    graph_location: graph_location.clone(),
                    data_flow: data_flow.clone(),
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
                    coordinate_contract: node.coordinate_contract.clone(),
                    substrate_raster: substrate_raster.clone(),
                    source_error: None,
                    style: None,
                    generated: node.generated,
                    evaluation: node.evaluation.clone(),
                    warnings: with_coordinate_warnings(warnings, &coordinate_warnings),
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
                    graph_container: None,
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
                    graph_location: graph_location.clone(),
                    data_flow: data_flow.clone(),
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
                    coordinate_contract: node.coordinate_contract.clone(),
                    substrate_raster: substrate_raster.clone(),
                    source_error: None,
                    style: None,
                    generated: node.generated,
                    evaluation: node.evaluation.clone(),
                    warnings: with_coordinate_warnings(
                        vec![format!(
                            "Projection contract: {}",
                            projection.repair_summary
                        )],
                        &coordinate_warnings,
                    ),
                    reference_consumers: reference_consumers.clone(),
                    reference_output_warning: reference_output_warning.clone(),
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    graph_container: None,
                    python_operator: None,
                    procedural_asset: None,
                    native_operator: None,
                }
            }
            NodeKind::GraphContainer => {
                let container = self.graph_container_info_for_node(node);
                let input_count = container.inputs.len();
                let output_count = container.outputs.len();
                let mut warnings = if container.status == GraphContainerStatus::Resolved {
                    Vec::new()
                } else {
                    vec![container.status.as_str().to_owned()]
                };
                warnings.extend(
                    container
                        .mappings
                        .iter()
                        .filter(|mapping| mapping.status != GraphBoundaryMappingStatus::Resolved)
                        .map(GraphBoundaryMappingInfo::diagnostic),
                );
                NodeInfo {
                    kind: node.kind,
                    role: node.kind.role(),
                    graph_location: graph_location.clone(),
                    data_flow: data_flow.clone(),
                    input_count,
                    output_count,
                    status: if warnings.is_empty() {
                        NodeStatus::Healthy
                    } else {
                        NodeStatus::Failed
                    },
                    data_kind: "Graph container",
                    record_count: 0,
                    bounds: None,
                    provenance: Some(self.source.metadata.provenance),
                    attributes: Vec::new(),
                    parameter: node.parameter.clone(),
                    summary: "Graph container points to an internal named graph; typed boundary ports are not declared yet.",
                    source_metadata: None,
                    coordinate_contract: None,
                    substrate_raster: None,
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
                    graph_container: Some(container),
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
                    graph_location: graph_location.clone(),
                    data_flow: data_flow.clone(),
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
                    coordinate_contract: node.coordinate_contract.clone(),
                    substrate_raster: substrate_raster.clone(),
                    source_error: None,
                    style: None,
                    generated: node.generated,
                    evaluation: node.evaluation.clone(),
                    warnings: with_coordinate_warnings(warnings, &coordinate_warnings),
                    reference_consumers: reference_consumers.clone(),
                    reference_output_warning: reference_output_warning.clone(),
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    graph_container: None,
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
                let external_artifact_warnings = declaration
                    .map(|declaration| {
                        declaration
                            .external_artifacts
                            .iter()
                            .filter_map(ProceduralAssetArtifactReference::warning)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let mut warnings = warnings;
                warnings.extend(external_artifact_warnings.clone());
                NodeInfo {
                    kind: node.kind,
                    role: node.kind.role(),
                    graph_location: graph_location.clone(),
                    data_flow: data_flow.clone(),
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
                    coordinate_contract: node.coordinate_contract.clone(),
                    substrate_raster: substrate_raster.clone(),
                    source_error: None,
                    style: None,
                    generated: node.generated,
                    evaluation: node.evaluation.clone(),
                    warnings: with_coordinate_warnings(warnings, &coordinate_warnings),
                    reference_consumers: reference_consumers.clone(),
                    reference_output_warning: reference_output_warning.clone(),
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    graph_container: None,
                    python_operator: None,
                    procedural_asset: Some(ProceduralAssetNodeInfo {
                        asset_id: asset_node.asset_id.clone(),
                        display_name: declaration
                            .map(|declaration| declaration.display_name.clone())
                            .unwrap_or_else(|| "Missing asset declaration".to_owned()),
                        instance_version: asset_node.instance_version.clone(),
                        current_version: declaration.map(|declaration| declaration.version.clone()),
                        contents_unlocked: asset_node.contents_unlocked,
                        can_save_definition: asset_node.contents_unlocked && declaration.is_some(),
                        can_match_definition: asset_node.contents_unlocked,
                        can_upgrade_to_current_definition: declaration.is_some_and(|declaration| {
                            declaration.version != asset_node.instance_version
                        }),
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
                        external_artifact_warnings,
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
                    graph_location: graph_location.clone(),
                    data_flow: data_flow.clone(),
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
                    coordinate_contract: node.coordinate_contract.clone(),
                    substrate_raster: substrate_raster.clone(),
                    source_error: None,
                    style: None,
                    generated: node.generated,
                    evaluation: node.evaluation.clone(),
                    warnings: with_coordinate_warnings(warnings, &coordinate_warnings),
                    reference_consumers: reference_consumers.clone(),
                    reference_output_warning: reference_output_warning.clone(),
                    output_operator: None,
                    null_operator: None,
                    reference_input: None,
                    graph_container: None,
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
                graph_location,
                data_flow,
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
                coordinate_contract: node.coordinate_contract.clone(),
                substrate_raster: substrate_raster.clone(),
                source_error: None,
                style: None,
                generated: node.generated,
                evaluation: node.evaluation.clone(),
                warnings: with_coordinate_warnings(Vec::new(), &coordinate_warnings),
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
                graph_container: None,
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
        self.substrate_raster = None;
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
            substrate_raster: self.substrate_raster.clone(),
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
            archetypes::{Image, LineStrips2D, Points2D, TextDocument},
            components::{LineStrip2D, Position2D, Radius},
        };

        let path = path.as_ref();
        let scene = self.rerun_scene_output_with_query_bridge(None);
        let rec = RecordingStreamBuilder::new("houdini_graph_output").save(path)?;

        rec.log_static(
            "houdini_graph/metadata",
            &TextDocument::new(scene.recording_metadata_markdown(self)),
        )?;

        if let Some(raster) = &scene.substrate_raster {
            let entity_path = raster.recording_entity_path();
            rec.log_static(
                entity_path.as_str(),
                &Image::from_l8(raster.luma8_pixels(), [raster.width, raster.height]),
            )?;
            rec.log_static(
                format!("{entity_path}/metadata"),
                &TextDocument::new(raster.recording_metadata_markdown()),
            )?;
        }

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
            substrate_raster_count: usize::from(scene.substrate_raster.is_some()),
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
        self.substrate_raster = None;
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
                self.substrate_raster = None;
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

    pub fn save_source_package_manifest(
        &self,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<SourcePackageManifestWriteResult> {
        self.save_source_package_manifest_with_choice(
            path,
            SourcePackageManifestInclusionChoice::Default,
        )
    }

    pub fn save_source_package_manifest_with_choice(
        &self,
        path: impl AsRef<Path>,
        choice: SourcePackageManifestInclusionChoice,
    ) -> anyhow::Result<SourcePackageManifestWriteResult> {
        let path = path.as_ref();
        let manifest = self
            .source
            .metadata
            .package_manifest_preview_with_choice(choice);
        std::fs::write(path, serde_json::to_string_pretty(&manifest)?)?;

        Ok(SourcePackageManifestWriteResult {
            path: path.to_path_buf(),
            artifact_count: manifest.artifacts.len(),
            expected_size_bytes: manifest.expected_size_bytes,
            remaining_external_reference_count: manifest.remaining_external_reference_count,
            missing_reference_count: manifest.missing_reference_count,
            reproducibility_warning_count: manifest.reproducibility_warnings.len(),
        })
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
        self.substrate_raster = None;
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
            GraphSourceMode::DemoFallback | GraphSourceMode::SyntheticMalware => &self.geometry,
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

    fn coordinate_contract_warnings(&self, node: &GraphNode) -> Vec<String> {
        if self.source.metadata.provenance != SourceProvenance::SyntheticMalware {
            return Vec::new();
        }
        let Some(contract) = &node.coordinate_contract else {
            return Vec::new();
        };
        let Some(bounds) = self.source.metadata.bounds else {
            return Vec::new();
        };

        let exceeds_x = bounds.min.x < 0.0 || bounds.max.x > contract.width as f32;
        let exceeds_y = bounds.min.y < 0.0 || bounds.max.y > contract.height as f32;
        if !exceeds_x && !exceeds_y {
            return Vec::new();
        }

        vec![format!(
            "Overlay bounds [{:.1}, {:.1}]..[{:.1}, {:.1}] exceed substrate {} {}x{} {:?}/{:?}.",
            bounds.min.x,
            bounds.min.y,
            bounds.max.x,
            bounds.max.y,
            contract.substrate_id,
            contract.width,
            contract.height,
            contract.origin,
            contract.y_axis,
        )]
    }

    fn substrate_raster_for_node(&self, node: &GraphNode) -> Option<SubstrateRaster> {
        let raster = self.substrate_raster.as_ref()?;
        let contract = node.coordinate_contract.as_ref()?;
        (raster.substrate_id == contract.substrate_id).then(|| raster.clone())
    }
}

#[cfg(not(target_arch = "wasm32"))]
const CUBIC_RECORDING_LIMITATION: &str = "Rerun recordings preserve cubic Bezier semantics as graph-owned control-point metadata. The current replay path visualizes cubic curves as native control points plus a control-polygon preview; dense polyline tessellation remains an adaptive boundary/export representation only.";

fn with_coordinate_warnings(
    mut warnings: Vec<String>,
    coordinate_warnings: &[String],
) -> Vec<String> {
    warnings.extend(coordinate_warnings.iter().cloned());
    warnings
}

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
    #[serde(default)]
    pub locator: SourceLocator,
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
            locator: SourceLocator::demo(),
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
            locator: SourceLocator::for_provenance(provenance, source_path.as_deref()),
            source_path,
            record_count: geometry.len(),
            polygon_count,
            cubic_bezier_count,
            bounds,
            attribute_names: vec!["score".to_owned()],
            recognized_control_point_columns,
        }
    }

    fn normalized(mut self) -> Self {
        if self.locator.kind == SourceLocatorKind::Generated
            && self.locator.location.is_none()
            && self.locator.label.is_none()
        {
            self.locator =
                SourceLocator::for_provenance(self.provenance, self.source_path.as_deref());
        }
        self
    }

    pub fn external_reference_report(&self) -> SourceExternalReferenceReport {
        SourceExternalReferenceReport::from_locator(&self.locator)
    }

    pub fn bundle_preview(&self) -> SourceBundlePreview {
        SourceBundlePreview::from_external_reference(self.external_reference_report())
    }

    pub fn package_manifest_preview(&self) -> SourcePackageManifestPreview {
        SourcePackageManifestPreview::from_source_metadata(self, self.bundle_preview())
    }

    pub fn package_manifest_preview_with_choice(
        &self,
        choice: SourcePackageManifestInclusionChoice,
    ) -> SourcePackageManifestPreview {
        SourcePackageManifestPreview::from_source_metadata_with_choice(
            self,
            self.bundle_preview(),
            choice,
        )
    }

    pub fn source_format_inference_report(&self) -> SourceFormatInferenceReport {
        SourceFormatInferenceReport::from_metadata(self)
    }

    pub fn external_reference_action_report(&self) -> SourceExternalReferenceActionReport {
        SourceExternalReferenceActionReport::from_metadata(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct SourceLocator {
    pub kind: SourceLocatorKind,
    pub location: Option<String>,
    pub label: Option<String>,
}

impl Default for SourceLocator {
    fn default() -> Self {
        Self {
            kind: SourceLocatorKind::Generated,
            location: None,
            label: None,
        }
    }
}

impl SourceLocator {
    fn for_provenance(provenance: SourceProvenance, source_path: Option<&str>) -> Self {
        if let Some(source_path) = source_path.filter(|path| !path.trim().is_empty()) {
            return Self::from_location(source_path);
        }

        match provenance {
            SourceProvenance::DemoFallback => Self::demo(),
            SourceProvenance::RecordingQuery => Self::recording_query(),
            SourceProvenance::SyntheticBenchmark => Self::generated("synthetic benchmark"),
            SourceProvenance::SyntheticMalware => Self::generated("synthetic malware starter"),
            SourceProvenance::PythonOperator => Self::generated("python operator output"),
            SourceProvenance::ParquetImport => Self::generated("parquet import"),
        }
    }

    pub(crate) fn from_location(location: &str) -> Self {
        let location = location.to_owned();
        let kind = if location.contains("://") {
            SourceLocatorKind::Uri
        } else {
            SourceLocatorKind::LocalPath
        };
        Self {
            kind,
            location: Some(location),
            label: None,
        }
    }

    fn demo() -> Self {
        Self {
            kind: SourceLocatorKind::Demo,
            location: None,
            label: Some("demo fallback".to_owned()),
        }
    }

    fn recording_query() -> Self {
        Self {
            kind: SourceLocatorKind::RecordingQuery,
            location: None,
            label: Some("active recording query".to_owned()),
        }
    }

    fn generated(label: impl Into<String>) -> Self {
        Self {
            kind: SourceLocatorKind::Generated,
            location: None,
            label: Some(label.into()),
        }
    }

    pub fn readable(&self) -> String {
        self.location
            .clone()
            .or_else(|| self.label.clone())
            .unwrap_or_else(|| self.kind.as_str().to_owned())
    }

    pub fn is_external_reference(&self) -> bool {
        matches!(
            self.kind,
            SourceLocatorKind::LocalPath | SourceLocatorKind::Uri
        )
    }

    pub fn is_generated(&self) -> bool {
        matches!(
            self.kind,
            SourceLocatorKind::Demo | SourceLocatorKind::Generated
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SourceLocatorKind {
    LocalPath,
    Uri,
    RecordingQuery,
    Generated,
    Demo,
}

impl SourceLocatorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalPath => "local path",
            Self::Uri => "uri",
            Self::RecordingQuery => "recording query",
            Self::Generated => "generated",
            Self::Demo => "demo",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceGalleryIndex {
    pub source: SourceLocator,
    pub items: Vec<SourceGalleryItem>,
    pub limit: usize,
    pub truncated: bool,
    pub warnings: Vec<String>,
}

#[allow(dead_code)]
impl SourceGalleryIndex {
    pub fn from_locator(source: SourceLocator, limit: usize) -> Self {
        let limit = limit.max(1);
        let mut warnings = Vec::new();

        let items = match source.kind {
            SourceLocatorKind::LocalPath => {
                let readable = source.readable();
                let path = Path::new(&readable);
                if path.is_dir() {
                    source_gallery_items_from_directory(path, limit, &mut warnings)
                } else {
                    vec![SourceGalleryItem::from_locator(source.clone())]
                }
            }
            SourceLocatorKind::Uri => {
                if source_gallery_uri_requires_manifest(&source.readable()) {
                    warnings.push(format!(
                        "Remote collection `{}` requires an explicit manifest; recursive URL listing is not supported.",
                        source.readable()
                    ));
                    Vec::new()
                } else {
                    vec![SourceGalleryItem::from_locator(source.clone())]
                }
            }
            SourceLocatorKind::RecordingQuery
            | SourceLocatorKind::Generated
            | SourceLocatorKind::Demo => {
                vec![SourceGalleryItem::from_locator(source.clone())]
            }
        };

        let truncated = items.len() > limit;
        let items = items.into_iter().take(limit).collect();

        Self {
            source,
            items,
            limit,
            truncated,
            warnings,
        }
    }

    pub fn from_locations(
        source: SourceLocator,
        locations: impl IntoIterator<Item = SourceLocator>,
        limit: usize,
    ) -> Self {
        let limit = limit.max(1);
        let mut items = locations
            .into_iter()
            .map(SourceGalleryItem::from_locator)
            .collect::<Vec<_>>();
        let truncated = items.len() > limit;
        items.truncate(limit);

        Self {
            source,
            items,
            limit,
            truncated,
            warnings: Vec::new(),
        }
    }

    pub fn from_manifest_json(
        source: SourceLocator,
        manifest_json: &str,
        limit: usize,
    ) -> Result<Self, SourceGalleryManifestError> {
        let manifest = SourceGalleryManifest::parse(manifest_json)?;
        let locations = manifest.entries.into_iter().map(|entry| {
            let mut locator = SourceLocator::from_location(&entry.location);
            locator.label = entry.label;
            locator
        });
        Ok(Self::from_locations(source, locations, limit))
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceGalleryItem {
    pub stable_id: String,
    pub display_name: String,
    pub locator: SourceLocator,
    pub kind: SourceGalleryItemKind,
    pub thumbnail_intent: SourceGalleryThumbnailIntent,
    pub external_reference_status: SourceExternalReferenceStatus,
    pub format_kind: Option<SourceFormatKind>,
    pub format_support_status: Option<SourceFormatSupportStatus>,
}

#[allow(dead_code)]
impl SourceGalleryItem {
    fn from_locator(locator: SourceLocator) -> Self {
        let metadata = SourceMetadata {
            provenance: SourceProvenance::ParquetImport,
            source_path: locator.location.clone(),
            locator: locator.clone(),
            ..Default::default()
        };
        let format_report = metadata.source_format_inference_report();
        let external_reference = metadata.external_reference_report();
        let kind = SourceGalleryItemKind::from_locator_and_format(&locator, &format_report);
        let stable_id = source_gallery_stable_id(&locator);

        Self {
            thumbnail_intent: SourceGalleryThumbnailIntent::from_kind_and_status(
                kind,
                external_reference.status,
                stable_id.clone(),
            ),
            stable_id,
            display_name: source_gallery_display_name(&locator),
            locator,
            kind,
            external_reference_status: external_reference.status,
            format_kind: format_report.kind,
            format_support_status: format_report.support_status,
        }
    }

    pub fn open_action_report(&self) -> SourceGalleryOpenActionReport {
        SourceGalleryOpenActionReport::from_item(self)
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceGalleryOpenActionReport {
    pub kind: SourceGalleryOpenActionKind,
    pub enabled: bool,
    pub label: &'static str,
    pub status: String,
}

#[allow(dead_code)]
impl SourceGalleryOpenActionReport {
    fn from_item(item: &SourceGalleryItem) -> Self {
        if item.external_reference_status == SourceExternalReferenceStatus::LocalMissing {
            return Self::disabled(
                SourceGalleryOpenActionKind::Unavailable,
                "Open in Rerun",
                "Source is missing on this machine.",
            );
        }

        match item.kind {
            SourceGalleryItemKind::Image => Self::enabled(
                SourceGalleryOpenActionKind::OpenImage2D,
                "Open Image",
                "Open through Rerun's file/URL loader and let the viewer select the image view.",
            ),
            SourceGalleryItemKind::Recording => Self::enabled(
                SourceGalleryOpenActionKind::OpenRecording,
                "Open Recording",
                "Open this .rrd through the existing Rerun recording loader.",
            ),
            SourceGalleryItemKind::LiveRecording => Self::disabled(
                SourceGalleryOpenActionKind::AlreadyLive,
                "Open Recording",
                "Live recording query sources are already viewer inputs.",
            ),
            SourceGalleryItemKind::Table
            | SourceGalleryItemKind::PolygonTable
            | SourceGalleryItemKind::PointCloud => Self::disabled(
                SourceGalleryOpenActionKind::Unsupported,
                "Open in Rerun",
                "This source type is cataloged here but is not opened directly by the gallery yet.",
            ),
            SourceGalleryItemKind::Manifest => Self::disabled(
                SourceGalleryOpenActionKind::Unsupported,
                "Open Manifest",
                "Use the manifest as a gallery source; individual manifest items can be opened.",
            ),
            SourceGalleryItemKind::Generated => Self::disabled(
                SourceGalleryOpenActionKind::Unavailable,
                "Open in Rerun",
                "Generated sources do not have an external locator to open.",
            ),
            SourceGalleryItemKind::Unknown => Self::disabled(
                SourceGalleryOpenActionKind::Unsupported,
                "Open in Rerun",
                "Unknown source format cannot be opened directly from the gallery.",
            ),
        }
    }

    fn enabled(kind: SourceGalleryOpenActionKind, label: &'static str, status: &str) -> Self {
        Self {
            kind,
            enabled: true,
            label,
            status: status.to_owned(),
        }
    }

    fn disabled(kind: SourceGalleryOpenActionKind, label: &'static str, status: &str) -> Self {
        Self {
            kind,
            enabled: false,
            label,
            status: status.to_owned(),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceGalleryOpenActionKind {
    OpenImage2D,
    OpenRecording,
    AlreadyLive,
    Unsupported,
    Unavailable,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SourceGalleryItemKind {
    Image,
    Table,
    PolygonTable,
    Recording,
    PointCloud,
    Manifest,
    Generated,
    LiveRecording,
    Unknown,
}

#[allow(dead_code)]
impl SourceGalleryItemKind {
    fn from_locator_and_format(
        locator: &SourceLocator,
        format_report: &SourceFormatInferenceReport,
    ) -> Self {
        match locator.kind {
            SourceLocatorKind::Generated | SourceLocatorKind::Demo => return Self::Generated,
            SourceLocatorKind::RecordingQuery => return Self::LiveRecording,
            SourceLocatorKind::LocalPath | SourceLocatorKind::Uri => {}
        }

        if source_gallery_is_image_locator(&locator.readable()) {
            return Self::Image;
        }

        if source_gallery_is_recording_locator(&locator.readable()) {
            return Self::Recording;
        }

        if source_gallery_is_manifest_locator(&locator.readable()) {
            return Self::Manifest;
        }

        match format_report.kind {
            Some(
                SourceFormatKind::GeoParquetLike
                | SourceFormatKind::GeoJson
                | SourceFormatKind::FlatGeobuf,
            ) => Self::PolygonTable,
            Some(
                SourceFormatKind::Parquet
                | SourceFormatKind::CsvCoordinates
                | SourceFormatKind::SqliteTableOrView,
            ) => Self::Table,
            Some(SourceFormatKind::LasLazPointCloud) => Self::PointCloud,
            Some(
                SourceFormatKind::GeoPackage
                | SourceFormatKind::SpatiaLite
                | SourceFormatKind::Shapefile,
            ) => Self::PolygonTable,
            None => Self::Unknown,
        }
    }

    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Table => "table",
            Self::PolygonTable => "polygon table",
            Self::Recording => "recording",
            Self::PointCloud => "point cloud",
            Self::Manifest => "manifest",
            Self::Generated => "generated",
            Self::LiveRecording => "live recording",
            Self::Unknown => "unknown",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SourceGalleryThumbnailIntent {
    Image(SourceGalleryImageThumbnailIntent),
    Generic(SourceGalleryGenericThumbnailIntent),
}

#[allow(dead_code)]
impl SourceGalleryThumbnailIntent {
    fn from_kind_and_status(
        kind: SourceGalleryItemKind,
        external_reference_status: SourceExternalReferenceStatus,
        cache_key: String,
    ) -> Self {
        let status = SourceGalleryThumbnailStatus::from_external_reference_status(
            kind,
            external_reference_status,
        );
        if kind == SourceGalleryItemKind::Image {
            Self::Image(SourceGalleryImageThumbnailIntent { cache_key, status })
        } else {
            Self::Generic(SourceGalleryGenericThumbnailIntent { kind, status })
        }
    }

    pub fn status(&self) -> SourceGalleryThumbnailStatus {
        match self {
            Self::Image(intent) => intent.status,
            Self::Generic(intent) => intent.status,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceGalleryImageThumbnailIntent {
    pub cache_key: String,
    pub status: SourceGalleryThumbnailStatus,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SourceGalleryGenericThumbnailIntent {
    pub kind: SourceGalleryItemKind,
    pub status: SourceGalleryThumbnailStatus,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceGalleryThumbnailStatus {
    DecodeReady,
    GenericOnly,
    MissingSource,
    RemoteUnverified,
    RuntimeInput,
}

#[allow(dead_code)]
impl SourceGalleryThumbnailStatus {
    fn from_external_reference_status(
        kind: SourceGalleryItemKind,
        status: SourceExternalReferenceStatus,
    ) -> Self {
        match status {
            SourceExternalReferenceStatus::LocalAvailable
                if kind == SourceGalleryItemKind::Image =>
            {
                Self::DecodeReady
            }
            SourceExternalReferenceStatus::LocalAvailable => Self::GenericOnly,
            SourceExternalReferenceStatus::LocalMissing => Self::MissingSource,
            SourceExternalReferenceStatus::UriUnverified => Self::RemoteUnverified,
            SourceExternalReferenceStatus::RecordingQuery => Self::RuntimeInput,
            SourceExternalReferenceStatus::NotExternal => Self::GenericOnly,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::DecodeReady => "decode ready",
            Self::GenericOnly => "generic",
            Self::MissingSource => "missing",
            Self::RemoteUnverified => "remote unverified",
            Self::RuntimeInput => "runtime input",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SourceGalleryThumbnailCache {
    entries: Vec<SourceGalleryThumbnailCacheEntry>,
}

#[allow(dead_code)]
impl SourceGalleryThumbnailCache {
    pub fn store_decoded(
        &mut self,
        cache_key: impl Into<String>,
        thumbnail: SourceGalleryDecodedThumbnail,
    ) {
        let cache_key = cache_key.into();
        self.entries.retain(|entry| entry.cache_key != cache_key);
        self.entries.push(SourceGalleryThumbnailCacheEntry {
            cache_key,
            state: SourceGalleryThumbnailCacheState::Decoded(thumbnail),
        });
    }

    pub fn record_fetch_failure(&mut self, cache_key: impl Into<String>, error: impl Into<String>) {
        let cache_key = cache_key.into();
        self.entries.retain(|entry| entry.cache_key != cache_key);
        self.entries.push(SourceGalleryThumbnailCacheEntry {
            cache_key,
            state: SourceGalleryThumbnailCacheState::FetchFailed(error.into()),
        });
    }

    pub fn get(&self, cache_key: &str) -> Option<&SourceGalleryThumbnailCacheState> {
        self.entries
            .iter()
            .find(|entry| entry.cache_key == cache_key)
            .map(|entry| &entry.state)
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceGalleryThumbnailCacheEntry {
    pub cache_key: String,
    pub state: SourceGalleryThumbnailCacheState,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SourceGalleryThumbnailCacheState {
    Decoded(SourceGalleryDecodedThumbnail),
    FetchFailed(String),
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceGalleryDecodedThumbnail {
    pub width: u32,
    pub height: u32,
    pub rgba_bytes: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SourceGalleryManifestError {
    InvalidJson(String),
    MissingItems,
}

impl std::fmt::Display for SourceGalleryManifestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(error) => {
                write!(formatter, "invalid source gallery manifest: {error}")
            }
            Self::MissingItems => write!(
                formatter,
                "source gallery manifest does not contain any items"
            ),
        }
    }
}

impl std::error::Error for SourceGalleryManifestError {}

#[allow(dead_code)]
struct SourceGalleryManifest {
    entries: Vec<SourceGalleryManifestEntry>,
}

#[allow(dead_code)]
impl SourceGalleryManifest {
    fn parse(json: &str) -> Result<Self, SourceGalleryManifestError> {
        let manifest: SourceGalleryManifestJson = serde_json::from_str(json)
            .map_err(|error| SourceGalleryManifestError::InvalidJson(error.to_string()))?;
        let entries = manifest.into_entries();
        if entries.is_empty() {
            return Err(SourceGalleryManifestError::MissingItems);
        }
        Ok(Self { entries })
    }
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum SourceGalleryManifestJson {
    Items {
        items: Vec<SourceGalleryManifestEntryJson>,
    },
    List(Vec<SourceGalleryManifestEntryJson>),
}

#[allow(dead_code)]
impl SourceGalleryManifestJson {
    fn into_entries(self) -> Vec<SourceGalleryManifestEntry> {
        match self {
            Self::Items { items } | Self::List(items) => items
                .into_iter()
                .map(SourceGalleryManifestEntryJson::into_entry)
                .collect(),
        }
    }
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum SourceGalleryManifestEntryJson {
    Location(String),
    Object {
        location: String,
        #[serde(default)]
        label: Option<String>,
    },
}

#[allow(dead_code)]
impl SourceGalleryManifestEntryJson {
    fn into_entry(self) -> SourceGalleryManifestEntry {
        match self {
            Self::Location(location) => SourceGalleryManifestEntry {
                location,
                label: None,
            },
            Self::Object { location, label } => SourceGalleryManifestEntry { location, label },
        }
    }
}

#[allow(dead_code)]
struct SourceGalleryManifestEntry {
    location: String,
    label: Option<String>,
}

#[allow(dead_code)]
fn source_gallery_items_from_directory(
    directory: &Path,
    limit: usize,
    warnings: &mut Vec<String>,
) -> Vec<SourceGalleryItem> {
    let mut paths = match std::fs::read_dir(directory) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect::<Vec<_>>(),
        Err(error) => {
            warnings.push(format!(
                "Could not read gallery directory `{}`: {error}",
                directory.display()
            ));
            return Vec::new();
        }
    };
    paths.sort();
    if paths.len() > limit {
        warnings.push(format!(
            "Gallery directory `{}` has more than {limit} files; only the first {limit} are indexed.",
            directory.display()
        ));
    }
    paths
        .into_iter()
        .map(|path| {
            SourceGalleryItem::from_locator(SourceLocator::from_location(
                &path.display().to_string(),
            ))
        })
        .collect()
}

#[allow(dead_code)]
fn source_gallery_uri_requires_manifest(locator: &str) -> bool {
    let without_query = source_gallery_without_query(locator);
    without_query.ends_with('/')
}

#[allow(dead_code)]
fn source_gallery_is_image_locator(locator: &str) -> bool {
    matches!(
        source_gallery_extension(locator).as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "bmp" | "tif" | "tiff" | "webp")
    )
}

#[allow(dead_code)]
fn source_gallery_is_recording_locator(locator: &str) -> bool {
    matches!(source_gallery_extension(locator).as_deref(), Some("rrd"))
}

#[allow(dead_code)]
fn source_gallery_is_manifest_locator(locator: &str) -> bool {
    matches!(
        source_gallery_extension(locator).as_deref(),
        Some("manifest" | "source-gallery" | "gallery")
    ) || source_gallery_without_query(locator)
        .to_ascii_lowercase()
        .ends_with(".gallery.json")
}

#[allow(dead_code)]
fn source_gallery_extension(locator: &str) -> Option<String> {
    let without_query = source_gallery_without_query(locator).to_ascii_lowercase();
    Path::new(&without_query)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(ToOwned::to_owned)
}

#[allow(dead_code)]
fn source_gallery_without_query(locator: &str) -> &str {
    locator.split(['?', '#']).next().unwrap_or(locator)
}

#[allow(dead_code)]
fn source_gallery_display_name(locator: &SourceLocator) -> String {
    if let Some(label) = locator
        .label
        .as_deref()
        .filter(|label| !label.trim().is_empty())
    {
        return label.to_owned();
    }

    let readable = locator.readable();
    Path::new(source_gallery_without_query(&readable))
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or(readable)
}

#[allow(dead_code)]
fn source_gallery_stable_id(locator: &SourceLocator) -> String {
    format!("{}:{}", locator.kind.as_str(), locator.readable())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceExternalReferenceReport {
    pub status: SourceExternalReferenceStatus,
    pub readable_locator: String,
    pub bundle_relevant: bool,
    pub warning: Option<String>,
}

impl SourceExternalReferenceReport {
    fn from_locator(locator: &SourceLocator) -> Self {
        let readable_locator = locator.readable();
        match locator.kind {
            SourceLocatorKind::LocalPath => {
                let exists = locator
                    .location
                    .as_deref()
                    .is_some_and(|location| Path::new(location).exists());
                if exists {
                    Self {
                        status: SourceExternalReferenceStatus::LocalAvailable,
                        readable_locator,
                        bundle_relevant: true,
                        warning: Some(
                            "Local source remains an external reference unless explicitly bundled."
                                .to_owned(),
                        ),
                    }
                } else {
                    Self {
                        status: SourceExternalReferenceStatus::LocalMissing,
                        readable_locator: readable_locator.clone(),
                        bundle_relevant: true,
                        warning: Some(format!(
                            "Local source `{readable_locator}` is missing from this machine."
                        )),
                    }
                }
            }
            SourceLocatorKind::Uri => Self {
                status: SourceExternalReferenceStatus::UriUnverified,
                readable_locator,
                bundle_relevant: true,
                warning: Some("URI source remains an external reference; availability and content hash are unverified.".to_owned()),
            },
            SourceLocatorKind::RecordingQuery => Self {
                status: SourceExternalReferenceStatus::RecordingQuery,
                readable_locator,
                bundle_relevant: false,
                warning: Some(
                    "Recording query sources are live viewer inputs, not bundled project data."
                        .to_owned(),
                ),
            },
            SourceLocatorKind::Generated | SourceLocatorKind::Demo => Self {
                status: SourceExternalReferenceStatus::NotExternal,
                readable_locator,
                bundle_relevant: false,
                warning: None,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SourceExternalReferenceStatus {
    NotExternal,
    LocalAvailable,
    LocalMissing,
    UriUnverified,
    RecordingQuery,
}

impl SourceExternalReferenceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotExternal => "not external",
            Self::LocalAvailable => "local available",
            Self::LocalMissing => "local missing",
            Self::UriUnverified => "uri unverified",
            Self::RecordingQuery => "recording query",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceBundlePreview {
    pub item: SourceBundlePreviewItem,
    pub expected_size_bytes: Option<u64>,
    pub remaining_external_reference_count: usize,
    pub missing_reference_count: usize,
    pub reproducibility_warnings: Vec<String>,
}

impl SourceBundlePreview {
    fn from_external_reference(reference: SourceExternalReferenceReport) -> Self {
        let (inclusion, expected_size_bytes, warning) = match reference.status {
            SourceExternalReferenceStatus::NotExternal => {
                (SourceBundleInclusion::NotExternal, None, None)
            }
            SourceExternalReferenceStatus::RecordingQuery => (
                SourceBundleInclusion::LiveInput,
                None,
                reference.warning.clone(),
            ),
            SourceExternalReferenceStatus::UriUnverified => (
                SourceBundleInclusion::ReferenceOnly,
                None,
                reference.warning.clone(),
            ),
            SourceExternalReferenceStatus::LocalMissing => (
                SourceBundleInclusion::Missing,
                None,
                reference.warning.clone(),
            ),
            SourceExternalReferenceStatus::LocalAvailable => {
                let expected_size_bytes = std::fs::metadata(&reference.readable_locator)
                    .ok()
                    .map(|metadata| metadata.len());
                (
                    SourceBundleInclusion::IncludeAvailable,
                    expected_size_bytes,
                    Some(format!(
                        "Local source `{}` can be included, but no content hash has been computed.",
                        reference.readable_locator
                    )),
                )
            }
        };

        let mut reproducibility_warnings = Vec::new();
        if let Some(warning) = warning {
            reproducibility_warnings.push(warning);
        }

        let remaining_external_reference_count =
            usize::from(inclusion == SourceBundleInclusion::ReferenceOnly);
        let missing_reference_count = usize::from(inclusion == SourceBundleInclusion::Missing);

        Self {
            item: SourceBundlePreviewItem {
                locator: reference.readable_locator,
                inclusion,
                expected_size_bytes,
            },
            expected_size_bytes,
            remaining_external_reference_count,
            missing_reference_count,
            reproducibility_warnings,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceBundlePreviewItem {
    pub locator: String,
    pub inclusion: SourceBundleInclusion,
    pub expected_size_bytes: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceBundleInclusion {
    NotExternal,
    IncludeAvailable,
    ReferenceOnly,
    Missing,
    LiveInput,
}

impl SourceBundleInclusion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotExternal => "not external",
            Self::IncludeAvailable => "include available",
            Self::ReferenceOnly => "reference only",
            Self::Missing => "missing",
            Self::LiveInput => "live input",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct SourceExternalReferenceActionReport {
    pub recommended: SourceExternalReferenceActionHint,
    pub secondary: Vec<SourceExternalReferenceActionHint>,
}

impl SourceExternalReferenceActionReport {
    fn from_metadata(metadata: &SourceMetadata) -> Self {
        let reference = metadata.external_reference_report();
        let bundle_preview = metadata.bundle_preview();
        let package_manifest = metadata.package_manifest_preview();
        let locator = reference.readable_locator.clone();

        match reference.status {
            SourceExternalReferenceStatus::NotExternal => Self {
                recommended: SourceExternalReferenceActionHint::new(
                    SourceExternalReferenceActionKind::InspectGeneratedSource,
                    "Inspect generated source",
                    "Generated sources are graph-owned and do not need external reference actions.",
                ),
                secondary: Vec::new(),
            },
            SourceExternalReferenceStatus::LocalAvailable => {
                let bundled_path = package_manifest
                    .artifacts
                    .first()
                    .and_then(|artifact| artifact.bundled_path.as_deref())
                    .unwrap_or("sources/source");
                Self {
                    recommended: SourceExternalReferenceActionHint::new(
                        SourceExternalReferenceActionKind::IncludeDuringPackageExport,
                        "Include during package/export",
                        format!(
                            "Eligible for explicit package/export inclusion at `{bundled_path}`."
                        ),
                    ),
                    secondary: vec![
                        SourceExternalReferenceActionHint::new(
                            SourceExternalReferenceActionKind::RevealLocalPath,
                            "Reveal local path",
                            format!("Inspect local source `{locator}` outside the project file."),
                        ),
                        SourceExternalReferenceActionHint::new(
                            SourceExternalReferenceActionKind::CopyLocator,
                            "Copy locator",
                            format!("Copy `{locator}` for sharing or relinking."),
                        ),
                    ],
                }
            }
            SourceExternalReferenceStatus::LocalMissing => Self {
                recommended: SourceExternalReferenceActionHint::new(
                    SourceExternalReferenceActionKind::RelinkMissingSource,
                    "Relink missing source",
                    format!("Choose a replacement for missing source `{locator}`."),
                ),
                secondary: vec![SourceExternalReferenceActionHint::new(
                    SourceExternalReferenceActionKind::CopyLocator,
                    "Copy locator",
                    format!("Copy missing locator `{locator}` for troubleshooting."),
                )],
            },
            SourceExternalReferenceStatus::UriUnverified => Self {
                recommended: SourceExternalReferenceActionHint::new(
                    SourceExternalReferenceActionKind::KeepUriReference,
                    "Keep URI reference",
                    format!(
                        "URI sources stay reference-only; {} external reference remains.",
                        bundle_preview.remaining_external_reference_count
                    ),
                ),
                secondary: vec![SourceExternalReferenceActionHint::new(
                    SourceExternalReferenceActionKind::CopyLocator,
                    "Copy locator",
                    format!("Copy URI `{locator}` for sharing or resolver setup."),
                )],
            },
            SourceExternalReferenceStatus::RecordingQuery => Self {
                recommended: SourceExternalReferenceActionHint::new(
                    SourceExternalReferenceActionKind::InspectLiveInput,
                    "Inspect live input",
                    "Recording query sources are live inputs and are not package artifacts.",
                ),
                secondary: Vec::new(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct SourceExternalReferenceActionHint {
    pub kind: SourceExternalReferenceActionKind,
    pub label: String,
    pub detail: String,
}

impl SourceExternalReferenceActionHint {
    fn new(
        kind: SourceExternalReferenceActionKind,
        label: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            label: label.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SourceExternalReferenceActionKind {
    InspectGeneratedSource,
    CopyLocator,
    RevealLocalPath,
    IncludeDuringPackageExport,
    RelinkMissingSource,
    KeepUriReference,
    InspectLiveInput,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct SourcePackageManifestPreview {
    pub schema_version: u32,
    pub artifacts: Vec<SourcePackageManifestArtifact>,
    pub expected_size_bytes: Option<u64>,
    pub remaining_external_reference_count: usize,
    pub missing_reference_count: usize,
    pub reproducibility_warnings: Vec<String>,
}

impl SourcePackageManifestPreview {
    const SCHEMA_VERSION: u32 = 1;

    fn from_source_metadata(
        metadata: &SourceMetadata,
        bundle_preview: SourceBundlePreview,
    ) -> Self {
        Self::from_source_metadata_with_choice(
            metadata,
            bundle_preview,
            SourcePackageManifestInclusionChoice::Default,
        )
    }

    fn from_source_metadata_with_choice(
        metadata: &SourceMetadata,
        bundle_preview: SourceBundlePreview,
        choice: SourcePackageManifestInclusionChoice,
    ) -> Self {
        let original_inclusion = bundle_preview.item.inclusion;
        let inclusion = choice.apply_to_bundle_inclusion(original_inclusion);
        let artifact_size_bytes = bundle_preview.item.expected_size_bytes;
        let expected_size_bytes = if inclusion == SourceBundleInclusion::IncludeAvailable {
            artifact_size_bytes
        } else {
            None
        };
        let remaining_external_reference_count =
            usize::from(inclusion == SourceBundleInclusion::ReferenceOnly);
        let missing_reference_count = usize::from(inclusion == SourceBundleInclusion::Missing);
        let mut reproducibility_warnings = bundle_preview.reproducibility_warnings;
        if original_inclusion == SourceBundleInclusion::IncludeAvailable
            && inclusion == SourceBundleInclusion::ReferenceOnly
        {
            reproducibility_warnings.push(format!(
                "Local source `{}` was left external by explicit package/export choice.",
                bundle_preview.item.locator
            ));
        }

        let artifact = SourcePackageManifestArtifact {
            role: SourcePackageManifestArtifactRole::from_bundle_inclusion(inclusion),
            original_locator: bundle_preview.item.locator.clone(),
            bundled_path: if inclusion == SourceBundleInclusion::IncludeAvailable {
                Some(source_package_manifest_bundled_path(
                    &bundle_preview.item.locator,
                ))
            } else {
                None
            },
            size_bytes: artifact_size_bytes,
            content_hash: None,
            source_provenance: metadata.provenance,
            external_status: SourcePackageManifestExternalStatus::from_bundle_inclusion(inclusion),
        };

        Self {
            schema_version: Self::SCHEMA_VERSION,
            artifacts: vec![artifact],
            expected_size_bytes,
            remaining_external_reference_count,
            missing_reference_count,
            reproducibility_warnings,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SourcePackageManifestInclusionChoice {
    #[default]
    Default,
    IncludeAvailable,
    ReferenceOnly,
}

impl SourcePackageManifestInclusionChoice {
    fn apply_to_bundle_inclusion(self, inclusion: SourceBundleInclusion) -> SourceBundleInclusion {
        match (self, inclusion) {
            (Self::ReferenceOnly, SourceBundleInclusion::IncludeAvailable) => {
                SourceBundleInclusion::ReferenceOnly
            }
            (Self::IncludeAvailable, SourceBundleInclusion::IncludeAvailable) => {
                SourceBundleInclusion::IncludeAvailable
            }
            _ => inclusion,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourcePackageManifestWriteResult {
    pub path: PathBuf,
    pub artifact_count: usize,
    pub expected_size_bytes: Option<u64>,
    pub remaining_external_reference_count: usize,
    pub missing_reference_count: usize,
    pub reproducibility_warning_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct SourcePackageManifestArtifact {
    pub role: SourcePackageManifestArtifactRole,
    pub original_locator: String,
    pub bundled_path: Option<String>,
    pub size_bytes: Option<u64>,
    pub content_hash: Option<String>,
    pub source_provenance: SourceProvenance,
    pub external_status: SourcePackageManifestExternalStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SourcePackageManifestArtifactRole {
    SourceDataset,
    LiveRecordingQuery,
    GeneratedSource,
}

impl SourcePackageManifestArtifactRole {
    fn from_bundle_inclusion(inclusion: SourceBundleInclusion) -> Self {
        match inclusion {
            SourceBundleInclusion::NotExternal => Self::GeneratedSource,
            SourceBundleInclusion::LiveInput => Self::LiveRecordingQuery,
            SourceBundleInclusion::IncludeAvailable
            | SourceBundleInclusion::ReferenceOnly
            | SourceBundleInclusion::Missing => Self::SourceDataset,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SourcePackageManifestExternalStatus {
    NotExternal,
    IncludedPendingWrite,
    ReferenceOnly,
    Missing,
    LiveInput,
}

impl SourcePackageManifestExternalStatus {
    fn from_bundle_inclusion(inclusion: SourceBundleInclusion) -> Self {
        match inclusion {
            SourceBundleInclusion::NotExternal => Self::NotExternal,
            SourceBundleInclusion::IncludeAvailable => Self::IncludedPendingWrite,
            SourceBundleInclusion::ReferenceOnly => Self::ReferenceOnly,
            SourceBundleInclusion::Missing => Self::Missing,
            SourceBundleInclusion::LiveInput => Self::LiveInput,
        }
    }
}

fn source_package_manifest_bundled_path(locator: &str) -> String {
    let file_name = Path::new(locator)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("source");
    format!(
        "sources/{}",
        sanitize_package_manifest_path_component(file_name)
    )
}

fn sanitize_package_manifest_path_component(component: &str) -> String {
    let sanitized = component
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();

    if sanitized.is_empty() {
        "source".to_owned()
    } else {
        sanitized
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct SourceFormatCapability {
    pub kind: SourceFormatKind,
    pub status: SourceFormatSupportStatus,
    pub geometry_kinds: Vec<HoudiniGeometryKind>,
    pub notes: String,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SourceFormatKind {
    Parquet,
    GeoParquetLike,
    GeoJson,
    FlatGeobuf,
    CsvCoordinates,
    LasLazPointCloud,
    SqliteTableOrView,
    GeoPackage,
    SpatiaLite,
    Shapefile,
}

#[allow(dead_code)]
impl SourceFormatKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Parquet => "Parquet",
            Self::GeoParquetLike => "GeoParquet-like table",
            Self::GeoJson => "GeoJSON",
            Self::FlatGeobuf => "FlatGeobuf",
            Self::CsvCoordinates => "CSV coordinates",
            Self::LasLazPointCloud => "LAS/LAZ point cloud",
            Self::SqliteTableOrView => "SQLite table/view",
            Self::GeoPackage => "GeoPackage",
            Self::SpatiaLite => "SpatiaLite",
            Self::Shapefile => "Shapefile",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SourceFormatSupportStatus {
    Supported,
    PlannedV1,
    LaterCompatibility,
    Deferred,
}

#[allow(dead_code)]
impl SourceFormatSupportStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::PlannedV1 => "planned v1",
            Self::LaterCompatibility => "later compatibility",
            Self::Deferred => "deferred",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct SourceFormatInferenceReport {
    pub status: SourceFormatInferenceStatus,
    pub readable_locator: String,
    pub kind: Option<SourceFormatKind>,
    pub support_status: Option<SourceFormatSupportStatus>,
    pub notes: String,
}

impl SourceFormatInferenceReport {
    fn from_metadata(metadata: &SourceMetadata) -> Self {
        let readable_locator = metadata.locator.readable();
        match metadata.locator.kind {
            SourceLocatorKind::Demo | SourceLocatorKind::Generated => Self {
                status: SourceFormatInferenceStatus::Generated,
                readable_locator,
                kind: None,
                support_status: None,
                notes: "Generated graph sources do not require a dataset parser.".to_owned(),
            },
            SourceLocatorKind::RecordingQuery => Self {
                status: SourceFormatInferenceStatus::LiveInput,
                readable_locator,
                kind: None,
                support_status: None,
                notes: "Recording query sources are live viewer inputs, not source files."
                    .to_owned(),
            },
            SourceLocatorKind::LocalPath | SourceLocatorKind::Uri => {
                let Some(kind) = infer_source_format_kind(&readable_locator) else {
                    return Self {
                        status: SourceFormatInferenceStatus::Unknown,
                        readable_locator,
                        kind: None,
                        support_status: None,
                        notes: "No known v1 source format could be inferred from the locator."
                            .to_owned(),
                    };
                };

                let capability = source_format_capabilities()
                    .into_iter()
                    .find(|capability| capability.kind == kind);
                let (support_status, notes) = capability
                    .map(|capability| (Some(capability.status), capability.notes))
                    .unwrap_or_else(|| {
                        (
                            None,
                            "No source capability record exists for the inferred format."
                                .to_owned(),
                        )
                    });

                Self {
                    status: SourceFormatInferenceStatus::Inferred,
                    readable_locator,
                    kind: Some(kind),
                    support_status,
                    notes,
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SourceFormatInferenceStatus {
    Generated,
    LiveInput,
    Inferred,
    Unknown,
}

impl SourceFormatInferenceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Generated => "generated",
            Self::LiveInput => "live input",
            Self::Inferred => "inferred",
            Self::Unknown => "unknown",
        }
    }
}

fn infer_source_format_kind(locator: &str) -> Option<SourceFormatKind> {
    let without_query = locator.split(['?', '#']).next().unwrap_or(locator);
    let lower = without_query.to_ascii_lowercase();

    if lower.ends_with(".geoparquet") {
        return Some(SourceFormatKind::GeoParquetLike);
    }

    let extension = Path::new(&lower).extension()?.to_str()?;
    match extension {
        "parquet" => Some(SourceFormatKind::Parquet),
        "geojson" | "json" => Some(SourceFormatKind::GeoJson),
        "fgb" => Some(SourceFormatKind::FlatGeobuf),
        "csv" | "tsv" => Some(SourceFormatKind::CsvCoordinates),
        "las" | "laz" => Some(SourceFormatKind::LasLazPointCloud),
        "sqlite" | "sqlite3" | "db" => Some(SourceFormatKind::SqliteTableOrView),
        "gpkg" => Some(SourceFormatKind::GeoPackage),
        "spatialite" => Some(SourceFormatKind::SpatiaLite),
        "shp" => Some(SourceFormatKind::Shapefile),
        _ => None,
    }
}

#[allow(dead_code)]
fn source_format_capabilities() -> Vec<SourceFormatCapability> {
    use SourceFormatKind as Kind;
    use SourceFormatSupportStatus as Status;

    vec![
        SourceFormatCapability {
            kind: Kind::Parquet,
            status: Status::Supported,
            geometry_kinds: vec![HoudiniGeometryKind::CubicBezier],
            notes: "Current native cubic Bezier import path expects eight control-point columns."
                .to_owned(),
        },
        SourceFormatCapability {
            kind: Kind::GeoParquetLike,
            status: Status::PlannedV1,
            geometry_kinds: vec![HoudiniGeometryKind::Polygon, HoudiniGeometryKind::CubicBezier],
            notes: "Planned as tabular geometry source metadata without changing native cubic storage."
                .to_owned(),
        },
        SourceFormatCapability {
            kind: Kind::GeoJson,
            status: Status::PlannedV1,
            geometry_kinds: vec![HoudiniGeometryKind::Polygon],
            notes: "Planned v1 source format for polygon-heavy exploration workflows.".to_owned(),
        },
        SourceFormatCapability {
            kind: Kind::FlatGeobuf,
            status: Status::PlannedV1,
            geometry_kinds: vec![HoudiniGeometryKind::Polygon],
            notes: "Planned v1 source format for portable feature datasets.".to_owned(),
        },
        SourceFormatCapability {
            kind: Kind::CsvCoordinates,
            status: Status::PlannedV1,
            geometry_kinds: vec![HoudiniGeometryKind::Polygon],
            notes: "Planned v1 source format when coordinate or geometry columns are configured."
                .to_owned(),
        },
        SourceFormatCapability {
            kind: Kind::LasLazPointCloud,
            status: Status::PlannedV1,
            geometry_kinds: Vec::new(),
            notes: "Planned v1 source family; point-cloud geometry records are not modeled in this slice."
                .to_owned(),
        },
        SourceFormatCapability {
            kind: Kind::SqliteTableOrView,
            status: Status::PlannedV1,
            geometry_kinds: vec![HoudiniGeometryKind::Polygon],
            notes: "Planned v1 source format for generic tables or views with configurable geometry columns."
                .to_owned(),
        },
        SourceFormatCapability {
            kind: Kind::GeoPackage,
            status: Status::LaterCompatibility,
            geometry_kinds: vec![HoudiniGeometryKind::Polygon],
            notes: "Later compatibility layer, not the first v1 source-format implementation."
                .to_owned(),
        },
        SourceFormatCapability {
            kind: Kind::SpatiaLite,
            status: Status::LaterCompatibility,
            geometry_kinds: vec![HoudiniGeometryKind::Polygon],
            notes: "Later compatibility layer after the projection-agnostic source model is stable."
                .to_owned(),
        },
        SourceFormatCapability {
            kind: Kind::Shapefile,
            status: Status::Deferred,
            geometry_kinds: vec![HoudiniGeometryKind::Polygon],
            notes: "Explicitly deferred because legacy packaging, encoding, and CRS expectations conflict with v1 scope."
                .to_owned(),
        },
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SourceProvenance {
    DemoFallback,
    ParquetImport,
    RecordingQuery,
    SyntheticBenchmark,
    SyntheticMalware,
    PythonOperator,
}

impl SourceProvenance {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DemoFallback => "demo fallback",
            Self::ParquetImport => "parquet import",
            Self::RecordingQuery => "recording query",
            Self::SyntheticBenchmark => "synthetic benchmark",
            Self::SyntheticMalware => "synthetic malware",
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

    fn malware_starter(metadata: SourceMetadata) -> Self {
        Self {
            mode: GraphSourceMode::SyntheticMalware,
            matching_entity_count: metadata.record_count,
            visible_data_result_count: metadata.record_count,
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
                locator: if has_recording_input {
                    SourceLocator::recording_query()
                } else {
                    SourceLocator::demo()
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
            GraphSourceMode::SyntheticMalware => "synthetic malware",
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
        let source_path = source_path.display().to_string();
        self.source_path = Some(source_path.clone());
        self.metadata.source_path = Some(source_path.clone());
        self.metadata.locator = SourceLocator::from_location(&source_path);
        self.import_error = Some(format!("{err}"));
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum GraphSourceMode {
    DemoFallback,
    RecordingQuery,
    SyntheticMalware,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct HoudiniGraphSidecar {
    version: u32,
    source: GraphSourceSidecar,
    #[serde(default)]
    graph_registry: ProjectGraphRegistry,
    #[serde(default)]
    graph_containers: Vec<GraphContainerMetadata>,
    #[serde(default)]
    data_flow_edges: Vec<GraphDataFlowEdge>,
    nodes: Vec<NodeSidecar>,
    #[serde(default)]
    annotations: Vec<GraphAnnotation>,
    #[serde(default)]
    network_view: NetworkViewDisplayOptions,
    layers: Vec<LayerSidecar>,
    #[serde(default)]
    style: GraphStyle,
    #[serde(default)]
    substrate_raster: Option<SubstrateRaster>,
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
    #[serde(default)]
    evaluation_mode: GraphEvaluationMode,
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
            graph_registry: graph.graph_registry.clone(),
            graph_containers: graph.graph_containers.clone(),
            data_flow_edges: graph.data_flow_edges.clone(),
            nodes: graph
                .nodes
                .iter()
                .map(|node| NodeSidecar {
                    node_id: node.node_id.clone(),
                    name: node.name.clone(),
                    parent_graph_id: graph.node_parent_graph_id(node).to_owned(),
                    kind: node.kind,
                    layout_position: node.layout_position,
                    parameter_value: node.parameter.value,
                    parameter_rule: node.parameter.rule_spec.clone(),
                    generated: node.generated,
                    coordinate_contract: Some(node.coordinate_contract.clone()),
                    source_node: node.source_node.clone(),
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
            substrate_raster: graph.substrate_raster.clone(),
            demo_geometry: graph.geometry.clone(),
            recording_geometry: graph.recording_geometry.clone(),
            python_operator_declarations: graph.python_operator_declarations.clone(),
            procedural_asset_declarations: graph.procedural_asset_declarations.clone(),
            native_operator_declarations: graph.native_operator_declarations.clone(),
            native_operator_trust: graph.native_operator_trust.clone(),
            python_environment: graph.python_environment.clone(),
            evaluation_mode: graph.evaluation_mode,
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
            metadata: self.source.metadata.normalized(),
            import_error: self.source.import_error,
        };
        graph.graph_registry = self.graph_registry.normalize();
        graph.graph_containers = self.graph_containers;
        graph.data_flow_edges = self.data_flow_edges;
        graph.geometry = self.demo_geometry;
        graph.recording_geometry = self.recording_geometry;
        graph.annotations = self.annotations;
        graph.network_view = self.network_view;
        graph.style = self.style;
        graph.substrate_raster = self.substrate_raster.or_else(|| {
            (graph.source.mode == GraphSourceMode::SyntheticMalware)
                .then(SubstrateRaster::mock_malware_byteplot)
        });
        graph.python_operator_declarations = self.python_operator_declarations;
        graph.procedural_asset_declarations = self.procedural_asset_declarations;
        graph.native_operator_declarations = self.native_operator_declarations;
        graph.native_operator_trust = self.native_operator_trust;
        graph.python_environment = self.python_environment;
        graph.evaluation_mode = self.evaluation_mode;
        graph.command_history = ProjectCommandHistory::default();
        graph.work_items.clear();

        let mut matched_node_indices = vec![false; graph.nodes.len()];
        for (snapshot_index, node_snapshot) in self.nodes.into_iter().enumerate() {
            let parent_graph_id = node_snapshot.parent_graph_id_or_main();
            let matching_node_index = graph.nodes.iter().enumerate().position(|(index, node)| {
                !matched_node_indices[index]
                    && node.kind == node_snapshot.kind
                    && node_matches_snapshot_identity(node, &node_snapshot)
            });
            if let Some(node_index) = matching_node_index {
                matched_node_indices[node_index] = true;
                let Some(node) = graph.nodes.get_mut(node_index) else {
                    continue;
                };
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
                node.parent_graph_id = parent_graph_id;
                node.generated = node_snapshot.generated;
                node.coordinate_contract = node_snapshot.coordinate_contract.unwrap_or_else(|| {
                    GraphDocument::default_coordinate_contract_for_kind(node.kind)
                });
                node.source_node = node_snapshot.source_node;
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
                matched_node_indices.insert(insert_index, true);
            }
        }

        if graph.data_flow_edges.is_empty() {
            graph.rebuild_default_data_flow_edges();
        }
        graph.normalize_graph_containers();

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
    #[serde(default)]
    parent_graph_id: String,
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
    source_node: Option<SourceNode>,
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
    fn parent_graph_id_or_main(&self) -> String {
        if self.parent_graph_id.is_empty() {
            MAIN_GRAPH_ID.to_owned()
        } else {
            self.parent_graph_id.clone()
        }
    }

    fn is_instance_node(&self) -> bool {
        matches!(
            self.kind,
            NodeKind::Source
                | NodeKind::Null
                | NodeKind::ReferenceInput
                | NodeKind::SubstrateProjection
                | NodeKind::GraphContainer
                | NodeKind::PythonOperator
                | NodeKind::ProceduralAsset
                | NodeKind::NativeOperator
        )
    }

    fn into_instance_node(self) -> GraphNode {
        let parent_graph_id = self.parent_graph_id_or_main();
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
            NodeKind::GraphContainer => (
                "Graph Container",
                "Navigates to an internal named graph without moving nodes yet.",
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
            parent_graph_id,
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
            source_node: self.source_node,
            output_operator: self.output_operator,
            null_operator: self.null_operator,
            reference_input: self.reference_input,
            substrate_projection: self.substrate_projection,
            python_operator: self.python_operator,
            procedural_asset: self.procedural_asset,
            native_operator: self.native_operator,
            evaluation: NodeEvaluation::clean(),
            participates_in_output: self.kind != NodeKind::GraphContainer,
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
        NodeKind::Null
        | NodeKind::ReferenceInput
        | NodeKind::SubstrateProjection
        | NodeKind::GraphContainer => {
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

#[allow(dead_code)]
fn path_exists<'a>(
    start: &'a str,
    target: &str,
    adjacency: &std::collections::BTreeMap<&'a str, Vec<&'a str>>,
) -> bool {
    let mut stack = vec![start];
    let mut visited = std::collections::BTreeSet::new();
    while let Some(node_id) = stack.pop() {
        if node_id == target {
            return true;
        }
        if !visited.insert(node_id) {
            continue;
        }
        if let Some(next_nodes) = adjacency.get(node_id) {
            stack.extend(next_nodes.iter().copied());
        }
    }
    false
}

fn readable_reference_path(graph_id: &str, node: &GraphNode, output_name: &str) -> String {
    format!("{graph_id}/{}:{output_name}", node.name)
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

#[derive(Clone)]
pub(crate) struct GraphNode {
    pub node_id: String,
    pub parent_graph_id: String,
    pub name: String,
    pub kind: NodeKind,
    pub layout_position: GraphPoint,
    pub generated: Option<GeneratedNodeInfo>,
    pub coordinate_contract: Option<SubstrateCoordinateContract>,
    pub source_node: Option<SourceNode>,
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
    #[serde(default = "default_main_graph_id")]
    pub parent_graph_id: String,
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
        parent_graph_id: String,
        title: String,
        position: GraphPoint,
        size: GraphPoint,
        member_node_ids: Vec<String>,
    ) -> Self {
        Self {
            annotation_id,
            parent_graph_id,
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
        parent_graph_id: String,
        title: String,
        text: String,
        position: GraphPoint,
        size: GraphPoint,
    ) -> Self {
        Self {
            annotation_id,
            parent_graph_id,
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
    pub comment_display_mode: NetworkCommentDisplayMode,
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
    pub unload_badge: NetworkBadgeVisibility,
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
            comment_display_mode: NetworkCommentDisplayMode::ManualOnly,
            error_badge: NetworkBadgeVisibility::Large,
            warning_badge: NetworkBadgeVisibility::Normal,
            comment_badge: NetworkBadgeVisibility::Large,
            time_dependent_badge: NetworkBadgeVisibility::Normal,
            lock_badge: NetworkBadgeVisibility::Normal,
            unload_badge: NetworkBadgeVisibility::Hide,
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

fn default_main_graph_id() -> String {
    MAIN_GRAPH_ID.to_owned()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum NetworkCommentDisplayMode {
    #[default]
    ManualOnly,
    AllCommented,
}

impl NetworkCommentDisplayMode {
    pub const ALL: [Self; 2] = [Self::ManualOnly, Self::AllCommented];

    pub fn label(self) -> &'static str {
        match self {
            Self::ManualOnly => "Manual",
            Self::AllCommented => "All Commented",
        }
    }

    pub fn shows_comment(self, comment: &str, show_comment_in_network: bool) -> bool {
        if comment.trim().is_empty() {
            return false;
        }
        match self {
            Self::ManualOnly => show_comment_in_network,
            Self::AllCommented => true,
        }
    }
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
    fn source_node(node_id: String, name: String, source_node: SourceNode) -> Self {
        Self {
            node_id,
            parent_graph_id: MAIN_GRAPH_ID.to_owned(),
            name,
            kind: NodeKind::Source,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: None,
            source_node: Some(source_node),
            output_operator: None,
            null_operator: None,
            reference_input: None,
            substrate_projection: None,
            python_operator: None,
            procedural_asset: None,
            native_operator: None,
            evaluation: NodeEvaluation::clean(),
            participates_in_output: false,
            comment: String::new(),
            show_comment_in_network: false,
            parameter: NodeParameter::scalar(
                "Available",
                1.0,
                0.0..=1.0,
                "External source locator placeholder; contents remain referenced, not embedded.",
            ),
            info: "References one or more external source locators without copying data into the graph.",
        }
    }

    fn null_operator(name: String) -> Self {
        Self {
            node_id: String::new(),
            parent_graph_id: MAIN_GRAPH_ID.to_owned(),
            name,
            kind: NodeKind::Null,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
            source_node: None,
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
            parent_graph_id: MAIN_GRAPH_ID.to_owned(),
            name: "Reference Input".to_owned(),
            kind: NodeKind::ReferenceInput,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
            source_node: None,
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
            parent_graph_id: MAIN_GRAPH_ID.to_owned(),
            name: "Substrate Projection".to_owned(),
            kind: NodeKind::SubstrateProjection,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: Some(projection.to_contract.clone()),
            source_node: None,
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

    fn graph_container(node_id: String, name: String) -> Self {
        Self {
            node_id,
            parent_graph_id: MAIN_GRAPH_ID.to_owned(),
            name,
            kind: NodeKind::GraphContainer,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: None,
            source_node: None,
            output_operator: None,
            null_operator: None,
            reference_input: None,
            substrate_projection: None,
            python_operator: None,
            procedural_asset: None,
            native_operator: None,
            evaluation: NodeEvaluation::clean(),
            participates_in_output: false,
            comment: String::new(),
            show_comment_in_network: false,
            parameter: NodeParameter::scalar(
                "Boundary",
                0.0,
                0.0..=1.0,
                "Graph container boundary placeholder; typed ports land in a later slice.",
            ),
            info: "Subnet-like graph container that points to an internal named graph without moving nodes yet.",
        }
    }

    fn python_operator(instance_id: String, declaration_id: String) -> Self {
        Self {
            node_id: instance_id.clone(),
            parent_graph_id: MAIN_GRAPH_ID.to_owned(),
            name: "Python Operator".to_owned(),
            kind: NodeKind::PythonOperator,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
            source_node: None,
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
            parent_graph_id: MAIN_GRAPH_ID.to_owned(),
            name: "Asset".to_owned(),
            kind: NodeKind::ProceduralAsset,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
            source_node: None,
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
            parent_graph_id: MAIN_GRAPH_ID.to_owned(),
            name: "Native Operator".to_owned(),
            kind: NodeKind::NativeOperator,
            layout_position: GraphPoint::new(0.5, 0.5),
            generated: None,
            coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
            source_node: None,
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

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct SourceNode {
    pub entries: Vec<SourceNodeEntry>,
}

impl SourceNode {
    fn from_gallery_item(item: &SourceGalleryItem) -> Self {
        Self {
            entries: vec![SourceNodeEntry::from_gallery_item(item)],
        }
    }

    fn from_gallery_items(items: &[SourceGalleryItem]) -> Option<Self> {
        (!items.is_empty()).then(|| Self {
            entries: items
                .iter()
                .map(SourceNodeEntry::from_gallery_item)
                .collect(),
        })
    }

    fn primary_metadata(&self) -> Option<SourceMetadata> {
        self.entries.first().map(SourceNodeEntry::source_metadata)
    }

    fn source_count(&self) -> usize {
        self.entries.len()
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct SourceNodeEntry {
    pub display_name: String,
    pub locator: SourceLocator,
    pub kind: SourceGalleryItemKind,
    pub external_reference_status: SourceExternalReferenceStatus,
    pub format_kind: Option<SourceFormatKind>,
    pub format_support_status: Option<SourceFormatSupportStatus>,
}

impl SourceNodeEntry {
    fn from_gallery_item(item: &SourceGalleryItem) -> Self {
        Self {
            display_name: item.display_name.clone(),
            locator: item.locator.clone(),
            kind: item.kind,
            external_reference_status: item.external_reference_status,
            format_kind: item.format_kind,
            format_support_status: item.format_support_status,
        }
    }

    fn source_metadata(&self) -> SourceMetadata {
        SourceMetadata {
            provenance: SourceProvenance::ParquetImport,
            source_path: self.locator.location.clone(),
            locator: self.locator.clone(),
            ..Default::default()
        }
        .normalized()
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

    fn malware_byteplot() -> Self {
        Self {
            substrate_id: "malware-byteplot-pixel-space".to_owned(),
            width: 256,
            height: 256,
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct SubstrateRaster {
    pub substrate_id: String,
    pub display_name: String,
    pub width: u32,
    pub height: u32,
    pub color_model: SubstrateRasterColorModel,
    pub source_path: Option<String>,
    pub recipe: SubstrateRasterRecipe,
}

impl SubstrateRaster {
    fn mock_malware_byteplot() -> Self {
        Self {
            substrate_id: "malware-byteplot-pixel-space".to_owned(),
            display_name: "Mock malware byteplot".to_owned(),
            width: 256,
            height: 256,
            color_model: SubstrateRasterColorModel::L8,
            source_path: Some("examples/malware_byteplot/mock-byteplot.png".to_owned()),
            recipe: SubstrateRasterRecipe::MockMalwareByteplot,
        }
    }

    pub fn byte_len(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    pub fn format_summary(&self) -> String {
        format!(
            "{} {}x{}",
            self.color_model.as_str(),
            self.width,
            self.height
        )
    }

    fn recording_entity_path(&self) -> String {
        format!(
            "houdini_graph/substrates/{}",
            sanitize_entity_path_part(&self.substrate_id)
        )
    }

    fn luma8_pixels(&self) -> Vec<u8> {
        match self.recipe {
            SubstrateRasterRecipe::MockMalwareByteplot => {
                let mut pixels = Vec::with_capacity(self.byte_len());
                for y in 0..self.height {
                    for x in 0..self.width {
                        let gradient = ((x.wrapping_mul(5) + y.wrapping_mul(3)) & 0xff) as u8;
                        let band = if (x / 16 + y / 32) % 2 == 0 { 34 } else { 0 };
                        let section = if (48..112).contains(&x) && (24..120).contains(&y) {
                            58
                        } else if (132..228).contains(&x) && (36..132).contains(&y) {
                            42
                        } else if (28..144).contains(&x) && (136..224).contains(&y) {
                            64
                        } else if (130..240).contains(&x) && (136..236).contains(&y) {
                            50
                        } else {
                            0
                        };
                        let texture =
                            (((x ^ y).wrapping_mul(13) + (x * y).wrapping_mul(3)) & 0x1f) as u8;
                        pixels
                            .push(gradient.saturating_add(band).saturating_add(section) ^ texture);
                    }
                }
                pixels
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn recording_metadata_markdown(&self) -> String {
        format!(
            "# Substrate raster\n\n\
             Name: `{}`\n\n\
             Substrate id: `{}`\n\n\
             Format: `{}`\n\n\
             Byte length: `{}`\n\n\
             Source path: `{}`\n\n\
             Recipe: `{:?}`\n",
            self.display_name,
            self.substrate_id,
            self.format_summary(),
            self.byte_len(),
            self.source_path.as_deref().unwrap_or("none"),
            self.recipe
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SubstrateRasterColorModel {
    L8,
}

impl SubstrateRasterColorModel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::L8 => "L8",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SubstrateRasterRecipe {
    MockMalwareByteplot,
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

fn procedural_asset_version_status(
    declarations: &[ProceduralAssetDeclaration],
    asset_node: &ProceduralAssetInstanceNode,
) -> OperatorVersionStatus {
    declarations
        .iter()
        .find(|declaration| declaration.asset_id == asset_node.asset_id)
        .map_or(OperatorVersionStatus::MissingDeclaration, |declaration| {
            if declaration.version == asset_node.instance_version {
                OperatorVersionStatus::Current
            } else {
                OperatorVersionStatus::NewerAvailable
            }
        })
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProceduralAssetBoundaryDirection {
    Input,
    Output,
}

impl ProceduralAssetBoundaryDirection {
    fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
        }
    }

    fn ports_mut(
        self,
        declaration: &mut ProceduralAssetDeclaration,
    ) -> &mut Vec<HoudiniOperatorPort> {
        match self {
            Self::Input => &mut declaration.inputs,
            Self::Output => &mut declaration.outputs,
        }
    }
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
    #[serde(default)]
    pub external_artifacts: Vec<ProceduralAssetArtifactReference>,
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
pub(crate) struct ProceduralAssetArtifactReference {
    pub role: ProceduralAssetArtifactRole,
    pub locator: String,
    pub source_node_id: Option<String>,
    pub source_node_name: Option<String>,
    pub size_bytes: Option<u64>,
    pub content_hash: Option<String>,
    pub status: ProceduralAssetArtifactStatus,
}

impl ProceduralAssetArtifactReference {
    fn warning(&self) -> Option<String> {
        match self.status {
            ProceduralAssetArtifactStatus::Referenced => Some(format!(
                "{} artifact `{}` remains an external reference.",
                self.role.as_str(),
                self.locator
            )),
            ProceduralAssetArtifactStatus::Missing => Some(format!(
                "{} artifact `{}` is missing and must be restored or rebound.",
                self.role.as_str(),
                self.locator
            )),
            ProceduralAssetArtifactStatus::Bundled => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum ProceduralAssetArtifactRole {
    Dataset,
    ModelWeights,
    PythonEnvironment,
    Recording,
    AnalysisFile,
    Other,
}

impl ProceduralAssetArtifactRole {
    fn as_str(self) -> &'static str {
        match self {
            Self::Dataset => "Dataset",
            Self::ModelWeights => "Model weights",
            Self::PythonEnvironment => "Python environment",
            Self::Recording => "Recording",
            Self::AnalysisFile => "Analysis file",
            Self::Other => "External",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum ProceduralAssetArtifactStatus {
    Referenced,
    Missing,
    Bundled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProceduralAssetArtifactInclusionChoice {
    pub locator: String,
    pub include: bool,
    pub bundled_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProceduralAssetBundlePreview {
    pub asset_id: String,
    pub display_name: String,
    pub version: String,
    pub artifacts: Vec<ProceduralAssetArtifactBundlePreview>,
    pub dependency_requirements: Vec<String>,
    pub expected_included_size_bytes: u64,
    pub unknown_included_size_count: usize,
    pub included_file_count: usize,
    pub remaining_external_reference_count: usize,
    pub missing_artifact_count: usize,
    pub reproducibility_warnings: Vec<String>,
}

impl ProceduralAssetBundlePreview {
    fn new(
        declaration: &ProceduralAssetDeclaration,
        artifacts: Vec<ProceduralAssetArtifactBundlePreview>,
    ) -> Self {
        let expected_included_size_bytes = artifacts
            .iter()
            .filter(|artifact| artifact.inclusion.includes_file())
            .filter_map(|artifact| artifact.size_bytes)
            .sum();
        let unknown_included_size_count = artifacts
            .iter()
            .filter(|artifact| artifact.inclusion.includes_file() && artifact.size_bytes.is_none())
            .count();
        let included_file_count = artifacts
            .iter()
            .filter(|artifact| artifact.inclusion.includes_file())
            .count();
        let remaining_external_reference_count = artifacts
            .iter()
            .filter(|artifact| {
                artifact.inclusion == ProceduralAssetArtifactBundleInclusion::ReferenceOnly
            })
            .count();
        let missing_artifact_count = artifacts
            .iter()
            .filter(|artifact| {
                artifact.inclusion == ProceduralAssetArtifactBundleInclusion::Missing
            })
            .count();
        let reproducibility_warnings = artifacts
            .iter()
            .flat_map(ProceduralAssetArtifactBundlePreview::reproducibility_warnings)
            .collect();
        let dependency_requirements = artifacts
            .iter()
            .map(ProceduralAssetArtifactBundlePreview::dependency_requirement)
            .collect();

        Self {
            asset_id: declaration.asset_id.clone(),
            display_name: declaration.display_name.clone(),
            version: declaration.version.clone(),
            artifacts,
            dependency_requirements,
            expected_included_size_bytes,
            unknown_included_size_count,
            included_file_count,
            remaining_external_reference_count,
            missing_artifact_count,
            reproducibility_warnings,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProceduralAssetArtifactBundlePreview {
    pub role: ProceduralAssetArtifactRole,
    pub original_locator: String,
    pub bundled_path: Option<String>,
    pub source_node_id: Option<String>,
    pub source_node_name: Option<String>,
    pub size_bytes: Option<u64>,
    pub content_hash: Option<String>,
    pub source_status: ProceduralAssetArtifactStatus,
    pub inclusion: ProceduralAssetArtifactBundleInclusion,
}

impl ProceduralAssetArtifactBundlePreview {
    fn from_reference(
        asset_id: &str,
        reference: &ProceduralAssetArtifactReference,
        choice: Option<&ProceduralAssetArtifactInclusionChoice>,
    ) -> Self {
        let include_requested = choice.is_some_and(|choice| choice.include);
        let bundled_path_choice = choice.and_then(|choice| choice.bundled_path.clone());
        let (inclusion, bundled_path) = match reference.status {
            ProceduralAssetArtifactStatus::Referenced if include_requested => (
                ProceduralAssetArtifactBundleInclusion::Include,
                Some(bundled_path_choice.unwrap_or_else(|| {
                    default_bundled_artifact_path(asset_id, &reference.locator)
                })),
            ),
            ProceduralAssetArtifactStatus::Referenced => {
                (ProceduralAssetArtifactBundleInclusion::ReferenceOnly, None)
            }
            ProceduralAssetArtifactStatus::Missing => {
                (ProceduralAssetArtifactBundleInclusion::Missing, None)
            }
            ProceduralAssetArtifactStatus::Bundled => (
                ProceduralAssetArtifactBundleInclusion::AlreadyBundled,
                Some(reference.locator.clone()),
            ),
        };

        Self {
            role: reference.role,
            original_locator: reference.locator.clone(),
            bundled_path,
            source_node_id: reference.source_node_id.clone(),
            source_node_name: reference.source_node_name.clone(),
            size_bytes: reference.size_bytes,
            content_hash: reference.content_hash.clone(),
            source_status: reference.status,
            inclusion,
        }
    }

    fn reproducibility_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        match self.inclusion {
            ProceduralAssetArtifactBundleInclusion::ReferenceOnly => warnings.push(format!(
                "{} artifact `{}` remains an external reference.",
                self.role.as_str(),
                self.original_locator
            )),
            ProceduralAssetArtifactBundleInclusion::Missing => warnings.push(format!(
                "{} artifact `{}` is missing and must be restored or rebound before packaging.",
                self.role.as_str(),
                self.original_locator
            )),
            ProceduralAssetArtifactBundleInclusion::Include => {
                if self.size_bytes.is_none() {
                    warnings.push(format!(
                        "{} artifact `{}` has unknown size.",
                        self.role.as_str(),
                        self.original_locator
                    ));
                }
                if self.content_hash.is_none() {
                    warnings.push(format!(
                        "{} artifact `{}` has no content hash for reproducibility.",
                        self.role.as_str(),
                        self.original_locator
                    ));
                }
            }
            ProceduralAssetArtifactBundleInclusion::AlreadyBundled => {}
        }
        warnings
    }

    fn dependency_requirement(&self) -> String {
        format!(
            "{} artifact `{}`",
            self.role.as_str(),
            self.original_locator
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProceduralAssetArtifactBundleInclusion {
    Include,
    ReferenceOnly,
    Missing,
    AlreadyBundled,
}

impl ProceduralAssetArtifactBundleInclusion {
    fn includes_file(self) -> bool {
        matches!(self, Self::Include | Self::AlreadyBundled)
    }
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CreateAssetDraft {
    pub asset_id: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub help: String,
    pub inputs: Vec<HoudiniOperatorPort>,
    pub outputs: Vec<HoudiniOperatorPort>,
    pub promoted_parameters: Vec<HoudiniParameterDeclaration>,
    pub external_artifacts: Vec<ProceduralAssetArtifactReference>,
    pub graph_snapshot: ProceduralAssetGraphSnapshot,
    pub wrapped_subgraph: ProceduralAssetSubgraphReference,
}

pub(crate) struct ProceduralAssetDefinitionSaveResult {
    pub asset_id: String,
    pub previous_version: String,
    pub new_version: String,
    pub update_available_instance_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GraphContainerAssetDraftError {
    MissingNodeIndex(usize),
    NotGraphContainer,
    MissingContainerMetadata,
    MissingInternalGraph,
    MissingOutputBoundary,
}

impl CreateAssetDraft {
    fn into_declaration(self) -> ProceduralAssetDeclaration {
        let graph_snapshot = self.graph_snapshot;
        let mut wrapped_subgraph = self.wrapped_subgraph;
        wrapped_subgraph.graph_snapshot = Some(graph_snapshot.clone());
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
                    "snapshot": &graph_snapshot,
                }))),
            },
            inputs: self.inputs,
            outputs: self.outputs,
            promoted_parameters: self.promoted_parameters,
            external_artifacts: self.external_artifacts,
            wrapped_subgraph,
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

impl HoudiniOperatorPort {
    fn geometry(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_kind: HoudiniDataKind::GeometryTable,
            required: true,
            help: help.into(),
        }
    }
}

fn normalized_asset_boundary_port(mut port: HoudiniOperatorPort) -> Option<HoudiniOperatorPort> {
    port.name = port.name.trim().to_owned();
    port.help = port.help.trim().to_owned();
    (!port.name.is_empty()).then_some(port)
}

fn default_bundled_artifact_path(asset_id: &str, locator: &str) -> String {
    let asset_slug = sanitize_asset_id_part(asset_id);
    let filename = Path::new(locator)
        .file_name()
        .and_then(|filename| filename.to_str())
        .filter(|filename| !filename.is_empty())
        .unwrap_or("artifact");
    format!("bundles/assets/{asset_slug}/artifacts/{filename}")
}

fn next_asset_definition_version(version: &str) -> String {
    let parts = version.split('.').collect::<Vec<_>>();
    if let [major, minor, patch] = parts.as_slice()
        && let (Ok(major), Ok(minor), Ok(patch)) = (
            major.parse::<u64>(),
            minor.parse::<u64>(),
            patch.parse::<u64>(),
        )
    {
        return format!("{major}.{minor}.{}", patch + 1);
    }

    if version.trim().is_empty() || version == "unknown" {
        "0.1.0".to_owned()
    } else {
        format!("{version}.1")
    }
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
    #[serde(default)]
    pub label: Option<String>,
    pub kind: HoudiniParameterKind,
    pub default_value: HoudiniParameterValue,
    #[serde(default)]
    pub current_value: Option<HoudiniParameterValue>,
    pub range: Option<HoudiniNumericRange>,
    pub allowed_values: Vec<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub binding: Option<HoudiniParameterBinding>,
    pub help: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct HoudiniParameterBinding {
    pub internal_node_id: String,
    pub internal_parameter_name: String,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum GraphEvaluationMode {
    Automatic,
    #[default]
    OnInteractionComplete,
    Manual,
}

impl GraphEvaluationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Automatic => "Automatic",
            Self::OnInteractionComplete => "On interaction complete",
            Self::Manual => "Manual",
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct ProjectCommandHistory {
    pub undo_stack: Vec<ProjectCommand>,
    pub redo_stack: Vec<ProjectCommand>,
}

#[derive(Clone)]
pub(crate) enum ProjectCommand {
    NodeRename {
        node_id: String,
        old_name: String,
        new_name: String,
    },
    NodeDuplicate {
        source_node_id: String,
        source_node_name: String,
        duplicated_node: Box<GraphNode>,
        insert_index: usize,
    },
    SourceNodeCreate {
        source_node: Box<GraphNode>,
        insert_index: usize,
    },
    NodeDelete {
        deleted_node: Box<GraphNode>,
        remove_index: usize,
        data_flow_edges_before: Vec<GraphDataFlowEdge>,
        data_flow_edges_after: Vec<GraphDataFlowEdge>,
    },
    ReferenceInputCreate {
        reference_node: Box<GraphNode>,
        insert_index: usize,
    },
    ReferenceTargetAdd {
        reference_node_id: String,
        reference_node_name: String,
        added_entry: ReferenceTargetEntry,
        target_index: usize,
    },
    DataFlowEdgeAdd {
        edge: GraphDataFlowEdge,
        readable_path: String,
    },
    DataFlowEdgeRemove {
        edge: GraphDataFlowEdge,
        readable_path: String,
    },
    DataFlowEdgeInsertNode {
        readable_path: String,
        inserted_node_name: String,
        removed_edge: GraphDataFlowEdge,
        added_edges: Vec<GraphDataFlowEdge>,
    },
    NodeParameterEdit {
        node_id: String,
        node_name: String,
        parameter_name: String,
        old_value: f32,
        new_value: f32,
    },
    NodeOutputParticipationEdit {
        node_id: String,
        node_name: String,
        old_participates: bool,
        new_participates: bool,
    },
    NodeCommentVisibilityEdit {
        node_id: String,
        node_name: String,
        old_show_comment: bool,
        new_show_comment: bool,
    },
    NodeManualCookEdit {
        node_id: String,
        node_name: String,
        old_manual: bool,
        new_manual: bool,
    },
    NodeLayoutEdit {
        node_id: String,
        node_name: String,
        old_position: GraphPoint,
        new_position: GraphPoint,
        network_box_changes: Vec<NetworkBoxOrganizationCommandSnapshot>,
    },
    ReferenceTargetEnablementEdit {
        reference_node_id: String,
        reference_node_name: String,
        target: ReferenceTargetIdentity,
        target_node_name: String,
        old_enabled: bool,
        new_enabled: bool,
    },
    ReferenceTargetRemove {
        reference_node_id: String,
        reference_node_name: String,
        removed_entry: ReferenceTargetEntry,
        target_index: usize,
    },
    AnnotationMoveEdit {
        annotation_id: String,
        annotation_title: String,
        old_position: GraphPoint,
        new_position: GraphPoint,
        moved_nodes: Vec<NodeLayoutCommandSnapshot>,
    },
    AnnotationCreate {
        annotation: GraphAnnotation,
        insert_index: usize,
    },
    AnnotationDelete {
        annotation: GraphAnnotation,
        remove_index: usize,
    },
    AnnotationResizeEdit {
        annotation_id: String,
        annotation_title: String,
        old_size: GraphPoint,
        new_size: GraphPoint,
    },
    AnnotationBoundsEdit {
        annotation_id: String,
        annotation_title: String,
        old_position: GraphPoint,
        new_position: GraphPoint,
        old_size: GraphPoint,
        new_size: GraphPoint,
    },
    AnnotationTitleEdit {
        annotation_id: String,
        annotation_title: String,
        old_title: String,
        new_title: String,
    },
    AnnotationTextEdit {
        annotation_id: String,
        annotation_title: String,
        old_text: String,
        new_text: String,
    },
    AnnotationCollapsedEdit {
        annotation_id: String,
        annotation_title: String,
        old_collapsed: bool,
        new_collapsed: bool,
    },
    AnnotationsCollapsedEdit {
        collapsed: bool,
        annotations: Vec<AnnotationCollapsedCommandSnapshot>,
    },
    LayerVisibilityEdit {
        layer_index: usize,
        layer_name: String,
        layer_kind: LayerKind,
        old_visible: bool,
        new_visible: bool,
    },
    LayerOrderEdit {
        layer_index: usize,
        layer_name: String,
        layer_kind: LayerKind,
        old_order: i32,
        new_order: i32,
    },
}

impl ProjectCommand {
    fn summary(&self) -> String {
        match self {
            Self::NodeRename { old_name, .. } => format!("Rename {old_name}"),
            Self::NodeDuplicate {
                source_node_id,
                source_node_name,
                ..
            } => {
                let _ = source_node_id;
                format!("Duplicate {source_node_name}")
            }
            Self::NodeDelete { deleted_node, .. } => {
                format!("Delete {}", deleted_node.name)
            }
            Self::SourceNodeCreate { source_node, .. } => {
                format!("Create {}", source_node.name)
            }
            Self::ReferenceInputCreate { reference_node, .. } => {
                format!("Create {}", reference_node.name)
            }
            Self::ReferenceTargetAdd {
                reference_node_name,
                added_entry,
                ..
            } => {
                format!(
                    "Add {reference_node_name} target {}",
                    added_entry.provenance.source_node_name
                )
            }
            Self::DataFlowEdgeAdd { readable_path, .. } => {
                format!("Add connection {readable_path}")
            }
            Self::DataFlowEdgeRemove { readable_path, .. } => {
                format!("Remove connection {readable_path}")
            }
            Self::DataFlowEdgeInsertNode {
                readable_path,
                inserted_node_name,
                ..
            } => {
                format!("Insert {inserted_node_name} on connection {readable_path}")
            }
            Self::NodeParameterEdit {
                node_name,
                parameter_name,
                ..
            } => format!("Edit {node_name} {parameter_name}"),
            Self::NodeOutputParticipationEdit { node_name, .. } => {
                format!("Set {node_name} output participation")
            }
            Self::NodeCommentVisibilityEdit { node_name, .. } => {
                format!("Set {node_name} comment visibility")
            }
            Self::NodeManualCookEdit { node_name, .. } => {
                format!("Set {node_name} manual cook")
            }
            Self::NodeLayoutEdit { node_name, .. } => format!("Move {node_name}"),
            Self::ReferenceTargetEnablementEdit {
                reference_node_name,
                target_node_name,
                ..
            } => {
                format!("Set {reference_node_name} target {target_node_name}")
            }
            Self::ReferenceTargetRemove {
                reference_node_name,
                removed_entry,
                ..
            } => {
                format!(
                    "Remove {reference_node_name} target {}",
                    removed_entry.provenance.source_node_name
                )
            }
            Self::AnnotationMoveEdit {
                annotation_title, ..
            } => {
                format!("Move {annotation_title}")
            }
            Self::AnnotationCreate { annotation, .. } => {
                format!("Create {}", annotation.title)
            }
            Self::AnnotationDelete { annotation, .. } => {
                format!("Delete {}", annotation.title)
            }
            Self::AnnotationResizeEdit {
                annotation_title, ..
            } => {
                format!("Resize {annotation_title}")
            }
            Self::AnnotationBoundsEdit {
                annotation_title, ..
            } => {
                format!("Fit {annotation_title} to contents")
            }
            Self::AnnotationTitleEdit {
                annotation_title, ..
            } => {
                format!("Edit {annotation_title} title")
            }
            Self::AnnotationTextEdit {
                annotation_title, ..
            } => {
                format!("Edit {annotation_title} note")
            }
            Self::AnnotationCollapsedEdit {
                annotation_title, ..
            } => {
                format!("Set {annotation_title} collapsed")
            }
            Self::AnnotationsCollapsedEdit { collapsed, .. } => {
                if *collapsed {
                    "Collapse boxes and notes".to_owned()
                } else {
                    "Expand boxes and notes".to_owned()
                }
            }
            Self::LayerVisibilityEdit {
                layer_name,
                layer_kind,
                ..
            } => {
                let _ = layer_kind;
                format!("Set {layer_name} visibility")
            }
            Self::LayerOrderEdit {
                layer_name,
                layer_kind,
                ..
            } => {
                let _ = layer_kind;
                format!("Set {layer_name} order")
            }
        }
    }

    fn coalesce_with(&mut self, next: &Self) -> bool {
        match (self, next) {
            (
                Self::AnnotationTitleEdit {
                    annotation_id,
                    new_title,
                    ..
                },
                Self::AnnotationTitleEdit {
                    annotation_id: next_annotation_id,
                    new_title: next_new_title,
                    ..
                },
            ) if annotation_id == next_annotation_id => {
                *new_title = next_new_title.clone();
                true
            }
            (
                Self::AnnotationTextEdit {
                    annotation_id,
                    new_text,
                    ..
                },
                Self::AnnotationTextEdit {
                    annotation_id: next_annotation_id,
                    new_text: next_new_text,
                    ..
                },
            ) if annotation_id == next_annotation_id => {
                *new_text = next_new_text.clone();
                true
            }
            _ => false,
        }
    }

    fn rebuilds_default_data_flow_edges_after_apply(&self) -> bool {
        matches!(
            self,
            Self::NodeDuplicate { .. }
                | Self::ReferenceInputCreate { .. }
                | Self::NodeOutputParticipationEdit { .. }
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NodeLayoutCommandSnapshot {
    node_id: String,
    old_position: GraphPoint,
    new_position: GraphPoint,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NetworkBoxOrganizationCommandSnapshot {
    annotation_id: String,
    old_state: NetworkBoxOrganizationSnapshot,
    new_state: NetworkBoxOrganizationSnapshot,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NetworkBoxOrganizationSnapshot {
    annotation_id: String,
    position: GraphPoint,
    size: GraphPoint,
    member_node_ids: Vec<String>,
}

impl NetworkBoxOrganizationSnapshot {
    fn from_annotation(annotation: &GraphAnnotation) -> Self {
        Self {
            annotation_id: annotation.annotation_id.clone(),
            position: annotation.position,
            size: annotation.size,
            member_node_ids: annotation.member_node_ids.clone(),
        }
    }

    fn apply_to_annotation(&self, annotation: &mut GraphAnnotation) {
        annotation.position = self.position;
        annotation.size = self.size;
        annotation.member_node_ids = self.member_node_ids.clone();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AnnotationCollapsedCommandSnapshot {
    annotation_id: String,
    old_collapsed: bool,
    new_collapsed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProjectCommandDirection {
    Undo,
    Redo,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphWorkItem {
    pub work_item_id: String,
    pub node_index: usize,
    pub node_id: String,
    pub node_name: String,
    pub output_name: String,
    pub status: GraphWorkItemStatus,
    pub fingerprint: String,
    pub summary: String,
    pub diagnostic: Option<String>,
    pub progress: f32,
    pub created_at_millis: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GraphWorkItemStatus {
    Waiting,
    Running,
    Cached,
    Canceled,
    Superseded,
    Failed,
    Complete,
}

impl GraphWorkItemStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Waiting => "Waiting",
            Self::Running => "Running",
            Self::Cached => "Cached",
            Self::Canceled => "Canceled",
            Self::Superseded => "Superseded",
            Self::Failed => "Failed",
            Self::Complete => "Complete",
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

fn normalized_asset_display_name(display_name: impl Into<String>, fallback: &str) -> String {
    let display_name = display_name.into();
    let trimmed = display_name.trim();
    if trimmed.is_empty() {
        fallback.to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn port_names(ports: &[HoudiniOperatorPort]) -> Vec<String> {
    ports
        .iter()
        .map(|port| format!("{} ({:?})", port.name, port.data_kind))
        .collect()
}

fn unique_boundary_port_name(candidate: &str, used_names: &mut Vec<String>) -> String {
    let sanitized = sanitize_asset_id_part(candidate);
    let base = if sanitized.is_empty() {
        PRIMARY_GEOMETRY_OUTPUT.to_owned()
    } else {
        sanitized
    };
    if !used_names.iter().any(|name| name == &base) {
        used_names.push(base.clone());
        return base;
    }

    let mut suffix = 2;
    loop {
        let name = format!("{base}_{suffix}");
        if !used_names.iter().any(|used_name| used_name == &name) {
            used_names.push(name.clone());
            return name;
        }
        suffix += 1;
    }
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
    pub edge_id: String,
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
    GraphContainer,
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
            Self::GraphContainer => "Graph Container",
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
            Self::GraphContainer => "Container",
            Self::PythonOperator => "Compute",
            Self::ProceduralAsset => "Asset",
            Self::NativeOperator => "Native",
            Self::Output => "Publish",
        }
    }

    fn duplicate_node_id_prefix(self) -> &'static str {
        match self {
            Self::Source => "source_copy",
            Self::Filter => "filter_copy",
            Self::Style => "style_copy",
            Self::Null => "null_copy",
            Self::ReferenceInput => "reference_input_copy",
            Self::SubstrateProjection => "substrate_projection_copy",
            Self::GraphContainer => "graph_container_copy",
            Self::PythonOperator => "python_operator_copy",
            Self::ProceduralAsset => "asset_copy",
            Self::NativeOperator => "native_operator_copy",
            Self::Output => "output_copy",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GraphNavigationTarget {
    pub graph_id: String,
    pub name: String,
    pub path: String,
    pub role: ProjectGraphRole,
}

impl GraphNavigationTarget {
    fn from_metadata(metadata: &ProjectGraphMetadata) -> Self {
        Self {
            graph_id: metadata.graph_id.clone(),
            name: metadata.name.clone(),
            path: metadata.path.clone(),
            role: metadata.role,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GraphNavigationChange {
    pub previous_graph: GraphNavigationTarget,
    pub selected_graph: GraphNavigationTarget,
    pub changed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GraphParentNavigationChange {
    pub navigation: GraphNavigationChange,
    pub container_node_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GraphNavigationError {
    MissingGraph {
        graph_id: String,
    },
    MissingNodeIndex(usize),
    NodeIsNotGraphContainer {
        node_id: String,
        node_name: String,
    },
    MissingContainerMetadata {
        node_id: String,
    },
    MissingInternalGraph {
        graph_id: String,
    },
    ContainerNotNavigable {
        node_id: String,
        internal_graph_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GraphLocationInfo {
    pub graph_id: String,
    pub graph_path: String,
    pub node_name: String,
    pub node_path: String,
    pub name_collision_count: usize,
}

impl GraphLocationInfo {
    pub fn name_is_unique_in_graph(&self) -> bool {
        self.name_collision_count == 1
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NodeDataFlowInfo {
    pub incoming_edge_count: usize,
    pub outgoing_edge_count: usize,
    pub diagnostics: Vec<GraphDataFlowEdgeDiagnostic>,
}

pub(crate) struct NodeInfo {
    pub kind: NodeKind,
    pub role: &'static str,
    pub graph_location: GraphLocationInfo,
    pub data_flow: NodeDataFlowInfo,
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
    pub coordinate_contract: Option<SubstrateCoordinateContract>,
    pub substrate_raster: Option<SubstrateRaster>,
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
    #[allow(dead_code)]
    pub graph_container: Option<GraphContainerNodeInfo>,
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
    pub target_node_path: String,
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

#[allow(dead_code)]
pub(crate) struct GraphContainerNodeInfo {
    pub kind: GraphContainerKind,
    pub internal_graph_id: String,
    pub internal_graph_name: Option<String>,
    pub internal_graph_path: Option<String>,
    pub inputs: Vec<HoudiniOperatorPort>,
    pub outputs: Vec<HoudiniOperatorPort>,
    pub mappings: Vec<GraphBoundaryMappingInfo>,
    pub collapse_manifest: Option<GraphContainerCollapseManifest>,
    pub navigable: bool,
    pub status: GraphContainerStatus,
}

#[allow(dead_code)]
pub(crate) struct GraphBoundaryMappingInfo {
    pub direction: GraphBoundaryMappingDirection,
    pub public_port_name: String,
    pub internal_node_id: String,
    pub internal_port_name: String,
    pub status: GraphBoundaryMappingStatus,
}

impl GraphBoundaryMappingInfo {
    fn diagnostic(&self) -> String {
        format!(
            "Boundary {} mapping `{}` -> `{}`:`{}` is {}.",
            self.direction.as_str(),
            self.public_port_name,
            self.internal_node_id,
            self.internal_port_name,
            self.status.as_str()
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GraphBoundaryMappingStatus {
    Resolved,
    MissingInternalGraph,
    MissingPublicPort,
    MissingInternalAnchor,
}

impl GraphBoundaryMappingStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::MissingInternalGraph => "missing internal graph",
            Self::MissingPublicPort => "missing public port",
            Self::MissingInternalAnchor => "missing internal anchor",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GraphContainerStatus {
    Resolved,
    MissingContainerMetadata,
    MissingInternalGraph,
}

impl GraphContainerStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "Resolved",
            Self::MissingContainerMetadata => "Missing container metadata",
            Self::MissingInternalGraph => "Missing internal graph",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GraphContainerCollapseError {
    EmptySelection,
    MissingNodeIndex(usize),
    DisconnectedSelection,
    UntypedExternalEdge(String),
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
    #[allow(dead_code)]
    pub can_save_definition: bool,
    #[allow(dead_code)]
    pub can_match_definition: bool,
    #[allow(dead_code)]
    pub can_upgrade_to_current_definition: bool,
    pub local_graph_id: Option<String>,
    pub description: String,
    pub labels: Vec<String>,
    pub promoted_parameters: Vec<String>,
    #[allow(dead_code)]
    pub external_artifact_warnings: Vec<String>,
    pub input_bindings: Vec<HoudiniNodeBinding>,
    pub output_summary: Option<String>,
    pub version_status: OperatorVersionStatus,
}

pub(crate) struct ProceduralAssetGalleryEntry {
    pub asset_id: String,
    pub display_name: String,
    pub version: Option<String>,
    pub description: String,
    pub labels: Vec<String>,
    pub input_count: usize,
    pub output_count: usize,
    pub promoted_parameter_count: usize,
    pub wrapped_graph_id: Option<String>,
    pub missing_declaration: bool,
    pub usages: Vec<ProceduralAssetUsageInfo>,
}

pub(crate) struct ProceduralAssetUsageInfo {
    pub node_index: usize,
    pub node_id: String,
    pub node_name: String,
    pub graph_id: String,
    pub graph_path: String,
    pub node_path: String,
    pub instance_version: String,
    pub contents_unlocked: bool,
    pub can_match_definition: bool,
    pub can_upgrade_to_current_definition: bool,
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
    pub substrate_raster: Option<SubstrateRaster>,
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
             Substrate rasters: `{}`\n\n\
             Polygons: `{}`\n\n\
             Native cubic Beziers: `{}`\n\n\
             Limitation: {}\n\n\
             | index | kind | layer | score | style |\n\
             | --- | --- | --- | ---: | --- |\n",
            graph.source.metadata.provenance.as_str(),
            self.items.len(),
            usize::from(self.substrate_raster.is_some()),
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
    pub substrate_raster_count: usize,
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
        GeneratedNodeBindingState, GeneratedNodeInfo, GeneratedNodeSource, Geometry,
        GeometryBounds, GeometryKind, GraphAnnotationKind, GraphBoundaryDeclaration,
        GraphBoundaryMapping, GraphBoundaryMappingDirection, GraphBoundaryMappingStatus,
        GraphColor, GraphContainerAssetDraftError, GraphContainerCollapseError, GraphContainerKind,
        GraphContainerMetadata, GraphContainerStatus, GraphDataFlowEdge,
        GraphDataFlowEdgeDiagnosticStatus, GraphDocument, GraphEvaluationMode,
        GraphNavigationError, GraphNavigationTarget, GraphNode, GraphPoint, GraphStyle,
        GraphWorkItemStatus, HoudiniCubicBezierParquetSchema, HoudiniDataKind, HoudiniGeometryKind,
        HoudiniGeometryRecord, HoudiniGeometrySchema, HoudiniNumericRange, HoudiniOperatorPort,
        HoudiniParameterBinding, HoudiniParameterDeclaration, HoudiniParameterKind,
        HoudiniParameterValue, LayerKind, NativeOperatorCapability, NativeOperatorDeclaration,
        NativeOperatorFailureMode, NativeOperatorImplementation, NativeOperatorLoadStatus,
        NativeOperatorOutputCounts, NativeOperatorProvenance, NetworkBadgeVisibility,
        NetworkCommentDisplayMode, NetworkNodeRingVisibility, NodeEvaluation, NodeKind,
        NodeParameter, NodeParameterKind, NodeStatus, OperatorVersionStatus,
        OutputCapabilityMapping, OutputOperatorKind, OutputOperatorNode, OutputTargetId,
        PRIMARY_GEOMETRY_OUTPUT, ProceduralAssetArtifactBundleInclusion,
        ProceduralAssetArtifactInclusionChoice, ProceduralAssetArtifactReference,
        ProceduralAssetArtifactRole, ProceduralAssetArtifactStatus,
        ProceduralAssetBoundaryDirection, ProceduralAssetDeclaration, ProceduralAssetGraphSnapshot,
        ProceduralAssetSource, ProceduralAssetSubgraphReference, ProjectCommand,
        ProjectGraphMetadata, ProjectGraphRegistry, ProjectGraphRole, PythonDependencyHealth,
        PythonEnvironmentDescriptor, PythonEnvironmentPathMode, PythonEnvironmentPaths,
        PythonEnvironmentResolveState, PythonEnvironmentResolveTrigger, PythonEnvironmentResolver,
        PythonEnvironmentStatus, PythonOperatorCapability, PythonOperatorDataKind,
        PythonOperatorDeclaration, PythonOperatorDependencies, PythonOperatorDependencyStatus,
        PythonOperatorEntryPoint, PythonOperatorNumericRange, PythonOperatorOutputCounts,
        PythonOperatorParameterDeclaration, PythonOperatorParameterKind,
        PythonOperatorParameterValue, PythonOperatorPort, PythonOperatorSource,
        PythonProjectRequirements, PythonRequirementSource, PythonRequirementsSource,
        ReferenceDiagnosticStatus, ReferenceTargetEntry, ReferenceTargetIdentity,
        ReferenceTargetProvenance, RerunSceneDebugItem, RerunSceneItem, SourceBundleInclusion,
        SourceExternalReferenceActionKind, SourceExternalReferenceStatus,
        SourceFormatInferenceStatus, SourceFormatKind, SourceFormatSupportStatus,
        SourceGalleryDecodedThumbnail, SourceGalleryIndex, SourceGalleryItemKind,
        SourceGalleryManifestError, SourceGalleryOpenActionKind, SourceGalleryThumbnailCache,
        SourceGalleryThumbnailCacheState, SourceGalleryThumbnailIntent,
        SourceGalleryThumbnailStatus, SourceLocator, SourceLocatorKind,
        SourcePackageManifestArtifactRole, SourcePackageManifestExternalStatus,
        SourcePackageManifestInclusionChoice, SourcePackageManifestPreview, SourceProvenance,
        SubstrateCoordinateContract, SubstrateOrigin, SubstrateYAxis, ViewerGeometry,
        load_cubic_bezier_parquet, load_cubic_bezier_parquet_with_metadata,
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
        node.parent_graph_id = graph.current_graph_id().to_owned();
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
    fn malware_starter_graph_models_byteplot_polygon_workflow() {
        let graph = GraphDocument::malware_starter();

        assert_eq!(graph.source.mode, super::GraphSourceMode::SyntheticMalware);
        assert_eq!(
            graph.source.metadata.provenance,
            SourceProvenance::SyntheticMalware
        );
        assert_eq!(graph.polygon_count(), 4);
        assert_eq!(graph.cubic_bezier_count(), 0);
        assert_eq!(graph.visible_output_count(), 3);
        let raster = graph
            .substrate_raster
            .as_ref()
            .expect("malware starter should carry a raster substrate");
        assert_eq!(raster.substrate_id, "malware-byteplot-pixel-space");
        assert_eq!(raster.format_summary(), "L8 256x256");
        assert_eq!(raster.byte_len(), 256 * 256);
        assert_eq!(raster.luma8_pixels().len(), raster.byte_len());
        assert!(
            graph
                .source
                .metadata
                .attribute_names
                .contains(&"hull_kind".to_owned())
        );
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| node.name == "Byteplot Substrate" && node.kind == NodeKind::Source)
        );
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| node.name == "Convex Hull Regions" && node.kind == NodeKind::Source)
        );
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| node.name == "Concave Hull Regions" && node.kind == NodeKind::Source)
        );

        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.name == "Rerun Malware Output")
            .expect("malware starter should include Rerun output node");
        let output_info = graph
            .selected_node_info(output_index)
            .expect("output node info should exist");
        assert_eq!(output_info.output_count, 3);
        assert_eq!(
            output_info
                .substrate_raster
                .as_ref()
                .map(|raster| raster.display_name.as_str()),
            Some("Mock malware byteplot")
        );
        assert_eq!(
            output_info
                .coordinate_contract
                .as_ref()
                .map(|contract| (contract.width, contract.height)),
            Some((256, 256))
        );
        assert_eq!(
            output_info
                .output_operator
                .as_ref()
                .and_then(|operator| operator.preferred_target),
            Some(OutputTargetId::Rerun)
        );

        let contract = graph.nodes[output_index]
            .coordinate_contract
            .as_ref()
            .expect("output should carry substrate pixel-space contract");
        assert_eq!(contract.substrate_id, "malware-byteplot-pixel-space");
        assert_eq!(contract.width, 256);
        assert_eq!(contract.height, 256);
        assert_eq!(contract.origin, SubstrateOrigin::TopLeft);
        assert_eq!(contract.y_axis, SubstrateYAxis::Down);

        let scene = graph.rerun_scene_output();
        assert_eq!(scene.substrate_raster, graph.substrate_raster);
        assert_eq!(scene.polygon_count(), 3);
    }

    #[test]
    fn malware_starter_node_info_warns_on_substrate_bounds_mismatch() {
        let mut graph = GraphDocument::malware_starter();
        graph.source.metadata.bounds = Some(GeometryBounds {
            min: GraphPoint::new(-4.0, 18.0),
            max: GraphPoint::new(280.0, 220.0),
        });

        let source_info = graph
            .selected_node_info(0)
            .expect("malware starter should include source node info");

        assert_eq!(source_info.status, NodeStatus::Healthy);
        assert!(
            source_info
                .warnings
                .iter()
                .any(|warning| warning.contains("exceed substrate malware-byteplot-pixel-space"))
        );
    }

    #[test]
    fn malware_starter_graph_round_trips_through_sidecar() {
        let graph = GraphDocument::malware_starter();

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(
            restored.source.mode,
            super::GraphSourceMode::SyntheticMalware
        );
        assert_eq!(
            restored.source.metadata.provenance,
            SourceProvenance::SyntheticMalware
        );
        assert_eq!(restored.nodes.len(), graph.nodes.len());
        assert_eq!(restored.visible_output_count(), 3);
        assert_eq!(restored.substrate_raster, graph.substrate_raster);
        assert!(
            restored
                .annotations
                .iter()
                .any(|annotation| annotation.title == "Independent Sources")
        );
        assert!(
            restored
                .nodes
                .iter()
                .any(|node| node.name == "Rerun Malware Output" && node.output_operator.is_some())
        );
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
    fn reference_input_creation_records_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        let null_index = graph.add_null_operator_node("OUT_CURVES");
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .expect("sample graph should include output node");
        let before_len = graph.nodes.len();

        let reference_index = graph
            .add_reference_input_node(null_index)
            .expect("null output should be a compatible reference target");
        let reference_node_id = graph.nodes[reference_index].node_id.clone();
        let reference_target = graph.nodes[reference_index]
            .reference_input
            .as_ref()
            .expect("reference node should carry reference input")
            .targets[0]
            .clone();

        assert_eq!(reference_index, output_index);
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::ReferenceInputCreate {
                reference_node,
                insert_index,
            }) if reference_node.node_id == reference_node_id
                && *insert_index == output_index
                && reference_node
                    .reference_input
                    .as_ref()
                    .is_some_and(|reference_input| reference_input.targets == vec![reference_target.clone()])
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Create Reference Input")
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.nodes.len(), before_len);
        assert!(
            !graph
                .nodes
                .iter()
                .any(|node| node.node_id == reference_node_id)
        );
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Create Reference Input")
        );

        assert!(graph.redo_project_command());
        assert_eq!(graph.nodes.len(), before_len + 1);
        assert_eq!(graph.nodes[output_index].node_id, reference_node_id);
        assert_eq!(
            graph.nodes[output_index]
                .reference_input
                .as_ref()
                .expect("reference node should be restored")
                .targets,
            vec![reference_target]
        );
        assert!(graph.add_reference_input_node(graph.nodes.len()).is_none());
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
    fn node_rename_records_undoable_project_command_and_keeps_references_resolved() {
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
        let node_id = graph.nodes[null_index].node_id.clone();

        assert!(graph.set_node_name(null_index, "OUT_FILTERED"));

        assert_eq!(graph.nodes[null_index].name, "OUT_FILTERED");
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::NodeRename {
                node_id: recorded_node_id,
                old_name,
                new_name,
            }) if recorded_node_id == &node_id
                && old_name == "OUT_ORIGINAL"
                && new_name == "OUT_FILTERED"
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Rename OUT_ORIGINAL")
        );
        assert_eq!(
            graph.resolve_reference_target(&target).readable_path,
            "main/OUT_FILTERED:geometry"
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.nodes[null_index].name, "OUT_ORIGINAL");
        assert_eq!(
            graph.resolve_reference_target(&target).readable_path,
            "main/OUT_ORIGINAL:geometry"
        );
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Rename OUT_ORIGINAL")
        );

        assert!(graph.redo_project_command());
        assert_eq!(graph.nodes[null_index].name, "OUT_FILTERED");
        assert_eq!(
            graph.resolve_reference_target(&target).readable_path,
            "main/OUT_FILTERED:geometry"
        );
        assert!(!graph.set_node_name(null_index, "OUT_FILTERED"));
        assert!(!graph.set_node_name(null_index, ""));
        assert!(!graph.set_node_name(graph.nodes.len(), "MISSING"));
    }

    #[test]
    fn duplicate_node_creates_new_identity_without_publication_state() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Filter)
            .expect("sample graph should include filter node");
        graph.set_node_parameter_value(filter_index, 0.82);
        graph.nodes[filter_index].comment = "Review high-confidence regions.".to_owned();
        graph.nodes[filter_index].show_comment_in_network = true;
        graph.nodes[filter_index].generated = Some(GeneratedNodeInfo::managed(
            GeneratedNodeSource::AttributeTableCommit,
        ));
        graph.nodes[filter_index].output_operator = Some(OutputOperatorNode::generic_scene());
        graph.nodes[filter_index].evaluation = NodeEvaluation {
            state: EvaluationState::Running,
            manual: true,
            message: Some("Running expensive filter".to_owned()),
        };
        graph.nodes[filter_index].participates_in_output = true;

        let original_id = graph.nodes[filter_index].node_id.clone();
        let original_position = graph.nodes[filter_index].layout_position;

        let duplicate_index = graph
            .duplicate_node(filter_index)
            .expect("selected filter should duplicate");
        let duplicate = &graph.nodes[duplicate_index];

        assert_eq!(duplicate.kind, NodeKind::Filter);
        assert_eq!(duplicate.name, "Filter_2");
        assert_ne!(duplicate.node_id, original_id);
        assert!(duplicate.node_id.starts_with("filter_copy."));
        assert_eq!(duplicate.parameter.value, 0.82);
        assert_eq!(duplicate.comment, "Review high-confidence regions.");
        assert!(duplicate.show_comment_in_network);
        assert_eq!(
            duplicate.layout_position,
            GraphPoint::new(original_position.x + 0.12, original_position.y + 0.08)
        );
        assert!(duplicate.generated.is_none());
        assert!(duplicate.output_operator.is_none());
        assert_eq!(duplicate.evaluation, NodeEvaluation::clean());
        assert!(!duplicate.participates_in_output);
        assert_eq!(graph.nodes[filter_index].node_id, original_id);
        assert!(graph.duplicate_node(graph.nodes.len()).is_none());
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
    fn node_delete_records_undoable_project_command_and_preserves_reference_identity() {
        let mut graph = GraphDocument::sample();
        let null_index = graph.add_null_operator_node("OUT_DELETE");
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
        let deleted_node_id = graph.nodes[null_index].node_id.clone();
        let original_node_count = graph.nodes.len();

        let deleted_node = graph
            .remove_node(null_index)
            .expect("ordinary null node should be removable");

        assert_eq!(deleted_node.node_id, deleted_node_id);
        assert_eq!(graph.nodes.len(), original_node_count - 1);
        assert_eq!(
            graph.resolve_reference_target(&target).status,
            ReferenceDiagnosticStatus::MissingNode
        );
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::NodeDelete {
                deleted_node,
                remove_index,
                ..
            }) if deleted_node.node_id == deleted_node_id && *remove_index == null_index
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Delete OUT_DELETE")
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.nodes.len(), original_node_count);
        assert_eq!(graph.nodes[null_index].node_id, deleted_node_id);
        assert_eq!(
            graph.resolve_reference_target(&target).status,
            ReferenceDiagnosticStatus::Resolved
        );
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Delete OUT_DELETE")
        );

        assert!(graph.redo_project_command());
        assert_eq!(graph.nodes.len(), original_node_count - 1);
        assert_eq!(
            graph.resolve_reference_target(&target).status,
            ReferenceDiagnosticStatus::MissingNode
        );
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .expect("sample graph should include output node");
        assert!(graph.remove_node(output_index).is_none());
        assert!(graph.remove_node(graph.nodes.len()).is_none());
    }

    #[test]
    fn node_delete_preserves_unrelated_explicit_data_flow_edges() {
        let mut graph = GraphDocument::sample();
        let source_node_id = graph.nodes[0].node_id.clone();
        let filter_node_id = graph.nodes[1].node_id.clone();
        let style_node_id = graph.nodes[2].node_id.clone();
        let output_node_id = graph.nodes[3].node_id.clone();
        let source_to_filter_edge_id = GraphDocument::data_flow_edge_id(
            &source_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
            &filter_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
        );
        let filter_to_style_edge_id = GraphDocument::data_flow_edge_id(
            &filter_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
            &style_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
        );
        let explicit_edge_id = graph
            .add_data_flow_edge(
                &source_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
                &output_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
            )
            .expect("source to output should be a valid explicit edge");
        let edge_count_before_delete = graph.data_flow_edges.len();

        graph
            .remove_node(1)
            .expect("ordinary filter node should be removable");

        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == explicit_edge_id)
        );
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == source_to_filter_edge_id)
        );
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == filter_to_style_edge_id)
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.data_flow_edges.len(), edge_count_before_delete);
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == explicit_edge_id)
        );
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == source_to_filter_edge_id)
        );
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == filter_to_style_edge_id)
        );

        assert!(graph.redo_project_command());
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == explicit_edge_id)
        );
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == source_to_filter_edge_id)
        );
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == filter_to_style_edge_id)
        );
    }

    #[test]
    fn reconnect_node_delete_adds_valid_bypass_edge() {
        let mut graph = GraphDocument::sample();
        let source_node_id = graph.nodes[0].node_id.clone();
        let filter_node_id = graph.nodes[1].node_id.clone();
        let style_node_id = graph.nodes[2].node_id.clone();
        let source_to_filter_edge_id = GraphDocument::data_flow_edge_id(
            &source_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
            &filter_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
        );
        let filter_to_style_edge_id = GraphDocument::data_flow_edge_id(
            &filter_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
            &style_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
        );
        let bypass_edge_id = GraphDocument::data_flow_edge_id(
            &source_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
            &style_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
        );

        let result = graph
            .remove_node_reconnecting_data_flow(1)
            .expect("ordinary filter node should be removable with reconnect");

        assert_eq!(result.deleted_node.node_id, filter_node_id);
        assert_eq!(result.added_edges.len(), 1);
        assert_eq!(result.added_edges[0].edge_id, bypass_edge_id);
        assert!(result.skipped_diagnostics.is_empty());
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == bypass_edge_id)
        );
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == source_to_filter_edge_id)
        );
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == filter_to_style_edge_id)
        );
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::NodeDelete {
                deleted_node,
                remove_index,
                ..
            }) if deleted_node.node_id == filter_node_id && *remove_index == 1
        ));

        assert!(graph.undo_project_command());
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == source_to_filter_edge_id)
        );
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == filter_to_style_edge_id)
        );
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == bypass_edge_id)
        );

        assert!(graph.redo_project_command());
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == bypass_edge_id)
        );
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == source_to_filter_edge_id)
        );
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == filter_to_style_edge_id)
        );
    }

    #[test]
    fn reconnect_node_delete_reports_skipped_cycle_candidate() {
        let mut graph = GraphDocument::sample();
        let source_node_id = graph.nodes[0].node_id.clone();
        let filter_node_id = graph.nodes[1].node_id.clone();
        let style_node_id = graph.nodes[2].node_id.clone();
        let bypass_edge_id = GraphDocument::data_flow_edge_id(
            &source_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
            &style_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
        );
        let explicit_cycle_edge_id = GraphDocument::data_flow_edge_id(
            &style_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
            &source_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
        );
        graph.data_flow_edges.push(GraphDataFlowEdge {
            edge_id: explicit_cycle_edge_id.clone(),
            from_node_id: style_node_id.clone(),
            from_output: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
            to_node_id: source_node_id.clone(),
            to_input: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
        });

        let result = graph
            .remove_node_reconnecting_data_flow(1)
            .expect("ordinary filter node should be removable with reconnect");

        assert_eq!(result.deleted_node.node_id, filter_node_id);
        assert!(result.added_edges.is_empty());
        assert_eq!(result.skipped_diagnostics.len(), 1);
        assert_eq!(result.skipped_diagnostics[0].edge_id, bypass_edge_id);
        assert_eq!(
            result.skipped_diagnostics[0].status,
            GraphDataFlowEdgeDiagnosticStatus::Cycle
        );
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == bypass_edge_id)
        );
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == explicit_cycle_edge_id)
        );

        assert!(graph.undo_project_command());
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == explicit_cycle_edge_id)
        );
        assert!(graph.redo_project_command());
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == bypass_edge_id)
        );
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == explicit_cycle_edge_id)
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
        assert_eq!(warning.target_node_path, "/obj/main/OUT_A");
        assert_eq!(warning.affected_references.len(), 1);
        assert_eq!(
            warning.affected_references[0].reference_node_index,
            reference_index
        );
    }

    #[test]
    fn reference_output_change_warning_uses_readable_graph_path() {
        let mut graph = GraphDocument::sample();
        graph.graph_registry.graphs.push(ProjectGraphMetadata {
            graph_id: "analysis".to_owned(),
            name: "Analysis".to_owned(),
            path: "/obj/analysis".to_owned(),
            role: ProjectGraphRole::Subgraph,
        });
        graph
            .select_graph_by_id("analysis")
            .expect("analysis graph should be selectable");
        let analysis_output_index = graph.add_null_operator_node("OUT_A");
        graph
            .select_graph_by_id("main")
            .expect("main graph should be selectable");
        let reference_index = graph
            .add_reference_input_node(analysis_output_index)
            .expect("analysis output should be referenceable from main graph");

        let warning = graph
            .reference_output_change_warning_for_node(analysis_output_index)
            .expect("cross-graph referenced output should warn before output changes");

        assert_eq!(warning.target_node_name, "OUT_A");
        assert_eq!(warning.target_node_path, "/obj/analysis/OUT_A");
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
    fn reference_target_addition_records_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        let first_null_index = graph.add_null_operator_node("OUT_A");
        let second_null_index = graph.add_null_operator_node("OUT_B");
        let reference_index = graph
            .add_reference_input_node(first_null_index)
            .expect("first null output should be referenceable");

        assert!(graph.add_reference_target_to_node(reference_index, second_null_index));
        let added_entry = graph.nodes[reference_index]
            .reference_input
            .as_ref()
            .expect("reference input should exist")
            .targets[1]
            .clone();

        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::ReferenceTargetAdd {
                reference_node_name,
                added_entry: recorded_entry,
                target_index: 1,
                ..
            }) if reference_node_name == "Reference Input" && recorded_entry == &added_entry
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Add Reference Input target OUT_B")
        );

        assert!(graph.undo_project_command());
        assert_eq!(
            graph.nodes[reference_index]
                .reference_input
                .as_ref()
                .expect("reference input should exist")
                .targets
                .len(),
            1
        );
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Add Reference Input target OUT_B")
        );

        assert!(graph.redo_project_command());
        let restored_targets = &graph.nodes[reference_index]
            .reference_input
            .as_ref()
            .expect("reference input should exist")
            .targets;
        assert_eq!(restored_targets.len(), 2);
        assert_eq!(restored_targets[1], added_entry);
        assert!(!graph.add_reference_target_to_node(reference_index, second_null_index));
        assert!(!graph.add_reference_target_to_node(graph.nodes.len(), second_null_index));
        assert!(!graph.add_reference_target_to_node(reference_index, graph.nodes.len()));
    }

    #[test]
    fn reference_target_enablement_records_undoable_project_command() {
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

        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::ReferenceTargetEnablementEdit {
                reference_node_name,
                target_node_name,
                old_enabled: true,
                new_enabled: false,
                ..
            }) if reference_node_name == "Reference Input" && target_node_name == "OUT_B"
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Set Reference Input target OUT_B")
        );
        assert!(
            !graph.nodes[reference_index]
                .reference_input
                .as_ref()
                .expect("reference input should exist")
                .targets
                .iter()
                .find(|entry| entry.target.node_id == second_target_node_id)
                .expect("target should exist")
                .enabled
        );

        assert!(graph.undo_project_command());
        assert!(
            graph.nodes[reference_index]
                .reference_input
                .as_ref()
                .expect("reference input should exist")
                .targets
                .iter()
                .find(|entry| entry.target.node_id == second_target_node_id)
                .expect("target should exist")
                .enabled
        );
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Set Reference Input target OUT_B")
        );

        assert!(graph.redo_project_command());
        assert!(
            !graph.nodes[reference_index]
                .reference_input
                .as_ref()
                .expect("reference input should exist")
                .targets
                .iter()
                .find(|entry| entry.target.node_id == second_target_node_id)
                .expect("target should exist")
                .enabled
        );
        assert!(!graph.set_reference_target_enabled(
            reference_index,
            &second_target_node_id,
            false,
        ));
        assert!(!graph.set_reference_target_enabled(
            graph.nodes.len(),
            &second_target_node_id,
            true,
        ));
    }

    #[test]
    fn reference_target_remove_records_undoable_project_command() {
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
        let removed_entry = graph.nodes[reference_index]
            .reference_input
            .as_ref()
            .expect("reference input should exist")
            .targets
            .iter()
            .find(|entry| entry.target.node_id == second_target_node_id)
            .expect("target should exist")
            .clone();

        assert!(graph.remove_reference_target_from_node(reference_index, &second_target_node_id,));

        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::ReferenceTargetRemove {
                reference_node_name,
                removed_entry: recorded_entry,
                target_index: 1,
                ..
            }) if reference_node_name == "Reference Input" && recorded_entry == &removed_entry
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Remove Reference Input target OUT_B")
        );
        assert_eq!(
            graph.nodes[reference_index]
                .reference_input
                .as_ref()
                .expect("reference input should exist")
                .targets
                .len(),
            1
        );

        assert!(graph.undo_project_command());
        let restored_targets = &graph.nodes[reference_index]
            .reference_input
            .as_ref()
            .expect("reference input should exist")
            .targets;
        assert_eq!(restored_targets.len(), 2);
        assert_eq!(restored_targets[1], removed_entry);
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Remove Reference Input target OUT_B")
        );

        assert!(graph.redo_project_command());
        assert_eq!(
            graph.nodes[reference_index]
                .reference_input
                .as_ref()
                .expect("reference input should exist")
                .targets
                .len(),
            1
        );
        assert!(!graph.remove_reference_target_from_node(reference_index, &second_target_node_id,));
        assert!(
            !graph.remove_reference_target_from_node(graph.nodes.len(), &second_target_node_id,)
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
    fn compatible_parameter_edit_preserves_managed_generated_binding() {
        let mut graph = GraphDocument::sample();
        assert!(
            graph.commit_attribute_table_query_as_filter(&AttributeTableQuery {
                search: String::new(),
                minimum_score: Some(0.8),
                sort: AttributeTableSort::RecordIndex,
                sort_descending: false,
            })
        );
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Filter)
            .expect("sample graph should include filter node");

        assert!(graph.set_node_parameter_value(filter_index, 0.6));

        let filter_info = graph
            .selected_node_info(filter_index)
            .expect("filter node info should exist");
        assert_eq!(
            filter_info
                .generated
                .expect("filter should remain generated")
                .binding_state,
            GeneratedNodeBindingState::Managed
        );
        assert_eq!(
            graph
                .filter_rule()
                .expect("generated filter should still expose typed rule")
                .value
                .as_f32(),
            Some(0.6)
        );
    }

    #[test]
    fn parameter_edits_record_undoable_project_commands() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Filter)
            .expect("sample graph should include filter node");
        let old_value = graph.nodes[filter_index].parameter.value;

        assert!(graph.set_node_parameter_value(filter_index, 0.72));

        assert_eq!(graph.command_history.undo_stack.len(), 1);
        assert!(graph.command_history.redo_stack.is_empty());
        assert_eq!(
            graph.nodes[filter_index].evaluation.state,
            EvaluationState::Stale
        );
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::NodeParameterEdit {
                node_name,
                parameter_name,
                old_value: recorded_old_value,
                new_value,
                ..
            }) if node_name == "Filter"
                && parameter_name == "Minimum score"
                && (*recorded_old_value - old_value).abs() <= f32::EPSILON
                && (*new_value - 0.72).abs() <= f32::EPSILON
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Edit Filter Minimum score")
        );
    }

    #[test]
    fn parameter_project_commands_undo_redo_and_clear_redo_on_new_edit() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Filter)
            .expect("sample graph should include filter node");
        let original_value = graph.nodes[filter_index].parameter.value;

        assert!(graph.set_node_parameter_value(filter_index, 0.72));
        assert!(graph.undo_project_command());
        assert_eq!(graph.nodes[filter_index].parameter.value, original_value);
        assert_eq!(
            graph.nodes[filter_index].evaluation.state,
            EvaluationState::Stale
        );
        assert_eq!(graph.command_history.undo_stack.len(), 0);
        assert_eq!(graph.command_history.redo_stack.len(), 1);
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Edit Filter Minimum score")
        );

        assert!(graph.redo_project_command());
        assert!((graph.nodes[filter_index].parameter.value - 0.72).abs() <= f32::EPSILON);
        assert_eq!(graph.command_history.undo_stack.len(), 1);
        assert!(graph.command_history.redo_stack.is_empty());

        assert!(graph.undo_project_command());
        assert!(graph.set_node_parameter_value(filter_index, 0.42));
        assert!(graph.command_history.redo_stack.is_empty());
        assert!((graph.nodes[filter_index].parameter.value - 0.42).abs() <= f32::EPSILON);
    }

    #[test]
    fn layer_visibility_records_undoable_project_commands() {
        let mut graph = GraphDocument::sample();
        assert!(graph.layers[0].visible);

        assert!(graph.set_layer_visibility(0, false));

        assert!(!graph.layers[0].visible);
        assert_eq!(graph.command_history.undo_stack.len(), 1);
        assert!(graph.command_history.redo_stack.is_empty());
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::LayerVisibilityEdit {
                layer_index: 0,
                layer_name,
                layer_kind: LayerKind::Polygons,
                old_visible: true,
                new_visible: false,
            }) if layer_name == "Polygons"
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Set Polygons visibility")
        );

        assert!(graph.undo_project_command());
        assert!(graph.layers[0].visible);
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Set Polygons visibility")
        );

        assert!(graph.redo_project_command());
        assert!(!graph.layers[0].visible);
        assert!(!graph.set_layer_visibility(0, false));
        assert!(!graph.set_layer_visibility(graph.layers.len(), true));
    }

    #[test]
    fn layer_order_records_undoable_project_commands_and_clears_redo() {
        let mut graph = GraphDocument::sample();
        let original_order = graph.layers[1].order;

        assert!(graph.set_layer_order(1, 42));

        assert_eq!(graph.layers[1].order, 42);
        assert_eq!(graph.command_history.undo_stack.len(), 1);
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::LayerOrderEdit {
                layer_index: 1,
                layer_name,
                layer_kind: LayerKind::Curves,
                old_order,
                new_order: 42,
            }) if layer_name == "Curves" && *old_order == original_order
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Set Curves order")
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.layers[1].order, original_order);
        assert_eq!(graph.command_history.redo_stack.len(), 1);

        assert!(graph.set_layer_order(1, 7));
        assert_eq!(graph.layers[1].order, 7);
        assert!(graph.command_history.redo_stack.is_empty());
        assert!(!graph.set_layer_order(1, 7));
        assert!(!graph.set_layer_order(graph.layers.len(), 9));
    }

    #[test]
    fn node_output_participation_records_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Filter)
            .expect("sample graph should include filter node");
        assert!(graph.nodes[filter_index].participates_in_output);

        assert!(graph.set_node_output_participation(filter_index, false));

        assert!(!graph.nodes[filter_index].participates_in_output);
        assert!(graph.command_history.redo_stack.is_empty());
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::NodeOutputParticipationEdit {
                node_name,
                old_participates: true,
                new_participates: false,
                ..
            }) if node_name == "Filter"
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Set Filter output participation")
        );

        assert!(graph.undo_project_command());
        assert!(graph.nodes[filter_index].participates_in_output);
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Set Filter output participation")
        );

        assert!(graph.redo_project_command());
        assert!(!graph.nodes[filter_index].participates_in_output);
        assert!(!graph.set_node_output_participation(filter_index, false));
        assert!(!graph.set_node_output_participation(graph.nodes.len(), true));
    }

    #[test]
    fn node_comment_visibility_records_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Filter)
            .expect("sample graph should include filter node");
        assert!(!graph.nodes[filter_index].show_comment_in_network);

        assert!(graph.set_node_comment_visibility(filter_index, true));

        assert!(graph.nodes[filter_index].show_comment_in_network);
        assert!(graph.command_history.redo_stack.is_empty());
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::NodeCommentVisibilityEdit {
                node_name,
                old_show_comment: false,
                new_show_comment: true,
                ..
            }) if node_name == "Filter"
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Set Filter comment visibility")
        );

        assert!(graph.undo_project_command());
        assert!(!graph.nodes[filter_index].show_comment_in_network);
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Set Filter comment visibility")
        );

        assert!(graph.redo_project_command());
        assert!(graph.nodes[filter_index].show_comment_in_network);
        assert!(!graph.set_node_comment_visibility(filter_index, true));
        assert!(!graph.set_node_comment_visibility(graph.nodes.len(), false));
    }

    #[test]
    fn node_manual_cook_flag_records_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Filter)
            .expect("sample graph should include filter node");
        assert!(!graph.nodes[filter_index].evaluation.manual);

        assert!(graph.set_node_manual(filter_index, true));

        assert!(graph.nodes[filter_index].evaluation.manual);
        assert_eq!(
            graph.nodes[filter_index].evaluation.state,
            EvaluationState::Manual
        );
        assert!(graph.nodes[filter_index].evaluation.message.is_none());
        assert!(graph.command_history.redo_stack.is_empty());
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::NodeManualCookEdit {
                node_name,
                old_manual: false,
                new_manual: true,
                ..
            }) if node_name == "Filter"
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Set Filter manual cook")
        );

        assert!(graph.undo_project_command());
        assert!(!graph.nodes[filter_index].evaluation.manual);
        assert_eq!(
            graph.nodes[filter_index].evaluation.state,
            EvaluationState::Stale
        );
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Set Filter manual cook")
        );

        assert!(graph.redo_project_command());
        assert!(graph.nodes[filter_index].evaluation.manual);
        assert_eq!(
            graph.nodes[filter_index].evaluation.state,
            EvaluationState::Manual
        );
        assert!(!graph.set_node_manual(filter_index, true));
        assert!(!graph.set_node_manual(graph.nodes.len(), false));
    }

    #[test]
    fn node_duplicate_records_undoable_project_command_with_new_stable_id() {
        let mut graph = GraphDocument::sample();
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .expect("sample graph should include output node");
        let source_node_id = graph.nodes[output_index].node_id.clone();
        let source_parameter = graph.nodes[output_index].parameter.clone();
        graph.nodes[output_index].comment = "Preserve ordinary editable note.".to_owned();
        graph.nodes[output_index].show_comment_in_network = true;
        let original_node_count = graph.nodes.len();

        let duplicate_index = graph
            .duplicate_node(output_index)
            .expect("selected output node should duplicate");

        let duplicate = &graph.nodes[duplicate_index];
        let duplicate_node_id = duplicate.node_id.clone();
        let duplicate_name = duplicate.name.clone();
        assert_ne!(duplicate_node_id, source_node_id);
        assert_eq!(duplicate.parameter.name, source_parameter.name);
        assert_eq!(duplicate.parameter.value, source_parameter.value);
        assert_eq!(duplicate.parameter.kind, source_parameter.kind);
        assert_eq!(duplicate.comment, "Preserve ordinary editable note.");
        assert!(duplicate.show_comment_in_network);
        assert!(!duplicate.participates_in_output);
        assert!(duplicate.output_operator.is_none());
        assert_eq!(duplicate.evaluation, NodeEvaluation::clean());
        assert_eq!(graph.command_history.undo_stack.len(), 1);
        assert!(graph.command_history.redo_stack.is_empty());
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::NodeDuplicate {
                source_node_id: recorded_source_id,
                source_node_name,
                duplicated_node,
                insert_index,
            }) if recorded_source_id == &source_node_id
                && source_node_name == "Rerun Output"
                && duplicated_node.node_id == duplicate_node_id
                && *insert_index == duplicate_index
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Duplicate Rerun Output")
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.nodes.len(), original_node_count);
        assert!(
            !graph
                .nodes
                .iter()
                .any(|node| node.node_id == duplicate_node_id)
        );
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Duplicate Rerun Output")
        );

        assert!(graph.redo_project_command());
        let restored_duplicate = graph
            .nodes
            .iter()
            .find(|node| node.node_id == duplicate_node_id)
            .expect("redo should restore the same duplicated node identity");
        assert_eq!(restored_duplicate.name, duplicate_name);
        assert_eq!(restored_duplicate.parameter.name, source_parameter.name);
        assert_eq!(restored_duplicate.parameter.value, source_parameter.value);
        assert_eq!(restored_duplicate.parameter.kind, source_parameter.kind);
        assert_eq!(
            restored_duplicate.comment,
            "Preserve ordinary editable note."
        );
        assert!(restored_duplicate.show_comment_in_network);
        assert!(!restored_duplicate.participates_in_output);
        assert!(restored_duplicate.output_operator.is_none());
        assert!(!graph.duplicate_node(graph.nodes.len()).is_some());
    }

    #[test]
    fn node_layout_drag_records_one_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Filter)
            .expect("sample graph should include filter node");
        let original_position = graph.nodes[filter_index].layout_position;
        let intermediate_position = GraphPoint::new(0.25, 0.75);
        let final_position = GraphPoint::new(-1.0, 2.0);

        graph.set_node_layout_position(filter_index, intermediate_position);
        graph.set_node_layout_position(filter_index, final_position);
        assert!(graph.command_history.undo_stack.is_empty());

        assert!(graph.finish_node_layout_drag(filter_index, original_position));

        assert_eq!(graph.nodes[filter_index].layout_position, final_position);
        assert!(graph.command_history.redo_stack.is_empty());
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::NodeLayoutEdit {
                node_name,
                old_position,
                new_position,
                ..
            }) if node_name == "Filter"
                && *old_position == original_position
                && *new_position == final_position
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Move Filter")
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.nodes[filter_index].layout_position, original_position);
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Move Filter")
        );

        assert!(graph.redo_project_command());
        assert_eq!(graph.nodes[filter_index].layout_position, final_position);
        assert!(!graph.finish_node_layout_drag(filter_index, final_position));
        assert!(!graph.finish_node_layout_drag(graph.nodes.len(), original_position));
    }

    #[test]
    fn node_layout_drag_restores_network_box_expansion() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "filter.main")
            .expect("sample filter node should exist");
        let box_index = graph
            .annotations
            .iter()
            .position(|annotation| annotation.annotation_id == "box.prep")
            .expect("sample network box should exist");
        let old_position = graph.nodes[filter_index].layout_position;
        let old_box_position = graph.annotations[box_index].position;
        let old_box_size = graph.annotations[box_index].size;
        let old_box_members = graph.annotations[box_index].member_node_ids.clone();
        let old_network_box_states = graph.network_box_organization_snapshots();
        let new_position = GraphPoint::new(0.93, 0.90);

        graph.set_node_layout_position(filter_index, new_position);
        assert!(graph.settle_node_drag_for_network_boxes(filter_index, false));
        let expanded_box_position = graph.annotations[box_index].position;
        let expanded_box_size = graph.annotations[box_index].size;

        assert!(graph.finish_node_layout_drag_with_network_box_snapshots(
            filter_index,
            old_position,
            &old_network_box_states,
        ));

        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::NodeLayoutEdit {
                network_box_changes,
                ..
            }) if network_box_changes.len() == 1
                && network_box_changes[0].annotation_id == "box.prep"
        ));

        assert!(graph.undo_project_command());
        assert_eq!(graph.nodes[filter_index].layout_position, old_position);
        assert_eq!(graph.annotations[box_index].position, old_box_position);
        assert_eq!(graph.annotations[box_index].size, old_box_size);
        assert_eq!(
            graph.annotations[box_index].member_node_ids,
            old_box_members
        );

        assert!(graph.redo_project_command());
        assert_eq!(graph.nodes[filter_index].layout_position, new_position);
        assert_eq!(graph.annotations[box_index].position, expanded_box_position);
        assert_eq!(graph.annotations[box_index].size, expanded_box_size);
        assert!(
            graph.annotations[box_index]
                .member_node_ids
                .contains(&"filter.main".to_owned())
        );
    }

    #[test]
    fn node_layout_drag_restores_network_box_fast_drag_membership_removal() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "filter.main")
            .expect("sample filter node should exist");
        let box_index = graph
            .annotations
            .iter()
            .position(|annotation| annotation.annotation_id == "box.prep")
            .expect("sample network box should exist");
        let old_position = graph.nodes[filter_index].layout_position;
        let old_box_members = graph.annotations[box_index].member_node_ids.clone();
        let old_network_box_states = graph.network_box_organization_snapshots();
        let new_position = GraphPoint::new(0.0, 0.0);

        assert!(old_box_members.contains(&"filter.main".to_owned()));
        graph.set_node_layout_position(filter_index, new_position);
        assert!(graph.settle_node_drag_for_network_boxes(filter_index, true));
        assert!(
            !graph.annotations[box_index]
                .member_node_ids
                .contains(&"filter.main".to_owned())
        );

        assert!(graph.finish_node_layout_drag_with_network_box_snapshots(
            filter_index,
            old_position,
            &old_network_box_states,
        ));
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::NodeLayoutEdit {
                network_box_changes,
                ..
            }) if network_box_changes.len() == 1
                && network_box_changes[0].annotation_id == "box.prep"
        ));

        assert!(graph.undo_project_command());
        assert_eq!(graph.nodes[filter_index].layout_position, old_position);
        assert_eq!(
            graph.annotations[box_index].member_node_ids,
            old_box_members
        );

        assert!(graph.redo_project_command());
        assert_eq!(graph.nodes[filter_index].layout_position, new_position);
        assert!(
            !graph.annotations[box_index]
                .member_node_ids
                .contains(&"filter.main".to_owned())
        );
    }

    #[test]
    fn annotation_drag_records_one_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        let box_index = graph
            .annotations
            .iter()
            .position(|annotation| annotation.kind == GraphAnnotationKind::NetworkBox)
            .expect("sample graph should include a network box");
        let annotation_title = graph.annotations[box_index].title.clone();
        let original_position = graph.annotations[box_index].position;
        let original_member_positions = graph.annotation_member_layout_positions(box_index);
        assert!(!original_member_positions.is_empty());

        graph.translate_annotation(box_index, GraphPoint::new(0.12, 0.08));
        graph.translate_annotation(box_index, GraphPoint::new(-0.02, 0.04));
        assert!(graph.command_history.undo_stack.is_empty());
        let final_position = graph.annotations[box_index].position;
        let final_member_positions = original_member_positions
            .iter()
            .filter_map(|(node_id, _)| {
                graph
                    .nodes
                    .iter()
                    .find(|node| node.node_id == *node_id)
                    .map(|node| (node_id.clone(), node.layout_position))
            })
            .collect::<Vec<_>>();

        assert!(graph.finish_annotation_drag(
            box_index,
            original_position,
            &original_member_positions,
        ));

        assert_eq!(graph.annotations[box_index].position, final_position);
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::AnnotationMoveEdit {
                annotation_title: recorded_title,
                old_position,
                new_position,
                moved_nodes,
                ..
            }) if recorded_title == &annotation_title
                && *old_position == original_position
                && *new_position == final_position
                && moved_nodes.len() == original_member_positions.len()
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some(format!("Move {annotation_title}").as_str())
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.annotations[box_index].position, original_position);
        for (node_id, original_node_position) in &original_member_positions {
            let node = graph
                .nodes
                .iter()
                .find(|node| node.node_id == *node_id)
                .expect("moved member node should still exist");
            assert_eq!(node.layout_position, *original_node_position);
        }

        assert!(graph.redo_project_command());
        assert_eq!(graph.annotations[box_index].position, final_position);
        for (node_id, final_node_position) in &final_member_positions {
            let node = graph
                .nodes
                .iter()
                .find(|node| node.node_id == *node_id)
                .expect("moved member node should still exist");
            assert_eq!(node.layout_position, *final_node_position);
        }
        assert!(!graph.finish_annotation_drag(box_index, final_position, &final_member_positions,));
        assert!(!graph.finish_annotation_drag(graph.annotations.len(), original_position, &[],));
    }

    #[test]
    fn organization_creation_records_undoable_project_commands() {
        let mut graph = GraphDocument::sample();
        graph.annotations.clear();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Filter)
            .expect("sample graph should include filter node");

        let box_index = graph
            .add_network_box_for_node(filter_index)
            .expect("network box should be created for selected node");
        let box_id = graph.annotations[box_index].annotation_id.clone();
        assert_eq!(
            graph.annotations[box_index].kind,
            GraphAnnotationKind::NetworkBox
        );
        assert_eq!(
            graph.annotations[box_index].member_node_ids,
            vec![graph.nodes[filter_index].node_id.clone()]
        );
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::AnnotationCreate {
                annotation,
                insert_index: 0,
            }) if annotation.annotation_id == box_id
                && annotation.kind == GraphAnnotationKind::NetworkBox
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Create Network Box")
        );

        assert!(graph.undo_project_command());
        assert!(graph.annotations.is_empty());
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Create Network Box")
        );

        assert!(graph.redo_project_command());
        assert_eq!(graph.annotations.len(), 1);
        assert_eq!(graph.annotations[0].annotation_id, box_id);

        let note_index = graph
            .add_sticky_note_near_node(filter_index)
            .expect("sticky note should be created near selected node");
        let note_id = graph.annotations[note_index].annotation_id.clone();
        assert_eq!(
            graph.annotations[note_index].kind,
            GraphAnnotationKind::StickyNote
        );
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::AnnotationCreate {
                annotation,
                insert_index: 1,
            }) if annotation.annotation_id == note_id
                && annotation.kind == GraphAnnotationKind::StickyNote
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Create Sticky Note")
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.annotations.len(), 1);
        assert!(
            !graph
                .annotations
                .iter()
                .any(|annotation| annotation.annotation_id == note_id)
        );
        assert!(graph.redo_project_command());
        assert_eq!(graph.annotations.len(), 2);
        assert_eq!(graph.annotations[1].annotation_id, note_id);
        assert!(graph.add_network_box_for_node(graph.nodes.len()).is_none());
        assert!(graph.add_sticky_note_near_node(graph.nodes.len()).is_none());
    }

    #[test]
    fn graph_annotations_are_scoped_to_current_graph() {
        let mut graph = GraphDocument::sample();
        graph.graph_registry.graphs.push(ProjectGraphMetadata {
            graph_id: "analysis".to_owned(),
            name: "Analysis".to_owned(),
            path: "/obj/main/analysis".to_owned(),
            role: ProjectGraphRole::Subgraph,
        });
        let main_annotation_count = graph.annotations.len();
        assert_eq!(
            graph.current_graph_annotation_indices().len(),
            main_annotation_count
        );

        graph
            .select_graph_by_id("analysis")
            .expect("analysis graph should be selectable");
        assert!(graph.current_graph_annotation_indices().is_empty());
        let analysis_node_index = graph.add_null_operator_node("OUT_ANALYSIS");
        let analysis_box_index = graph
            .add_network_box_for_node(analysis_node_index)
            .expect("analysis graph should support network boxes");
        let analysis_note_index = graph
            .add_sticky_note_near_node(analysis_node_index)
            .expect("analysis graph should support sticky notes");

        assert_eq!(
            graph.current_graph_annotation_indices(),
            vec![analysis_box_index, analysis_note_index]
        );
        assert_eq!(
            graph.annotations[analysis_box_index].parent_graph_id,
            "analysis"
        );
        assert_eq!(
            graph.annotations[analysis_note_index].parent_graph_id,
            "analysis"
        );
        assert!(
            graph
                .network_box_organization_snapshots()
                .iter()
                .all(|snapshot| {
                    snapshot.annotation_id == graph.annotations[analysis_box_index].annotation_id
                })
        );

        assert!(graph.set_all_annotations_collapsed(true));
        assert!(graph.annotations[analysis_box_index].collapsed);
        assert!(graph.annotations[analysis_note_index].collapsed);
        assert!(
            graph.annotations[..main_annotation_count]
                .iter()
                .all(|annotation| !annotation.collapsed)
        );

        graph
            .select_graph_by_id("main")
            .expect("main graph should be selectable");
        assert_eq!(
            graph.current_graph_annotation_indices(),
            (0..main_annotation_count).collect::<Vec<_>>()
        );
        assert!(
            graph
                .current_graph_annotation_indices()
                .iter()
                .all(|index| { graph.annotations[*index].parent_graph_id == "main" })
        );
    }

    #[test]
    fn annotation_delete_records_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        graph.annotations.clear();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Filter)
            .expect("sample graph should include filter node");

        let box_index = graph
            .add_network_box_for_node(filter_index)
            .expect("network box should be created for selected node");
        let deleted_box = graph.annotations[box_index].clone();
        let member_node_id = graph.nodes[filter_index].node_id.clone();

        assert_eq!(
            graph.remove_annotation(box_index),
            Some(deleted_box.clone())
        );

        assert!(
            graph
                .nodes
                .iter()
                .any(|node| node.node_id == member_node_id)
        );
        assert!(graph.annotations.is_empty());
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::AnnotationDelete {
                annotation,
                remove_index: 0,
            }) if annotation == &deleted_box
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Delete Network Box")
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.annotations.len(), 1);
        assert_eq!(graph.annotations[0], deleted_box);
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Delete Network Box")
        );

        assert!(graph.redo_project_command());
        assert!(graph.annotations.is_empty());

        let note_index = graph
            .add_sticky_note_near_node(filter_index)
            .expect("sticky note should be created near selected node");
        assert!(graph.set_annotation_text(note_index, "Review output bounds.".to_owned()));
        assert!(graph.set_annotation_collapsed(note_index, true));
        let deleted_note = graph.annotations[note_index].clone();

        assert_eq!(
            graph.remove_annotation(note_index),
            Some(deleted_note.clone())
        );
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::AnnotationDelete {
                annotation,
                remove_index: 0,
            }) if annotation == &deleted_note
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Delete Sticky Note")
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.annotations.len(), 1);
        assert_eq!(graph.annotations[0], deleted_note);
        assert!(graph.redo_project_command());
        assert!(graph.annotations.is_empty());
        assert!(graph.remove_annotation(graph.annotations.len()).is_none());
    }

    #[test]
    fn annotation_resize_records_one_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        let box_index = graph
            .annotations
            .iter()
            .position(|annotation| annotation.kind == GraphAnnotationKind::NetworkBox)
            .expect("sample graph should include a network box");
        let annotation_title = graph.annotations[box_index].title.clone();
        let original_size = graph.annotations[box_index].size;
        let final_size = GraphPoint::new(0.42, 0.36);

        assert!(graph.set_annotation_size(box_index, GraphPoint::new(0.32, 0.28)));
        assert!(graph.set_annotation_size(box_index, final_size));
        assert!(graph.command_history.undo_stack.is_empty());

        assert!(graph.finish_annotation_resize(box_index, original_size));

        assert_eq!(graph.annotations[box_index].size, final_size);
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::AnnotationResizeEdit {
                annotation_title: recorded_title,
                old_size,
                new_size,
                ..
            }) if recorded_title == &annotation_title
                && *old_size == original_size
                && *new_size == final_size
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some(format!("Resize {annotation_title}").as_str())
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.annotations[box_index].size, original_size);
        assert!(graph.redo_project_command());
        assert_eq!(graph.annotations[box_index].size, final_size);
        assert!(!graph.finish_annotation_resize(box_index, final_size));
        assert!(!graph.finish_annotation_resize(graph.annotations.len(), original_size));
    }

    #[test]
    fn annotation_title_edits_coalesce_into_one_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        let box_index = graph
            .annotations
            .iter()
            .position(|annotation| annotation.kind == GraphAnnotationKind::NetworkBox)
            .expect("sample graph should include a network box");
        let original_title = graph.annotations[box_index].title.clone();

        assert!(graph.set_annotation_title(box_index, "Region Prep".to_owned()));
        assert!(graph.set_annotation_title(box_index, "Region Prep Notes".to_owned()));

        assert_eq!(graph.annotations[box_index].title, "Region Prep Notes");
        assert_eq!(graph.command_history.undo_stack.len(), 1);
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::AnnotationTitleEdit {
                annotation_title,
                old_title,
                new_title,
                ..
            }) if annotation_title == &original_title
                && old_title == &original_title
                && new_title == "Region Prep Notes"
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some(format!("Edit {original_title} title").as_str())
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.annotations[box_index].title, original_title);
        assert!(graph.redo_project_command());
        assert_eq!(graph.annotations[box_index].title, "Region Prep Notes");
        assert!(!graph.set_annotation_title(box_index, "Region Prep Notes".to_owned()));
        assert!(!graph.set_annotation_title(graph.annotations.len(), "Missing".to_owned()));
    }

    #[test]
    fn sticky_note_text_edits_coalesce_into_one_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        let note_index = graph
            .annotations
            .iter()
            .position(|annotation| annotation.kind == GraphAnnotationKind::StickyNote)
            .expect("sample graph should include a sticky note");
        let original_text = graph.annotations[note_index].text.clone();
        let title = graph.annotations[note_index].title.clone();

        assert!(graph.set_annotation_text(note_index, "Review".to_owned()));
        assert!(graph.set_annotation_text(note_index, "Review before output".to_owned()));

        assert_eq!(graph.annotations[note_index].text, "Review before output");
        assert_eq!(graph.command_history.undo_stack.len(), 1);
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::AnnotationTextEdit {
                annotation_title,
                old_text,
                new_text,
                ..
            }) if annotation_title == &title
                && old_text == &original_text
                && new_text == "Review before output"
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some(format!("Edit {title} note").as_str())
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.annotations[note_index].text, original_text);
        assert!(graph.redo_project_command());
        assert_eq!(graph.annotations[note_index].text, "Review before output");
        assert!(!graph.set_annotation_text(note_index, "Review before output".to_owned()));
        assert!(!graph.set_annotation_text(graph.annotations.len(), "Missing".to_owned()));
        assert!(!graph.set_annotation_text(0, "Network boxes do not use body text".to_owned()));
    }

    #[test]
    fn annotation_collapsed_state_records_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        let box_index = graph
            .annotations
            .iter()
            .position(|annotation| annotation.kind == GraphAnnotationKind::NetworkBox)
            .expect("sample graph should include a network box");
        let title = graph.annotations[box_index].title.clone();
        assert!(!graph.annotations[box_index].collapsed);

        assert!(graph.set_annotation_collapsed(box_index, true));

        assert!(graph.annotations[box_index].collapsed);
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::AnnotationCollapsedEdit {
                annotation_title,
                old_collapsed: false,
                new_collapsed: true,
                ..
            }) if annotation_title == &title
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some(format!("Set {title} collapsed").as_str())
        );

        assert!(graph.undo_project_command());
        assert!(!graph.annotations[box_index].collapsed);
        assert!(graph.redo_project_command());
        assert!(graph.annotations[box_index].collapsed);
        assert!(!graph.set_annotation_collapsed(box_index, true));
        assert!(!graph.set_annotation_collapsed(graph.annotations.len(), false));
    }

    #[test]
    fn all_annotation_collapse_records_one_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        assert!(
            graph
                .annotations
                .iter()
                .any(|annotation| !annotation.collapsed)
        );

        assert!(graph.set_all_annotations_collapsed(true));

        assert!(
            graph
                .annotations
                .iter()
                .all(|annotation| annotation.collapsed)
        );
        assert_eq!(graph.command_history.undo_stack.len(), 1);
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::AnnotationsCollapsedEdit {
                collapsed: true,
                annotations,
            }) if annotations.len() == graph.annotations.len()
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Collapse boxes and notes")
        );

        assert!(graph.undo_project_command());
        assert!(
            graph
                .annotations
                .iter()
                .all(|annotation| !annotation.collapsed)
        );
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Collapse boxes and notes")
        );

        assert!(graph.redo_project_command());
        assert!(
            graph
                .annotations
                .iter()
                .all(|annotation| annotation.collapsed)
        );
        assert!(!graph.set_all_annotations_collapsed(true));
        assert!(graph.set_all_annotations_collapsed(false));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Expand boxes and notes")
        );
    }

    #[test]
    fn command_history_is_runtime_state_not_sidecar_state() {
        let mut graph = GraphDocument::sample();
        let original_position = graph.nodes[1].layout_position;
        let original_annotation_position = graph.annotations[0].position;
        let original_annotation_members = graph.annotation_member_layout_positions(0);
        let original_annotation_size = graph.annotations[0].size;
        assert!(graph.set_node_name(1, "FILTER_LOW"));
        assert!(graph.set_node_parameter_value(1, 0.72));
        assert!(graph.set_layer_visibility(0, false));
        assert!(graph.set_layer_order(1, 42));
        assert!(graph.set_node_output_participation(1, false));
        assert!(graph.set_node_comment_visibility(1, true));
        assert!(graph.set_node_manual(1, true));
        assert!(graph.add_network_box_for_node(1).is_some());
        assert!(graph.add_sticky_note_near_node(1).is_some());
        graph.set_node_layout_position(1, GraphPoint::new(0.25, 0.75));
        assert!(graph.finish_node_layout_drag(1, original_position));
        assert!(graph.translate_annotation(0, GraphPoint::new(0.04, 0.04)));
        assert!(graph.finish_annotation_drag(
            0,
            original_annotation_position,
            &original_annotation_members,
        ));
        assert!(graph.set_annotation_size(0, GraphPoint::new(0.42, 0.36)));
        assert!(graph.finish_annotation_resize(0, original_annotation_size));
        assert!(graph.set_annotation_title(0, "Workflow Notes".to_owned()));
        assert!(graph.set_annotation_text(1, "Check overlay bounds.".to_owned()));
        assert!(graph.set_annotation_collapsed(0, true));
        assert!(graph.set_all_annotations_collapsed(true));
        assert!(graph.remove_annotation(0).is_some());
        assert!(!graph.command_history.undo_stack.is_empty());

        let json = graph.to_sidecar_json().unwrap();
        assert!(!json.contains("command_history"));
        assert!(!json.contains("undo_stack"));

        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert!(restored.command_history.undo_stack.is_empty());
        assert!(restored.command_history.redo_stack.is_empty());
    }

    #[test]
    fn layer_commit_does_not_rewrite_adopted_or_unbound_generated_filter() {
        let mut graph = GraphDocument::sample();
        assert!(
            graph.commit_attribute_table_query_as_filter(&AttributeTableQuery {
                search: String::new(),
                minimum_score: Some(0.8),
                sort: AttributeTableSort::RecordIndex,
                sort_descending: false,
            })
        );
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Filter)
            .expect("sample graph should include filter node");

        assert!(graph.set_generated_node_binding_state(
            filter_index,
            GeneratedNodeBindingState::Adopted,
        ));
        assert!(
            !graph.commit_attribute_table_query_as_filter(&AttributeTableQuery {
                search: String::new(),
                minimum_score: Some(0.3),
                sort: AttributeTableSort::RecordIndex,
                sort_descending: false,
            })
        );
        assert_eq!(
            graph
                .filter_rule()
                .expect("adopted filter should keep its graph-owned rule")
                .value
                .as_f32(),
            Some(0.8)
        );
        assert_eq!(
            graph.nodes[filter_index]
                .generated
                .expect("filter should remain generated")
                .binding_state,
            GeneratedNodeBindingState::Adopted
        );

        assert!(graph.set_generated_node_binding_state(
            filter_index,
            GeneratedNodeBindingState::Unbound,
        ));
        assert!(
            !graph.commit_attribute_table_query_as_filter(&AttributeTableQuery {
                search: String::new(),
                minimum_score: Some(0.2),
                sort: AttributeTableSort::RecordIndex,
                sort_descending: false,
            })
        );
        assert_eq!(
            graph
                .filter_rule()
                .expect("unbound filter should keep its graph-owned rule")
                .value
                .as_f32(),
            Some(0.8)
        );
        assert_eq!(
            graph.nodes[filter_index]
                .generated
                .expect("filter should remain generated")
                .binding_state,
            GeneratedNodeBindingState::Unbound
        );
    }

    #[test]
    fn generated_node_binding_state_is_graph_owned_and_durable() {
        let mut graph = GraphDocument::sample();
        assert!(
            graph.commit_attribute_table_query_as_filter(&AttributeTableQuery {
                search: String::new(),
                minimum_score: Some(0.8),
                sort: AttributeTableSort::RecordIndex,
                sort_descending: false,
            })
        );
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Filter)
            .expect("sample graph should include filter node");

        assert!(graph.set_generated_node_binding_state(
            filter_index,
            GeneratedNodeBindingState::Adopted,
        ));
        assert_eq!(
            graph
                .selected_node_info(filter_index)
                .expect("filter node info should exist")
                .generated
                .expect("filter node info should expose generated metadata")
                .binding_state,
            GeneratedNodeBindingState::Adopted
        );
        assert!(!graph.set_generated_node_binding_state(
            filter_index,
            GeneratedNodeBindingState::Adopted,
        ));
        assert!(!graph.set_generated_node_binding_state(
            graph.nodes.len(),
            GeneratedNodeBindingState::Unbound,
        ));

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(
            restored
                .selected_node_info(filter_index)
                .expect("restored filter node info should exist")
                .generated
                .expect("restored filter node should expose generated metadata")
                .binding_state,
            GeneratedNodeBindingState::Adopted
        );
    }

    #[test]
    fn structural_output_replacement_adopts_managed_generated_node() {
        let mut graph = GraphDocument::sample();
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .expect("sample graph should include output node");
        graph.nodes[output_index].generated = Some(GeneratedNodeInfo::managed(
            GeneratedNodeSource::AttributeTableCommit,
        ));

        assert!(
            graph.set_output_operator_for_node(output_index, OutputOperatorNode::generic_scene(),)
        );

        let output_info = graph
            .selected_node_info(output_index)
            .expect("output node info should exist");
        assert_eq!(
            output_info
                .generated
                .expect("output should retain generated provenance")
                .binding_state,
            GeneratedNodeBindingState::Adopted
        );
        assert_eq!(
            output_info
                .output_operator
                .expect("output operator info should exist")
                .kind,
            OutputOperatorKind::Generic
        );
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
        assert_eq!(graph.data_flow_edges.len(), graph.nodes.len() - 1);
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
    fn graph_layout_scopes_nodes_and_edges_to_selected_graph() {
        let mut graph = GraphDocument::sample();
        graph.graph_registry.graphs.push(ProjectGraphMetadata {
            graph_id: "analysis".to_owned(),
            name: "Analysis".to_owned(),
            path: "/obj/analysis".to_owned(),
            role: ProjectGraphRole::Subgraph,
        });
        graph
            .select_graph_by_id("analysis")
            .expect("analysis graph should be selectable");
        let first_analysis_index = graph.add_null_operator_node("OUT_A");
        let second_analysis_index = graph.add_null_operator_node("OUT_B");

        assert_eq!(
            graph.graph_local_node_indices("analysis"),
            vec![first_analysis_index, second_analysis_index]
        );
        assert_eq!(
            graph.current_graph_node_indices(),
            vec![first_analysis_index, second_analysis_index]
        );
        assert!(graph.data_flow_edges.iter().all(|edge| {
            graph.node_parent_graph_id(
                graph
                    .nodes
                    .iter()
                    .find(|node| node.node_id == edge.from_node_id)
                    .expect("source node should exist"),
            ) == graph.node_parent_graph_id(
                graph
                    .nodes
                    .iter()
                    .find(|node| node.node_id == edge.to_node_id)
                    .expect("target node should exist"),
            )
        }));

        let analysis_layout = graph.graph_layout();
        assert_eq!(analysis_layout.nodes.len(), 2);
        assert_eq!(analysis_layout.nodes[0].node_index, first_analysis_index);
        assert_eq!(analysis_layout.nodes[1].node_index, second_analysis_index);
        assert_eq!(analysis_layout.edges.len(), 1);
        assert_eq!(analysis_layout.edges[0].from_node, first_analysis_index);
        assert_eq!(analysis_layout.edges[0].to_node, second_analysis_index);

        let main_layout = graph.graph_layout_for_graph("main");
        assert_eq!(main_layout.nodes.len(), 4);
        assert_eq!(main_layout.edges.len(), 3);
    }

    #[test]
    fn graph_layout_hides_loaded_cross_graph_edges_but_diagnostics_keep_readable_paths() {
        let mut graph = GraphDocument::sample();
        graph.graph_registry.graphs.push(ProjectGraphMetadata {
            graph_id: "analysis".to_owned(),
            name: "Analysis".to_owned(),
            path: "/obj/analysis".to_owned(),
            role: ProjectGraphRole::Subgraph,
        });
        graph
            .select_graph_by_id("analysis")
            .expect("analysis graph should be selectable");
        let analysis_index = graph.add_null_operator_node("OUT_A");
        graph.data_flow_edges.push(GraphDataFlowEdge {
            edge_id: "cross-graph-broken".to_owned(),
            from_node_id: "source.main".to_owned(),
            from_output: "missing".to_owned(),
            to_node_id: graph.nodes[analysis_index].node_id.clone(),
            to_input: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
        });

        let layout = graph.graph_layout();
        assert_eq!(layout.nodes.len(), 1);
        assert!(
            layout
                .edges
                .iter()
                .all(|edge| edge.from_node != 0 && edge.to_node != 0)
        );

        let diagnostic = graph
            .data_flow_edge_diagnostic(
                graph
                    .data_flow_edges
                    .iter()
                    .find(|edge| edge.edge_id == "cross-graph-broken")
                    .expect("cross-graph edge should remain loaded"),
            )
            .expect("broken loaded edge should remain diagnostic");
        assert_eq!(
            diagnostic.status,
            GraphDataFlowEdgeDiagnosticStatus::MissingSourcePort
        );
        assert_eq!(
            diagnostic.readable_path,
            "/obj/main/Source:missing -> /obj/analysis/OUT_A:geometry"
        );
    }

    #[test]
    fn graph_data_flow_edges_default_to_stable_node_id_endpoints() {
        let graph = GraphDocument::sample();
        let first_edge = graph
            .data_flow_edges
            .first()
            .expect("sample graph should include data-flow edge metadata");

        assert_eq!(first_edge.from_node_id, graph.nodes[0].node_id);
        assert_eq!(first_edge.from_output, "geometry");
        assert_eq!(first_edge.to_node_id, graph.nodes[1].node_id);
        assert_eq!(first_edge.to_input, "geometry");
        assert!(first_edge.edge_id.contains(&graph.nodes[0].node_id));
        assert!(first_edge.edge_id.contains(&graph.nodes[1].node_id));
    }

    #[test]
    fn graph_data_flow_edges_round_trip_through_sidecar() {
        let mut graph = GraphDocument::sample();
        graph.data_flow_edges[0].from_output = "primary_geometry".to_owned();
        graph.data_flow_edges[0].to_input = "source_geometry".to_owned();

        let json = graph.to_sidecar_json().unwrap();
        assert!(json.contains("\"data_flow_edges\""));
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(restored.data_flow_edges, graph.data_flow_edges);
        assert_eq!(
            restored.graph_layout().edges.len(),
            graph.graph_layout().edges.len()
        );
    }

    #[test]
    fn graph_data_flow_edge_validation_accepts_dag_addition() {
        let graph = GraphDocument::sample();
        let edge = GraphDataFlowEdge {
            edge_id: "source_to_output_preview".to_owned(),
            from_node_id: graph.nodes[0].node_id.clone(),
            from_output: "geometry".to_owned(),
            to_node_id: graph.nodes[3].node_id.clone(),
            to_input: "geometry".to_owned(),
        };

        assert!(graph.can_add_data_flow_edge(&edge));
        assert!(graph.data_flow_edge_addition_diagnostic(&edge).is_none());
        assert!(graph.data_flow_edge_diagnostics().is_empty());
    }

    #[test]
    fn graph_data_flow_edge_validation_rejects_cycle_addition() {
        let graph = GraphDocument::sample();
        let edge = GraphDataFlowEdge {
            edge_id: "style_to_source_cycle".to_owned(),
            from_node_id: graph.nodes[2].node_id.clone(),
            from_output: "geometry".to_owned(),
            to_node_id: graph.nodes[0].node_id.clone(),
            to_input: "geometry".to_owned(),
        };
        let diagnostic = graph
            .data_flow_edge_addition_diagnostic(&edge)
            .expect("reverse edge should be cyclic");

        assert!(!graph.can_add_data_flow_edge(&edge));
        assert_eq!(diagnostic.status, GraphDataFlowEdgeDiagnosticStatus::Cycle);
        assert_eq!(diagnostic.edge_id, "style_to_source_cycle");
    }

    #[test]
    fn preview_add_data_flow_edge_reports_diagnostics_without_mutating() {
        let graph = GraphDocument::sample();
        let initial_edges = graph.data_flow_edges.clone();
        let source_node_id = graph.nodes[0].node_id.clone();
        let filter_node_id = graph.nodes[1].node_id.clone();
        let output_node_id = graph.nodes[3].node_id.clone();

        let valid_edge_id = graph
            .preview_add_data_flow_edge(
                &source_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
                &output_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
            )
            .expect("source to output should preview as a valid DAG edge");
        assert_eq!(
            valid_edge_id,
            GraphDocument::data_flow_edge_id(
                &source_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
                &output_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
            )
        );

        let duplicate = graph
            .preview_add_data_flow_edge(
                &source_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
                &filter_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
            )
            .expect_err("existing source to filter edge should preview as duplicate");
        assert_eq!(
            duplicate.status,
            GraphDataFlowEdgeDiagnosticStatus::DuplicateConnection
        );
        assert_eq!(graph.data_flow_edges, initial_edges);
        assert!(graph.command_history.undo_stack.is_empty());
    }

    #[test]
    fn add_data_flow_edge_records_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        let initial_edge_count = graph.data_flow_edges.len();
        let source_node_id = graph.nodes[0].node_id.clone();
        let output_node_id = graph.nodes[3].node_id.clone();
        let edge_id = graph
            .add_data_flow_edge(
                &source_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
                &output_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
            )
            .expect("source to output connection should be a valid DAG edge");

        assert_eq!(graph.data_flow_edges.len(), initial_edge_count + 1);
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == edge_id)
        );
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::DataFlowEdgeAdd { edge, .. }) if edge.edge_id == edge_id
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Add connection /obj/main/Source:geometry -> /obj/main/Rerun Output:geometry")
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.data_flow_edges.len(), initial_edge_count);
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == edge_id)
        );
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Add connection /obj/main/Source:geometry -> /obj/main/Rerun Output:geometry")
        );

        assert!(graph.redo_project_command());
        assert_eq!(graph.data_flow_edges.len(), initial_edge_count + 1);
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == edge_id)
        );
    }

    #[test]
    fn add_data_flow_edge_rejects_duplicate_invalid_and_cyclic_edges() {
        let mut graph = GraphDocument::sample();
        let initial_edge_count = graph.data_flow_edges.len();
        let source_node_id = graph.nodes[0].node_id.clone();
        let filter_node_id = graph.nodes[1].node_id.clone();
        let style_node_id = graph.nodes[2].node_id.clone();

        let duplicate = graph
            .add_data_flow_edge(
                &source_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
                &filter_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
            )
            .expect_err("default source to filter connection already exists");
        assert_eq!(
            duplicate.status,
            GraphDataFlowEdgeDiagnosticStatus::DuplicateConnection
        );

        let invalid_port = graph
            .add_data_flow_edge(
                &source_node_id,
                "mask",
                &filter_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
            )
            .expect_err("unknown source port should be rejected");
        assert_eq!(
            invalid_port.status,
            GraphDataFlowEdgeDiagnosticStatus::MissingSourcePort
        );

        let cycle = graph
            .add_data_flow_edge(
                &style_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
                &source_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
            )
            .expect_err("reverse style to source connection should be cyclic");
        assert_eq!(cycle.status, GraphDataFlowEdgeDiagnosticStatus::Cycle);

        assert_eq!(graph.data_flow_edges.len(), initial_edge_count);
        assert!(graph.command_history.undo_stack.is_empty());
    }

    #[test]
    fn explicit_data_flow_edge_survives_non_topology_undo() {
        let mut graph = GraphDocument::sample();
        let source_node_id = graph.nodes[0].node_id.clone();
        let output_node_id = graph.nodes[3].node_id.clone();
        let edge_id = graph
            .add_data_flow_edge(
                &source_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
                &output_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
            )
            .expect("source to output connection should be a valid DAG edge");

        assert!(graph.set_node_parameter_value(1, 0.9));
        assert!(graph.undo_project_command());

        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == edge_id)
        );
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Add connection /obj/main/Source:geometry -> /obj/main/Rerun Output:geometry")
        );
    }

    #[test]
    fn remove_data_flow_edge_records_undoable_project_command() {
        let mut graph = GraphDocument::sample();
        let initial_edge_count = graph.data_flow_edges.len();
        let source_node_id = graph.nodes[0].node_id.clone();
        let output_node_id = graph.nodes[3].node_id.clone();
        let edge_id = graph
            .add_data_flow_edge(
                &source_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
                &output_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
            )
            .expect("source to output connection should be a valid DAG edge");
        assert_eq!(graph.data_flow_edges.len(), initial_edge_count + 1);

        let removed_edge = graph
            .remove_data_flow_edge(&edge_id)
            .expect("explicit edge should be removable");

        assert_eq!(removed_edge.edge_id, edge_id);
        assert_eq!(graph.data_flow_edges.len(), initial_edge_count);
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == edge_id)
        );
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Remove connection /obj/main/Source:geometry -> /obj/main/Rerun Output:geometry")
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.data_flow_edges.len(), initial_edge_count + 1);
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == edge_id)
        );

        assert!(graph.redo_project_command());
        assert_eq!(graph.data_flow_edges.len(), initial_edge_count);
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == edge_id)
        );
    }

    #[test]
    fn remove_data_flow_edge_rejects_missing_edge_without_history() {
        let mut graph = GraphDocument::sample();
        let initial_edge_count = graph.data_flow_edges.len();

        assert!(graph.remove_data_flow_edge("missing.edge").is_none());

        assert_eq!(graph.data_flow_edges.len(), initial_edge_count);
        assert!(graph.command_history.undo_stack.is_empty());
    }

    #[test]
    fn insert_node_on_data_flow_edge_records_atomic_rewire_command() {
        let mut graph = GraphDocument::sample();
        let source_node_id = graph.nodes[0].node_id.clone();
        let filter_node_id = graph.nodes[1].node_id.clone();
        let removed_edge_id = GraphDocument::data_flow_edge_id(
            &source_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
            &filter_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
        );
        let inserted_node_index = graph
            .duplicate_node(1)
            .expect("filter should duplicate as compatible insert node");
        let inserted_node_id = graph.nodes[inserted_node_index].node_id.clone();
        let inserted_node_name = graph.nodes[inserted_node_index].name.clone();
        let incoming_edge_id = GraphDocument::data_flow_edge_id(
            &source_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
            &inserted_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
        );
        let outgoing_edge_id = GraphDocument::data_flow_edge_id(
            &inserted_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
            &filter_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
        );
        let edge_count_before_insert = graph.data_flow_edges.len();

        let result = graph
            .insert_node_on_data_flow_edge(
                &removed_edge_id,
                &inserted_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
                PRIMARY_GEOMETRY_OUTPUT,
            )
            .expect("source to filter edge should exist")
            .unwrap_or_else(|diagnostics| panic!("insert should be valid: {diagnostics:?}"));

        assert_eq!(result.removed_edge.edge_id, removed_edge_id);
        assert_eq!(result.added_edges.len(), 2);
        assert_eq!(graph.data_flow_edges.len(), edge_count_before_insert + 1);
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == removed_edge_id)
        );
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == incoming_edge_id)
        );
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == outgoing_edge_id)
        );
        assert_eq!(
            graph.nodes[inserted_node_index].evaluation.state,
            EvaluationState::Stale
        );
        assert_eq!(graph.nodes[1].evaluation.state, EvaluationState::Stale);
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::DataFlowEdgeInsertNode {
                inserted_node_name: recorded_name,
                removed_edge,
                added_edges,
                ..
            }) if recorded_name == &inserted_node_name
                && removed_edge.edge_id == removed_edge_id
                && added_edges.len() == 2
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some(
                format!(
                    "Insert {inserted_node_name} on connection /obj/main/Source:geometry -> /obj/main/Filter:geometry"
                )
                .as_str()
            )
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.data_flow_edges.len(), edge_count_before_insert);
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == removed_edge_id)
        );
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == incoming_edge_id)
        );
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == outgoing_edge_id)
        );

        assert!(graph.redo_project_command());
        assert!(
            !graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == removed_edge_id)
        );
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == incoming_edge_id)
        );
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == outgoing_edge_id)
        );
    }

    #[test]
    fn insert_node_on_data_flow_edge_returns_diagnostics_without_mutating() {
        let mut graph = GraphDocument::sample();
        let source_node_id = graph.nodes[0].node_id.clone();
        let filter_node_id = graph.nodes[1].node_id.clone();
        let output_node_id = graph.nodes[3].node_id.clone();
        let removed_edge_id = GraphDocument::data_flow_edge_id(
            &source_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
            &filter_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
        );
        let edges_before = graph.data_flow_edges.clone();

        let diagnostics = match graph
            .insert_node_on_data_flow_edge(
                &removed_edge_id,
                &output_node_id,
                PRIMARY_GEOMETRY_OUTPUT,
                PRIMARY_GEOMETRY_OUTPUT,
            )
            .expect("source to filter edge should exist")
        {
            Ok(_) => panic!("output node should not be insertable as a producing middle node"),
            Err(diagnostics) => diagnostics,
        };

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].status,
            GraphDataFlowEdgeDiagnosticStatus::IncompatibleDataKind
        );
        assert_eq!(graph.data_flow_edges, edges_before);
        assert!(graph.command_history.undo_stack.is_empty());
        assert!(
            graph
                .insert_node_on_data_flow_edge(
                    "missing.edge",
                    &output_node_id,
                    PRIMARY_GEOMETRY_OUTPUT,
                    PRIMARY_GEOMETRY_OUTPUT,
                )
                .is_none()
        );
    }

    #[test]
    fn graph_data_flow_edge_diagnostics_retain_loaded_invalid_edges() {
        let mut graph = GraphDocument::sample();
        graph.data_flow_edges.push(GraphDataFlowEdge {
            edge_id: "missing_source".to_owned(),
            from_node_id: "missing.node".to_owned(),
            from_output: "geometry".to_owned(),
            to_node_id: graph.nodes[0].node_id.clone(),
            to_input: "geometry".to_owned(),
        });
        graph.data_flow_edges.push(GraphDataFlowEdge {
            edge_id: "loaded_cycle".to_owned(),
            from_node_id: graph.nodes[2].node_id.clone(),
            from_output: "geometry".to_owned(),
            to_node_id: graph.nodes[0].node_id.clone(),
            to_input: "geometry".to_owned(),
        });

        let diagnostics = graph.data_flow_edge_diagnostics();

        assert_eq!(graph.data_flow_edges.len(), graph.nodes.len() + 1);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.edge_id == "missing_source"
                && diagnostic.status == GraphDataFlowEdgeDiagnosticStatus::MissingSourceNode
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.edge_id == "loaded_cycle"
                && diagnostic.status == GraphDataFlowEdgeDiagnosticStatus::Cycle
        }));
        assert!(
            graph
                .data_flow_edges
                .iter()
                .any(|edge| edge.edge_id == "loaded_cycle")
        );
    }

    #[test]
    fn graph_data_flow_edge_diagnostics_scope_to_selected_graph() {
        let mut graph = GraphDocument::sample();
        graph.graph_registry.graphs.push(ProjectGraphMetadata {
            graph_id: "analysis".to_owned(),
            name: "Analysis".to_owned(),
            path: "/obj/analysis".to_owned(),
            role: ProjectGraphRole::Subgraph,
        });
        graph
            .select_graph_by_id("analysis")
            .expect("analysis graph should be selectable");
        let analysis_source_index = graph.add_null_operator_node("OUT_A");
        let analysis_target_index = graph.add_null_operator_node("OUT_B");
        graph.data_flow_edges.push(GraphDataFlowEdge {
            edge_id: "analysis_missing_source".to_owned(),
            from_node_id: "missing.analysis_source".to_owned(),
            from_output: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
            to_node_id: graph.nodes[analysis_target_index].node_id.clone(),
            to_input: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
        });
        graph.data_flow_edges.push(GraphDataFlowEdge {
            edge_id: "analysis_missing_target".to_owned(),
            from_node_id: graph.nodes[analysis_source_index].node_id.clone(),
            from_output: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
            to_node_id: "missing.analysis_target".to_owned(),
            to_input: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
        });
        graph.data_flow_edges.push(GraphDataFlowEdge {
            edge_id: "main_loaded_cycle".to_owned(),
            from_node_id: graph.nodes[2].node_id.clone(),
            from_output: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
            to_node_id: graph.nodes[0].node_id.clone(),
            to_input: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
        });

        let selected_graph_diagnostics = graph.current_graph_data_flow_edge_diagnostics();

        assert!(selected_graph_diagnostics.iter().any(|diagnostic| {
            diagnostic.edge_id == "analysis_missing_source"
                && diagnostic.status == GraphDataFlowEdgeDiagnosticStatus::MissingSourceNode
        }));
        assert!(selected_graph_diagnostics.iter().any(|diagnostic| {
            diagnostic.edge_id == "analysis_missing_target"
                && diagnostic.status == GraphDataFlowEdgeDiagnosticStatus::MissingTargetNode
        }));
        assert!(
            selected_graph_diagnostics
                .iter()
                .all(|diagnostic| diagnostic.edge_id != "main_loaded_cycle")
        );
        assert!(
            graph
                .data_flow_edge_diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.edge_id == "main_loaded_cycle"
                    && diagnostic.status == GraphDataFlowEdgeDiagnosticStatus::Cycle)
        );
    }

    #[test]
    fn graph_data_flow_edge_diagnostics_distinguish_ports_and_kind() {
        let mut graph = GraphDocument::sample();
        graph.data_flow_edges.push(GraphDataFlowEdge {
            edge_id: "missing_source_port".to_owned(),
            from_node_id: graph.nodes[0].node_id.clone(),
            from_output: "mask".to_owned(),
            to_node_id: graph.nodes[1].node_id.clone(),
            to_input: "geometry".to_owned(),
        });
        graph.data_flow_edges.push(GraphDataFlowEdge {
            edge_id: "missing_target_port".to_owned(),
            from_node_id: graph.nodes[0].node_id.clone(),
            from_output: "geometry".to_owned(),
            to_node_id: graph.nodes[1].node_id.clone(),
            to_input: "mask".to_owned(),
        });
        graph.data_flow_edges.push(GraphDataFlowEdge {
            edge_id: "incompatible_output_kind".to_owned(),
            from_node_id: graph.nodes[3].node_id.clone(),
            from_output: "geometry".to_owned(),
            to_node_id: graph.nodes[1].node_id.clone(),
            to_input: "geometry".to_owned(),
        });

        let diagnostics = graph.data_flow_edge_diagnostics();

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.edge_id == "missing_source_port"
                && diagnostic.status == GraphDataFlowEdgeDiagnosticStatus::MissingSourcePort
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.edge_id == "missing_target_port"
                && diagnostic.status == GraphDataFlowEdgeDiagnosticStatus::MissingTargetPort
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.edge_id == "incompatible_output_kind"
                && diagnostic.status == GraphDataFlowEdgeDiagnosticStatus::IncompatibleDataKind
        }));
    }

    #[test]
    fn selected_node_info_reports_explicit_edge_counts() {
        let graph = GraphDocument::sample();
        let source_info = graph
            .selected_node_info(0)
            .expect("source node info should exist");
        let filter_info = graph
            .selected_node_info(1)
            .expect("filter node info should exist");
        let output_info = graph
            .selected_node_info(3)
            .expect("output node info should exist");

        assert_eq!(source_info.data_flow.incoming_edge_count, 0);
        assert_eq!(source_info.data_flow.outgoing_edge_count, 1);
        assert_eq!(filter_info.data_flow.incoming_edge_count, 1);
        assert_eq!(filter_info.data_flow.outgoing_edge_count, 1);
        assert_eq!(output_info.data_flow.incoming_edge_count, 1);
        assert_eq!(output_info.data_flow.outgoing_edge_count, 0);
    }

    #[test]
    fn selected_node_info_reports_readable_connection_diagnostics() {
        let mut graph = GraphDocument::sample();
        graph.data_flow_edges.push(GraphDataFlowEdge {
            edge_id: "filter_bad_target_port".to_owned(),
            from_node_id: graph.nodes[0].node_id.clone(),
            from_output: "geometry".to_owned(),
            to_node_id: graph.nodes[1].node_id.clone(),
            to_input: "mask".to_owned(),
        });
        graph.data_flow_edges.push(GraphDataFlowEdge {
            edge_id: "style_cycle".to_owned(),
            from_node_id: graph.nodes[2].node_id.clone(),
            from_output: "geometry".to_owned(),
            to_node_id: graph.nodes[0].node_id.clone(),
            to_input: "geometry".to_owned(),
        });

        let source_info = graph
            .selected_node_info(0)
            .expect("source node info should exist");
        let filter_info = graph
            .selected_node_info(1)
            .expect("filter node info should exist");

        assert!(source_info.data_flow.diagnostics.iter().any(|diagnostic| {
            diagnostic.status == GraphDataFlowEdgeDiagnosticStatus::Cycle
                && diagnostic.readable_path
                    == "/obj/main/Style:geometry -> /obj/main/Source:geometry"
        }));
        assert!(filter_info.data_flow.diagnostics.iter().any(|diagnostic| {
            diagnostic.status == GraphDataFlowEdgeDiagnosticStatus::MissingTargetPort
                && diagnostic.readable_path == "/obj/main/Source:geometry -> /obj/main/Filter:mask"
        }));
    }

    #[test]
    fn sidecar_without_data_flow_edges_rebuilds_default_edge_spine() {
        let graph = GraphDocument::sample();
        let mut value =
            serde_json::from_str::<serde_json::Value>(&graph.to_sidecar_json().unwrap())
                .expect("sidecar should be valid json");
        value
            .as_object_mut()
            .expect("sidecar should be an object")
            .remove("data_flow_edges");
        let json = serde_json::to_string_pretty(&value).unwrap();

        let mut restored = GraphDocument::sample();
        restored.data_flow_edges.clear();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(restored.data_flow_edges, graph.data_flow_edges);
        assert_eq!(restored.graph_layout().edges.len(), graph.nodes.len() - 1);
    }

    #[test]
    fn evaluation_mode_defaults_to_on_interaction_complete() {
        let graph = GraphDocument::sample();

        assert_eq!(
            graph.evaluation_mode,
            GraphEvaluationMode::OnInteractionComplete
        );
        assert_eq!(graph.evaluation_mode.as_str(), "On interaction complete");
    }

    #[test]
    fn evaluation_mode_round_trips_through_sidecar_as_project_intent() {
        let mut graph = GraphDocument::sample();
        graph.evaluation_mode = GraphEvaluationMode::Manual;
        graph.request_node_run(1);

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(restored.evaluation_mode, GraphEvaluationMode::Manual);
        assert!(restored.work_items.is_empty());
    }

    #[test]
    fn output_demand_evaluates_stale_connected_nodes_only() {
        let mut graph = GraphDocument::sample();
        graph.nodes.push(GraphNode {
            node_id: "scratch.filter".to_owned(),
            parent_graph_id: "main".to_owned(),
            name: "Scratch Filter".to_owned(),
            kind: NodeKind::Filter,
            layout_position: GraphPoint::new(0.5, 0.1),
            generated: None,
            coordinate_contract: Some(SubstrateCoordinateContract::demo_byteplot()),
            source_node: None,
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
    fn manual_evaluation_mode_queues_stale_output_instead_of_caching() {
        let mut graph = GraphDocument::sample();
        graph.evaluation_mode = GraphEvaluationMode::Manual;
        graph.mark_node_stale(1);

        graph.demand_output_evaluation();

        assert_eq!(graph.nodes[1].evaluation.state, EvaluationState::Manual);
        assert!(graph.nodes[1].evaluation.manual);
        assert_eq!(
            graph.nodes[1].evaluation.message.as_deref(),
            Some("Waiting for manual evaluation")
        );
        assert!(graph.work_items.iter().any(|item| {
            item.node_index == 1
                && item.status == GraphWorkItemStatus::Waiting
                && item.summary == "Manual evaluation mode is waiting for an explicit run"
        }));
    }

    #[test]
    fn work_items_record_manual_run_cancel_retry_and_completion() {
        let mut graph = GraphDocument::sample();
        let node_index = 1;

        graph.queue_node_evaluation(node_index);
        assert_eq!(graph.work_items.len(), 1);
        assert_eq!(graph.work_items[0].status, GraphWorkItemStatus::Waiting);
        assert_eq!(
            graph.nodes[node_index].evaluation.state,
            EvaluationState::Stale
        );

        graph.request_node_run(node_index);
        assert_eq!(
            graph.work_items.last().map(|item| item.status),
            Some(GraphWorkItemStatus::Running)
        );
        assert_eq!(
            graph.nodes[node_index].evaluation.state,
            EvaluationState::Running
        );

        graph.cancel_node_run(node_index);
        assert_eq!(
            graph.work_items.last().map(|item| item.status),
            Some(GraphWorkItemStatus::Canceled)
        );
        assert_eq!(
            graph.nodes[node_index].evaluation.state,
            EvaluationState::Manual
        );

        graph.retry_work_item_for_node(node_index);
        let retry = graph
            .work_items
            .last()
            .expect("retry should add a work item");
        assert_eq!(retry.status, GraphWorkItemStatus::Running);
        assert_eq!(retry.summary, "Retry requested for current graph request");
        assert_eq!(retry.output_name, PRIMARY_GEOMETRY_OUTPUT);
        assert_eq!(
            retry.fingerprint,
            graph.evaluation_fingerprint_for_node(node_index)
        );

        graph.complete_node_run(node_index);
        assert_eq!(
            graph.work_items.last().map(|item| item.status),
            Some(GraphWorkItemStatus::Complete)
        );
        assert_eq!(
            graph.nodes[node_index].evaluation.state,
            EvaluationState::Clean
        );
    }

    #[test]
    fn demand_evaluation_records_cached_work_items() {
        let mut graph = GraphDocument::sample();
        graph.nodes[1].evaluation = NodeEvaluation {
            state: EvaluationState::Stale,
            manual: false,
            message: None,
        };

        graph.demand_output_evaluation();

        assert_eq!(graph.nodes[1].evaluation.state, EvaluationState::Cached);
        assert!(graph.work_items.iter().any(|item| {
            item.node_index == 1
                && item.status == GraphWorkItemStatus::Cached
                && item.summary == "Cached output reused"
        }));
    }

    #[test]
    fn queued_evaluation_supersedes_running_work_for_same_node() {
        let mut graph = GraphDocument::sample();

        graph.request_node_run(1);
        graph.queue_node_evaluation(1);

        assert_eq!(graph.work_items[0].status, GraphWorkItemStatus::Superseded);
        assert_eq!(
            graph.work_items[0].summary,
            "Queued evaluation superseded previous running work"
        );
        assert_eq!(
            graph.work_items.last().map(|item| item.status),
            Some(GraphWorkItemStatus::Waiting)
        );
    }

    #[test]
    fn work_items_are_runtime_state_not_sidecar_state() {
        let mut graph = GraphDocument::sample();
        graph.request_node_run(1);
        assert!(!graph.work_items.is_empty());

        let json = graph.to_sidecar_json().unwrap();
        assert!(!json.contains("work_items"));
        assert!(!json.contains("work_item_"));

        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert!(restored.work_items.is_empty());
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
        assert_eq!(restored.annotations[0].parent_graph_id, "main");
        assert_eq!(
            restored.annotations[1].kind,
            GraphAnnotationKind::StickyNote
        );
        assert_eq!(restored.annotations[1].title, "Publish Note");
        assert_eq!(
            restored.annotations[1].text,
            "Raise threshold before output."
        );
        assert_eq!(restored.annotations[1].parent_graph_id, "main");

        let mut legacy_value: serde_json::Value =
            serde_json::from_str(&json).expect("sidecar should be valid json");
        if let Some(annotations) = legacy_value
            .get_mut("annotations")
            .and_then(|annotations| annotations.as_array_mut())
        {
            for annotation in annotations {
                annotation
                    .as_object_mut()
                    .expect("annotation should be object")
                    .remove("parent_graph_id");
            }
        }
        let legacy_json =
            serde_json::to_string_pretty(&legacy_value).expect("legacy sidecar should serialize");
        let mut legacy_restored = GraphDocument::sample();
        legacy_restored.apply_sidecar_json(&legacy_json).unwrap();
        assert!(
            legacy_restored
                .annotations
                .iter()
                .all(|annotation| annotation.parent_graph_id == "main")
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
        graph.network_view.comment_display_mode = NetworkCommentDisplayMode::AllCommented;
        graph.network_view.error_badge = NetworkBadgeVisibility::Hide;
        graph.network_view.warning_badge = NetworkBadgeVisibility::Large;
        graph.network_view.comment_badge = NetworkBadgeVisibility::Normal;
        graph.network_view.time_dependent_badge = NetworkBadgeVisibility::Hide;
        graph.network_view.lock_badge = NetworkBadgeVisibility::Large;
        graph.network_view.unload_badge = NetworkBadgeVisibility::Normal;
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
    fn graph_registry_defaults_to_main_graph_metadata() {
        let graph = GraphDocument::sample();
        let source_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Source)
            .expect("sample graph should include source node");
        let target = graph
            .reference_target_for_node(source_index)
            .expect("source should be referenceable");

        assert_eq!(graph.current_graph_id(), "main");
        assert_eq!(graph.current_graph_path(), "/obj/main");
        assert_eq!(graph.graph_registry.graphs.len(), 1);
        assert_eq!(
            graph.graph_registry.selected_graph(),
            Some(&ProjectGraphMetadata {
                graph_id: "main".to_owned(),
                name: "Main".to_owned(),
                path: "/obj/main".to_owned(),
                role: ProjectGraphRole::Main,
            })
        );
        assert_eq!(target.graph_id, "main");
    }

    #[test]
    fn graph_registry_round_trips_through_sidecar() {
        let mut graph = GraphDocument::sample();
        graph.graph_registry = ProjectGraphRegistry {
            selected_graph_id: "analysis".to_owned(),
            graphs: vec![
                ProjectGraphMetadata {
                    graph_id: "main".to_owned(),
                    name: "Main".to_owned(),
                    path: "/obj/main".to_owned(),
                    role: ProjectGraphRole::Main,
                },
                ProjectGraphMetadata {
                    graph_id: "analysis".to_owned(),
                    name: "Analysis".to_owned(),
                    path: "/obj/analysis".to_owned(),
                    role: ProjectGraphRole::Subgraph,
                },
            ],
        };
        let null_index = graph.add_null_operator_node("OUT_ANALYSIS");
        let target = graph
            .reference_target_for_node(null_index)
            .expect("null should be referenceable");

        assert_eq!(target.graph_id, "analysis");
        assert_eq!(
            graph.resolve_reference_target(&target).readable_path,
            "analysis/OUT_ANALYSIS:geometry"
        );

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(restored.graph_registry, graph.graph_registry);
        assert_eq!(restored.current_graph_id(), "analysis");
        assert_eq!(restored.current_graph_path(), "/obj/analysis");
        assert_eq!(
            restored.resolve_reference_target(&target).readable_path,
            "analysis/OUT_ANALYSIS:geometry"
        );
    }

    #[test]
    fn graph_navigation_targets_expose_registry_metadata() {
        let mut graph = GraphDocument::sample();
        graph.graph_registry = ProjectGraphRegistry {
            selected_graph_id: "main".to_owned(),
            graphs: vec![
                ProjectGraphMetadata {
                    graph_id: "main".to_owned(),
                    name: "Main".to_owned(),
                    path: "/obj/main".to_owned(),
                    role: ProjectGraphRole::Main,
                },
                ProjectGraphMetadata {
                    graph_id: "analysis".to_owned(),
                    name: "Analysis".to_owned(),
                    path: "/obj/analysis".to_owned(),
                    role: ProjectGraphRole::Subgraph,
                },
            ],
        };

        assert_eq!(
            graph.graph_navigation_targets(),
            vec![
                GraphNavigationTarget {
                    graph_id: "main".to_owned(),
                    name: "Main".to_owned(),
                    path: "/obj/main".to_owned(),
                    role: ProjectGraphRole::Main,
                },
                GraphNavigationTarget {
                    graph_id: "analysis".to_owned(),
                    name: "Analysis".to_owned(),
                    path: "/obj/analysis".to_owned(),
                    role: ProjectGraphRole::Subgraph,
                },
            ]
        );
    }

    #[test]
    fn select_graph_by_id_updates_current_graph_and_reports_noops() {
        let mut graph = GraphDocument::sample();
        graph.graph_registry.graphs.push(ProjectGraphMetadata {
            graph_id: "analysis".to_owned(),
            name: "Analysis".to_owned(),
            path: "/obj/analysis".to_owned(),
            role: ProjectGraphRole::Subgraph,
        });

        let change = graph
            .select_graph_by_id("analysis")
            .expect("analysis graph should be selectable");
        assert!(change.changed);
        assert_eq!(change.previous_graph.graph_id, "main");
        assert_eq!(change.selected_graph.graph_id, "analysis");
        assert_eq!(graph.current_graph_id(), "analysis");
        assert_eq!(graph.current_graph_path(), "/obj/analysis");

        let same_graph_change = graph
            .select_graph_by_id("analysis")
            .expect("selecting current graph should be a no-op");
        assert!(!same_graph_change.changed);
        assert_eq!(same_graph_change.previous_graph.graph_id, "analysis");
        assert_eq!(same_graph_change.selected_graph.graph_id, "analysis");

        assert_eq!(
            graph.select_graph_by_id("missing"),
            Err(GraphNavigationError::MissingGraph {
                graph_id: "missing".to_owned(),
            })
        );
        assert_eq!(graph.current_graph_id(), "analysis");
    }

    #[test]
    fn enter_graph_container_node_selects_resolved_internal_graph() {
        let mut graph = GraphDocument::sample();
        let container_index = graph.add_graph_container_node(
            "Cleanup Subnet",
            ProjectGraphMetadata {
                graph_id: "graph.cleanup".to_owned(),
                name: "Cleanup".to_owned(),
                path: "/obj/main/cleanup".to_owned(),
                role: ProjectGraphRole::Subgraph,
            },
        );

        let change = graph
            .enter_graph_container_node(container_index)
            .expect("resolved graph container should be navigable");

        assert!(change.changed);
        assert_eq!(change.previous_graph.graph_id, "main");
        assert_eq!(change.selected_graph.graph_id, "graph.cleanup");
        assert_eq!(change.selected_graph.path, "/obj/main/cleanup");
        assert_eq!(graph.current_graph_id(), "graph.cleanup");
        assert_eq!(graph.current_graph_path(), "/obj/main/cleanup");
    }

    #[test]
    fn exit_current_graph_returns_to_parent_container() {
        let mut graph = GraphDocument::sample();
        let container_index = graph.add_graph_container_node(
            "Cleanup Subnet",
            ProjectGraphMetadata {
                graph_id: "graph.cleanup".to_owned(),
                name: "Cleanup".to_owned(),
                path: "/obj/main/cleanup".to_owned(),
                role: ProjectGraphRole::Subgraph,
            },
        );
        graph
            .enter_graph_container_node(container_index)
            .expect("resolved graph container should be navigable");

        let change = graph
            .exit_current_graph_to_parent_container()
            .expect("internal graph should have a parent container");

        assert!(change.navigation.changed);
        assert_eq!(change.container_node_index, container_index);
        assert_eq!(change.navigation.previous_graph.graph_id, "graph.cleanup");
        assert_eq!(change.navigation.selected_graph.graph_id, "main");
        assert_eq!(graph.current_graph_id(), "main");
        assert_eq!(
            graph.current_graph_parent_container_node_index(),
            None,
            "top-level graph should not report a parent subnet"
        );
    }

    #[test]
    fn enter_graph_container_node_rejects_unresolved_or_non_navigable_targets() {
        let mut graph = GraphDocument::sample();
        let source_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Source)
            .expect("sample graph should include source node");
        assert_eq!(
            graph.enter_graph_container_node(source_index),
            Err(GraphNavigationError::NodeIsNotGraphContainer {
                node_id: "source.main".to_owned(),
                node_name: "Source".to_owned(),
            })
        );

        let container_index = graph.add_graph_container_node(
            "Cleanup Subnet",
            ProjectGraphMetadata {
                graph_id: "graph.cleanup".to_owned(),
                name: "Cleanup".to_owned(),
                path: "/obj/main/cleanup".to_owned(),
                role: ProjectGraphRole::Subgraph,
            },
        );
        graph.graph_containers[0].navigable = false;
        assert_eq!(
            graph.enter_graph_container_node(container_index),
            Err(GraphNavigationError::ContainerNotNavigable {
                node_id: graph.nodes[container_index].node_id.clone(),
                internal_graph_id: "graph.cleanup".to_owned(),
            })
        );

        graph.graph_containers[0].navigable = true;
        graph
            .graph_registry
            .graphs
            .retain(|metadata| metadata.graph_id != "graph.cleanup");
        assert_eq!(
            graph.enter_graph_container_node(container_index),
            Err(GraphNavigationError::MissingInternalGraph {
                graph_id: "graph.cleanup".to_owned(),
            })
        );
        assert_eq!(graph.current_graph_id(), "main");
    }

    #[test]
    fn graph_container_node_points_to_internal_named_graph() {
        let mut graph = GraphDocument::sample();
        let node_index = graph.add_graph_container_node(
            "Cleanup Subnet",
            ProjectGraphMetadata {
                graph_id: "graph.cleanup".to_owned(),
                name: "Cleanup".to_owned(),
                path: "/obj/main/cleanup".to_owned(),
                role: ProjectGraphRole::Subgraph,
            },
        );

        let node = &graph.nodes[node_index];
        let info = graph
            .selected_node_info(node_index)
            .expect("graph container info should exist");
        let container = info
            .graph_container
            .expect("container inspector info should exist");

        assert_eq!(node.kind, NodeKind::GraphContainer);
        assert_eq!(node.name, "Cleanup Subnet");
        assert!(!node.participates_in_output);
        assert_eq!(info.status, NodeStatus::Healthy);
        assert_eq!(info.input_count, 1);
        assert_eq!(info.output_count, 1);
        assert_eq!(container.status, GraphContainerStatus::Resolved);
        assert_eq!(container.kind.as_str(), "Subnet");
        assert_eq!(container.inputs[0].name, "geometry");
        assert_eq!(
            container.inputs[0].data_kind,
            HoudiniDataKind::GeometryTable
        );
        assert_eq!(container.outputs[0].name, "geometry");
        assert_eq!(
            container.outputs[0].data_kind,
            HoudiniDataKind::GeometryTable
        );
        assert_eq!(container.internal_graph_id, "graph.cleanup");
        assert_eq!(container.internal_graph_name.as_deref(), Some("Cleanup"));
        assert_eq!(
            container.internal_graph_path.as_deref(),
            Some("/obj/main/cleanup")
        );
        assert!(container.navigable);
        assert_eq!(
            graph.graph_containers[0].container_node_id,
            graph.nodes[node_index].node_id
        );
    }

    #[test]
    fn graph_container_metadata_round_trips_through_sidecar() {
        let mut graph = GraphDocument::sample();
        let node_index = graph.add_graph_container_node(
            "Branch Subnet",
            ProjectGraphMetadata {
                graph_id: "graph.branch".to_owned(),
                name: "Branch".to_owned(),
                path: "/obj/main/branch".to_owned(),
                role: ProjectGraphRole::Subgraph,
            },
        );
        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        let restored_node_index = restored
            .nodes
            .iter()
            .position(|node| node.node_id == graph.nodes[node_index].node_id)
            .expect("container node should restore");
        let restored_info = restored
            .selected_node_info(restored_node_index)
            .expect("container info should restore");
        let restored_container = restored_info
            .graph_container
            .expect("container metadata should restore");

        assert_eq!(restored.graph_containers, graph.graph_containers);
        assert!(json.contains("graph_containers"));
        assert!(json.contains("boundary"));
        assert_eq!(restored_container.status, GraphContainerStatus::Resolved);
        assert_eq!(restored_container.internal_graph_id, "graph.branch");
        assert_eq!(
            restored_container.outputs[0].data_kind,
            HoudiniDataKind::GeometryTable
        );
        assert_eq!(
            restored_container.internal_graph_path.as_deref(),
            Some("/obj/main/branch")
        );
    }

    #[test]
    fn graph_container_sidecar_without_boundary_uses_geometry_default() {
        let mut graph = GraphDocument::sample();
        let node_index = graph.add_graph_container_node(
            "Legacy Subnet",
            ProjectGraphMetadata {
                graph_id: "graph.legacy".to_owned(),
                name: "Legacy".to_owned(),
                path: "/obj/main/legacy".to_owned(),
                role: ProjectGraphRole::Subgraph,
            },
        );
        let mut value =
            serde_json::from_str::<serde_json::Value>(&graph.to_sidecar_json().unwrap())
                .expect("sidecar should be valid json");
        value["graph_containers"][0]
            .as_object_mut()
            .expect("container metadata should be an object")
            .remove("boundary");
        let json = serde_json::to_string_pretty(&value).unwrap();

        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        let restored_node_index = restored
            .nodes
            .iter()
            .position(|node| node.node_id == graph.nodes[node_index].node_id)
            .expect("container node should restore");
        let restored_container = restored
            .selected_node_info(restored_node_index)
            .expect("container info should restore")
            .graph_container
            .expect("container metadata should restore");

        assert_eq!(restored_container.inputs.len(), 1);
        assert_eq!(restored_container.outputs.len(), 1);
        assert_eq!(restored_container.outputs[0].name, PRIMARY_GEOMETRY_OUTPUT);
        assert_eq!(
            restored_container.outputs[0].data_kind,
            HoudiniDataKind::GeometryTable
        );
    }

    #[test]
    fn graph_container_boundary_output_is_referenceable() {
        let mut graph = GraphDocument::sample();
        let node_index = graph.add_graph_container_node(
            "Referenceable Subnet",
            ProjectGraphMetadata {
                graph_id: "graph.referenceable".to_owned(),
                name: "Referenceable".to_owned(),
                path: "/obj/main/referenceable".to_owned(),
                role: ProjectGraphRole::Subgraph,
            },
        );
        graph.graph_containers[0].boundary.outputs[0].name = "clean_geometry".to_owned();

        let target = graph
            .reference_target_for_node(node_index)
            .expect("graph container boundary should be referenceable");
        let reference_index = graph
            .add_reference_input_node(node_index)
            .expect("graph container boundary output should create a reference input");
        let resolution = graph.resolve_reference_target(&target);

        assert_eq!(target.node_id, graph.nodes[node_index].node_id);
        assert_eq!(target.output_name, "clean_geometry");
        assert_eq!(resolution.status, ReferenceDiagnosticStatus::Resolved);
        assert_eq!(resolution.output_kind, Some(HoudiniDataKind::GeometryTable));
        assert_eq!(
            graph.nodes[reference_index]
                .reference_input
                .as_ref()
                .expect("reference input should exist")
                .targets[0]
                .target
                .output_name,
            "clean_geometry"
        );
    }

    #[test]
    fn graph_container_boundary_data_flow_diagnostics_use_port_kind() {
        let mut graph = GraphDocument::sample();
        let node_index = graph.add_graph_container_node(
            "Scalar Subnet",
            ProjectGraphMetadata {
                graph_id: "graph.scalar".to_owned(),
                name: "Scalar".to_owned(),
                path: "/obj/main/scalar".to_owned(),
                role: ProjectGraphRole::Subgraph,
            },
        );
        graph.graph_containers[0].boundary = GraphBoundaryDeclaration {
            inputs: vec![HoudiniOperatorPort::geometry(
                PRIMARY_GEOMETRY_OUTPUT,
                "Geometry input.",
            )],
            outputs: vec![HoudiniOperatorPort {
                name: "threshold".to_owned(),
                data_kind: HoudiniDataKind::Scalar,
                required: true,
                help: "Scalar threshold output.".to_owned(),
            }],
            mappings: Vec::new(),
        };
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .expect("sample graph should include output node");
        let edge = GraphDataFlowEdge {
            edge_id: "scalar-edge".to_owned(),
            from_node_id: graph.nodes[node_index].node_id.clone(),
            from_output: "threshold".to_owned(),
            to_node_id: graph.nodes[output_index].node_id.clone(),
            to_input: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
        };

        let diagnostic = graph
            .data_flow_edge_endpoint_diagnostic(&edge)
            .expect("scalar boundary output should not satisfy geometry edge");

        assert_eq!(
            diagnostic.status,
            GraphDataFlowEdgeDiagnosticStatus::IncompatibleDataKind
        );
        assert!(diagnostic.message.contains("threshold"));
    }

    #[test]
    fn graph_container_boundary_mappings_round_trip_and_inspect() {
        let mut graph = GraphDocument::sample();
        let node_index = graph.add_graph_container_node(
            "Mapped Subnet",
            ProjectGraphMetadata {
                graph_id: "graph.mapped".to_owned(),
                name: "Mapped".to_owned(),
                path: "/obj/main/mapped".to_owned(),
                role: ProjectGraphRole::Subgraph,
            },
        );
        graph.graph_containers[0]
            .boundary
            .mappings
            .push(GraphBoundaryMapping {
                direction: GraphBoundaryMappingDirection::Input,
                public_port_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
                internal_node_id: "IN_GEOMETRY".to_owned(),
                internal_port_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
            });
        graph.graph_containers[0]
            .boundary
            .mappings
            .push(GraphBoundaryMapping {
                direction: GraphBoundaryMappingDirection::Output,
                public_port_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
                internal_node_id: "OUT_GEOMETRY".to_owned(),
                internal_port_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
            });

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();
        let restored_node_index = restored
            .nodes
            .iter()
            .position(|node| node.node_id == graph.nodes[node_index].node_id)
            .expect("container node should restore");
        let restored_info = restored
            .selected_node_info(restored_node_index)
            .expect("container info should restore");
        let restored_container = restored_info
            .graph_container
            .expect("container mappings should inspect");

        assert!(json.contains("mappings"));
        assert_eq!(
            restored.graph_containers[0].boundary.mappings,
            graph.graph_containers[0].boundary.mappings
        );
        assert_eq!(restored_info.status, NodeStatus::Healthy);
        assert_eq!(restored_container.mappings.len(), 2);
        assert_eq!(
            restored_container.mappings[0].status,
            GraphBoundaryMappingStatus::Resolved
        );
        assert_eq!(
            restored_container.mappings[1].internal_node_id,
            "OUT_GEOMETRY"
        );
    }

    #[test]
    fn graph_container_boundary_mapping_diagnostics_report_unresolved_metadata() {
        let mut graph = GraphDocument::sample();
        let node_index = graph.add_graph_container_node(
            "Broken Mapping Subnet",
            ProjectGraphMetadata {
                graph_id: "graph.broken_mapping".to_owned(),
                name: "Broken Mapping".to_owned(),
                path: "/obj/main/broken_mapping".to_owned(),
                role: ProjectGraphRole::Subgraph,
            },
        );
        graph.graph_containers[0]
            .boundary
            .mappings
            .push(GraphBoundaryMapping {
                direction: GraphBoundaryMappingDirection::Output,
                public_port_name: "missing_output".to_owned(),
                internal_node_id: "OUT_GEOMETRY".to_owned(),
                internal_port_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
            });
        graph.graph_containers[0]
            .boundary
            .mappings
            .push(GraphBoundaryMapping {
                direction: GraphBoundaryMappingDirection::Input,
                public_port_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
                internal_node_id: String::new(),
                internal_port_name: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
            });

        let info = graph
            .selected_node_info(node_index)
            .expect("container info should exist");
        let container = info
            .graph_container
            .expect("container mappings should inspect");

        assert_eq!(info.status, NodeStatus::Failed);
        assert_eq!(
            container.mappings[0].status,
            GraphBoundaryMappingStatus::MissingPublicPort
        );
        assert_eq!(
            container.mappings[1].status,
            GraphBoundaryMappingStatus::MissingInternalAnchor
        );
        assert!(
            info.warnings
                .iter()
                .any(|warning| warning.contains("missing public port"))
        );
        assert!(
            info.warnings
                .iter()
                .any(|warning| warning.contains("missing internal anchor"))
        );
    }

    #[test]
    fn graph_container_collapse_manifest_captures_connected_selection_crossings() {
        let mut graph = GraphDocument::sample();
        let source_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "source.main")
            .expect("sample graph should include source");
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "filter.main")
            .expect("sample graph should include filter");
        let style_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "style.main")
            .expect("sample graph should include style");
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "output.rerun")
            .expect("sample graph should include output");
        let source_node_id = graph.nodes[source_index].node_id.clone();
        let output_node_id = graph.nodes[output_index].node_id.clone();
        let selected_node_ids = vec![
            graph.nodes[filter_index].node_id.clone(),
            graph.nodes[style_index].node_id.clone(),
        ];

        let container_index = graph
            .add_graph_container_collapse_manifest_for_node_set(
                "Cleanup Subnet",
                &[filter_index, style_index],
            )
            .expect("connected filter and style selection should collapse");
        let container_node_id = graph.nodes[container_index].node_id.clone();
        let metadata = graph
            .graph_containers
            .iter()
            .find(|container| container.container_node_id == container_node_id)
            .expect("container metadata should be created");
        let manifest = metadata
            .collapse_manifest
            .as_ref()
            .expect("collapse manifest should be recorded");

        assert_eq!(metadata.internal_graph_id, "graph.cleanup_subnet");
        assert_eq!(metadata.boundary.inputs.len(), 1);
        assert_eq!(metadata.boundary.outputs.len(), 1);
        assert_eq!(metadata.boundary.mappings.len(), 2);
        assert_eq!(manifest.source_graph_id, "main");
        assert_eq!(manifest.captured_node_ids, selected_node_ids);
        assert_eq!(manifest.external_edges.len(), 2);
        assert!(graph.nodes.iter().any(|node| node.node_id == "filter.main"));
        assert!(graph.nodes.iter().any(|node| node.node_id == "style.main"));
        assert_eq!(
            graph
                .nodes
                .iter()
                .find(|node| node.node_id == "filter.main")
                .expect("filter node should remain graph-owned")
                .parent_graph_id,
            "graph.cleanup_subnet"
        );
        assert_eq!(
            graph
                .nodes
                .iter()
                .find(|node| node.node_id == "style.main")
                .expect("style node should remain graph-owned")
                .parent_graph_id,
            "graph.cleanup_subnet"
        );

        let input_edge = manifest
            .external_edges
            .iter()
            .find(|edge| edge.direction == GraphBoundaryMappingDirection::Input)
            .expect("collapse manifest should capture incoming edge");
        assert_eq!(input_edge.external_node_id, source_node_id);
        assert_eq!(input_edge.internal_node_id, "filter.main");
        assert_eq!(input_edge.public_port_name, PRIMARY_GEOMETRY_OUTPUT);
        assert_eq!(input_edge.data_kind, HoudiniDataKind::GeometryTable);

        let output_edge = manifest
            .external_edges
            .iter()
            .find(|edge| edge.direction == GraphBoundaryMappingDirection::Output)
            .expect("collapse manifest should capture outgoing edge");
        assert_eq!(output_edge.internal_node_id, "style.main");
        assert_eq!(output_edge.external_node_id, output_node_id);
        assert_eq!(output_edge.public_port_name, PRIMARY_GEOMETRY_OUTPUT);
        assert_eq!(output_edge.data_kind, HoudiniDataKind::GeometryTable);

        let main_layout = graph.graph_layout();
        assert_eq!(main_layout.nodes.len(), 3);
        assert_eq!(main_layout.edges.len(), 2);
        assert_eq!(
            graph.nodes[main_layout.edges[0].from_node].node_id,
            source_node_id
        );
        assert_eq!(
            graph.nodes[main_layout.edges[0].to_node].node_id,
            container_node_id
        );
        assert_eq!(
            graph.nodes[main_layout.edges[1].from_node].node_id,
            container_node_id
        );
        assert_eq!(
            graph.nodes[main_layout.edges[1].to_node].node_id,
            output_node_id
        );

        let internal_layout = graph.graph_layout_for_graph("graph.cleanup_subnet");
        assert_eq!(internal_layout.nodes.len(), 2);
        assert_eq!(internal_layout.edges.len(), 1);
        assert_eq!(
            graph.nodes[internal_layout.edges[0].from_node].node_id,
            "filter.main"
        );
        assert_eq!(
            graph.nodes[internal_layout.edges[0].to_node].node_id,
            "style.main"
        );

        let info = graph
            .selected_node_info(container_index)
            .expect("container node should inspect");
        let container_info = info
            .graph_container
            .expect("container info should be exposed");
        assert_eq!(
            container_info
                .collapse_manifest
                .expect("inspector info should include collapse manifest")
                .external_edges,
            manifest.external_edges
        );
    }

    #[test]
    fn graph_container_collapse_manifest_round_trips_through_sidecar() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "filter.main")
            .expect("sample graph should include filter");
        let style_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "style.main")
            .expect("sample graph should include style");
        let container_index = graph
            .add_graph_container_collapse_manifest_for_node_set(
                "Cleanup Subnet",
                &[filter_index, style_index],
            )
            .expect("connected selection should collapse");
        let container_node_id = graph.nodes[container_index].node_id.clone();
        let json = graph.to_sidecar_json().unwrap();

        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();
        let restored_container = restored
            .graph_containers
            .iter()
            .find(|container| container.container_node_id == container_node_id)
            .expect("container metadata should restore");

        assert!(json.contains("collapse_manifest"));
        assert_eq!(restored_container, &graph.graph_containers[0]);
        assert_eq!(
            restored_container
                .collapse_manifest
                .as_ref()
                .expect("collapse manifest should restore")
                .captured_node_ids,
            vec!["filter.main".to_owned(), "style.main".to_owned()]
        );
        assert_eq!(restored.data_flow_edges, graph.data_flow_edges);
        assert_eq!(
            restored
                .nodes
                .iter()
                .find(|node| node.node_id == "filter.main")
                .expect("filter node should restore")
                .parent_graph_id,
            "graph.cleanup_subnet"
        );
        assert_eq!(
            restored
                .nodes
                .iter()
                .find(|node| node.node_id == "style.main")
                .expect("style node should restore")
                .parent_graph_id,
            "graph.cleanup_subnet"
        );
        assert_eq!(restored.graph_layout().edges.len(), 2);
        assert_eq!(
            restored
                .graph_layout_for_graph("graph.cleanup_subnet")
                .edges
                .len(),
            1
        );
    }

    #[test]
    fn graph_container_collapse_manifest_rejects_invalid_selection() {
        let mut graph = GraphDocument::sample();
        let source_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "source.main")
            .expect("sample graph should include source");
        let style_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "style.main")
            .expect("sample graph should include style");

        assert_eq!(
            graph.add_graph_container_collapse_manifest_for_node_set("Empty", &[]),
            Err(GraphContainerCollapseError::EmptySelection)
        );
        assert_eq!(
            graph.add_graph_container_collapse_manifest_for_node_set("Missing", &[usize::MAX]),
            Err(GraphContainerCollapseError::MissingNodeIndex(usize::MAX))
        );
        assert_eq!(
            graph.add_graph_container_collapse_manifest_for_node_set(
                "Disconnected",
                &[source_index, style_index],
            ),
            Err(GraphContainerCollapseError::DisconnectedSelection)
        );
    }

    #[test]
    fn graph_container_reports_missing_internal_graph() {
        let mut graph = GraphDocument::sample();
        let node_index = graph.add_graph_container_node(
            "Missing Subnet",
            ProjectGraphMetadata {
                graph_id: "graph.deleted".to_owned(),
                name: "Deleted".to_owned(),
                path: "/obj/main/deleted".to_owned(),
                role: ProjectGraphRole::Subgraph,
            },
        );
        graph
            .graph_registry
            .graphs
            .retain(|metadata| metadata.graph_id != "graph.deleted");

        let info = graph
            .selected_node_info(node_index)
            .expect("container info should exist");
        let container = info
            .graph_container
            .expect("container inspector info should exist");

        assert_eq!(info.status, NodeStatus::Failed);
        assert_eq!(container.status, GraphContainerStatus::MissingInternalGraph);
        assert_eq!(container.internal_graph_id, "graph.deleted");
        assert!(!container.navigable);
        assert!(
            info.warnings
                .iter()
                .any(|warning| warning.contains("Missing internal graph"))
        );
    }

    #[test]
    fn selected_node_info_reports_current_graph_readable_node_path() {
        let mut graph = GraphDocument::sample();
        graph.graph_registry = ProjectGraphRegistry {
            selected_graph_id: "analysis".to_owned(),
            graphs: vec![
                ProjectGraphMetadata {
                    graph_id: "main".to_owned(),
                    name: "Main".to_owned(),
                    path: "/obj/main".to_owned(),
                    role: ProjectGraphRole::Main,
                },
                ProjectGraphMetadata {
                    graph_id: "analysis".to_owned(),
                    name: "Analysis".to_owned(),
                    path: "/obj/analysis".to_owned(),
                    role: ProjectGraphRole::Subgraph,
                },
            ],
        };
        let source_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Source)
            .expect("sample graph should include source node");
        graph.nodes[source_index].parent_graph_id = "analysis".to_owned();
        let info = graph
            .selected_node_info(source_index)
            .expect("selected node should report info");

        assert_eq!(
            graph.readable_node_path(source_index).as_deref(),
            Some("/obj/analysis/Source")
        );
        assert_eq!(info.graph_location.graph_id, "analysis");
        assert_eq!(info.graph_location.graph_path, "/obj/analysis");
        assert_eq!(info.graph_location.node_name, "Source");
        assert_eq!(info.graph_location.node_path, "/obj/analysis/Source");
        assert!(info.graph_location.name_is_unique_in_graph());
        assert_eq!(info.graph_location.name_collision_count, 1);
    }

    #[test]
    fn selected_node_info_reports_graph_local_unique_generated_name() {
        let mut graph = GraphDocument::sample();
        graph.graph_registry = ProjectGraphRegistry {
            selected_graph_id: "analysis".to_owned(),
            graphs: vec![
                ProjectGraphMetadata {
                    graph_id: "main".to_owned(),
                    name: "Main".to_owned(),
                    path: "/obj/main".to_owned(),
                    role: ProjectGraphRole::Main,
                },
                ProjectGraphMetadata {
                    graph_id: "analysis".to_owned(),
                    name: "Analysis".to_owned(),
                    path: "/obj/analysis".to_owned(),
                    role: ProjectGraphRole::Subgraph,
                },
            ],
        };

        let first_analysis_source_index = graph.add_null_operator_node("Source");
        let duplicate_name_index = graph.add_null_operator_node("Source");
        assert_eq!(graph.nodes[first_analysis_source_index].name, "Source");
        assert_eq!(
            graph.nodes[first_analysis_source_index].parent_graph_id,
            "analysis"
        );
        let info = graph
            .selected_node_info(duplicate_name_index)
            .expect("selected node should report info");

        assert_eq!(info.graph_location.node_name, "Source_2");
        assert_eq!(info.graph_location.node_path, "/obj/analysis/Source_2");
        assert!(info.graph_location.name_is_unique_in_graph());
        assert_eq!(info.graph_location.name_collision_count, 1);
    }

    #[test]
    fn graph_local_node_names_allow_same_name_across_parent_graphs() {
        let mut graph = GraphDocument::sample();
        graph.graph_registry = ProjectGraphRegistry {
            selected_graph_id: "analysis".to_owned(),
            graphs: vec![
                ProjectGraphMetadata {
                    graph_id: "main".to_owned(),
                    name: "Main".to_owned(),
                    path: "/obj/main".to_owned(),
                    role: ProjectGraphRole::Main,
                },
                ProjectGraphMetadata {
                    graph_id: "analysis".to_owned(),
                    name: "Analysis".to_owned(),
                    path: "/obj/analysis".to_owned(),
                    role: ProjectGraphRole::Subgraph,
                },
            ],
        };
        let analysis_source_index = graph.add_null_operator_node("Source");
        assert_eq!(graph.nodes[analysis_source_index].name, "Source");
        assert_eq!(
            graph.nodes[analysis_source_index].parent_graph_id,
            "analysis"
        );

        assert!(graph.set_node_name(analysis_source_index, "Filter"));
        assert_eq!(graph.nodes[analysis_source_index].name, "Filter");
        assert_eq!(
            graph.readable_node_path(analysis_source_index).as_deref(),
            Some("/obj/analysis/Filter")
        );

        let duplicate_index = graph
            .duplicate_node(analysis_source_index)
            .expect("analysis node should duplicate");
        assert_eq!(graph.nodes[duplicate_index].name, "Filter_2");
        assert_eq!(graph.nodes[duplicate_index].parent_graph_id, "analysis");

        let target = graph
            .reference_target_for_node(analysis_source_index)
            .expect("analysis node should expose a reference target");
        assert_eq!(target.graph_id, "analysis");
        graph.graph_registry.selected_graph_id = "main".to_owned();
        let resolution = graph.resolve_reference_target(&target);
        assert_eq!(resolution.status, ReferenceDiagnosticStatus::Resolved);
        assert_eq!(resolution.readable_path, "analysis/Filter:geometry");

        let main_filter_count = graph
            .nodes
            .iter()
            .filter(|node| node.parent_graph_id == "main" && node.name == "Filter")
            .count();
        assert_eq!(main_filter_count, 1);
    }

    #[test]
    fn graph_local_node_parent_round_trips_and_legacy_defaults_to_main() {
        let mut graph = GraphDocument::sample();
        graph.graph_registry.graphs.push(ProjectGraphMetadata {
            graph_id: "analysis".to_owned(),
            name: "Analysis".to_owned(),
            path: "/obj/analysis".to_owned(),
            role: ProjectGraphRole::Subgraph,
        });
        graph.nodes[0].parent_graph_id = "analysis".to_owned();

        let json = graph.to_sidecar_json().unwrap();
        assert!(json.contains("parent_graph_id"));
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();
        assert_eq!(restored.nodes[0].parent_graph_id, "analysis");
        assert_eq!(
            restored.readable_node_path(0).as_deref(),
            Some("/obj/analysis/Source")
        );

        let mut legacy_value =
            serde_json::from_str::<serde_json::Value>(&json).expect("sidecar should be valid json");
        for node in legacy_value["nodes"]
            .as_array_mut()
            .expect("nodes should be an array")
        {
            node.as_object_mut()
                .expect("node sidecar should be an object")
                .remove("parent_graph_id");
        }
        let legacy_json = serde_json::to_string_pretty(&legacy_value).unwrap();
        let mut legacy_restored = GraphDocument::sample();
        legacy_restored.apply_sidecar_json(&legacy_json).unwrap();
        assert!(
            legacy_restored
                .nodes
                .iter()
                .all(|node| node.parent_graph_id == "main")
        );
    }

    #[test]
    fn sidecar_without_graph_registry_uses_main_graph_default() {
        let graph = GraphDocument::sample();
        let mut value =
            serde_json::from_str::<serde_json::Value>(&graph.to_sidecar_json().unwrap())
                .expect("sidecar should be valid json");
        value
            .as_object_mut()
            .expect("sidecar should be an object")
            .remove("graph_registry");
        let json = serde_json::to_string_pretty(&value).unwrap();

        let mut restored = GraphDocument::sample();
        restored.graph_registry = ProjectGraphRegistry {
            selected_graph_id: "stale".to_owned(),
            graphs: Vec::new(),
        };
        restored.apply_sidecar_json(&json).unwrap();

        assert_eq!(restored.current_graph_id(), "main");
        assert_eq!(restored.current_graph_path(), "/obj/main");
        assert_eq!(restored.graph_registry, ProjectGraphRegistry::default());
    }

    #[test]
    fn network_comment_display_mode_controls_comment_visibility() {
        assert!(NetworkCommentDisplayMode::ManualOnly.shows_comment("review", true));
        assert!(!NetworkCommentDisplayMode::ManualOnly.shows_comment("review", false));
        assert!(NetworkCommentDisplayMode::AllCommented.shows_comment("review", false));
        assert!(!NetworkCommentDisplayMode::AllCommented.shows_comment("   ", true));
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
            restored.network_view.comment_display_mode,
            NetworkCommentDisplayMode::ManualOnly
        );
        assert_eq!(
            restored.network_view.lock_badge,
            NetworkBadgeVisibility::Normal
        );
        assert_eq!(
            restored.network_view.unload_badge,
            NetworkBadgeVisibility::Hide
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
        let original_position = graph.annotations[box_index].position;
        let original_size = graph.annotations[box_index].size;
        let annotation_title = graph.annotations[box_index].title.clone();

        assert!(graph.resize_network_box_to_contents(box_index));

        assert!((graph.annotations[box_index].position.x - 0.42).abs() < 0.0001);
        assert!((graph.annotations[box_index].position.y - 0.36).abs() < 0.0001);
        assert!((graph.annotations[box_index].size.x - 0.16).abs() < 0.0001);
        assert!((graph.annotations[box_index].size.y - 0.28).abs() < 0.0001);
        assert!(matches!(
            graph.command_history.undo_stack.last(),
            Some(ProjectCommand::AnnotationBoundsEdit {
                annotation_title: recorded_title,
                old_position,
                old_size,
                ..
            }) if recorded_title == &annotation_title
                && *old_position == original_position
                && *old_size == original_size
        ));
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Fit Network Box to contents")
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.annotations[box_index].position, original_position);
        assert_eq!(graph.annotations[box_index].size, original_size);
        assert_eq!(
            graph.redo_project_command_label().as_deref(),
            Some("Fit Network Box to contents")
        );

        assert!(graph.redo_project_command());
        assert!((graph.annotations[box_index].position.x - 0.42).abs() < 0.0001);
        assert!((graph.annotations[box_index].position.y - 0.36).abs() < 0.0001);
        assert!((graph.annotations[box_index].size.x - 0.16).abs() < 0.0001);
        assert!((graph.annotations[box_index].size.y - 0.28).abs() < 0.0001);
        assert!(!graph.resize_network_box_to_contents(box_index));
        assert!(!graph.resize_network_box_to_contents(graph.annotations.len()));
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
        assert_eq!(load.metadata.locator.kind, SourceLocatorKind::LocalPath);
        assert_eq!(
            load.metadata.locator.readable(),
            sample_path.display().to_string()
        );
        assert!(load.metadata.locator.is_external_reference());
        assert!(!load.metadata.locator.is_generated());
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

    #[test]
    fn source_locator_metadata_classifies_generated_and_external_sources() {
        let graph = GraphDocument::sample();
        assert_eq!(graph.source.metadata.locator.kind, SourceLocatorKind::Demo);
        assert!(!graph.source.metadata.locator.is_external_reference());
        assert!(graph.source.metadata.locator.is_generated());

        let uri_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some("s3://bucket/curves.parquet".to_owned()),
            &graph.geometry,
            Vec::new(),
        );
        assert_eq!(uri_metadata.locator.kind, SourceLocatorKind::Uri);
        assert_eq!(
            uri_metadata.locator.readable(),
            "s3://bucket/curves.parquet"
        );
        assert!(uri_metadata.locator.is_external_reference());
    }

    #[test]
    fn source_gallery_index_reports_local_mixed_collection_items() {
        let temp_dir = tempfile::tempdir().unwrap();
        let image_path = temp_dir.path().join("frame.png");
        let table_path = temp_dir.path().join("curves.parquet");
        let recording_path = temp_dir.path().join("scene.rrd");
        let unknown_path = temp_dir.path().join("notes.bin");
        let nested_dir = temp_dir.path().join("nested");
        std::fs::write(&image_path, b"not decoded in index slice").unwrap();
        std::fs::write(&table_path, b"parquet placeholder").unwrap();
        std::fs::write(&recording_path, b"rrd placeholder").unwrap();
        std::fs::write(&unknown_path, b"unknown placeholder").unwrap();
        std::fs::create_dir(&nested_dir).unwrap();

        let index = SourceGalleryIndex::from_locator(
            SourceLocator::from_location(&temp_dir.path().display().to_string()),
            16,
        );

        assert_eq!(index.items.len(), 4);
        assert!(!index.truncated);
        assert!(index.warnings.is_empty());

        let image = index
            .items
            .iter()
            .find(|item| item.display_name == "frame.png")
            .expect("image item should be indexed");
        assert_eq!(image.kind, SourceGalleryItemKind::Image);
        match &image.thumbnail_intent {
            SourceGalleryThumbnailIntent::Image(intent) => {
                assert_eq!(intent.cache_key, image.stable_id);
                assert_eq!(intent.status, SourceGalleryThumbnailStatus::DecodeReady);
            }
            SourceGalleryThumbnailIntent::Generic(_) => {
                panic!("image should request image thumbnail")
            }
        }
        assert_eq!(
            image.external_reference_status,
            SourceExternalReferenceStatus::LocalAvailable
        );

        let table = index
            .items
            .iter()
            .find(|item| item.display_name == "curves.parquet")
            .expect("parquet item should be indexed");
        assert_eq!(table.kind, SourceGalleryItemKind::Table);
        match &table.thumbnail_intent {
            SourceGalleryThumbnailIntent::Generic(intent) => {
                assert_eq!(intent.kind, SourceGalleryItemKind::Table);
                assert_eq!(intent.status, SourceGalleryThumbnailStatus::GenericOnly);
            }
            SourceGalleryThumbnailIntent::Image(_) => {
                panic!("parquet should use generic thumbnail")
            }
        }
        assert_eq!(table.format_kind, Some(SourceFormatKind::Parquet));
        assert_eq!(
            table.format_support_status,
            Some(SourceFormatSupportStatus::Supported)
        );

        let recording = index
            .items
            .iter()
            .find(|item| item.display_name == "scene.rrd")
            .expect("recording item should be indexed");
        assert_eq!(recording.kind, SourceGalleryItemKind::Recording);

        let unknown = index
            .items
            .iter()
            .find(|item| item.display_name == "notes.bin")
            .expect("unknown item should be indexed");
        assert_eq!(unknown.kind, SourceGalleryItemKind::Unknown);
        assert_eq!(
            unknown.thumbnail_intent.status(),
            SourceGalleryThumbnailStatus::GenericOnly
        );
    }

    #[test]
    fn source_gallery_open_actions_follow_kind_and_availability() {
        let temp_dir = tempfile::tempdir().unwrap();
        let image_path = temp_dir.path().join("frame.png");
        let recording_path = temp_dir.path().join("scene.rrd");
        let table_path = temp_dir.path().join("curves.parquet");
        let missing_path = temp_dir.path().join("missing.png");
        std::fs::write(&image_path, b"image placeholder").unwrap();
        std::fs::write(&recording_path, b"rrd placeholder").unwrap();
        std::fs::write(&table_path, b"parquet placeholder").unwrap();

        let index = SourceGalleryIndex::from_locations(
            SourceLocator::from_location("inline-gallery"),
            vec![
                SourceLocator::from_location(&image_path.display().to_string()),
                SourceLocator::from_location(&recording_path.display().to_string()),
                SourceLocator::from_location(&table_path.display().to_string()),
                SourceLocator::from_location(&missing_path.display().to_string()),
                SourceLocator::from_location("https://example.test/frame.png"),
                SourceLocator::generated("synthetic source"),
            ],
            16,
        );

        let image = index
            .items
            .iter()
            .find(|item| {
                item.display_name == "frame.png"
                    && item.locator.kind == SourceLocatorKind::LocalPath
            })
            .unwrap();
        let image_action = image.open_action_report();
        assert!(image_action.enabled);
        assert_eq!(image_action.kind, SourceGalleryOpenActionKind::OpenImage2D);

        let remote_image = index
            .items
            .iter()
            .find(|item| item.locator.readable() == "https://example.test/frame.png")
            .unwrap();
        let remote_action = remote_image.open_action_report();
        assert!(remote_action.enabled);
        assert_eq!(remote_action.kind, SourceGalleryOpenActionKind::OpenImage2D);
        assert_eq!(
            remote_image.external_reference_status,
            SourceExternalReferenceStatus::UriUnverified
        );

        let recording = index
            .items
            .iter()
            .find(|item| item.display_name == "scene.rrd")
            .unwrap();
        let recording_action = recording.open_action_report();
        assert!(recording_action.enabled);
        assert_eq!(
            recording_action.kind,
            SourceGalleryOpenActionKind::OpenRecording
        );

        let table = index
            .items
            .iter()
            .find(|item| item.display_name == "curves.parquet")
            .unwrap();
        let table_action = table.open_action_report();
        assert!(!table_action.enabled);
        assert_eq!(table_action.kind, SourceGalleryOpenActionKind::Unsupported);

        let missing = index
            .items
            .iter()
            .find(|item| item.display_name == "missing.png")
            .unwrap();
        let missing_action = missing.open_action_report();
        assert!(!missing_action.enabled);
        assert_eq!(
            missing_action.kind,
            SourceGalleryOpenActionKind::Unavailable
        );
        assert!(missing_action.status.contains("missing"));

        let generated = index
            .items
            .iter()
            .find(|item| item.display_name == "synthetic source")
            .unwrap();
        let generated_action = generated.open_action_report();
        assert!(!generated_action.enabled);
        assert_eq!(
            generated_action.kind,
            SourceGalleryOpenActionKind::Unavailable
        );
        assert!(generated_action.status.contains("external locator"));
    }

    #[test]
    fn source_gallery_single_entry_creates_undoable_source_node() {
        let temp_dir = tempfile::tempdir().unwrap();
        let image_path = temp_dir.path().join("frame.png");
        std::fs::write(&image_path, b"image bytes stay outside sidecar").unwrap();
        let index = SourceGalleryIndex::from_locator(
            SourceLocator::from_location(&image_path.display().to_string()),
            16,
        );
        let item = index.items.first().unwrap();
        let mut graph = GraphDocument::sample();
        let initial_node_count = graph.nodes.len();

        let node_index = graph.add_source_gallery_item_node(item);

        assert_eq!(graph.nodes.len(), initial_node_count + 1);
        let node = &graph.nodes[node_index];
        assert_eq!(node.kind, NodeKind::Source);
        assert!(!node.participates_in_output);
        let source_node = node.source_node.as_ref().expect("source payload");
        assert_eq!(source_node.entries.len(), 1);
        assert_eq!(source_node.entries[0].display_name, "frame.png");
        assert_eq!(
            source_node.entries[0].locator.kind,
            SourceLocatorKind::LocalPath
        );
        assert_eq!(
            graph.undo_project_command_label().as_deref(),
            Some("Create Source frame.png")
        );

        assert!(graph.undo_project_command());
        assert_eq!(graph.nodes.len(), initial_node_count);

        assert!(graph.redo_project_command());
        assert_eq!(graph.nodes.len(), initial_node_count + 1);
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| node.source_node.as_ref().is_some_and(|source| {
                    source.entries[0].locator.readable() == image_path.display().to_string()
                }))
        );
    }

    #[test]
    fn source_gallery_collection_node_round_trips_without_embedded_contents_or_thumbnails() {
        let temp_dir = tempfile::tempdir().unwrap();
        let image_path = temp_dir.path().join("frame.png");
        let table_path = temp_dir.path().join("polygons.geoparquet");
        std::fs::write(&image_path, b"image bytes stay outside sidecar").unwrap();
        std::fs::write(&table_path, b"polygon bytes stay outside sidecar").unwrap();
        let index = SourceGalleryIndex::from_locations(
            SourceLocator::from_location("inline-gallery"),
            vec![
                SourceLocator::from_location(&image_path.display().to_string()),
                SourceLocator::from_location(&table_path.display().to_string()),
            ],
            16,
        );
        let mut graph = GraphDocument::sample();

        let node_index = graph
            .add_source_gallery_collection_node(&index.items)
            .expect("non-empty collection should create source node");
        let source_node = graph.nodes[node_index].source_node.as_ref().unwrap();
        assert_eq!(source_node.entries.len(), 2);
        assert_eq!(
            source_node.entries[1].kind,
            SourceGalleryItemKind::PolygonTable
        );

        let json = graph.to_sidecar_json().unwrap();
        assert!(json.contains("polygons.geoparquet"));
        assert!(!json.contains("image bytes stay outside sidecar"));
        assert!(!json.contains("polygon bytes stay outside sidecar"));
        assert!(!json.contains("thumbnail"));

        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();
        let restored_source = restored
            .nodes
            .iter()
            .find_map(|node| {
                node.source_node
                    .as_ref()
                    .filter(|source| source.entries.len() == 2)
            })
            .expect("collection source should round trip");
        assert_eq!(
            restored_source.entries[0].locator.readable(),
            image_path.display().to_string()
        );
        assert_eq!(
            restored_source.entries[1].external_reference_status,
            SourceExternalReferenceStatus::LocalAvailable
        );
    }

    #[test]
    fn source_gallery_index_reports_missing_local_file_as_gallery_item() {
        let temp_dir = tempfile::tempdir().unwrap();
        let missing_path = temp_dir.path().join("missing-polygons.geoparquet");

        let index = SourceGalleryIndex::from_locator(
            SourceLocator::from_location(&missing_path.display().to_string()),
            16,
        );

        assert_eq!(index.items.len(), 1);
        let item = &index.items[0];
        assert_eq!(item.display_name, "missing-polygons.geoparquet");
        assert_eq!(item.kind, SourceGalleryItemKind::PolygonTable);
        assert_eq!(
            item.external_reference_status,
            SourceExternalReferenceStatus::LocalMissing
        );
        assert_eq!(
            item.thumbnail_intent.status(),
            SourceGalleryThumbnailStatus::MissingSource
        );
        assert_eq!(item.format_kind, Some(SourceFormatKind::GeoParquetLike));
        assert_eq!(
            item.format_support_status,
            Some(SourceFormatSupportStatus::PlannedV1)
        );
    }

    #[test]
    fn source_gallery_index_accepts_direct_urls_but_rejects_unbounded_url_listing() {
        let direct = SourceGalleryIndex::from_locator(
            SourceLocator::from_location("https://example.test/images/frame.webp?download=1"),
            16,
        );
        assert_eq!(direct.items.len(), 1);
        assert_eq!(direct.items[0].kind, SourceGalleryItemKind::Image);
        assert_eq!(
            direct.items[0].thumbnail_intent.status(),
            SourceGalleryThumbnailStatus::RemoteUnverified
        );
        assert_eq!(
            direct.items[0].external_reference_status,
            SourceExternalReferenceStatus::UriUnverified
        );

        let listing = SourceGalleryIndex::from_locator(
            SourceLocator::from_location("https://example.test/gallery/"),
            16,
        );
        assert!(listing.items.is_empty());
        assert!(
            listing
                .warnings
                .iter()
                .any(|warning| warning.contains("explicit manifest"))
        );
    }

    #[test]
    fn source_gallery_manifest_entries_create_bounded_remote_index() {
        let manifest_json = r#"
        {
            "items": [
                "https://example.test/images/frame.png",
                {
                    "location": "https://example.test/tables/polygons.geoparquet",
                    "label": "Curated polygons"
                },
                {
                    "location": "s3://bucket/run/output.rrd"
                }
            ]
        }
        "#;

        let index = SourceGalleryIndex::from_manifest_json(
            SourceLocator::from_location("https://example.test/gallery.gallery.json"),
            manifest_json,
            2,
        )
        .unwrap();

        assert_eq!(index.items.len(), 2);
        assert!(index.truncated);
        assert!(
            index
                .items
                .iter()
                .any(|item| item.kind == SourceGalleryItemKind::Image)
        );
        assert!(
            !index
                .items
                .iter()
                .any(|item| item.display_name == "output.rrd"),
            "manifest order should define which bounded entries are retained"
        );

        let polygons = index
            .items
            .iter()
            .find(|item| item.display_name == "Curated polygons")
            .expect("manifest labels should be used as display names");
        assert_eq!(polygons.kind, SourceGalleryItemKind::PolygonTable);
        match &polygons.thumbnail_intent {
            SourceGalleryThumbnailIntent::Generic(intent) => {
                assert_eq!(intent.kind, SourceGalleryItemKind::PolygonTable);
                assert_eq!(
                    intent.status,
                    SourceGalleryThumbnailStatus::RemoteUnverified
                );
            }
            SourceGalleryThumbnailIntent::Image(_) => {
                panic!("polygon table should use generic thumbnail")
            }
        }
        assert_eq!(
            polygons.external_reference_status,
            SourceExternalReferenceStatus::UriUnverified
        );
    }

    #[test]
    fn source_gallery_thumbnail_cache_is_runtime_state_not_sidecar_state() {
        let graph = GraphDocument::sample();
        let sidecar_before = graph.to_sidecar_json().unwrap();
        let mut cache = SourceGalleryThumbnailCache::default();
        cache.store_decoded(
            "local path:/tmp/frame.png",
            SourceGalleryDecodedThumbnail {
                width: 2,
                height: 1,
                rgba_bytes: vec![255, 0, 0, 255, 0, 0, 255, 255],
            },
        );
        cache.record_fetch_failure("uri:https://example.test/frame.png", "timeout");

        assert!(matches!(
            cache.get("local path:/tmp/frame.png"),
            Some(SourceGalleryThumbnailCacheState::Decoded(thumbnail))
                if thumbnail.width == 2 && thumbnail.height == 1
        ));
        assert!(matches!(
            cache.get("uri:https://example.test/frame.png"),
            Some(SourceGalleryThumbnailCacheState::FetchFailed(error))
                if error == "timeout"
        ));
        assert_eq!(graph.to_sidecar_json().unwrap(), sidecar_before);
        assert!(!sidecar_before.contains("thumbnail"));
        assert!(!sidecar_before.contains("frame.png"));
    }

    #[test]
    fn source_gallery_manifest_rejects_empty_or_invalid_lists() {
        let source = SourceLocator::from_location("https://example.test/gallery.gallery.json");

        let empty = SourceGalleryIndex::from_manifest_json(source.clone(), r#"{"items":[]}"#, 16)
            .expect_err("empty manifest should be rejected");
        assert_eq!(empty, SourceGalleryManifestError::MissingItems);

        let invalid = SourceGalleryIndex::from_manifest_json(source, r#"{"items": "#, 16)
            .expect_err("invalid manifest should be rejected");
        assert!(matches!(
            invalid,
            SourceGalleryManifestError::InvalidJson(_)
        ));
    }

    #[test]
    fn source_external_reference_report_distinguishes_generated_local_missing_and_uri_sources() {
        let graph = GraphDocument::sample();
        let demo_report = graph.source.metadata.external_reference_report();
        assert_eq!(
            demo_report.status,
            SourceExternalReferenceStatus::NotExternal
        );
        assert!(!demo_report.bundle_relevant);
        assert!(demo_report.warning.is_none());

        let local_file = tempfile::NamedTempFile::new().unwrap();
        let local_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some(local_file.path().display().to_string()),
            &graph.geometry,
            Vec::new(),
        );
        let local_report = local_metadata.external_reference_report();
        assert_eq!(
            local_report.status,
            SourceExternalReferenceStatus::LocalAvailable
        );
        assert!(local_report.bundle_relevant);
        assert!(
            local_report
                .warning
                .as_deref()
                .is_some_and(|warning| warning.contains("external reference"))
        );

        let missing_dir = tempfile::tempdir().unwrap();
        let missing_path = missing_dir.path().join("missing-source.parquet");
        let missing_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some(missing_path.display().to_string()),
            &graph.geometry,
            Vec::new(),
        );
        let missing_report = missing_metadata.external_reference_report();
        assert_eq!(
            missing_report.status,
            SourceExternalReferenceStatus::LocalMissing
        );
        assert!(missing_report.bundle_relevant);
        assert!(
            missing_report
                .warning
                .as_deref()
                .is_some_and(|warning| warning.contains("missing"))
        );

        let uri_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some("s3://bucket/curves.parquet".to_owned()),
            &graph.geometry,
            Vec::new(),
        );
        let uri_report = uri_metadata.external_reference_report();
        assert_eq!(
            uri_report.status,
            SourceExternalReferenceStatus::UriUnverified
        );
        assert!(uri_report.bundle_relevant);
        assert!(
            uri_report
                .warning
                .as_deref()
                .is_some_and(|warning| warning.contains("unverified"))
        );
    }

    #[test]
    fn recording_query_source_report_is_not_a_bundle_artifact() {
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
        let source = super::GraphSource::from_query_bridge(&bridge);
        let report = source.metadata.external_reference_report();

        assert_eq!(report.status, SourceExternalReferenceStatus::RecordingQuery);
        assert!(!report.bundle_relevant);
        assert!(
            report
                .warning
                .as_deref()
                .is_some_and(|warning| warning.contains("live viewer inputs"))
        );
    }

    #[test]
    fn source_external_reference_action_hints_cover_generated_local_missing_and_uri_sources() {
        let graph = GraphDocument::sample();
        let demo_actions = graph.source.metadata.external_reference_action_report();
        assert_eq!(
            demo_actions.recommended.kind,
            SourceExternalReferenceActionKind::InspectGeneratedSource
        );
        assert!(demo_actions.secondary.is_empty());

        let mut local_file = tempfile::Builder::new()
            .prefix("local-source")
            .suffix(".parquet")
            .tempfile()
            .unwrap();
        use std::io::Write as _;
        local_file.write_all(b"native cubic bytes").unwrap();
        local_file.flush().unwrap();
        let local_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some(local_file.path().display().to_string()),
            &graph.geometry,
            Vec::new(),
        );
        let local_actions = local_metadata.external_reference_action_report();
        assert_eq!(
            local_actions.recommended.kind,
            SourceExternalReferenceActionKind::IncludeDuringPackageExport
        );
        assert!(
            local_actions
                .recommended
                .detail
                .contains("sources/local-source")
        );
        assert!(
            local_actions
                .secondary
                .iter()
                .any(|action| action.kind == SourceExternalReferenceActionKind::RevealLocalPath)
        );
        assert!(
            local_actions
                .secondary
                .iter()
                .any(|action| action.kind == SourceExternalReferenceActionKind::CopyLocator)
        );

        let missing_dir = tempfile::tempdir().unwrap();
        let missing_path = missing_dir.path().join("missing-source.parquet");
        let missing_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some(missing_path.display().to_string()),
            &graph.geometry,
            Vec::new(),
        );
        let missing_actions = missing_metadata.external_reference_action_report();
        assert_eq!(
            missing_actions.recommended.kind,
            SourceExternalReferenceActionKind::RelinkMissingSource
        );
        assert!(
            missing_actions
                .secondary
                .iter()
                .any(|action| action.kind == SourceExternalReferenceActionKind::CopyLocator)
        );

        let uri_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some("s3://bucket/curves.parquet".to_owned()),
            &graph.geometry,
            Vec::new(),
        );
        let uri_actions = uri_metadata.external_reference_action_report();
        assert_eq!(
            uri_actions.recommended.kind,
            SourceExternalReferenceActionKind::KeepUriReference
        );
        assert!(
            uri_actions
                .recommended
                .detail
                .contains("1 external reference")
        );
        assert!(
            uri_actions
                .secondary
                .iter()
                .any(|action| action.kind == SourceExternalReferenceActionKind::CopyLocator)
        );
    }

    #[test]
    fn source_external_reference_action_hints_keep_recording_query_live() {
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
        let source = super::GraphSource::from_query_bridge(&bridge);
        let actions = source.metadata.external_reference_action_report();

        assert_eq!(
            actions.recommended.kind,
            SourceExternalReferenceActionKind::InspectLiveInput
        );
        assert!(actions.recommended.detail.contains("live inputs"));
        assert!(actions.secondary.is_empty());
    }

    #[test]
    fn source_bundle_preview_reports_inclusion_size_and_warnings_without_copying() {
        let graph = GraphDocument::sample();
        let demo_preview = graph.source.metadata.bundle_preview();
        assert_eq!(
            demo_preview.item.inclusion,
            SourceBundleInclusion::NotExternal
        );
        assert_eq!(demo_preview.expected_size_bytes, None);
        assert_eq!(demo_preview.remaining_external_reference_count, 0);
        assert_eq!(demo_preview.missing_reference_count, 0);
        assert!(demo_preview.reproducibility_warnings.is_empty());

        let mut local_file = tempfile::NamedTempFile::new().unwrap();
        use std::io::Write as _;
        local_file.write_all(b"native cubic bytes").unwrap();
        local_file.flush().unwrap();
        let local_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some(local_file.path().display().to_string()),
            &graph.geometry,
            Vec::new(),
        );
        let local_preview = local_metadata.bundle_preview();
        assert_eq!(
            local_preview.item.inclusion,
            SourceBundleInclusion::IncludeAvailable
        );
        assert_eq!(
            local_preview.expected_size_bytes,
            Some("native cubic bytes".len() as u64)
        );
        assert_eq!(local_preview.remaining_external_reference_count, 0);
        assert_eq!(local_preview.missing_reference_count, 0);
        assert!(
            local_preview
                .reproducibility_warnings
                .iter()
                .any(|warning| warning.contains("no content hash"))
        );

        let uri_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some("s3://bucket/curves.parquet".to_owned()),
            &graph.geometry,
            Vec::new(),
        );
        let uri_preview = uri_metadata.bundle_preview();
        assert_eq!(
            uri_preview.item.inclusion,
            SourceBundleInclusion::ReferenceOnly
        );
        assert_eq!(uri_preview.remaining_external_reference_count, 1);
        assert_eq!(uri_preview.missing_reference_count, 0);

        let missing_dir = tempfile::tempdir().unwrap();
        let missing_path = missing_dir.path().join("missing-source.parquet");
        let missing_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some(missing_path.display().to_string()),
            &graph.geometry,
            Vec::new(),
        );
        let missing_preview = missing_metadata.bundle_preview();
        assert_eq!(
            missing_preview.item.inclusion,
            SourceBundleInclusion::Missing
        );
        assert_eq!(missing_preview.remaining_external_reference_count, 0);
        assert_eq!(missing_preview.missing_reference_count, 1);
    }

    #[test]
    fn recording_query_source_bundle_preview_stays_live_input() {
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
        let source = super::GraphSource::from_query_bridge(&bridge);
        let preview = source.metadata.bundle_preview();

        assert_eq!(preview.item.inclusion, SourceBundleInclusion::LiveInput);
        assert_eq!(preview.remaining_external_reference_count, 0);
        assert_eq!(preview.missing_reference_count, 0);
        assert!(
            preview
                .reproducibility_warnings
                .iter()
                .any(|warning| warning.contains("live viewer inputs"))
        );
    }

    #[test]
    fn source_package_manifest_preview_reports_local_include_record_without_copying() {
        let graph = GraphDocument::sample();
        let mut local_file = tempfile::Builder::new()
            .prefix("native cubic")
            .suffix(".parquet")
            .tempfile()
            .unwrap();
        use std::io::Write as _;
        local_file.write_all(b"native cubic bytes").unwrap();
        local_file.flush().unwrap();

        let metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some(local_file.path().display().to_string()),
            &graph.geometry,
            Vec::new(),
        );
        let preview = metadata.package_manifest_preview();

        assert_eq!(preview.schema_version, 1);
        assert_eq!(preview.artifacts.len(), 1);
        assert_eq!(
            preview.expected_size_bytes,
            Some("native cubic bytes".len() as u64)
        );
        assert_eq!(preview.remaining_external_reference_count, 0);
        assert_eq!(preview.missing_reference_count, 0);
        assert!(
            preview
                .reproducibility_warnings
                .iter()
                .any(|warning| warning.contains("no content hash"))
        );

        let artifact = &preview.artifacts[0];
        assert_eq!(
            artifact.role,
            SourcePackageManifestArtifactRole::SourceDataset
        );
        assert_eq!(
            artifact.original_locator,
            local_file.path().display().to_string()
        );
        assert!(
            artifact
                .bundled_path
                .as_deref()
                .is_some_and(|path| path.starts_with("sources/native_cubic"))
        );
        assert_eq!(artifact.size_bytes, Some("native cubic bytes".len() as u64));
        assert_eq!(artifact.content_hash, None);
        assert_eq!(artifact.source_provenance, SourceProvenance::ParquetImport);
        assert_eq!(
            artifact.external_status,
            SourcePackageManifestExternalStatus::IncludedPendingWrite
        );

        let json = serde_json::to_string_pretty(&preview).unwrap();
        assert!(json.contains("\"schema_version\": 1"));
        assert!(json.contains("\"role\": \"source_dataset\""));
        assert!(json.contains("\"external_status\": \"included_pending_write\""));
        let round_trip: SourcePackageManifestPreview = serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip, preview);
    }

    #[test]
    fn source_package_manifest_writes_explicit_json_without_copying_sources() {
        let mut graph = GraphDocument::sample();
        let temp_dir = tempfile::tempdir().unwrap();
        let source_path = temp_dir.path().join("native cubic.parquet");
        std::fs::write(&source_path, b"native cubic bytes").unwrap();
        graph.source.metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some(source_path.display().to_string()),
            &graph.geometry,
            Vec::new(),
        );

        let manifest_path = temp_dir.path().join("houdini-source-package-manifest.json");
        let result = graph.save_source_package_manifest(&manifest_path).unwrap();

        assert_eq!(result.path, manifest_path);
        assert_eq!(result.artifact_count, 1);
        assert_eq!(
            result.expected_size_bytes,
            Some("native cubic bytes".len() as u64)
        );
        assert_eq!(result.remaining_external_reference_count, 0);
        assert_eq!(result.missing_reference_count, 0);
        assert_eq!(result.reproducibility_warning_count, 1);
        assert!(!temp_dir.path().join("sources").exists());

        let written_json = std::fs::read_to_string(&manifest_path).unwrap();
        let written_manifest: SourcePackageManifestPreview =
            serde_json::from_str(&written_json).unwrap();
        assert_eq!(written_manifest.schema_version, 1);
        assert_eq!(
            written_manifest.artifacts[0].external_status,
            SourcePackageManifestExternalStatus::IncludedPendingWrite
        );
        assert!(
            written_manifest.artifacts[0]
                .bundled_path
                .as_deref()
                .is_some_and(|path| path.starts_with("sources/native_cubic"))
        );
        assert_eq!(written_manifest.artifacts[0].content_hash, None);
    }

    #[test]
    fn source_package_manifest_inclusion_choice_can_leave_local_source_external() {
        let graph = GraphDocument::sample();
        let temp_dir = tempfile::tempdir().unwrap();
        let source_path = temp_dir.path().join("native cubic.parquet");
        std::fs::write(&source_path, b"native cubic bytes").unwrap();
        let metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some(source_path.display().to_string()),
            &graph.geometry,
            Vec::new(),
        );

        let reference_only = metadata.package_manifest_preview_with_choice(
            SourcePackageManifestInclusionChoice::ReferenceOnly,
        );

        assert_eq!(reference_only.expected_size_bytes, None);
        assert_eq!(reference_only.remaining_external_reference_count, 1);
        assert_eq!(reference_only.missing_reference_count, 0);
        assert_eq!(
            reference_only.artifacts[0].external_status,
            SourcePackageManifestExternalStatus::ReferenceOnly
        );
        assert_eq!(reference_only.artifacts[0].bundled_path, None);
        assert_eq!(
            reference_only.artifacts[0].size_bytes,
            Some("native cubic bytes".len() as u64)
        );
        assert!(
            reference_only
                .reproducibility_warnings
                .iter()
                .any(|warning| warning.contains("explicit package/export choice"))
        );

        let missing_path = temp_dir.path().join("missing-source.parquet");
        let missing_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some(missing_path.display().to_string()),
            &graph.geometry,
            Vec::new(),
        );
        let missing_include = missing_metadata.package_manifest_preview_with_choice(
            SourcePackageManifestInclusionChoice::IncludeAvailable,
        );
        assert_eq!(
            missing_include.artifacts[0].external_status,
            SourcePackageManifestExternalStatus::Missing
        );
        assert_eq!(missing_include.missing_reference_count, 1);
    }

    #[test]
    fn source_package_manifest_writer_honors_reference_only_choice_without_copying() {
        let mut graph = GraphDocument::sample();
        let temp_dir = tempfile::tempdir().unwrap();
        let source_path = temp_dir.path().join("native cubic.parquet");
        std::fs::write(&source_path, b"native cubic bytes").unwrap();
        graph.source.metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some(source_path.display().to_string()),
            &graph.geometry,
            Vec::new(),
        );

        let manifest_path = temp_dir.path().join("reference-only-source-manifest.json");
        let result = graph
            .save_source_package_manifest_with_choice(
                &manifest_path,
                SourcePackageManifestInclusionChoice::ReferenceOnly,
            )
            .unwrap();

        assert_eq!(result.expected_size_bytes, None);
        assert_eq!(result.remaining_external_reference_count, 1);
        assert_eq!(result.missing_reference_count, 0);
        assert!(!temp_dir.path().join("sources").exists());

        let written_json = std::fs::read_to_string(&manifest_path).unwrap();
        let written_manifest: SourcePackageManifestPreview =
            serde_json::from_str(&written_json).unwrap();
        assert_eq!(
            written_manifest.artifacts[0].external_status,
            SourcePackageManifestExternalStatus::ReferenceOnly
        );
        assert_eq!(written_manifest.artifacts[0].bundled_path, None);
    }

    #[test]
    fn source_package_manifest_preview_reports_reference_missing_generated_and_live_records() {
        let graph = GraphDocument::sample();
        let demo_preview = graph.source.metadata.package_manifest_preview();
        assert_eq!(demo_preview.artifacts.len(), 1);
        assert_eq!(
            demo_preview.artifacts[0].role,
            SourcePackageManifestArtifactRole::GeneratedSource
        );
        assert_eq!(
            demo_preview.artifacts[0].external_status,
            SourcePackageManifestExternalStatus::NotExternal
        );
        assert_eq!(demo_preview.artifacts[0].bundled_path, None);

        let uri_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some("s3://bucket/curves.parquet".to_owned()),
            &graph.geometry,
            Vec::new(),
        );
        let uri_preview = uri_metadata.package_manifest_preview();
        assert_eq!(
            uri_preview.artifacts[0].role,
            SourcePackageManifestArtifactRole::SourceDataset
        );
        assert_eq!(
            uri_preview.artifacts[0].external_status,
            SourcePackageManifestExternalStatus::ReferenceOnly
        );
        assert_eq!(uri_preview.artifacts[0].bundled_path, None);
        assert_eq!(uri_preview.remaining_external_reference_count, 1);
        assert!(uri_preview.reproducibility_warnings[0].contains("unverified"));

        let missing_dir = tempfile::tempdir().unwrap();
        let missing_path = missing_dir.path().join("missing-source.parquet");
        let missing_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some(missing_path.display().to_string()),
            &graph.geometry,
            Vec::new(),
        );
        let missing_preview = missing_metadata.package_manifest_preview();
        assert_eq!(
            missing_preview.artifacts[0].external_status,
            SourcePackageManifestExternalStatus::Missing
        );
        assert_eq!(missing_preview.missing_reference_count, 1);
        assert!(
            missing_preview
                .reproducibility_warnings
                .iter()
                .any(|warning| warning.contains("missing"))
        );

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
        let source = super::GraphSource::from_query_bridge(&bridge);
        let live_preview = source.metadata.package_manifest_preview();
        assert_eq!(
            live_preview.artifacts[0].role,
            SourcePackageManifestArtifactRole::LiveRecordingQuery
        );
        assert_eq!(
            live_preview.artifacts[0].external_status,
            SourcePackageManifestExternalStatus::LiveInput
        );
        assert_eq!(live_preview.artifacts[0].bundled_path, None);
    }

    #[test]
    fn source_format_capabilities_record_adr_0015_statuses() {
        let graph = GraphDocument::sample();
        let capabilities = graph.source_format_capabilities();

        let parquet = capabilities
            .iter()
            .find(|capability| capability.kind == SourceFormatKind::Parquet)
            .expect("Parquet capability should be present");
        assert_eq!(parquet.status, SourceFormatSupportStatus::Supported);
        assert_eq!(
            parquet.geometry_kinds,
            vec![HoudiniGeometryKind::CubicBezier]
        );
        assert!(parquet.notes.contains("eight control-point columns"));

        let planned =
            graph.source_format_capabilities_with_status(SourceFormatSupportStatus::PlannedV1);
        assert!(
            planned
                .iter()
                .any(|capability| capability.kind == SourceFormatKind::GeoJson)
        );
        assert!(
            planned
                .iter()
                .any(|capability| capability.kind == SourceFormatKind::FlatGeobuf)
        );
        assert!(
            planned
                .iter()
                .any(|capability| capability.kind == SourceFormatKind::CsvCoordinates)
        );
        assert!(
            planned
                .iter()
                .any(|capability| capability.kind == SourceFormatKind::LasLazPointCloud)
        );
        assert!(
            planned
                .iter()
                .any(|capability| capability.kind == SourceFormatKind::SqliteTableOrView)
        );

        let later = graph
            .source_format_capabilities_with_status(SourceFormatSupportStatus::LaterCompatibility);
        assert_eq!(later.len(), 2);
        assert!(
            later
                .iter()
                .any(|capability| capability.kind == SourceFormatKind::GeoPackage)
        );
        assert!(
            later
                .iter()
                .any(|capability| capability.kind == SourceFormatKind::SpatiaLite)
        );

        let deferred =
            graph.source_format_capabilities_with_status(SourceFormatSupportStatus::Deferred);
        assert_eq!(deferred.len(), 1);
        assert_eq!(deferred[0].kind, SourceFormatKind::Shapefile);
        assert!(deferred[0].notes.contains("CRS expectations"));
    }

    #[test]
    fn source_format_inference_reports_capabilities_from_locator_extensions() {
        let graph = GraphDocument::sample();
        let cases = [
            (
                "/tmp/curves.parquet",
                SourceFormatKind::Parquet,
                SourceFormatSupportStatus::Supported,
            ),
            (
                "s3://bucket/curves.geoparquet?version=1",
                SourceFormatKind::GeoParquetLike,
                SourceFormatSupportStatus::PlannedV1,
            ),
            (
                "https://example.test/features.geojson#latest",
                SourceFormatKind::GeoJson,
                SourceFormatSupportStatus::PlannedV1,
            ),
            (
                "/tmp/features.fgb",
                SourceFormatKind::FlatGeobuf,
                SourceFormatSupportStatus::PlannedV1,
            ),
            (
                "/tmp/points.csv",
                SourceFormatKind::CsvCoordinates,
                SourceFormatSupportStatus::PlannedV1,
            ),
            (
                "/tmp/points.laz",
                SourceFormatKind::LasLazPointCloud,
                SourceFormatSupportStatus::PlannedV1,
            ),
            (
                "/tmp/source.sqlite3",
                SourceFormatKind::SqliteTableOrView,
                SourceFormatSupportStatus::PlannedV1,
            ),
            (
                "/tmp/source.gpkg",
                SourceFormatKind::GeoPackage,
                SourceFormatSupportStatus::LaterCompatibility,
            ),
            (
                "/tmp/source.spatialite",
                SourceFormatKind::SpatiaLite,
                SourceFormatSupportStatus::LaterCompatibility,
            ),
            (
                "/tmp/source.shp",
                SourceFormatKind::Shapefile,
                SourceFormatSupportStatus::Deferred,
            ),
        ];

        for (locator, expected_kind, expected_status) in cases {
            let metadata = super::SourceMetadata::from_geometry(
                SourceProvenance::ParquetImport,
                Some(locator.to_owned()),
                &graph.geometry,
                Vec::new(),
            );
            let report = metadata.source_format_inference_report();

            assert_eq!(report.status, SourceFormatInferenceStatus::Inferred);
            assert_eq!(report.readable_locator, locator);
            assert_eq!(report.kind, Some(expected_kind));
            assert_eq!(report.support_status, Some(expected_status));
            assert!(!report.notes.is_empty());
        }
    }

    #[test]
    fn source_format_inference_reports_unknown_generated_and_live_sources() {
        let graph = GraphDocument::sample();
        let demo_report = graph.source.metadata.source_format_inference_report();
        assert_eq!(demo_report.status, SourceFormatInferenceStatus::Generated);
        assert_eq!(demo_report.kind, None);
        assert_eq!(demo_report.support_status, None);

        let unknown_metadata = super::SourceMetadata::from_geometry(
            SourceProvenance::ParquetImport,
            Some("/tmp/source.unknown".to_owned()),
            &graph.geometry,
            Vec::new(),
        );
        let unknown_report = unknown_metadata.source_format_inference_report();
        assert_eq!(unknown_report.status, SourceFormatInferenceStatus::Unknown);
        assert_eq!(unknown_report.kind, None);
        assert_eq!(unknown_report.support_status, None);
        assert!(unknown_report.notes.contains("No known v1 source format"));

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
        let source = super::GraphSource::from_query_bridge(&bridge);
        let live_report = source.metadata.source_format_inference_report();
        assert_eq!(live_report.status, SourceFormatInferenceStatus::LiveInput);
        assert_eq!(live_report.kind, None);
        assert_eq!(live_report.support_status, None);
        assert!(live_report.notes.contains("live viewer inputs"));
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
        assert_eq!(recording.substrate_raster_count, 0);
        assert!(
            recording
                .limitation_note
                .contains("cubic Bezier semantics as graph-owned control-point metadata")
        );
        assert!(recording_path.exists());
        assert!(std::fs::metadata(&recording_path).unwrap().len() > 0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn malware_starter_recording_writes_substrate_raster_with_overlays() {
        let recording_dir = tempfile::tempdir().unwrap();
        let recording_path = recording_dir.path().join("malware-byteplot-output.rrd");
        let graph = GraphDocument::malware_starter();

        let recording = graph.save_rerun_recording(&recording_path).unwrap();

        assert_eq!(recording.path, recording_path);
        assert_eq!(recording.item_count, 3);
        assert_eq!(recording.polygon_count, 3);
        assert_eq!(recording.native_cubic_bezier_count, 0);
        assert_eq!(recording.substrate_raster_count, 1);
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
        assert_eq!(
            restored.source.metadata.locator.kind,
            SourceLocatorKind::LocalPath
        );
        assert_eq!(
            restored.source.metadata.locator.readable(),
            sample_path.display().to_string()
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
    fn legacy_sidecar_without_source_locator_infers_from_source_path() {
        let mut graph = GraphDocument::sample();
        let sample_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/houdini_cubic_sample.parquet");
        graph
            .import_cubic_bezier_parquet_path(&sample_path)
            .unwrap();

        let mut value: serde_json::Value =
            serde_json::from_str(&graph.to_sidecar_json().unwrap()).unwrap();
        value["source"]["metadata"]
            .as_object_mut()
            .expect("source metadata should be an object")
            .remove("locator");
        let legacy_json = serde_json::to_string(&value).unwrap();

        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&legacy_json).unwrap();

        assert_eq!(
            restored.source.metadata.locator.kind,
            SourceLocatorKind::LocalPath
        );
        assert_eq!(
            restored.source.metadata.locator.readable(),
            sample_path.display().to_string()
        );
        assert!(restored.source.metadata.locator.is_external_reference());
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
    fn matching_asset_definition_relocks_without_upgrading_pinned_version() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        let node_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");
        graph.procedural_asset_declarations[0].version = "0.2.0".to_owned();
        graph.refresh_asset_version_statuses();
        assert!(graph.set_procedural_asset_contents_unlocked(node_index, true));

        let unlocked_info = graph
            .selected_node_info(node_index)
            .expect("asset info should exist")
            .procedural_asset
            .expect("asset node info should exist");
        assert!(unlocked_info.can_match_definition);
        assert!(unlocked_info.can_upgrade_to_current_definition);

        assert!(graph.match_procedural_asset_definition(node_index));
        graph.refresh_asset_version_statuses();

        let info = graph
            .selected_node_info(node_index)
            .expect("asset info should exist");
        let asset = info.procedural_asset.expect("asset node info should exist");
        assert!(!asset.contents_unlocked);
        assert!(!asset.can_match_definition);
        assert!(asset.can_upgrade_to_current_definition);
        assert_eq!(asset.instance_version, "0.1.0");
        assert_eq!(asset.current_version.as_deref(), Some("0.2.0"));
        assert_eq!(asset.version_status, OperatorVersionStatus::NewerAvailable);
        assert_eq!(
            info.evaluation.message.as_deref(),
            Some("Asset declaration version changed after this instance was created.")
        );
    }

    #[test]
    fn upgrading_asset_instance_explicitly_changes_pinned_version() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        let node_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");
        graph.procedural_asset_declarations[0].version = "0.2.0".to_owned();
        graph.refresh_asset_version_statuses();
        assert!(graph.set_procedural_asset_contents_unlocked(node_index, true));

        assert!(graph.upgrade_procedural_asset_to_current_definition(node_index));

        let info = graph
            .selected_node_info(node_index)
            .expect("asset info should exist");
        let asset = info.procedural_asset.expect("asset node info should exist");
        assert_eq!(asset.instance_version, "0.2.0");
        assert_eq!(asset.current_version.as_deref(), Some("0.2.0"));
        assert_eq!(asset.version_status, OperatorVersionStatus::Current);
        assert!(!asset.contents_unlocked);
        assert!(!asset.can_match_definition);
        assert!(!asset.can_upgrade_to_current_definition);
        assert_eq!(info.status, NodeStatus::Healthy);
        assert_eq!(
            info.evaluation.message.as_deref(),
            Some("Asset instance upgraded to definition version 0.2.0.")
        );

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();
        let restored_asset = restored.nodes[node_index]
            .procedural_asset
            .as_ref()
            .expect("asset instance should restore");

        assert_eq!(restored_asset.instance_version, "0.2.0");
        assert!(!restored_asset.contents_unlocked);
    }

    #[test]
    fn procedural_asset_save_definition_relocks_source_instance_and_preserves_other_pins() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        let saved_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");
        let sibling_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");
        let previous_digest = graph.procedural_asset_declarations[0]
            .source
            .source_digest
            .clone();
        assert!(graph.set_procedural_asset_contents_unlocked(saved_index, true));

        let unlocked_info = graph
            .selected_node_info(saved_index)
            .expect("asset info should exist")
            .procedural_asset
            .expect("asset node info should exist");
        assert!(unlocked_info.can_save_definition);

        let result = graph
            .save_procedural_asset_definition(saved_index)
            .expect("unlocked asset should save its definition");

        assert_eq!(result.asset_id, "vy.asset.curve_cleanup");
        assert_eq!(result.previous_version, "0.1.0");
        assert_eq!(result.new_version, "0.1.1");
        assert_eq!(result.update_available_instance_count, 1);
        assert_eq!(graph.procedural_asset_declarations[0].version, "0.1.1");
        assert_ne!(
            graph.procedural_asset_declarations[0].source.source_digest,
            previous_digest
        );
        assert_eq!(
            graph.procedural_asset_declarations[0]
                .wrapped_subgraph
                .graph_snapshot
                .as_ref()
                .expect("saved definition should keep a graph snapshot")
                .node_count,
            graph.nodes.len()
        );

        let saved_info = graph
            .selected_node_info(saved_index)
            .expect("saved asset info should exist")
            .procedural_asset
            .expect("saved asset node info should exist");
        assert_eq!(saved_info.instance_version, "0.1.1");
        assert_eq!(saved_info.current_version.as_deref(), Some("0.1.1"));
        assert_eq!(saved_info.version_status, OperatorVersionStatus::Current);
        assert!(!saved_info.contents_unlocked);
        assert!(!saved_info.can_save_definition);

        let sibling_info = graph
            .selected_node_info(sibling_index)
            .expect("sibling asset info should exist")
            .procedural_asset
            .expect("sibling asset node info should exist");
        assert_eq!(sibling_info.instance_version, "0.1.0");
        assert_eq!(sibling_info.current_version.as_deref(), Some("0.1.1"));
        assert_eq!(
            sibling_info.version_status,
            OperatorVersionStatus::NewerAvailable
        );
        assert!(sibling_info.can_upgrade_to_current_definition);
    }

    #[test]
    fn asset_instance_actions_reject_missing_or_current_definitions() {
        let mut graph = GraphDocument::sample();
        let missing_index = graph.add_procedural_asset_node("vy.asset.missing");
        assert!(!graph.match_procedural_asset_definition(missing_index));
        assert!(!graph.upgrade_procedural_asset_to_current_definition(missing_index));
        assert!(
            graph
                .save_procedural_asset_definition(missing_index)
                .is_none()
        );

        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        let current_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");
        assert!(!graph.match_procedural_asset_definition(current_index));
        assert!(!graph.upgrade_procedural_asset_to_current_definition(current_index));
        assert!(
            graph
                .save_procedural_asset_definition(current_index)
                .is_none()
        );
    }

    #[test]
    fn procedural_asset_boundary_model_actions_edit_typed_ports() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        let node_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");

        assert!(graph.add_procedural_asset_boundary_port(
            "vy.asset.curve_cleanup",
            ProceduralAssetBoundaryDirection::Input,
            HoudiniOperatorPort {
                name: "  threshold  ".to_owned(),
                data_kind: HoudiniDataKind::Scalar,
                required: false,
                help: " Optional score threshold. ".to_owned(),
            },
        ));
        assert!(graph.replace_procedural_asset_boundary_port(
            "vy.asset.curve_cleanup",
            ProceduralAssetBoundaryDirection::Output,
            "geometry",
            HoudiniOperatorPort {
                name: "clean_curves".to_owned(),
                data_kind: HoudiniDataKind::GeometryTable,
                required: true,
                help: "Clean native cubic Bezier and polygon geometry.".to_owned(),
            },
        ));
        assert!(graph.remove_procedural_asset_boundary_port(
            "vy.asset.curve_cleanup",
            ProceduralAssetBoundaryDirection::Input,
            "threshold",
        ));

        let declaration = &graph.procedural_asset_declarations[0];
        assert_eq!(
            declaration
                .inputs
                .iter()
                .map(|port| port.name.as_str())
                .collect::<Vec<_>>(),
            vec!["geometry"]
        );
        assert_eq!(
            declaration
                .outputs
                .iter()
                .map(|port| port.name.as_str())
                .collect::<Vec<_>>(),
            vec!["clean_curves"]
        );
        assert!(
            declaration.outputs[0]
                .data_kind
                .preserves_native_cubic_bezier()
        );

        let info = graph
            .selected_node_info(node_index)
            .expect("asset info should exist");
        assert_eq!(info.input_count, 1);
        assert_eq!(info.output_count, 1);
        assert_eq!(info.evaluation.state, EvaluationState::Stale);
        assert_eq!(
            info.evaluation.message.as_deref(),
            Some("Asset input boundary changed; review instance bindings before running.")
        );
    }

    #[test]
    fn procedural_asset_boundary_actions_reject_ambiguous_edits() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());

        assert!(!graph.add_procedural_asset_boundary_port(
            "vy.asset.curve_cleanup",
            ProceduralAssetBoundaryDirection::Input,
            geometry_port("geometry", "Duplicate input."),
        ));
        assert!(!graph.add_procedural_asset_boundary_port(
            "vy.asset.curve_cleanup",
            ProceduralAssetBoundaryDirection::Output,
            geometry_port(" ", "Missing stable port name."),
        ));
        assert!(!graph.replace_procedural_asset_boundary_port(
            "vy.asset.curve_cleanup",
            ProceduralAssetBoundaryDirection::Output,
            "missing",
            geometry_port("renamed", "Missing original port."),
        ));
        assert!(!graph.remove_procedural_asset_boundary_port(
            "vy.asset.curve_cleanup",
            ProceduralAssetBoundaryDirection::Output,
            "geometry",
        ));

        assert!(graph.add_procedural_asset_boundary_port(
            "vy.asset.curve_cleanup",
            ProceduralAssetBoundaryDirection::Output,
            HoudiniOperatorPort {
                name: "attributes".to_owned(),
                data_kind: HoudiniDataKind::AttributeTable,
                required: false,
                help: "Optional attribute table output.".to_owned(),
            },
        ));
        assert!(!graph.replace_procedural_asset_boundary_port(
            "vy.asset.curve_cleanup",
            ProceduralAssetBoundaryDirection::Output,
            "attributes",
            geometry_port("geometry", "Would duplicate the existing output."),
        ));
        assert!(graph.remove_procedural_asset_boundary_port(
            "vy.asset.curve_cleanup",
            ProceduralAssetBoundaryDirection::Output,
            "attributes",
        ));
        assert!(!graph.add_procedural_asset_boundary_port(
            "vy.asset.missing",
            ProceduralAssetBoundaryDirection::Input,
            geometry_port("new_input", "Missing declaration."),
        ));

        let declaration = &graph.procedural_asset_declarations[0];
        assert_eq!(
            declaration
                .inputs
                .iter()
                .map(|port| port.name.as_str())
                .collect::<Vec<_>>(),
            vec!["geometry"]
        );
        assert_eq!(
            declaration
                .outputs
                .iter()
                .map(|port| port.name.as_str())
                .collect::<Vec<_>>(),
            vec!["geometry"]
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
        let minimum_score = draft
            .promoted_parameters
            .iter()
            .find(|parameter| parameter.name == "minimum_score")
            .expect("minimum score should be promoted");
        assert_eq!(minimum_score.label.as_deref(), Some("Minimum score"));
        assert_eq!(
            minimum_score.current_value,
            Some(HoudiniParameterValue::Float(0.55))
        );
        assert_eq!(minimum_score.group.as_deref(), Some("Filter"));
        assert_eq!(
            minimum_score.binding.as_ref().map(|binding| (
                binding.internal_node_id.as_str(),
                binding.internal_parameter_name.as_str()
            )),
            Some(("filter.main", "score_threshold"))
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
        assert!(declaration.external_artifacts.is_empty());
        assert!(!graph.to_sidecar_json().unwrap().contains("cached_output"));
    }

    #[test]
    fn procedural_asset_drafts_get_collision_safe_project_ids() {
        let mut graph = GraphDocument::sample();
        let first_draft =
            graph.create_asset_draft_from_graph("My Cleanup Asset", "First.", "First.");
        let first_asset_id = graph.commit_asset_draft(first_draft);

        let second_draft =
            graph.create_asset_draft_from_graph("My Cleanup Asset", "Second.", "Second.");
        let second_asset_id = graph.commit_asset_draft(second_draft);

        assert_eq!(first_asset_id, "project.asset.my_cleanup_asset");
        assert_eq!(second_asset_id, "project.asset.my_cleanup_asset_2");
        assert_eq!(graph.procedural_asset_declarations.len(), 2);
        assert_eq!(
            graph.procedural_asset_declarations[1]
                .wrapped_subgraph
                .graph_id,
            "project.asset.my_cleanup_asset_2.graph"
        );
    }

    #[test]
    fn procedural_asset_draft_from_graph_container_uses_container_boundary() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "filter.main")
            .expect("sample graph should include filter");
        let style_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "style.main")
            .expect("sample graph should include style");
        let container_index = graph
            .add_graph_container_collapse_manifest_for_node_set(
                "Cleanup Subnet",
                &[filter_index, style_index],
            )
            .expect("connected selection should collapse");

        let draft = graph
            .create_asset_draft_from_graph_container(
                container_index,
                "Cleanup Asset",
                "Promoted from a graph container.",
                "Use as a reusable cleanup asset.",
            )
            .expect("resolved graph container should create an asset draft");

        assert_eq!(draft.asset_id, "project.asset.cleanup_asset");
        assert_eq!(draft.inputs.len(), 1);
        assert_eq!(draft.outputs.len(), 1);
        assert_eq!(draft.inputs[0].name, PRIMARY_GEOMETRY_OUTPUT);
        assert_eq!(draft.outputs[0].data_kind, HoudiniDataKind::GeometryTable);
        assert_eq!(draft.wrapped_subgraph.graph_id, "graph.cleanup_subnet");
        assert_eq!(draft.wrapped_subgraph.output_node_id, "style.main");
        assert!(draft.wrapped_subgraph.captures_native_cubic_bezier);
        assert_eq!(draft.graph_snapshot.node_count, 2);
        assert_eq!(draft.graph_snapshot.edge_count, 1);
        assert!(
            draft
                .promoted_parameters
                .iter()
                .any(|parameter| parameter.name == "minimum_score")
        );
        assert!(
            draft
                .promoted_parameters
                .iter()
                .any(|parameter| parameter.name == "stroke_scale")
        );

        let asset_id = graph.commit_asset_draft(draft);
        let declaration = graph
            .procedural_asset_declarations
            .iter()
            .find(|declaration| declaration.asset_id == asset_id)
            .expect("asset declaration should be committed");

        assert_eq!(declaration.display_name, "Cleanup Asset");
        assert_eq!(declaration.inputs[0].name, PRIMARY_GEOMETRY_OUTPUT);
        assert_eq!(
            declaration.wrapped_subgraph.graph_id,
            "graph.cleanup_subnet"
        );
        assert_eq!(declaration.wrapped_subgraph.output_node_id, "style.main");
        assert_eq!(
            declaration
                .wrapped_subgraph
                .graph_snapshot
                .as_ref()
                .expect("asset declaration should retain graph snapshot")
                .node_count,
            2
        );
    }

    #[test]
    fn procedural_asset_draft_from_graph_container_falls_back_to_subnet_name() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "filter.main")
            .expect("sample graph should include filter");
        let style_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "style.main")
            .expect("sample graph should include style");
        let container_index = graph
            .add_graph_container_collapse_manifest_for_node_set(
                "Cleanup Subnet",
                &[filter_index, style_index],
            )
            .expect("connected selection should collapse");

        let draft = graph
            .create_asset_draft_from_graph_container(
                container_index,
                " ",
                "Promoted from a graph container.",
                "Use as a reusable cleanup asset.",
            )
            .expect("resolved graph container should create an asset draft");

        assert_eq!(draft.display_name, "Cleanup Subnet");
        assert_eq!(draft.asset_id, "project.asset.cleanup_subnet");
    }

    #[test]
    fn procedural_asset_draft_from_graph_container_rejects_unresolved_targets() {
        let mut graph = GraphDocument::sample();
        assert_eq!(
            graph.create_asset_draft_from_graph_container(
                usize::MAX,
                "Missing",
                "Missing node.",
                "Missing node.",
            ),
            Err(GraphContainerAssetDraftError::MissingNodeIndex(usize::MAX))
        );
        assert_eq!(
            graph.create_asset_draft_from_graph_container(
                0,
                "Source",
                "Not a graph container.",
                "Not a graph container.",
            ),
            Err(GraphContainerAssetDraftError::NotGraphContainer)
        );

        let container_index = graph.add_graph_container_node(
            "Broken Subnet",
            ProjectGraphMetadata {
                graph_id: "graph.broken_asset".to_owned(),
                name: "Broken Asset".to_owned(),
                path: "/obj/main/broken_asset".to_owned(),
                role: ProjectGraphRole::Subgraph,
            },
        );
        graph.graph_containers.clear();
        assert_eq!(
            graph.create_asset_draft_from_graph_container(
                container_index,
                "Broken",
                "Missing metadata.",
                "Missing metadata.",
            ),
            Err(GraphContainerAssetDraftError::MissingContainerMetadata)
        );

        let container_node_id = graph.nodes[container_index].node_id.clone();
        graph.graph_containers.push(GraphContainerMetadata {
            container_node_id,
            internal_graph_id: "graph.missing_internal".to_owned(),
            kind: GraphContainerKind::Subnet,
            boundary: GraphBoundaryDeclaration::geometry_passthrough(),
            collapse_manifest: None,
            navigable: true,
        });
        assert_eq!(
            graph.create_asset_draft_from_graph_container(
                container_index,
                "Broken",
                "Missing graph.",
                "Missing graph.",
            ),
            Err(GraphContainerAssetDraftError::MissingInternalGraph)
        );
    }

    #[test]
    fn procedural_asset_external_artifacts_round_trip_and_warn() {
        let mut graph = GraphDocument::sample();
        let mut declaration = sample_procedural_asset_declaration();
        declaration
            .external_artifacts
            .push(ProceduralAssetArtifactReference {
                role: ProceduralAssetArtifactRole::Dataset,
                locator: "data/training/curves.parquet".to_owned(),
                source_node_id: Some("source.main".to_owned()),
                source_node_name: Some("Source".to_owned()),
                size_bytes: Some(42_000_000),
                content_hash: Some("sha256:curves".to_owned()),
                status: ProceduralAssetArtifactStatus::Referenced,
            });
        declaration
            .external_artifacts
            .push(ProceduralAssetArtifactReference {
                role: ProceduralAssetArtifactRole::ModelWeights,
                locator: "models/cleanup.safetensors".to_owned(),
                source_node_id: None,
                source_node_name: None,
                size_bytes: None,
                content_hash: None,
                status: ProceduralAssetArtifactStatus::Missing,
            });
        declaration
            .external_artifacts
            .push(ProceduralAssetArtifactReference {
                role: ProceduralAssetArtifactRole::Recording,
                locator: "bundles/previews/cleanup.rrd".to_owned(),
                source_node_id: None,
                source_node_name: None,
                size_bytes: Some(4096),
                content_hash: Some("sha256:preview".to_owned()),
                status: ProceduralAssetArtifactStatus::Bundled,
            });
        graph.procedural_asset_declarations.push(declaration);
        let node_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");

        let info = graph
            .selected_node_info(node_index)
            .expect("asset info should exist");
        let asset = info.procedural_asset.expect("asset node info should exist");

        assert_eq!(asset.external_artifact_warnings.len(), 2);
        assert!(asset.external_artifact_warnings[0].contains("external reference"));
        assert!(asset.external_artifact_warnings[1].contains("missing"));
        assert_eq!(info.warnings.len(), 2);

        let json = graph.to_sidecar_json().unwrap();
        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert!(json.contains("external_artifacts"));
        assert_eq!(
            restored.procedural_asset_declarations[0].external_artifacts,
            graph.procedural_asset_declarations[0].external_artifacts
        );
    }

    #[test]
    fn procedural_asset_bundle_preview_reports_inclusion_metadata_without_copying() {
        let mut graph = GraphDocument::sample();
        let mut declaration = sample_procedural_asset_declaration();
        declaration
            .external_artifacts
            .push(ProceduralAssetArtifactReference {
                role: ProceduralAssetArtifactRole::Dataset,
                locator: "data/training/curves.parquet".to_owned(),
                source_node_id: Some("source.main".to_owned()),
                source_node_name: Some("Source".to_owned()),
                size_bytes: Some(42_000_000),
                content_hash: Some("sha256:curves".to_owned()),
                status: ProceduralAssetArtifactStatus::Referenced,
            });
        declaration
            .external_artifacts
            .push(ProceduralAssetArtifactReference {
                role: ProceduralAssetArtifactRole::ModelWeights,
                locator: "models/cleanup.safetensors".to_owned(),
                source_node_id: Some("python.cleanup".to_owned()),
                source_node_name: Some("Cleanup model".to_owned()),
                size_bytes: None,
                content_hash: None,
                status: ProceduralAssetArtifactStatus::Referenced,
            });
        declaration
            .external_artifacts
            .push(ProceduralAssetArtifactReference {
                role: ProceduralAssetArtifactRole::AnalysisFile,
                locator: "analysis/missing.json".to_owned(),
                source_node_id: None,
                source_node_name: None,
                size_bytes: None,
                content_hash: None,
                status: ProceduralAssetArtifactStatus::Missing,
            });
        declaration
            .external_artifacts
            .push(ProceduralAssetArtifactReference {
                role: ProceduralAssetArtifactRole::Recording,
                locator: "bundles/previews/cleanup.rrd".to_owned(),
                source_node_id: None,
                source_node_name: None,
                size_bytes: Some(4096),
                content_hash: Some("sha256:preview".to_owned()),
                status: ProceduralAssetArtifactStatus::Bundled,
            });
        graph.procedural_asset_declarations.push(declaration);
        let before_json = graph.to_sidecar_json().unwrap();

        let preview = graph
            .procedural_asset_bundle_preview(
                "vy.asset.curve_cleanup",
                &[
                    ProceduralAssetArtifactInclusionChoice {
                        locator: "data/training/curves.parquet".to_owned(),
                        include: true,
                        bundled_path: Some("bundle/data/curves.parquet".to_owned()),
                    },
                    ProceduralAssetArtifactInclusionChoice {
                        locator: "models/cleanup.safetensors".to_owned(),
                        include: false,
                        bundled_path: None,
                    },
                ],
            )
            .expect("asset preview should exist");

        assert_eq!(preview.asset_id, "vy.asset.curve_cleanup");
        assert_eq!(preview.artifacts.len(), 4);
        assert_eq!(preview.dependency_requirements.len(), 4);
        assert!(preview.dependency_requirements.iter().any(
            |requirement| requirement == "Model weights artifact `models/cleanup.safetensors`"
        ));
        assert_eq!(preview.included_file_count, 2);
        assert_eq!(preview.expected_included_size_bytes, 42_004_096);
        assert_eq!(preview.unknown_included_size_count, 0);
        assert_eq!(preview.remaining_external_reference_count, 1);
        assert_eq!(preview.missing_artifact_count, 1);
        assert_eq!(
            preview.artifacts[0].inclusion,
            ProceduralAssetArtifactBundleInclusion::Include
        );
        assert_eq!(
            preview.artifacts[0].bundled_path.as_deref(),
            Some("bundle/data/curves.parquet")
        );
        assert_eq!(
            preview.artifacts[1].inclusion,
            ProceduralAssetArtifactBundleInclusion::ReferenceOnly
        );
        assert_eq!(
            preview.artifacts[2].inclusion,
            ProceduralAssetArtifactBundleInclusion::Missing
        );
        assert_eq!(
            preview.artifacts[3].inclusion,
            ProceduralAssetArtifactBundleInclusion::AlreadyBundled
        );
        assert!(
            preview
                .reproducibility_warnings
                .iter()
                .any(|warning| warning.contains("remains an external reference"))
        );
        assert!(
            preview
                .reproducibility_warnings
                .iter()
                .any(|warning| warning.contains("missing"))
        );
        assert_eq!(graph.to_sidecar_json().unwrap(), before_json);
        assert!(
            graph
                .procedural_asset_bundle_preview("vy.asset.missing", &[])
                .is_none()
        );
    }

    #[test]
    fn procedural_asset_bundle_preview_warns_on_included_unknowns() {
        let mut graph = GraphDocument::sample();
        let mut declaration = sample_procedural_asset_declaration();
        declaration
            .external_artifacts
            .push(ProceduralAssetArtifactReference {
                role: ProceduralAssetArtifactRole::ModelWeights,
                locator: "models/cleanup.safetensors".to_owned(),
                source_node_id: None,
                source_node_name: None,
                size_bytes: None,
                content_hash: None,
                status: ProceduralAssetArtifactStatus::Referenced,
            });
        graph.procedural_asset_declarations.push(declaration);

        let preview = graph
            .procedural_asset_bundle_preview(
                "vy.asset.curve_cleanup",
                &[ProceduralAssetArtifactInclusionChoice {
                    locator: "models/cleanup.safetensors".to_owned(),
                    include: true,
                    bundled_path: None,
                }],
            )
            .expect("asset preview should exist");

        assert_eq!(preview.included_file_count, 1);
        assert_eq!(preview.expected_included_size_bytes, 0);
        assert_eq!(preview.unknown_included_size_count, 1);
        assert_eq!(
            preview.artifacts[0].bundled_path.as_deref(),
            Some("bundles/assets/vy_asset_curve_cleanup/artifacts/cleanup.safetensors")
        );
        assert!(
            preview
                .reproducibility_warnings
                .iter()
                .any(|warning| warning.contains("unknown size"))
        );
        assert!(
            preview
                .reproducibility_warnings
                .iter()
                .any(|warning| warning.contains("no content hash"))
        );
    }

    #[test]
    fn legacy_asset_declaration_without_external_artifacts_loads_empty() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        let mut value =
            serde_json::from_str::<serde_json::Value>(&graph.to_sidecar_json().unwrap())
                .expect("sidecar should be valid json");
        value["procedural_asset_declarations"][0]
            .as_object_mut()
            .expect("asset declaration should be an object")
            .remove("external_artifacts");
        let json = serde_json::to_string_pretty(&value).unwrap();

        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();

        assert!(
            restored.procedural_asset_declarations[0]
                .external_artifacts
                .is_empty()
        );
    }

    #[test]
    fn legacy_asset_promoted_parameters_load_without_authoring_metadata() {
        let mut graph = GraphDocument::sample();
        let draft = graph.create_asset_draft_from_graph(
            "Legacy Cleanup Asset",
            "Cleans the current graph.",
            "Use inside this project.",
        );
        graph.commit_asset_draft(draft);
        let mut value =
            serde_json::from_str::<serde_json::Value>(&graph.to_sidecar_json().unwrap())
                .expect("sidecar should be valid json");
        let parameter = value["procedural_asset_declarations"][0]["promoted_parameters"][0]
            .as_object_mut()
            .expect("promoted parameter should be an object");
        parameter.remove("label");
        parameter.remove("current_value");
        parameter.remove("group");
        parameter.remove("binding");
        let json = serde_json::to_string_pretty(&value).unwrap();

        let mut restored = GraphDocument::sample();
        restored.apply_sidecar_json(&json).unwrap();
        let parameter = &restored.procedural_asset_declarations[0].promoted_parameters[0];

        assert_eq!(parameter.name, "minimum_score");
        assert!(parameter.label.is_none());
        assert!(parameter.current_value.is_none());
        assert!(parameter.group.is_none());
        assert!(parameter.binding.is_none());
    }

    #[test]
    fn create_asset_instance_from_graph_places_visible_asset_node() {
        let mut graph = GraphDocument::sample();
        let initial_node_count = graph.nodes.len();

        let (asset_id, node_index) = graph.create_asset_instance_from_graph(
            "Shelf Cleanup Asset",
            "Created from the built-in shelf.",
            "Use from the shelf.",
        );

        assert_eq!(asset_id, "project.asset.shelf_cleanup_asset");
        assert_eq!(graph.nodes.len(), initial_node_count + 1);
        assert_eq!(graph.nodes[node_index].kind, NodeKind::ProceduralAsset);
        assert_eq!(
            graph.nodes[node_index]
                .procedural_asset
                .as_ref()
                .map(|asset| asset.asset_id.as_str()),
            Some("project.asset.shelf_cleanup_asset")
        );
        assert!(
            graph
                .procedural_asset_declarations
                .iter()
                .any(|declaration| declaration.asset_id == asset_id)
        );
    }

    #[test]
    fn procedural_asset_gallery_entries_report_usages_across_graphs() {
        let mut graph = GraphDocument::sample();
        graph
            .procedural_asset_declarations
            .push(sample_procedural_asset_declaration());
        let main_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");
        graph.graph_registry.graphs.push(ProjectGraphMetadata {
            graph_id: "analysis".to_owned(),
            name: "Analysis".to_owned(),
            path: "/obj/analysis".to_owned(),
            role: ProjectGraphRole::Subgraph,
        });
        graph
            .select_graph_by_id("analysis")
            .expect("analysis graph should be selectable");
        let analysis_index = graph.add_procedural_asset_node("vy.asset.curve_cleanup");
        let missing_index = graph.add_procedural_asset_node("vy.asset.missing");

        let entries = graph.procedural_asset_gallery_entries();
        let asset_entry = entries
            .iter()
            .find(|entry| entry.asset_id == "vy.asset.curve_cleanup")
            .expect("declared asset should have a gallery entry");
        assert_eq!(asset_entry.display_name, "Curve cleanup");
        assert_eq!(asset_entry.version.as_deref(), Some("0.1.0"));
        assert_eq!(asset_entry.input_count, 1);
        assert_eq!(asset_entry.output_count, 1);
        assert_eq!(asset_entry.promoted_parameter_count, 2);
        assert_eq!(asset_entry.usages.len(), 2);
        assert!(asset_entry.usages.iter().any(|usage| {
            usage.node_index == main_index
                && usage.graph_id == "main"
                && usage.node_path == "/obj/main/Asset"
                && usage.version_status == OperatorVersionStatus::Current
        }));
        assert!(asset_entry.usages.iter().any(|usage| {
            usage.node_index == analysis_index
                && usage.graph_id == "analysis"
                && usage.node_path == "/obj/analysis/Asset"
                && usage.version_status == OperatorVersionStatus::Current
        }));

        let missing_entry = entries
            .iter()
            .find(|entry| entry.asset_id == "vy.asset.missing")
            .expect("missing asset usage should still have a gallery entry");
        assert!(missing_entry.missing_declaration);
        assert_eq!(missing_entry.version, None);
        assert_eq!(missing_entry.usages.len(), 1);
        assert_eq!(missing_entry.usages[0].node_index, missing_index);
        assert_eq!(
            missing_entry.usages[0].version_status,
            OperatorVersionStatus::MissingDeclaration
        );
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
                    label: Some("Minimum score".to_owned()),
                    kind: HoudiniParameterKind::Float,
                    default_value: HoudiniParameterValue::Float(0.55),
                    current_value: Some(HoudiniParameterValue::Float(0.55)),
                    range: Some(HoudiniNumericRange { min: 0.0, max: 1.0 }),
                    allowed_values: Vec::new(),
                    group: Some("Filter".to_owned()),
                    binding: Some(HoudiniParameterBinding {
                        internal_node_id: "filter.main".to_owned(),
                        internal_parameter_name: "score_threshold".to_owned(),
                    }),
                    help: "Promoted filter threshold.".to_owned(),
                },
                HoudiniParameterDeclaration {
                    name: "layer_name".to_owned(),
                    label: Some("Layer name".to_owned()),
                    kind: HoudiniParameterKind::String,
                    default_value: HoudiniParameterValue::String("Clean curves".to_owned()),
                    current_value: Some(HoudiniParameterValue::String("Clean curves".to_owned())),
                    range: None,
                    allowed_values: Vec::new(),
                    group: Some("Output".to_owned()),
                    binding: Some(HoudiniParameterBinding {
                        internal_node_id: "output.main".to_owned(),
                        internal_parameter_name: "layer_name".to_owned(),
                    }),
                    help: "Output layer label.".to_owned(),
                },
            ],
            external_artifacts: Vec::new(),
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
                label: None,
                kind: HoudiniParameterKind::Float,
                default_value: HoudiniParameterValue::Float(0.1),
                current_value: None,
                range: Some(HoudiniNumericRange {
                    min: 0.0,
                    max: 10.0,
                }),
                allowed_values: Vec::new(),
                group: None,
                binding: None,
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
