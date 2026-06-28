use egui::epaint::CubicBezierShape;
use egui::{
    Align2, Color32, FontId, Pos2, Rect, Response, Sense, Slider, Stroke, StrokeKind, Ui, Vec2,
};
use re_ui::UiExt as _;

mod model;

use self::model::{ExportGeometry, GraphDocument, GraphPoint, LayerKind, ViewerGeometry};

pub(crate) struct HoudiniGraphPanel {
    graph: GraphDocument,
    selected_node: usize,
}

impl Default for HoudiniGraphPanel {
    fn default() -> Self {
        Self {
            graph: GraphDocument::sample(),
            selected_node: 1,
        }
    }
}

impl HoudiniGraphPanel {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        egui::Frame {
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.strong("Houdini Graph");
            ui.small("Product-fork spike panel");
            ui.add_space(6.0);

            ui.strong("Graph Canvas");
            self.node_graph_ui(ui);

            ui.add_space(8.0);
            ui.strong("Layers");
            for layer in &mut self.graph.layers {
                ui.re_checkbox(&mut layer.visible, layer.name);
            }

            ui.add_space(8.0);
            ui.strong("Parameters");
            if let Some(node) = self.graph.nodes.get_mut(self.selected_node) {
                ui.label(node.info);
                let response = ui.add(Slider::new(&mut node.weight, 0.0..=1.0).text(node.parameter));
                if node.name == "Rerun Output" {
                    response.on_hover_text(
                        "Controls only the prepared export polyline. The native cubic remains four points.",
                    );
                }
            }

            ui.add_space(8.0);
            ui.strong("Node Info");
            self.node_info_ui(ui);

            ui.add_space(8.0);
            ui.strong("Pipeline Trace");
            self.pipeline_trace_ui(ui);

            ui.add_space(8.0);
            ui.strong("Viewer Output Preview");
            self.output_preview_ui(ui);

            ui.add_space(8.0);
            ui.strong("Graph Model");
            let export_output = self.graph.adaptive_export_output();
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
                self.graph.polygon_count(),
                self.graph.cubic_bezier_count()
            ));
            ui.label(format!(
                "{} polygon vertices, {} cubic control points",
                self.graph.polygon_vertex_count(),
                self.graph.cubic_control_point_count()
            ));
            ui.label(format!(
                "{} visible output items after layer and filter controls",
                self.graph.visible_output_count()
            ));
            ui.label(format!(
                "{} prepared export points at output boundary",
                export_polyline_points
            ));
            ui.label(format!(
                "{} adaptive segments per emitted cubic at boundary",
                self.graph.export_segments()
            ));
        });
    }

    fn node_graph_ui(&mut self, ui: &mut Ui) -> Response {
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
        let layout = self.graph.graph_layout();
        let mut node_rects = vec![Rect::NOTHING; self.graph.nodes.len()];
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
            let Some(node) = self.graph.nodes.get(layout_node.node_index) else {
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
                format!("{:.2}", node.weight),
                FontId::monospace(11.0),
                ui.visuals().weak_text_color(),
            );
        }

        response
    }

    fn node_info_ui(&self, ui: &mut Ui) {
        if let Some(info) = self.graph.selected_node_info(self.selected_node) {
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
                        info.parameter_name, info.parameter_value
                    ));
                    ui.end_row();
                });
            ui.label(info.summary);
        }
    }

    fn pipeline_trace_ui(&self, ui: &mut Ui) {
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

                for stage in self.graph.pipeline_stages() {
                    ui.label(stage.name);
                    ui.label(stage.input_count.to_string());
                    ui.label(stage.output_count.to_string());
                    ui.label(stage.note);
                    ui.end_row();
                }
            });
    }

    fn output_preview_ui(&self, ui: &mut Ui) -> Response {
        let desired_size = egui::vec2(ui.available_width().max(280.0), 150.0);
        let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
        let rect = response.rect.shrink(4.0);
        let bg = ui.visuals().extreme_bg_color;
        let border = ui.visuals().widgets.noninteractive.bg_stroke;

        painter.rect_filled(rect, 4.0, bg);
        painter.rect_stroke(rect, 4.0, border, StrokeKind::Inside);

        let viewport = rect.shrink2(egui::vec2(16.0, 14.0));
        let output = self.graph.viewer_output();
        let debug_visible = self.graph.layer_visible(LayerKind::Debug);

        if debug_visible {
            self.debug_output_preview_ui(ui, viewport);
        }

        for geometry in &output.items {
            match geometry {
                ViewerGeometry::Polygon(polygon) => {
                    let points = polygon
                        .points
                        .iter()
                        .map(|point| map_preview_point(viewport, *point))
                        .collect::<Vec<_>>();
                    painter.add(egui::Shape::convex_polygon(
                        points.clone(),
                        Color32::from_rgba_unmultiplied(38, 125, 255, 45),
                        Stroke::new(
                            1.0 + 3.0 * output.stroke_scale,
                            Color32::from_rgb(91, 169, 255),
                        ),
                    ));
                    for point in points {
                        painter.circle_filled(point, 3.0, Color32::from_rgb(131, 192, 255));
                    }
                }
                ViewerGeometry::CubicBezier(curve) => {
                    let points = curve
                        .control_points()
                        .map(|point| map_preview_point(viewport, point));
                    painter.add(CubicBezierShape {
                        points,
                        closed: false,
                        fill: Color32::TRANSPARENT,
                        stroke: Stroke::new(
                            1.0 + 4.0 * output.stroke_scale,
                            Color32::from_rgb(239, 188, 84),
                        )
                        .into(),
                    });
                    for point in points {
                        painter.circle_filled(point, 2.5, Color32::from_rgb(250, 212, 124));
                    }
                }
            }
        }

        painter.text(
            rect.left_top() + egui::vec2(8.0, 8.0),
            Align2::LEFT_TOP,
            format!("{} emitted", self.graph.visible_output_count()),
            FontId::monospace(11.0),
            ui.visuals().weak_text_color(),
        );

        response
    }

    fn debug_output_preview_ui(&self, ui: &mut Ui, viewport: Rect) {
        let painter = ui.painter();
        let control_stroke = Stroke::new(1.0, Color32::from_rgb(150, 150, 150));
        let export_stroke = Stroke::new(1.0, Color32::from_rgb(115, 210, 155));
        let export_output = self.graph.adaptive_export_output();

        for geometry in &export_output.items {
            if let ExportGeometry::Polyline(points) = geometry {
                for pair in points.windows(2) {
                    painter.line_segment(
                        [
                            map_preview_point(viewport, pair[0]),
                            map_preview_point(viewport, pair[1]),
                        ],
                        export_stroke,
                    );
                }
                for point in points {
                    painter.circle_filled(
                        map_preview_point(viewport, *point),
                        1.5,
                        Color32::from_rgb(115, 210, 155),
                    );
                }
            }
        }

        for geometry in &self.graph.viewer_output().items {
            if let ViewerGeometry::CubicBezier(curve) = geometry {
                let control_points = curve.control_points();
                for pair in control_points.windows(2) {
                    painter.line_segment(
                        [
                            map_preview_point(viewport, pair[0]),
                            map_preview_point(viewport, pair[1]),
                        ],
                        control_stroke,
                    );
                }
            }
        }
    }
}

fn map_preview_point(rect: Rect, point: GraphPoint) -> Pos2 {
    let x = rect.left() + point.x * rect.width();
    let y = rect.bottom() - point.y * rect.height();
    Pos2::new(x, y)
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
