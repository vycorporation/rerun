//! Code for automatic layout of views.
//!
//! This uses some very rough heuristics and have a lot of room for improvement.

use std::collections::BTreeMap;

use itertools::Itertools as _;
use re_sdk_types::ViewClassIdentifier;
use re_viewer_context::{ContainerId, ViewId, blueprint_id_to_tile_id};

use crate::ViewBlueprint;

#[derive(Clone, Debug)]
struct SpaceMakeInfo {
    id: ViewId,
    class_identifier: ViewClassIdentifier,
    layout_priority: re_viewer_context::ViewClassLayoutPriority,
    tab_group: Option<&'static str>,
}

#[derive(Clone, Debug)]
enum LayoutItem {
    Pane(SpaceMakeInfo),
    TabGroup {
        name: &'static str,
        spaces: Vec<SpaceMakeInfo>,
        layout_priority: re_viewer_context::ViewClassLayoutPriority,
    },
}

impl LayoutItem {
    fn layout_priority(&self) -> re_viewer_context::ViewClassLayoutPriority {
        match self {
            Self::Pane(space) => space.layout_priority,
            Self::TabGroup {
                layout_priority, ..
            } => *layout_priority,
        }
    }
}

pub(crate) fn tree_from_views(
    view_class_registry: &re_viewer_context::ViewClassRegistry,
    views: &BTreeMap<ViewId, ViewBlueprint>,
) -> egui_tiles::Tree<ViewId> {
    re_log::trace!("Auto-layout of {} views", views.len());

    let space_make_infos = views
        .iter()
        // Sort for determinism:
        .sorted_by_key(|(view_id, view)| (&view.space_origin, &view.display_name, *view_id))
        .map(|(view_id, view)| {
            let class_identifier = view.class_identifier();
            let class = view.class(view_class_registry);
            let layout_priority = class.layout_priority();
            let tab_group = class.default_spawned_tab_group();
            SpaceMakeInfo {
                id: *view_id,
                class_identifier,
                layout_priority,
                tab_group,
            }
        })
        .collect_vec();

    let layout_items = grouped_layout_items(space_make_infos);
    let mut tiles = egui_tiles::Tiles::default();

    let root = if layout_items.len() == 1 {
        insert_layout_item(layout_items[0].clone(), &mut tiles)
    } else if layout_items.len() == 3 {
        // Special-case for common case that doesn't fit nicely in a grid
        arrange_three(
            [
                layout_items[0].clone(),
                layout_items[1].clone(),
                layout_items[2].clone(),
            ],
            &mut tiles,
        )
    } else if layout_items.len() <= 12 {
        // Arrange it all in a grid that is responsive to changes in viewport size:
        let child_tile_ids = layout_items
            .into_iter()
            .map(|item| insert_layout_item(item, &mut tiles))
            .collect_vec();
        tiles.insert_grid_tile(child_tile_ids)
    } else {
        // So many views - lets group by class and put the members of each group into tabs:
        let mut grouped_by_class: BTreeMap<Option<ViewClassIdentifier>, Vec<LayoutItem>> =
            Default::default();
        for item in layout_items {
            grouped_by_class
                .entry(match &item {
                    LayoutItem::Pane(space) => Some(space.class_identifier),
                    LayoutItem::TabGroup { .. } => None,
                })
                .or_default()
                .push(item);
        }

        let groups = grouped_by_class
            .values()
            .cloned()
            .sorted_by_key(|group| -(group[0].layout_priority() as isize));

        let tabs = groups
            .into_iter()
            .map(|group| {
                let children = group
                    .into_iter()
                    .map(|item| insert_layout_item(item, &mut tiles))
                    .collect_vec();
                tiles.insert_tab_tile(children)
            })
            .collect_vec();

        if tabs.len() == 1 {
            tabs[0]
        } else {
            tiles.insert_grid_tile(tabs)
        }
    };

    egui_tiles::Tree::new("viewport_tree", root, tiles)
}

fn grouped_layout_items(spaces: Vec<SpaceMakeInfo>) -> Vec<LayoutItem> {
    let mut grouped: BTreeMap<&'static str, Vec<SpaceMakeInfo>> = Default::default();
    let mut items = Vec::new();

    for space in spaces {
        if let Some(group) = space.tab_group {
            grouped.entry(group).or_default().push(space);
        } else {
            items.push(LayoutItem::Pane(space));
        }
    }

    for (name, spaces) in grouped {
        let layout_priority = spaces
            .iter()
            .map(|space| space.layout_priority)
            .max()
            .unwrap_or_default();
        items.push(LayoutItem::TabGroup {
            name,
            spaces,
            layout_priority,
        });
    }

    items.sort_by_key(|item| -(item.layout_priority() as isize));
    items
}

fn insert_layout_item(
    item: LayoutItem,
    tiles: &mut egui_tiles::Tiles<ViewId>,
) -> egui_tiles::TileId {
    match item {
        LayoutItem::Pane(space) => tiles.insert_pane(space.id),
        LayoutItem::TabGroup { name, spaces, .. } => {
            let children = spaces
                .into_iter()
                .map(|space| tiles.insert_pane(space.id))
                .collect_vec();
            let tile_id = blueprint_id_to_tile_id(&default_tab_group_container_id(name));
            tiles.insert(
                tile_id,
                egui_tiles::Tile::Container(egui_tiles::Container::new(
                    egui_tiles::ContainerKind::Tabs,
                    children,
                )),
            );
            tile_id
        }
    }
}

pub(crate) fn default_tab_group_container_id(group_name: &str) -> ContainerId {
    ContainerId::hashed_from_str(&format!("default_tab_group/{group_name}"))
}

pub(crate) fn default_tab_group_container_name(container_id: ContainerId) -> Option<&'static str> {
    (container_id == default_tab_group_container_id("Inspector")).then_some("Inspector")
}

fn arrange_three(
    mut spaces: [LayoutItem; 3],
    tiles: &mut egui_tiles::Tiles<ViewId>,
) -> egui_tiles::TileId {
    // We will arrange it like so:
    //
    // +-------------+
    // |             |
    // |             |
    // |             |
    // +-------+-----+
    // |       |     |
    // |       |     |
    // +-------+-----+
    //
    // or like so:
    //
    // +-----------------------+
    // |          |            |
    // |          |            |
    // |          +------------+
    // |          |            |
    // |          |            |
    // |          |            |
    // +----------+------------+
    //
    // But which space gets a full side, and which doesn't?
    // Answer: we prioritize them based on a class-specific layout priority:

    spaces.sort_by_key(|item| -(item.layout_priority() as isize));

    let pane_ids = spaces
        .into_iter()
        .map(|item| insert_layout_item(item, tiles))
        .collect_vec();

    let inner_grid = tiles.insert_grid_tile(vec![pane_ids[1], pane_ids[2]]);
    tiles.insert_grid_tile(vec![pane_ids[0], inner_grid])
}
