use std::sync::{Arc, Mutex, MutexGuard};

use egui::{
    Align2, Color32, DragValue, FontId, Pos2, Rect, Response, Sense, Slider, Stroke, StrokeKind,
    Ui, Vec2,
};
use re_log_channel::RecordingOpenBehavior;
use re_ui::UiExt as _;
use re_viewer_context::{
    ViewerContext,
    open_url::{OpenUrlOptions, ViewerOpenUrl},
};

pub(crate) mod model;

use self::model::{
    AttributeTableQuery, AttributeTableRow, AttributeTableSort, EvaluationState,
    GeneratedNodeBindingState, GeometryBounds, GraphAnnotationKind, GraphContainerAssetDraftError,
    GraphContainerCollapseError, GraphDocument, GraphEvaluationMode, GraphNavigationError,
    GraphPoint, GraphStyle, GraphWorkItemStatus, HoudiniNodeBinding, LayerKind,
    NativeOperatorLoadStatus, NetworkBadgeVisibility, NetworkBoxOrganizationSnapshot,
    NetworkCommentDisplayMode, NetworkNodeRingVisibility, NetworkViewDisplayOptions, NodeKind,
    NodeStatus, PRIMARY_GEOMETRY_OUTPUT, PythonEnvironmentResolveTrigger, PythonEnvironmentStatus,
    PythonOperatorDependencyStatus, ReferenceDiagnosticStatus, SourceExternalReferenceActionHint,
    SourceExternalReferenceActionKind, SourceExternalReferenceActionReport, SourceGalleryIndex,
    SourceGalleryItem, SourceGalleryItemKind, SourceLocator, SourceMetadata,
    SubstrateCoordinateContract,
};

const LARGE_ATTRIBUTE_TABLE_ROW_LIMIT: usize = 2_500;
const ATTRIBUTE_TABLE_PREVIEW_ROWS: usize = 200;
const NETWORK_BOX_FAST_DRAG_PEAK_DELTA_PIXELS: f32 = 18.0;
const NETWORK_DISPLAY_OPTIONS_ID: &str = "houdini_graph_network_display_options";
const SOURCE_GALLERY_INDEX_LIMIT: usize = 256;
const DEFAULT_ASSET_NAME: &str = "Curve cleanup";
const DEFAULT_ASSET_DESCRIPTION: &str = "Project-local graph asset.";
const DEFAULT_ASSET_HELP: &str = "Created from the current Houdini graph.";

pub(crate) type SharedHoudiniGraph = Arc<Mutex<GraphDocument>>;
pub(crate) type SharedHoudiniGraphPanel = Arc<Mutex<HoudiniGraphPanel>>;

pub(crate) fn new_shared_houdini_graph() -> SharedHoudiniGraph {
    Arc::new(Mutex::new(GraphDocument::sample()))
}

pub(crate) fn new_shared_houdini_graph_panel() -> SharedHoudiniGraphPanel {
    Arc::new(Mutex::new(HoudiniGraphPanel::default()))
}

pub(crate) fn install_shared_houdini_graph(egui_ctx: &egui::Context, graph: &SharedHoudiniGraph) {
    egui_ctx.data_mut(|data| data.insert_temp(shared_houdini_graph_id(), graph.clone()));
}

pub(crate) fn install_shared_houdini_graph_panel(
    egui_ctx: &egui::Context,
    panel: &SharedHoudiniGraphPanel,
) {
    egui_ctx.data_mut(|data| data.insert_temp(shared_houdini_graph_panel_id(), panel.clone()));
}

pub(crate) fn shared_houdini_graph_from_context(
    egui_ctx: &egui::Context,
) -> Option<SharedHoudiniGraph> {
    egui_ctx.data(|data| data.get_temp(shared_houdini_graph_id()))
}

pub(crate) fn shared_houdini_graph_panel_from_context(
    egui_ctx: &egui::Context,
) -> Option<SharedHoudiniGraphPanel> {
    egui_ctx.data(|data| data.get_temp(shared_houdini_graph_panel_id()))
}

pub(crate) fn lock_houdini_graph(graph: &SharedHoudiniGraph) -> MutexGuard<'_, GraphDocument> {
    graph
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(crate) fn lock_houdini_graph_panel(
    panel: &SharedHoudiniGraphPanel,
) -> MutexGuard<'_, HoudiniGraphPanel> {
    panel
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn shared_houdini_graph_id() -> egui::Id {
    egui::Id::new("houdini_graph_state")
}

fn shared_houdini_graph_panel_id() -> egui::Id {
    egui::Id::new("houdini_graph_panel_state")
}

pub(crate) struct HoudiniGraphPanel {
    selected_node: usize,
    selected_nodes: Vec<usize>,
    selected_edge: Option<String>,
    selected_annotation: Option<usize>,
    context_menu_canvas: bool,
    context_menu_edge: Option<String>,
    active_graph_pane: GraphWorkbenchPane,
    dragging_node: Option<usize>,
    node_drag_start_position: Option<GraphPoint>,
    node_drag_start_network_box_states: Vec<NetworkBoxOrganizationSnapshot>,
    node_drag_peak_delta_pixels: f32,
    dragging_annotation: Option<usize>,
    annotation_drag_start_position: Option<GraphPoint>,
    annotation_drag_start_member_positions: Vec<(String, GraphPoint)>,
    resizing_annotation: Option<usize>,
    annotation_resize_start_size: Option<GraphPoint>,
    connection_drag: Option<ConnectionDragState>,
    selection_drag: Option<SelectionDragState>,
    graph_view_zoom: f32,
    graph_view_pan: Vec2,
    pending_frame_selected: bool,
    tab_menu_open: bool,
    tab_menu_anchor: Pos2,
    tab_menu_filter_needs_focus: bool,
    tab_menu_selection_index: usize,
    last_parquet_path: Option<String>,
    parquet_status: Option<String>,
    graph_document_status: Option<String>,
    recording_status: Option<String>,
    package_manifest_status: Option<String>,
    benchmark_status: Option<String>,
    shelf_status: Option<String>,
    graph_container_status: Option<String>,
    benchmark_curve_count: usize,
    benchmark_polygon_count: usize,
    operator_filter: String,
    operator_history: Vec<OperatorPaletteAction>,
    graph_search_filter: String,
    node_info_open: bool,
    node_info_pinned: bool,
    node_info_refresh_automatically: bool,
    node_info_show_additional: bool,
    node_info_show_debug: bool,
    table_search: String,
    table_minimum_score_enabled: bool,
    table_minimum_score: f32,
    table_sort: AttributeTableSort,
    table_sort_descending: bool,
    table_commit_status: Option<String>,
    table_selected_record_fingerprint: Option<String>,
    table_selection_status: Option<String>,
    asset_name: String,
    asset_description: String,
    asset_help: String,
    asset_gallery_filter: String,
    asset_status: Option<String>,
    source_gallery_location: String,
    source_gallery_manifest_json: String,
    source_gallery_filter: String,
    source_gallery_selected_id: Option<String>,
    source_gallery_checked_ids: Vec<String>,
    source_gallery_index: Option<SourceGalleryIndex>,
    source_gallery_status: Option<String>,
    source_reference_copied_locator: Option<String>,
    python_uv_executable_path: String,
    python_existing_environment_path: String,
    python_create_environment_path: String,
}

