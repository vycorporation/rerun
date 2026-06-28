use std::sync::{Arc, Mutex, MutexGuard};

use egui::{
    Align2, Color32, FontId, Pos2, Rect, Response, Sense, Slider, Stroke, StrokeKind, Ui, Vec2,
};
use re_ui::UiExt as _;

pub(crate) mod model;

use self::model::{
    ExportGeometry, GeometryBounds, GraphDocument, GraphPoint, GraphStyle, SourceMetadata,
};

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
}

impl Default for HoudiniGraphPanel {
    fn default() -> Self {
        Self {
            selected_node: 1,
            dragging_node: None,
            last_parquet_path: None,
            parquet_status: None,
            graph_document_status: None,
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
            for layer in &mut graph.layers {
                ui.re_checkbox(&mut layer.visible, layer.name);
            }

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

            ui.add_space(8.0);
            ui.strong("Node Info");
            self.node_info_ui(ui, &graph);

            ui.add_space(8.0);
            ui.strong("Pipeline Trace");
            self.pipeline_trace_ui(ui, &graph);

            ui.add_space(8.0);
            ui.strong("Graph Model");
            self.parquet_import_ui(ui, &mut graph);
            self.graph_document_ui(ui, &mut graph);
            ui.add_space(6.0);
            let export_output = graph.adaptive_export_output();
            let export_polyline_points = export_output
                .items
                .iter()
                .map(|geometry| match geometry {
                    ExportGeometry::Polygon(points) | ExportGeometry::Polyline(points) => {
                        points.len()
                    }
                })
                .sum::<usize>();
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
            ui.label(format!(
                "{} adaptive segments per emitted cubic at boundary",
                graph.export_segments()
            ));
            self.source_metadata_ui(ui, &graph.source.metadata, "graph_model");
        });
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
                });
            ui.label(info.summary);
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

fn format_list(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_owned()
    } else {
        values.join(", ")
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
