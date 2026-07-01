use re_sdk_types::ViewClassIdentifier;
use re_ui::UICommandSender as _;
use re_viewer_context::ViewId;
use re_viewport_blueprint::{ViewBlueprint, ViewportBlueprint};

use crate::ui::{
    HoudiniDataView, HoudiniDisplayView, HoudiniFindView, HoudiniGraphView, HoudiniInfoView,
    HoudiniLayersView, HoudiniNetworkView, HoudiniOperatorsView, HoudiniOutputsView,
    HoudiniParametersView, HoudiniProjectView,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HoudiniWorkbenchPreset {
    NetworkAndInspector,
    HoudiniDefault,
    GraphReview,
    DataInspection,
    OutputDebug,
}

struct ViewSpec {
    class_identifier: ViewClassIdentifier,
    display_name: &'static str,
}

struct PresetViews {
    network: ViewId,
    parameters: ViewId,
    info: ViewId,
    display: ViewId,
    operators: ViewId,
    find: ViewId,
    layers: ViewId,
    data: ViewId,
    outputs: ViewId,
    project: ViewId,
    graph: ViewId,
}

impl HoudiniWorkbenchPreset {
    fn label(self) -> &'static str {
        match self {
            Self::NetworkAndInspector => "Network + Inspector",
            Self::HoudiniDefault => "Houdini Default",
            Self::GraphReview => "Graph Review",
            Self::DataInspection => "Data Inspection",
            Self::OutputDebug => "Output / Debug",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::NetworkAndInspector => {
                "Network editor beside Parameters, Display, Info, Ops, Find, and Layers tabs."
            }
            Self::HoudiniDefault => {
                "Rendered graph viewport on the left, with Parameters and Network stacked on the right."
            }
            Self::GraphReview => {
                "Output viewport beside inspection tabs, with project data and exports nearby."
            }
            Self::DataInspection => {
                "Rendered graph viewport beside project data, attributes, info, and outputs."
            }
            Self::OutputDebug => {
                "Rendered graph viewport beside output, display, layers, info, find, and ops tabs."
            }
        }
    }
}

pub(crate) fn houdini_workbench_toolbar_ui(
    ui: &mut egui::Ui,
    ctx: &re_viewer_context::ViewerContext<'_>,
    viewport_blueprint: &ViewportBlueprint,
) {
    egui::Frame::new()
        .fill(ui.visuals().panel_fill)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong("Workbench");

                ui.menu_button("Layouts", |ui| {
                    for preset in [
                        HoudiniWorkbenchPreset::NetworkAndInspector,
                        HoudiniWorkbenchPreset::HoudiniDefault,
                        HoudiniWorkbenchPreset::GraphReview,
                        HoudiniWorkbenchPreset::DataInspection,
                        HoudiniWorkbenchPreset::OutputDebug,
                    ] {
                        if ui
                            .button(preset.label())
                            .on_hover_text(preset.description())
                            .clicked()
                        {
                            apply_houdini_workbench_preset(ctx, viewport_blueprint, preset);
                            ui.close();
                        }
                    }
                });

                if ui
                    .small_button("Open saved layout...")
                    .on_hover_text(
                        "Open a saved Rerun blueprint file (.rbl) as a workbench layout.",
                    )
                    .clicked()
                {
                    ctx.command_sender().send_ui(re_ui::UICommand::Open);
                }

                if ui
                    .small_button("Save workbench as...")
                    .on_hover_text(
                        "Duplicate the current workbench layout to a named Rerun blueprint file (.rbl).",
                    )
                    .clicked()
                {
                    ctx.command_sender()
                        .send_ui(re_ui::UICommand::SaveBlueprint);
                }
            });
        });
}