impl Default for HoudiniGraphPanel {
    fn default() -> Self {
        Self {
            selected_node: 1,
            selected_nodes: vec![1],
            selected_edge: None,
            selected_annotation: None,
            context_menu_canvas: false,
            context_menu_edge: None,
            active_graph_pane: GraphWorkbenchPane::Parameters,
            dragging_node: None,
            node_drag_start_position: None,
            node_drag_start_network_box_states: Vec::new(),
            node_drag_peak_delta_pixels: 0.0,
            dragging_annotation: None,
            annotation_drag_start_position: None,
            annotation_drag_start_member_positions: Vec::new(),
            resizing_annotation: None,
            annotation_resize_start_size: None,
            connection_drag: None,
            selection_drag: None,
            graph_view_zoom: 1.0,
            graph_view_pan: Vec2::ZERO,
            pending_frame_selected: false,
            tab_menu_open: false,
            tab_menu_anchor: Pos2::ZERO,
            tab_menu_filter_needs_focus: false,
            tab_menu_selection_index: 0,
            last_parquet_path: None,
            parquet_status: None,
            graph_document_status: None,
            recording_status: None,
            package_manifest_status: None,
            benchmark_status: None,
            shelf_status: None,
            graph_container_status: None,
            benchmark_curve_count: 10_000,
            benchmark_polygon_count: 1_000,
            operator_filter: String::new(),
            operator_history: Vec::new(),
            graph_search_filter: String::new(),
            node_info_open: true,
            node_info_pinned: false,
            node_info_refresh_automatically: true,
            node_info_show_additional: false,
            node_info_show_debug: false,
            table_search: String::new(),
            table_minimum_score_enabled: false,
            table_minimum_score: 0.0,
            table_sort: AttributeTableSort::RecordIndex,
            table_sort_descending: false,
            table_commit_status: None,
            table_selected_record_fingerprint: None,
            table_selection_status: None,
            asset_name: DEFAULT_ASSET_NAME.to_owned(),
            asset_description: DEFAULT_ASSET_DESCRIPTION.to_owned(),
            asset_help: DEFAULT_ASSET_HELP.to_owned(),
            asset_gallery_filter: String::new(),
            asset_status: None,
            source_gallery_location: String::new(),
            source_gallery_manifest_json: String::new(),
            source_gallery_filter: String::new(),
            source_gallery_selected_id: None,
            source_gallery_checked_ids: Vec::new(),
            source_gallery_index: None,
            source_gallery_status: None,
            source_reference_copied_locator: None,
            python_uv_executable_path: String::new(),
            python_existing_environment_path: String::new(),
            python_create_environment_path: String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GraphWorkbenchPane {
    Operators,
    Find,
    Assets,
    Gallery,
    Parameters,
    Info,
    Display,
    Layers,
}

impl GraphWorkbenchPane {
    fn label(self) -> &'static str {
        match self {
            Self::Operators => "Ops",
            Self::Find => "Find",
            Self::Assets => "Assets",
            Self::Gallery => "Gallery",
            Self::Parameters => "Parms",
            Self::Info => "Info",
            Self::Display => "Display",
            Self::Layers => "Layers",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ConnectionDragState {
    from_node_index: usize,
    from_node_id: String,
    from_output: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SelectionDragState {
    start: Pos2,
    current: Pos2,
}

impl SelectionDragState {
    fn rect(self) -> Rect {
        Rect::from_two_pos(self.start, self.current)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodePortKind {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NodePortHit {
    node_index: usize,
    kind: NodePortKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ConnectionDragPreview {
    Floating,
    NonInput,
    Valid,
    Invalid(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperatorPaletteAction {
    AddOutNull,
    AddReference,
    AddRepairProjection,
    DuplicateSelected,
    CollapseSelectionToSubnet,
    EnterSelectedSubnet,
    GoUpOneGraph,
    CreateAssetFromSelectedSubnet,
    AddNetworkBox,
    AddStickyNote,
    DuplicatePolygons,
    DuplicateCurves,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperatorPaletteCategory {
    Create,
    Navigate,
    Organize,
    LayerActions,
}

impl OperatorPaletteCategory {
    const ALL: [Self; 4] = [
        Self::Create,
        Self::Navigate,
        Self::Organize,
        Self::LayerActions,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Create => "Create",
            Self::Navigate => "Navigate",
            Self::Organize => "Organize",
            Self::LayerActions => "Layer Actions",
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Navigate => "navigate",
            Self::Organize => "organize",
            Self::LayerActions => "layer_actions",
        }
    }

    fn default_open(self, filter_is_empty: bool) -> bool {
        !filter_is_empty || matches!(self, Self::Create | Self::Navigate)
    }
}

#[derive(Clone, Copy)]
struct OperatorPaletteEntry {
    action: OperatorPaletteAction,
    category: OperatorPaletteCategory,
    label: &'static str,
    detail: &'static str,
    aliases: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct OperatorPaletteUiOptions {
    id_salt: &'static str,
    grouped: bool,
    show_recent: bool,
    include_organization: bool,
    include_layers: bool,
    highlighted_action: Option<OperatorPaletteAction>,
}

#[derive(Clone, Debug)]
struct GraphSearchResult {
    target: GraphSearchTarget,
    label: String,
    kind: &'static str,
    detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum GraphSearchTarget {
    Node { index: usize, graph_id: String },
    Annotation(usize),
}

impl HoudiniGraphPanel {
    pub(crate) fn show_network_view(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        install_shared_houdini_graph(ui.ctx(), shared_graph);
        let mut graph = lock_houdini_graph(shared_graph);
        egui::Frame {
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        }
        .show(ui, |ui| self.network_view_contents_ui(ui, &mut graph));
    }

    pub(crate) fn show_parameters_view(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        self.show_workbench_view(ui, shared_graph, GraphWorkbenchPane::Parameters);
    }

    pub(crate) fn show_info_view(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        self.show_workbench_view(ui, shared_graph, GraphWorkbenchPane::Info);
    }

    pub(crate) fn show_display_view(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        self.show_workbench_view(ui, shared_graph, GraphWorkbenchPane::Display);
    }

    pub(crate) fn show_operators_view(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        self.show_workbench_view(ui, shared_graph, GraphWorkbenchPane::Operators);
    }

    pub(crate) fn show_find_view(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        self.show_workbench_view(ui, shared_graph, GraphWorkbenchPane::Find);
    }

    pub(crate) fn show_layers_view(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        self.show_workbench_view(ui, shared_graph, GraphWorkbenchPane::Layers);
    }

    pub(crate) fn show_assets_view(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        self.show_workbench_view(ui, shared_graph, GraphWorkbenchPane::Assets);
    }

    pub(crate) fn show_gallery_view(
        &mut self,
        ui: &mut Ui,
        shared_graph: &SharedHoudiniGraph,
        viewer_ctx: Option<&ViewerContext<'_>>,
    ) {
        self.active_graph_pane = GraphWorkbenchPane::Gallery;
        install_shared_houdini_graph(ui.ctx(), shared_graph);
        let mut graph = lock_houdini_graph(shared_graph);
        egui::Frame {
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        }
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("houdini_graph_workbench_view_Gallery")
                .auto_shrink([false, false])
                .max_height(ui.available_height().max(240.0))
                .show(ui, |ui| {
                    self.source_gallery_ui(ui, &mut graph, viewer_ctx);
                });
        });
    }

    pub(crate) fn show_data_view(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        install_shared_houdini_graph(ui.ctx(), shared_graph);
        let mut graph = lock_houdini_graph(shared_graph);
        egui::Frame {
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        }
        .show(ui, |ui| self.data_workspace_ui(ui, &mut graph));
    }

    pub(crate) fn show_outputs_view(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        install_shared_houdini_graph(ui.ctx(), shared_graph);
        let mut graph = lock_houdini_graph(shared_graph);
        egui::Frame {
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        }
        .show(ui, |ui| self.outputs_workspace_ui(ui, &mut graph));
    }

    pub(crate) fn show_shelf_view(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        install_shared_houdini_graph(ui.ctx(), shared_graph);
        let mut graph = lock_houdini_graph(shared_graph);
        egui::Frame {
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        }
        .show(ui, |ui| self.shelf_workspace_ui(ui, &mut graph));
    }

    pub(crate) fn show_execution_view(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        install_shared_houdini_graph(ui.ctx(), shared_graph);
        let mut graph = lock_houdini_graph(shared_graph);
        egui::Frame {
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        }
        .show(ui, |ui| self.execution_workspace_ui(ui, &mut graph));
    }

    pub(crate) fn show_project_view(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        install_shared_houdini_graph(ui.ctx(), shared_graph);
        let mut graph = lock_houdini_graph(shared_graph);
        egui::Frame {
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        }
        .show(ui, |ui| self.project_workspace_ui(ui, &mut graph));
    }

    fn show_graph_workbench_pane(&mut self, pane: GraphWorkbenchPane) {
        self.active_graph_pane = pane;
    }

    fn network_view_contents_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        self.network_editor_toolbar_ui(ui, graph);
        ui.add_space(4.0);
        self.node_graph_ui(ui, graph, (ui.available_height() - 4.0).max(300.0));
    }

    fn show_workbench_view(
        &mut self,
        ui: &mut Ui,
        shared_graph: &SharedHoudiniGraph,
        pane: GraphWorkbenchPane,
    ) {
        self.active_graph_pane = pane;
        install_shared_houdini_graph(ui.ctx(), shared_graph);
        let mut graph = lock_houdini_graph(shared_graph);
        egui::Frame {
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        }
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt(format!("houdini_graph_workbench_view_{}", pane.label()))
                .auto_shrink([false, false])
                .max_height(ui.available_height().max(240.0))
                .show(ui, |ui| {
                    self.graph_workbench_content_ui(ui, &mut graph, self.active_graph_pane);
                });
        });
    }

    fn network_editor_toolbar_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        let selected_label = self.selected_item_label(graph);

        ui.horizontal_wrapped(|ui| {
            ui.weak("/obj/main");
            ui.separator();

            ui.menu_button("Add", |ui| {
                if ui.button("TAB Menu...").clicked() {
                    self.open_operator_chooser_at(ui.cursor().min);
                    ui.close();
                }
                ui.separator();
                self.operator_menu_action_ui(ui, graph, OperatorPaletteAction::AddOutNull);
                self.operator_menu_action_ui(ui, graph, OperatorPaletteAction::AddReference);
                self.operator_menu_action_ui(ui, graph, OperatorPaletteAction::AddRepairProjection);
                ui.separator();
                self.operator_menu_action_ui_with_label(
                    ui,
                    graph,
                    OperatorPaletteAction::AddNetworkBox,
                    "Network Box from Selected    Shift+O",
                );
                self.operator_menu_action_ui_with_label(
                    ui,
                    graph,
                    OperatorPaletteAction::AddStickyNote,
                    "Sticky Note    Shift+P",
                );
            });

            ui.menu_button("Edit", |ui| {
                self.operator_menu_action_ui_with_label(
                    ui,
                    graph,
                    OperatorPaletteAction::DuplicateSelected,
                    "Duplicate Selected",
                );
                ui.separator();
                if ui.button("Parameters").clicked() {
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Parameters);
                    ui.close();
                }
                if ui.button("Node Information").clicked() {
                    self.node_info_open = true;
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Info);
                    ui.close();
                }
                if ui.button("Pin Node Information").clicked() {
                    self.node_info_open = true;
                    self.node_info_pinned = true;
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Info);
                    ui.close();
                }
                if ui.button("Edit Comment").clicked() {
                    self.node_info_open = true;
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Info);
                    ui.close();
                }
            });

            ui.menu_button("Go", |ui| {
                if ui.button("Home Network    H").clicked() {
                    self.reset_graph_view();
                    ui.close();
                }
                if ui.button("Frame Selected    F").clicked() {
                    self.pending_frame_selected = true;
                    ui.close();
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("Display Options").clicked() {
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Display);
                    toggle_network_display_options(ui);
                    ui.close();
                }
                ui.separator();
                ui.weak("Show Node Ring");
                for visibility in NetworkNodeRingVisibility::ALL {
                    if ui
                        .selectable_label(
                            graph.network_view.node_ring_visibility == visibility,
                            visibility.label(),
                        )
                        .clicked()
                    {
                        graph.network_view.node_ring_visibility = visibility;
                        ui.close();
                    }
                }
            });

            ui.menu_button("Tools", |ui| {
                if ui.button("Operators").clicked() {
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Operators);
                    ui.close();
                }
                if ui.button("Find").clicked() {
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Find);
                    ui.close();
                }
                if ui.button("Assets").clicked() {
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Assets);
                    ui.close();
                }
                if ui.button("Gallery").clicked() {
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Gallery);
                    ui.close();
                }
                if ui.button("Parameters").clicked() {
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Parameters);
                    ui.close();
                }
                if ui.button("Node Info").clicked() {
                    self.node_info_open = true;
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Info);
                    ui.close();
                }
                if ui.button("Display").clicked() {
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Display);
                    ui.close();
                }
                if ui.button("Layers").clicked() {
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Layers);
                    ui.close();
                }
                ui.separator();
                if ui.button("Run Selected").clicked() {
                    graph.request_node_run(self.selected_node);
                    graph.complete_node_run(self.selected_node);
                    ui.close();
                }
                if ui.button("Evaluate Output").clicked() {
                    graph.demand_output_evaluation();
                    ui.close();
                }
            });

            ui.menu_button("Layout", |ui| {
                if ui.button("Reset View    H").clicked() {
                    self.reset_graph_view();
                    ui.close();
                }
                if ui
                    .button("Resize Selected Box to Contents    Shift+M")
                    .clicked()
                {
                    self.resize_selected_network_box_to_contents(graph);
                    ui.close();
                }
                if ui.button("Resize Boxes to Contents").clicked() {
                    self.resize_all_network_boxes_to_contents(graph);
                    ui.close();
                }
                ui.separator();
                if ui.button("Collapse Boxes and Notes").clicked() {
                    self.set_all_annotation_collapsed(graph, true);
                    ui.close();
                }
                if ui.button("Expand Boxes and Notes").clicked() {
                    self.set_all_annotation_collapsed(graph, false);
                    ui.close();
                }
            });

            ui.separator();
            ui.weak(selected_label);
            ui.separator();
            if ui.small_button("-").clicked() {
                self.zoom_graph_view(1.0 / 1.15);
            }
            if ui.small_button("1:1").clicked() {
                self.graph_view_zoom = 1.0;
                self.graph_view_pan = Vec2::ZERO;
            }
            if ui.small_button("+").clicked() {
                self.zoom_graph_view(1.15);
            }
            ui.weak(format!("{:.0}%", self.graph_view_zoom * 100.0));
        });
    }

    fn zoom_graph_view(&mut self, factor: f32) {
        self.graph_view_zoom = (self.graph_view_zoom * factor).clamp(0.45, 2.6);
    }

    fn reset_graph_view(&mut self) {
        self.graph_view_zoom = 1.0;
        self.graph_view_pan = Vec2::ZERO;
    }

    fn selected_item_label(&self, graph: &GraphDocument) -> String {
        if let Some(annotation_index) = self.selected_annotation
            && graph.annotation_belongs_to_current_graph(annotation_index)
            && let Some(annotation) = graph.annotations.get(annotation_index)
        {
            return format!("{}: {}", annotation.kind.as_str(), annotation.title);
        }

        graph
            .nodes
            .get(self.selected_node)
            .map(|node| format!("{} ({})", node.name, node.kind.as_str()))
            .unwrap_or_else(|| "none".to_owned())
    }

    fn frame_selected_node_in_rect(
        &mut self,
        graph: &GraphDocument,
        layout_rect: Rect,
        node_size: Vec2,
    ) -> bool {
        let Some(node) = graph.nodes.get(self.selected_node) else {
            return false;
        };
        let selected_center = map_node_layout_point(
            layout_rect,
            node.layout_position,
            node_size,
            self.graph_view_zoom,
            Vec2::ZERO,
        );
        self.graph_view_pan = layout_rect.center() - selected_center;
        true
    }

    fn frame_selected_annotation_in_rect(
        &mut self,
        graph: &GraphDocument,
        layout_rect: Rect,
    ) -> bool {
        let Some(annotation_index) = self.selected_annotation else {
            return false;
        };
        if !graph.annotation_belongs_to_current_graph(annotation_index) {
            self.selected_annotation = None;
            return false;
        }
        let Some(annotation) = graph.annotations.get(annotation_index) else {
            return false;
        };
        let selected_center =
            display_annotation_rect(layout_rect, annotation, self.graph_view_zoom, Vec2::ZERO)
                .center();
        self.graph_view_pan = layout_rect.center() - selected_center;
        true
    }

    fn frame_selected_item_in_rect(
        &mut self,
        graph: &GraphDocument,
        layout_rect: Rect,
        node_size: Vec2,
    ) -> bool {
        if self.selected_annotation.is_some() {
            self.frame_selected_annotation_in_rect(graph, layout_rect)
        } else {
            self.frame_selected_node_in_rect(graph, layout_rect, node_size)
        }
    }

    fn resize_selected_network_box_to_contents(&mut self, graph: &mut GraphDocument) -> bool {
        if let Some(annotation_index) = self.selected_annotation
            && graph.annotation_belongs_to_current_graph(annotation_index)
            && graph
                .annotations
                .get(annotation_index)
                .is_some_and(|annotation| annotation.kind == GraphAnnotationKind::NetworkBox)
        {
            return graph.resize_network_box_to_contents(annotation_index);
        }

        let Some(annotation_index) =
            graph
                .current_graph_annotation_indices()
                .into_iter()
                .find(|annotation_index| {
                    graph
                        .annotations
                        .get(*annotation_index)
                        .is_some_and(|annotation| {
                            annotation.kind == GraphAnnotationKind::NetworkBox
                                && graph.nodes.get(self.selected_node).is_some_and(|node| {
                                    annotation
                                        .member_node_ids
                                        .iter()
                                        .any(|member_id| member_id == &node.node_id)
                                })
                        })
                })
        else {
            return false;
        };
        graph.resize_network_box_to_contents(annotation_index)
    }

    fn resize_all_network_boxes_to_contents(&mut self, graph: &mut GraphDocument) {
        let network_box_indices = graph
            .current_graph_annotation_indices()
            .into_iter()
            .filter(|index| {
                graph
                    .annotations
                    .get(*index)
                    .is_some_and(|annotation| annotation.kind == GraphAnnotationKind::NetworkBox)
            })
            .collect::<Vec<_>>();
        for index in network_box_indices {
            graph.resize_network_box_to_contents(index);
        }
    }

    fn set_all_annotation_collapsed(&mut self, graph: &mut GraphDocument, collapsed: bool) {
        graph.set_all_annotations_collapsed(collapsed);
    }

    fn open_operator_chooser_at(&mut self, anchor: Pos2) {
        self.operator_filter.clear();
        self.tab_menu_open = true;
        self.tab_menu_anchor = anchor + egui::vec2(6.0, 6.0);
        self.tab_menu_filter_needs_focus = true;
        self.tab_menu_selection_index = 0;
        self.show_graph_workbench_pane(GraphWorkbenchPane::Operators);
    }

    fn operator_menu_action_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        action: OperatorPaletteAction,
    ) {
        let entry = operator_palette_entry(action);
        self.operator_menu_action_ui_with_label(ui, graph, action, entry.label);
    }

    fn operator_menu_action_ui_with_label(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        action: OperatorPaletteAction,
        label: &str,
    ) {
        if !operator_palette_action_available(
            graph,
            self.selected_node,
            &self.selected_nodes,
            action,
        ) {
            return;
        }
        let entry = operator_palette_entry(action);
        if ui.button(label).on_hover_text(entry.detail).clicked() {
            self.apply_operator_palette_action(graph, action);
            ui.close();
        }
    }

    fn operator_palette_compact_button_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        action: OperatorPaletteAction,
    ) {
        if !operator_palette_action_available(
            graph,
            self.selected_node,
            &self.selected_nodes,
            action,
        ) {
            return;
        }
        let entry = operator_palette_entry(action);
        if ui
            .small_button(entry.label)
            .on_hover_text(entry.detail)
            .clicked()
        {
            self.apply_operator_palette_action(graph, action);
        }
    }

    fn operator_palette_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        options: OperatorPaletteUiOptions,
    ) -> bool {
        let filter = self.operator_filter.trim().to_lowercase();
        let filter_is_empty = filter.is_empty();
        let mut applied_action = false;
        let mut shown_operator = false;

        if options.show_recent && filter_is_empty {
            ui.horizontal_wrapped(|ui| {
                ui.weak("Recent");
                let recent_actions = self.operator_history.clone();
                if recent_actions.is_empty() {
                    ui.weak("No recent operators yet.");
                } else {
                    for action in recent_actions {
                        if !operator_palette_action_included(
                            action,
                            options.include_organization,
                            options.include_layers,
                        ) {
                            continue;
                        }
                        if !operator_palette_action_available(
                            graph,
                            self.selected_node,
                            &self.selected_nodes,
                            action,
                        ) {
                            continue;
                        }
                        let entry = operator_palette_entry(action);
                        if ui
                            .small_button(entry.label)
                            .on_hover_text(entry.detail)
                            .clicked()
                        {
                            applied_action = self.apply_operator_palette_action(graph, action);
                        }
                    }
                }
            });
            ui.separator();
        }

        let entries = operator_palette_entries(
            graph,
            self.selected_node,
            &self.selected_nodes,
            options.include_organization,
            options.include_layers,
        );

        for category in OperatorPaletteCategory::ALL {
            let matching_entries = entries
                .iter()
                .copied()
                .filter(|entry| {
                    entry.category == category
                        && operator_matches(&filter, entry.label, entry.aliases)
                })
                .collect::<Vec<_>>();
            if matching_entries.is_empty() {
                continue;
            }

            shown_operator = true;
            if options.grouped {
                egui::CollapsingHeader::new(category.label())
                    .id_salt(format!("{}_{}", options.id_salt, category.id()))
                    .default_open(category.default_open(filter_is_empty))
                    .show(ui, |ui| {
                        for entry in matching_entries {
                            if operator_palette_button_ui(
                                ui,
                                entry,
                                options.highlighted_action == Some(entry.action),
                            ) {
                                applied_action =
                                    self.apply_operator_palette_action(graph, entry.action);
                            }
                        }
                    });
            } else {
                ui.weak(category.label());
                for entry in matching_entries {
                    if operator_palette_button_ui(
                        ui,
                        entry,
                        options.highlighted_action == Some(entry.action),
                    ) {
                        applied_action = self.apply_operator_palette_action(graph, entry.action);
                    }
                }
            }
        }

        if !shown_operator {
            ui.weak("No matching graph-backed operators.");
        }

        applied_action
    }

    fn matching_operator_palette_actions(
        &self,
        graph: &GraphDocument,
        include_organization: bool,
        include_layers: bool,
    ) -> Vec<OperatorPaletteAction> {
        let filter = self.operator_filter.trim().to_lowercase();
        let entries = operator_palette_entries(
            graph,
            self.selected_node,
            &self.selected_nodes,
            include_organization,
            include_layers,
        );
        let mut actions = Vec::new();
        for category in OperatorPaletteCategory::ALL {
            actions.extend(entries.iter().filter_map(|entry| {
                (entry.category == category
                    && operator_matches(&filter, entry.label, entry.aliases))
                .then_some(entry.action)
            }));
        }
        actions
    }

    fn apply_operator_palette_action(
        &mut self,
        graph: &mut GraphDocument,
        action: OperatorPaletteAction,
    ) -> bool {
        let applied = match action {
            OperatorPaletteAction::AddOutNull => {
                let node_index = graph.add_null_operator_node("OUT_MAIN");
                self.select_single_node(node_index);
                self.node_info_open = true;
                self.show_graph_workbench_pane(GraphWorkbenchPane::Parameters);
                true
            }
            OperatorPaletteAction::AddReference => {
                if let Some(index) = graph.add_reference_input_node(self.selected_node) {
                    self.select_single_node(index);
                    self.node_info_open = true;
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Parameters);
                    true
                } else {
                    false
                }
            }
            OperatorPaletteAction::AddRepairProjection => {
                if let Some(index) = graph
                    .create_assisted_projection_for_first_repairable_reference_target(
                        self.selected_node,
                    )
                {
                    self.select_single_node(index);
                    self.node_info_open = true;
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Parameters);
                    true
                } else {
                    false
                }
            }
            OperatorPaletteAction::DuplicateSelected => {
                if let Some(index) = graph.duplicate_node(self.selected_node) {
                    self.select_single_node(index);
                    self.selected_annotation = None;
                    self.node_info_open = true;
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Parameters);
                    true
                } else {
                    false
                }
            }
            OperatorPaletteAction::CollapseSelectionToSubnet => {
                self.collapse_selected_nodes_to_graph_container(graph)
            }
            OperatorPaletteAction::EnterSelectedSubnet => {
                self.enter_selected_graph_container(graph)
            }
            OperatorPaletteAction::GoUpOneGraph => self.exit_current_graph_to_parent(graph),
            OperatorPaletteAction::CreateAssetFromSelectedSubnet => {
                self.create_asset_from_selected_graph_container(graph)
            }
            OperatorPaletteAction::AddNetworkBox => {
                self.selected_annotation = graph.add_network_box_for_node(self.selected_node);
                self.show_graph_workbench_pane(GraphWorkbenchPane::Operators);
                true
            }
            OperatorPaletteAction::AddStickyNote => {
                self.selected_annotation = graph.add_sticky_note_near_node(self.selected_node);
                self.show_graph_workbench_pane(GraphWorkbenchPane::Operators);
                true
            }
            OperatorPaletteAction::DuplicatePolygons => {
                graph.duplicate_layer_view(LayerKind::Polygons, "Polygons Copy");
                true
            }
            OperatorPaletteAction::DuplicateCurves => {
                graph.duplicate_layer_view(LayerKind::Curves, "Curves Copy");
                true
            }
        };

        if applied {
            self.record_operator_palette_action(action);
        }

        applied
    }

    fn record_operator_palette_action(&mut self, action: OperatorPaletteAction) {
        self.operator_history
            .retain(|existing_action| *existing_action != action);
        self.operator_history.insert(0, action);
        self.operator_history.truncate(4);
    }

    fn data_workspace_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.strong("Attribute Table");
        self.attribute_table_ui(ui, graph);

        ui.add_space(8.0);
        ui.strong("Import");
        self.parquet_import_ui(ui, graph);

        ui.add_space(8.0);
        ui.strong("Benchmark");
        self.render_benchmark_ui(ui, graph);
    }

    fn outputs_workspace_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.strong("Recording");
        self.recording_export_ui(ui, graph);

        ui.add_space(8.0);
        ui.strong("Output Summary");
        self.output_summary_ui(ui, graph);

        ui.add_space(8.0);
        ui.strong("Node Info");
        self.node_info_ui(ui, graph);
    }

    fn shelf_workspace_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        egui::ScrollArea::vertical()
            .id_salt("houdini_shelf_tools")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.strong("Evaluate");
                ui.horizontal_wrapped(|ui| {
                    let has_selected_node = self.selected_node < graph.nodes.len();
                    if ui
                        .add_enabled(has_selected_node, egui::Button::new("Queue Selected"))
                        .on_hover_text("Queue the selected graph node as a work item.")
                        .clicked()
                    {
                        graph.queue_node_evaluation(self.selected_node);
                        self.shelf_status = graph
                            .nodes
                            .get(self.selected_node)
                            .map(|node| format!("Queued selected node: {}", node.name));
                    }
                    if ui
                        .add_enabled(has_selected_node, egui::Button::new("Run Selected"))
                        .on_hover_text("Request a manual run for the selected graph node.")
                        .clicked()
                    {
                        graph.request_node_run(self.selected_node);
                        self.shelf_status = graph.nodes.get(self.selected_node).map(|node| {
                            format!("Requested run for selected node: {}", node.name)
                        });
                    }
                    if ui
                        .button("Evaluate Output")
                        .on_hover_text("Evaluate participating output nodes through the graph model.")
                        .clicked()
                    {
                        graph.demand_output_evaluation();
                        self.shelf_status = Some("Evaluated graph output.".to_owned());
                    }
                });

                ui.add_space(10.0);
                ui.strong("Starter Graphs");
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button("Malware Byteplot")
                        .on_hover_text("Load the malware byteplot starter graph.")
                        .clicked()
                    {
                        *graph = GraphDocument::malware_starter();
                        self.select_single_node(0);
                        self.selected_annotation = None;
                        self.node_info_open = true;
                        self.active_graph_pane = GraphWorkbenchPane::Info;
                        self.shelf_status =
                            Some("Loaded malware byteplot starter graph.".to_owned());
                    }
                    if ui
                        .button("Cubic Sample")
                        .on_hover_text("Load the built-in native cubic Bezier sample graph.")
                        .clicked()
                    {
                        *graph = GraphDocument::sample();
                        self.select_single_node(0);
                        self.selected_annotation = None;
                        self.node_info_open = true;
                        self.active_graph_pane = GraphWorkbenchPane::Info;
                        self.shelf_status = Some("Loaded cubic sample graph.".to_owned());
                    }
                });

                ui.add_space(10.0);
                ui.strong("Assets");
                if ui
                    .button("Create Asset Node")
                    .on_hover_text(
                        "Create a project-local asset from the current graph and place an instance node.",
                    )
                    .clicked()
                {
                    let (asset_id, node_index) = graph.create_asset_instance_from_graph(
                        self.asset_name.trim(),
                        self.asset_description.trim(),
                        self.asset_help.trim(),
                    );
                    self.select_single_node(node_index);
                    self.selected_annotation = None;
                    self.active_graph_pane = GraphWorkbenchPane::Assets;
                    self.asset_status = Some(format!("Created project asset: {asset_id}"));
                    self.shelf_status = Some(format!("Created asset node: {asset_id}"));
                }

                if ui
                    .button("Asset Gallery")
                    .on_hover_text("Open the project-local asset gallery.")
                    .clicked()
                {
                    self.show_graph_workbench_pane(GraphWorkbenchPane::Assets);
                }

                if let Some(status) = &self.shelf_status {
                    ui.add_space(8.0);
                    ui.weak(status);
                }
            });
    }

    fn execution_workspace_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.strong("Work Items");

        ui.horizontal(|ui| {
            ui.weak("Evaluation");
            egui::ComboBox::from_id_salt("houdini_execution_evaluation_mode")
                .selected_text(graph.evaluation_mode.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut graph.evaluation_mode,
                        GraphEvaluationMode::Automatic,
                        GraphEvaluationMode::Automatic.as_str(),
                    );
                    ui.selectable_value(
                        &mut graph.evaluation_mode,
                        GraphEvaluationMode::OnInteractionComplete,
                        GraphEvaluationMode::OnInteractionComplete.as_str(),
                    );
                    ui.selectable_value(
                        &mut graph.evaluation_mode,
                        GraphEvaluationMode::Manual,
                        GraphEvaluationMode::Manual.as_str(),
                    );
                });
        });

        ui.horizontal(|ui| {
            let has_selected_node = self.selected_node < graph.nodes.len();
            if ui
                .add_enabled(has_selected_node, egui::Button::new("Queue Selected"))
                .clicked()
            {
                graph.queue_node_evaluation(self.selected_node);
            }
            if ui
                .add_enabled(has_selected_node, egui::Button::new("Run Selected"))
                .clicked()
            {
                graph.request_node_run(self.selected_node);
            }
            if ui
                .add_enabled(has_selected_node, egui::Button::new("Cancel"))
                .clicked()
            {
                graph.cancel_node_run(self.selected_node);
            }
            if ui
                .add_enabled(has_selected_node, egui::Button::new("Retry"))
                .clicked()
            {
                graph.retry_work_item_for_node(self.selected_node);
            }
            if ui
                .add_enabled(has_selected_node, egui::Button::new("Complete"))
                .clicked()
            {
                graph.complete_node_run(self.selected_node);
            }
        });

        ui.weak("Runtime evaluation state; not saved with the graph sidecar.");
        ui.add_space(8.0);

        if graph.work_items.is_empty() {
            ui.weak("No graph evaluation work has been requested.");
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("houdini_execution_work_items")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("houdini_execution_work_item_grid")
                    .num_columns(6)
                    .spacing([12.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Node");
                        ui.strong("Output");
                        ui.strong("Status");
                        ui.strong("Progress");
                        ui.strong("Fingerprint");
                        ui.strong("Summary");
                        ui.end_row();

                        for item in graph.work_items.iter().rev() {
                            ui.label(&item.node_name);
                            ui.label(&item.output_name);
                            ui.colored_label(
                                work_item_status_color(ui, item.status),
                                item.status.as_str(),
                            );
                            ui.label(format!("{:.0}%", item.progress * 100.0));
                            ui.monospace(&item.fingerprint);
                            ui.vertical(|ui| {
                                ui.label(&item.summary);
                                if let Some(diagnostic) = &item.diagnostic {
                                    ui.colored_label(ui.visuals().warn_fg_color, diagnostic);
                                }
                            });
                            ui.end_row();
                        }
                    });
            });
    }

    fn project_workspace_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.strong("Graph Model");
        self.graph_document_ui(ui, graph);

        ui.add_space(8.0);
        ui.strong("Asset");
        self.asset_authoring_ui(ui, graph);
        if ui
            .button("Open Asset Gallery")
            .on_hover_text("Show project-local asset definitions and all graph usages.")
            .clicked()
        {
            self.show_graph_workbench_pane(GraphWorkbenchPane::Assets);
        }

        ui.add_space(8.0);
        self.asset_gallery_ui(ui, graph);

        ui.add_space(8.0);
        ui.strong("Python");
        self.python_environment_ui(ui, graph);

        ui.add_space(8.0);
        ui.strong("Source");
        self.source_summary_ui(ui, graph);
    }

    fn selected_node_controls_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        let Some(info) = graph.selected_node_info(self.selected_node) else {
            ui.weak("Select a node to edit graph-owned parameters.");
            return;
        };

        self.project_command_history_ui(ui, graph);
        self.selected_node_identity_ui(ui, graph, &info);
        if self.selected_node_graph_container_ui(ui, graph) {
            return;
        }
        self.selected_node_parameter_ui(ui, graph, &info);
        self.selected_node_flags_ui(ui, graph, &info);
        self.selected_node_evaluation_ui(ui, graph, &info);
        self.selected_node_comment_ui(ui, graph);
        self.selected_node_operator_settings_ui(ui, graph, &info);
    }

    fn project_command_history_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        egui::CollapsingHeader::new("Command History")
            .id_salt("houdini_graph_parms_command_history")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let undo_label = graph
                        .undo_project_command_label()
                        .unwrap_or_else(|| "Nothing to undo".to_owned());
                    let redo_label = graph
                        .redo_project_command_label()
                        .unwrap_or_else(|| "Nothing to redo".to_owned());
                    if ui
                        .add_enabled(
                            graph.undo_project_command_label().is_some(),
                            egui::Button::new("Undo"),
                        )
                        .on_hover_text(&undo_label)
                        .clicked()
                    {
                        graph.undo_project_command();
                    }
                    if ui
                        .add_enabled(
                            graph.redo_project_command_label().is_some(),
                            egui::Button::new("Redo"),
                        )
                        .on_hover_text(&redo_label)
                        .clicked()
                    {
                        graph.redo_project_command();
                    }
                });
                ui.weak("Restores project intent; runtime work items and caches are not restored.");
            });
    }

    fn selected_node_graph_container_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) -> bool {
        let can_collapse = self.selected_node_can_collapse_to_graph_container(graph);
        let selected_node_set = self.collapsible_selected_node_set(graph);
        let can_collapse_selection = selected_node_set.len() > 1;
        let mut collapsed = false;

        egui::CollapsingHeader::new("Subnet")
            .id_salt("houdini_graph_parms_subnet")
            .default_open(true)
            .show(ui, |ui| {
                if ui
                    .add_enabled(can_collapse, egui::Button::new("Collapse to Subnet"))
                    .on_hover_text(
                        "Move the selected node into a new internal graph and leave a typed graph-container node in this graph.",
                    )
                    .clicked()
                {
                    collapsed = self.collapse_selected_node_to_graph_container(graph);
                }
                if ui
                    .add_enabled(
                        can_collapse_selection,
                        egui::Button::new("Collapse Selection to Subnet"),
                    )
                    .on_hover_text(
                        "Move the selected node set into a new internal graph and rewire compatible external crossings through typed subnet boundaries.",
                    )
                    .clicked()
                {
                    collapsed = self.collapse_selected_nodes_to_graph_container(graph);
                }
                if self.selected_nodes.len() > 1 {
                    ui.weak(format!("{} selected node(s)", self.selected_nodes.len()));
                }
                if !can_collapse {
                    ui.weak("Select a non-output, non-container node in the current graph.");
                }
                if let Some(status) = &self.graph_container_status {
                    ui.weak(status);
                }
            });

        collapsed
    }

    fn select_single_node(&mut self, node_index: usize) {
        self.selected_node = node_index;
        self.selected_nodes.clear();
        self.selected_nodes.push(node_index);
    }

    fn set_selected_node_set(&mut self, mut node_indices: Vec<usize>) {
        node_indices.sort_unstable();
        node_indices.dedup();
        if let Some(primary) = node_indices.first().copied() {
            self.selected_node = primary;
            self.selected_nodes = node_indices;
        } else {
            self.selected_nodes.clear();
        }
    }

    fn collapsible_selected_node_set(&self, graph: &GraphDocument) -> Vec<usize> {
        collapsible_node_indices_for_selection(graph, &self.selected_nodes)
    }

    fn selected_node_can_collapse_to_graph_container(&self, graph: &GraphDocument) -> bool {
        graph.nodes.get(self.selected_node).is_some_and(|node| {
            node.parent_graph_id == graph.current_graph_id()
                && !matches!(node.kind, NodeKind::Output | NodeKind::GraphContainer)
        })
    }

    fn selected_node_can_enter_graph_container(&self, graph: &GraphDocument) -> bool {
        graph
            .selected_node_info(self.selected_node)
            .and_then(|info| info.graph_container)
            .is_some_and(|container| container.navigable)
    }

    fn enter_selected_graph_container(&mut self, graph: &mut GraphDocument) -> bool {
        match graph.enter_graph_container_node(self.selected_node) {
            Ok(_) => {
                let node_index = graph
                    .current_graph_node_indices()
                    .first()
                    .copied()
                    .unwrap_or(graph.nodes.len());
                self.select_single_node(node_index);
                self.selected_annotation = None;
                self.selected_edge = None;
                self.node_info_open = true;
                self.show_graph_workbench_pane(GraphWorkbenchPane::Info);
                self.reset_graph_view();
                self.graph_container_status =
                    Some(format!("Entered {}.", graph.current_graph_path()));
                true
            }
            Err(err) => {
                self.graph_container_status = Some(format!(
                    "Enter subnet failed: {}.",
                    graph_navigation_error_message(&err)
                ));
                false
            }
        }
    }

    fn exit_current_graph_to_parent(&mut self, graph: &mut GraphDocument) -> bool {
        let Some(change) = graph.exit_current_graph_to_parent_container() else {
            self.graph_container_status = Some("Already at the top-level graph.".to_owned());
            return false;
        };

        self.select_single_node(change.container_node_index);
        self.selected_annotation = None;
        self.selected_edge = None;
        self.node_info_open = true;
        self.show_graph_workbench_pane(GraphWorkbenchPane::Info);
        self.reset_graph_view();
        self.graph_container_status = Some(format!(
            "Returned to {}.",
            change.navigation.selected_graph.path
        ));
        true
    }

    fn collapse_selected_node_to_graph_container(&mut self, graph: &mut GraphDocument) -> bool {
        let Some(node) = graph.nodes.get(self.selected_node) else {
            self.graph_container_status = Some("Select a node to collapse.".to_owned());
            return false;
        };
        if !self.selected_node_can_collapse_to_graph_container(graph) {
            self.graph_container_status = Some(format!(
                "Cannot collapse selected {} node.",
                node.kind.as_str()
            ));
            return false;
        }

        let original_node_name = node.name.clone();
        let subnet_name = format!("{original_node_name} Subnet");
        match graph.add_graph_container_collapse_manifest_for_node_set(
            subnet_name.clone(),
            &[self.selected_node],
        ) {
            Ok(container_index) => {
                self.set_selected_node_set(vec![container_index]);
                self.selected_annotation = None;
                self.selected_edge = None;
                self.node_info_open = true;
                self.show_graph_workbench_pane(GraphWorkbenchPane::Info);
                self.graph_container_status = Some(format!(
                    "Collapsed {original_node_name} into graph container {subnet_name}."
                ));
                self.pending_frame_selected = true;
                true
            }
            Err(err) => {
                self.graph_container_status = Some(format!(
                    "Subnet collapse failed: {}.",
                    graph_container_collapse_error_message(&err)
                ));
                false
            }
        }
    }

    fn collapse_selected_nodes_to_graph_container(&mut self, graph: &mut GraphDocument) -> bool {
        let selected_nodes = self.collapsible_selected_node_set(graph);
        if selected_nodes.len() <= 1 {
            self.graph_container_status =
                Some("Select multiple non-output nodes to collapse.".to_owned());
            return false;
        }

        let subnet_name = graph
            .nodes
            .get(self.selected_node)
            .map(|node| format!("{} Selection Subnet", node.name))
            .unwrap_or_else(|| "Selection Subnet".to_owned());
        let selected_count = selected_nodes.len();
        match graph.add_graph_container_collapse_manifest_for_node_set(
            subnet_name.clone(),
            &selected_nodes,
        ) {
            Ok(container_index) => {
                self.set_selected_node_set(vec![container_index]);
                self.selected_annotation = None;
                self.selected_edge = None;
                self.node_info_open = true;
                self.show_graph_workbench_pane(GraphWorkbenchPane::Info);
                self.graph_container_status = Some(format!(
                    "Collapsed {selected_count} selected nodes into graph container {subnet_name}."
                ));
                self.pending_frame_selected = true;
                true
            }
            Err(err) => {
                self.graph_container_status = Some(format!(
                    "Selection subnet collapse failed: {}.",
                    graph_container_collapse_error_message(&err)
                ));
                false
            }
        }
    }

    fn selected_node_identity_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        info: &self::model::NodeInfo,
    ) {
        egui::CollapsingHeader::new("Node")
            .id_salt("houdini_graph_parms_node")
            .default_open(true)
            .show(ui, |ui| {
                if self.selected_node < graph.nodes.len() {
                    let mut node_name = graph.nodes[self.selected_node].name.clone();
                    ui.horizontal(|ui| {
                        ui.weak("Name");
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut node_name)
                                    .desired_width(ui.available_width().clamp(128.0, 220.0)),
                            )
                            .changed()
                        {
                            graph.set_node_name(self.selected_node, node_name.clone());
                        }
                    });
                }

                egui::Grid::new("houdini_graph_parms_node_identity")
                    .num_columns(2)
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        parms_row(ui, "Path", &self.selected_node_path(graph));
                        parms_row(ui, "Type", info.kind.as_str());
                        parms_row(ui, "Role", info.role);
                        parms_row(ui, "Data", info.data_kind);
                        ui.weak("Status");
                        ui.colored_label(status_color(ui, info.status), info.status.as_str());
                        ui.end_row();
                        parms_row(ui, "Records", &info.record_count.to_string());
                        parms_row(
                            ui,
                            "Ports",
                            &format!("{} in / {} out", info.input_count, info.output_count),
                        );
                        parms_row(ui, "Time dependent", "No");
                    });
                ui.weak(info.summary);
                if let Some(node) = graph.nodes.get(self.selected_node) {
                    ui.weak(node.info);
                }

                for warning in &info.warnings {
                    ui.colored_label(ui.visuals().warn_fg_color, warning);
                }
            });
    }

    fn selected_node_parameter_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        info: &self::model::NodeInfo,
    ) {
        egui::CollapsingHeader::new("Parameters")
            .id_salt("houdini_graph_parms_parameters")
            .default_open(true)
            .show(ui, |ui| {
                let mut selected_parameter_changed = false;
                if let Some(node) = graph.nodes.get(self.selected_node) {
                    egui::Grid::new("houdini_graph_parms_parameter_metadata")
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            parms_row(ui, "Parameter", node.parameter.name);
                            parms_row(ui, "Kind", node.parameter.kind.as_str());
                            if let Some(rule) = node.parameter.as_attribute_rule() {
                                parms_row(
                                    ui,
                                    "Rule",
                                    &format!(
                                        "{} {} value",
                                        rule.attribute_name,
                                        rule.comparison.as_str()
                                    ),
                                );
                            }
                        });

                    let mut parameter_value = node.parameter.value;
                    selected_parameter_changed = ui
                        .add(
                            Slider::new(&mut parameter_value, node.parameter.range.clone())
                                .text(node.parameter.name),
                        )
                        .on_hover_text(node.parameter.help)
                        .changed();
                    ui.weak(node.parameter.help);
                    if selected_parameter_changed {
                        selected_parameter_changed =
                            graph.set_node_parameter_value(self.selected_node, parameter_value);
                    }
                }

                if selected_parameter_changed {
                    graph.mark_reference_inputs_stale_for_target_index(self.selected_node);
                }

                if let Some(bounds) = &info.bounds {
                    egui::Grid::new("houdini_graph_parms_bounds")
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            parms_row(
                                ui,
                                "Bounds min",
                                &format!("{:.3}, {:.3}", bounds.min.x, bounds.min.y),
                            );
                            parms_row(
                                ui,
                                "Bounds max",
                                &format!("{:.3}, {:.3}", bounds.max.x, bounds.max.y),
                            );
                        });
                }
            });
    }

    fn selected_node_flags_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        info: &self::model::NodeInfo,
    ) {
        if self.selected_node >= graph.nodes.len() {
            return;
        }

        egui::CollapsingHeader::new("Flags")
            .id_salt("houdini_graph_parms_flags")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if self.selected_node < graph.nodes.len() {
                        let mut participates =
                            graph.nodes[self.selected_node].participates_in_output;
                        if ui
                            .re_checkbox(&mut participates, "Display output")
                            .changed()
                        {
                            graph.set_node_output_participation(self.selected_node, participates);
                        }

                        let mut show_comment =
                            graph.nodes[self.selected_node].show_comment_in_network;
                        if ui.re_checkbox(&mut show_comment, "Show comment").changed() {
                            graph.set_node_comment_visibility(self.selected_node, show_comment);
                        }
                    }

                    let mut manual = graph.nodes[self.selected_node].evaluation.manual;
                    if ui.re_checkbox(&mut manual, "Manual cook").changed() {
                        graph.set_node_manual(self.selected_node, manual);
                    }

                    if ui.button("Info").clicked() {
                        self.node_info_open = true;
                        self.active_graph_pane = GraphWorkbenchPane::Info;
                    }
                });
                ui.weak(format!(
                    "Shared graph display context; {} reference consumer(s).",
                    info.reference_consumers.len()
                ));
            });
    }

    fn selected_node_evaluation_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        info: &self::model::NodeInfo,
    ) {
        egui::CollapsingHeader::new("Cook")
            .id_salt("houdini_graph_parms_cook")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("houdini_graph_parms_cook_state")
                    .num_columns(2)
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        ui.weak("State");
                        ui.colored_label(
                            evaluation_color(ui, info.evaluation.state),
                            info.evaluation.state.as_str(),
                        );
                        ui.end_row();

                        parms_row(ui, "Manual", yes_no(info.evaluation.manual));
                        parms_row(
                            ui,
                            "Message",
                            info.evaluation.message.as_deref().unwrap_or("none"),
                        );
                    });
                self.evaluation_controls_ui(ui, graph);
            });
    }

    fn selected_node_operator_settings_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        info: &self::model::NodeInfo,
    ) {
        egui::CollapsingHeader::new("Operator")
            .id_salt("houdini_graph_parms_operator")
            .default_open(true)
            .show(ui, |ui| {
                let mut rendered = false;

                if let Some(null_operator) = &info.null_operator {
                    rendered = true;
                    egui::Grid::new("houdini_graph_parms_null_operator")
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            parms_row(ui, "Convention", null_operator.convention.as_str());
                            parms_row(ui, "Input kind", &format!("{:?}", null_operator.input_kind));
                            parms_row(
                                ui,
                                "Output kind",
                                &format!("{:?}", null_operator.output_kind),
                            );
                            parms_row(
                                ui,
                                "Record identity",
                                yes_no(null_operator.preserves_record_identity),
                            );
                            parms_row(
                                ui,
                                "Provenance",
                                yes_no(null_operator.preserves_source_provenance),
                            );
                        });
                }

                if let Some(reference_input) = &info.reference_input {
                    rendered = true;
                    egui::Grid::new("houdini_graph_parms_reference_input")
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            parms_row(ui, "Reference", &reference_input.readable_path);
                            ui.weak("Status");
                            ui.colored_label(
                                reference_status_color(ui, reference_input.status),
                                reference_input.status.as_str(),
                            );
                            ui.end_row();
                            parms_row(
                                ui,
                                "Hidden transform",
                                yes_no(reference_input.applies_hidden_transform),
                            );
                            parms_row(
                                ui,
                                "Preserves data",
                                yes_no(reference_input.preserves_source_data),
                            );
                            parms_row(ui, "Targets", &reference_input.targets.len().to_string());
                        });
                }

                if let Some(output_operator) = &info.output_operator {
                    rendered = true;
                    egui::Grid::new("houdini_graph_parms_output_operator")
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            parms_row(ui, "Output", output_operator.kind.as_str());
                            parms_row(ui, "Payload", output_operator.semantic_payload.as_str());
                            parms_row(ui, "Command", output_operator.command.as_str());
                            parms_row(
                                ui,
                                "Preferred target",
                                output_operator
                                    .preferred_target
                                    .map(|target| target.as_str())
                                    .unwrap_or("choose target"),
                            );
                            parms_row(
                                ui,
                                "Viewer state",
                                if output_operator.graph_viewport_state_separate {
                                    "target-owned"
                                } else {
                                    "graph-owned"
                                },
                            );
                        });
                    if let Some(rerun_options) = &output_operator.rerun_options {
                        ui.weak(format!(
                            "Rerun options: debug items {}, cubic metadata {}.",
                            yes_no(rerun_options.include_debug_items),
                            yes_no(rerun_options.preserve_native_cubic_metadata)
                        ));
                    }
                    for negotiation in &output_operator.negotiations {
                        ui.weak(format!(
                            "{}: {} - {}",
                            negotiation.target.as_str(),
                            negotiation.mapping.as_str(),
                            negotiation.reason
                        ));
                    }
                }

                if let Some(python_operator) = &info.python_operator {
                    rendered = true;
                    egui::Grid::new("houdini_graph_parms_python_operator")
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            parms_row(ui, "Operator", &python_operator.display_name);
                            parms_row(ui, "Declaration", &python_operator.declaration_id);
                            parms_row(ui, "Version", &python_operator.version);
                            ui.weak("Dependencies");
                            ui.colored_label(
                                python_operator_dependency_color(
                                    ui,
                                    python_operator.dependency_status,
                                ),
                                python_operator.dependency_status.as_str(),
                            );
                            ui.end_row();
                            parms_row(
                                ui,
                                "Requirements",
                                &format_list(&python_operator.requirements),
                            );
                            parms_row(
                                ui,
                                "Cache key",
                                python_operator
                                    .cache_key_summary
                                    .as_deref()
                                    .unwrap_or("none"),
                            );
                        });
                    ui.weak(&python_operator.dependency_summary);
                    if let Some(provenance) = &python_operator.provenance_summary {
                        ui.weak(provenance);
                    }
                    if let Some(failure) = &python_operator.last_failure_summary {
                        ui.colored_label(ui.visuals().error_fg_color, failure);
                    }
                }

                if let Some(asset) = &info.procedural_asset {
                    rendered = true;
                    egui::Grid::new("houdini_graph_parms_asset_operator")
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            parms_row(ui, "Asset", &asset.display_name);
                            parms_row(ui, "Asset id", &asset.asset_id);
                            parms_row(ui, "Version", &asset.instance_version);
                            parms_row(
                                ui,
                                "Current",
                                asset.current_version.as_deref().unwrap_or("missing"),
                            );
                            parms_row(ui, "Unlocked", yes_no(asset.contents_unlocked));
                            parms_row(ui, "Parameters", &format_list(&asset.promoted_parameters));
                            parms_row(ui, "Bindings", &format_bindings(&asset.input_bindings));
                        });
                    if !asset.description.is_empty() {
                        ui.weak(&asset.description);
                    }
                    if let Some(output_summary) = &asset.output_summary {
                        ui.weak(output_summary);
                    }
                }

                if let Some(native_operator) = &info.native_operator {
                    rendered = true;
                    egui::Grid::new("houdini_graph_parms_native_operator")
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            parms_row(ui, "Operator", &native_operator.display_name);
                            parms_row(ui, "Operator id", &native_operator.operator_id);
                            parms_row(ui, "Version", &native_operator.version);
                            parms_row(ui, "Host", &native_operator.host_compatibility_version);
                            ui.weak("Load");
                            ui.colored_label(
                                native_operator_load_status_color(ui, native_operator.load_status),
                                native_operator.load_status.as_str(),
                            );
                            ui.end_row();
                            parms_row(ui, "Inputs", &format_list(&native_operator.inputs));
                            parms_row(ui, "Outputs", &format_list(&native_operator.outputs));
                            parms_row(ui, "Parameters", &format_list(&native_operator.parameters));
                            parms_row(
                                ui,
                                "Capabilities",
                                &format_list(&native_operator.capabilities),
                            );
                            parms_row(
                                ui,
                                "Cache key",
                                native_operator
                                    .cache_key_summary
                                    .as_deref()
                                    .unwrap_or("none"),
                            );
                        });
                    ui.weak(&native_operator.provenance_summary);
                    if let Some(provenance) = &native_operator.output_provenance_summary {
                        ui.weak(provenance);
                    }
                    if let Some(failure) = &native_operator.last_failure_summary {
                        ui.colored_label(ui.visuals().error_fg_color, failure);
                    }
                    self.native_operator_trust_controls_ui(ui, graph, native_operator);
                }

                if !rendered {
                    ui.weak("No specialized operator settings for this node.");
                }
            });
    }

    fn selected_node_path(&self, graph: &GraphDocument) -> String {
        graph
            .readable_node_path(self.selected_node)
            .unwrap_or_else(|| "/obj/main/<none>".to_owned())
    }

    fn graph_workbench_content_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        pane: GraphWorkbenchPane,
    ) {
        match pane {
            GraphWorkbenchPane::Operators => {
                self.operator_strip_ui(ui, graph);
                ui.add_space(8.0);
                self.network_organization_ui(ui, graph);
            }
            GraphWorkbenchPane::Find => {
                self.graph_search_ui(ui, graph);
            }
            GraphWorkbenchPane::Assets => {
                self.asset_gallery_ui(ui, graph);
            }
            GraphWorkbenchPane::Gallery => {
                self.source_gallery_ui(ui, graph, None);
            }
            GraphWorkbenchPane::Parameters => {
                self.selected_node_controls_ui(ui, graph);
            }
            GraphWorkbenchPane::Info => {
                self.graph_workbench_node_info_ui(ui, graph);
                ui.add_space(8.0);
                egui::CollapsingHeader::new("Pipeline Trace")
                    .id_salt("houdini_graph_workbench_pipeline_trace")
                    .default_open(false)
                    .show(ui, |ui| {
                        self.pipeline_trace_ui(ui, graph);
                    });
            }
            GraphWorkbenchPane::Display => {
                self.network_view_options_ui(ui, graph);
            }
            GraphWorkbenchPane::Layers => {
                self.compact_layer_stack_ui(ui, graph);
            }
        }
    }

    fn network_view_options_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        let id = ui.make_persistent_id(NETWORK_DISPLAY_OPTIONS_ID);
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.label("Display Options");
            })
            .body(|ui| {
                let options = &mut graph.network_view;
                ui.horizontal(|ui| {
                    ui.weak("Show Node Ring");
                    egui::ComboBox::from_id_salt("houdini_graph_node_ring_visibility")
                        .selected_text(options.node_ring_visibility.label())
                        .show_ui(ui, |ui| {
                            for visibility in NetworkNodeRingVisibility::ALL {
                                ui.selectable_value(
                                    &mut options.node_ring_visibility,
                                    visibility,
                                    visibility.label(),
                                );
                            }
                        });
                });
                ui.add(
                    Slider::new(&mut options.max_node_name_width, 48.0..=180.0)
                        .text("Maximum Node Name Width"),
                );
                ui.add(
                    Slider::new(&mut options.long_wire_fading, 0.0..=1.0).text("Long Wire Fading"),
                );
                ui.add(
                    Slider::new(&mut options.background_brightness, 0.0..=1.0)
                        .text("Background Brightness"),
                );
                ui.horizontal(|ui| {
                    ui.weak("Grid Spacing");
                    ui.add(DragValue::new(&mut options.grid_spacing).range(0.5..=6.0));
                    ui.weak("x");
                    ui.label("1.0");
                });
                ui.horizontal(|ui| {
                    ui.weak("Comments");
                    egui::ComboBox::from_id_salt("houdini_graph_comment_display_mode")
                        .selected_text(options.comment_display_mode.label())
                        .show_ui(ui, |ui| {
                            for mode in NetworkCommentDisplayMode::ALL {
                                ui.selectable_value(
                                    &mut options.comment_display_mode,
                                    mode,
                                    mode.label(),
                                );
                            }
                        });
                });
                badge_visibility_combo_ui(ui, "Error Badge", &mut options.error_badge);
                badge_visibility_combo_ui(ui, "Warning Badge", &mut options.warning_badge);
                badge_visibility_combo_ui(ui, "Comment Badge", &mut options.comment_badge);
                badge_visibility_combo_ui(
                    ui,
                    "Time Dependent Badge",
                    &mut options.time_dependent_badge,
                );
                badge_visibility_combo_ui(ui, "Asset Lock Badge", &mut options.lock_badge);
                badge_visibility_combo_ui(ui, "Unload Badge", &mut options.unload_badge);
                badge_visibility_combo_ui(ui, "Has Data Badge", &mut options.has_data_badge);
                badge_visibility_combo_ui(ui, "Cached Code Badge", &mut options.cached_code_badge);
                badge_visibility_combo_ui(ui, "Constraint Badge", &mut options.constraint_badge);
                badge_visibility_combo_ui(ui, "Compilable Badge", &mut options.compilable_badge);
            });
    }

    fn network_organization_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        egui::CollapsingHeader::new("Boxes and Notes")
            .id_salt("houdini_graph_network_organization")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.operator_palette_compact_button_ui(
                        ui,
                        graph,
                        OperatorPaletteAction::AddNetworkBox,
                    );
                    self.operator_palette_compact_button_ui(
                        ui,
                        graph,
                        OperatorPaletteAction::AddStickyNote,
                    );
                });

                let node_names = graph
                    .nodes
                    .iter()
                    .map(|node| (node.node_id.clone(), node.name.clone()))
                    .collect::<Vec<_>>();
                let annotation_indices = graph.current_graph_annotation_indices();
                for annotation_index in annotation_indices.iter().copied() {
                    let mut resize_to_contents = false;
                    let Some(annotation) = graph.annotations.get(annotation_index).cloned() else {
                        continue;
                    };

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.weak(annotation.kind.as_str());
                        let mut collapsed = annotation.collapsed;
                        if ui.re_checkbox(&mut collapsed, "Collapsed").changed() {
                            graph.set_annotation_collapsed(annotation_index, collapsed);
                        }
                    });

                    let mut title = annotation.title.clone();
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut title)
                                .desired_width(ui.available_width().max(80.0))
                                .hint_text("title"),
                        )
                        .changed()
                    {
                        graph.set_annotation_title(annotation_index, title);
                    }

                    if annotation.kind == GraphAnnotationKind::StickyNote {
                        let mut text = annotation.text.clone();
                        if ui
                            .add(
                                egui::TextEdit::multiline(&mut text)
                                    .desired_rows(2)
                                    .hint_text("note"),
                            )
                            .changed()
                        {
                            graph.set_annotation_text(annotation_index, text);
                        }
                    } else {
                        let member_names = annotation
                            .member_node_ids
                            .iter()
                            .filter_map(|member_id| {
                                node_names
                                    .iter()
                                    .find(|(node_id, _)| node_id == member_id)
                                    .map(|(_, name)| name.as_str())
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        ui.weak(format!(
                            "{} member node{}",
                            annotation.member_node_ids.len(),
                            if annotation.member_node_ids.len() == 1 {
                                ""
                            } else {
                                "s"
                            }
                        ));
                        if !member_names.is_empty() {
                            ui.weak(member_names);
                        }
                        if ui.button("Resize to Contents").clicked() {
                            resize_to_contents = true;
                        }
                    }

                    let mut size = annotation.size;
                    ui.horizontal(|ui| {
                        ui.weak("Size");
                        ui.add(DragValue::new(&mut size.x).speed(0.01).range(0.08..=0.95));
                        ui.add(DragValue::new(&mut size.y).speed(0.01).range(0.08..=0.95));
                    });
                    if size != annotation.size {
                        graph.set_annotation_size(annotation_index, size);
                    }

                    let mut next_position = annotation.position;
                    ui.horizontal(|ui| {
                        ui.weak("Pos");
                        ui.add(DragValue::new(&mut next_position.x).speed(0.01));
                        ui.add(DragValue::new(&mut next_position.y).speed(0.01));
                    });
                    if next_position != annotation.position {
                        graph.translate_annotation(
                            annotation_index,
                            GraphPoint {
                                x: next_position.x - annotation.position.x,
                                y: next_position.y - annotation.position.y,
                            },
                        );
                    }
                    if resize_to_contents {
                        graph.resize_network_box_to_contents(annotation_index);
                    }
                }

                if annotation_indices.is_empty() {
                    ui.weak("No graph annotations.");
                }
            });
    }

    fn graph_workbench_node_info_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.horizontal_wrapped(|ui| {
            ui.re_checkbox(&mut self.node_info_pinned, "Pin");
            ui.re_checkbox(
                &mut self.node_info_refresh_automatically,
                "Refresh automatically",
            );
            if ui.button("Refresh").clicked() {
                self.node_info_open = true;
            }
            if self.node_info_open && !self.node_info_pinned && ui.button("Close").clicked() {
                self.node_info_open = false;
            }
        });

        if !self.node_info_open {
            ui.weak("Node info hidden; select a node or pin this panel.");
            return;
        }

        self.graph_workbench_graph_diagnostics_ui(ui, graph);

        let Some(info) = graph.selected_node_info(self.selected_node) else {
            ui.weak("Select a node to inspect graph-owned metadata.");
            return;
        };

        ui.horizontal_wrapped(|ui| {
            ui.colored_label(status_color(ui, info.status), info.status.as_str());
            ui.separator();
            ui.label(info.kind.as_str());
            ui.weak(info.role);
        });
        ui.weak(format!(
            "{} record(s), {} input(s), {} output(s)",
            info.record_count, info.input_count, info.output_count
        ));
        ui.weak(format!(
            "Connections {} in, {} out",
            info.data_flow.incoming_edge_count, info.data_flow.outgoing_edge_count
        ));
        ui.weak(format!("Graph {}", info.graph_location.graph_path));
        ui.weak(format!("Node path {}", info.graph_location.node_path));
        ui.weak(if info.graph_location.name_is_unique_in_graph() {
            "Name unique in current graph".to_owned()
        } else {
            format!(
                "Name shared by {} nodes in current graph",
                info.graph_location.name_collision_count
            )
        });
        ui.weak(info.summary);
        ui.horizontal_wrapped(|ui| {
            ui.weak("Time dependent");
            ui.label("No");
        });

        for warning in &info.warnings {
            ui.colored_label(ui.visuals().warn_fg_color, warning);
        }
        for diagnostic in &info.data_flow.diagnostics {
            ui.colored_label(
                ui.visuals().warn_fg_color,
                format!(
                    "Connection {}: {}",
                    diagnostic.status.as_str(),
                    diagnostic.readable_path
                ),
            )
            .on_hover_text(&diagnostic.message);
        }

        self.selected_node_comment_ui(ui, graph);
        self.generated_node_binding_controls_ui(ui, graph);
        self.graph_container_controls_ui(ui, graph, &info);

        ui.re_checkbox(&mut self.node_info_show_additional, "Show additional info");
        if self.node_info_show_additional {
            self.graph_workbench_additional_node_info_ui(ui, &info);
        }

        ui.re_checkbox(&mut self.node_info_show_debug, "Show debug");
        if self.node_info_show_debug {
            egui::ScrollArea::vertical()
                .id_salt("houdini_graph_workbench_node_info_debug_scroll")
                .max_height(180.0)
                .show(ui, |ui| {
                    self.node_info_ui(ui, graph);
                });
        }
    }

    fn graph_workbench_graph_diagnostics_ui(&self, ui: &mut Ui, graph: &GraphDocument) {
        let diagnostics = graph.current_graph_data_flow_edge_diagnostics();
        if diagnostics.is_empty() {
            ui.weak(format!(
                "Graph diagnostics none in {}",
                graph.current_graph_path()
            ));
            return;
        }

        ui.colored_label(
            ui.visuals().warn_fg_color,
            format!(
                "Graph diagnostics {} in {}",
                diagnostics.len(),
                graph.current_graph_path()
            ),
        );
        for diagnostic in diagnostics {
            ui.colored_label(
                ui.visuals().warn_fg_color,
                format!(
                    "Connection {}: {}",
                    diagnostic.status.as_str(),
                    diagnostic.readable_path
                ),
            )
            .on_hover_text(&diagnostic.message);
        }
    }

    fn graph_container_controls_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        info: &self::model::NodeInfo,
    ) {
        let Some(container) = &info.graph_container else {
            return;
        };

        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.colored_label(ui.visuals().selection.stroke.color, "Graph container");
            ui.label(container.status.as_str());
            if let Some(path) = &container.internal_graph_path {
                ui.weak(path);
            } else {
                ui.colored_label(ui.visuals().warn_fg_color, "missing internal graph");
            }

            if container.navigable {
                let enter_clicked = ui
                    .button("Enter Subnet")
                    .on_hover_text("Open this container's internal graph. Shortcut: Enter.")
                    .clicked();
                if enter_clicked {
                    self.enter_selected_graph_container(graph);
                }
            }
        });
    }

    fn generated_node_binding_controls_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        let Some(generated) = graph
            .nodes
            .get(self.selected_node)
            .and_then(|node| node.generated)
        else {
            return;
        };

        ui.horizontal_wrapped(|ui| {
            ui.weak("Layer binding");
            ui.label(generated.binding_state.as_str())
                .on_hover_text(generated.binding_state.description());

            if generated.binding_state != GeneratedNodeBindingState::Adopted
                && ui.button("Adopt").clicked()
            {
                graph.set_generated_node_binding_state(
                    self.selected_node,
                    GeneratedNodeBindingState::Adopted,
                );
            }
            if generated.binding_state != GeneratedNodeBindingState::Managed
                && ui.button("Manage").clicked()
            {
                graph.set_generated_node_binding_state(
                    self.selected_node,
                    GeneratedNodeBindingState::Managed,
                );
            }
            if generated.binding_state != GeneratedNodeBindingState::Unbound
                && ui.button("Unbind").clicked()
            {
                graph.set_generated_node_binding_state(
                    self.selected_node,
                    GeneratedNodeBindingState::Unbound,
                );
            }
        });
    }

    fn selected_node_comment_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        let mut show_comment = {
            let Some(node) = graph.nodes.get_mut(self.selected_node) else {
                return;
            };

            ui.separator();
            ui.horizontal(|ui| {
                ui.colored_label(ui.visuals().selection.stroke.color, "Comment");
                if node.comment.trim().is_empty() {
                    ui.weak("empty");
                }
            });
            ui.add(
                egui::TextEdit::multiline(&mut node.comment)
                    .desired_rows(2)
                    .hint_text("Click to enter a comment"),
            );
            node.show_comment_in_network
        };
        if ui
            .re_checkbox(&mut show_comment, "Show comment in Network")
            .changed()
        {
            graph.set_node_comment_visibility(self.selected_node, show_comment);
        }
    }

    fn graph_workbench_additional_node_info_ui(
        &mut self,
        ui: &mut Ui,
        info: &self::model::NodeInfo,
    ) {
        if let Some(contract) = &info.coordinate_contract {
            ui.horizontal_wrapped(|ui| {
                ui.weak("Substrate");
                ui.label(&contract.substrate_id);
                ui.weak(format!("{}x{}", contract.width, contract.height));
                ui.weak(format!("{:?}/{:?}", contract.origin, contract.y_axis));
            });
        }

        if let Some(raster) = &info.substrate_raster {
            ui.horizontal_wrapped(|ui| {
                ui.weak("Raster");
                ui.label(&raster.display_name);
                ui.weak(raster.format_summary());
                ui.weak(format!("{} bytes", raster.byte_len()));
                if let Some(source_path) = &raster.source_path {
                    ui.weak(source_path);
                }
            });
        }

        if let Some(reference_input) = &info.reference_input {
            ui.horizontal_wrapped(|ui| {
                ui.weak("Reference");
                ui.colored_label(
                    status_color(ui, info.status),
                    reference_input.status.as_str(),
                );
                ui.label(&reference_input.readable_path);
            });
            for target in &reference_input.targets {
                if let Some(diagnostic) = &target.diagnostic {
                    ui.colored_label(
                        ui.visuals().warn_fg_color,
                        format!("{}: {diagnostic}", target.readable_path),
                    );
                }
            }
        }

        if !info.reference_consumers.is_empty() {
            ui.colored_label(
                ui.visuals().warn_fg_color,
                format!(
                    "{} reference consumer(s) depend on this output",
                    info.reference_consumers.len()
                ),
            );
        }

        if let Some(warning) = &info.reference_output_warning {
            ui.colored_label(
                ui.visuals().warn_fg_color,
                format!(
                    "Changing or deleting {}:{} affects {} reference(s)",
                    warning.target_node_path,
                    warning.output_name,
                    warning.affected_references.len()
                ),
            );
        }

        if let Some(output_operator) = &info.output_operator {
            ui.horizontal_wrapped(|ui| {
                ui.weak("Output");
                ui.label(output_operator.kind.as_str());
                ui.weak("->");
                ui.label(
                    output_operator
                        .preferred_target
                        .map(|target| target.as_str())
                        .unwrap_or("choose target"),
                );
            });
            for negotiation in &output_operator.negotiations {
                if negotiation.reason != "native mapping available" {
                    ui.weak(format!(
                        "{}: {}",
                        negotiation.target.as_str(),
                        negotiation.reason
                    ));
                }
            }
        }
    }

    fn output_summary_ui(&self, ui: &mut Ui, graph: &GraphDocument) {
        let export_polyline_points = graph.prepared_export_point_count();
        let feasibility = graph.render_feasibility_summary();

        egui::Grid::new("houdini_graph_output_summary")
            .num_columns(2)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.weak("Visible items");
                ui.label(graph.visible_output_count().to_string());
                ui.end_row();

                ui.weak("Prepared points");
                ui.label(export_polyline_points.to_string());
                ui.end_row();

                ui.weak("Native primitives");
                ui.label(feasibility.native_viewer_primitive_count.to_string());
                ui.end_row();

                ui.weak("Graph points");
                ui.label(feasibility.graph_owned_point_count.to_string());
                ui.end_row();

                ui.weak("Boundary debug points");
                ui.label(feasibility.prepared_boundary_debug_point_count.to_string());
                ui.end_row();

                ui.weak("Cubic segments");
                ui.label(graph.export_segments().to_string());
                ui.end_row();
            });
    }

    fn source_summary_ui(&mut self, ui: &mut Ui, graph: &GraphDocument) {
        egui::Grid::new("houdini_graph_source_summary")
            .num_columns(2)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.weak("Source");
                ui.label(format!(
                    "{} ({} matching, {} visible)",
                    graph.source.as_str(),
                    graph.source.matching_entity_count,
                    graph.source.visible_data_result_count
                ));
                ui.end_row();

                ui.weak("Provenance");
                ui.label(graph.source.metadata.provenance.as_str());
                ui.end_row();

                if let Some(source_path) = &graph.source.source_path {
                    ui.weak("Path");
                    ui.label(source_path);
                    ui.end_row();
                }

                if let Some(import_error) = &graph.source.import_error {
                    ui.weak("Source error");
                    ui.colored_label(ui.visuals().error_fg_color, import_error);
                    ui.end_row();
                }

                ui.weak("Geometry");
                ui.label(format!(
                    "{} polygons, {} cubic Beziers",
                    graph.polygon_count(),
                    graph.cubic_bezier_count()
                ));
                ui.end_row();

                ui.weak("Points");
                ui.label(format!(
                    "{} polygon vertices, {} cubic controls",
                    graph.polygon_vertex_count(),
                    graph.cubic_control_point_count()
                ));
                ui.end_row();
            });

        self.source_metadata_ui(ui, &graph.source.metadata, "project_source");
    }

    fn sync_python_environment_inputs(&mut self, graph: &GraphDocument) {
        if self.python_uv_executable_path.is_empty() {
            self.python_uv_executable_path = graph
                .python_environment
                .resolver
                .executable_path
                .clone()
                .unwrap_or_default();
        }
        if self.python_existing_environment_path.is_empty() {
            self.python_existing_environment_path = graph
                .python_environment
                .paths
                .existing_environment_path
                .clone()
                .unwrap_or_default();
        }
        if self.python_create_environment_path.is_empty() {
            self.python_create_environment_path = graph
                .python_environment
                .paths
                .create_environment_path
                .clone();
        }
    }

    fn parquet_import_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.horizontal(|ui| {
            if ui.button("Load Sample").clicked() {
                self.import_parquet_path(graph, sample_parquet_path());
            }

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Import Parquet...").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Parquet", &["parquet"])
                    .pick_file()
            {
                self.import_parquet_path(graph, path);
            }

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Import CSV/TSV...").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("CSV/TSV", &["csv", "tsv"])
                    .pick_file()
            {
                self.import_csv_path(graph, path);
            }

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Import GeoJSON...").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("GeoJSON", &["geojson", "json"])
                    .pick_file()
            {
                self.import_geojson_path(graph, path);
            }
        });
        if let Some(path) = &self.last_parquet_path {
            ui.weak(path);
        }
        if let Some(status) = &self.parquet_status {
            ui.weak(status);
        }
    }

    fn python_environment_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        self.sync_python_environment_inputs(graph);
        let environment = &graph.python_environment;
        ui.add_space(6.0);
        ui.strong("Python Environment");
        egui::Grid::new("houdini_python_environment_status")
            .num_columns(2)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.weak("Status");
                ui.colored_label(
                    python_environment_status_color(ui, environment.lock_status),
                    environment.lock_status.as_str(),
                );
                ui.end_row();

                ui.weak("Python");
                ui.label(&environment.python_version_requirement);
                ui.end_row();

                ui.weak("Requirements");
                ui.label(environment.requirements_source.as_str());
                ui.end_row();

                ui.weak("Resolver");
                ui.label(format!(
                    "{} {}",
                    environment.resolver.tool,
                    environment
                        .resolver
                        .version
                        .as_deref()
                        .unwrap_or("version pending")
                ));
                ui.end_row();

                ui.weak("uv executable");
                ui.label(
                    environment
                        .resolver
                        .executable_path
                        .as_deref()
                        .unwrap_or("not configured"),
                );
                ui.end_row();

                ui.weak("Environment mode");
                ui.label(environment.paths.mode.as_str());
                ui.end_row();

                ui.weak("Lock");
                ui.label(environment.lock_digest.as_deref().unwrap_or("none"));
                ui.end_row();

                ui.weak("Environment path");
                ui.label(
                    environment
                        .environment_path
                        .as_deref()
                        .unwrap_or("not created"),
                );
                ui.end_row();

                ui.weak("Existing env");
                ui.label(
                    environment
                        .paths
                        .existing_environment_path
                        .as_deref()
                        .unwrap_or("none"),
                );
                ui.end_row();

                ui.weak("Create target");
                ui.label(&environment.paths.create_environment_path);
                ui.end_row();

                ui.weak("Packages");
                ui.label(environment.dependency_health.package_count.to_string());
                ui.end_row();

                ui.weak("Health");
                ui.label(
                    if environment.lock_status != PythonEnvironmentStatus::Ready {
                        "not checked"
                    } else if environment.dependency_health.is_healthy() {
                        "healthy"
                    } else {
                        "needs attention"
                    },
                );
                ui.end_row();

                if !environment.dependency_health.missing_packages.is_empty() {
                    ui.weak("Missing packages");
                    ui.label(format_list(&environment.dependency_health.missing_packages));
                    ui.end_row();
                }

                if !environment.dependency_health.conflicts.is_empty() {
                    ui.weak("Conflicts");
                    ui.label(format_list(&environment.dependency_health.conflicts));
                    ui.end_row();
                }

                if !environment.dependency_health.failed_imports.is_empty() {
                    ui.weak("Failed imports");
                    ui.label(format_list(&environment.dependency_health.failed_imports));
                    ui.end_row();
                }
            });
        ui.add_space(4.0);
        egui::Grid::new("houdini_python_environment_paths")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.weak("uv path");
                ui.text_edit_singleline(&mut self.python_uv_executable_path);
                ui.end_row();

                ui.weak("Existing env");
                ui.text_edit_singleline(&mut self.python_existing_environment_path);
                ui.end_row();

                ui.weak("Create env at");
                ui.text_edit_singleline(&mut self.python_create_environment_path);
                ui.end_row();
            });
        ui.horizontal(|ui| {
            if ui.button("Apply uv path").clicked() {
                graph.configure_python_uv_executable_path(&self.python_uv_executable_path);
            }
            if ui.button("Use existing env").clicked() {
                graph.select_existing_python_environment(&self.python_existing_environment_path);
            }
            if ui.button("Use create target").clicked() {
                graph.select_python_environment_create_path(&self.python_create_environment_path);
            }
        });
        ui.weak(graph.python_environment.status_summary());
        if graph.python_environment.lock_status == PythonEnvironmentStatus::Failed {
            ui.weak("Resolve or repair the project environment before running Python operators.");
        }

        if let Some(plan) = &graph.python_environment.resolve_state.last_plan {
            ui.weak(format!(
                "Resolve plan: {} requirement(s), {}",
                plan.unique_requirement_count(),
                plan.conflict_summary()
            ));
            for conflict in &plan.conflicts {
                ui.colored_label(ui.visuals().warn_fg_color, conflict.summary());
            }
        }

        let resolving = graph.python_environment.lock_status == PythonEnvironmentStatus::Resolving;
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!resolving, egui::Button::new("Resolve with uv"))
                .clicked()
            {
                graph.begin_python_environment_resolve(
                    PythonEnvironmentResolveTrigger::ExplicitUserAction,
                );
            }
            if ui
                .add_enabled(resolving, egui::Button::new("Cancel resolve"))
                .clicked()
            {
                graph.cancel_python_environment_resolve();
            }
        });
    }

    fn render_benchmark_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.horizontal(|ui| {
            ui.label("Benchmark curves");
            ui.add(
                DragValue::new(&mut self.benchmark_curve_count)
                    .range(0..=50_000)
                    .speed(100),
            );
            ui.label("polygons");
            ui.add(
                DragValue::new(&mut self.benchmark_polygon_count)
                    .range(0..=20_000)
                    .speed(25),
            );

            if ui.button("Load Benchmark").clicked() {
                let report = graph.load_synthetic_render_benchmark(
                    self.benchmark_curve_count,
                    self.benchmark_polygon_count,
                );
                self.benchmark_status = Some(format!(
                    "Loaded {} native cubics and {} polygons; {} prepared boundary/debug points are derived only at viewer/export edges.",
                    report.native_cubic_bezier_count,
                    report.polygon_count,
                    report.prepared_boundary_debug_point_count
                ));
            }
        });

        if let Some(status) = &self.benchmark_status {
            ui.weak(status);
        }
    }

    fn graph_document_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.horizontal(|ui| {
            if ui.button("Load Malware Starter").clicked() {
                *graph = GraphDocument::malware_starter();
                self.select_single_node(0);
                self.selected_annotation = None;
                self.node_info_open = true;
                self.show_graph_workbench_pane(GraphWorkbenchPane::Info);
                self.graph_document_status =
                    Some("Loaded malware byteplot starter graph.".to_owned());
            }

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Save Graph...").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Houdini Graph", &["json"])
                    .set_file_name("houdini-graph.json")
                    .save_file()
            {
                match graph.save_sidecar_json(&path) {
                    Ok(()) => {
                        self.graph_document_status =
                            Some(format!("Saved graph document: {}", path.display()));
                    }
                    Err(err) => {
                        self.graph_document_status = Some(format!("Graph save failed: {err}"));
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Load Graph...").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Houdini Graph", &["json"])
                    .pick_file()
            {
                match graph.load_sidecar_json(&path) {
                    Ok(()) => {
                        self.graph_document_status =
                            Some(format!("Loaded graph document: {}", path.display()));
                    }
                    Err(err) => {
                        self.graph_document_status = Some(format!("Graph load failed: {err}"));
                    }
                }
            }
        });
        if let Some(status) = &self.graph_document_status {
            ui.weak(status);
        }
    }

    fn asset_authoring_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.add_space(6.0);
        ui.strong("Asset");
        egui::Grid::new("houdini_create_asset")
            .num_columns(2)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.weak("Name");
                ui.text_edit_singleline(&mut self.asset_name);
                ui.end_row();

                ui.weak("Description");
                ui.text_edit_singleline(&mut self.asset_description);
                ui.end_row();

                ui.weak("Help");
                ui.text_edit_singleline(&mut self.asset_help);
                ui.end_row();
            });
        if ui.button("Create Asset from Graph").clicked() {
            let draft = graph.create_asset_draft_from_graph(
                self.asset_name.trim(),
                self.asset_description.trim(),
                self.asset_help.trim(),
            );
            let asset_id = graph.commit_asset_draft(draft);
            self.asset_status = Some(format!("Created project asset: {asset_id}"));
            self.show_graph_workbench_pane(GraphWorkbenchPane::Assets);
        }
        let selected_graph_container =
            self.selected_node_can_create_asset_from_graph_container(graph);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    selected_graph_container,
                    egui::Button::new("Create Asset from Subnet"),
                )
                .on_hover_text("Create a project-local asset definition from the selected graph container boundary.")
                .clicked()
            {
                self.create_asset_from_selected_graph_container(graph);
            }
            if !selected_graph_container {
                ui.weak("Select a subnet to create an asset from its boundary.");
            }
        });

        ui.add_space(6.0);
        ui.strong("Selected Asset");
        let selected_asset = graph
            .selected_node_info(self.selected_node)
            .and_then(|info| info.procedural_asset);
        if let Some(asset) = selected_asset {
            egui::Grid::new("houdini_selected_asset_definition")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    ui.weak("Definition");
                    ui.label(format!("{} ({})", asset.display_name, asset.asset_id));
                    ui.end_row();

                    ui.weak("Version");
                    ui.label(format!(
                        "instance {} / current {} / {}",
                        asset.instance_version,
                        asset.current_version.as_deref().unwrap_or("missing"),
                        asset.version_status.as_str()
                    ));
                    ui.end_row();

                    ui.weak("State");
                    ui.label(if asset.contents_unlocked {
                        "unlocked"
                    } else {
                        "matched"
                    });
                    ui.end_row();
                });
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        asset.can_save_definition,
                        egui::Button::new("Save Asset Definition"),
                    )
                    .on_hover_text(
                        "Write compatible unlocked asset edits back to the project-local definition.",
                    )
                    .clicked()
                {
                    self.asset_status = Some(
                        graph
                            .save_procedural_asset_definition(self.selected_node)
                            .map_or_else(
                                || "Selected asset definition was not saved.".to_owned(),
                                |result| {
                                    format!(
                                        "Saved {} from {} to {}; {} matched instance(s) now have an explicit upgrade available.",
                                        result.asset_id,
                                        result.previous_version,
                                        result.new_version,
                                        result.update_available_instance_count
                                    )
                                },
                            ),
                    );
                }
                if !asset.can_save_definition {
                    ui.weak("Unlock a procedural asset instance before saving its definition.");
                }
                if ui
                    .add_enabled(
                        asset.can_match_definition,
                        egui::Button::new("Match"),
                    )
                    .on_hover_text(
                        "Relock local contents to the pinned asset definition without changing the pinned version.",
                    )
                    .clicked()
                {
                    self.match_selected_asset_definition(graph);
                }
                if ui
                    .add_enabled(
                        asset.can_upgrade_to_current_definition,
                        egui::Button::new("Upgrade"),
                    )
                    .on_hover_text(
                        "Upgrade this asset instance to the current project-local definition version.",
                    )
                    .clicked()
                {
                    self.upgrade_selected_asset_to_current_definition(graph);
                }
            });
        } else {
            ui.weak("Select a procedural asset node to save or update its definition.");
        }
        if let Some(status) = &self.asset_status {
            ui.weak(status);
        }
    }

    fn asset_gallery_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.strong("Asset Gallery");
        ui.horizontal(|ui| {
            ui.weak("Find");
            ui.add(
                egui::TextEdit::singleline(&mut self.asset_gallery_filter)
                    .hint_text("asset, label, graph, or usage"),
            );
        });
        let entries = self.filtered_asset_gallery_entries(graph);
        if entries.is_empty() {
            if self.asset_gallery_filter.trim().is_empty() {
                ui.weak("No project assets.");
            } else {
                ui.weak("No project assets match the current filter.");
            }
            return;
        }

        for entry in entries {
            let header_label = format!(
                "{} ({})",
                entry.display_name,
                entry.version.as_deref().unwrap_or("missing")
            );
            egui::CollapsingHeader::new(header_label)
                .id_salt(format!("houdini_asset_gallery_{}", entry.asset_id))
                .default_open(entry.missing_declaration || !entry.usages.is_empty())
                .show(ui, |ui| {
                    egui::Grid::new(format!("houdini_asset_gallery_meta_{}", entry.asset_id))
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            ui.weak("Asset id");
                            ui.monospace(&entry.asset_id);
                            ui.end_row();

                            ui.weak("Interface");
                            ui.label(format!(
                                "{} input(s), {} output(s), {} promoted parameter(s)",
                                entry.input_count,
                                entry.output_count,
                                entry.promoted_parameter_count
                            ));
                            ui.end_row();

                            ui.weak("Labels");
                            ui.label(format_list(&entry.labels));
                            ui.end_row();

                            ui.weak("Wrapped graph");
                            ui.label(entry.wrapped_graph_id.as_deref().unwrap_or("missing"));
                            ui.end_row();
                        });
                    if !entry.description.is_empty() {
                        ui.weak(&entry.description);
                    }
                    ui.add_space(4.0);
                    ui.strong(format!("Usage ({})", entry.usages.len()));
                    if entry.usages.is_empty() {
                        ui.weak("No instances in this project.");
                    } else {
                        egui::Grid::new(format!("houdini_asset_gallery_usage_{}", entry.asset_id))
                            .num_columns(6)
                            .spacing([10.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.strong("Node");
                                ui.strong("Version");
                                ui.strong("State");
                                ui.strong("");
                                ui.strong("");
                                ui.strong("");
                                ui.end_row();

                                for group in asset_usage_graph_groups(&entry.usages) {
                                    ui.strong(group.graph_path);
                                    for _ in 0..5 {
                                        ui.strong("");
                                    }
                                    ui.end_row();

                                    for usage in group.usages {
                                        ui.label(&usage.node_name).on_hover_text(&usage.node_path);
                                        ui.label(format!(
                                            "{} / {}",
                                            usage.instance_version,
                                            usage.version_status.as_str()
                                        ));
                                        ui.label(if usage.contents_unlocked {
                                            "unlocked"
                                        } else {
                                            "matched"
                                        });
                                        if ui.button("Go").clicked() {
                                            self.jump_to_graph_node(
                                                graph,
                                                usage.node_index,
                                                &usage.graph_id,
                                            );
                                        }
                                        if ui
                                            .add_enabled(
                                                usage.can_match_definition,
                                                egui::Button::new("Match"),
                                            )
                                            .on_hover_text(
                                                "Relock local contents to the pinned definition.",
                                            )
                                            .clicked()
                                        {
                                            self.match_asset_definition(graph, usage.node_index);
                                        }
                                        if ui
                                            .add_enabled(
                                                usage.can_upgrade_to_current_definition,
                                                egui::Button::new("Upgrade"),
                                            )
                                            .on_hover_text(
                                                "Upgrade this instance to the current definition.",
                                            )
                                            .clicked()
                                        {
                                            self.upgrade_asset_to_current_definition(
                                                graph,
                                                usage.node_index,
                                            );
                                        }
                                        ui.end_row();
                                    }
                                }
                            });
                    }
                });
        }
    }

    fn source_gallery_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        viewer_ctx: Option<&ViewerContext<'_>>,
    ) {
        ui.strong("Source Gallery");
        ui.add_space(4.0);

        egui::Grid::new("houdini_source_gallery_entry_controls")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                ui.weak("Source");
                ui.add(
                    egui::TextEdit::singleline(&mut self.source_gallery_location)
                        .hint_text("path, file URL, or manifest URL")
                        .desired_width(340.0),
                );
                ui.end_row();

                ui.weak("Manifest JSON");
                ui.add(
                    egui::TextEdit::multiline(&mut self.source_gallery_manifest_json)
                        .hint_text(r#"{"items":["https://example.test/frame.png"]}"#)
                        .desired_rows(4)
                        .desired_width(340.0),
                );
                ui.end_row();
            });

        ui.horizontal(|ui| {
            if ui.button("Index").clicked() {
                self.rebuild_source_gallery_index();
            }
            if ui.button("Clear").clicked() {
                self.source_gallery_index = None;
                self.source_gallery_selected_id = None;
                self.source_gallery_checked_ids.clear();
                self.source_gallery_status = None;
            }
            if let Some(status) = &self.source_gallery_status {
                ui.weak(status);
            }
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.weak("Find");
            ui.add(
                egui::TextEdit::singleline(&mut self.source_gallery_filter)
                    .hint_text("name, kind, status, format, or locator"),
            );
        });
        ui.add_space(6.0);

        let Some(index) = self.source_gallery_index.clone() else {
            ui.weak("No source gallery indexed.");
            return;
        };

        for warning in &index.warnings {
            ui.weak(warning);
        }

        let filtered_items = source_gallery_filtered_items(&index, &self.source_gallery_filter)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        if filtered_items.is_empty() {
            ui.weak("No gallery items match the current filter.");
            return;
        }

        ui.horizontal(|ui| {
            ui.weak(format!(
                "{} of {} item(s)",
                filtered_items.len(),
                index.items.len()
            ));
            if index.truncated {
                ui.weak(format!("limited to {}", index.limit));
            }
        });

        egui::Grid::new("houdini_source_gallery_thumbnail_grid")
            .num_columns(3)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                for (item_index, item) in filtered_items.iter().enumerate() {
                    let selected =
                        self.source_gallery_selected_id.as_deref() == Some(item.stable_id.as_str());
                    ui.allocate_ui_with_layout(
                        egui::vec2(172.0, 94.0),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            if ui
                                .selectable_label(selected, source_gallery_thumbnail_label(item))
                                .clicked()
                            {
                                self.source_gallery_selected_id = Some(item.stable_id.clone());
                            }
                            let mut checked = self
                                .source_gallery_checked_ids
                                .iter()
                                .any(|id| id == &item.stable_id);
                            if ui.checkbox(&mut checked, "Collection").changed() {
                                self.set_source_gallery_item_checked(&item.stable_id, checked);
                            }
                            ui.label(&item.display_name);
                            ui.weak(source_gallery_tile_detail(item));
                        },
                    );
                    if (item_index + 1) % 3 == 0 {
                        ui.end_row();
                    }
                }
            });

        let selected_id = self
            .source_gallery_selected_id
            .as_deref()
            .or_else(|| filtered_items.first().map(|item| item.stable_id.as_str()));
        if let Some(selected_item) = source_gallery_selected_item(&index, selected_id) {
            let selected_item = selected_item.clone();
            let checked_items = filtered_items
                .iter()
                .filter(|item| {
                    self.source_gallery_checked_ids
                        .iter()
                        .any(|id| id == &item.stable_id)
                })
                .cloned()
                .collect::<Vec<_>>();
            ui.add_space(8.0);
            self.source_gallery_actions_ui(ui, graph, &selected_item, &checked_items, viewer_ctx);
            ui.add_space(4.0);
            self.source_gallery_metadata_ui(ui, &selected_item);
        }
    }

    fn source_gallery_actions_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        item: &SourceGalleryItem,
        checked_items: &[SourceGalleryItem],
        viewer_ctx: Option<&ViewerContext<'_>>,
    ) {
        let action = item.open_action_report();
        ui.horizontal(|ui| {
            if ui
                .add_enabled(action.enabled, egui::Button::new(action.label))
                .on_hover_text(&action.status)
                .clicked()
            {
                self.execute_source_gallery_open_action(item, viewer_ctx);
            }
            ui.weak(&action.status);
        });
        ui.horizontal(|ui| {
            if ui.button("Create Source Node").clicked() {
                let node_index = graph.add_source_gallery_item_node(item);
                self.selected_node = node_index;
                self.selected_nodes = vec![node_index];
                self.source_gallery_status =
                    Some(format!("Created source node for {}.", item.display_name));
            }

            let collection_enabled = checked_items.len() >= 2;
            if ui
                .add_enabled(
                    collection_enabled,
                    egui::Button::new("Create Source Collection"),
                )
                .on_hover_text("Create a graph-owned source collection from checked gallery items.")
                .clicked()
                && let Some(node_index) = graph.add_source_gallery_collection_node(checked_items)
            {
                self.selected_node = node_index;
                self.selected_nodes = vec![node_index];
                self.source_gallery_status = Some(format!(
                    "Created source collection with {} item(s).",
                    checked_items.len()
                ));
            }
            if !collection_enabled {
                ui.weak("Check at least two gallery items for a collection.");
            }
        });
    }

    fn set_source_gallery_item_checked(&mut self, stable_id: &str, checked: bool) {
        if checked {
            if !self
                .source_gallery_checked_ids
                .iter()
                .any(|id| id == stable_id)
            {
                self.source_gallery_checked_ids.push(stable_id.to_owned());
            }
        } else {
            self.source_gallery_checked_ids.retain(|id| id != stable_id);
        }
    }

    fn execute_source_gallery_open_action(
        &mut self,
        item: &SourceGalleryItem,
        viewer_ctx: Option<&ViewerContext<'_>>,
    ) -> bool {
        let action = item.open_action_report();
        if !action.enabled {
            self.source_gallery_status = Some(action.status);
            return false;
        }

        let Some(viewer_ctx) = viewer_ctx else {
            self.source_gallery_status =
                Some("Source action requires an active Rerun viewer context.".to_owned());
            return false;
        };

        let locator = item.locator.readable();
        let options = re_data_source::FromUriOptions {
            follow: false,
            accept_extensionless_http: false,
        };
        match ViewerOpenUrl::parse_with_options(&locator, &options) {
            Ok(open_url) => {
                open_url.open(
                    viewer_ctx.egui_ctx(),
                    &OpenUrlOptions {
                        follow: false,
                        recording_open_behavior: RecordingOpenBehavior::OpenAndSelect,
                        show_loader: true,
                    },
                    viewer_ctx.command_sender(),
                );
                self.source_gallery_status = Some(format!("Requested {}: {locator}", action.label));
                true
            }
            Err(error) => {
                self.source_gallery_status =
                    Some(format!("Cannot open `{locator}` in Rerun: {error}"));
                false
            }
        }
    }

    fn rebuild_source_gallery_index(&mut self) {
        let location = self.source_gallery_location.trim();
        if location.is_empty() {
            self.source_gallery_status = Some("Enter a source locator.".to_owned());
            return;
        }

        let source = SourceLocator::from_location(location);
        let manifest_json = self.source_gallery_manifest_json.trim();
        let result = if manifest_json.is_empty() {
            Ok(SourceGalleryIndex::from_locator(
                source,
                SOURCE_GALLERY_INDEX_LIMIT,
            ))
        } else {
            SourceGalleryIndex::from_manifest_json(
                source,
                manifest_json,
                SOURCE_GALLERY_INDEX_LIMIT,
            )
        };

        match result {
            Ok(index) => {
                self.source_gallery_selected_id =
                    index.items.first().map(|item| item.stable_id.clone());
                self.source_gallery_status =
                    Some(format!("Indexed {} item(s).", index.items.len()));
                self.source_gallery_index = Some(index);
            }
            Err(error) => {
                self.source_gallery_status = Some(error.to_string());
            }
        }
    }

    fn source_gallery_metadata_ui(&self, ui: &mut Ui, item: &SourceGalleryItem) {
        ui.strong("Selected Source");
        egui::Grid::new("houdini_source_gallery_selected_metadata")
            .num_columns(2)
            .spacing([12.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.weak("Name");
                ui.label(&item.display_name);
                ui.end_row();

                ui.weak("Kind");
                ui.label(item.kind.as_str());
                ui.end_row();

                ui.weak("Availability");
                ui.label(item.external_reference_status.as_str());
                ui.end_row();

                ui.weak("Thumbnail");
                ui.label(item.thumbnail_intent.status().as_str());
                ui.end_row();

                ui.weak("Format");
                ui.label(
                    item.format_kind
                        .map(|kind| kind.as_str())
                        .unwrap_or("unknown"),
                );
                ui.end_row();

                ui.weak("Support");
                ui.label(
                    item.format_support_status
                        .map(|status| status.as_str())
                        .unwrap_or("not inferred"),
                );
                ui.end_row();

                ui.weak("Locator");
                ui.monospace(item.locator.readable());
                ui.end_row();

                ui.weak("Stable id");
                ui.monospace(&item.stable_id);
                ui.end_row();
            });
    }

    fn filtered_asset_gallery_entries(
        &self,
        graph: &GraphDocument,
    ) -> Vec<model::ProceduralAssetGalleryEntry> {
        let filter = self.asset_gallery_filter.trim().to_lowercase();
        let entries = graph.procedural_asset_gallery_entries();
        if filter.is_empty() {
            return entries;
        }

        entries
            .into_iter()
            .filter(|entry| asset_gallery_entry_matches(entry, &filter))
            .collect()
    }

    fn jump_to_graph_node(
        &mut self,
        graph: &mut GraphDocument,
        node_index: usize,
        graph_id: &str,
    ) -> bool {
        if graph.select_graph_by_id(graph_id).is_err() || node_index >= graph.nodes.len() {
            self.asset_status = Some("Asset usage target is no longer available.".to_owned());
            return false;
        }
        self.select_single_node(node_index);
        self.selected_annotation = None;
        self.selected_edge = None;
        self.node_info_open = true;
        self.show_graph_workbench_pane(GraphWorkbenchPane::Info);
        self.pending_frame_selected = true;
        self.asset_status = graph
            .readable_node_path(node_index)
            .map(|path| format!("Selected asset instance: {path}"));
        true
    }

    fn match_selected_asset_definition(&mut self, graph: &mut GraphDocument) -> bool {
        self.match_asset_definition(graph, self.selected_node)
    }

    fn match_asset_definition(&mut self, graph: &mut GraphDocument, node_index: usize) -> bool {
        if graph.match_procedural_asset_definition(node_index) {
            self.asset_status = Some(graph.readable_node_path(node_index).map_or_else(
                || "Matched selected asset to its pinned definition.".to_owned(),
                |path| format!("Matched asset instance to pinned definition: {path}"),
            ));
            true
        } else {
            self.asset_status = Some("Selected asset cannot be matched.".to_owned());
            false
        }
    }

    fn upgrade_selected_asset_to_current_definition(&mut self, graph: &mut GraphDocument) -> bool {
        self.upgrade_asset_to_current_definition(graph, self.selected_node)
    }

    fn upgrade_asset_to_current_definition(
        &mut self,
        graph: &mut GraphDocument,
        node_index: usize,
    ) -> bool {
        let previous_version = graph
            .nodes
            .get(node_index)
            .and_then(|node| node.procedural_asset.as_ref())
            .map(|asset| asset.instance_version.clone());
        if graph.upgrade_procedural_asset_to_current_definition(node_index) {
            let current_version = graph
                .nodes
                .get(node_index)
                .and_then(|node| node.procedural_asset.as_ref())
                .map(|asset| asset.instance_version.clone())
                .unwrap_or_else(|| "current".to_owned());
            self.asset_status = Some(graph.readable_node_path(node_index).map_or_else(
                || {
                    format!(
                        "Upgraded asset instance from {} to {current_version}.",
                        previous_version.as_deref().unwrap_or("unknown")
                    )
                },
                |path| {
                    format!(
                        "Upgraded {path} from {} to {current_version}.",
                        previous_version.as_deref().unwrap_or("unknown")
                    )
                },
            ));
            true
        } else {
            self.asset_status = Some("Selected asset cannot be upgraded.".to_owned());
            false
        }
    }

    fn selected_node_can_create_asset_from_graph_container(&self, graph: &GraphDocument) -> bool {
        graph
            .selected_node_info(self.selected_node)
            .and_then(|info| info.graph_container)
            .is_some_and(|container| container.navigable && !container.outputs.is_empty())
    }

    fn create_asset_from_selected_graph_container(&mut self, graph: &mut GraphDocument) -> bool {
        let (asset_name, asset_description, asset_help) =
            self.selected_graph_container_asset_metadata(graph);
        match graph.create_asset_draft_from_graph_container(
            self.selected_node,
            asset_name,
            asset_description,
            asset_help,
        ) {
            Ok(draft) => {
                let asset_id = graph.commit_asset_draft(draft);
                self.asset_status = Some(format!(
                    "Created project asset from selected subnet: {asset_id}"
                ));
                self.show_graph_workbench_pane(GraphWorkbenchPane::Assets);
                true
            }
            Err(err) => {
                self.asset_status = Some(format!(
                    "Subnet asset creation failed: {}.",
                    graph_container_asset_draft_error_message(&err)
                ));
                false
            }
        }
    }

    fn selected_graph_container_asset_metadata(
        &self,
        graph: &GraphDocument,
    ) -> (String, String, String) {
        let selected_name = graph
            .nodes
            .get(self.selected_node)
            .map(|node| node.name.trim())
            .filter(|name| !name.is_empty())
            .unwrap_or(DEFAULT_ASSET_NAME);

        let asset_name =
            metadata_value_or_fallback(&self.asset_name, DEFAULT_ASSET_NAME, selected_name);
        let asset_description = metadata_value_or_fallback(
            &self.asset_description,
            DEFAULT_ASSET_DESCRIPTION,
            &format!("Project-local asset from {selected_name}."),
        );
        let asset_help = metadata_value_or_fallback(
            &self.asset_help,
            DEFAULT_ASSET_HELP,
            "Created from the selected Houdini subnet.",
        );
        (asset_name, asset_description, asset_help)
    }

    fn recording_export_ui(&mut self, ui: &mut Ui, graph: &GraphDocument) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            ui.horizontal(|ui| {
                if ui.button("Save Recording...").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Rerun recording", &["rrd"])
                        .set_file_name("houdini-graph-output.rrd")
                        .save_file()
                {
                    match graph.save_rerun_recording(&path) {
                        Ok(recording) => {
                            self.recording_status = Some(format!(
                                "Saved recording: {} ({} items, {} rasters, {} polygons, {} native cubics). {}",
                                recording.path.display(),
                                recording.item_count,
                                recording.substrate_raster_count,
                                recording.polygon_count,
                                recording.native_cubic_bezier_count,
                                recording.limitation_note
                            ));
                        }
                        Err(err) => {
                            self.recording_status = Some(format!("Recording save failed: {err}"));
                        }
                    }
                }

                if ui.button("Save Source Manifest...").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON manifest", &["json"])
                        .set_file_name("houdini-source-package-manifest.json")
                        .save_file()
                {
                    match graph.save_source_package_manifest(&path) {
                        Ok(manifest) => {
                            self.package_manifest_status = Some(format!(
                                "Saved source manifest: {} ({} artifacts, {} external references, {} missing, {} warnings). Source files were not copied or hashed.",
                                manifest.path.display(),
                                manifest.artifact_count,
                                manifest.remaining_external_reference_count,
                                manifest.missing_reference_count,
                                manifest.reproducibility_warning_count
                            ));
                        }
                        Err(err) => {
                            self.package_manifest_status =
                                Some(format!("Source manifest save failed: {err}"));
                        }
                    }
                }

                if ui.button("Save Source Package...").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .set_directory(".")
                        .pick_folder()
                {
                    match graph.save_source_package(&path) {
                        Ok(package) => {
                            self.package_manifest_status = Some(format!(
                                "Saved source package: {} ({} copied of {} artifacts, {} external references, {} missing, {} warnings).",
                                package.package_dir.display(),
                                package.copied_artifact_count,
                                package.artifact_count,
                                package.remaining_external_reference_count,
                                package.missing_reference_count,
                                package.reproducibility_warning_count
                            ));
                        }
                        Err(err) => {
                            self.package_manifest_status =
                                Some(format!("Source package save failed: {err}"));
                        }
                    }
                }
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            ui.weak(
                "Recording export and source manifest writing are available in the native viewer.",
            );
        }

        if let Some(status) = &self.recording_status {
            ui.weak(status);
        }
        if let Some(status) = &self.package_manifest_status {
            ui.weak(status);
        }
    }

    fn operator_strip_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.horizontal(|ui| {
            ui.weak("TAB");
            ui.add(
                egui::TextEdit::singleline(&mut self.operator_filter)
                    .desired_width((ui.available_width() - 44.0).clamp(96.0, 180.0))
                    .hint_text("operator"),
            );
            if ui.small_button("Clear").clicked() {
                self.operator_filter.clear();
            }
        });

        ui.weak(format!("Selected: {}", self.selected_item_label(graph)));

        self.operator_palette_ui(
            ui,
            graph,
            OperatorPaletteUiOptions {
                id_salt: "houdini_operator_side_palette",
                grouped: true,
                show_recent: true,
                include_organization: true,
                include_layers: true,
                highlighted_action: None,
            },
        );
    }

    fn graph_search_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.horizontal(|ui| {
            ui.weak("Find");
            ui.add(
                egui::TextEdit::singleline(&mut self.graph_search_filter)
                    .desired_width((ui.available_width() - 48.0).clamp(120.0, 220.0))
                    .hint_text("node, note, box"),
            );
            if ui.small_button("Clear").clicked() {
                self.graph_search_filter.clear();
            }
        });
        ui.weak("Search graph metadata; dataset records are unchanged.");

        let results = self.graph_search_results(graph);
        if self.graph_search_filter.trim().is_empty() {
            ui.weak("Type to find nodes, comments, graph notes, or network boxes.");
            return;
        }
        if results.is_empty() {
            ui.weak("No matching graph items.");
            return;
        }

        ui.separator();
        for result in results {
            let selected = match &result.target {
                GraphSearchTarget::Node { index, graph_id } => {
                    self.selected_annotation.is_none()
                        && self.selected_node == *index
                        && graph.current_graph_id() == graph_id
                }
                GraphSearchTarget::Annotation(index) => self.selected_annotation == Some(*index),
            };
            let clicked = ui
                .selectable_label(selected, format!("{}  {}", result.kind, result.label))
                .on_hover_text(&result.detail)
                .clicked();
            ui.weak(&result.detail);
            if clicked {
                self.apply_graph_search_result(graph, result.target.clone());
            }
        }
    }

    fn graph_search_results(&self, graph: &GraphDocument) -> Vec<GraphSearchResult> {
        let filter = self.graph_search_filter.trim().to_lowercase();
        if filter.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        for (index, node) in graph.nodes.iter().enumerate() {
            let graph_location = graph
                .selected_node_info(index)
                .map(|info| info.graph_location);
            let node_path = graph_location
                .as_ref()
                .map(|location| location.node_path.as_str())
                .unwrap_or(node.name.as_str());
            let parameter_value = format!("{:.2}", node.parameter.value);
            let haystack = [
                node_path,
                node.name.as_str(),
                node.kind.as_str(),
                node.parameter.name,
                node.parameter.kind.as_str(),
                parameter_value.as_str(),
                node.comment.as_str(),
                node.info,
            ]
            .join(" ")
            .to_lowercase();
            if haystack.contains(&filter) {
                results.push(GraphSearchResult {
                    target: GraphSearchTarget::Node {
                        index,
                        graph_id: graph_location
                            .as_ref()
                            .map(|location| location.graph_id.clone())
                            .unwrap_or_else(|| graph.current_graph_id().to_owned()),
                    },
                    label: node.name.clone(),
                    kind: "Node",
                    detail: format!(
                        "{}; {}; {} = {:.2}",
                        node_path,
                        node.kind.as_str(),
                        node.parameter.name,
                        node.parameter.value
                    ),
                });
            }
        }

        for index in graph.current_graph_annotation_indices() {
            let Some(annotation) = graph.annotations.get(index) else {
                continue;
            };
            let haystack = [
                annotation.title.as_str(),
                annotation.text.as_str(),
                annotation.kind.as_str(),
            ]
            .join(" ")
            .to_lowercase();
            if haystack.contains(&filter) {
                results.push(GraphSearchResult {
                    target: GraphSearchTarget::Annotation(index),
                    label: annotation.title.clone(),
                    kind: annotation.kind.as_str(),
                    detail: if annotation.text.trim().is_empty() {
                        format!("{} layout item", annotation.kind.as_str())
                    } else {
                        format_sticky_note_text(&annotation.text)
                    },
                });
            }
        }

        results
    }

    fn apply_graph_search_result(&mut self, graph: &mut GraphDocument, target: GraphSearchTarget) {
        match target {
            GraphSearchTarget::Node { index, graph_id } => {
                let _ = graph.select_graph_by_id(&graph_id);
                self.select_single_node(index);
                self.selected_annotation = None;
                self.context_menu_canvas = false;
                self.node_info_open = true;
                self.pending_frame_selected = true;
            }
            GraphSearchTarget::Annotation(index) => {
                self.selected_annotation = Some(index);
                self.selected_edge = None;
                self.selected_nodes.clear();
                self.context_menu_canvas = false;
                self.pending_frame_selected = true;
                self.active_graph_pane = GraphWorkbenchPane::Find;
            }
        }
    }

    fn compact_layer_stack_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        egui::Grid::new("houdini_graph_compact_layer_stack")
            .num_columns(4)
            .spacing([8.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.weak("On");
                ui.weak("Order");
                ui.weak("Name");
                ui.weak("Kind");
                ui.end_row();

                for layer_index in 0..graph.layers.len() {
                    let mut visible = graph.layers[layer_index].visible;
                    if ui.re_checkbox(&mut visible, "").changed() {
                        graph.set_layer_visibility(layer_index, visible);
                    }

                    let mut order = graph.layers[layer_index].order;
                    if ui
                        .add(DragValue::new(&mut order).speed(1).range(-99..=99))
                        .changed()
                    {
                        graph.set_layer_order(layer_index, order);
                    }

                    let kind = graph.layers[layer_index].kind;
                    if let Some(layer) = graph.layers.get_mut(layer_index) {
                        ui.add(egui::TextEdit::singleline(&mut layer.name).desired_width(96.0));
                    }
                    ui.label(kind.as_str());
                    ui.end_row();
                }
            });

        let generated_nodes = graph
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| node.generated.map(|generated| (index, node, generated)))
            .collect::<Vec<_>>();
        if generated_nodes.is_empty() {
            return;
        }

        ui.add_space(6.0);
        ui.weak("Graph-backed layer controls");
        egui::Grid::new("houdini_graph_generated_layer_bindings")
            .num_columns(3)
            .spacing([8.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.weak("Node");
                ui.weak("Binding");
                ui.weak("Source");
                ui.end_row();

                for (index, node, generated) in generated_nodes {
                    if ui
                        .selectable_label(self.selected_node == index, &node.name)
                        .clicked()
                    {
                        self.select_single_node(index);
                        self.selected_annotation = None;
                        self.pending_frame_selected = true;
                    }
                    ui.label(generated.binding_state.as_str())
                        .on_hover_text(generated.binding_state.description());
                    ui.weak(generated.source.as_str());
                    ui.end_row();
                }
            });
    }

    fn import_parquet_path(
        &mut self,
        graph: &mut GraphDocument,
        path: impl AsRef<std::path::Path>,
    ) {
        let path = path.as_ref();
        match graph.import_cubic_bezier_parquet_path(path) {
            Ok(imported) => {
                self.last_parquet_path = Some(path.display().to_string());
                self.parquet_status =
                    Some(format!("Imported {imported} native cubic Bezier curves"));
            }
            Err(err) => {
                self.last_parquet_path = Some(path.display().to_string());
                self.parquet_status = Some(format!("Parquet import failed: {err}"));
            }
        }
    }

    fn import_csv_path(&mut self, graph: &mut GraphDocument, path: impl AsRef<std::path::Path>) {
        let path = path.as_ref();
        match graph.import_polygon_csv_path(path) {
            Ok(imported) => {
                self.last_parquet_path = Some(path.display().to_string());
                self.parquet_status = Some(format!("Imported {imported} CSV polygon records"));
            }
            Err(err) => {
                self.last_parquet_path = Some(path.display().to_string());
                self.parquet_status = Some(format!("CSV import failed: {err}"));
            }
        }
    }

    fn import_geojson_path(
        &mut self,
        graph: &mut GraphDocument,
        path: impl AsRef<std::path::Path>,
    ) {
        let path = path.as_ref();
        match graph.import_geojson_polygon_path(path) {
            Ok(imported) => {
                self.last_parquet_path = Some(path.display().to_string());
                self.parquet_status = Some(format!("Imported {imported} GeoJSON polygon records"));
            }
            Err(err) => {
                self.last_parquet_path = Some(path.display().to_string());
                self.parquet_status = Some(format!("GeoJSON import failed: {err}"));
            }
        }
    }

    fn evaluation_controls_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        if self.selected_node >= graph.nodes.len() {
            return;
        }

        ui.horizontal(|ui| {
            if ui.button("Run").clicked() {
                graph.request_node_run(self.selected_node);
                graph.complete_node_run(self.selected_node);
            }
            if ui.button("Start").clicked() {
                graph.request_node_run(self.selected_node);
            }
            if ui.button("Cancel").clicked() {
                graph.cancel_node_run(self.selected_node);
            }
            if ui.button("Retry").clicked() {
                graph.request_node_run(self.selected_node);
                graph.complete_node_run(self.selected_node);
            }
        });
        if ui.button("Evaluate Output").clicked() {
            graph.demand_output_evaluation();
        }
    }

    fn node_graph_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        desired_height: f32,
    ) -> Response {
        let desired_size = egui::vec2(ui.available_width().max(280.0), desired_height);
        let (response, painter) = ui.allocate_painter(desired_size, Sense::click_and_drag());
        let canvas_rect = response.rect;
        let network_view = graph.network_view;
        painter.rect_filled(
            canvas_rect,
            4.0,
            network_background_color(ui.visuals(), network_view.background_brightness),
        );
        painter.rect_stroke(
            canvas_rect,
            4.0,
            ui.visuals().widgets.noninteractive.bg_stroke,
            StrokeKind::Inside,
        );
        draw_network_grid(
            &painter,
            canvas_rect,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
            network_view.grid_spacing,
            self.graph_view_zoom,
            self.graph_view_pan,
        );

        let layout_rect = canvas_rect.shrink2(egui::vec2(12.0, 10.0));
        let node_size = Vec2::new(116.0, 48.0);
        let mut node_rects = layout_node_rects(
            graph,
            layout_rect,
            node_size,
            self.graph_view_zoom,
            self.graph_view_pan,
        );
        let mut annotation_rects = layout_annotation_rects(
            graph,
            layout_rect,
            self.graph_view_zoom,
            self.graph_view_pan,
        );

        let mut layout_changed = false;
        if self.pending_frame_selected {
            layout_changed |= self.frame_selected_item_in_rect(graph, layout_rect, node_size);
            self.pending_frame_selected = false;
        }
        if response.hovered() && !self.tab_menu_open {
            let shortcut = ui.input(|input| {
                let shift_only = modifiers_are_shift_only(input.modifiers);
                (
                    input.key_pressed(egui::Key::D) && input.modifiers.is_none(),
                    input.key_pressed(egui::Key::Tab),
                    input
                        .pointer
                        .hover_pos()
                        .unwrap_or_else(|| canvas_rect.center()),
                    input.key_pressed(egui::Key::H) && input.modifiers.is_none(),
                    input.key_pressed(egui::Key::F) && input.modifiers.is_none(),
                    input.key_pressed(egui::Key::F) && input.modifiers.command,
                    input.key_pressed(egui::Key::O) && shift_only,
                    input.key_pressed(egui::Key::P) && shift_only,
                    input.key_pressed(egui::Key::M) && shift_only,
                    input.key_pressed(egui::Key::I) && input.modifiers.is_none(),
                    input.key_pressed(egui::Key::Q) && input.modifiers.is_none(),
                    input.key_pressed(egui::Key::M) && input.modifiers.is_none(),
                    input.key_pressed(egui::Key::R) && input.modifiers.is_none(),
                    input.key_pressed(egui::Key::Enter) && input.modifiers.is_none(),
                    input.key_pressed(egui::Key::U) && input.modifiers.is_none(),
                )
            });
            let (
                display_options_pressed,
                tab_pressed,
                pointer_anchor,
                home_pressed,
                frame_selected_pressed,
                find_pressed,
                add_network_box_pressed,
                add_sticky_note_pressed,
                resize_box_pressed,
                node_info_pressed,
                display_flag_pressed,
                manual_flag_pressed,
                run_node_pressed,
                enter_subnet_pressed,
                go_up_pressed,
            ) = shortcut;

            if display_options_pressed {
                self.active_graph_pane = GraphWorkbenchPane::Display;
                toggle_network_display_options(ui);
            }
            if tab_pressed {
                self.open_operator_chooser_at(pointer_anchor);
            }
            if home_pressed {
                self.reset_graph_view();
                layout_changed = true;
            }
            if frame_selected_pressed {
                layout_changed |= self.frame_selected_item_in_rect(graph, layout_rect, node_size);
            }
            if find_pressed {
                self.active_graph_pane = GraphWorkbenchPane::Find;
            }
            if add_network_box_pressed {
                layout_changed |=
                    self.apply_operator_palette_action(graph, OperatorPaletteAction::AddNetworkBox);
            }
            if add_sticky_note_pressed {
                layout_changed |=
                    self.apply_operator_palette_action(graph, OperatorPaletteAction::AddStickyNote);
            }
            if resize_box_pressed {
                layout_changed |= self.resize_selected_network_box_to_contents(graph);
            }
            if node_info_pressed {
                self.apply_node_ring_action(graph, self.selected_node, NodeRingAction::Info);
            }
            if display_flag_pressed {
                self.apply_node_ring_action(graph, self.selected_node, NodeRingAction::Display);
            }
            if manual_flag_pressed {
                self.apply_node_ring_action(graph, self.selected_node, NodeRingAction::Manual);
            }
            if run_node_pressed {
                self.apply_node_ring_action(graph, self.selected_node, NodeRingAction::Run);
            }
            if enter_subnet_pressed && self.selected_node_can_enter_graph_container(graph) {
                layout_changed |= self.enter_selected_graph_container(graph);
            }
            if go_up_pressed {
                layout_changed |= self.exit_current_graph_to_parent(graph);
            }
        }
        if layout_changed {
            node_rects = layout_node_rects(
                graph,
                layout_rect,
                node_size,
                self.graph_view_zoom,
                self.graph_view_pan,
            );
            annotation_rects = layout_annotation_rects(
                graph,
                layout_rect,
                self.graph_view_zoom,
                self.graph_view_pan,
            );
        }

        let generated_lane_y = transform_layout_pos(
            layout_rect,
            Pos2::new(
                layout_rect.left(),
                layout_rect.top() + layout_rect.height() * 0.82,
            ),
            self.graph_view_zoom,
            self.graph_view_pan,
        )
        .y;
        painter.line_segment(
            [
                Pos2::new(layout_rect.left(), generated_lane_y),
                Pos2::new(layout_rect.right(), generated_lane_y),
            ],
            Stroke::new(1.0, ui.visuals().weak_text_color()),
        );
        painter.text(
            Pos2::new(layout_rect.left() + 4.0, generated_lane_y - 14.0),
            Align2::LEFT_TOP,
            "Generated",
            FontId::monospace(10.0),
            ui.visuals().weak_text_color(),
        );

        let hovered_annotation_index = if response.hovered() {
            ui.input(|input| input.pointer.hover_pos())
                .and_then(|pointer_pos| {
                    annotation_rects
                        .iter()
                        .rev()
                        .find_map(|(index, annotation_rect)| {
                            annotation_rect.contains(pointer_pos).then_some(*index)
                        })
                })
        } else {
            None
        };

        for (annotation_index, _) in &annotation_rects {
            let Some(annotation) = graph.annotations.get(*annotation_index) else {
                continue;
            };
            draw_graph_annotation(
                &painter,
                layout_rect,
                annotation,
                self.graph_view_zoom,
                self.graph_view_pan,
                self.selected_annotation == Some(*annotation_index),
                hovered_annotation_index == Some(*annotation_index),
                ui.visuals(),
            );
        }

        if response.hovered() {
            self.update_graph_viewport(ui, layout_rect);
        }

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if response.dragged_by(egui::PointerButton::Middle) {
                self.graph_view_pan += ui.input(|input| input.pointer.delta());
            }

            if response.clicked_by(egui::PointerButton::Primary)
                || response.drag_started_by(egui::PointerButton::Primary)
            {
                self.context_menu_canvas = false;
                self.context_menu_edge = None;
                self.dragging_node = None;
                self.dragging_annotation = None;
                self.resizing_annotation = None;
                self.selection_drag = None;
                let mut hit_node = false;
                if let Some(port_hit) =
                    node_primary_port_at(graph, &node_rects, pointer_pos, self.graph_view_zoom)
                {
                    self.select_single_node(port_hit.node_index);
                    self.selected_edge = None;
                    self.selected_annotation = None;
                    self.node_info_open = true;
                    if port_hit.kind == NodePortKind::Output
                        && let Some(node) = graph.nodes.get(port_hit.node_index)
                    {
                        self.connection_drag = Some(ConnectionDragState {
                            from_node_index: port_hit.node_index,
                            from_node_id: node.node_id.clone(),
                            from_output: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
                        });
                    }
                    hit_node = true;
                }
                if !hit_node {
                    for (index, node_rect) in node_rects.iter().enumerate() {
                        if let Some(flag_action) = node_flag_action_at(*node_rect, pointer_pos) {
                            self.select_single_node(index);
                            self.apply_node_ring_action(graph, index, flag_action);
                            hit_node = true;
                            break;
                        }
                        let ring_visible = node_ring_visible(
                            network_view.node_ring_visibility,
                            self.selected_node == index,
                            node_rect.contains(pointer_pos)
                                || node_ring_action_at(
                                    *node_rect,
                                    pointer_pos,
                                    self.graph_view_zoom,
                                )
                                .is_some(),
                        );
                        if ring_visible
                            && let Some(ring_action) =
                                node_ring_action_at(*node_rect, pointer_pos, self.graph_view_zoom)
                        {
                            self.select_single_node(index);
                            self.apply_node_ring_action(graph, index, ring_action);
                            hit_node = true;
                            break;
                        }
                        if node_rect.contains(pointer_pos) {
                            self.select_single_node(index);
                            self.selected_edge = None;
                            self.selected_annotation = None;
                            self.node_info_open = true;
                            if response.double_clicked_by(egui::PointerButton::Primary)
                                && self.selected_node_can_enter_graph_container(graph)
                            {
                                self.enter_selected_graph_container(graph);
                                hit_node = true;
                                break;
                            }
                            self.dragging_node = Some(index);
                            self.node_drag_start_position =
                                graph.nodes.get(index).map(|node| node.layout_position);
                            self.node_drag_start_network_box_states =
                                graph.network_box_organization_snapshots();
                            self.node_drag_peak_delta_pixels = 0.0;
                            hit_node = true;
                            break;
                        }
                    }
                }

                let mut hit_annotation = false;
                if !hit_node {
                    for (index, annotation_rect) in annotation_rects.iter().rev() {
                        if annotation_collapse_toggle_rect(*annotation_rect).contains(pointer_pos) {
                            if let Some(annotation) = graph.annotations.get(*index) {
                                graph.set_annotation_collapsed(*index, !annotation.collapsed);
                            }
                            self.selected_annotation = Some(*index);
                            self.selected_edge = None;
                            self.selected_nodes.clear();
                            hit_annotation = true;
                            break;
                        }
                        if annotation_resize_handle_rect(*annotation_rect).contains(pointer_pos) {
                            self.selected_annotation = Some(*index);
                            self.selected_edge = None;
                            self.selected_nodes.clear();
                            self.resizing_annotation = Some(*index);
                            self.annotation_resize_start_size = graph
                                .annotations
                                .get(*index)
                                .map(|annotation| annotation.size);
                            hit_annotation = true;
                            break;
                        }
                        if annotation_rect.contains(pointer_pos) {
                            self.selected_annotation = Some(*index);
                            self.selected_edge = None;
                            self.selected_nodes.clear();
                            self.dragging_annotation = Some(*index);
                            self.annotation_drag_start_position = graph
                                .annotations
                                .get(*index)
                                .map(|annotation| annotation.position);
                            self.annotation_drag_start_member_positions =
                                graph.annotation_member_layout_positions(*index);
                            hit_annotation = true;
                            break;
                        }
                    }
                }

                let hit_edge = if !hit_node && !hit_annotation {
                    graph_edge_at(
                        graph,
                        &node_rects,
                        pointer_pos,
                        edge_hit_radius(self.graph_view_zoom),
                    )
                    .map(|edge_id| {
                        self.selected_edge = Some(edge_id);
                        self.selected_annotation = None;
                        self.selected_nodes.clear();
                    })
                    .is_some()
                } else {
                    false
                };

                if response.clicked_by(egui::PointerButton::Primary)
                    && !hit_node
                    && !hit_annotation
                    && !hit_edge
                    && !self.node_info_pinned
                {
                    self.node_info_open = false;
                    self.selected_annotation = None;
                    self.selected_edge = None;
                    self.selected_nodes.clear();
                }

                if response.drag_started_by(egui::PointerButton::Primary)
                    && !hit_node
                    && !hit_annotation
                    && !hit_edge
                {
                    self.selected_annotation = None;
                    self.selected_edge = None;
                    self.selection_drag = Some(SelectionDragState {
                        start: pointer_pos,
                        current: pointer_pos,
                    });
                }
            }

            if response.clicked_by(egui::PointerButton::Secondary) {
                self.selected_annotation = None;
                self.context_menu_edge = None;
                self.context_menu_canvas = true;
                let mut hit_annotation = false;
                for (index, annotation_rect) in annotation_rects.iter().rev() {
                    if annotation_rect.contains(pointer_pos) {
                        self.selected_annotation = Some(*index);
                        self.selected_edge = None;
                        self.selected_nodes.clear();
                        self.context_menu_canvas = false;
                        hit_annotation = true;
                        break;
                    }
                }
                let mut hit_node = false;
                if !hit_annotation {
                    for (index, node_rect) in node_rects.iter().enumerate() {
                        if node_rect.contains(pointer_pos) {
                            self.select_single_node(index);
                            self.selected_edge = None;
                            self.selected_annotation = None;
                            self.context_menu_canvas = false;
                            self.node_info_open = true;
                            hit_node = true;
                            break;
                        }
                    }
                }
                if !hit_annotation
                    && !hit_node
                    && let Some(edge_id) = graph_edge_at(
                        graph,
                        &node_rects,
                        pointer_pos,
                        edge_hit_radius(self.graph_view_zoom),
                    )
                {
                    self.selected_edge = Some(edge_id.clone());
                    self.context_menu_edge = Some(edge_id);
                    self.selected_annotation = None;
                    self.selected_nodes.clear();
                    self.context_menu_canvas = false;
                }
            }

            if response.dragged_by(egui::PointerButton::Primary) {
                if self.connection_drag.is_some() {
                    // Connection preview is drawn later after existing wires.
                } else if let Some(dragging_node) = self.dragging_node {
                    let pointer_delta = ui.input(|input| input.pointer.delta());
                    self.node_drag_peak_delta_pixels =
                        self.node_drag_peak_delta_pixels.max(pointer_delta.length());
                    graph.set_node_layout_position(
                        dragging_node,
                        unmap_node_layout_point(
                            layout_rect,
                            pointer_pos,
                            node_size,
                            self.graph_view_zoom,
                            self.graph_view_pan,
                        ),
                    );
                    node_rects = layout_node_rects(
                        graph,
                        layout_rect,
                        node_size,
                        self.graph_view_zoom,
                        self.graph_view_pan,
                    );
                } else if let Some(resizing_annotation) = self.resizing_annotation {
                    let pointer_delta = ui.input(|input| input.pointer.delta());
                    let size = graph
                        .annotations
                        .get(resizing_annotation)
                        .map(|annotation| GraphPoint {
                            x: (annotation.size.x
                                + pointer_delta.x / (layout_rect.width() * self.graph_view_zoom))
                                .clamp(0.08, 0.95),
                            y: (annotation.size.y
                                + pointer_delta.y / (layout_rect.height() * self.graph_view_zoom))
                                .clamp(0.08, 0.95),
                        });
                    if let Some(size) = size {
                        graph.set_annotation_size(resizing_annotation, size);
                    }
                } else if let Some(dragging_annotation) = self.dragging_annotation {
                    let pointer_delta = ui.input(|input| input.pointer.delta());
                    graph.translate_annotation(
                        dragging_annotation,
                        GraphPoint {
                            x: pointer_delta.x / (layout_rect.width() * self.graph_view_zoom),
                            y: pointer_delta.y / (layout_rect.height() * self.graph_view_zoom),
                        },
                    );
                } else if let Some(selection_drag) = self.selection_drag.as_mut() {
                    selection_drag.current = pointer_pos;
                }
            }
        }

        if ui.input(|input| input.pointer.any_released()) {
            if let Some(connection_drag) = self.connection_drag.take()
                && let Some(pointer_pos) = ui.input(|input| input.pointer.latest_pos())
            {
                self.finish_connection_drag(graph, &node_rects, connection_drag, pointer_pos);
            }
            if let Some(dragging_node) = self.dragging_node {
                graph.settle_node_drag_for_network_boxes(
                    dragging_node,
                    self.node_drag_peak_delta_pixels >= NETWORK_BOX_FAST_DRAG_PEAK_DELTA_PIXELS,
                );
                if let Some(start_position) = self.node_drag_start_position {
                    graph.finish_node_layout_drag_with_network_box_snapshots(
                        dragging_node,
                        start_position,
                        &self.node_drag_start_network_box_states,
                    );
                }
            }
            if let Some(dragging_annotation) = self.dragging_annotation
                && let Some(start_position) = self.annotation_drag_start_position
            {
                graph.finish_annotation_drag(
                    dragging_annotation,
                    start_position,
                    &self.annotation_drag_start_member_positions,
                );
            }
            if let Some(resizing_annotation) = self.resizing_annotation
                && let Some(start_size) = self.annotation_resize_start_size
            {
                graph.finish_annotation_resize(resizing_annotation, start_size);
            }
            if let Some(selection_drag) = self.selection_drag.take() {
                let selected_nodes =
                    node_indices_in_selection_rect(graph, &node_rects, selection_drag.rect());
                if selected_nodes.is_empty() {
                    self.selected_nodes.clear();
                    self.graph_container_status =
                        Some("Marquee selected no graph nodes.".to_owned());
                } else {
                    let selected_count = selected_nodes.len();
                    self.set_selected_node_set(selected_nodes);
                    self.selected_annotation = None;
                    self.selected_edge = None;
                    self.node_info_open = true;
                    self.graph_container_status =
                        Some(format!("Marquee selected {selected_count} graph node(s)."));
                }
            }
            self.dragging_node = None;
            self.node_drag_start_position = None;
            self.node_drag_start_network_box_states.clear();
            self.node_drag_peak_delta_pixels = 0.0;
            self.dragging_annotation = None;
            self.annotation_drag_start_position = None;
            self.annotation_drag_start_member_positions.clear();
            self.resizing_annotation = None;
            self.annotation_resize_start_size = None;
        }

        let hovered_ring_action = if response.hovered() {
            ui.input(|input| input.pointer.hover_pos())
                .and_then(|pointer_pos| {
                    node_rects
                        .iter()
                        .enumerate()
                        .find_map(|(index, node_rect)| {
                            let ring_visible = node_ring_visible(
                                network_view.node_ring_visibility,
                                self.selected_node == index,
                                node_rect.contains(pointer_pos)
                                    || node_ring_action_at(
                                        *node_rect,
                                        pointer_pos,
                                        self.graph_view_zoom,
                                    )
                                    .is_some(),
                            );
                            ring_visible
                                .then(|| {
                                    node_ring_action_at(
                                        *node_rect,
                                        pointer_pos,
                                        self.graph_view_zoom,
                                    )
                                    .map(|action| (index, action, pointer_pos))
                                })
                                .flatten()
                        })
                })
        } else {
            None
        };

        let hovered_node_flag_action = if response.hovered() {
            ui.input(|input| input.pointer.hover_pos())
                .and_then(|pointer_pos| {
                    node_rects
                        .iter()
                        .enumerate()
                        .find_map(|(index, node_rect)| {
                            node_flag_action_at(*node_rect, pointer_pos)
                                .map(|action| (index, action, pointer_pos))
                        })
                })
        } else {
            None
        };

        let hovered_edge_id = if response.hovered() {
            ui.input(|input| input.pointer.hover_pos())
                .and_then(|pointer_pos| {
                    graph_edge_at(
                        graph,
                        &node_rects,
                        pointer_pos,
                        edge_hit_radius(self.graph_view_zoom),
                    )
                })
        } else {
            None
        };

        let connector_color = ui.visuals().widgets.noninteractive.fg_stroke.color;
        for edge in graph.graph_layout().edges {
            let from_rect = node_rects[edge.from_node];
            let to_rect = node_rects[edge.to_node];
            let start = Pos2::new(from_rect.right(), from_rect.center().y);
            let end = Pos2::new(to_rect.left(), to_rect.center().y);
            let wire_length = (end.x - start.x).hypot(end.y - start.y);
            let fade = if wire_length > layout_rect.width() * 0.34 {
                1.0 - network_view.long_wire_fading * 0.65
            } else {
                1.0
            };
            let selected = self
                .selected_edge
                .as_deref()
                .is_some_and(|edge_id| edge_id == edge.edge_id);
            let hovered = hovered_edge_id
                .as_deref()
                .is_some_and(|edge_id| edge_id == edge.edge_id);
            let connector_stroke = if selected || hovered {
                Stroke::new(
                    if selected { 3.0 } else { 2.2 },
                    ui.visuals().selection.stroke.color,
                )
            } else {
                Stroke::new(1.5, faded_color(connector_color, fade))
            };
            painter.line_segment([start, end], connector_stroke);
            draw_arrowhead(&painter, end, connector_stroke.color);
        }

        if let Some(connection_drag) = &self.connection_drag
            && let Some(from_rect) = node_rects.get(connection_drag.from_node_index)
            && let Some(pointer_pos) = ui.input(|input| input.pointer.hover_pos())
        {
            let start =
                node_primary_port_rect(*from_rect, NodePortKind::Output, self.graph_view_zoom)
                    .center();
            let preview = connection_drag_preview(
                graph,
                &node_rects,
                connection_drag,
                pointer_pos,
                self.graph_view_zoom,
            );
            let stroke = Stroke::new(2.0, connection_drag_preview_color(&preview, ui.visuals()));
            painter.line_segment([start, pointer_pos], stroke);
            draw_arrowhead(&painter, pointer_pos, stroke.color);
        }

        if let Some(selection_drag) = self.selection_drag {
            let selection_rect = selection_drag.rect();
            painter.rect_filled(
                selection_rect,
                0.0,
                faded_color(ui.visuals().selection.bg_fill, 0.28),
            );
            painter.rect_stroke(
                selection_rect,
                0.0,
                Stroke::new(1.5, ui.visuals().selection.stroke.color),
                StrokeKind::Inside,
            );
        }

        for layout_node in graph.graph_layout().nodes {
            let Some(node) = graph.nodes.get(layout_node.node_index) else {
                continue;
            };
            let node_rect = node_rects[layout_node.node_index];
            let selected = self.selected_node == layout_node.node_index
                || self.selected_nodes.contains(&layout_node.node_index);
            let hovered = response.hovered()
                && ui.input(|input| {
                    input
                        .pointer
                        .hover_pos()
                        .is_some_and(|pointer_pos| node_rect.contains(pointer_pos))
                });
            let show_ring = node_ring_visible(network_view.node_ring_visibility, selected, hovered);
            if show_ring {
                let hovered_action = hovered_ring_action.and_then(|(node_index, action, _)| {
                    (node_index == layout_node.node_index).then_some(action)
                });
                draw_node_ring(
                    &painter,
                    node_rect,
                    selected,
                    node,
                    hovered_action,
                    ui.visuals(),
                );
            }

            let fill = if selected {
                ui.visuals().selection.bg_fill
            } else if hovered {
                ui.visuals().widgets.hovered.bg_fill
            } else {
                ui.visuals().widgets.inactive.bg_fill
            };
            let stroke = if selected {
                Stroke::new(2.0, ui.visuals().selection.stroke.color)
            } else if hovered {
                Stroke::new(1.5, ui.visuals().widgets.hovered.fg_stroke.color)
            } else {
                ui.visuals().widgets.inactive.fg_stroke
            };

            if selected {
                painter.rect_stroke(
                    node_rect.expand(3.0),
                    7.0,
                    Stroke::new(1.0, faded_color(ui.visuals().selection.stroke.color, 0.72)),
                    StrokeKind::Inside,
                );
            }
            painter.rect_filled(node_rect, 6.0, fill);
            painter.rect_stroke(node_rect, 6.0, stroke, StrokeKind::Inside);
            painter.text(
                node_rect.center_top() + egui::vec2(0.0, 10.0),
                Align2::CENTER_TOP,
                format_node_name(layout_node.name, network_view.max_node_name_width),
                FontId::proportional(13.0),
                ui.visuals().text_color(),
            );
            painter.text(
                node_rect.center_bottom() - egui::vec2(0.0, 8.0),
                Align2::CENTER_BOTTOM,
                format!("{:.2}", node.parameter.value),
                FontId::monospace(11.0),
                ui.visuals().weak_text_color(),
            );
            painter.text(
                node_rect.left_bottom() + egui::vec2(6.0, -8.0),
                Align2::LEFT_BOTTOM,
                node.evaluation.state.as_str(),
                FontId::monospace(9.0),
                evaluation_color(ui, node.evaluation.state),
            );
            if let Some(generated) = node.generated {
                painter.text(
                    node_rect.right_top() + egui::vec2(-6.0, 6.0),
                    Align2::RIGHT_TOP,
                    generated.binding_state.badge(),
                    FontId::monospace(10.0),
                    ui.visuals().warn_fg_color,
                );
            }
            if network_comment_visible(network_view.comment_display_mode, node) {
                painter.text(
                    node_rect.right_center() + egui::vec2(10.0, 0.0),
                    Align2::LEFT_CENTER,
                    format_node_comment(&node.comment),
                    FontId::proportional(12.0),
                    ui.visuals().weak_text_color(),
                );
            }
            let hovered_flag_action =
                hovered_node_flag_action.and_then(|(node_index, action, _)| {
                    (node_index == layout_node.node_index).then_some(action)
                });
            draw_node_flag_strip(&painter, node_rect, node, hovered_flag_action, ui.visuals());
            draw_node_badges(
                &painter,
                node_rect,
                graph,
                layout_node.node_index,
                node,
                network_view,
                ui.visuals(),
            );
            draw_node_primary_ports(
                &painter,
                node_rect,
                graph,
                layout_node.node_index,
                self.graph_view_zoom,
                ui.visuals(),
            );
        }

        if let Some((node_index, action, pointer_pos)) =
            hovered_ring_action.or(hovered_node_flag_action)
            && let Some(node) = graph.nodes.get(node_index)
        {
            draw_node_ring_action_tooltip(
                &painter,
                canvas_rect,
                pointer_pos,
                node,
                action,
                ui.visuals(),
            );
        }

        painter.text(
            canvas_rect.left_bottom() + egui::vec2(8.0, -8.0),
            Align2::LEFT_BOTTOM,
            format!("{:.0}%", self.graph_view_zoom * 100.0),
            FontId::monospace(10.0),
            ui.visuals().weak_text_color(),
        );

        response.context_menu(|ui| self.node_graph_context_menu_ui(ui, graph));
        self.node_graph_tab_menu_ui(ui, graph);

        response
    }

    fn finish_connection_drag(
        &mut self,
        graph: &mut GraphDocument,
        node_rects: &[Rect],
        connection_drag: ConnectionDragState,
        pointer_pos: Pos2,
    ) {
        let Some(port_hit) =
            node_primary_port_at(graph, node_rects, pointer_pos, self.graph_view_zoom)
        else {
            self.shelf_status = Some("Connection canceled.".to_owned());
            return;
        };
        if port_hit.kind != NodePortKind::Input {
            self.shelf_status = Some("Drop on a compatible input port.".to_owned());
            return;
        }
        let Some(target_node) = graph.nodes.get(port_hit.node_index) else {
            self.shelf_status = Some("Connection target disappeared.".to_owned());
            return;
        };
        let target_node_id = target_node.node_id.clone();
        let target_node_name = target_node.name.clone();
        match graph.add_data_flow_edge(
            &connection_drag.from_node_id,
            &connection_drag.from_output,
            &target_node_id,
            PRIMARY_GEOMETRY_OUTPUT,
        ) {
            Ok(_) => {
                self.select_single_node(port_hit.node_index);
                self.selected_edge = None;
                self.shelf_status = Some(format!(
                    "Connected {} to {}.",
                    graph
                        .nodes
                        .get(connection_drag.from_node_index)
                        .map(|node| node.name.as_str())
                        .unwrap_or("source"),
                    target_node_name
                ));
            }
            Err(diagnostic) => {
                self.shelf_status = Some(diagnostic.message);
            }
        }
    }

    fn node_graph_tab_menu_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        if !self.tab_menu_open {
            return;
        }
        if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.tab_menu_open = false;
            return;
        }

        let mut open = true;
        egui::Window::new("TAB Menu")
            .id(egui::Id::new("houdini_graph_canvas_tab_menu"))
            .fixed_pos(self.tab_menu_anchor)
            .collapsible(false)
            .resizable(false)
            .title_bar(true)
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(320.0);
                ui.horizontal(|ui| {
                    ui.weak("TAB");
                    let filter_response = egui::TextEdit::singleline(&mut self.operator_filter)
                        .desired_width(248.0)
                        .hint_text("operator")
                        .show(ui)
                        .response;
                    if self.tab_menu_filter_needs_focus {
                        filter_response.request_focus();
                        self.tab_menu_filter_needs_focus = false;
                    }
                    if filter_response.changed() {
                        self.tab_menu_selection_index = 0;
                    }
                    if ui.small_button("Clear").clicked() {
                        self.operator_filter.clear();
                        self.tab_menu_selection_index = 0;
                        filter_response.request_focus();
                    }
                });
                let matching_actions = self.matching_operator_palette_actions(graph, true, false);
                if self.tab_menu_selection_index >= matching_actions.len() {
                    self.tab_menu_selection_index = 0;
                }
                if !matching_actions.is_empty() {
                    ui.input(|input| {
                        if input.key_pressed(egui::Key::ArrowDown) {
                            self.tab_menu_selection_index =
                                (self.tab_menu_selection_index + 1) % matching_actions.len();
                        }
                        if input.key_pressed(egui::Key::ArrowUp) {
                            self.tab_menu_selection_index = if self.tab_menu_selection_index == 0 {
                                matching_actions.len() - 1
                            } else {
                                self.tab_menu_selection_index - 1
                            };
                        }
                    });
                }
                let highlighted_action =
                    matching_actions.get(self.tab_menu_selection_index).copied();
                let accepted_keyboard_action = ui
                    .input(|input| input.key_pressed(egui::Key::Enter))
                    && highlighted_action
                        .is_some_and(|action| self.apply_operator_palette_action(graph, action));
                if accepted_keyboard_action {
                    self.tab_menu_open = false;
                    return;
                }
                ui.separator();
                if self.operator_palette_ui(
                    ui,
                    graph,
                    OperatorPaletteUiOptions {
                        id_salt: "houdini_graph_canvas_tab_palette",
                        grouped: true,
                        show_recent: true,
                        include_organization: true,
                        include_layers: false,
                        highlighted_action,
                    },
                ) {
                    self.tab_menu_open = false;
                }
            });

        if !open {
            self.tab_menu_open = false;
        }
    }

    fn node_graph_context_menu_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        if self.selected_annotation.is_some() {
            self.annotation_context_menu_ui(ui, graph);
            return;
        }
        if self.context_menu_edge.is_some() {
            self.edge_context_menu_ui(ui, graph);
            return;
        }
        if self.context_menu_canvas {
            self.canvas_context_menu_ui(ui, graph);
            return;
        }

        if let Some(node) = graph.nodes.get(self.selected_node) {
            ui.strong(&node.name);
            ui.weak(node.kind.as_str());
            ui.separator();
        }

        if ui.button("Show Node Information").clicked() {
            self.node_info_open = true;
            self.active_graph_pane = GraphWorkbenchPane::Info;
            ui.close();
        }
        if ui.button("Pin Node Information").clicked() {
            self.node_info_open = true;
            self.node_info_pinned = true;
            self.active_graph_pane = GraphWorkbenchPane::Info;
            ui.close();
        }
        self.operator_menu_action_ui_with_label(
            ui,
            graph,
            OperatorPaletteAction::EnterSelectedSubnet,
            "Enter Subnet    Enter",
        );
        self.operator_menu_action_ui_with_label(
            ui,
            graph,
            OperatorPaletteAction::GoUpOneGraph,
            "Go Up    U",
        );
        self.operator_menu_action_ui(
            ui,
            graph,
            OperatorPaletteAction::CreateAssetFromSelectedSubnet,
        );
        if ui.button("Run Selected").clicked() {
            graph.request_node_run(self.selected_node);
            graph.complete_node_run(self.selected_node);
            ui.close();
        }
        if ui.button("Evaluate Output").clicked() {
            graph.demand_output_evaluation();
            ui.close();
        }
        if ui
            .add_enabled(
                self.selected_node_can_collapse_to_graph_container(graph),
                egui::Button::new("Collapse to Subnet"),
            )
            .on_hover_text("Move the selected node into a new typed graph container.")
            .clicked()
        {
            self.collapse_selected_node_to_graph_container(graph);
            ui.close();
        }

        ui.separator();
        ui.weak("Node Flags");
        self.node_flag_menu_action_ui(ui, graph, NodeRingAction::Info, "Node Info    I");
        self.node_flag_menu_action_ui(ui, graph, NodeRingAction::Display, "Display Output    Q");
        self.node_flag_menu_action_ui(ui, graph, NodeRingAction::Manual, "Manual Cook    M");
        self.node_flag_menu_action_ui(ui, graph, NodeRingAction::Run, "Run Node    R");

        ui.separator();
        ui.weak("Comment");
        if ui.button("Edit Comment").clicked() {
            self.node_info_open = true;
            self.active_graph_pane = GraphWorkbenchPane::Info;
            ui.close();
        }
        if let Some(node) = graph.nodes.get(self.selected_node) {
            let mut show_comment = node.show_comment_in_network;
            if ui
                .checkbox(&mut show_comment, "Show Comment in Network")
                .changed()
            {
                graph.set_node_comment_visibility(self.selected_node, show_comment);
                ui.close();
            }
        }

        ui.separator();
        if ui.button("TAB Menu...").clicked() {
            let anchor = ui
                .input(|input| input.pointer.hover_pos())
                .unwrap_or_else(|| ui.cursor().min);
            self.open_operator_chooser_at(anchor);
            ui.close();
        }
        ui.separator();
        self.operator_menu_action_ui(ui, graph, OperatorPaletteAction::AddOutNull);
        self.operator_menu_action_ui(ui, graph, OperatorPaletteAction::AddReference);
        self.operator_menu_action_ui(ui, graph, OperatorPaletteAction::AddRepairProjection);
        self.operator_menu_action_ui_with_label(
            ui,
            graph,
            OperatorPaletteAction::AddNetworkBox,
            "Network Box from Selected    Shift+O",
        );
        self.operator_menu_action_ui_with_label(
            ui,
            graph,
            OperatorPaletteAction::AddStickyNote,
            "Sticky Note    Shift+P",
        );

        ui.separator();
        if ui.button("Resize Box to Contents    Shift+M").clicked() {
            self.resize_selected_network_box_to_contents(graph);
            ui.close();
        }
        if ui.button("Display Options").clicked() {
            self.active_graph_pane = GraphWorkbenchPane::Display;
            toggle_network_display_options(ui);
            ui.close();
        }
        if ui.button("Reset View    H").clicked() {
            self.reset_graph_view();
            ui.close();
        }
        if ui.button("Frame Selected    F").clicked() {
            self.pending_frame_selected = true;
            ui.close();
        }
    }

    fn edge_context_menu_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        let Some(edge_id) = self.context_menu_edge.clone() else {
            return;
        };
        ui.strong("Connection");
        if let Some(readable_path) = graph.data_flow_edge_readable_path(&edge_id) {
            ui.weak(readable_path);
            ui.separator();
            if ui.button("Remove Connection").clicked() {
                if graph.remove_data_flow_edge(&edge_id).is_some() {
                    if self.selected_edge.as_deref() == Some(edge_id.as_str()) {
                        self.selected_edge = None;
                    }
                    self.context_menu_edge = None;
                }
                ui.close();
            }
        } else {
            ui.weak("Connection no longer exists.");
            if ui.button("Clear Selection").clicked() {
                self.selected_edge = None;
                self.context_menu_edge = None;
                ui.close();
            }
        }
    }

    fn canvas_context_menu_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.strong("Network");
        ui.weak(graph.current_graph_path());
        ui.separator();

        if ui.button("TAB Menu...").clicked() {
            let anchor = ui
                .input(|input| input.pointer.hover_pos())
                .unwrap_or_else(|| ui.cursor().min);
            self.open_operator_chooser_at(anchor);
            ui.close();
        }
        self.operator_menu_action_ui(ui, graph, OperatorPaletteAction::AddOutNull);
        self.operator_menu_action_ui(ui, graph, OperatorPaletteAction::AddReference);
        self.operator_menu_action_ui(ui, graph, OperatorPaletteAction::AddRepairProjection);
        self.operator_menu_action_ui_with_label(
            ui,
            graph,
            OperatorPaletteAction::AddNetworkBox,
            "Network Box from Selected    Shift+O",
        );
        self.operator_menu_action_ui_with_label(
            ui,
            graph,
            OperatorPaletteAction::AddStickyNote,
            "Sticky Note    Shift+P",
        );
        self.operator_menu_action_ui_with_label(
            ui,
            graph,
            OperatorPaletteAction::GoUpOneGraph,
            "Go Up    U",
        );

        ui.separator();
        if ui.button("Display Options").clicked() {
            self.active_graph_pane = GraphWorkbenchPane::Display;
            toggle_network_display_options(ui);
            ui.close();
        }
        if ui.button("Reset View    H").clicked() {
            self.reset_graph_view();
            ui.close();
        }
        if ui.button("Frame Selected    F").clicked() {
            self.pending_frame_selected = true;
            ui.close();
        }
    }

    fn annotation_context_menu_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        let Some(annotation_index) = self.selected_annotation else {
            return;
        };
        if annotation_index >= graph.annotations.len()
            || !graph.annotation_belongs_to_current_graph(annotation_index)
        {
            self.selected_annotation = None;
            ui.weak("No annotation selected.");
            return;
        }

        let mut resize_to_contents = false;
        let annotation = graph.annotations[annotation_index].clone();
        ui.strong(&annotation.title);
        ui.weak(annotation.kind.as_str());
        ui.separator();

        ui.weak("Title");
        let mut title = annotation.title.clone();
        if ui
            .add(
                egui::TextEdit::singleline(&mut title)
                    .desired_width(190.0)
                    .hint_text("title"),
            )
            .changed()
        {
            graph.set_annotation_title(annotation_index, title);
        }

        if annotation.kind == GraphAnnotationKind::StickyNote {
            ui.weak("Note");
            let mut text = annotation.text.clone();
            if ui
                .add(
                    egui::TextEdit::multiline(&mut text)
                        .desired_width(190.0)
                        .desired_rows(3)
                        .hint_text("note"),
                )
                .changed()
            {
                graph.set_annotation_text(annotation_index, text);
            }
        }

        ui.separator();
        let collapse_label = if annotation.collapsed {
            "Expand"
        } else {
            "Collapse"
        };
        if ui.button(collapse_label).clicked() {
            graph.set_annotation_collapsed(annotation_index, !annotation.collapsed);
            ui.close();
        }

        if annotation.kind == GraphAnnotationKind::NetworkBox {
            ui.weak(format!(
                "{} member node{}",
                annotation.member_node_ids.len(),
                if annotation.member_node_ids.len() == 1 {
                    ""
                } else {
                    "s"
                }
            ));
            if ui.button("Resize to Contents    Shift+M").clicked() {
                resize_to_contents = true;
                ui.close();
            }
        }

        if resize_to_contents {
            graph.resize_network_box_to_contents(annotation_index);
        }

        ui.separator();
        if ui.button("Delete").clicked() {
            if graph.remove_annotation(annotation_index).is_some() {
                self.selected_annotation = None;
            }
            ui.close();
        }
        if !graph.annotations.is_empty() {
            ui.separator();
        }
        if ui.button("Organization Pane").clicked() {
            self.active_graph_pane = GraphWorkbenchPane::Operators;
            ui.close();
        }
    }

    fn node_flag_menu_action_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        action: NodeRingAction,
        label: &str,
    ) {
        let selected = match action {
            NodeRingAction::Display => graph
                .nodes
                .get(self.selected_node)
                .is_some_and(|node| node.participates_in_output),
            NodeRingAction::Manual => graph
                .nodes
                .get(self.selected_node)
                .is_some_and(|node| node.evaluation.manual),
            NodeRingAction::Info | NodeRingAction::Run => false,
        };
        if ui
            .selectable_label(selected, label)
            .on_hover_text(action.detail())
            .clicked()
        {
            self.apply_node_ring_action(graph, self.selected_node, action);
            ui.close();
        }
    }

    fn apply_node_ring_action(
        &mut self,
        graph: &mut GraphDocument,
        node_index: usize,
        action: NodeRingAction,
    ) {
        match action {
            NodeRingAction::Info => {
                self.node_info_open = true;
                self.active_graph_pane = GraphWorkbenchPane::Info;
            }
            NodeRingAction::Display => {
                if let Some(node) = graph.nodes.get(node_index) {
                    graph.set_node_output_participation(node_index, !node.participates_in_output);
                }
            }
            NodeRingAction::Manual => {
                let Some(node) = graph.nodes.get(node_index) else {
                    return;
                };
                graph.set_node_manual(node_index, !node.evaluation.manual);
            }
            NodeRingAction::Run => {
                graph.request_node_run(node_index);
                graph.complete_node_run(node_index);
            }
        }
    }

    fn update_graph_viewport(&mut self, ui: &mut Ui, layout_rect: Rect) {
        let Some(pointer_pos) = ui.input(|input| input.pointer.hover_pos()) else {
            return;
        };

        let (zoom_delta, scroll_delta) =
            ui.input(|input| (input.zoom_delta(), input.smooth_scroll_delta()));
        if scroll_delta != Vec2::ZERO {
            ui.input_mut(|input| {
                input.smooth_scroll_delta = Vec2::ZERO;
            });
        }

        let wheel_zoom_delta = if scroll_delta.y.abs() > 0.0 {
            (scroll_delta.y / 360.0).exp()
        } else {
            1.0
        };
        let combined_zoom_delta = zoom_delta * wheel_zoom_delta;
        if (combined_zoom_delta - 1.0).abs() <= f32::EPSILON {
            return;
        }

        let previous_zoom = self.graph_view_zoom;
        let new_zoom = (self.graph_view_zoom * combined_zoom_delta).clamp(0.45, 2.6);
        if (new_zoom - previous_zoom).abs() <= f32::EPSILON {
            return;
        }

        let center = layout_rect.center();
        self.graph_view_pan = pointer_pos
            - center
            - (pointer_pos - center - self.graph_view_pan) * new_zoom / previous_zoom;
        self.graph_view_zoom = new_zoom;
    }

    fn node_info_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        if let Some(info) = graph.selected_node_info(self.selected_node) {
            egui::Grid::new("houdini_graph_node_info")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    ui.weak("Kind");
                    ui.label(info.kind.as_str());
                    ui.end_row();

                    ui.weak("Role");
                    ui.label(info.role);
                    ui.end_row();

                    ui.weak("Graph");
                    ui.label(&info.graph_location.graph_path);
                    ui.end_row();

                    ui.weak("Graph id");
                    ui.label(&info.graph_location.graph_id);
                    ui.end_row();

                    ui.weak("Node path");
                    ui.label(&info.graph_location.node_path);
                    ui.end_row();

                    ui.weak("Name scope");
                    if info.graph_location.name_is_unique_in_graph() {
                        ui.label("unique in graph");
                    } else {
                        ui.colored_label(
                            ui.visuals().warn_fg_color,
                            format!(
                                "shared by {} nodes in graph",
                                info.graph_location.name_collision_count
                            ),
                        );
                    }
                    ui.end_row();

                    ui.weak("Incoming edges");
                    ui.label(info.data_flow.incoming_edge_count.to_string());
                    ui.end_row();

                    ui.weak("Outgoing edges");
                    ui.label(info.data_flow.outgoing_edge_count.to_string());
                    ui.end_row();

                    ui.weak("Edge diagnostics");
                    if info.data_flow.diagnostics.is_empty() {
                        ui.label("none");
                    } else {
                        ui.vertical(|ui| {
                            for diagnostic in &info.data_flow.diagnostics {
                                ui.colored_label(
                                    ui.visuals().warn_fg_color,
                                    format!(
                                        "{}: {}",
                                        diagnostic.status.as_str(),
                                        diagnostic.readable_path
                                    ),
                                )
                                .on_hover_text(&diagnostic.message);
                            }
                        });
                    }
                    ui.end_row();

                    ui.weak("Inputs");
                    ui.label(info.input_count.to_string());
                    ui.end_row();

                    ui.weak("Outputs");
                    ui.label(info.output_count.to_string());
                    ui.end_row();

                    ui.weak("Status");
                    ui.colored_label(status_color(ui, info.status), info.status.as_str());
                    ui.end_row();

                    ui.weak("Data");
                    ui.label(info.data_kind);
                    ui.end_row();

                    ui.weak("Records");
                    ui.label(info.record_count.to_string());
                    ui.end_row();

                    ui.weak("Bounds");
                    ui.label(format_bounds(info.bounds.as_ref()));
                    ui.end_row();

                    if let Some(provenance) = info.provenance {
                        ui.weak("Provenance");
                        ui.label(provenance.as_str());
                        ui.end_row();
                    }

                    ui.weak("Attributes");
                    ui.label(format_list(&info.attributes));
                    ui.end_row();

                    if let Some(generated) = info.generated {
                        ui.weak("Generated");
                        ui.colored_label(ui.visuals().warn_fg_color, generated.as_str());
                        ui.end_row();

                        ui.weak("Layer binding");
                        ui.colored_label(
                            ui.visuals().warn_fg_color,
                            generated.binding_state.as_str(),
                        )
                        .on_hover_text(generated.binding_state.description());
                        ui.end_row();
                    }

                    ui.weak("Eval");
                    ui.colored_label(
                        evaluation_color(ui, info.evaluation.state),
                        info.evaluation.state.as_str(),
                    );
                    ui.end_row();

                    if let Some(message) = &info.evaluation.message {
                        ui.weak("Eval note");
                        ui.label(message);
                        ui.end_row();
                    }

                    ui.weak("Parameter");
                    ui.label(format!(
                        "{} = {:.2}",
                        info.parameter.name, info.parameter.value
                    ));
                    ui.end_row();

                    ui.weak("Type");
                    ui.label(info.parameter.kind.as_str());
                    ui.end_row();

                    if let Some(rule) = info.parameter.as_attribute_rule() {
                        ui.weak("Rule");
                        ui.label(format!(
                            "{} {} {:.2}",
                            rule.attribute_name,
                            rule.comparison.as_str(),
                            rule.value.as_f32().unwrap_or(info.parameter.value)
                        ));
                        ui.end_row();
                    }

                    if let Some(style) = info.style {
                        ui.weak("Style");
                        ui.label(format_style(style));
                        ui.end_row();
                    }

                    if let Some(null_operator) = &info.null_operator {
                        ui.weak("Convention");
                        ui.label(null_operator.convention.as_str());
                        ui.end_row();

                        ui.weak("Pass-through");
                        ui.label(format!(
                            "{:?} -> {:?}",
                            null_operator.input_kind, null_operator.output_kind
                        ));
                        ui.end_row();

                        ui.weak("Preserves");
                        ui.label(format!(
                            "records: {}, provenance: {}",
                            null_operator.preserves_record_identity,
                            null_operator.preserves_source_provenance
                        ));
                        ui.end_row();
                    }

                    if let Some(reference_input) = &info.reference_input {
                        ui.weak("Target");
                        ui.label(&reference_input.readable_path);
                        ui.end_row();

                        ui.weak("Target status");
                        ui.colored_label(
                            status_color(ui, info.status),
                            reference_input.status.as_str(),
                        );
                        ui.end_row();

                        ui.weak("Target id");
                        ui.label(format!(
                            "{}/{}:{}",
                            reference_input.target.graph_id,
                            reference_input.target.node_id,
                            reference_input.target.output_name
                        ));
                        ui.end_row();

                        ui.weak("Target data");
                        ui.label(
                            reference_input
                                .output_kind
                                .map(|kind| format!("{kind:?}"))
                                .unwrap_or_else(|| "missing".to_owned()),
                        );
                        ui.end_row();

                        ui.weak("Coordinates");
                        ui.label(
                            reference_input
                                .coordinate_contract
                                .as_ref()
                                .map(format_coordinate_contract)
                                .unwrap_or_else(|| "missing".to_owned()),
                        );
                        ui.end_row();

                        if let Some(provenance) = reference_input.source_provenance {
                            ui.weak("Target provenance");
                            ui.label(provenance.as_str());
                            ui.end_row();
                        }

                        ui.weak("Reference mode");
                        ui.label(format!(
                            "copy: {}, hidden transform: {}",
                            !reference_input.preserves_source_data,
                            reference_input.applies_hidden_transform
                        ));
                        ui.end_row();

                        ui.weak("Target set");
                        ui.label(format!("{} target(s)", reference_input.targets.len()));
                        ui.end_row();

                        for (index, target) in reference_input.targets.iter().enumerate() {
                            ui.weak(format!("Target {}", index + 1));
                            ui.label(format!(
                                "{} [{} / {}], {} record(s), id {}/{}:{}, provenance {} / {}:{}, source {}",
                                target.readable_path,
                                if target.enabled { "enabled" } else { "disabled" },
                                target.status.as_str(),
                                target.record_count,
                                target.target.graph_id,
                                target.target.node_id,
                                target.target.output_name,
                                target.provenance.source_graph_id,
                                target.provenance.source_node_name,
                                target.provenance.source_output_name,
                                target
                                    .source_provenance
                                    .map(|provenance| provenance.as_str())
                                    .unwrap_or("missing")
                            ));
                            ui.end_row();

                            ui.weak("Target kind");
                            ui.label(
                                target
                                    .output_kind
                                    .map(|kind| format!("{kind:?}"))
                                    .unwrap_or_else(|| "missing".to_owned()),
                            );
                            ui.end_row();

                            if let Some(diagnostic) = &target.diagnostic {
                                ui.weak("Target diagnostic");
                                ui.colored_label(ui.visuals().warn_fg_color, diagnostic);
                                ui.end_row();
                            }

                            if let Some(target_node_index) = target.target_node_index {
                                ui.weak("Target navigation");
                                ui.push_id(("jump_source", index), |ui| {
                                    if ui.button("Jump source").clicked() {
                                        self.select_single_node(target_node_index);
                                    }
                                });
                                ui.end_row();
                            }

                            ui.weak("Target coordinates");
                            ui.label(
                                target
                                    .coordinate_contract
                                    .as_ref()
                                    .map(format_coordinate_contract)
                                    .unwrap_or_else(|| "missing".to_owned()),
                            );
                            ui.end_row();

                            if let Some(expected) = &target.expected_coordinate_contract {
                                ui.weak("Expected coordinates");
                                ui.label(format_coordinate_contract(expected));
                                ui.end_row();
                            }
                        }
                    }

                    if !info.reference_consumers.is_empty() {
                        ui.weak("Reference consumers");
                        ui.label(format!("{} consumer(s)", info.reference_consumers.len()));
                        ui.end_row();

                        for consumer in &info.reference_consumers {
                            ui.weak("Consumer");
                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "{} [{} / {}] -> {}",
                                    consumer.reference_node_name,
                                    if consumer.enabled { "enabled" } else { "disabled" },
                                    consumer.status.as_str(),
                                    consumer.target_output_name
                                ));
                                ui.push_id(
                                    ("jump_consumer", consumer.reference_node_index),
                                    |ui| {
                                        if ui.button("Jump consumer").clicked() {
                                            self.select_single_node(
                                                consumer.reference_node_index,
                                            );
                                        }
                                    },
                                );
                            });
                            ui.end_row();

                            if let Some(diagnostic) = &consumer.diagnostic {
                                ui.weak("Consumer diagnostic");
                                ui.colored_label(ui.visuals().warn_fg_color, diagnostic);
                                ui.end_row();
                            }
                        }
                    }

                    if let Some(warning) = &info.reference_output_warning {
                        ui.weak("Output warning");
                        ui.colored_label(
                            ui.visuals().warn_fg_color,
                            format!(
                                "Changing or deleting {}:{} affects {} reference(s)",
                                warning.target_node_path,
                                warning.output_name,
                                warning.affected_references.len()
                            ),
                        );
                        ui.end_row();
                    }

                    if let Some(output_operator) = &info.output_operator {
                        ui.weak("Output operator");
                        ui.label(output_operator.kind.as_str());
                        ui.end_row();

                        ui.weak("Payload");
                        ui.label(output_operator.semantic_payload.as_str());
                        ui.end_row();

                        ui.weak("Command");
                        ui.label(output_operator.command.as_str());
                        ui.end_row();

                        ui.weak("Preferred target");
                        ui.label(
                            output_operator
                                .preferred_target
                                .map(|target| target.as_str())
                                .unwrap_or("none"),
                        );
                        ui.end_row();

                        for negotiation in &output_operator.negotiations {
                            ui.weak(format!("Target {}", negotiation.target.as_str()));
                            ui.label(format!(
                                "{}: {}",
                                negotiation.mapping.as_str(),
                                negotiation.reason
                            ));
                            ui.end_row();
                        }

                        if let Some(rerun_options) = &output_operator.rerun_options {
                            ui.weak("Rerun options");
                            ui.label(format!(
                                "debug: {}, cubic metadata: {}",
                                rerun_options.include_debug_items,
                                rerun_options.preserve_native_cubic_metadata
                            ));
                            ui.end_row();
                        }

                        ui.weak("Viewport state");
                        ui.label(if output_operator.graph_viewport_state_separate {
                            "graph editing viewport is separate"
                        } else {
                            "stored on output operator"
                        });
                        ui.end_row();
                    }

                    if let Some(python_operator) = &info.python_operator {
                        ui.weak("Operator");
                        ui.label(format!(
                            "{} ({})",
                            python_operator.display_name, python_operator.declaration_id
                        ));
                        ui.end_row();

                        ui.weak("Version");
                        ui.label(&python_operator.version);
                        ui.end_row();

                        ui.weak("Dependencies");
                        ui.colored_label(
                            python_operator_dependency_color(ui, python_operator.dependency_status),
                            python_operator.dependency_status.as_str(),
                        );
                        ui.end_row();

                        ui.weak("Requirements");
                        ui.label(format_list(&python_operator.requirements));
                        ui.end_row();

                        if let Some(provenance) = &python_operator.provenance_summary {
                            ui.weak("Python provenance");
                            ui.label(provenance);
                            ui.end_row();
                        }

                        if let Some(cache_key) = &python_operator.cache_key_summary {
                            ui.weak("Cache key");
                            ui.label(cache_key);
                            ui.end_row();
                        }

                        if let Some(last_failure) = &python_operator.last_failure_summary {
                            ui.weak("Last failure");
                            ui.label(last_failure);
                            ui.end_row();
                        }
                    }

                    if let Some(asset) = &info.procedural_asset {
                        ui.weak("Asset");
                        ui.label(format!("{} ({})", asset.display_name, asset.asset_id));
                        ui.end_row();

                        ui.weak("Version");
                        ui.label(format!(
                            "instance {} / current {} / {} / {}",
                            asset.instance_version,
                            asset.current_version.as_deref().unwrap_or("missing"),
                            asset.version_status.as_str(),
                            if asset.contents_unlocked {
                                "unlocked"
                            } else {
                                "matched"
                            }
                        ));
                        ui.end_row();

                        if let Some(local_graph_id) = &asset.local_graph_id {
                            ui.weak("Local graph");
                            ui.label(local_graph_id);
                            ui.end_row();
                        }

                        ui.weak("Labels");
                        ui.label(format_list(&asset.labels));
                        ui.end_row();

                        ui.weak("Description");
                        ui.label(&asset.description);
                        ui.end_row();

                        ui.weak("Promoted");
                        ui.label(format_list(&asset.promoted_parameters));
                        ui.end_row();

                        ui.weak("Bindings");
                        ui.label(format_bindings(&asset.input_bindings));
                        ui.end_row();

                        if let Some(output_summary) = &asset.output_summary {
                            ui.weak("Asset output");
                            ui.label(output_summary);
                            ui.end_row();
                        }
                    }

                    if let Some(native_operator) = &info.native_operator {
                        ui.weak("Native");
                        ui.label(format!(
                            "{} ({})",
                            native_operator.display_name, native_operator.operator_id
                        ));
                        ui.end_row();

                        ui.weak("Version");
                        ui.label(format!(
                            "{} / {}",
                            native_operator.version,
                            native_operator.version_status.as_str()
                        ));
                        ui.end_row();

                        ui.weak("Host");
                        ui.label(&native_operator.host_compatibility_version);
                        ui.end_row();

                        ui.weak("Load");
                        ui.label(native_operator.load_status.as_str());
                        ui.end_row();

                        ui.weak("Inputs");
                        ui.label(format_list(&native_operator.inputs));
                        ui.end_row();

                        ui.weak("Outputs");
                        ui.label(format_list(&native_operator.outputs));
                        ui.end_row();

                        ui.weak("Parameters");
                        ui.label(format_list(&native_operator.parameters));
                        ui.end_row();

                        ui.weak("Capabilities");
                        ui.label(format_list(&native_operator.capabilities));
                        ui.end_row();

                        ui.weak("Provenance");
                        ui.label(&native_operator.provenance_summary);
                        ui.end_row();

                        if let Some(output_provenance) = &native_operator.output_provenance_summary
                        {
                            ui.weak("Last output provenance");
                            ui.label(output_provenance);
                            ui.end_row();
                        }

                        if let Some(cache_key) = &native_operator.cache_key_summary {
                            ui.weak("Cache key");
                            ui.label(cache_key);
                            ui.end_row();
                        }

                        ui.weak("Failure modes");
                        ui.label(format_list(&native_operator.failure_modes));
                        ui.end_row();

                        if let Some(cache_key) = &native_operator.last_valid_cache_key {
                            ui.weak("Last valid cache");
                            ui.label(cache_key);
                            ui.end_row();
                        }

                        if let Some(last_failure) = &native_operator.last_failure_summary {
                            ui.weak("Last failure");
                            ui.label(last_failure);
                            ui.end_row();
                        }
                    }
                });
            if let Some(native_operator) = &info.native_operator {
                self.native_operator_trust_controls_ui(ui, graph, native_operator);
            }
            ui.label(info.summary);
            if let Some(python_operator) = &info.python_operator {
                ui.weak(&python_operator.dependency_summary);
            }
            for warning in &info.warnings {
                ui.colored_label(ui.visuals().warn_fg_color, warning);
            }
            if let Some(source_error) = &info.source_error {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    format!("Source error: {source_error}"),
                );
            }
            if let Some(source_metadata) = &info.source_metadata {
                self.source_metadata_ui(ui, source_metadata, "node_info");
            }
        }
    }

    fn native_operator_trust_controls_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut GraphDocument,
        native_operator: &self::model::NativeOperatorNodeInfo,
    ) {
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            let mut project_trusted = native_operator.project_trusted;
            if ui
                .re_checkbox(&mut project_trusted, "Trust native operators")
                .changed()
            {
                graph.set_native_operator_project_trusted(project_trusted);
            }

            let mut operator_enabled = native_operator.operator_enabled;
            if ui
                .re_checkbox(&mut operator_enabled, "Enable this operator")
                .changed()
            {
                graph.set_native_operator_enabled(&native_operator.operator_id, operator_enabled);
            }
        });

        if !native_operator.capability_grants.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.weak("Capability grants");
                for grant in &native_operator.capability_grants {
                    let mut granted = grant.granted;
                    if ui.re_checkbox(&mut granted, &grant.label).changed() {
                        graph.set_native_operator_capability_grant(grant.capability, granted);
                    }
                }
            });
        }

        if native_operator.load_status != NativeOperatorLoadStatus::Ready {
            ui.colored_label(
                native_operator_load_status_color(ui, native_operator.load_status),
                native_operator.load_status.summary(),
            );
            if !native_operator.missing_capability_grants.is_empty() {
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    format!(
                        "Missing grants: {}",
                        format_list(&native_operator.missing_capability_grants)
                    ),
                );
            }
        }
    }

    fn source_metadata_ui(
        &mut self,
        ui: &mut Ui,
        metadata: &SourceMetadata,
        id_suffix: &'static str,
    ) {
        let external_reference = metadata.external_reference_report();
        let bundle_preview = metadata.bundle_preview();
        let package_manifest = metadata.package_manifest_preview();
        let format_report = metadata.source_format_inference_report();
        let action_report = metadata.external_reference_action_report();
        egui::Grid::new(("houdini_graph_source_metadata", id_suffix))
            .num_columns(2)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.weak("Records");
                ui.label(metadata.record_count.to_string());
                ui.end_row();

                ui.weak("Geometry");
                ui.label(format!(
                    "{} polygons, {} cubic Beziers",
                    metadata.polygon_count, metadata.cubic_bezier_count
                ));
                ui.end_row();

                ui.weak("Locator");
                ui.label(metadata.locator.kind.as_str());
                ui.end_row();

                ui.weak("Location");
                ui.label(metadata.locator.readable());
                ui.end_row();

                ui.weak("External");
                ui.label(yes_no(metadata.locator.is_external_reference()));
                ui.end_row();

                ui.weak("Generated");
                ui.label(yes_no(metadata.locator.is_generated()));
                ui.end_row();

                ui.weak("Reference");
                ui.label(external_reference.status.as_str());
                ui.end_row();

                ui.weak("Bundle item");
                ui.label(yes_no(external_reference.bundle_relevant));
                ui.end_row();

                if let Some(warning) = &external_reference.warning {
                    ui.weak("Reference warning");
                    ui.colored_label(ui.visuals().warn_fg_color, warning);
                    ui.end_row();
                }

                ui.weak("Bundle preview");
                ui.label(bundle_preview.item.inclusion.as_str());
                ui.end_row();

                ui.weak("Bundle size");
                ui.label(
                    bundle_preview
                        .expected_size_bytes
                        .map(|size| format!("{size} bytes"))
                        .unwrap_or_else(|| "unknown".to_owned()),
                );
                ui.end_row();

                if !bundle_preview.reproducibility_warnings.is_empty() {
                    ui.weak("Bundle warning");
                    ui.colored_label(
                        ui.visuals().warn_fg_color,
                        format_list(&bundle_preview.reproducibility_warnings),
                    );
                    ui.end_row();
                }

                ui.weak("Manifest items");
                ui.label(package_manifest.artifacts.len().to_string());
                ui.end_row();

                ui.weak("Manifest external");
                ui.label(
                    package_manifest
                        .remaining_external_reference_count
                        .to_string(),
                );
                ui.end_row();

                ui.weak("Manifest missing");
                ui.label(package_manifest.missing_reference_count.to_string());
                ui.end_row();

                ui.weak("Source format");
                ui.label(
                    format_report
                        .kind
                        .map(|kind| kind.as_str())
                        .unwrap_or_else(|| format_report.status.as_str()),
                );
                ui.end_row();

                ui.weak("Format support");
                ui.label(
                    format_report
                        .support_status
                        .map(|status| status.as_str())
                        .unwrap_or("not applicable"),
                );
                ui.end_row();

                ui.weak("Reference action");
                ui.horizontal(|ui| {
                    let locator = metadata.locator.readable();
                    ui.label(&action_report.recommended.label);
                    if copy_locator_action_hint(&action_report).is_some()
                        && ui.small_button("Copy locator").clicked()
                    {
                        ui.copy_text(locator.clone());
                        self.source_reference_copied_locator = Some(locator.clone());
                    }
                    if self.source_reference_copied_locator.as_deref() == Some(locator.as_str()) {
                        ui.weak("Copied locator");
                    }
                });
                ui.end_row();

                ui.weak("Bounds");
                ui.label(format_bounds(metadata.bounds.as_ref()));
                ui.end_row();

                ui.weak("Attributes");
                ui.label(format_list(&metadata.attribute_names));
                ui.end_row();

                ui.weak("Control columns");
                ui.label(format_list(&metadata.recognized_control_point_columns));
                ui.end_row();
            });
    }

    fn pipeline_trace_ui(&self, ui: &mut Ui, graph: &GraphDocument) {
        egui::Grid::new("houdini_graph_pipeline_trace")
            .num_columns(4)
            .spacing([10.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.weak("Stage");
                ui.weak("In");
                ui.weak("Out");
                ui.weak("Operation");
                ui.end_row();

                for stage in graph.pipeline_stages() {
                    ui.label(stage.name);
                    ui.label(stage.input_count.to_string());
                    ui.label(stage.output_count.to_string());
                    ui.label(stage.note);
                    ui.end_row();
                }
            });
    }

    fn attribute_table_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.horizontal(|ui| {
            ui.label("Search");
            ui.add(
                egui::TextEdit::singleline(&mut self.table_search)
                    .desired_width(160.0)
                    .hint_text("kind, layer, source"),
            );

            ui.re_checkbox(&mut self.table_minimum_score_enabled, "Min score");
            if self.table_minimum_score_enabled {
                ui.add(
                    Slider::new(&mut self.table_minimum_score, 0.0..=1.0)
                        .text("")
                        .show_value(true),
                );
            }
        });

        ui.horizontal(|ui| {
            ui.weak("Sort");
            for sort in [
                AttributeTableSort::RecordIndex,
                AttributeTableSort::GeometryKind,
                AttributeTableSort::Score,
                AttributeTableSort::Layer,
            ] {
                if ui
                    .selectable_label(self.table_sort == sort, sort.as_str())
                    .clicked()
                {
                    if self.table_sort == sort {
                        self.table_sort_descending = !self.table_sort_descending;
                    } else {
                        self.table_sort = sort;
                        self.table_sort_descending = false;
                    }
                }
            }
            ui.weak(if self.table_sort_descending {
                "descending"
            } else {
                "ascending"
            });
        });

        let query = AttributeTableQuery {
            search: self.table_search.clone(),
            minimum_score: self
                .table_minimum_score_enabled
                .then_some(self.table_minimum_score),
            sort: self.table_sort,
            sort_descending: self.table_sort_descending,
        };

        ui.horizontal(|ui| {
            let commit_enabled = query.minimum_score.is_some();
            ui.add_enabled_ui(commit_enabled, |ui| {
                if ui.button("Commit min score to Filter").clicked()
                    && graph.commit_attribute_table_query_as_filter(&query)
                {
                    self.table_commit_status = Some(format!(
                        "Committed score >= {:.2} to Filter node",
                        query.minimum_score.unwrap_or_default()
                    ));
                }
            });
            if !commit_enabled {
                ui.weak("Enable Min score to commit a graph-backed filter");
            }
        });
        if let Some(status) = &self.table_commit_status {
            ui.weak(status);
        }
        if let Some(status) = &self.table_selection_status {
            ui.weak(status);
        }

        let visible_output_count = graph.visible_output_count();
        let use_preview = visible_output_count > LARGE_ATTRIBUTE_TABLE_ROW_LIMIT;
        let rows = if use_preview {
            graph.attribute_table_preview_rows(&query, ATTRIBUTE_TABLE_PREVIEW_ROWS)
        } else {
            graph.attribute_table_rows(&query)
        };

        if use_preview {
            ui.weak(format!(
                "Large table preview: showing first {} of about {} visible records; commit filters still update the graph",
                rows.len(),
                visible_output_count
            ));
            if self.table_sort != AttributeTableSort::RecordIndex || self.table_sort_descending {
                ui.weak("Large previews are record-order only; use graph filters to narrow before sorting.");
            }
        } else {
            ui.weak(format!(
                "{} visible read-only records; table filters do not change graph output",
                rows.len()
            ));
        }
        let selected_row =
            self.table_selected_record_fingerprint
                .as_ref()
                .and_then(|fingerprint| {
                    rows.iter()
                        .find(|row| row.geometry_fingerprint == *fingerprint)
                });
        ui.horizontal(|ui| {
            ui.add_enabled_ui(selected_row.is_some(), |ui| {
                if ui.button("Commit selected row to Selection").clicked() {
                    if let Some(row) = selected_row {
                        let report = graph.transient_table_selection_for_row(row);
                        match graph.commit_transient_selection_as_subset(&report) {
                            Ok(node_index) => {
                                self.set_selected_node_set(vec![node_index]);
                                self.table_selection_status =
                                    Some("committed: created graph Selection node".to_owned());
                            }
                            Err(err) => {
                                self.table_selection_status = Some(err.summary());
                            }
                        }
                    }
                }
            });
            if selected_row.is_none() {
                ui.weak("Select a visible row to commit graph-backed subset data");
            }
        });
        egui::ScrollArea::vertical()
            .id_salt("houdini_graph_attribute_table")
            .max_height(160.0)
            .show(ui, |ui| {
                egui::Grid::new("houdini_graph_attribute_table_grid")
                    .num_columns(7)
                    .spacing([10.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.weak("Index");
                        ui.weak("Kind");
                        ui.weak("Score");
                        ui.weak("Layer");
                        ui.weak("Points");
                        ui.weak("Provenance");
                        ui.weak("Source");
                        ui.end_row();

                        for row in rows {
                            self.attribute_table_row_ui(ui, graph, &row);
                        }
                    });
            });
    }

    fn attribute_table_row_ui(
        &mut self,
        ui: &mut Ui,
        graph: &GraphDocument,
        row: &AttributeTableRow,
    ) {
        let selected = self.table_selected_record_fingerprint.as_deref()
            == Some(row.geometry_fingerprint.as_str());
        if ui
            .selectable_label(selected, row.record_index.to_string())
            .on_hover_text("Select this read-only record for graph identity inspection.")
            .clicked()
        {
            self.table_selected_record_fingerprint = Some(row.geometry_fingerprint.clone());
            self.table_selection_status =
                Some(graph.transient_table_selection_for_row(row).summary());
        }
        ui.label(row.geometry_kind.as_str());
        ui.label(format!("{:.2}", row.score));
        ui.label(row.layer.as_str());
        ui.label(row.point_count.to_string());
        ui.label(row.provenance.as_str());
        ui.label(row.source_path.as_deref().unwrap_or("none"));
        ui.end_row();
    }
}

