use std::sync::{Arc, Mutex, MutexGuard};

use egui::{
    Align2, Color32, DragValue, FontId, Pos2, Rect, Response, Sense, Slider, Stroke, StrokeKind,
    Ui, Vec2,
};
use re_ui::UiExt as _;

pub(crate) mod model;

use self::model::{
    AttributeTableQuery, AttributeTableRow, AttributeTableSort, EvaluationState, GeometryBounds,
    GraphAnnotationKind, GraphDocument, GraphPoint, GraphStyle, HoudiniNodeBinding, LayerKind,
    NetworkBadgeVisibility, NetworkNodeRingVisibility, NetworkViewDisplayOptions, NodeStatus,
    PythonEnvironmentResolveTrigger, PythonEnvironmentStatus, PythonOperatorDependencyStatus,
    SourceMetadata, SubstrateCoordinateContract,
};

const LARGE_ATTRIBUTE_TABLE_ROW_LIMIT: usize = 2_500;
const ATTRIBUTE_TABLE_PREVIEW_ROWS: usize = 200;
const NETWORK_BOX_FAST_DRAG_PEAK_DELTA_PIXELS: f32 = 18.0;
const NETWORK_DISPLAY_OPTIONS_ID: &str = "houdini_graph_network_display_options";

pub(crate) type SharedHoudiniGraph = Arc<Mutex<GraphDocument>>;

pub(crate) fn new_shared_houdini_graph() -> SharedHoudiniGraph {
    Arc::new(Mutex::new(GraphDocument::sample()))
}

pub(crate) fn install_shared_houdini_graph(egui_ctx: &egui::Context, graph: &SharedHoudiniGraph) {
    egui_ctx.data_mut(|data| data.insert_temp(shared_houdini_graph_id(), graph.clone()));
}

pub(crate) fn shared_houdini_graph_from_context(
    egui_ctx: &egui::Context,
) -> Option<SharedHoudiniGraph> {
    egui_ctx.data(|data| data.get_temp(shared_houdini_graph_id()))
}

pub(crate) fn lock_houdini_graph(graph: &SharedHoudiniGraph) -> MutexGuard<'_, GraphDocument> {
    graph
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn shared_houdini_graph_id() -> egui::Id {
    egui::Id::new("houdini_graph_state")
}

pub(crate) struct HoudiniGraphPanel {
    selected_node: usize,
    active_workspace: HoudiniGraphWorkspace,
    active_graph_pane: GraphWorkbenchPane,
    dragging_node: Option<usize>,
    node_drag_peak_delta_pixels: f32,
    dragging_annotation: Option<usize>,
    resizing_annotation: Option<usize>,
    graph_view_zoom: f32,
    graph_view_pan: Vec2,
    pending_frame_selected: bool,
    tab_menu_open: bool,
    tab_menu_anchor: Pos2,
    tab_menu_filter_needs_focus: bool,
    last_parquet_path: Option<String>,
    parquet_status: Option<String>,
    graph_document_status: Option<String>,
    recording_status: Option<String>,
    benchmark_status: Option<String>,
    benchmark_curve_count: usize,
    benchmark_polygon_count: usize,
    operator_filter: String,
    operator_history: Vec<OperatorPaletteAction>,
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
    asset_name: String,
    asset_description: String,
    asset_help: String,
    asset_status: Option<String>,
    python_uv_executable_path: String,
    python_existing_environment_path: String,
    python_create_environment_path: String,
}

