use egui::epaint::CubicBezierShape;
use egui::{
    Align2, Color32, FontId, Pos2, Rect, Response, Sense, Slider, Stroke, StrokeKind, Ui, Vec2,
};
use re_ui::UiExt as _;

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
                ui.add(Slider::new(&mut node.weight, 0.0..=1.0).text(node.parameter));
            }

            ui.add_space(8.0);
            ui.strong("Viewer Output Preview");
            self.output_preview_ui(ui);

            ui.add_space(8.0);
            ui.strong("Graph Model");
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
        });
    }

    fn node_graph_ui(&mut self, ui: &mut Ui) -> Response {
        let desired_size = egui::vec2(ui.available_width().max(280.0), 118.0);
        let (response, painter) = ui.allocate_painter(desired_size, Sense::click());
        let rect = response.rect;
        let lane_y = rect.center().y;
        let node_count = self.graph.nodes.len().max(1);
        let node_size = Vec2::new(116.0, 48.0);
        let usable_width = (rect.width() - node_size.x).max(1.0);

        let mut node_rects = Vec::with_capacity(node_count);
        for index in 0..node_count {
            let t = if node_count == 1 {
                0.5
            } else {
                index as f32 / (node_count - 1) as f32
            };
            let center = Pos2::new(rect.left() + node_size.x * 0.5 + usable_width * t, lane_y);
            node_rects.push(Rect::from_center_size(center, node_size));
        }

        let connector_stroke =
            Stroke::new(1.5, ui.visuals().widgets.noninteractive.fg_stroke.color);
        for pair in node_rects.windows(2) {
            let start = Pos2::new(pair[0].right(), pair[0].center().y);
            let end = Pos2::new(pair[1].left(), pair[1].center().y);
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

        for (index, (node, node_rect)) in self.graph.nodes.iter().zip(node_rects).enumerate() {
            let selected = self.selected_node == index;
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
                node.name,
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

    fn output_preview_ui(&self, ui: &mut Ui) -> Response {
        let desired_size = egui::vec2(ui.available_width().max(280.0), 150.0);
        let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
        let rect = response.rect.shrink(4.0);
        let bg = ui.visuals().extreme_bg_color;
        let border = ui.visuals().widgets.noninteractive.bg_stroke;

        painter.rect_filled(rect, 4.0, bg);
        painter.rect_stroke(rect, 4.0, border, StrokeKind::Inside);

        let viewport = rect.shrink2(egui::vec2(16.0, 14.0));
        let style_scale = self.graph.style_scale();

        for geometry in &self.graph.geometry {
            if !self.graph.emits(geometry) {
                continue;
            }

            match geometry {
                Geometry::Polygon(polygon) => {
                    let points = polygon
                        .points
                        .iter()
                        .map(|point| map_preview_point(viewport, *point))
                        .collect::<Vec<_>>();
                    painter.add(egui::Shape::convex_polygon(
                        points.clone(),
                        Color32::from_rgba_unmultiplied(38, 125, 255, 45),
                        Stroke::new(1.0 + 3.0 * style_scale, Color32::from_rgb(91, 169, 255)),
                    ));
                    for point in points {
                        painter.circle_filled(point, 3.0, Color32::from_rgb(131, 192, 255));
                    }
                }
                Geometry::CubicBezier(curve) => {
                    let points = [
                        map_preview_point(viewport, curve.start),
                        map_preview_point(viewport, curve.control_1),
                        map_preview_point(viewport, curve.control_2),
                        map_preview_point(viewport, curve.end),
                    ];
                    painter.add(CubicBezierShape {
                        points,
                        closed: false,
                        fill: Color32::TRANSPARENT,
                        stroke: Stroke::new(
                            1.0 + 4.0 * style_scale,
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
}

struct GraphDocument {
    nodes: Vec<GraphNode>,
    layers: Vec<Layer>,
    geometry: Vec<Geometry>,
}

impl GraphDocument {
    fn sample() -> Self {
        Self {
            nodes: vec![
                GraphNode {
                    name: "Source",
                    weight: 1.0,
                    parameter: "Read",
                    info: "Loads polygon and cubic Bezier records.",
                },
                GraphNode {
                    name: "Filter",
                    weight: 0.55,
                    parameter: "Minimum score",
                    info: "Filters features by sample score.",
                },
                GraphNode {
                    name: "Style",
                    weight: 0.75,
                    parameter: "Stroke scale",
                    info: "Assigns visual parameters before viewer output.",
                },
                GraphNode {
                    name: "Rerun Output",
                    weight: 1.0,
                    parameter: "Export",
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
                    points: vec![(0.0, 0.0), (1.0, 0.1), (0.8, 1.0), (0.0, 0.8)],
                    score: 0.62,
                }),
                Geometry::CubicBezier(CubicBezier {
                    start: (0.0, 0.0),
                    control_1: (0.25, 1.0),
                    control_2: (0.75, -0.4),
                    end: (1.0, 0.6),
                    score: 0.82,
                }),
            ],
        }
    }

    fn polygon_count(&self) -> usize {
        self.geometry
            .iter()
            .filter(|geometry| matches!(geometry, Geometry::Polygon(_)))
            .count()
    }

    fn cubic_bezier_count(&self) -> usize {
        self.geometry
            .iter()
            .filter(|geometry| matches!(geometry, Geometry::CubicBezier(_)))
            .count()
    }

    fn polygon_vertex_count(&self) -> usize {
        self.geometry
            .iter()
            .map(|geometry| match geometry {
                Geometry::Polygon(polygon) => polygon.points.len(),
                Geometry::CubicBezier(_) => 0,
            })
            .sum()
    }

    fn cubic_control_point_count(&self) -> usize {
        self.geometry
            .iter()
            .map(|geometry| match geometry {
                Geometry::Polygon(_) => 0,
                Geometry::CubicBezier(curve) => {
                    let points = [curve.start, curve.control_1, curve.control_2, curve.end];
                    points.len()
                }
            })
            .sum()
    }

    fn filter_minimum_score(&self) -> f32 {
        self.nodes
            .iter()
            .find(|node| node.name == "Filter")
            .map_or(0.0, |node| node.weight)
    }

    fn style_scale(&self) -> f32 {
        self.nodes
            .iter()
            .find(|node| node.name == "Style")
            .map_or(0.5, |node| node.weight)
    }

    fn layer_visible(&self, kind: LayerKind) -> bool {
        self.layers
            .iter()
            .find(|layer| layer.kind == kind)
            .is_some_and(|layer| layer.visible)
    }

    fn emits(&self, geometry: &Geometry) -> bool {
        let layer_visible = match geometry {
            Geometry::Polygon(_) => self.layer_visible(LayerKind::Polygons),
            Geometry::CubicBezier(_) => self.layer_visible(LayerKind::Curves),
        };

        layer_visible && geometry.score() >= self.filter_minimum_score()
    }

    fn visible_output_count(&self) -> usize {
        self.geometry
            .iter()
            .filter(|geometry| self.emits(geometry))
            .count()
    }
}

struct GraphNode {
    name: &'static str,
    weight: f32,
    parameter: &'static str,
    info: &'static str,
}

struct Layer {
    name: &'static str,
    kind: LayerKind,
    visible: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LayerKind {
    Polygons,
    Curves,
    Debug,
}

enum Geometry {
    Polygon(Polygon),
    CubicBezier(CubicBezier),
}

impl Geometry {
    fn score(&self) -> f32 {
        match self {
            Self::Polygon(polygon) => polygon.score,
            Self::CubicBezier(curve) => curve.score,
        }
    }
}

struct Polygon {
    points: Vec<(f32, f32)>,
    score: f32,
}

struct CubicBezier {
    start: (f32, f32),
    control_1: (f32, f32),
    control_2: (f32, f32),
    end: (f32, f32),
    score: f32,
}

fn map_preview_point(rect: Rect, point: (f32, f32)) -> Pos2 {
    let x = rect.left() + point.0 * rect.width();
    let y = rect.bottom() - point.1 * rect.height();
    Pos2::new(x, y)
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