fn apply_houdini_workbench_preset(
    ctx: &re_viewer_context::ViewerContext<'_>,
    viewport_blueprint: &ViewportBlueprint,
    preset: HoudiniWorkbenchPreset,
) {
    viewport_blueprint.set_auto_layout(false, ctx);
    viewport_blueprint.set_auto_views(false, ctx);
    viewport_blueprint.set_maximized(None, ctx);

    let (views, views_to_add) = resolve_preset_views(viewport_blueprint);
    viewport_blueprint.add_views(views_to_add.into_iter(), None, None);

    let (tree, container_display_names) = build_preset_tree(preset, views);
    viewport_blueprint.set_tree_with_container_names(tree, container_display_names);
}

fn resolve_preset_views(
    viewport_blueprint: &ViewportBlueprint,
) -> (PresetViews, Vec<ViewBlueprint>) {
    let mut views_to_add = Vec::new();

    let network = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniNetworkView>("Network"),
        &mut views_to_add,
    );
    let parameters = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniParametersView>("Parameters"),
        &mut views_to_add,
    );
    let info = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniInfoView>("Info"),
        &mut views_to_add,
    );
    let display = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniDisplayView>("Display"),
        &mut views_to_add,
    );
    let operators = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniOperatorsView>("Operators"),
        &mut views_to_add,
    );
    let find = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniFindView>("Find"),
        &mut views_to_add,
    );
    let layers = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniLayersView>("Layers"),
        &mut views_to_add,
    );
    let data = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniDataView>("Data"),
        &mut views_to_add,
    );
    let outputs = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniOutputsView>("Outputs"),
        &mut views_to_add,
    );
    let project = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniProjectView>("Project"),
        &mut views_to_add,
    );
    let graph = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniGraphView>("Houdini Graph"),
        &mut views_to_add,
    );

    (
        PresetViews {
            network,
            parameters,
            info,
            display,
            operators,
            find,
            layers,
            data,
            outputs,
            project,
            graph,
        },
        views_to_add,
    )
}

fn resolve_view(
    viewport_blueprint: &ViewportBlueprint,
    spec: ViewSpec,
    views_to_add: &mut Vec<ViewBlueprint>,
) -> ViewId {
    if let Some(view) = viewport_blueprint
        .views
        .values()
        .find(|view| view.class_identifier() == spec.class_identifier)
    {
        return view.id;
    }

    let mut view = ViewBlueprint::new_with_root_wildcard(spec.class_identifier);
    view.display_name = Some(spec.display_name.to_owned());
    let view_id = view.id;
    views_to_add.push(view);
    view_id
}

fn view_spec<T: re_viewer_context::ViewClass>(display_name: &'static str) -> ViewSpec {
    ViewSpec {
        class_identifier: T::identifier(),
        display_name,
    }
}