fn sample_parquet_path() -> &'static std::path::Path {
    std::path::Path::new("crates/viewer/re_viewer/data/houdini_cubic_sample.parquet")
}

fn format_bounds(bounds: Option<&GeometryBounds>) -> String {
    bounds.map_or_else(
        || "none".to_owned(),
        |bounds| {
            format!(
                "({:.2}, {:.2}) - ({:.2}, {:.2})",
                bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y
            )
        },
    )
}

fn format_style(style: GraphStyle) -> String {
    format!(
        "rgb({}, {}, {}), opacity {:.2}, stroke {:.2}",
        style.color.r, style.color.g, style.color.b, style.opacity, style.stroke_scale
    )
}

fn format_coordinate_contract(contract: &SubstrateCoordinateContract) -> String {
    format!(
        "{} {}x{} {:?}/{:?}",
        contract.substrate_id, contract.width, contract.height, contract.origin, contract.y_axis
    )
}

fn status_color(ui: &Ui, status: NodeStatus) -> Color32 {
    match status {
        NodeStatus::Healthy => ui.visuals().text_color(),
        NodeStatus::Warning => ui.visuals().warn_fg_color,
        NodeStatus::Failed => ui.visuals().error_fg_color,
    }
}

fn evaluation_color(ui: &Ui, state: EvaluationState) -> Color32 {
    match state {
        EvaluationState::Clean => ui.visuals().text_color(),
        EvaluationState::Cached => ui.visuals().weak_text_color(),
        EvaluationState::Stale | EvaluationState::Manual => ui.visuals().warn_fg_color,
        EvaluationState::Running => ui.visuals().selection.stroke.color,
        EvaluationState::Failed => ui.visuals().error_fg_color,
    }
}