impl Default for HoudiniGraphPanel {
    fn default() -> Self {
        Self {
            selected_node: 1,
            active_workspace: HoudiniGraphWorkspace::Graph,
            active_graph_pane: GraphWorkbenchPane::Parameters,
            dragging_node: None,
            node_drag_peak_delta_pixels: 0.0,
            dragging_annotation: None,
            resizing_annotation: None,
            graph_view_zoom: 1.0,
            graph_view_pan: Vec2::ZERO,
            pending_frame_selected: false,
            tab_menu_open: false,
            tab_menu_anchor: Pos2::ZERO,
            tab_menu_filter_needs_focus: false,
            last_parquet_path: None,
            parquet_status: None,
            graph_document_status: None,
            recording_status: None,
            benchmark_status: None,
            benchmark_curve_count: 10_000,
            benchmark_polygon_count: 1_000,
            operator_filter: String::new(),
            operator_history: Vec::new(),
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
            asset_name: "Curve cleanup".to_owned(),
            asset_description: "Project-local graph asset.".to_owned(),
            asset_help: "Created from the current Houdini graph.".to_owned(),
            asset_status: None,
            python_uv_executable_path: String::new(),
            python_existing_environment_path: String::new(),
            python_create_environment_path: String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HoudiniGraphWorkspace {
    Graph,
    Inspect,
    Data,
    Outputs,
    Project,
}

impl HoudiniGraphWorkspace {
    const ALL: [Self; 5] = [
        Self::Graph,
        Self::Inspect,
        Self::Data,
        Self::Outputs,
        Self::Project,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Graph => "Graph",
            Self::Inspect => "Inspect",
            Self::Data => "Data",
            Self::Outputs => "Outputs",
            Self::Project => "Project",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GraphWorkbenchPane {
    Operators,
    Parameters,
    Info,
    Display,
    Layers,
}

impl GraphWorkbenchPane {
    const ALL: [Self; 5] = [
        Self::Operators,
        Self::Parameters,
        Self::Info,
        Self::Display,
        Self::Layers,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Operators => "Ops",
            Self::Parameters => "Parms",
            Self::Info => "Info",
            Self::Display => "Display",
            Self::Layers => "Layers",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperatorPaletteAction {
    AddOutNull,
    AddReference,
    AddRepairProjection,
    AddNetworkBox,
    AddStickyNote,
    DuplicatePolygons,
    DuplicateCurves,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperatorPaletteCategory {
    Create,
    Organize,
    LayerActions,
}

impl OperatorPaletteCategory {
    const ALL: [Self; 3] = [Self::Create, Self::Organize, Self::LayerActions];

    fn label(self) -> &'static str {
        match self {
            Self::Create => "Create",
            Self::Organize => "Organize",
            Self::LayerActions => "Layer Actions",
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Organize => "organize",
            Self::LayerActions => "layer_actions",
        }
    }

    fn default_open(self, filter_is_empty: bool) -> bool {
        !filter_is_empty || matches!(self, Self::Create)
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

impl HoudiniGraphPanel {
    pub(crate) fn show(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        install_shared_houdini_graph(ui.ctx(), shared_graph);
        let mut graph = lock_houdini_graph(shared_graph);
        egui::Frame {
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        }
        .show(ui, |ui| {
            self.workspace_tabs_ui(ui);
            ui.add_space(6.0);

            match self.active_workspace {
                HoudiniGraphWorkspace::Graph => self.graph_workspace_ui(ui, &mut graph),
                HoudiniGraphWorkspace::Inspect => self.inspect_workspace_ui(ui, &mut graph),
                HoudiniGraphWorkspace::Data => self.data_workspace_ui(ui, &mut graph),
                HoudiniGraphWorkspace::Outputs => self.outputs_workspace_ui(ui, &mut graph),
                HoudiniGraphWorkspace::Project => self.project_workspace_ui(ui, &mut graph),
            }
        });
    }

    fn workspace_tabs_ui(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.strong("Houdini Graph");
            ui.separator();
            for workspace in HoudiniGraphWorkspace::ALL {
                if ui
                    .selectable_label(self.active_workspace == workspace, workspace.label())
                    .clicked()
                {
                    self.active_workspace = workspace;
                }
            }
        });
    }

    fn graph_workspace_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        let available_width = ui.available_width();
        let available_height = ui.available_height();
        let pane_height = if available_height.is_finite() && available_height > 0.0 {
            available_height.clamp(380.0, 720.0)
        } else {
            520.0
        };
        let wide_workbench = available_width >= 680.0;

        if wide_workbench {
            let side_width = (available_width * 0.30).clamp(260.0, 360.0);
            let canvas_width = (available_width - side_width - 14.0).max(360.0);

            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(canvas_width, pane_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_min_size(egui::vec2(canvas_width, pane_height));
                        ui.strong("Network Editor");
                        ui.add_space(4.0);
                        self.network_editor_toolbar_ui(ui, graph);
                        ui.add_space(4.0);
                        self.node_graph_ui(ui, graph, (pane_height - 58.0).max(320.0));
                    },
                );

                ui.separator();

                ui.allocate_ui_with_layout(
                    egui::vec2(side_width, pane_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_min_size(egui::vec2(side_width, pane_height));
                        egui::ScrollArea::vertical()
                            .id_salt("houdini_graph_workbench_pane_scroll")
                            .auto_shrink([false, false])
                            .max_height(pane_height)
                            .show(ui, |ui| {
                                self.graph_workbench_side_strip_ui(ui, graph);
                            });
                    },
                );
            });
        } else {
            ui.strong("Network Editor");
            self.network_editor_toolbar_ui(ui, graph);
            ui.add_space(4.0);
            self.node_graph_ui(ui, graph, 340.0);

            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .id_salt("houdini_graph_workbench_narrow_pane_scroll")
                .auto_shrink([false, false])
                .max_height(360.0)
                .show(ui, |ui| {
                    self.graph_workbench_side_strip_ui(ui, graph);
                });
        }
    }

    fn network_editor_toolbar_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        let selected_name = graph
            .nodes
            .get(self.selected_node)
            .map(|node| node.name.clone())
            .unwrap_or_else(|| "none".to_owned());

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
                if ui.button("Parameters").clicked() {
                    self.active_graph_pane = GraphWorkbenchPane::Parameters;
                    ui.close();
                }
                if ui.button("Node Information").clicked() {
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
                if ui.button("Edit Comment").clicked() {
                    self.node_info_open = true;
                    self.active_graph_pane = GraphWorkbenchPane::Info;
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
                    self.active_graph_pane = GraphWorkbenchPane::Display;
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
                    self.active_graph_pane = GraphWorkbenchPane::Operators;
                    ui.close();
                }
                if ui.button("Parameters").clicked() {
                    self.active_graph_pane = GraphWorkbenchPane::Parameters;
                    ui.close();
                }
                if ui.button("Node Info").clicked() {
                    self.node_info_open = true;
                    self.active_graph_pane = GraphWorkbenchPane::Info;
                    ui.close();
                }
                if ui.button("Display").clicked() {
                    self.active_graph_pane = GraphWorkbenchPane::Display;
                    ui.close();
                }
                if ui.button("Layers").clicked() {
                    self.active_graph_pane = GraphWorkbenchPane::Layers;
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
            });

            ui.separator();
            ui.weak(selected_name);
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

    fn resize_selected_network_box_to_contents(&mut self, graph: &mut GraphDocument) -> bool {
        let Some(annotation_index) = graph.annotations.iter().position(|annotation| {
            annotation.kind == GraphAnnotationKind::NetworkBox
                && graph.nodes.get(self.selected_node).is_some_and(|node| {
                    annotation
                        .member_node_ids
                        .iter()
                        .any(|member_id| member_id == &node.node_id)
                })
        }) else {
            return false;
        };
        graph.resize_network_box_to_contents(annotation_index)
    }

    fn resize_all_network_boxes_to_contents(&mut self, graph: &mut GraphDocument) {
        let network_box_indices = graph
            .annotations
            .iter()
            .enumerate()
            .filter_map(|(index, annotation)| {
                (annotation.kind == GraphAnnotationKind::NetworkBox).then_some(index)
            })
            .collect::<Vec<_>>();
        for index in network_box_indices {
            graph.resize_network_box_to_contents(index);
        }
    }

    fn open_operator_chooser_at(&mut self, anchor: Pos2) {
        self.operator_filter.clear();
        self.tab_menu_open = true;
        self.tab_menu_anchor = anchor + egui::vec2(6.0, 6.0);
        self.tab_menu_filter_needs_focus = true;
        self.active_graph_pane = GraphWorkbenchPane::Operators;
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
        if !operator_palette_action_available(graph, self.selected_node, action) {
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
        if !operator_palette_action_available(graph, self.selected_node, action) {
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
                        if !operator_palette_action_available(graph, self.selected_node, action) {
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

    fn first_matching_operator_palette_action(
        &self,
        graph: &GraphDocument,
        include_organization: bool,
        include_layers: bool,
    ) -> Option<OperatorPaletteAction> {
        let filter = self.operator_filter.trim().to_lowercase();
        let entries = operator_palette_entries(
            graph,
            self.selected_node,
            include_organization,
            include_layers,
        );
        OperatorPaletteCategory::ALL
            .into_iter()
            .find_map(|category| {
                entries
                    .iter()
                    .find(|entry| {
                        entry.category == category
                            && operator_matches(&filter, entry.label, entry.aliases)
                    })
                    .map(|entry| entry.action)
            })
    }

    fn apply_operator_palette_action(
        &mut self,
        graph: &mut GraphDocument,
        action: OperatorPaletteAction,
    ) -> bool {
        let applied = match action {
            OperatorPaletteAction::AddOutNull => {
                self.selected_node = graph.add_null_operator_node("OUT_MAIN");
                self.node_info_open = true;
                self.active_graph_pane = GraphWorkbenchPane::Parameters;
                true
            }
            OperatorPaletteAction::AddReference => {
                if let Some(index) = graph.add_reference_input_node(self.selected_node) {
                    self.selected_node = index;
                    self.node_info_open = true;
                    self.active_graph_pane = GraphWorkbenchPane::Parameters;
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
                    self.selected_node = index;
                    self.node_info_open = true;
                    self.active_graph_pane = GraphWorkbenchPane::Parameters;
                    true
                } else {
                    false
                }
            }
            OperatorPaletteAction::AddNetworkBox => {
                graph.add_network_box_for_node(self.selected_node);
                self.active_graph_pane = GraphWorkbenchPane::Operators;
                true
            }
            OperatorPaletteAction::AddStickyNote => {
                graph.add_sticky_note_near_node(self.selected_node);
                self.active_graph_pane = GraphWorkbenchPane::Operators;
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

    fn inspect_workspace_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.strong("Parameters");
        self.selected_node_controls_ui(ui, graph);

        ui.add_space(8.0);
        ui.strong("Node Info");
        self.node_info_ui(ui, graph);

        ui.add_space(8.0);
        ui.strong("Pipeline Trace");
        self.pipeline_trace_ui(ui, graph);
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

    fn project_workspace_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.strong("Graph Model");
        self.graph_document_ui(ui, graph);

        ui.add_space(8.0);
        ui.strong("Asset");
        self.asset_authoring_ui(ui, graph);

        ui.add_space(8.0);
        ui.strong("Python");
        self.python_environment_ui(ui, graph);

        ui.add_space(8.0);
        ui.strong("Source");
        self.source_summary_ui(ui, graph);
    }

    fn selected_node_controls_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        let mut selected_parameter_changed = false;
        if let Some(node) = graph.nodes.get_mut(self.selected_node) {
            ui.label(node.info);
            selected_parameter_changed = ui
                .add(
                    Slider::new(&mut node.parameter.value, node.parameter.range.clone())
                        .text(node.parameter.name),
                )
                .on_hover_text(node.parameter.help)
                .changed();
        }
        if selected_parameter_changed {
            graph.mark_reference_inputs_stale_for_target_index(self.selected_node);
        }
        self.evaluation_controls_ui(ui, graph);
    }

    fn graph_workbench_side_strip_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        self.graph_workbench_pane_tabs_ui(ui);
        ui.separator();

        match self.active_graph_pane {
            GraphWorkbenchPane::Operators => {
                self.operator_strip_ui(ui, graph);
                ui.add_space(8.0);
                self.network_organization_ui(ui, graph);
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

    fn graph_workbench_pane_tabs_ui(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for pane in GraphWorkbenchPane::ALL {
                if ui
                    .selectable_label(self.active_graph_pane == pane, pane.label())
                    .clicked()
                {
                    self.active_graph_pane = pane;
                }
            }
        });
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
                badge_visibility_combo_ui(ui, "Error Badge", &mut options.error_badge);
                badge_visibility_combo_ui(ui, "Warning Badge", &mut options.warning_badge);
                badge_visibility_combo_ui(ui, "Comment Badge", &mut options.comment_badge);
                badge_visibility_combo_ui(
                    ui,
                    "Time Dependent Badge",
                    &mut options.time_dependent_badge,
                );
                badge_visibility_combo_ui(ui, "Asset Lock Badge", &mut options.lock_badge);
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
                for annotation_index in 0..graph.annotations.len() {
                    let mut resize_to_contents = false;
                    let mut position_update = None;
                    {
                        let annotation = &mut graph.annotations[annotation_index];
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.weak(annotation.kind.as_str());
                            ui.re_checkbox(&mut annotation.collapsed, "Collapsed");
                        });
                        ui.add(
                            egui::TextEdit::singleline(&mut annotation.title)
                                .desired_width(ui.available_width().max(80.0))
                                .hint_text("title"),
                        );
                        if annotation.kind == GraphAnnotationKind::StickyNote {
                            ui.add(
                                egui::TextEdit::multiline(&mut annotation.text)
                                    .desired_rows(2)
                                    .hint_text("note"),
                            );
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

                        ui.horizontal(|ui| {
                            ui.weak("Size");
                            ui.add(
                                DragValue::new(&mut annotation.size.x)
                                    .speed(0.01)
                                    .range(0.08..=0.95),
                            );
                            ui.add(
                                DragValue::new(&mut annotation.size.y)
                                    .speed(0.01)
                                    .range(0.08..=0.95),
                            );
                        });
                        let original_position = annotation.position;
                        let mut next_position = annotation.position;
                        ui.horizontal(|ui| {
                            ui.weak("Pos");
                            ui.add(DragValue::new(&mut next_position.x).speed(0.01));
                            ui.add(DragValue::new(&mut next_position.y).speed(0.01));
                        });
                        if next_position != original_position {
                            position_update =
                                Some((annotation.kind, original_position, next_position));
                        }
                    }

                    if let Some((kind, original_position, next_position)) = position_update {
                        if kind == GraphAnnotationKind::NetworkBox {
                            graph.translate_annotation(
                                annotation_index,
                                GraphPoint {
                                    x: next_position.x - original_position.x,
                                    y: next_position.y - original_position.y,
                                },
                            );
                        } else if let Some(annotation) = graph.annotations.get_mut(annotation_index)
                        {
                            annotation.position = next_position;
                        }
                    }
                    if resize_to_contents {
                        graph.resize_network_box_to_contents(annotation_index);
                    }
                }

                if graph.annotations.is_empty() {
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
        ui.weak(info.summary);
        ui.horizontal_wrapped(|ui| {
            ui.weak("Time dependent");
            ui.label("No");
        });

        for warning in &info.warnings {
            ui.colored_label(ui.visuals().warn_fg_color, warning);
        }

        self.selected_node_comment_ui(ui, graph);

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

    fn selected_node_comment_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
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
        ui.re_checkbox(&mut node.show_comment_in_network, "Show comment in Network");
    }

    fn graph_workbench_additional_node_info_ui(
        &mut self,
        ui: &mut Ui,
        info: &self::model::NodeInfo,
    ) {
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
                    warning.target_node_name,
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

    fn source_summary_ui(&self, ui: &mut Ui, graph: &GraphDocument) {
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
        }
        if let Some(status) = &self.asset_status {
            ui.weak(status);
        }
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
                                "Saved recording: {} ({} items, {} polygons, {} native cubics). {}",
                                recording.path.display(),
                                recording.item_count,
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
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            ui.weak("Recording export is available in the native viewer.");
        }

        if let Some(status) = &self.recording_status {
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

        if let Some(node) = graph.nodes.get(self.selected_node) {
            ui.weak(format!("Selected: {} ({})", node.name, node.kind.as_str()));
        }

        self.operator_palette_ui(
            ui,
            graph,
            OperatorPaletteUiOptions {
                id_salt: "houdini_operator_side_palette",
                grouped: true,
                show_recent: true,
                include_organization: false,
                include_layers: true,
                highlighted_action: None,
            },
        );
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

                for layer in &mut graph.layers {
                    ui.re_checkbox(&mut layer.visible, "");
                    ui.add(DragValue::new(&mut layer.order).speed(1).range(-99..=99));
                    ui.add(egui::TextEdit::singleline(&mut layer.name).desired_width(96.0));
                    ui.label(layer.kind.as_str());
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

    fn evaluation_controls_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        if self.selected_node >= graph.nodes.len() {
            return;
        }

        ui.horizontal(|ui| {
            let mut manual = graph.nodes[self.selected_node].evaluation.manual;
            if ui.re_checkbox(&mut manual, "Manual").changed() {
                graph.set_node_manual(self.selected_node, manual);
            }

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
            layout_changed |= self.frame_selected_node_in_rect(graph, layout_rect, node_size);
            self.pending_frame_selected = false;
        }
        if response.hovered() {
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
                    input.key_pressed(egui::Key::O) && shift_only,
                    input.key_pressed(egui::Key::P) && shift_only,
                    input.key_pressed(egui::Key::M) && shift_only,
                )
            });
            let (
                display_options_pressed,
                tab_pressed,
                pointer_anchor,
                home_pressed,
                frame_selected_pressed,
                add_network_box_pressed,
                add_sticky_note_pressed,
                resize_box_pressed,
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
                layout_changed |= self.frame_selected_node_in_rect(graph, layout_rect, node_size);
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

        for annotation in &graph.annotations {
            draw_graph_annotation(
                &painter,
                layout_rect,
                annotation,
                self.graph_view_zoom,
                self.graph_view_pan,
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
                self.dragging_node = None;
                self.dragging_annotation = None;
                self.resizing_annotation = None;
                let mut hit_node = false;
                for (index, node_rect) in node_rects.iter().enumerate() {
                    let ring_visible = node_ring_visible(
                        network_view.node_ring_visibility,
                        self.selected_node == index,
                        node_rect.contains(pointer_pos)
                            || node_ring_action_at(*node_rect, pointer_pos, self.graph_view_zoom)
                                .is_some(),
                    );
                    if ring_visible
                        && let Some(ring_action) =
                            node_ring_action_at(*node_rect, pointer_pos, self.graph_view_zoom)
                    {
                        self.selected_node = index;
                        self.apply_node_ring_action(graph, index, ring_action);
                        hit_node = true;
                        break;
                    }
                    if node_rect.contains(pointer_pos) {
                        self.selected_node = index;
                        self.node_info_open = true;
                        self.dragging_node = Some(index);
                        self.node_drag_peak_delta_pixels = 0.0;
                        hit_node = true;
                        break;
                    }
                }

                let mut hit_annotation = false;
                if !hit_node {
                    for (index, annotation_rect) in annotation_rects.iter().enumerate().rev() {
                        if annotation_collapse_toggle_rect(*annotation_rect).contains(pointer_pos) {
                            if let Some(annotation) = graph.annotations.get_mut(index) {
                                annotation.collapsed = !annotation.collapsed;
                            }
                            hit_annotation = true;
                            break;
                        }
                        if annotation_resize_handle_rect(*annotation_rect).contains(pointer_pos) {
                            self.resizing_annotation = Some(index);
                            hit_annotation = true;
                            break;
                        }
                        if annotation_rect.contains(pointer_pos) {
                            self.dragging_annotation = Some(index);
                            hit_annotation = true;
                            break;
                        }
                    }
                }

                if response.clicked_by(egui::PointerButton::Primary)
                    && !hit_node
                    && !hit_annotation
                    && !self.node_info_pinned
                {
                    self.node_info_open = false;
                }
            }

            if response.clicked_by(egui::PointerButton::Secondary) {
                for (index, node_rect) in node_rects.iter().enumerate() {
                    if node_rect.contains(pointer_pos) {
                        self.selected_node = index;
                        self.node_info_open = true;
                        break;
                    }
                }
            }

            if response.dragged_by(egui::PointerButton::Primary) {
                if let Some(dragging_node) = self.dragging_node {
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
                    if let Some(annotation) = graph.annotations.get_mut(resizing_annotation) {
                        let pointer_delta = ui.input(|input| input.pointer.delta());
                        annotation.size.x = (annotation.size.x
                            + pointer_delta.x / (layout_rect.width() * self.graph_view_zoom))
                            .clamp(0.08, 0.95);
                        annotation.size.y = (annotation.size.y
                            + pointer_delta.y / (layout_rect.height() * self.graph_view_zoom))
                            .clamp(0.08, 0.95);
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
                }
            }
        }

        if ui.input(|input| input.pointer.any_released()) {
            if let Some(dragging_node) = self.dragging_node {
                graph.settle_node_drag_for_network_boxes(
                    dragging_node,
                    self.node_drag_peak_delta_pixels >= NETWORK_BOX_FAST_DRAG_PEAK_DELTA_PIXELS,
                );
            }
            self.dragging_node = None;
            self.node_drag_peak_delta_pixels = 0.0;
            self.dragging_annotation = None;
            self.resizing_annotation = None;
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
            let connector_stroke = Stroke::new(1.5, faded_color(connector_color, fade));
            painter.line_segment([start, end], connector_stroke);
            draw_arrowhead(&painter, end, connector_stroke.color);
        }

        for layout_node in graph.graph_layout().nodes {
            let Some(node) = graph.nodes.get(layout_node.node_index) else {
                continue;
            };
            let node_rect = node_rects[layout_node.node_index];
            let selected = self.selected_node == layout_node.node_index;
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
            } else {
                ui.visuals().widgets.inactive.bg_fill
            };
            let stroke = if selected {
                Stroke::new(1.5, ui.visuals().selection.stroke.color)
            } else {
                ui.visuals().widgets.inactive.fg_stroke
            };

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
            if node.generated.is_some() {
                painter.text(
                    node_rect.right_top() + egui::vec2(-6.0, 6.0),
                    Align2::RIGHT_TOP,
                    "gen",
                    FontId::monospace(10.0),
                    ui.visuals().warn_fg_color,
                );
            }
            if node.show_comment_in_network && !node.comment.trim().is_empty() {
                painter.text(
                    node_rect.right_center() + egui::vec2(10.0, 0.0),
                    Align2::LEFT_CENTER,
                    format_node_comment(&node.comment),
                    FontId::proportional(12.0),
                    ui.visuals().weak_text_color(),
                );
            }
            draw_node_badges(&painter, node_rect, node, network_view, ui.visuals());
        }

        if let Some((node_index, action, pointer_pos)) = hovered_ring_action
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
                    if ui.small_button("Clear").clicked() {
                        self.operator_filter.clear();
                        filter_response.request_focus();
                    }
                });
                let highlighted_action =
                    self.first_matching_operator_palette_action(graph, true, false);
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
        if ui.button("Run Selected").clicked() {
            graph.request_node_run(self.selected_node);
            graph.complete_node_run(self.selected_node);
            ui.close();
        }
        if ui.button("Evaluate Output").clicked() {
            graph.demand_output_evaluation();
            ui.close();
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
                if let Some(node) = graph.nodes.get_mut(node_index) {
                    node.participates_in_output = !node.participates_in_output;
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

    fn node_info_ui(&mut self, ui: &mut Ui, graph: &GraphDocument) {
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
                                        self.selected_node = target_node_index;
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
                                            self.selected_node = consumer.reference_node_index;
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
                                warning.target_node_name,
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

    fn source_metadata_ui(&self, ui: &mut Ui, metadata: &SourceMetadata, id_suffix: &'static str) {
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
                            self.attribute_table_row_ui(ui, &row);
                        }
                    });
            });
    }

    fn attribute_table_row_ui(&self, ui: &mut Ui, row: &AttributeTableRow) {
        ui.label(row.record_index.to_string());
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

fn operator_matches(filter: &str, label: &str, aliases: &[&str]) -> bool {
    filter.is_empty()
        || label.to_lowercase().contains(filter)
        || aliases.iter().any(|alias| alias.contains(filter))
}

fn operator_palette_entries(
    graph: &GraphDocument,
    selected_node: usize,
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
    action: OperatorPaletteAction,
) -> bool {
    match action {
        OperatorPaletteAction::AddRepairProjection => graph
            .reference_coordinate_repair_summary(selected_node)
            .is_some(),
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
    node: &self::model::GraphNode,
    network_view: NetworkViewDisplayOptions,
    visuals: &egui::Visuals,
) {
    let mut badges = Vec::new();
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

fn layout_annotation_rects(graph: &GraphDocument, rect: Rect, zoom: f32, pan: Vec2) -> Vec<Rect> {
    graph
        .annotations
        .iter()
        .map(|annotation| display_annotation_rect(rect, annotation, zoom, pan))
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
    visuals: &egui::Visuals,
) {
    let annotation_rect = display_annotation_rect(layout_rect, annotation, zoom, pan);
    match annotation.kind {
        GraphAnnotationKind::NetworkBox => {
            let body_fill = Color32::from_rgba_unmultiplied(150, 150, 150, 72);
            let header_fill = Color32::from_rgba_unmultiplied(185, 185, 185, 132);
            let stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(210, 210, 210, 150));
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
            let stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(230, 210, 72, 210));
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

fn format_sticky_note_text(text: &str) -> String {
    let text = text.trim().replace('\n', " ");
    if text.chars().count() <= 56 {
        text
    } else {
        let prefix = text.chars().take(55).collect::<String>();
        format!("{prefix}…")
    }
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
