use std::sync::{Arc, Mutex, MutexGuard};

use egui::{
    Align2, Color32, DragValue, FontId, Pos2, Rect, Response, Sense, Slider, Stroke, StrokeKind,
    Ui, Vec2,
};
use re_ui::UiExt as _;

pub(crate) mod model;

use self::model::{
    AttributeTableQuery, AttributeTableRow, AttributeTableSort, EvaluationState, GeometryBounds,
    GraphDocument, GraphPoint, GraphStyle, HoudiniNodeBinding, LayerKind, NodeStatus,
    PythonEnvironmentResolveTrigger, PythonEnvironmentStatus, PythonOperatorDependencyStatus,
    SourceMetadata,
};

const LARGE_ATTRIBUTE_TABLE_ROW_LIMIT: usize = 2_500;
const ATTRIBUTE_TABLE_PREVIEW_ROWS: usize = 200;

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
    dragging_node: Option<usize>,
    last_parquet_path: Option<String>,
    parquet_status: Option<String>,
    graph_document_status: Option<String>,
    recording_status: Option<String>,
    benchmark_status: Option<String>,
    benchmark_curve_count: usize,
    benchmark_polygon_count: usize,
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
            dragging_node: None,
            last_parquet_path: None,
            parquet_status: None,
            graph_document_status: None,
            recording_status: None,
            benchmark_status: None,
            benchmark_curve_count: 10_000,
            benchmark_polygon_count: 1_000,
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