fn work_item_status_color(ui: &Ui, status: GraphWorkItemStatus) -> Color32 {
    match status {
        GraphWorkItemStatus::Waiting | GraphWorkItemStatus::Superseded => {
            ui.visuals().warn_fg_color
        }
        GraphWorkItemStatus::Running => ui.visuals().selection.stroke.color,
        GraphWorkItemStatus::Cached => ui.visuals().weak_text_color(),
        GraphWorkItemStatus::Canceled | GraphWorkItemStatus::Failed => ui.visuals().error_fg_color,
        GraphWorkItemStatus::Complete => ui.visuals().text_color(),
    }
}

fn python_environment_status_color(ui: &Ui, status: PythonEnvironmentStatus) -> Color32 {
    match status {
        PythonEnvironmentStatus::Ready => ui.visuals().text_color(),
        PythonEnvironmentStatus::Resolving | PythonEnvironmentStatus::Locked => {
            ui.visuals().selection.stroke.color
        }
        PythonEnvironmentStatus::Missing
        | PythonEnvironmentStatus::Unlocked
        | PythonEnvironmentStatus::Stale
        | PythonEnvironmentStatus::Disabled => ui.visuals().warn_fg_color,
        PythonEnvironmentStatus::Failed => ui.visuals().error_fg_color,
    }
}

