use egui::epaint::CubicBezierShape;
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind};
use re_sdk_types::ViewClassIdentifier;
use re_ui::{Help, icons};
use re_viewer_context::external::re_log_types::EntityPath;
use re_viewer_context::{
    IdentifiedViewSystem, IndicatedEntities, Item, PerVisualizerType, RecommendedVisualizers,
    SystemCommand, SystemCommandSender as _, SystemExecutionOutput, ViewClass,
    ViewClassLayoutPriority, ViewClassRegistryError, ViewContext, ViewContextCollection, ViewQuery,
    ViewSpawnHeuristics, ViewState, ViewStateExt as _, ViewSystemExecutionError,
    ViewSystemIdentifier, ViewSystemRegistrator, ViewerContext, VisualizableReason,
    VisualizerExecutionOutput, VisualizerQueryInfo, VisualizerSystem,
};

use crate::ui::houdini_graph_panel::model::{
    CubicBezier, GraphDocument, GraphPoint, LayerKind, RerunQueryBridge, RerunQueryBridgeMode,
    RerunSceneDebugItem, RerunSceneItem, RerunSceneOutput,
};
use crate::ui::houdini_graph_panel::{lock_houdini_graph, shared_houdini_graph_from_context};

#[derive(Default)]
pub(crate) struct HoudiniGraphView;

#[derive(Default)]
struct HoudiniGraphViewState {}

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

#[derive(Default)]
struct HoudiniGraphSourceVisualizer;

impl IdentifiedViewSystem for HoudiniGraphSourceVisualizer {
    fn identifier() -> ViewSystemIdentifier {
        re_viewer_context::external::re_string_interner::intern_static!(
            ViewSystemIdentifier,
            "HoudiniGraphSource"
        )
    }
}

impl VisualizerSystem for HoudiniGraphSourceVisualizer {
    fn visualizer_query_info(
        &self,
        _app_options: &re_viewer_context::AppOptions,
    ) -> VisualizerQueryInfo {
        VisualizerQueryInfo::empty()
    }

    fn execute(
        &self,
        _ctx: &ViewContext<'_>,
        _query: &ViewQuery<'_>,
        _context_systems: &ViewContextCollection,
    ) -> Result<VisualizerExecutionOutput, ViewSystemExecutionError> {
        Ok(VisualizerExecutionOutput::default())
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
        system_registry: &mut ViewSystemRegistrator<'_>,
    ) -> Result<(), ViewClassRegistryError> {
        system_registry.register_visualizer::<HoudiniGraphSourceVisualizer>()
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

    fn recommended_visualizers_for_entity(
        &self,
        _entity_path: &EntityPath,
        visualizers_with_reason: &[(ViewSystemIdentifier, &VisualizableReason)],
        _indicated_entities_per_visualizer: &PerVisualizerType<&IndicatedEntities>,
    ) -> RecommendedVisualizers {
        if visualizers_with_reason
            .iter()
            .any(|(visualizer, _reason)| *visualizer == HoudiniGraphSourceVisualizer::identifier())
        {
            RecommendedVisualizers::default(HoudiniGraphSourceVisualizer::identifier())
        } else {
            RecommendedVisualizers::empty()
        }
    }

    fn selection_ui(
        &self,
        _ctx: &ViewerContext<'_>,
        ui: &mut egui::Ui,
        state: &mut dyn ViewState,
        _space_origin: &EntityPath,
        _view_id: re_viewer_context::ViewId,
    ) -> Result<(), ViewSystemExecutionError> {
        state.downcast_ref::<HoudiniGraphViewState>()?;
        if let Some(shared_graph) = shared_houdini_graph_from_context(ui.ctx()) {
            let graph = lock_houdini_graph(&shared_graph);
            ui.label("Product-fork spike view. The graph model is not Rerun viewer state.");
            ui.label(format!(
                "{} polygons, {} native cubic Bezier curves",
                graph.polygon_count(),
                graph.cubic_bezier_count()
            ));
            ui.label(format!(
                "{} adaptive export segments at the current output boundary",
                graph.export_segments()
            ));
        } else {
            ui.weak("Houdini graph state is not installed for this frame.");
        }
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
        state.downcast_mut::<HoudiniGraphViewState>()?;
        let rect = ui.max_rect();
        let response = ui.allocate_rect(rect, Sense::click());

        if response.hovered() {
            ctx.selection_state().set_hovered(Item::View(query.view_id));
        }

        if response.clicked() {
            ctx.command_sender()
                .send_system(SystemCommand::set_selection(Item::View(query.view_id)));
        }

        if let Some(shared_graph) = shared_houdini_graph_from_context(ui.ctx()) {
            let query_bridge = query_bridge_from_view_query(ctx, query);
            let mut graph = lock_houdini_graph(&shared_graph);
            graph.update_source_from_query_bridge(&query_bridge);
            draw_houdini_output_view(ui, rect, &graph, query_bridge);
        } else {
            ui.painter()
                .rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Houdini graph state is not installed for this frame.",
                FontId::proportional(13.0),
                ui.visuals().weak_text_color(),
            );
        }
        Ok(())
    }
}