impl HoudiniGraphPanel {
    pub(crate) fn show(&mut self, ui: &mut Ui, shared_graph: &SharedHoudiniGraph) {
        install_shared_houdini_graph(ui.ctx(), shared_graph);
        let mut graph = lock_houdini_graph(shared_graph);
        egui::Frame {
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.strong("Houdini Graph");
            ui.small("Product-fork spike panel");
            ui.add_space(6.0);

            ui.strong("Graph Canvas");
            self.node_graph_ui(ui, &mut graph);

            ui.add_space(8.0);
            ui.strong("Layers");
            self.layer_stack_ui(ui, &mut graph);

            ui.add_space(8.0);
            ui.strong("Parameters");
            if let Some(node) = graph.nodes.get_mut(self.selected_node) {
                ui.label(node.info);
                ui.add(
                    Slider::new(&mut node.parameter.value, node.parameter.range.clone())
                        .text(node.parameter.name),
                )
                .on_hover_text(node.parameter.help);
            }
            self.evaluation_controls_ui(ui, &mut graph);

            ui.add_space(8.0);
            ui.strong("Node Info");
            self.node_info_ui(ui, &graph);

            ui.add_space(8.0);
            ui.strong("Pipeline Trace");
            self.pipeline_trace_ui(ui, &graph);

            ui.add_space(8.0);
            ui.strong("Attribute Table");
            self.attribute_table_ui(ui, &mut graph);

            ui.add_space(8.0);
            ui.strong("Graph Model");
            self.parquet_import_ui(ui, &mut graph);
            self.render_benchmark_ui(ui, &mut graph);
            self.graph_document_ui(ui, &mut graph);
            self.asset_authoring_ui(ui, &mut graph);
            self.recording_export_ui(ui, &graph);
            self.python_environment_ui(ui, &mut graph);
            ui.add_space(6.0);
            let export_polyline_points = graph.prepared_export_point_count();
            ui.label(format!(
                "Source: {} ({} matching entities, {} visible query results)",
                graph.source.as_str(),
                graph.source.matching_entity_count,
                graph.source.visible_data_result_count
            ));
            ui.label(format!(
                "Provenance: {}",
                graph.source.metadata.provenance.as_str()
            ));
            if let Some(source_path) = &graph.source.source_path {
                ui.label(format!("Source path: {source_path}"));
            }
            if let Some(import_error) = &graph.source.import_error {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    format!("Source error: {import_error}"),
                );
            }
            ui.label(format!(
                "{} source polygons, {} source cubic Bezier curves",
                graph.polygon_count(),
                graph.cubic_bezier_count()
            ));
            ui.label(format!(
                "{} polygon vertices, {} cubic control points",
                graph.polygon_vertex_count(),
                graph.cubic_control_point_count()
            ));
            ui.label(format!(
                "{} visible output items after layer and filter controls",
                graph.visible_output_count()
            ));
            ui.label(format!(
                "{} prepared export points at output boundary",
                export_polyline_points
            ));
            let feasibility = graph.render_feasibility_summary();
            ui.label(format!(
                "{} native viewer primitives, {} graph-owned control/vertex points",
                feasibility.native_viewer_primitive_count, feasibility.graph_owned_point_count
            ));
            ui.label(format!(
                "{} prepared boundary/debug points (not stored graph geometry)",
                feasibility.prepared_boundary_debug_point_count
            ));
            ui.label(format!(
                "{} adaptive segments per emitted cubic at boundary",
                graph.export_segments()
            ));
            self.source_metadata_ui(ui, &graph.source.metadata, "graph_model");
        });
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

    fn layer_stack_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) {
        ui.horizontal(|ui| {
            if ui.button("Add OUT Null").clicked() {
                let index = graph.add_null_operator_node("OUT_MAIN");
                self.selected_node = index;
            }
            if ui.button("Duplicate Polygons").clicked() {
                graph.duplicate_layer_view(LayerKind::Polygons, "Polygons Copy");
            }
            if ui.button("Duplicate Curves").clicked() {
                graph.duplicate_layer_view(LayerKind::Curves, "Curves Copy");
            }
        });

        egui::Grid::new("houdini_graph_layer_stack")
            .num_columns(4)
            .spacing([10.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.weak("Visible");
                ui.weak("Order");
                ui.weak("Name");
                ui.weak("Kind");
                ui.end_row();

                for layer in &mut graph.layers {
                    ui.re_checkbox(&mut layer.visible, "");
                    ui.add(egui::DragValue::new(&mut layer.order).speed(1));
                    ui.text_edit_singleline(&mut layer.name);
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

    fn node_graph_ui(&mut self, ui: &mut Ui, graph: &mut GraphDocument) -> Response {
        let desired_size = egui::vec2(ui.available_width().max(280.0), 176.0);
        let (response, painter) = ui.allocate_painter(desired_size, Sense::click_and_drag());
        let canvas_rect = response.rect;
        painter.rect_filled(canvas_rect, 4.0, ui.visuals().extreme_bg_color);
        painter.rect_stroke(
            canvas_rect,
            4.0,
            ui.visuals().widgets.noninteractive.bg_stroke,
            StrokeKind::Inside,
        );

        let layout_rect = canvas_rect.shrink2(egui::vec2(12.0, 10.0));
        let node_size = Vec2::new(116.0, 48.0);
        let mut node_rects = layout_node_rects(graph, layout_rect, node_size);
        let generated_lane_y = layout_rect.top() + layout_rect.height() * 0.82;
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

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if response.clicked() || response.drag_started() {
                self.dragging_node = None;
                for (index, node_rect) in node_rects.iter().enumerate() {
                    if node_rect.contains(pointer_pos) {
                        self.selected_node = index;
                        self.dragging_node = Some(index);
                        break;
                    }
                }
            }

            if response.dragged() {
                if let Some(dragging_node) = self.dragging_node {
                    graph.set_node_layout_position(
                        dragging_node,
                        unmap_node_layout_point(layout_rect, pointer_pos, node_size),
                    );
                    node_rects = layout_node_rects(graph, layout_rect, node_size);
                }
            }
        }

        if ui.input(|input| input.pointer.any_released()) {
            self.dragging_node = None;
        }

        let connector_stroke =
            Stroke::new(1.5, ui.visuals().widgets.noninteractive.fg_stroke.color);
        for edge in graph.graph_layout().edges {
            let from_rect = node_rects[edge.from_node];
            let to_rect = node_rects[edge.to_node];
            let start = Pos2::new(from_rect.right(), from_rect.center().y);
            let end = Pos2::new(to_rect.left(), to_rect.center().y);
            painter.line_segment([start, end], connector_stroke);
            draw_arrowhead(&painter, end, connector_stroke.color);
        }

        for layout_node in graph.graph_layout().nodes {
            let Some(node) = graph.nodes.get(layout_node.node_index) else {
                continue;
            };
            let node_rect = node_rects[layout_node.node_index];
            let selected = self.selected_node == layout_node.node_index;
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
                layout_node.name,
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
        }

        response
    }

    fn node_info_ui(&self, ui: &mut Ui, graph: &GraphDocument) {
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
                            "instance {} / current {} / {}",
                            asset.instance_version,
                            asset.current_version.as_deref().unwrap_or("missing"),
                            asset.version_status.as_str()
                        ));
                        ui.end_row();

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

fn map_node_layout_point(rect: Rect, point: GraphPoint, node_size: Vec2) -> Pos2 {
    let usable_width = (rect.width() - node_size.x).max(1.0);
    let usable_height = (rect.height() - node_size.y).max(1.0);
    Pos2::new(
        rect.left() + node_size.x * 0.5 + usable_width * point.x,
        rect.top() + node_size.y * 0.5 + usable_height * point.y,
    )
}

fn unmap_node_layout_point(rect: Rect, position: Pos2, node_size: Vec2) -> GraphPoint {
    let usable_width = (rect.width() - node_size.x).max(1.0);
    let usable_height = (rect.height() - node_size.y).max(1.0);
    GraphPoint {
        x: ((position.x - rect.left() - node_size.x * 0.5) / usable_width).clamp(0.0, 1.0),
        y: ((position.y - rect.top() - node_size.y * 0.5) / usable_height).clamp(0.0, 1.0),
    }
}

fn layout_node_rects(graph: &GraphDocument, rect: Rect, node_size: Vec2) -> Vec<Rect> {
    let layout = graph.graph_layout();
    let mut node_rects = vec![Rect::NOTHING; graph.nodes.len()];
    for layout_node in &layout.nodes {
        node_rects[layout_node.node_index] = Rect::from_center_size(
            map_node_layout_point(rect, layout_node.position, node_size),
            node_size,
        );
    }
    node_rects
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