fn python_operator_dependency_color(ui: &Ui, status: PythonOperatorDependencyStatus) -> Color32 {
    match status {
        PythonOperatorDependencyStatus::Ready => ui.visuals().text_color(),
        PythonOperatorDependencyStatus::ResolvingEnvironment => ui.visuals().selection.stroke.color,
        PythonOperatorDependencyStatus::DeclarationMissing
        | PythonOperatorDependencyStatus::FailedEnvironment => ui.visuals().error_fg_color,
        PythonOperatorDependencyStatus::MissingEnvironment
        | PythonOperatorDependencyStatus::StaleEnvironment
        | PythonOperatorDependencyStatus::DisabledEnvironment => ui.visuals().warn_fg_color,
    }
}

fn reference_status_color(ui: &Ui, status: ReferenceDiagnosticStatus) -> Color32 {
    match status {
        ReferenceDiagnosticStatus::Resolved => ui.visuals().text_color(),
        ReferenceDiagnosticStatus::CoordinateIncompatibleRepairable => ui.visuals().warn_fg_color,
        ReferenceDiagnosticStatus::MissingNode
        | ReferenceDiagnosticStatus::MissingOutput
        | ReferenceDiagnosticStatus::DisallowedBoundary
        | ReferenceDiagnosticStatus::AssetPrivateInternal
        | ReferenceDiagnosticStatus::CoordinateContractMissing => ui.visuals().error_fg_color,
    }
}