fn query_bridge_from_view_query(
    ctx: &ViewerContext<'_>,
    query: &ViewQuery<'_>,
) -> RerunQueryBridge {
    let query_result = ctx.lookup_query_result(query.view_id);
    let visible_data_result_count = query_result
        .tree
        .iter_data_results()
        .filter(|data_result| data_result.visible && !data_result.tree_prefix_only)
        .count();

    RerunQueryBridge {
        mode: RerunQueryBridgeMode::ProductForkViewOwned,
        view_id: query.view_id.to_string(),
        space_origin: query.space_origin.to_string(),
        timeline: query.timeline.to_string(),
        latest_at: query.latest_at.as_i64(),
        matching_entity_count: query_result.num_matching_entities,
        visualized_entity_count: query_result.num_visualized_entities,
        visible_data_result_count,
    }
}

fn draw_houdini_output_view(
    ui: &mut egui::Ui,
    rect: Rect,
    graph: &GraphDocument,
    query_bridge: RerunQueryBridge,
) {
    let scene = graph.rerun_scene_output_with_query_bridge(Some(query_bridge));
    let view_transform =
        HoudiniViewTransform::from_scene(&scene, rect.shrink2(egui::vec2(24.0, 22.0)));
    ui.painter()
        .rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);

    let viewport = view_transform.viewport;
    ui.painter().rect_stroke(
        viewport,
        4.0,
        ui.visuals().widgets.noninteractive.bg_stroke,
        StrokeKind::Inside,
    );

    if graph.layer_visible(LayerKind::Debug) {
        draw_debug_boundary(ui, view_transform, &scene);
    }

    for geometry in &scene.items {
        match geometry {
            RerunSceneItem::Polygon { points } => {
                let points = points
                    .iter()
                    .map(|point| view_transform.map_point(*point))
                    .collect::<Vec<_>>();
                ui.painter().add(egui::Shape::convex_polygon(
                    points.clone(),
                    Color32::from_rgba_unmultiplied(38, 125, 255, 50),
                    Stroke::new(
                        1.0 + 3.0 * scene.stroke_scale,
                        Color32::from_rgb(91, 169, 255),
                    ),
                ));
                for point in points {
                    ui.painter()
                        .circle_filled(point, 3.5, Color32::from_rgb(131, 192, 255));
                }
            }
            RerunSceneItem::NativeCubicBezier(curve) => {
                draw_native_cubic(ui, view_transform, *curve, scene.stroke_scale);
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
            scene.items.len(),
            scene.export_segments
        ),
        FontId::monospace(11.0),
        ui.visuals().weak_text_color(),
    );
    if let Some(query_bridge) = &scene.query_bridge {
        ui.painter().text(
            rect.left_top() + egui::vec2(14.0, 50.0),
            Align2::LEFT_TOP,
            format!(
                "{}: {} visible query results at {}={}",
                query_bridge.mode.as_str(),
                query_bridge.visible_data_result_count,
                query_bridge.timeline,
                query_bridge.latest_at
            ),
            FontId::monospace(11.0),
            ui.visuals().weak_text_color(),
        );
    }
}

