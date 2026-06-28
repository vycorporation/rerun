use egui::epaint::CubicBezierShape;
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind};
use re_sdk_types::ViewClassIdentifier;
use re_ui::{Help, icons};
use re_viewer_context::external::re_log_types::EntityPath;
use re_viewer_context::{
    Item, SystemCommand, SystemCommandSender as _, SystemExecutionOutput, ViewClass,
    ViewClassLayoutPriority, ViewClassRegistryError, ViewQuery, ViewSpawnHeuristics, ViewState,
    ViewStateExt as _, ViewSystemExecutionError, ViewSystemRegistrator, ViewerContext,
};

use crate::ui::houdini_graph_panel::model::{
    CubicBezier, ExportGeometry, GraphDocument, GraphPoint, LayerKind, ViewerGeometry,
};

#[derive(Default)]
pub(crate) struct HoudiniGraphView;

struct HoudiniGraphViewState {
    graph: GraphDocument,
}

impl Default for HoudiniGraphViewState {
    fn default() -> Self {
        Self {
            graph: GraphDocument::sample(),
        }
    }
}

impl ViewState for HoudiniGraphViewState {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn heap_size_bytes(&self) -> u64 {
        0
    }
}

impl ViewClass for HoudiniGraphView {
    fn identifier() -> ViewClassIdentifier {
        re_viewer_context::external::re_string_interner::intern_static!(
            ViewClassIdentifier,
            "HoudiniGraph"
        )
    }

    fn display_name(&self) -> &'static str {
        "Houdini Graph"
    }

    fn is_experimental(&self) -> bool {
        true
    }

    fn icon(&self) -> &'static re_ui::Icon {
        &icons::VIEW_GRAPH
    }

    fn help(&self, _os: egui::os::OperatingSystem) -> Help {
        Help::new("Houdini graph view")
            .markdown("Product-fork spike view for native polygon and cubic Bezier graph output.")
    }

    fn on_register(
        &self,
        _system_registry: &mut ViewSystemRegistrator<'_>,
    ) -> Result<(), ViewClassRegistryError> {
        Ok(())
    }

    fn new_state(&self) -> Box<dyn ViewState> {
        Box::<HoudiniGraphViewState>::default()
    }

    fn preferred_tile_aspect_ratio(&self, _state: &dyn ViewState) -> Option<f32> {
        Some(16.0 / 9.0)
    }

    fn layout_priority(&self) -> ViewClassLayoutPriority {
        ViewClassLayoutPriority::High
    }

    fn spawn_heuristics(
        &self,
        _ctx: &ViewerContext<'_>,
        _include_entity: &dyn Fn(&EntityPath) -> bool,
    ) -> ViewSpawnHeuristics {
        ViewSpawnHeuristics::root().with_max_views_spawned(1)
    }

    fn selection_ui(
        &self,
        _ctx: &ViewerContext<'_>,
        ui: &mut egui::Ui,
        state: &mut dyn ViewState,
        _space_origin: &EntityPath,
        _view_id: re_viewer_context::ViewId,
    ) -> Result<(), ViewSystemExecutionError> {
        let state = state.downcast_ref::<HoudiniGraphViewState>()?;
        ui.label("Product-fork spike view. The graph model is not Rerun viewer state.");
        ui.label(format!(
            "{} polygons, {} native cubic Bezier curves",
            state.graph.polygon_count(),
            state.graph.cubic_bezier_count()
        ));
        ui.label(format!(
            "{} adaptive export segments at the current output boundary",
            state.graph.export_segments()
        ));
        Ok(())
    }

    fn ui(
        &self,
        ctx: &ViewerContext<'_>,
        _missing_chunk_reporter: &re_viewer_context::MissingChunkReporter,
        ui: &mut egui::Ui,
        state: &mut dyn ViewState,
        query: &ViewQuery<'_>,
        _system_output: SystemExecutionOutput,
    ) -> Result<(), ViewSystemExecutionError> {
        let state = state.downcast_mut::<HoudiniGraphViewState>()?;
        let rect = ui.max_rect();
        let response = ui.allocate_rect(rect, Sense::click());

        if response.hovered() {
            ctx.selection_state().set_hovered(Item::View(query.view_id));
        }

        if response.clicked() {
            ctx.command_sender()
                .send_system(SystemCommand::set_selection(Item::View(query.view_id)));
        }

        draw_houdini_output_view(ui, rect, &state.graph);
        Ok(())
    }
}

