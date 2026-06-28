use std::sync::{Arc, Mutex, MutexGuard};

use egui::{
    Align2, Color32, FontId, Pos2, Rect, Response, Sense, Slider, Stroke, StrokeKind, Ui, Vec2,
};
use re_ui::UiExt as _;

pub(crate) mod model;

use self::model::{ExportGeometry, GraphDocument, GraphPoint};

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
}

impl Default for HoudiniGraphPanel {
    fn default() -> Self {
        Self { selected_node: 1 }
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
            self.node_graph_ui(ui, &graph);

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
        });
    }

    fn node_graph_ui(&mut self, ui: &mut Ui, graph: &GraphDocument) -> Response {
        let desired_size = egui::vec2(ui.available_width().max(280.0), 176.0);
        let (response, painter) = ui.allocate_painter(desired_size, Sense::click());
        let rect = response.rect;
        painter.rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);
        painter.rect_stroke(
            rect,
            4.0,
            ui.visuals().widgets.noninteractive.bg_stroke,
            StrokeKind::Inside,
        );

        let rect = rect.shrink2(egui::vec2(12.0, 10.0));
        let node_size = Vec2::new(116.0, 48.0);
        let layout = graph.graph_layout();
        let mut node_rects = vec![Rect::NOTHING; graph.nodes.len()];
        for layout_node in &layout.nodes {
            node_rects[layout_node.node_index] = Rect::from_center_size(
                map_node_layout_point(rect, layout_node.position, node_size),
                node_size,
            );
        }

        let connector_stroke =
            Stroke::new(1.5, ui.visuals().widgets.noninteractive.fg_stroke.color);
        for edge in &layout.edges {
            let from_rect = node_rects[edge.from_node];
            let to_rect = node_rects[edge.to_node];
            let start = Pos2::new(from_rect.right(), from_rect.center().y);
            let end = Pos2::new(to_rect.left(), to_rect.center().y);
            painter.line_segment([start, end], connector_stroke);
            draw_arrowhead(&painter, end, connector_stroke.color);
        }

        let pointer_pos = response.interact_pointer_pos();
        if response.clicked() {
            if let Some(pointer_pos) = pointer_pos {
                for (index, node_rect) in node_rects.iter().enumerate() {
                    if node_rect.contains(pointer_pos) {
                        self.selected_node = index;
                    }
                }
            }
        }

        for layout_node in &layout.nodes {
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
                });
            ui.label(info.summary);
        }
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

fn map_node_layout_point(rect: Rect, point: GraphPoint, node_size: Vec2) -> Pos2 {
    let usable_width = (rect.width() - node_size.x).max(1.0);
    let usable_height = (rect.height() - node_size.y).max(1.0);
    Pos2::new(
        rect.left() + node_size.x * 0.5 + usable_width * point.x,
        rect.top() + node_size.y * 0.5 + usable_height * point.y,
    )
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
