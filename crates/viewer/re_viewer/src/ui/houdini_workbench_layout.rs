use re_sdk_types::ViewClassIdentifier;
use re_ui::UICommandSender as _;
use re_viewer_context::{SystemCommand, SystemCommandSender as _, ViewId};
use re_viewport_blueprint::{ViewBlueprint, ViewportBlueprint};

use crate::ui::{
    HoudiniAssetsView, HoudiniDataView, HoudiniDisplayView, HoudiniExecutionView, HoudiniFindView,
    HoudiniGalleryView, HoudiniGraphView, HoudiniInfoView, HoudiniLayersView, HoudiniNetworkView,
    HoudiniOperatorsView, HoudiniOutputsView, HoudiniParametersView, HoudiniProjectView,
    HoudiniShelfView,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
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
    assets: ViewId,
    gallery: ViewId,
    data: ViewId,
    outputs: ViewId,
    shelf: ViewId,
    execution: ViewId,
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
                "Network editor beside Shelf, Parameters, Display, Info, Ops, Find, and Layers tabs."
            }
            Self::HoudiniDefault => {
                "Rendered graph viewport on the left, with Shelf, Parameters, and Network stacked on the right."
            }
            Self::GraphReview => {
                "Output viewport beside inspection tabs, with Shelf, project data, and exports nearby."
            }
            Self::DataInspection => {
                "Rendered graph viewport beside Shelf, project data, attributes, info, and outputs."
            }
            Self::OutputDebug => {
                "Rendered graph viewport beside Shelf, output, display, layers, info, find, and ops tabs."
            }
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::NetworkAndInspector => "network-and-inspector",
            Self::HoudiniDefault => "houdini-default",
            Self::GraphReview => "graph-review",
            Self::DataInspection => "data-inspection",
            Self::OutputDebug => "output-debug",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkbenchCatalog {
    bundled: Vec<WorkbenchLayoutEntry>,
    personal: Vec<WorkbenchLayoutEntry>,
    project: Vec<WorkbenchLayoutEntry>,
}

impl WorkbenchCatalog {
    fn load() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(root) = workbench_metadata_root() {
            return Self::load_from_root(&root);
        }

