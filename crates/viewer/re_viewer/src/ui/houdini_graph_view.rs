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
    ui.painter()
        .rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);

    let viewport = rect.shrink2(egui::vec2(24.0, 22.0));
    ui.painter().rect_stroke(
        viewport,
        4.0,
        ui.visuals().widgets.noninteractive.bg_stroke,
        StrokeKind::Inside,
    );

    if graph.layer_visible(LayerKind::Debug) {
        draw_debug_boundary(ui, viewport, &scene);
    }

    for geometry in &scene.items {
        match geometry {
            RerunSceneItem::Polygon { points } => {
                let points = points
                    .iter()
                    .map(|point| map_view_point(viewport, *point))
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
                draw_native_cubic(ui, viewport, *curve, scene.stroke_scale);
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

fn draw_debug_boundary(ui: &mut egui::Ui, viewport: Rect, scene: &RerunSceneOutput) {
    let painter = ui.painter();
    let control_stroke = Stroke::new(1.0, Color32::from_rgb(150, 150, 150));
    let export_stroke = Stroke::new(1.0, Color32::from_rgb(115, 210, 155));

    for geometry in &scene.debug_items {
        match geometry {
            RerunSceneDebugItem::PreparedExportPolyline(points) => {
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
            RerunSceneDebugItem::NativeCubicControlPolygon(control_points) => {
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
}

fn map_view_point(rect: Rect, point: GraphPoint) -> Pos2 {
    let x = rect.left() + point.x * rect.width();
    let y = rect.bottom() - point.y * rect.height();
    Pos2::new(x, y)
}