fn draw_native_cubic(
    ui: &mut egui::Ui,
    view_transform: HoudiniViewTransform,
    curve: CubicBezier,
    stroke_scale: f32,
) {
    let painter = ui.painter();
    let points = curve
        .control_points()
        .map(|point| view_transform.map_point(point));
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

fn draw_debug_boundary(
    ui: &mut egui::Ui,
    view_transform: HoudiniViewTransform,
    scene: &RerunSceneOutput,
) {
    let painter = ui.painter();
    let control_stroke = Stroke::new(1.0, Color32::from_rgb(150, 150, 150));
    let export_stroke = Stroke::new(1.0, Color32::from_rgb(115, 210, 155));

    for geometry in &scene.debug_items {
        match geometry {
            RerunSceneDebugItem::PreparedExportPolyline(points) => {
                for pair in points.windows(2) {
                    painter.line_segment(
                        [
                            view_transform.map_point(pair[0]),
                            view_transform.map_point(pair[1]),
                        ],
                        export_stroke,
                    );
                }
            }
            RerunSceneDebugItem::NativeCubicControlPolygon(control_points) => {
                for pair in control_points.windows(2) {
                    painter.line_segment(
                        [
                            view_transform.map_point(pair[0]),
                            view_transform.map_point(pair[1]),
                        ],
                        control_stroke,
                    );
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct HoudiniViewTransform {
    viewport: Rect,
    content_rect: Rect,
    bounds: GraphBounds,
}

impl HoudiniViewTransform {
    fn from_scene(scene: &RerunSceneOutput, viewport: Rect) -> Self {
        let bounds = GraphBounds::from_scene(scene).unwrap_or_else(GraphBounds::unit);
        Self {
            viewport,
            content_rect: fitted_content_rect(viewport, bounds),
            bounds,
        }
    }

    fn map_point(self, point: GraphPoint) -> Pos2 {
        let width = self.bounds.width();
        let height = self.bounds.height();
        let x = self.content_rect.left()
            + ((point.x - self.bounds.min.x) / width) * self.content_rect.width();
        let y = self.content_rect.bottom()
            - ((point.y - self.bounds.min.y) / height) * self.content_rect.height();
        Pos2::new(x, y)
    }
}

fn fitted_content_rect(viewport: Rect, bounds: GraphBounds) -> Rect {
    let data_aspect = bounds.width() / bounds.height();
    let viewport_aspect = viewport.width() / viewport.height();

    if data_aspect > viewport_aspect {
        let height = viewport.width() / data_aspect;
        Rect::from_center_size(viewport.center(), egui::vec2(viewport.width(), height))
    } else {
        let width = viewport.height() * data_aspect;
        Rect::from_center_size(viewport.center(), egui::vec2(width, viewport.height()))
    }
}

#[derive(Clone, Copy)]
struct GraphBounds {
    min: GraphPoint,
    max: GraphPoint,
}

impl GraphBounds {
    const MIN_SPAN: f32 = 1.0e-3;
    const PADDING_FRACTION: f32 = 0.08;

    fn unit() -> Self {
        Self {
            min: GraphPoint { x: 0.0, y: 0.0 },
            max: GraphPoint { x: 1.0, y: 1.0 },
        }
    }

    fn from_scene(scene: &RerunSceneOutput) -> Option<Self> {
        let mut bounds = None;
        for item in &scene.items {
            match item {
                RerunSceneItem::Polygon { points } => {
                    for point in points {
                        Self::include_point(&mut bounds, *point);
                    }
                }
                RerunSceneItem::NativeCubicBezier(curve) => {
                    for point in curve.control_points() {
                        Self::include_point(&mut bounds, point);
                    }
                }
            }
        }
        bounds.map(Self::expanded_for_view)
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

    fn expanded_for_view(self) -> Self {
        let center = GraphPoint {
            x: (self.min.x + self.max.x) * 0.5,
            y: (self.min.y + self.max.y) * 0.5,
        };
        let fallback_span = self.width().max(self.height()).max(1.0);
        let width = self
            .width()
            .max(Self::MIN_SPAN)
            .max(if self.width() <= Self::MIN_SPAN {
                fallback_span
            } else {
                0.0
            });
        let height = self
            .height()
            .max(Self::MIN_SPAN)
            .max(if self.height() <= Self::MIN_SPAN {
                fallback_span
            } else {
                0.0
            });
        let x_padding = width * Self::PADDING_FRACTION;
        let y_padding = height * Self::PADDING_FRACTION;

        Self {
            min: GraphPoint {
                x: center.x - width * 0.5 - x_padding,
                y: center.y - height * 0.5 - y_padding,
            },
            max: GraphPoint {
                x: center.x + width * 0.5 + x_padding,
                y: center.y + height * 0.5 + y_padding,
            },
        }
    }

    fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    fn height(self) -> f32 {
        self.max.y - self.min.y
    }
}

#[cfg(test)]
mod tests {
    use super::{GraphBounds, HoudiniViewTransform};
    use crate::ui::houdini_graph_panel::model::{
        CubicBezier, GraphPoint, RerunSceneItem, RerunSceneOutput,
    };

    fn test_viewport() -> egui::Rect {
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(200.0, 100.0))
    }

    fn point(x: f32, y: f32) -> GraphPoint {
        GraphPoint { x, y }
    }

    #[test]
    fn view_transform_fits_arbitrary_cubic_coordinates_without_mutating_them() {
        let curve = CubicBezier {
            start: point(100.0, -50.0),
            control_1: point(125.0, 150.0),
            control_2: point(200.0, -25.0),
            end: point(250.0, 100.0),
            score: 1.0,
        };
        let scene = RerunSceneOutput {
            items: vec![RerunSceneItem::NativeCubicBezier(curve)],
            debug_items: vec![],
            stroke_scale: 1.0,
            export_segments: 8,
            query_bridge: None,
        };

        let transform = HoudiniViewTransform::from_scene(&scene, test_viewport());

        for point in curve.control_points() {
            assert!(test_viewport().contains(transform.map_point(point)));
        }
        assert_eq!(curve.start, point(100.0, -50.0));
    }

    #[test]
    fn view_bounds_expand_flat_geometry() {
        let scene = RerunSceneOutput {
            items: vec![RerunSceneItem::Polygon {
                points: vec![point(2.0, 5.0), point(4.0, 5.0)],
            }],
            debug_items: vec![],
            stroke_scale: 1.0,
            export_segments: 8,
            query_bridge: None,
        };

        let bounds = GraphBounds::from_scene(&scene).expect("flat geometry still has bounds");

        assert!(bounds.width() > 2.0);
        assert!(bounds.height() > 0.0);
    }

    #[test]
    fn view_transform_preserves_geometry_aspect_ratio() {
        let scene = RerunSceneOutput {
            items: vec![RerunSceneItem::Polygon {
                points: vec![point(0.0, 0.0), point(10.0, 10.0)],
            }],
            debug_items: vec![],
            stroke_scale: 1.0,
            export_segments: 8,
            query_bridge: None,
        };

        let transform = HoudiniViewTransform::from_scene(&scene, test_viewport());
        let min = transform.map_point(point(0.0, 0.0));
        let max = transform.map_point(point(10.0, 10.0));

        assert!(((max.x - min.x).abs() - (max.y - min.y).abs()).abs() < 0.01);
    }
}