fn native_operator_load_status_color(ui: &Ui, status: NativeOperatorLoadStatus) -> Color32 {
    match status {
        NativeOperatorLoadStatus::Ready => ui.visuals().text_color(),
        NativeOperatorLoadStatus::TrustRequired
        | NativeOperatorLoadStatus::MissingCapabilityGrant => ui.visuals().warn_fg_color,
        NativeOperatorLoadStatus::DeclarationMissing
        | NativeOperatorLoadStatus::HostIncompatible
        | NativeOperatorLoadStatus::ImplementationDigestMissing
        | NativeOperatorLoadStatus::LoadFailed
        | NativeOperatorLoadStatus::RuntimeFailed
        | NativeOperatorLoadStatus::TimedOut
        | NativeOperatorLoadStatus::OutputSchemaMismatch => ui.visuals().error_fg_color,
    }
}

fn parms_row(ui: &mut Ui, label: &str, value: &str) {
    ui.weak(label);
    ui.label(value);
    ui.end_row();
}

fn yes_no(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}

fn copy_locator_action_hint(
    report: &SourceExternalReferenceActionReport,
) -> Option<&SourceExternalReferenceActionHint> {
    std::iter::once(&report.recommended)
        .chain(report.secondary.iter())
        .find(|hint| hint.kind == SourceExternalReferenceActionKind::CopyLocator)
}

fn format_list(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_owned()
    } else {
        values.join(", ")
    }
}

