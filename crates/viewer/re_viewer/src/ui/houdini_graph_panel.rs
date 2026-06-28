use egui::{Slider, Ui};
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

            ui.horizontal_wrapped(|ui| {
                for (index, node) in self.graph.nodes.iter().enumerate() {
                    if index > 0 {
                        ui.label("->");
                    }

                    if ui
                        .selectable_label(self.selected_node == index, node.name)
                        .clicked()
                    {
                        self.selected_node = index;
                    }
                }
            });

            ui.add_space(8.0);
            ui.strong("Layers");
            for layer in &mut self.graph.layers {
                ui.re_checkbox(&mut layer.visible, layer.name);
            }

            ui.add_space(8.0);
            ui.strong("Parameters");
            if let Some(node) = self.graph.nodes.get_mut(self.selected_node) {
                ui.label(node.info);
                ui.add(Slider::new(&mut node.weight, 0.0..=1.0).text("Weight"));
            }

            ui.add_space(8.0);
            ui.strong("Native Geometry");
            ui.label(format!(
                "{} polygons, {} cubic Bezier curves",
                self.graph.polygon_count(),
                self.graph.cubic_bezier_count()
            ));
            ui.label(format!(
                "{} polygon vertices, {} cubic control points",
                self.graph.polygon_vertex_count(),
                self.graph.cubic_control_point_count()
            ));
        });
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
                    info: "Loads polygon and cubic Bezier records.",
                },
                GraphNode {
                    name: "Filter",
                    weight: 0.55,
                    info: "Filters features by style weight.",
                },
                GraphNode {
                    name: "Style",
                    weight: 0.75,
                    info: "Assigns visual parameters before viewer output.",
                },
                GraphNode {
                    name: "Rerun Output",
                    weight: 1.0,
                    info: "Prepares adaptive viewer geometry only at the output edge.",
                },
            ],
            layers: vec![
                Layer {
                    name: "Polygons",
                    visible: true,
                },
                Layer {
                    name: "Curves",
                    visible: true,
                },
                Layer {
                    name: "Debug Output",
                    visible: false,
                },
            ],
            geometry: vec![
                Geometry::Polygon(Polygon {
                    points: vec![(0.0, 0.0), (1.0, 0.1), (0.8, 1.0), (0.0, 0.8)],
                }),
                Geometry::CubicBezier(CubicBezier {
                    start: (0.0, 0.0),
                    control_1: (0.25, 1.0),
                    control_2: (0.75, -0.4),
                    end: (1.0, 0.6),
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
}

struct GraphNode {
    name: &'static str,
    weight: f32,
    info: &'static str,
}

struct Layer {
    name: &'static str,
    visible: bool,
}

enum Geometry {
    Polygon(Polygon),
    CubicBezier(CubicBezier),
}

struct Polygon {
    points: Vec<(f32, f32)>,
}

struct CubicBezier {
    start: (f32, f32),
    control_1: (f32, f32),
    control_2: (f32, f32),
    end: (f32, f32),
}