fn build_preset_tree(
    preset: HoudiniWorkbenchPreset,
    views: PresetViews,
) -> (egui_tiles::Tree<ViewId>, Vec<(egui_tiles::TileId, String)>) {
    let mut tiles = egui_tiles::Tiles::default();
    let mut container_display_names = Vec::new();

    let root = match preset {
        HoudiniWorkbenchPreset::NetworkAndInspector => {
            let network = tiles.insert_pane(views.network);
            let inspector = insert_named_tabs(
                &mut tiles,
                &mut container_display_names,
                "Inspector",
                vec![
                    views.parameters,
                    views.display,
                    views.info,
                    views.operators,
                    views.find,
                    views.layers,
                ],
            );
            insert_named_horizontal(
                &mut tiles,
                &mut container_display_names,
                "Network Workbench",
                vec![network, inspector],
            )
        }
        HoudiniWorkbenchPreset::HoudiniDefault => {
            let graph = tiles.insert_pane(views.graph);
            let parameters = tiles.insert_pane(views.parameters);
            let network = tiles.insert_pane(views.network);
            let right_side = insert_named_vertical(
                &mut tiles,
                &mut container_display_names,
                "Parameters + Network",
                vec![parameters, network],
            );
            insert_named_horizontal(
                &mut tiles,
                &mut container_display_names,
                "Houdini Default Workbench",
                vec![graph, right_side],
            )
        }
        HoudiniWorkbenchPreset::GraphReview => {
            let graph = tiles.insert_pane(views.graph);
            let data_tabs = insert_named_tabs(
                &mut tiles,
                &mut container_display_names,
                "Project Data",
                vec![views.data, views.outputs, views.project],
            );
            let review_tabs = insert_named_tabs(
                &mut tiles,
                &mut container_display_names,
                "Review",
                vec![views.info, views.display, views.layers, views.parameters],
            );
            let side = insert_named_vertical(
                &mut tiles,
                &mut container_display_names,
                "Review Controls",
                vec![review_tabs, data_tabs],
            );
            insert_named_horizontal(
                &mut tiles,
                &mut container_display_names,
                "Graph Review Workbench",
                vec![graph, side],
            )
        }
        HoudiniWorkbenchPreset::DataInspection => {
            let graph = tiles.insert_pane(views.graph);
            let data_tabs = insert_named_tabs(
                &mut tiles,
                &mut container_display_names,
                "Data + Attributes",
                vec![views.data, views.project, views.info],
            );
            let outputs = tiles.insert_pane(views.outputs);
            let side = insert_named_vertical(
                &mut tiles,
                &mut container_display_names,
                "Data Review",
                vec![data_tabs, outputs],
            );
            insert_named_horizontal(
                &mut tiles,
                &mut container_display_names,
                "Data Inspection Workbench",
                vec![graph, side],
            )
        }
        HoudiniWorkbenchPreset::OutputDebug => {
            let graph = tiles.insert_pane(views.graph);
            let output_tabs = insert_named_tabs(
                &mut tiles,
                &mut container_display_names,
                "Output",
                vec![views.outputs, views.display, views.layers],
            );
            let debug_tabs = insert_named_tabs(
                &mut tiles,
                &mut container_display_names,
                "Debug",
                vec![views.info, views.find, views.operators],
            );
            let side = insert_named_vertical(
                &mut tiles,
                &mut container_display_names,
                "Output + Debug Controls",
                vec![output_tabs, debug_tabs],
            );
            insert_named_horizontal(
                &mut tiles,
                &mut container_display_names,
                "Output Debug Workbench",
                vec![graph, side],
            )
        }
    };

    (
        egui_tiles::Tree::new("viewport_tree", root, tiles),
        container_display_names,
    )
}

fn insert_named_tabs(
    tiles: &mut egui_tiles::Tiles<ViewId>,
    container_display_names: &mut Vec<(egui_tiles::TileId, String)>,
    name: &str,
    views: Vec<ViewId>,
) -> egui_tiles::TileId {
    let children = views
        .into_iter()
        .map(|view| tiles.insert_pane(view))
        .collect();
    let tile_id = tiles.insert_tab_tile(children);
    container_display_names.push((tile_id, name.to_owned()));
    tile_id
}

fn insert_named_horizontal(
    tiles: &mut egui_tiles::Tiles<ViewId>,
    container_display_names: &mut Vec<(egui_tiles::TileId, String)>,
    name: &str,
    children: Vec<egui_tiles::TileId>,
) -> egui_tiles::TileId {
    let tile_id = tiles.insert_horizontal_tile(children);
    container_display_names.push((tile_id, name.to_owned()));
    tile_id
}

fn insert_named_vertical(
    tiles: &mut egui_tiles::Tiles<ViewId>,
    container_display_names: &mut Vec<(egui_tiles::TileId, String)>,
    name: &str,
    children: Vec<egui_tiles::TileId>,
) -> egui_tiles::TileId {
    let tile_id = tiles.insert_vertical_tile(children);
    container_display_names.push((tile_id, name.to_owned()));
    tile_id
}

#[cfg(test)]
mod tests {
    use egui_tiles::{ContainerKind, Tile};
    use re_viewer_context::ViewId;

    use super::{HoudiniWorkbenchPreset, PresetViews, build_preset_tree};