fn format_bindings(bindings: &[HoudiniNodeBinding]) -> String {
    if bindings.is_empty() {
        "none".to_owned()
    } else {
        bindings
            .iter()
            .map(|binding| format!("{} <- {}", binding.port_name, binding.source_summary))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn metadata_value_or_fallback(value: &str, default_value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == default_value {
        fallback.to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn asset_gallery_entry_matches(entry: &model::ProceduralAssetGalleryEntry, filter: &str) -> bool {
    let matches = |value: &str| value.to_lowercase().contains(filter);
    matches(&entry.display_name)
        || matches(&entry.asset_id)
        || matches(&entry.description)
        || entry.labels.iter().any(|label| matches(label))
        || entry.wrapped_graph_id.as_deref().is_some_and(matches)
        || entry.usages.iter().any(|usage| {
            matches(&usage.node_name)
                || matches(&usage.node_id)
                || matches(&usage.graph_id)
                || matches(&usage.graph_path)
                || matches(&usage.node_path)
        })
}

struct AssetUsageGraphGroup<'a> {
    graph_path: &'a str,
    usages: Vec<&'a model::ProceduralAssetUsageInfo>,
}

fn asset_usage_graph_groups(
    usages: &[model::ProceduralAssetUsageInfo],
) -> Vec<AssetUsageGraphGroup<'_>> {
    let mut groups = Vec::<AssetUsageGraphGroup<'_>>::new();
    for usage in usages {
        if let Some(group) = groups
            .last_mut()
            .filter(|group| group.graph_path == usage.graph_path)
        {
            group.usages.push(usage);
        } else {
            groups.push(AssetUsageGraphGroup {
                graph_path: &usage.graph_path,
                usages: vec![usage],
            });
        }
    }
    groups
}

fn operator_matches(filter: &str, label: &str, aliases: &[&str]) -> bool {
    filter.is_empty()
        || label.to_lowercase().contains(filter)
        || aliases.iter().any(|alias| alias.contains(filter))
}

fn collapsible_node_indices_for_selection(
    graph: &GraphDocument,
    selected_nodes: &[usize],
) -> Vec<usize> {
    selected_nodes
        .iter()
        .copied()
        .filter(|node_index| {
            graph.nodes.get(*node_index).is_some_and(|node| {
                node.parent_graph_id == graph.current_graph_id()
                    && !matches!(node.kind, NodeKind::Output | NodeKind::GraphContainer)
            })
        })
        .collect()
}

fn operator_palette_entries(
    graph: &GraphDocument,
    selected_node: usize,
    selected_nodes: &[usize],
    include_organization: bool,
    include_layers: bool,
) -> Vec<OperatorPaletteEntry> {
    let mut entries = vec![
        operator_palette_entry(OperatorPaletteAction::AddOutNull),
        operator_palette_entry(OperatorPaletteAction::AddReference),
    ];

    if graph
        .reference_coordinate_repair_summary(selected_node)
        .is_some()
    {
        entries.push(operator_palette_entry(
            OperatorPaletteAction::AddRepairProjection,
        ));
    }
    entries.push(operator_palette_entry(
        OperatorPaletteAction::DuplicateSelected,
    ));
    if operator_palette_action_available(
        graph,
        selected_node,
        selected_nodes,
        OperatorPaletteAction::EnterSelectedSubnet,
    ) {
        entries.push(operator_palette_entry(
            OperatorPaletteAction::EnterSelectedSubnet,
        ));
    }
    if operator_palette_action_available(
        graph,
        selected_node,
        selected_nodes,
        OperatorPaletteAction::GoUpOneGraph,
    ) {
        entries.push(operator_palette_entry(OperatorPaletteAction::GoUpOneGraph));
    }
    if operator_palette_action_available(
        graph,
        selected_node,
        selected_nodes,
        OperatorPaletteAction::CreateAssetFromSelectedSubnet,
    ) {
        entries.push(operator_palette_entry(
            OperatorPaletteAction::CreateAssetFromSelectedSubnet,
        ));
    }
    if operator_palette_action_available(
        graph,
        selected_node,
        selected_nodes,
        OperatorPaletteAction::CollapseSelectionToSubnet,
    ) {
        entries.push(operator_palette_entry(
            OperatorPaletteAction::CollapseSelectionToSubnet,
        ));
    }

    if include_organization {
        entries.extend([
            operator_palette_entry(OperatorPaletteAction::AddNetworkBox),
            operator_palette_entry(OperatorPaletteAction::AddStickyNote),
        ]);
    }

    if include_layers {
        entries.extend([
            operator_palette_entry(OperatorPaletteAction::DuplicatePolygons),
            operator_palette_entry(OperatorPaletteAction::DuplicateCurves),
        ]);
    }

    entries
}

fn operator_palette_action_available(
    graph: &GraphDocument,
    selected_node: usize,
    selected_nodes: &[usize],
    action: OperatorPaletteAction,
) -> bool {
    match action {
        OperatorPaletteAction::AddRepairProjection => graph
            .reference_coordinate_repair_summary(selected_node)
            .is_some(),
        OperatorPaletteAction::DuplicateSelected => selected_node < graph.nodes.len(),
        OperatorPaletteAction::CollapseSelectionToSubnet => {
            collapsible_node_indices_for_selection(graph, selected_nodes).len() > 1
        }
        OperatorPaletteAction::EnterSelectedSubnet => graph
            .selected_node_info(selected_node)
            .and_then(|info| info.graph_container)
            .is_some_and(|container| container.navigable),
        OperatorPaletteAction::GoUpOneGraph => {
            graph.current_graph_parent_container_node_index().is_some()
        }
        OperatorPaletteAction::CreateAssetFromSelectedSubnet => graph
            .selected_node_info(selected_node)
            .and_then(|info| info.graph_container)
            .is_some_and(|container| container.navigable && !container.outputs.is_empty()),
        OperatorPaletteAction::AddOutNull
        | OperatorPaletteAction::AddReference
        | OperatorPaletteAction::AddNetworkBox
        | OperatorPaletteAction::AddStickyNote
        | OperatorPaletteAction::DuplicatePolygons
        | OperatorPaletteAction::DuplicateCurves => true,
    }
}

fn operator_palette_action_included(
    action: OperatorPaletteAction,
    include_organization: bool,
    include_layers: bool,
) -> bool {
    match operator_palette_entry(action).category {
        OperatorPaletteCategory::Create => true,
        OperatorPaletteCategory::Navigate => true,
        OperatorPaletteCategory::Organize => include_organization,
        OperatorPaletteCategory::LayerActions => include_layers,
    }
}

fn operator_palette_entry(action: OperatorPaletteAction) -> OperatorPaletteEntry {
    match action {
        OperatorPaletteAction::AddOutNull => OperatorPaletteEntry {
            action,
            category: OperatorPaletteCategory::Create,
            label: "OUT Null",
            detail: "Typed pass-through anchor using the OUT_* naming convention.",
            aliases: &["null", "anchor", "out", "in"],
        },
        OperatorPaletteAction::AddReference => OperatorPaletteEntry {
            action,
            category: OperatorPaletteCategory::Create,
            label: "Reference",
            detail: "Visible live reference to the selected node output.",
            aliases: &["object merge", "import", "target"],
        },
        OperatorPaletteAction::AddRepairProjection => OperatorPaletteEntry {
            action,
            category: OperatorPaletteCategory::Create,
            label: "Repair Projection",
            detail: "Insert a visible substrate projection node for the selected reference.",
            aliases: &["projection", "repair", "coordinates"],
        },
        OperatorPaletteAction::DuplicateSelected => OperatorPaletteEntry {
            action,
            category: OperatorPaletteCategory::Create,
            label: "Duplicate Selected",
            detail: "Duplicate the selected node with a new stable node identity.",
            aliases: &["copy", "paste", "clone"],
        },
        OperatorPaletteAction::CollapseSelectionToSubnet => OperatorPaletteEntry {
            action,
            category: OperatorPaletteCategory::Create,
            label: "Subnet from Selection",
            detail: "Move the selected node set into a typed graph container.",
            aliases: &[
                "subnet",
                "collapse",
                "graph container",
                "digital asset",
                "asset",
            ],
        },
        OperatorPaletteAction::EnterSelectedSubnet => OperatorPaletteEntry {
            action,
            category: OperatorPaletteCategory::Navigate,
            label: "Enter Subnet",
            detail: "Open the selected subnet's internal graph.",
            aliases: &["enter", "dive", "inside", "subnet", "graph container"],
        },
        OperatorPaletteAction::GoUpOneGraph => OperatorPaletteEntry {
            action,
            category: OperatorPaletteCategory::Navigate,
            label: "Go Up",
            detail: "Return to the parent graph and select the containing subnet.",
            aliases: &["up", "parent", "out", "back", "network"],
        },
        OperatorPaletteAction::CreateAssetFromSelectedSubnet => OperatorPaletteEntry {
            action,
            category: OperatorPaletteCategory::Create,
            label: "Asset from Subnet",
            detail: "Create a project-local asset definition from the selected subnet boundary.",
            aliases: &["asset", "hda", "digital asset", "definition", "subnet"],
        },
        OperatorPaletteAction::AddNetworkBox => OperatorPaletteEntry {
            action,
            category: OperatorPaletteCategory::Organize,
            label: "Network Box",
            detail: "Group selected graph items as durable network organization.",
            aliases: &["box", "organize", "group"],
        },
        OperatorPaletteAction::AddStickyNote => OperatorPaletteEntry {
            action,
            category: OperatorPaletteCategory::Organize,
            label: "Sticky Note",
            detail: "Create a durable canvas note near the selected node.",
            aliases: &["note", "comment", "sticky"],
        },
        OperatorPaletteAction::DuplicatePolygons => OperatorPaletteEntry {
            action,
            category: OperatorPaletteCategory::LayerActions,
            label: "Duplicate Polygons",
            detail: "Create another graph-backed polygon layer view.",
            aliases: &["polygon", "layer"],
        },
        OperatorPaletteAction::DuplicateCurves => OperatorPaletteEntry {
            action,
            category: OperatorPaletteCategory::LayerActions,
            label: "Duplicate Curves",
            detail: "Create another graph-backed native cubic layer view.",
            aliases: &["curve", "bezier", "layer"],
        },
    }
}

fn operator_palette_button_ui(ui: &mut Ui, entry: OperatorPaletteEntry, highlighted: bool) -> bool {
    let mut clicked = false;
    ui.horizontal_wrapped(|ui| {
        let mut button = egui::Button::new(entry.label);
        if highlighted {
            button = button
                .fill(ui.visuals().selection.bg_fill)
                .stroke(ui.visuals().selection.stroke);
        }
        clicked = ui.add(button).on_hover_text(entry.detail).clicked();
        ui.weak(entry.detail);
    });
    clicked
}

fn badge_visibility_combo_ui(ui: &mut Ui, label: &str, visibility: &mut NetworkBadgeVisibility) {
    ui.horizontal(|ui| {
        ui.weak(label);
        egui::ComboBox::from_id_salt(format!("houdini_graph_badge_visibility_{label}"))
            .selected_text(visibility.label())
            .show_ui(ui, |ui| {
                for option in NetworkBadgeVisibility::ALL {
                    ui.selectable_value(visibility, option, option.label());
                }
            });
    });
}

fn toggle_network_display_options(ui: &Ui) {
    let id = ui.make_persistent_id(NETWORK_DISPLAY_OPTIONS_ID);
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
    state.toggle(ui);
    state.store(ui.ctx());
}

fn modifiers_are_shift_only(modifiers: egui::Modifiers) -> bool {
    modifiers.shift && !modifiers.alt && !modifiers.ctrl && !modifiers.mac_cmd && !modifiers.command
}

fn format_node_name(name: impl AsRef<str>, max_width: f32) -> String {
    let name = name.as_ref();
    let max_chars = (max_width / 7.0).round().clamp(6.0, 32.0) as usize;
    if name.chars().count() <= max_chars {
        name.to_owned()
    } else {
        let prefix = name
            .chars()
            .take(max_chars.saturating_sub(1))
            .collect::<String>();
        format!("{prefix}…")
    }
}

fn format_node_comment(comment: &str) -> String {
    let comment = comment.trim().replace('\n', " ");
    if comment.chars().count() <= 34 {
        comment
    } else {
        let prefix = comment.chars().take(33).collect::<String>();
        format!("{prefix}…")
    }
}

fn network_comment_visible(mode: NetworkCommentDisplayMode, node: &self::model::GraphNode) -> bool {
    mode.shows_comment(&node.comment, node.show_comment_in_network)
}

fn faded_color(color: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        ((color.a() as f32) * alpha.clamp(0.0, 1.0)).round() as u8,
    )
}

fn network_background_color(visuals: &egui::Visuals, brightness: f32) -> Color32 {
    lerp_color(
        visuals.extreme_bg_color,
        visuals.widgets.noninteractive.bg_fill,
        brightness,
    )
}

fn lerp_color(from: Color32, to: Color32, amount: f32) -> Color32 {
    let amount = amount.clamp(0.0, 1.0);
    let lerp_channel =
        |from: u8, to: u8| ((from as f32) + ((to as f32) - (from as f32)) * amount).round() as u8;
    Color32::from_rgba_unmultiplied(
        lerp_channel(from.r(), to.r()),
        lerp_channel(from.g(), to.g()),
        lerp_channel(from.b(), to.b()),
        lerp_channel(from.a(), to.a()),
    )
}

fn draw_network_grid(
    painter: &egui::Painter,
    rect: Rect,
    color: Color32,
    spacing_scale: f32,
    zoom: f32,
    pan: Vec2,
) {
    let spacing = (24.0 * spacing_scale * zoom).clamp(8.0, 180.0);
    let grid_color = faded_color(color, 0.32);
    let stroke = Stroke::new(1.0, grid_color);
    let origin = rect.center() + pan;

    let mut x = origin.x + ((rect.left() - origin.x) / spacing).floor() * spacing;
    while x < rect.right() {
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            stroke,
        );
        x += spacing;
    }

    let mut y = origin.y + ((rect.top() - origin.y) / spacing).floor() * spacing;
    while y < rect.bottom() {
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            stroke,
        );
        y += spacing;
    }
}

fn draw_node_ring(
    painter: &egui::Painter,
    node_rect: Rect,
    selected: bool,
    node: &self::model::GraphNode,
    hovered_action: Option<NodeRingAction>,
    visuals: &egui::Visuals,
) {
    let center = node_rect.center();
    let ring_radius = node_rect.width() * if selected { 0.76 } else { 0.68 };
    painter.circle_stroke(
        center,
        ring_radius,
        Stroke::new(2.0, faded_color(visuals.weak_text_color(), 0.55)),
    );

    let status_color = evaluation_color_from_visuals(visuals, node.evaluation.state);
    for action in NODE_RING_ACTIONS {
        let fill = match action {
            NodeRingAction::Info => visuals.widgets.noninteractive.bg_fill,
            NodeRingAction::Display => {
                if node.participates_in_output {
                    visuals.selection.stroke.color
                } else {
                    visuals.widgets.inactive.bg_fill
                }
            }
            NodeRingAction::Manual => {
                if node.evaluation.manual {
                    visuals.warn_fg_color
                } else {
                    visuals.widgets.inactive.bg_fill
                }
            }
            NodeRingAction::Run => status_color,
        };
        let highlighted = hovered_action == Some(action);
        let angle = action.angle();
        let pos = center + egui::vec2(angle.cos(), angle.sin()) * ring_radius;
        painter.circle_filled(
            pos,
            if highlighted { 13.0 } else { 11.0 },
            faded_color(fill, if highlighted { 1.0 } else { 0.85 }),
        );
        painter.circle_stroke(
            pos,
            if highlighted { 13.0 } else { 11.0 },
            Stroke::new(
                if highlighted { 2.0 } else { 1.0 },
                if highlighted {
                    visuals.selection.stroke.color
                } else {
                    visuals.widgets.inactive.fg_stroke.color
                },
            ),
        );
        painter.text(
            pos,
            Align2::CENTER_CENTER,
            action.glyph(),
            FontId::monospace(10.0),
            visuals.text_color(),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeRingAction {
    Info,
    Display,
    Manual,
    Run,
}

const NODE_RING_ACTIONS: [NodeRingAction; 4] = [
    NodeRingAction::Info,
    NodeRingAction::Display,
    NodeRingAction::Manual,
    NodeRingAction::Run,
];

impl NodeRingAction {
    fn glyph(self) -> &'static str {
        match self {
            Self::Info => "i",
            Self::Display => "D",
            Self::Manual => "M",
            Self::Run => "!",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Info => "Node Info",
            Self::Display => "Display Output",
            Self::Manual => "Manual Cook",
            Self::Run => "Run Node",
        }
    }

    fn detail(self) -> &'static str {
        match self {
            Self::Info => "Open concise info.",
            Self::Display => "Toggle output participation.",
            Self::Manual => "Toggle manual evaluation.",
            Self::Run => "Cook this node now.",
        }
    }

    fn angle(self) -> f32 {
        match self {
            Self::Info => -std::f32::consts::PI,
            Self::Display => -0.38 * std::f32::consts::PI,
            Self::Manual => 0.38 * std::f32::consts::PI,
            Self::Run => 0.78 * std::f32::consts::PI,
        }
    }
}

fn node_ring_visible(visibility: NetworkNodeRingVisibility, selected: bool, hovered: bool) -> bool {
    match visibility {
        NetworkNodeRingVisibility::Hidden => false,
        NetworkNodeRingVisibility::Selected => selected || hovered,
        NetworkNodeRingVisibility::Always => true,
    }
}

fn node_ring_action_at(node_rect: Rect, pointer_pos: Pos2, zoom: f32) -> Option<NodeRingAction> {
    let center = node_rect.center();
    let ring_radius = node_rect.width() * 0.76;
    let hit_radius = (13.0 * zoom).clamp(9.0, 18.0);

    NODE_RING_ACTIONS.into_iter().find(|action| {
        let angle = action.angle();
        let pos = center + egui::vec2(angle.cos(), angle.sin()) * ring_radius;
        pos.distance(pointer_pos) <= hit_radius
    })
}

fn draw_node_ring_action_tooltip(
    painter: &egui::Painter,
    canvas_rect: Rect,
    pointer_pos: Pos2,
    node: &self::model::GraphNode,
    action: NodeRingAction,
    visuals: &egui::Visuals,
) {
    let tooltip_size = egui::vec2(292.0, 52.0);
    let min_x = canvas_rect.left() + 8.0;
    let min_y = canvas_rect.top() + 8.0;
    let max_x = (canvas_rect.right() - tooltip_size.x - 8.0).max(min_x);
    let max_y = (canvas_rect.bottom() - tooltip_size.y - 8.0).max(min_y);
    let min = Pos2::new(
        (pointer_pos.x + 14.0).clamp(min_x, max_x),
        (pointer_pos.y + 14.0).clamp(min_y, max_y),
    );
    let rect = Rect::from_min_size(min, tooltip_size);

    painter.rect_filled(rect, 5.0, visuals.extreme_bg_color);
    painter.rect_stroke(
        rect,
        5.0,
        Stroke::new(1.0, visuals.selection.stroke.color),
        StrokeKind::Inside,
    );
    painter.text(
        rect.left_top() + egui::vec2(10.0, 8.0),
        Align2::LEFT_TOP,
        format!("{}  {}", action.glyph(), action.label()),
        FontId::proportional(13.0),
        visuals.text_color(),
    );
    painter.text(
        rect.left_top() + egui::vec2(10.0, 27.0),
        Align2::LEFT_TOP,
        format!(
            "{}: {}",
            format_node_name(&node.name, 116.0),
            action.detail()
        ),
        FontId::proportional(11.0),
        visuals.weak_text_color(),
    );
}

fn draw_node_badges(
    painter: &egui::Painter,
    node_rect: Rect,
    graph: &GraphDocument,
    node_index: usize,
    node: &self::model::GraphNode,
    network_view: NetworkViewDisplayOptions,
    visuals: &egui::Visuals,
) {
    let mut badges = Vec::new();
    let node_info = graph.selected_node_info(node_index);
    if node.evaluation.state == EvaluationState::Failed {
        badges.push(("!", visuals.error_fg_color, network_view.error_badge));
    } else if node.evaluation.message.is_some() {
        badges.push(("!", visuals.warn_fg_color, network_view.warning_badge));
    }
    if !node.comment.trim().is_empty() {
        badges.push((
            "C",
            Color32::from_rgb(42, 178, 168),
            network_view.comment_badge,
        ));
    }
    if let Some(asset) = &node.procedural_asset {
        badges.push((
            if asset.contents_unlocked { "U" } else { "L" },
            if asset.contents_unlocked {
                Color32::from_rgb(242, 176, 61)
            } else {
                Color32::from_rgb(122, 154, 212)
            },
            network_view.lock_badge,
        ));
    }
    if node_info.as_ref().is_some_and(|info| info.record_count > 0) {
        badges.push((
            "H",
            Color32::from_rgb(96, 180, 116),
            network_view.has_data_badge,
        ));
    }
    if node.evaluation.state == EvaluationState::Cached
        || node
            .python_operator
            .as_ref()
            .is_some_and(|operator| operator.cache_key.is_some())
        || node
            .native_operator
            .as_ref()
            .is_some_and(|operator| operator.cache_key.is_some())
    {
        badges.push((
            "K",
            Color32::from_rgb(116, 151, 230),
            network_view.cached_code_badge,
        ));
    }
    if node.coordinate_contract.is_some()
        || node_info
            .as_ref()
            .and_then(|info| info.reference_input.as_ref())
            .is_some_and(|reference_input| reference_input.coordinate_contract.is_some())
    {
        badges.push((
            "X",
            Color32::from_rgb(205, 154, 90),
            network_view.constraint_badge,
        ));
    }
    if node_info.as_ref().is_some_and(node_info_is_compilable) {
        badges.push((
            "P",
            Color32::from_rgb(178, 132, 222),
            network_view.compilable_badge,
        ));
    }

    let mut offset = 0.0;
    for (label, color, visibility) in badges {
        let Some(radius) = visibility.radius() else {
            continue;
        };
        let center = node_rect.left_top() + egui::vec2(8.0 + offset, 8.0);
        painter.circle_filled(center, radius, faded_color(color, 0.92));
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(1.0, visuals.widgets.inactive.fg_stroke.color),
        );
        if radius >= 5.0 {
            painter.text(
                center,
                Align2::CENTER_CENTER,
                label,
                FontId::monospace(7.0),
                visuals.text_color(),
            );
        }
        offset += radius * 2.0 + 3.0;
    }
}

fn node_info_is_compilable(info: &self::model::NodeInfo) -> bool {
    info.python_operator
        .as_ref()
        .is_some_and(|operator| operator.dependency_status == PythonOperatorDependencyStatus::Ready)
        || info
            .native_operator
            .as_ref()
            .is_some_and(|operator| operator.load_status == NativeOperatorLoadStatus::Ready)
}

fn draw_node_flag_strip(
    painter: &egui::Painter,
    node_rect: Rect,
    node: &self::model::GraphNode,
    hovered_action: Option<NodeRingAction>,
    visuals: &egui::Visuals,
) {
    let flags = [
        (
            NodeRingAction::Display,
            "D",
            node.participates_in_output,
            visuals.selection.stroke.color,
        ),
        (
            NodeRingAction::Manual,
            "M",
            node.evaluation.manual,
            visuals.warn_fg_color,
        ),
    ];

    for (action, label, active, active_color) in flags {
        let Some(rect) = node_flag_rect(node_rect, action) else {
            continue;
        };
        let hovered = hovered_action == Some(action);
        let fill = if active {
            faded_color(active_color, 0.86)
        } else if hovered {
            visuals.widgets.hovered.bg_fill
        } else {
            visuals.widgets.inactive.bg_fill
        };
        painter.rect_filled(rect, 2.0, fill);
        painter.rect_stroke(
            rect,
            2.0,
            Stroke::new(
                if hovered { 1.5 } else { 1.0 },
                if hovered || active {
                    active_color
                } else {
                    visuals.widgets.inactive.fg_stroke.color
                },
            ),
            StrokeKind::Inside,
        );
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::monospace(8.0),
            visuals.text_color(),
        );
    }
}

fn node_flag_action_at(node_rect: Rect, pointer_pos: Pos2) -> Option<NodeRingAction> {
    [NodeRingAction::Display, NodeRingAction::Manual]
        .into_iter()
        .find(|action| {
            node_flag_rect(node_rect, *action).is_some_and(|rect| rect.contains(pointer_pos))
        })
}

fn node_flag_rect(node_rect: Rect, action: NodeRingAction) -> Option<Rect> {
    let index = match action {
        NodeRingAction::Display => 0,
        NodeRingAction::Manual => 1,
        NodeRingAction::Info | NodeRingAction::Run => return None,
    };
    let flag_size = egui::vec2(13.0, 13.0);
    let origin = node_rect.right_bottom() + egui::vec2(-32.0, -19.0);
    Some(Rect::from_min_size(
        origin + egui::vec2(index as f32 * 15.0, 0.0),
        flag_size,
    ))
}

fn node_primary_port_rect(node_rect: Rect, kind: NodePortKind, zoom: f32) -> Rect {
    let diameter = (12.0 * zoom).clamp(8.0, 16.0);
    let center = match kind {
        NodePortKind::Input => node_rect.left_center(),
        NodePortKind::Output => node_rect.right_center(),
    };
    Rect::from_center_size(center, egui::vec2(diameter, diameter))
}

fn node_primary_port_at(
    graph: &GraphDocument,
    node_rects: &[Rect],
    pointer_pos: Pos2,
    zoom: f32,
) -> Option<NodePortHit> {
    node_rects
        .iter()
        .enumerate()
        .find_map(|(node_index, rect)| {
            if graph.node_has_primary_geometry_output(node_index)
                && node_primary_port_rect(*rect, NodePortKind::Output, zoom).contains(pointer_pos)
            {
                return Some(NodePortHit {
                    node_index,
                    kind: NodePortKind::Output,
                });
            }
            if graph.node_has_primary_geometry_input(node_index)
                && node_primary_port_rect(*rect, NodePortKind::Input, zoom).contains(pointer_pos)
            {
                return Some(NodePortHit {
                    node_index,
                    kind: NodePortKind::Input,
                });
            }
            None
        })
}

fn connection_drag_preview(
    graph: &GraphDocument,
    node_rects: &[Rect],
    connection_drag: &ConnectionDragState,
    pointer_pos: Pos2,
    zoom: f32,
) -> ConnectionDragPreview {
    let Some(port_hit) = node_primary_port_at(graph, node_rects, pointer_pos, zoom) else {
        return ConnectionDragPreview::Floating;
    };
    if port_hit.kind != NodePortKind::Input {
        return ConnectionDragPreview::NonInput;
    }
    let Some(target_node) = graph.nodes.get(port_hit.node_index) else {
        return ConnectionDragPreview::Invalid("Connection target disappeared.".to_owned());
    };

    match graph.preview_add_data_flow_edge(
        &connection_drag.from_node_id,
        &connection_drag.from_output,
        &target_node.node_id,
        PRIMARY_GEOMETRY_OUTPUT,
    ) {
        Ok(_) => ConnectionDragPreview::Valid,
        Err(diagnostic) => ConnectionDragPreview::Invalid(diagnostic.message),
    }
}

fn connection_drag_preview_color(
    preview: &ConnectionDragPreview,
    visuals: &egui::Visuals,
) -> Color32 {
    match preview {
        ConnectionDragPreview::Valid => visuals.selection.stroke.color,
        ConnectionDragPreview::Invalid(_) => visuals.warn_fg_color,
        ConnectionDragPreview::Floating | ConnectionDragPreview::NonInput => {
            visuals.weak_text_color()
        }
    }
}

fn draw_node_primary_ports(
    painter: &egui::Painter,
    node_rect: Rect,
    graph: &GraphDocument,
    node_index: usize,
    zoom: f32,
    visuals: &egui::Visuals,
) {
    for (kind, available) in [
        (
            NodePortKind::Input,
            graph.node_has_primary_geometry_input(node_index),
        ),
        (
            NodePortKind::Output,
            graph.node_has_primary_geometry_output(node_index),
        ),
    ] {
        if !available {
            continue;
        }
        let rect = node_primary_port_rect(node_rect, kind, zoom);
        painter.circle_filled(
            rect.center(),
            rect.width() * 0.5,
            visuals.widgets.inactive.bg_fill,
        );
        painter.circle_stroke(
            rect.center(),
            rect.width() * 0.5,
            Stroke::new(1.0, visuals.widgets.inactive.fg_stroke.color),
        );
    }
}

fn edge_hit_radius(zoom: f32) -> f32 {
    (8.0 * zoom).clamp(6.0, 14.0)
}

fn graph_edge_at(
    graph: &GraphDocument,
    node_rects: &[Rect],
    pointer_pos: Pos2,
    hit_radius: f32,
) -> Option<String> {
    graph
        .graph_layout()
        .edges
        .into_iter()
        .filter_map(|edge| {
            let from_rect = *node_rects.get(edge.from_node)?;
            let to_rect = *node_rects.get(edge.to_node)?;
            let start = Pos2::new(from_rect.right(), from_rect.center().y);
            let end = Pos2::new(to_rect.left(), to_rect.center().y);
            let distance = distance_to_segment(pointer_pos, start, end);
            (distance <= hit_radius).then_some((edge.edge_id, distance))
        })
        .min_by(|(_, left_distance), (_, right_distance)| {
            left_distance
                .partial_cmp(right_distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(edge_id, _)| edge_id)
}

fn distance_to_segment(point: Pos2, start: Pos2, end: Pos2) -> f32 {
    let segment = end - start;
    let length_squared = segment.length_sq();
    if length_squared <= f32::EPSILON {
        return point.distance(start);
    }
    let point_delta = point - start;
    let t = (point_delta.dot(segment) / length_squared).clamp(0.0, 1.0);
    point.distance(start + segment * t)
}

fn evaluation_color_from_visuals(visuals: &egui::Visuals, state: EvaluationState) -> Color32 {
    match state {
        EvaluationState::Clean => visuals.text_color(),
        EvaluationState::Cached => visuals.weak_text_color(),
        EvaluationState::Stale | EvaluationState::Manual => visuals.warn_fg_color,
        EvaluationState::Running => visuals.selection.stroke.color,
        EvaluationState::Failed => visuals.error_fg_color,
    }
}

fn transform_layout_pos(rect: Rect, position: Pos2, zoom: f32, pan: Vec2) -> Pos2 {
    rect.center() + (position - rect.center()) * zoom + pan
}

fn inverse_transform_layout_pos(rect: Rect, position: Pos2, zoom: f32, pan: Vec2) -> Pos2 {
    rect.center() + (position - rect.center() - pan) / zoom
}

fn map_node_layout_point(
    rect: Rect,
    point: GraphPoint,
    node_size: Vec2,
    zoom: f32,
    pan: Vec2,
) -> Pos2 {
    let usable_width = (rect.width() - node_size.x).max(1.0);
    let usable_height = (rect.height() - node_size.y).max(1.0);
    let position = Pos2::new(
        rect.left() + node_size.x * 0.5 + usable_width * point.x,
        rect.top() + node_size.y * 0.5 + usable_height * point.y,
    );
    transform_layout_pos(rect, position, zoom, pan)
}

fn unmap_node_layout_point(
    rect: Rect,
    position: Pos2,
    node_size: Vec2,
    zoom: f32,
    pan: Vec2,
) -> GraphPoint {
    let position = inverse_transform_layout_pos(rect, position, zoom, pan);
    let usable_width = (rect.width() - node_size.x).max(1.0);
    let usable_height = (rect.height() - node_size.y).max(1.0);
    GraphPoint {
        x: (position.x - rect.left() - node_size.x * 0.5) / usable_width,
        y: (position.y - rect.top() - node_size.y * 0.5) / usable_height,
    }
}

fn layout_node_rects(
    graph: &GraphDocument,
    rect: Rect,
    node_size: Vec2,
    zoom: f32,
    pan: Vec2,
) -> Vec<Rect> {
    let layout = graph.graph_layout();
    let mut node_rects = vec![Rect::NOTHING; graph.nodes.len()];
    for layout_node in &layout.nodes {
        node_rects[layout_node.node_index] = Rect::from_center_size(
            map_node_layout_point(rect, layout_node.position, node_size, zoom, pan),
            node_size * zoom,
        );
    }
    node_rects
}

fn node_indices_in_selection_rect(
    graph: &GraphDocument,
    node_rects: &[Rect],
    selection_rect: Rect,
) -> Vec<usize> {
    graph
        .graph_layout()
        .nodes
        .iter()
        .filter_map(|layout_node| {
            node_rects
                .get(layout_node.node_index)
                .filter(|node_rect| node_rect.intersects(selection_rect))
                .map(|_| layout_node.node_index)
        })
        .collect()
}

fn layout_annotation_rects(
    graph: &GraphDocument,
    rect: Rect,
    zoom: f32,
    pan: Vec2,
) -> Vec<(usize, Rect)> {
    graph
        .current_graph_annotation_indices()
        .into_iter()
        .filter_map(|annotation_index| {
            graph.annotations.get(annotation_index).map(|annotation| {
                (
                    annotation_index,
                    display_annotation_rect(rect, annotation, zoom, pan),
                )
            })
        })
        .collect()
}

fn map_annotation_rect(
    rect: Rect,
    position: GraphPoint,
    size: GraphPoint,
    zoom: f32,
    pan: Vec2,
) -> Rect {
    let left = rect.left() + rect.width() * position.x;
    let top = rect.top() + rect.height() * position.y;
    let width = (rect.width() * size.x.clamp(0.04, 1.0)).max(44.0);
    let height = (rect.height() * size.y.clamp(0.04, 1.0)).max(28.0);
    Rect::from_min_size(
        transform_layout_pos(rect, Pos2::new(left, top), zoom, pan),
        egui::vec2(width, height) * zoom,
    )
}

fn display_annotation_rect(
    rect: Rect,
    annotation: &self::model::GraphAnnotation,
    zoom: f32,
    pan: Vec2,
) -> Rect {
    let full_annotation_rect =
        map_annotation_rect(rect, annotation.position, annotation.size, zoom, pan);
    if annotation.collapsed {
        Rect::from_min_size(
            full_annotation_rect.min,
            egui::vec2(full_annotation_rect.width(), 20.0),
        )
    } else {
        full_annotation_rect
    }
}

fn annotation_resize_handle_rect(annotation_rect: Rect) -> Rect {
    Rect::from_min_size(
        annotation_rect.right_bottom() - egui::vec2(12.0, 12.0),
        egui::vec2(12.0, 12.0),
    )
}

fn annotation_collapse_toggle_rect(annotation_rect: Rect) -> Rect {
    Rect::from_min_size(
        annotation_rect.left_top() + egui::vec2(5.0, 3.0),
        egui::vec2(12.0, 12.0),
    )
}

fn draw_graph_annotation(
    painter: &egui::Painter,
    layout_rect: Rect,
    annotation: &self::model::GraphAnnotation,
    zoom: f32,
    pan: Vec2,
    selected: bool,
    hovered: bool,
    visuals: &egui::Visuals,
) {
    let annotation_rect = display_annotation_rect(layout_rect, annotation, zoom, pan);
    match annotation.kind {
        GraphAnnotationKind::NetworkBox => {
            let body_fill = Color32::from_rgba_unmultiplied(150, 150, 150, 72);
            let header_fill = Color32::from_rgba_unmultiplied(185, 185, 185, 132);
            let stroke = annotation_stroke(
                visuals,
                selected,
                hovered,
                Color32::from_rgba_unmultiplied(210, 210, 210, 150),
            );
            painter.rect_filled(annotation_rect, 6.0, body_fill);
            painter.rect_stroke(annotation_rect, 6.0, stroke, StrokeKind::Inside);

            let header_rect = Rect::from_min_max(
                annotation_rect.min,
                Pos2::new(annotation_rect.right(), annotation_rect.top() + 18.0),
            );
            painter.rect_filled(header_rect, 6.0, header_fill);
            let toggle_rect = annotation_collapse_toggle_rect(header_rect);
            painter.rect_filled(toggle_rect, 2.0, body_fill);
            painter.rect_stroke(toggle_rect, 2.0, stroke, StrokeKind::Inside);
            painter.text(
                toggle_rect.center(),
                Align2::CENTER_CENTER,
                if annotation.collapsed { "+" } else { "-" },
                FontId::monospace(11.0),
                visuals.text_color(),
            );
            painter.text(
                header_rect.left_center() + egui::vec2(23.0, 0.0),
                Align2::LEFT_CENTER,
                &annotation.title,
                FontId::proportional(12.0),
                visuals.text_color(),
            );
            draw_annotation_resize_handle(painter, annotation_rect, stroke.color);
        }
        GraphAnnotationKind::StickyNote => {
            let body_fill = Color32::from_rgba_unmultiplied(214, 90, 176, 150);
            let header_fill = Color32::from_rgba_unmultiplied(244, 106, 205, 185);
            let stroke = annotation_stroke(
                visuals,
                selected,
                hovered,
                Color32::from_rgba_unmultiplied(230, 210, 72, 210),
            );
            painter.rect_filled(annotation_rect, 5.0, body_fill);
            painter.rect_stroke(annotation_rect, 5.0, stroke, StrokeKind::Inside);

            let header_rect = Rect::from_min_max(
                annotation_rect.min,
                Pos2::new(annotation_rect.right(), annotation_rect.top() + 18.0),
            );
            painter.rect_filled(header_rect, 5.0, header_fill);
            let toggle_rect = annotation_collapse_toggle_rect(header_rect);
            painter.rect_filled(toggle_rect, 2.0, body_fill);
            painter.rect_stroke(toggle_rect, 2.0, stroke, StrokeKind::Inside);
            painter.text(
                toggle_rect.center(),
                Align2::CENTER_CENTER,
                if annotation.collapsed { "+" } else { "-" },
                FontId::monospace(11.0),
                Color32::BLACK,
            );
            painter.text(
                header_rect.left_center() + egui::vec2(23.0, 0.0),
                Align2::LEFT_CENTER,
                &annotation.title,
                FontId::proportional(12.0),
                Color32::BLACK,
            );

            if !annotation.collapsed && !annotation.text.trim().is_empty() {
                painter.text(
                    annotation_rect.left_top() + egui::vec2(8.0, 26.0),
                    Align2::LEFT_TOP,
                    format_sticky_note_text(&annotation.text),
                    FontId::proportional(11.0),
                    Color32::BLACK,
                );
            }
            draw_annotation_resize_handle(painter, annotation_rect, stroke.color);
        }
    }
}

fn draw_annotation_resize_handle(painter: &egui::Painter, annotation_rect: Rect, color: Color32) {
    let handle_rect = annotation_resize_handle_rect(annotation_rect).shrink(3.0);
    painter.line_segment(
        [handle_rect.left_bottom(), handle_rect.right_top()],
        Stroke::new(1.0, color),
    );
    painter.line_segment(
        [
            handle_rect.left_bottom() + egui::vec2(4.0, 0.0),
            handle_rect.right_top() + egui::vec2(0.0, 4.0),
        ],
        Stroke::new(1.0, color),
    );
}

fn annotation_stroke(
    visuals: &egui::Visuals,
    selected: bool,
    hovered: bool,
    fallback_color: Color32,
) -> Stroke {
    if selected {
        Stroke::new(2.0, visuals.selection.stroke.color)
    } else if hovered {
        Stroke::new(1.5, visuals.widgets.hovered.fg_stroke.color)
    } else {
        Stroke::new(1.0, fallback_color)
    }
}

fn format_sticky_note_text(text: &str) -> String {
    let text = text.trim().replace('\n', " ");
    if text.chars().count() <= 56 {
        text
    } else {
        let prefix = text.chars().take(55).collect::<String>();
        format!("{prefix}…")
    }
}

fn graph_container_collapse_error_message(error: &GraphContainerCollapseError) -> String {
    match error {
        GraphContainerCollapseError::EmptySelection => "selection is empty".to_owned(),
        GraphContainerCollapseError::MissingNodeIndex(index) => {
            format!("node index {index} is missing")
        }
        GraphContainerCollapseError::DisconnectedSelection => {
            "selected nodes are not connected".to_owned()
        }
        GraphContainerCollapseError::UntypedExternalEdge(edge_id) => {
            format!("external edge {edge_id} has no typed data kind")
        }
    }
}

fn graph_navigation_error_message(error: &GraphNavigationError) -> String {
    match error {
        GraphNavigationError::MissingGraph { graph_id } => {
            format!("graph {graph_id} is missing")
        }
        GraphNavigationError::MissingNodeIndex(index) => {
            format!("node index {index} is missing")
        }
        GraphNavigationError::NodeIsNotGraphContainer { node_name, .. } => {
            format!("{node_name} is not a graph container")
        }
        GraphNavigationError::MissingContainerMetadata { node_id } => {
            format!("container metadata for {node_id} is missing")
        }
        GraphNavigationError::MissingInternalGraph { graph_id } => {
            format!("internal graph {graph_id} is missing")
        }
        GraphNavigationError::ContainerNotNavigable { node_id, .. } => {
            format!("container {node_id} is not navigable")
        }
    }
}

fn graph_container_asset_draft_error_message(error: &GraphContainerAssetDraftError) -> String {
    match error {
        GraphContainerAssetDraftError::MissingNodeIndex(index) => {
            format!("node index {index} is missing")
        }
        GraphContainerAssetDraftError::NotGraphContainer => {
            "selected node is not a subnet".to_owned()
        }
        GraphContainerAssetDraftError::MissingContainerMetadata => {
            "selected subnet has no graph container metadata".to_owned()
        }
        GraphContainerAssetDraftError::MissingInternalGraph => {
            "selected subnet points to a missing internal graph".to_owned()
        }
        GraphContainerAssetDraftError::MissingOutputBoundary => {
            "selected subnet has no typed output boundary".to_owned()
        }
    }
}

fn source_gallery_filtered_items<'a>(
    index: &'a SourceGalleryIndex,
    filter: &str,
) -> Vec<&'a SourceGalleryItem> {
    index
        .items
        .iter()
        .filter(|item| source_gallery_item_matches_filter(item, filter))
        .collect()
}