fn draw_houdini_output_view(ui: &mut egui::Ui, rect: Rect, graph: &GraphDocument) {
    ui.painter()
        .rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);

    let viewport = rect.shrink2(egui::vec2(24.0, 22.0));
    ui.painter().rect_stroke(
        viewport,
        4.0,
        ui.visuals().widgets.noninteractive.bg_stroke,
        StrokeKind::Inside,
    );

    let output = graph.viewer_output();
    if graph.layer_visible(LayerKind::Debug) {
        draw_debug_boundary(ui, viewport, graph);
    }

    for geometry in &output.items {
        match geometry {
            ViewerGeometry::Polygon(polygon) => {
                let points = polygon
                    .points
                    .iter()
                    .map(|point| map_view_point(viewport, *point))
                    .collect::<Vec<_>>();
                ui.painter().add(egui::Shape::convex_polygon(
                    points.clone(),
                    Color32::from_rgba_unmultiplied(38, 125, 255, 50),
                    Stroke::new(
                        1.0 + 3.0 * output.stroke_scale,
                        Color32::from_rgb(91, 169, 255),
                    ),
                ));
                for point in points {
                    ui.painter()
                        .circle_filled(point, 3.5, Color32::from_rgb(131, 192, 255));
                }
            }
            ViewerGeometry::CubicBezier(curve) => {
                draw_native_cubic(ui, viewport, *curve, output.stroke_scale);
            }
        }
    }

    ui.painter().text(
        rect.left_top() + egui::vec2(14.0, 12.0),
        Align2::LEFT_TOP,
        "Houdini Graph Output",
        FontId::proportional(14.0),
        ui.visuals().text_color(),
    );
    ui.painter().text(
        rect.left_top() + egui::vec2(14.0, 32.0),
        Align2::LEFT_TOP,
        format!(
            "{} emitted, {} export segments per cubic",
            graph.visible_output_count(),
            graph.export_segments()
        ),
        FontId::monospace(11.0),
        ui.visuals().weak_text_color(),
    );
}

fn draw_native_cubic(ui: &mut egui::Ui, viewport: Rect, curve: CubicBezier, stroke_scale: f32) {
    let painter = ui.painter();
    let points = curve
        .control_points()
        .map(|point| map_view_point(viewport, point));
    painter.add(CubicBezierShape {
        points,
        closed: false,
        fill: Color32::TRANSPARENT,
        stroke: Stroke::new(1.0 + 4.0 * stroke_scale, Color32::from_rgb(239, 188, 84)).into(),
    });

    for point in points {
        painter.circle_filled(point, 3.0, Color32::from_rgb(250, 212, 124));
    }
}

fn draw_debug_boundary(ui: &mut egui::Ui, viewport: Rect, graph: &GraphDocument) {
    let painter = ui.painter();
    let control_stroke = Stroke::new(1.0, Color32::from_rgb(150, 150, 150));
    let export_stroke = Stroke::new(1.0, Color32::from_rgb(115, 210, 155));

    for geometry in &graph.adaptive_export_output().items {
        if let ExportGeometry::Polyline(points) = geometry {
            for pair in points.windows(2) {
                painter.line_segment(
                    [
                        map_view_point(viewport, pair[0]),
                        map_view_point(viewport, pair[1]),
                    ],
                    export_stroke,
                );
            }
        }
    }

    for geometry in &graph.viewer_output().items {
        if let ViewerGeometry::CubicBezier(curve) = geometry {
            let control_points = curve.control_points();
            for pair in control_points.windows(2) {
                painter.line_segment(
                    [
                        map_view_point(viewport, pair[0]),
                        map_view_point(viewport, pair[1]),
                    ],
                    control_stroke,
                );
            }
        }
    }
}

fn map_view_point(rect: Rect, point: GraphPoint) -> Pos2 {
    let x = rect.left() + point.x * rect.width();
    let y = rect.bottom() - point.y * rect.height();
    Pos2::new(x, y)
}