        Self {
            bundled: bundled_workbench_entries(),
            personal: Vec::new(),
            project: Vec::new(),
        }
    }

    fn load_from_root(root: &std::path::Path) -> Self {
        Self {
            bundled: bundled_workbench_entries(),
            personal: load_saved_workbench_entries(root, WorkbenchLayoutScope::Personal),
            project: load_saved_workbench_entries(root, WorkbenchLayoutScope::Project),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
struct WorkbenchBrowserDraft {
    display_name: String,
    scope: WorkbenchLayoutScope,
    default_for_user: bool,
    default_for_project: bool,
}

impl Default for WorkbenchBrowserDraft {
    fn default() -> Self {
        Self {
            display_name: "My Workbench".to_owned(),
            scope: WorkbenchLayoutScope::Personal,
            default_for_user: false,
            default_for_project: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
struct WorkbenchLayoutEntry {
    id: String,
    display_name: String,
    scope: WorkbenchLayoutScope,
    source: WorkbenchLayoutSource,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    blueprint_path: Option<std::path::PathBuf>,
    #[serde(default)]
    default_for_project: bool,
    #[serde(default)]
    default_for_user: bool,
    #[serde(default = "workbench_compatibility_version")]
    compatibility_version: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum WorkbenchLayoutScope {
    Bundled,
    Personal,
    Project,
}

impl WorkbenchLayoutScope {
    fn directory_name(self) -> &'static str {
        match self {
            Self::Bundled => "bundled",
            Self::Personal => "personal",
            Self::Project => "project",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum WorkbenchLayoutSource {
    BundledPreset(HoudiniWorkbenchPreset),
    SavedBlueprint,
}

fn bundled_workbench_entries() -> Vec<WorkbenchLayoutEntry> {
    [
        HoudiniWorkbenchPreset::NetworkAndInspector,
        HoudiniWorkbenchPreset::HoudiniDefault,
        HoudiniWorkbenchPreset::GraphReview,
        HoudiniWorkbenchPreset::DataInspection,
        HoudiniWorkbenchPreset::OutputDebug,
    ]
    .into_iter()
    .map(|preset| WorkbenchLayoutEntry {
        id: format!("bundled:{}", preset.slug()),
        display_name: preset.label().to_owned(),
        scope: WorkbenchLayoutScope::Bundled,
        source: WorkbenchLayoutSource::BundledPreset(preset),
        description: Some(preset.description().to_owned()),
        blueprint_path: None,
        default_for_project: preset == HoudiniWorkbenchPreset::HoudiniDefault,
        default_for_user: false,
        compatibility_version: workbench_compatibility_version(),
    })
    .collect()
}

fn load_saved_workbench_entries(
    root: &std::path::Path,
    scope: WorkbenchLayoutScope,
) -> Vec<WorkbenchLayoutEntry> {
    let directory = root.join(scope.directory_name());
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut layouts = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension().and_then(|extension| extension.to_str()) == Some("json"))
                .then_some(path)
        })
        .filter_map(|path| {
            std::fs::read_to_string(path)
                .ok()
                .and_then(|json| serde_json::from_str::<WorkbenchLayoutEntry>(&json).ok())
        })
        .filter(|entry| entry.scope == scope)
        .collect::<Vec<_>>();
    layouts.sort_by(|left, right| left.display_name.cmp(&right.display_name));
    layouts
}

#[cfg(not(target_arch = "wasm32"))]
fn register_workbench_duplicate(
    scope: WorkbenchLayoutScope,
    source_entry: &WorkbenchLayoutEntry,
    preset: HoudiniWorkbenchPreset,
    display_name: &str,
    default_for_user: bool,
    default_for_project: bool,
) -> anyhow::Result<WorkbenchLayoutEntry> {
    register_workbench_copy(
        scope,
        display_name,
        Some(format!("Copy of {}", source_entry.display_name)),
        default_for_user,
        default_for_project,
        |entry| Some(format!("{} copy", preset.label())).filter(|_| entry.description.is_none()),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn register_saved_workbench_copy(
    source_entry: &WorkbenchLayoutEntry,
    draft: &WorkbenchBrowserDraft,
) -> anyhow::Result<WorkbenchLayoutEntry> {
    let registered = register_workbench_copy(
        draft.scope,
        &draft.display_name,
        Some(format!("Copy of {}", source_entry.display_name)),
        draft.default_for_user,
        draft.default_for_project,
        |_| None,
    )?;

    if let (Some(source_path), Some(destination_path)) =
        (&source_entry.blueprint_path, &registered.blueprint_path)
        && source_path.exists()
    {
        std::fs::copy(source_path, destination_path)?;
    }

    Ok(registered)
}

#[cfg(not(target_arch = "wasm32"))]
fn register_current_workbench(
    draft: &WorkbenchBrowserDraft,
) -> anyhow::Result<WorkbenchLayoutEntry> {
    register_workbench_copy(
        draft.scope,
        &draft.display_name,
        Some("Saved from the current edited workbench layout.".to_owned()),
        draft.default_for_user,
        draft.default_for_project,
        |_| None,
    )
}

#[cfg(target_arch = "wasm32")]
fn register_current_workbench(
    _draft: &WorkbenchBrowserDraft,
) -> anyhow::Result<WorkbenchLayoutEntry> {
    anyhow::bail!("workbench metadata is only available in the native viewer")
}

#[cfg(target_arch = "wasm32")]
fn register_saved_workbench_copy(
    _source_entry: &WorkbenchLayoutEntry,
    _draft: &WorkbenchBrowserDraft,
) -> anyhow::Result<WorkbenchLayoutEntry> {
    anyhow::bail!("workbench metadata is only available in the native viewer")
}

#[cfg(not(target_arch = "wasm32"))]
fn register_workbench_copy(
    scope: WorkbenchLayoutScope,
    display_name: &str,
    description: Option<String>,
    default_for_user: bool,
    default_for_project: bool,
    fallback_description: impl FnOnce(&WorkbenchLayoutEntry) -> Option<String>,
) -> anyhow::Result<WorkbenchLayoutEntry> {
    let root = workbench_metadata_root()
        .ok_or_else(|| anyhow::anyhow!("could not locate Rerun storage directory"))?;
    let directory = root.join(scope.directory_name());
    std::fs::create_dir_all(&directory)?;

    let display_name = sanitized_workbench_display_name(display_name);
    let slug = sanitize_workbench_id(&display_name);
    let blueprint_path = directory.join(format!("{slug}.rbl"));
    let entry = WorkbenchLayoutEntry {
        id: format!("{}:{slug}", scope.directory_name()),
        display_name: display_name.clone(),
        scope,
        source: WorkbenchLayoutSource::SavedBlueprint,
        description,
        blueprint_path: Some(blueprint_path),
        default_for_project,
        default_for_user,
        compatibility_version: workbench_compatibility_version(),
    };
    let mut entry = entry;
    if entry.description.is_none() {
        entry.description = fallback_description(&entry);
    }
    std::fs::write(
        directory.join(format!("{slug}.json")),
        serde_json::to_string_pretty(&entry)?,
    )?;
    if entry.default_for_user {
        set_workbench_default(&root, scope, &entry.id, WorkbenchDefaultKind::Personal)?;
    }
    if entry.default_for_project {
        set_workbench_default(&root, scope, &entry.id, WorkbenchDefaultKind::Project)?;
    }
    Ok(entry)
}

#[cfg(target_arch = "wasm32")]
fn register_workbench_duplicate(
    _scope: WorkbenchLayoutScope,
    _source_entry: &WorkbenchLayoutEntry,
    _preset: HoudiniWorkbenchPreset,
    _display_name: &str,
    _default_for_user: bool,
    _default_for_project: bool,
) -> anyhow::Result<WorkbenchLayoutEntry> {
    anyhow::bail!("workbench metadata is only available in the native viewer")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkbenchDefaultKind {
    Personal,
    Project,
}

#[cfg(not(target_arch = "wasm32"))]
fn set_workbench_default(
    root: &std::path::Path,
    scope: WorkbenchLayoutScope,
    target_id: &str,
    kind: WorkbenchDefaultKind,
) -> anyhow::Result<()> {
    let directory = root.join(scope.directory_name());
    let Ok(entries) = std::fs::read_dir(&directory) else {
        return Ok(());
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let Ok(json) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(mut layout) = serde_json::from_str::<WorkbenchLayoutEntry>(&json) else {
            continue;
        };
        if layout.scope != scope {
            continue;
        }
        match kind {
            WorkbenchDefaultKind::Personal => layout.default_for_user = layout.id == target_id,
            WorkbenchDefaultKind::Project => layout.default_for_project = layout.id == target_id,
        }
        std::fs::write(path, serde_json::to_string_pretty(&layout)?)?;
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn set_saved_workbench_default(
    entry: &WorkbenchLayoutEntry,
    kind: WorkbenchDefaultKind,
) -> anyhow::Result<()> {
    let root = workbench_metadata_root()
        .ok_or_else(|| anyhow::anyhow!("could not locate Rerun storage directory"))?;
    set_workbench_default(&root, entry.scope, &entry.id, kind)
}

#[cfg(target_arch = "wasm32")]
fn set_saved_workbench_default(
    _entry: &WorkbenchLayoutEntry,
    _kind: WorkbenchDefaultKind,
) -> anyhow::Result<()> {
    anyhow::bail!("workbench metadata is only available in the native viewer")
}

#[cfg(not(target_arch = "wasm32"))]
fn workbench_metadata_root() -> Option<std::path::PathBuf> {
    eframe::storage_dir(crate::native::APP_ID).map(|storage_dir| storage_dir.join("workbenches"))
}

fn workbench_compatibility_version() -> String {
    "re_viewer-houdini-workbench-0.1".to_owned()
}

fn sanitize_workbench_id(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if sanitized.is_empty() {
        "workbench".to_owned()
    } else {
        sanitized
    }
}

fn sanitized_workbench_display_name(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        "My Workbench".to_owned()
    } else {
        value.to_owned()
    }
}

pub(crate) fn houdini_workbench_toolbar_ui(
    ui: &mut egui::Ui,
    ctx: &re_viewer_context::ViewerContext<'_>,
    viewport_blueprint: &ViewportBlueprint,
) {
    let workbenches = WorkbenchCatalog::load();

    egui::Frame::new()
        .fill(ui.visuals().panel_fill)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong("Workbench");

                ui.menu_button("Layouts", |ui| workbench_browser_ui(ui, ctx, viewport_blueprint, &workbenches));

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

fn workbench_browser_ui(
    ui: &mut egui::Ui,
    ctx: &re_viewer_context::ViewerContext<'_>,
    viewport_blueprint: &ViewportBlueprint,
    catalog: &WorkbenchCatalog,
) {
    let draft_id = ui.make_persistent_id("houdini_workbench_browser_draft");
    let mut draft = ui.data_mut(|data| {
        data.get_persisted_mut_or_insert_with(draft_id, WorkbenchBrowserDraft::default)
            .clone()
    });

    ui.horizontal(|ui| {
        ui.weak("Name");
        ui.add(egui::TextEdit::singleline(&mut draft.display_name).desired_width(180.0));
    });
    ui.horizontal_wrapped(|ui| {
        ui.selectable_value(&mut draft.scope, WorkbenchLayoutScope::Personal, "Personal");
        ui.selectable_value(&mut draft.scope, WorkbenchLayoutScope::Project, "Project");
        ui.checkbox(&mut draft.default_for_user, "Personal default");
        ui.checkbox(&mut draft.default_for_project, "Project default");
        if ui
            .small_button("Save Current")
            .on_hover_text("Save the current edited blueprint as the named workbench.")
            .clicked()
        {
            if let Ok(registered) = register_current_workbench(&draft)
                && let Some(path) = registered.blueprint_path
            {
                save_active_workbench_blueprint(ctx, path);
            }
            ui.close();
        }
    });
    ui.data_mut(|data| data.insert_persisted(draft_id, draft.clone()));
    ui.separator();

    ui.strong("Bundled");
    for entry in &catalog.bundled {
        ui.horizontal(|ui| {
            if ui
                .button(entry.display_name.as_str())
                .on_hover_text(entry.description.as_deref().unwrap_or_default())
                .clicked()
            {
                if let WorkbenchLayoutSource::BundledPreset(preset) = entry.source {
                    apply_houdini_workbench_preset(ctx, viewport_blueprint, preset);
                }
                ui.close();
            }

            if ui
                .small_button("Duplicate")
                .on_hover_text(
                    "Register a personal copy and save the current blueprint to its .rbl payload.",
                )
                .clicked()
            {
                if let WorkbenchLayoutSource::BundledPreset(preset) = entry.source {
                    if let Ok(registered) = register_workbench_duplicate(
                        draft.scope,
                        entry,
                        preset,
                        &draft.display_name,
                        draft.default_for_user,
                        draft.default_for_project,
                    ) {
                        apply_houdini_workbench_preset(ctx, viewport_blueprint, preset);
                        if let Some(path) = registered.blueprint_path {
                            save_active_workbench_blueprint(ctx, path);
                        }
                    }
                }
                ui.close();
            }
        });
    }

    ui.separator();
    saved_workbench_section_ui(ui, ctx, "Personal", &catalog.personal, &draft);
    ui.separator();
    saved_workbench_section_ui(ui, ctx, "Project", &catalog.project, &draft);
}

fn saved_workbench_section_ui(
    ui: &mut egui::Ui,
    ctx: &re_viewer_context::ViewerContext<'_>,
    heading: &str,
    entries: &[WorkbenchLayoutEntry],
    draft: &WorkbenchBrowserDraft,
) {
    ui.strong(heading);
    if entries.is_empty() {
        ui.weak("No saved layouts");
        return;
    }

    for entry in entries {
        ui.horizontal(|ui| {
            let load_enabled = entry
                .blueprint_path
                .as_ref()
                .is_some_and(|path| path.exists());
            if ui
                .add_enabled(load_enabled, egui::Button::new(entry.display_name.as_str()))
                .on_hover_text(
                    entry
                        .blueprint_path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "No .rbl payload path registered".to_owned()),
                )
                .clicked()
            {
                if let Some(path) = &entry.blueprint_path {
                    open_workbench_blueprint(ctx, path.clone());
                }
                ui.close();
            }
            if entry.default_for_project {
                ui.weak("project default");
            }
            if entry.default_for_user {
                ui.weak("personal default");
            }
            if !entry.default_for_user
                && ui
                    .small_button("Use personal default")
                    .on_hover_text("Make this saved workbench the personal default.")
                    .clicked()
            {
                let _ = set_saved_workbench_default(entry, WorkbenchDefaultKind::Personal);
                ui.close();
            }
            if !entry.default_for_project
                && ui
                    .small_button("Use project default")
                    .on_hover_text("Make this saved workbench the project default.")
                    .clicked()
            {
                let _ = set_saved_workbench_default(entry, WorkbenchDefaultKind::Project);
                ui.close();
            }
            if entry.scope == WorkbenchLayoutScope::Project
                && ui
                    .small_button("Duplicate")
                    .on_hover_text(
                        "Copy this project workbench metadata into a personal or project layout.",
                    )
                    .clicked()
            {
                if let Ok(registered) = register_saved_workbench_copy(entry, draft) {
                    if let Some(path) = registered.blueprint_path {
                        open_workbench_blueprint(ctx, path);
                    }
                }
                ui.close();
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn open_workbench_blueprint(ctx: &re_viewer_context::ViewerContext<'_>, path: std::path::PathBuf) {
    use re_data_source::LogDataSource;
    use re_log_types::FileSource;

    ctx.command_sender()
        .send_system(SystemCommand::LoadDataSource(LogDataSource::FilePath {
            file_source: FileSource::FileDialog {
                recommended_store_id: None,
                force_store_info: false,
            },
            path,
            follow: false,
        }));
}

#[cfg(target_arch = "wasm32")]
fn open_workbench_blueprint(
    _ctx: &re_viewer_context::ViewerContext<'_>,
    _path: std::path::PathBuf,
) {
}

#[cfg(not(target_arch = "wasm32"))]
fn save_active_workbench_blueprint(
    ctx: &re_viewer_context::ViewerContext<'_>,
    path: std::path::PathBuf,
) {
    ctx.command_sender()
        .send_system(SystemCommand::SaveActiveBlueprintToPath(path));
}

#[cfg(target_arch = "wasm32")]
fn save_active_workbench_blueprint(
    _ctx: &re_viewer_context::ViewerContext<'_>,
    _path: std::path::PathBuf,
) {
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
    let assets = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniAssetsView>("Assets"),
        &mut views_to_add,
    );
    let gallery = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniGalleryView>("Gallery"),
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
    let shelf = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniShelfView>("Shelf"),
        &mut views_to_add,
    );
    let execution = resolve_view(
        viewport_blueprint,
        view_spec::<HoudiniExecutionView>("Execution"),
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
            assets,
            gallery,
            data,
            outputs,
            shelf,
            execution,
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
                    views.shelf,
                    views.parameters,
                    views.display,
                    views.info,
                    views.execution,
                    views.operators,
                    views.find,
                    views.assets,
                    views.gallery,
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
            let shelf = tiles.insert_pane(views.shelf);
            let gallery = tiles.insert_pane(views.gallery);
            let assets = tiles.insert_pane(views.assets);
            let parameters = tiles.insert_pane(views.parameters);
            let network = tiles.insert_pane(views.network);
            let right_side = insert_named_vertical(
                &mut tiles,
                &mut container_display_names,
                "Shelf + Gallery + Assets + Parameters + Network",
                vec![shelf, gallery, assets, parameters, network],
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
                vec![
                    views.shelf,
                    views.data,
                    views.outputs,
                    views.execution,
                    views.gallery,
                    views.assets,
                    views.project,
                ],
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
                vec![
                    views.shelf,
                    views.data,
                    views.execution,
                    views.gallery,
                    views.assets,
                    views.project,
                    views.info,
                ],
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
                vec![
                    views.shelf,
                    views.outputs,
                    views.execution,
                    views.gallery,
                    views.assets,
                    views.display,
                    views.layers,
                ],
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

    use super::{
        HoudiniWorkbenchPreset, PresetViews, WorkbenchCatalog, WorkbenchDefaultKind,
        WorkbenchLayoutEntry, WorkbenchLayoutScope, WorkbenchLayoutSource, build_preset_tree,
        sanitize_workbench_id, sanitized_workbench_display_name, set_workbench_default,
        workbench_compatibility_version,
    };

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
            assets: view_id(8),
            gallery: view_id(9),
            data: view_id(10),
            outputs: view_id(11),
            shelf: view_id(12),
            execution: view_id(13),
            project: view_id(14),
            graph: view_id(15),
        }
    }

    fn tree_contains_pane(tree: &egui_tiles::Tree<ViewId>, view_id: ViewId) -> bool {
        tree.tiles
            .iter()
            .any(|(_, tile)| tile == &Tile::Pane(view_id))
    }

    #[test]
    fn network_workbench_preset_uses_named_native_containers() {
        let (tree, names) =
            build_preset_tree(HoudiniWorkbenchPreset::NetworkAndInspector, preset_views());

        assert!(tree.root().is_some());
        assert_eq!(
            tree.tiles.iter().filter(|(_, tile)| tile.is_pane()).count(),
            11
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
            12
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
            6
        );
        assert!(
            names
                .iter()
                .any(|(_, name)| name == "Houdini Default Workbench")
        );
        assert!(
            names
                .iter()
                .any(|(_, name)| name == "Shelf + Gallery + Assets + Parameters + Network")
        );

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
        assert!(tree_contains_pane(&tree, preset_views().gallery));
    }

    #[test]
    fn data_inspection_preset_groups_project_data_with_outputs() {
        let (tree, names) =
            build_preset_tree(HoudiniWorkbenchPreset::DataInspection, preset_views());

        assert!(tree.root().is_some());
        assert_eq!(
            tree.tiles.iter().filter(|(_, tile)| tile.is_pane()).count(),
            9
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
            11
        );
        assert!(
            names
                .iter()
                .any(|(_, name)| name == "Output Debug Workbench")
        );
        assert!(names.iter().any(|(_, name)| name == "Output"));
        assert!(names.iter().any(|(_, name)| name == "Debug"));
    }

    #[test]
    fn workbench_catalog_lists_bundled_presets_with_defaults() {
        let catalog = WorkbenchCatalog::load_from_root(std::path::Path::new("/missing"));

        assert_eq!(catalog.bundled.len(), 5);
        assert!(
            catalog
                .bundled
                .iter()
                .any(|entry| entry.display_name == "Houdini Default" && entry.default_for_project)
        );
        assert!(catalog.personal.is_empty());
        assert!(catalog.project.is_empty());
    }

    #[test]
    fn workbench_catalog_loads_personal_and_project_metadata_by_name() {
        let root = tempfile::tempdir().unwrap();
        let personal_dir = root.path().join("personal");
        let project_dir = root.path().join("project");
        std::fs::create_dir_all(&personal_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();

        let personal = saved_entry(
            "personal:analysis",
            "Analysis",
            WorkbenchLayoutScope::Personal,
            true,
            false,
        );
        let project = saved_entry(
            "project:team-default",
            "Team Default",
            WorkbenchLayoutScope::Project,
            false,
            true,
        );
        std::fs::write(
            personal_dir.join("analysis.json"),
            serde_json::to_string_pretty(&personal).unwrap(),
        )
        .unwrap();
        std::fs::write(
            project_dir.join("team-default.json"),
            serde_json::to_string_pretty(&project).unwrap(),
        )
        .unwrap();

        let catalog = WorkbenchCatalog::load_from_root(root.path());

        assert_eq!(catalog.personal, vec![personal]);
        assert_eq!(catalog.project, vec![project]);
    }

    #[test]
    fn workbench_metadata_ids_are_filename_safe() {
        assert_eq!(
            sanitize_workbench_id("Copy of Houdini Default!"),
            "copy-of-houdini-default"
        );
        assert_eq!(sanitize_workbench_id("///"), "workbench");
        assert_eq!(sanitized_workbench_display_name("  "), "My Workbench");
        assert_eq!(sanitized_workbench_display_name("  Review  "), "Review");
    }

    #[test]
    fn workbench_default_selection_updates_only_matching_scope() {
        let root = tempfile::tempdir().unwrap();
        let personal_dir = root.path().join("personal");
        let project_dir = root.path().join("project");
        std::fs::create_dir_all(&personal_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();

        let first = saved_entry(
            "personal:first",
            "First",
            WorkbenchLayoutScope::Personal,
            true,
            false,
        );
        let second = saved_entry(
            "personal:second",
            "Second",
            WorkbenchLayoutScope::Personal,
            false,
            false,
        );
        let project = saved_entry(
            "project:shared",
            "Shared",
            WorkbenchLayoutScope::Project,
            false,
            true,
        );
        write_entry(&personal_dir, "first", &first);
        write_entry(&personal_dir, "second", &second);
        write_entry(&project_dir, "shared", &project);

        set_workbench_default(
            root.path(),
            WorkbenchLayoutScope::Personal,
            "personal:second",
            WorkbenchDefaultKind::Personal,
        )
        .unwrap();

        let catalog = WorkbenchCatalog::load_from_root(root.path());
        assert!(
            catalog
                .personal
                .iter()
                .any(|entry| entry.id == "personal:first" && !entry.default_for_user)
        );
        assert!(
            catalog
                .personal
                .iter()
                .any(|entry| entry.id == "personal:second" && entry.default_for_user)
        );
        assert_eq!(catalog.project, vec![project]);
    }

    fn saved_entry(
        id: &str,
        display_name: &str,
        scope: WorkbenchLayoutScope,
        default_for_user: bool,
        default_for_project: bool,
    ) -> WorkbenchLayoutEntry {
        WorkbenchLayoutEntry {
            id: id.to_owned(),
            display_name: display_name.to_owned(),
            scope,
            source: WorkbenchLayoutSource::SavedBlueprint,
            description: None,
            blueprint_path: Some(std::path::PathBuf::from(format!("{display_name}.rbl"))),
            default_for_project,
            default_for_user,
            compatibility_version: workbench_compatibility_version(),
        }
    }

    fn write_entry(directory: &std::path::Path, name: &str, entry: &WorkbenchLayoutEntry) {
        std::fs::write(
            directory.join(format!("{name}.json")),
            serde_json::to_string_pretty(entry).unwrap(),
        )
        .unwrap();
    }
}