    fn view_id(byte: u8) -> ViewId {
        ViewId::from_bytes([byte; 16])
    }

    fn preset_views() -> PresetViews {
        PresetViews {
            network: view_id(1),
            parameters: view_id(2),
            info: view_id(3),
            display: view_id(4),
            operators: view_id(5),
            find: view_id(6),
            layers: view_id(7),
            data: view_id(8),
            outputs: view_id(9),
            project: view_id(10),
            graph: view_id(11),
        }
    }

    #[test]
    fn network_workbench_preset_uses_named_native_containers() {
        let (tree, names) =
            build_preset_tree(HoudiniWorkbenchPreset::NetworkAndInspector, preset_views());

        assert!(tree.root().is_some());
        assert_eq!(
            tree.tiles.iter().filter(|(_, tile)| tile.is_pane()).count(),
            7
        );
        assert!(names.iter().any(|(_, name)| name == "Network Workbench"));
        assert!(names.iter().any(|(_, name)| name == "Inspector"));
    }

    #[test]
    fn graph_review_preset_keeps_review_and_data_tabs_named() {
        let (tree, names) = build_preset_tree(HoudiniWorkbenchPreset::GraphReview, preset_views());

        assert!(tree.root().is_some());
        assert_eq!(
            tree.tiles.iter().filter(|(_, tile)| tile.is_pane()).count(),
            8
        );
        assert!(
            names
                .iter()
                .any(|(_, name)| name == "Graph Review Workbench")
        );
        assert!(names.iter().any(|(_, name)| name == "Review"));
        assert!(names.iter().any(|(_, name)| name == "Project Data"));
    }

    #[test]
    fn houdini_default_preset_places_graph_viewport_beside_params_and_network() {
        let (tree, names) =
            build_preset_tree(HoudiniWorkbenchPreset::HoudiniDefault, preset_views());

        assert!(tree.root().is_some());
        assert_eq!(
            tree.tiles.iter().filter(|(_, tile)| tile.is_pane()).count(),
            3
        );
        assert!(
            names
                .iter()
                .any(|(_, name)| name == "Houdini Default Workbench")
        );
        assert!(names.iter().any(|(_, name)| name == "Parameters + Network"));

        let root_id = tree.root().expect("preset should have a root tile");
        let root_container = tree
            .tiles
            .get_container(root_id)
            .expect("root tile should be a container");
        assert_eq!(root_container.kind(), ContainerKind::Horizontal);

        let root_children = root_container.children_vec();
        assert_eq!(root_children.len(), 2);
        assert_eq!(
            tree.tiles.get(root_children[0]),
            Some(&Tile::Pane(preset_views().graph))
        );
    }

    #[test]
    fn data_inspection_preset_groups_project_data_with_outputs() {
        let (tree, names) =
            build_preset_tree(HoudiniWorkbenchPreset::DataInspection, preset_views());

        assert!(tree.root().is_some());
        assert_eq!(
            tree.tiles.iter().filter(|(_, tile)| tile.is_pane()).count(),
            5
        );
        assert!(
            names
                .iter()
                .any(|(_, name)| name == "Data Inspection Workbench")
        );
        assert!(names.iter().any(|(_, name)| name == "Data + Attributes"));
        assert!(names.iter().any(|(_, name)| name == "Data Review"));
    }

    #[test]
    fn output_debug_preset_groups_output_and_diagnostics() {
        let (tree, names) = build_preset_tree(HoudiniWorkbenchPreset::OutputDebug, preset_views());

        assert!(tree.root().is_some());
        assert_eq!(
            tree.tiles.iter().filter(|(_, tile)| tile.is_pane()).count(),
            7
        );
        assert!(
            names
                .iter()
                .any(|(_, name)| name == "Output Debug Workbench")
        );
        assert!(names.iter().any(|(_, name)| name == "Output"));
        assert!(names.iter().any(|(_, name)| name == "Debug"));
    }
}