fn source_gallery_item_matches_filter(item: &SourceGalleryItem, filter: &str) -> bool {
    let filter = filter.trim().to_ascii_lowercase();
    if filter.is_empty() {
        return true;
    }

    let mut haystack = vec![
        item.display_name.to_ascii_lowercase(),
        item.locator.readable().to_ascii_lowercase(),
        item.kind.as_str().to_owned(),
        item.external_reference_status.as_str().to_owned(),
        item.thumbnail_intent.status().as_str().to_owned(),
    ];
    if let Some(kind) = item.format_kind {
        haystack.push(kind.as_str().to_ascii_lowercase());
    }
    if let Some(status) = item.format_support_status {
        haystack.push(status.as_str().to_ascii_lowercase());
    }

    haystack.iter().any(|candidate| candidate.contains(&filter))
}

fn source_gallery_selected_item<'a>(
    index: &'a SourceGalleryIndex,
    selected_id: Option<&str>,
) -> Option<&'a SourceGalleryItem> {
    selected_id.and_then(|selected_id| {
        index
            .items
            .iter()
            .find(|item| item.stable_id == selected_id)
    })
}

fn source_gallery_thumbnail_label(item: &SourceGalleryItem) -> String {
    let kind = match item.kind {
        SourceGalleryItemKind::Image => "IMG",
        SourceGalleryItemKind::Table => "TABLE",
        SourceGalleryItemKind::PolygonTable => "POLY",
        SourceGalleryItemKind::Recording => "RRD",
        SourceGalleryItemKind::PointCloud => "POINT",
        SourceGalleryItemKind::Manifest => "LIST",
        SourceGalleryItemKind::Generated => "GEN",
        SourceGalleryItemKind::LiveRecording => "LIVE",
        SourceGalleryItemKind::Unknown => "FILE",
    };
    format!("{kind}  {}", item.thumbnail_intent.status().as_str())
}

fn source_gallery_tile_detail(item: &SourceGalleryItem) -> String {
    let format = item
        .format_kind
        .map(|kind| kind.as_str())
        .unwrap_or_else(|| item.kind.as_str());
    format!("{} / {}", format, item.external_reference_status.as_str())
}

fn draw_arrowhead(painter: &egui::Painter, tip: Pos2, color: Color32) {
    let size = 5.0;
    painter.add(egui::Shape::convex_polygon(
        vec![
            tip,
            Pos2::new(tip.x - size, tip.y - size * 0.7),
            Pos2::new(tip.x - size, tip.y + size * 0.7),
        ],
        color,
        Stroke::NONE,
    ));
}

#[cfg(test)]
mod tests {
    use super::{
        ConnectionDragPreview, ConnectionDragState, GraphSearchTarget, GraphWorkbenchPane,
        HoudiniGraphPanel, NodePortKind, OperatorPaletteAction, asset_usage_graph_groups,
        connection_drag_preview, copy_locator_action_hint, distance_to_segment, graph_edge_at,
        layout_node_rects,
        model::{
            GraphContainerStatus, NodeKind, ProjectGraphMetadata, ProjectGraphRole,
            SourceExternalReferenceActionKind, SourceGalleryIndex, SourceLocator, SourceMetadata,
            SourceProvenance,
        },
        node_indices_in_selection_rect, node_primary_port_at, node_primary_port_rect,
        operator_palette_action_available, operator_palette_entries, source_gallery_filtered_items,
        source_gallery_selected_item, source_gallery_thumbnail_label, source_gallery_tile_detail,
    };
    use crate::ui::houdini_graph_panel::model::{GraphDocument, PRIMARY_GEOMETRY_OUTPUT};

    #[test]
    fn distance_to_segment_clamps_to_nearest_endpoint_or_segment() {
        let start = egui::pos2(10.0, 10.0);
        let end = egui::pos2(30.0, 10.0);

        assert_eq!(distance_to_segment(egui::pos2(20.0, 13.0), start, end), 3.0);
        assert_eq!(distance_to_segment(egui::pos2(5.0, 10.0), start, end), 5.0);
        assert_eq!(distance_to_segment(egui::pos2(35.0, 10.0), start, end), 5.0);
    }

    #[test]
    fn copy_locator_action_hint_is_available_for_external_sources() {
        let temp_dir = tempfile::tempdir().unwrap();
        let source_path = temp_dir.path().join("source.csv");
        std::fs::write(&source_path, b"x0,y0,x1,y1,x2,y2\n0,0,1,0,0,1\n").unwrap();
        let metadata = SourceMetadata {
            provenance: SourceProvenance::CsvImport,
            source_path: Some(source_path.display().to_string()),
            locator: SourceLocator::from_location(&source_path.display().to_string()),
            record_count: 0,
            polygon_count: 0,
            cubic_bezier_count: 0,
            bounds: None,
            attribute_names: Vec::new(),
            recognized_control_point_columns: Vec::new(),
        };

        let actions = metadata.external_reference_action_report();
        let copy_hint =
            copy_locator_action_hint(&actions).expect("local external source should copy locator");

        assert_eq!(
            copy_hint.kind,
            SourceExternalReferenceActionKind::CopyLocator
        );
        assert!(
            copy_hint
                .detail
                .contains(&source_path.display().to_string())
        );
    }

    #[test]
    fn copy_locator_action_hint_is_absent_for_generated_and_live_sources() {
        let graph = GraphDocument::sample();
        assert!(
            copy_locator_action_hint(&graph.source.metadata.external_reference_action_report())
                .is_none()
        );

        let bridge = crate::ui::houdini_graph_panel::model::RerunQueryBridge {
            mode: crate::ui::houdini_graph_panel::model::RerunQueryBridgeMode::ProductForkViewOwned,
            view_id: "view(1234)".to_owned(),
            space_origin: "/".to_owned(),
            timeline: "frame".to_owned(),
            latest_at: 42,
            matching_entity_count: 1,
            visualized_entity_count: 1,
            visible_data_result_count: 1,
        };
        let mut live_graph = GraphDocument::sample();
        live_graph.update_source_from_query_bridge(&bridge);
        assert!(
            copy_locator_action_hint(
                &live_graph
                    .source
                    .metadata
                    .external_reference_action_report()
            )
            .is_none()
        );
    }

    #[test]
    fn graph_edge_at_returns_stable_edge_id_for_drawn_connection() {
        let graph = GraphDocument::sample();
        let layout_rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(480.0, 220.0));
        let node_rects = layout_node_rects(
            &graph,
            layout_rect,
            egui::vec2(116.0, 48.0),
            1.0,
            egui::Vec2::ZERO,
        );
        let layout = graph.graph_layout();
        let first_edge = layout
            .edges
            .first()
            .expect("sample graph should have a first edge");
        let start = egui::pos2(
            node_rects[first_edge.from_node].right(),
            node_rects[first_edge.from_node].center().y,
        );
        let end = egui::pos2(
            node_rects[first_edge.to_node].left(),
            node_rects[first_edge.to_node].center().y,
        );
        let midpoint = start + (end - start) * 0.5;

        assert_eq!(
            graph_edge_at(&graph, &node_rects, midpoint + egui::vec2(0.0, 3.0), 8.0),
            Some(first_edge.edge_id.clone())
        );
        assert_eq!(
            graph_edge_at(&graph, &node_rects, egui::pos2(8.0, 210.0), 4.0),
            None
        );
    }

    #[test]
    fn node_primary_port_at_distinguishes_input_and_output_ports() {
        let graph = GraphDocument::sample();
        let layout_rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(480.0, 220.0));
        let node_rects = layout_node_rects(
            &graph,
            layout_rect,
            egui::vec2(116.0, 48.0),
            1.0,
            egui::Vec2::ZERO,
        );

        let source_output = node_primary_port_rect(node_rects[0], NodePortKind::Output, 1.0);
        assert_eq!(
            node_primary_port_at(&graph, &node_rects, source_output.center(), 1.0),
            Some(super::NodePortHit {
                node_index: 0,
                kind: NodePortKind::Output,
            })
        );
        let source_input = node_primary_port_rect(node_rects[0], NodePortKind::Input, 1.0);
        assert_eq!(
            node_primary_port_at(&graph, &node_rects, source_input.center(), 1.0),
            None
        );

        let output_input = node_primary_port_rect(node_rects[3], NodePortKind::Input, 1.0);
        assert_eq!(
            node_primary_port_at(&graph, &node_rects, output_input.center(), 1.0),
            Some(super::NodePortHit {
                node_index: 3,
                kind: NodePortKind::Input,
            })
        );

        let output_output = node_primary_port_rect(node_rects[3], NodePortKind::Output, 1.0);
        assert_eq!(
            node_primary_port_at(&graph, &node_rects, output_output.center(), 1.0),
            None
        );
    }

    #[test]
    fn marquee_selection_rect_returns_intersecting_graph_nodes() {
        let graph = GraphDocument::sample();
        let layout_rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(480.0, 220.0));
        let node_rects = layout_node_rects(
            &graph,
            layout_rect,
            egui::vec2(116.0, 48.0),
            1.0,
            egui::Vec2::ZERO,
        );
        let selection_rect =
            egui::Rect::from_min_max(node_rects[1].min, node_rects[2].max).shrink(2.0);

        assert_eq!(
            node_indices_in_selection_rect(&graph, &node_rects, selection_rect),
            vec![1, 2]
        );
        assert!(
            node_indices_in_selection_rect(
                &graph,
                &node_rects,
                egui::Rect::from_min_size(egui::pos2(0.0, 200.0), egui::vec2(20.0, 20.0)),
            )
            .is_empty()
        );
    }

    #[test]
    fn connection_drag_preview_reports_valid_and_invalid_targets() {
        let graph = GraphDocument::sample();
        let node_size = egui::vec2(116.0, 48.0);
        let node_rects = vec![
            egui::Rect::from_min_size(egui::pos2(20.0, 20.0), node_size),
            egui::Rect::from_min_size(egui::pos2(180.0, 20.0), node_size),
            egui::Rect::from_min_size(egui::pos2(340.0, 20.0), node_size),
            egui::Rect::from_min_size(egui::pos2(500.0, 20.0), node_size),
        ];
        let source_drag = ConnectionDragState {
            from_node_index: 0,
            from_node_id: graph.nodes[0].node_id.clone(),
            from_output: PRIMARY_GEOMETRY_OUTPUT.to_owned(),
        };

        let output_input = node_primary_port_rect(node_rects[3], NodePortKind::Input, 1.0);
        assert_eq!(
            connection_drag_preview(
                &graph,
                &node_rects,
                &source_drag,
                output_input.center(),
                1.0
            ),
            ConnectionDragPreview::Valid
        );

        let filter_input = node_primary_port_rect(node_rects[1], NodePortKind::Input, 1.0);
        assert_eq!(
            connection_drag_preview(
                &graph,
                &node_rects,
                &source_drag,
                filter_input.center(),
                1.0
            ),
            ConnectionDragPreview::Invalid("Connection already exists.".to_owned())
        );

        let source_output = node_primary_port_rect(node_rects[0], NodePortKind::Output, 1.0);
        assert_eq!(
            connection_drag_preview(
                &graph,
                &node_rects,
                &source_drag,
                source_output.center(),
                1.0,
            ),
            ConnectionDragPreview::NonInput
        );
        assert_eq!(
            connection_drag_preview(
                &graph,
                &node_rects,
                &source_drag,
                egui::pos2(8.0, 210.0),
                1.0,
            ),
            ConnectionDragPreview::Floating
        );
    }

    #[test]
    fn graph_search_result_switches_to_node_parent_graph() {
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
        let analysis_node_index = graph.add_null_operator_node("OUT_A");
        graph
            .select_graph_by_id("main")
            .expect("main graph should be selectable");

        let mut panel = HoudiniGraphPanel {
            graph_search_filter: "/obj/analysis/out_a".to_owned(),
            ..HoudiniGraphPanel::default()
        };
        let result = panel
            .graph_search_results(&graph)
            .into_iter()
            .find(|result| {
                matches!(
                    &result.target,
                    GraphSearchTarget::Node { index, graph_id }
                        if *index == analysis_node_index && graph_id == "analysis"
                )
            })
            .expect("search should find the analysis node by readable path");
        assert!(result.detail.contains("/obj/analysis/OUT_A"));

        panel.apply_graph_search_result(&mut graph, result.target);

        assert_eq!(graph.current_graph_id(), "analysis");
        assert_eq!(panel.selected_node, analysis_node_index);
        assert!(panel.selected_annotation.is_none());
        assert!(panel.node_info_open);
    }

    #[test]
    fn source_gallery_filter_selection_and_tile_labels_cover_visible_view_contract() {
        let temp_dir = tempfile::tempdir().unwrap();
        let image_path = temp_dir.path().join("frame.png");
        let table_path = temp_dir.path().join("polygons.geoparquet");
        std::fs::write(&image_path, b"image bytes").unwrap();
        std::fs::write(&table_path, b"table bytes").unwrap();
        let index = SourceGalleryIndex::from_locations(
            SourceLocator::from_location("inline-gallery"),
            vec![
                SourceLocator::from_location(&image_path.display().to_string()),
                SourceLocator::from_location(&table_path.display().to_string()),
                SourceLocator::from_location("https://example.test/remote.png"),
            ],
            16,
        );

        assert_eq!(source_gallery_filtered_items(&index, "").len(), 3);

        let image_matches = source_gallery_filtered_items(&index, "image");
        assert_eq!(image_matches.len(), 2);
        assert!(
            image_matches
                .iter()
                .any(|item| source_gallery_thumbnail_label(item).contains("IMG"))
        );

        let polygon_matches = source_gallery_filtered_items(&index, "polygon table");
        assert_eq!(polygon_matches.len(), 1);
        assert!(source_gallery_tile_detail(polygon_matches[0]).contains("GeoParquet"));

        let remote_matches = source_gallery_filtered_items(&index, "remote unverified");
        assert_eq!(remote_matches.len(), 1);
        assert_eq!(remote_matches[0].display_name, "remote.png");

        assert!(source_gallery_filtered_items(&index, "no-such-source").is_empty());

        let selected_id = polygon_matches[0].stable_id.as_str();
        let selected = source_gallery_selected_item(&index, Some(selected_id))
            .expect("selected source gallery item should resolve by stable id");
        assert_eq!(selected.display_name, "polygons.geoparquet");
        assert!(source_gallery_selected_item(&index, Some("missing")).is_none());
    }

    #[test]
    fn source_gallery_disabled_open_action_does_not_mutate_graph_state() {
        let temp_dir = tempfile::tempdir().unwrap();
        let missing_path = temp_dir.path().join("missing.png");
        let index = SourceGalleryIndex::from_locator(
            SourceLocator::from_location(&missing_path.display().to_string()),
            16,
        );
        let item = index
            .items
            .first()
            .expect("missing source should be indexed");
        let graph = GraphDocument::sample();
        let undo_count_before = graph.command_history.undo_stack.len();
        let redo_count_before = graph.command_history.redo_stack.len();
        let node_count_before = graph.nodes.len();
        let mut panel = HoudiniGraphPanel::default();

        assert!(!panel.execute_source_gallery_open_action(item, None));

        assert_eq!(graph.command_history.undo_stack.len(), undo_count_before);
        assert_eq!(graph.command_history.redo_stack.len(), redo_count_before);
        assert_eq!(graph.nodes.len(), node_count_before);
        assert!(
            panel
                .source_gallery_status
                .as_deref()
                .is_some_and(|status| status.contains("missing"))
        );
    }

    #[test]
    fn selected_node_collapse_ui_creates_navigable_graph_container() {
        let mut graph = GraphDocument::sample();
        let filter_index = graph
            .nodes
            .iter()
            .position(|node| node.node_id == "filter.main")
            .expect("sample graph should include filter");
        let mut panel = HoudiniGraphPanel {
            selected_node: filter_index,
            selected_nodes: vec![filter_index],
            ..HoudiniGraphPanel::default()
        };

        assert!(panel.selected_node_can_collapse_to_graph_container(&graph));
        assert!(panel.collapse_selected_node_to_graph_container(&mut graph));

        let selected = graph
            .nodes
            .get(panel.selected_node)
            .expect("new graph container should be selected");
        assert_eq!(selected.kind, NodeKind::GraphContainer);
        assert_eq!(selected.parent_graph_id, "main");
        assert_eq!(panel.selected_nodes, vec![panel.selected_node]);
        assert!(
            panel
                .graph_container_status
                .as_deref()
                .is_some_and(|status| status.contains("Collapsed Filter"))
        );
        assert!(panel.selected_annotation.is_none());
        assert!(panel.selected_edge.is_none());
        assert!(panel.node_info_open);

        let filter = graph
            .nodes
            .iter()
            .find(|node| node.node_id == "filter.main")
            .expect("filter node should stay graph-owned");
        assert_eq!(filter.parent_graph_id, "graph.filter_subnet");

        let info = graph
            .selected_node_info(panel.selected_node)
            .expect("container should inspect");
        let container = info
            .graph_container
            .expect("container metadata should be exposed");
        assert_eq!(container.status, GraphContainerStatus::Resolved);
        assert_eq!(container.internal_graph_id, "graph.filter_subnet");
        assert_eq!(
            container.internal_graph_path.as_deref(),
            Some("/obj/main/filter_subnet")
        );
    }

    #[test]
    fn selected_node_set_collapse_ui_creates_navigable_graph_container() {
        let mut graph = GraphDocument::sample();
        let mut panel = HoudiniGraphPanel::default();
        panel.set_selected_node_set(vec![1, 2]);

        assert!(panel.collapse_selected_nodes_to_graph_container(&mut graph));

        let container_index = panel.selected_node;
        let container = graph
            .selected_node_info(container_index)
            .expect("container node should inspect")
            .graph_container
            .expect("container info should exist");
        let internal_graph_id = container.internal_graph_id.clone();
        let collapse_manifest = container
            .collapse_manifest
            .as_ref()
            .expect("selection collapse should record manifest");

        assert_eq!(graph.nodes[container_index].kind, NodeKind::GraphContainer);
        assert_eq!(panel.selected_nodes, vec![container_index]);
        assert_eq!(container.status, GraphContainerStatus::Resolved);
        assert_eq!(
            collapse_manifest.captured_node_ids,
            vec!["filter.main".to_owned(), "style.main".to_owned()]
        );
        assert_eq!(
            graph.graph_layout_for_graph(&internal_graph_id).nodes.len(),
            2
        );
        assert!(
            panel
                .graph_container_status
                .as_deref()
                .is_some_and(|status| status.contains("Collapsed 2 selected nodes"))
        );
    }

    #[test]
    fn operator_palette_exposes_subnet_action_for_selected_node_set() {
        let graph = GraphDocument::sample();
        let selected_nodes = vec![1, 2];

        assert!(operator_palette_action_available(
            &graph,
            1,
            &selected_nodes,
            OperatorPaletteAction::CollapseSelectionToSubnet,
        ));
        assert!(!operator_palette_action_available(
            &graph,
            1,
            &[1],
            OperatorPaletteAction::CollapseSelectionToSubnet,
        ));

        let entries = operator_palette_entries(&graph, 1, &selected_nodes, true, false);
        let subnet_entry = entries
            .iter()
            .find(|entry| entry.action == OperatorPaletteAction::CollapseSelectionToSubnet)
            .expect("selected node set should expose subnet action");

        assert_eq!(subnet_entry.label, "Subnet from Selection");
        assert!(subnet_entry.aliases.contains(&"subnet"));
        assert!(subnet_entry.aliases.contains(&"digital asset"));
    }

    #[test]
    fn operator_palette_subnet_action_collapses_selected_node_set() {
        let mut graph = GraphDocument::sample();
        let mut panel = HoudiniGraphPanel::default();
        panel.set_selected_node_set(vec![1, 2]);

        assert!(panel.apply_operator_palette_action(
            &mut graph,
            OperatorPaletteAction::CollapseSelectionToSubnet,
        ));

        assert_eq!(
            graph.nodes[panel.selected_node].kind,
            NodeKind::GraphContainer
        );
        assert_eq!(panel.selected_nodes, vec![panel.selected_node]);
        assert_eq!(
            panel.operator_history.first(),
            Some(&OperatorPaletteAction::CollapseSelectionToSubnet)
        );
    }

    #[test]
    fn operator_palette_enters_and_exits_selected_subnet() {
        let mut graph = GraphDocument::sample();
        let mut panel = HoudiniGraphPanel::default();
        panel.set_selected_node_set(vec![1, 2]);
        assert!(panel.apply_operator_palette_action(
            &mut graph,
            OperatorPaletteAction::CollapseSelectionToSubnet,
        ));
        let container_index = panel.selected_node;

        assert!(operator_palette_action_available(
            &graph,
            container_index,
            &panel.selected_nodes,
            OperatorPaletteAction::EnterSelectedSubnet,
        ));
        assert!(
            panel.apply_operator_palette_action(
                &mut graph,
                OperatorPaletteAction::EnterSelectedSubnet,
            )
        );

        assert_ne!(graph.current_graph_id(), "main");
        assert_ne!(panel.selected_node, container_index);
        assert!(operator_palette_action_available(
            &graph,
            panel.selected_node,
            &panel.selected_nodes,
            OperatorPaletteAction::GoUpOneGraph,
        ));
        assert!(
            panel.apply_operator_palette_action(&mut graph, OperatorPaletteAction::GoUpOneGraph,)
        );

        assert_eq!(graph.current_graph_id(), "main");
        assert_eq!(panel.selected_node, container_index);
        assert_eq!(panel.selected_nodes, vec![container_index]);
    }

    #[test]
    fn operator_palette_creates_asset_from_selected_subnet() {
        let mut graph = GraphDocument::sample();
        let mut panel = HoudiniGraphPanel::default();
        panel.set_selected_node_set(vec![1, 2]);
        assert!(panel.apply_operator_palette_action(
            &mut graph,
            OperatorPaletteAction::CollapseSelectionToSubnet,
        ));
        let container_index = panel.selected_node;

        assert!(operator_palette_action_available(
            &graph,
            container_index,
            &panel.selected_nodes,
            OperatorPaletteAction::CreateAssetFromSelectedSubnet,
        ));
        assert!(panel.apply_operator_palette_action(
            &mut graph,
            OperatorPaletteAction::CreateAssetFromSelectedSubnet,
        ));

        assert_eq!(graph.procedural_asset_declarations.len(), 1);
        assert_eq!(
            graph.procedural_asset_declarations[0].asset_id,
            "project.asset.filter_selection_subnet"
        );
        assert_eq!(
            graph.procedural_asset_declarations[0].display_name,
            "Filter Selection Subnet"
        );
        assert_eq!(
            graph.procedural_asset_declarations[0].description,
            "Project-local asset from Filter Selection Subnet."
        );
        assert_eq!(
            graph.procedural_asset_declarations[0].help,
            "Created from the selected Houdini subnet."
        );
        assert_eq!(
            graph.procedural_asset_declarations[0]
                .wrapped_subgraph
                .graph_id,
            "graph.filter_selection_subnet"
        );
        assert!(
            panel
                .asset_status
                .as_deref()
                .is_some_and(|status| status.contains("Created project asset from selected subnet"))
        );
    }

    #[test]
    fn subnet_asset_metadata_preserves_user_fields() {
        let mut graph = GraphDocument::sample();
        let mut panel = HoudiniGraphPanel {
            asset_name: "Custom Cleanup".to_owned(),
            asset_description: "Custom description.".to_owned(),
            asset_help: "Custom help.".to_owned(),
            ..HoudiniGraphPanel::default()
        };
        panel.set_selected_node_set(vec![1, 2]);
        assert!(panel.apply_operator_palette_action(
            &mut graph,
            OperatorPaletteAction::CollapseSelectionToSubnet,
        ));

        let (name, description, help) = panel.selected_graph_container_asset_metadata(&graph);

        assert_eq!(name, "Custom Cleanup");
        assert_eq!(description, "Custom description.");
        assert_eq!(help, "Custom help.");
    }

    #[test]
    fn asset_gallery_jump_selects_usage_node_in_owning_graph() {
        let mut graph = GraphDocument::sample();
        let (asset_id, _) = graph.create_asset_instance_from_graph(
            "Gallery Cleanup",
            "Created for gallery navigation.",
            "Use from the asset gallery.",
        );
        graph.graph_registry.graphs.push(ProjectGraphMetadata {
            graph_id: "analysis".to_owned(),
            name: "Analysis".to_owned(),
            path: "/obj/analysis".to_owned(),
            role: ProjectGraphRole::Subgraph,
        });
        graph
            .select_graph_by_id("analysis")
            .expect("analysis graph should be selectable");
        let analysis_asset_index = graph.add_procedural_asset_node(asset_id);
        let mut panel = HoudiniGraphPanel::default();

        assert!(panel.jump_to_graph_node(&mut graph, analysis_asset_index, "analysis"));

        assert_eq!(graph.current_graph_id(), "analysis");
        assert_eq!(panel.selected_node, analysis_asset_index);
        assert_eq!(panel.selected_nodes, vec![analysis_asset_index]);
        assert!(panel.selected_annotation.is_none());
        assert!(panel.selected_edge.is_none());
        assert_eq!(panel.active_graph_pane, GraphWorkbenchPane::Info);
        assert!(panel.pending_frame_selected);
        assert!(
            panel
                .asset_status
                .as_deref()
                .is_some_and(|status| status.contains("/obj/analysis/Asset"))
        );
    }

    #[test]
    fn selected_asset_actions_match_and_upgrade_instances() {
        let mut graph = GraphDocument::sample();
        let (_asset_id, node_index) = graph.create_asset_instance_from_graph(
            "Action Cleanup",
            "Created for action tests.",
            "Use from the asset panel.",
        );
        let mut panel = HoudiniGraphPanel::default();
        panel.select_single_node(node_index);

        assert!(graph.set_procedural_asset_contents_unlocked(node_index, true));
        assert!(panel.match_selected_asset_definition(&mut graph));
        let matched_asset = graph.nodes[node_index]
            .procedural_asset
            .as_ref()
            .expect("asset node should exist");
        assert!(!matched_asset.contents_unlocked);
        assert_eq!(matched_asset.instance_version, "0.1.0");
        assert!(
            panel
                .asset_status
                .as_deref()
                .is_some_and(|status| status.contains("pinned definition"))
        );

        graph.procedural_asset_declarations[0].version = "0.2.0".to_owned();
        graph.refresh_asset_version_statuses();
        assert!(panel.upgrade_selected_asset_to_current_definition(&mut graph));
        let upgraded_asset = graph.nodes[node_index]
            .procedural_asset
            .as_ref()
            .expect("asset node should exist");
        assert_eq!(upgraded_asset.instance_version, "0.2.0");
        assert!(!upgraded_asset.contents_unlocked);
        assert!(
            panel
                .asset_status
                .as_deref()
                .is_some_and(|status| status.contains("from 0.1.0 to 0.2.0"))
        );
    }

    #[test]
    fn asset_gallery_usage_actions_repair_non_selected_instances() {
        let mut graph = GraphDocument::sample();
        let (asset_id, _) = graph.create_asset_instance_from_graph(
            "Gallery Actions",
            "Created for gallery action tests.",
            "Use from the asset gallery.",
        );
        let sibling_index = graph.add_procedural_asset_node(asset_id);
        assert!(graph.set_procedural_asset_contents_unlocked(sibling_index, true));
        graph.procedural_asset_declarations[0].version = "0.2.0".to_owned();
        graph.refresh_asset_version_statuses();

        let entry = graph
            .procedural_asset_gallery_entries()
            .into_iter()
            .find(|entry| entry.display_name == "Gallery Actions")
            .expect("asset gallery entry should exist");
        let usage = entry
            .usages
            .iter()
            .find(|usage| usage.node_index == sibling_index)
            .expect("sibling usage should be listed");
        assert!(usage.can_match_definition);
        assert!(usage.can_upgrade_to_current_definition);

        let mut panel = HoudiniGraphPanel::default();
        assert!(panel.match_asset_definition(&mut graph, sibling_index));
        assert!(
            !graph.nodes[sibling_index]
                .procedural_asset
                .as_ref()
                .expect("asset node should exist")
                .contents_unlocked
        );

        assert!(panel.upgrade_asset_to_current_definition(&mut graph, sibling_index));
        assert_eq!(
            graph.nodes[sibling_index]
                .procedural_asset
                .as_ref()
                .expect("asset node should exist")
                .instance_version,
            "0.2.0"
        );
    }

    #[test]
    fn asset_gallery_filter_matches_asset_metadata_and_usage_paths() {
        let mut graph = GraphDocument::sample();
        let (_asset_id, _) = graph.create_asset_instance_from_graph(
            "Gallery Filter",
            "Findable gallery description.",
            "Use from the asset gallery.",
        );
        graph.add_procedural_asset_node("vy.asset.missing_cleanup");
        let mut panel = HoudiniGraphPanel::default();

        panel.asset_gallery_filter = "filter".to_owned();
        let filtered_entries = panel.filtered_asset_gallery_entries(&graph);
        assert_eq!(filtered_entries.len(), 1);
        assert_eq!(filtered_entries[0].display_name, "Gallery Filter");

        panel.asset_gallery_filter = "/obj/main/Asset".to_owned();
        let path_entries = panel.filtered_asset_gallery_entries(&graph);
        assert!(
            path_entries
                .iter()
                .any(|entry| entry.display_name == "Gallery Filter")
        );

        panel.asset_gallery_filter = "missing_cleanup".to_owned();
        let missing_entries = panel.filtered_asset_gallery_entries(&graph);
        assert_eq!(missing_entries.len(), 1);
        assert!(missing_entries[0].missing_declaration);

        panel.asset_gallery_filter = "no such asset".to_owned();
        assert!(panel.filtered_asset_gallery_entries(&graph).is_empty());
    }

    #[test]
    fn asset_gallery_usages_group_by_graph_path() {
        let mut graph = GraphDocument::sample();
        let (asset_id, _) = graph.create_asset_instance_from_graph(
            "Grouped Gallery",
            "Created for grouping tests.",
            "Use from the asset gallery.",
        );
        graph.graph_registry.graphs.push(ProjectGraphMetadata {
            graph_id: "analysis".to_owned(),
            name: "Analysis".to_owned(),
            path: "/obj/analysis".to_owned(),
            role: ProjectGraphRole::Subgraph,
        });
        graph
            .select_graph_by_id("analysis")
            .expect("analysis graph should be selectable");
        graph.add_procedural_asset_node(asset_id);

        let entry = graph
            .procedural_asset_gallery_entries()
            .into_iter()
            .find(|entry| entry.display_name == "Grouped Gallery")
            .expect("asset gallery entry should exist");
        let groups = asset_usage_graph_groups(&entry.usages);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].graph_path, "/obj/analysis");
        assert_eq!(groups[0].usages.len(), 1);
        assert_eq!(groups[1].graph_path, "/obj/main");
        assert_eq!(groups[1].usages.len(), 1);
    }

    #[test]
    fn selected_node_collapse_ui_rejects_output_node() {
        let graph = GraphDocument::sample();
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == NodeKind::Output)
            .expect("sample graph should include output");
        let panel = HoudiniGraphPanel {
            selected_node: output_index,
            ..HoudiniGraphPanel::default()
        };

        assert!(!panel.selected_node_can_collapse_to_graph_container(&graph));
    }
}
