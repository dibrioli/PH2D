//! **The Add Node catalog + the full-screen palette model** — split from `motion_bridge` for the shell
//! LOC cap. `super` is `render_loop::motion_bridge`. Gated by the parent's `#[cfg(feature =
//! "panel-motion-graph")]` mod declaration, so nothing here needs its own cfg.
//!
//! Two concerns, one place: `build_catalog` is the flat list the graph menu publishes; `build_palette_model`
//! groups that same list into the coloured, sub-clustered model the shell's "Add Node" palette renders.
//! Both derive from the registry — the single source of what a node is called and which category it wears.
//! Plus the two handshake ends: `open_pending_palette` (a gesture asked → open, filtered) and
//! `route_palette_pick` (last frame's pick → the right graph edit).

use crate::motion_state::MotionState;
use ph2d_editor::HeroScreen;

/// Route LAST frame's palette pick into a graph edit — mapping the picked id back to its canonical
/// `type_name`, then turning it (WITH the gesture's wire context, drained from `library_open`) into the
/// right intent via `library_pick`: a plain add, a smart-connect, or a splice. Call BEFORE draining
/// intents so the edit lands this frame. A click and an Enter on the palette both arrive here, so they
/// route identically.
pub(super) fn route_palette_pick(hero: &mut HeroScreen, motion: &mut MotionState) {
    // ⚠️ CONDICIONAL, e é o que torna a ordem dos drenos irrelevante. O canal do pick tem DOIS
    // consumidores desde a paleta de comandos global (`hero::global_palette`); um `take`
    // incondicional aqui engoliria um comando do chrome e devolveria `None` a quem o soubesse
    // executar, com o sintoma a ser *«às vezes não faz nada»*.
    let known = |id| {
        motion
            .registry
            .manifests()
            .any(|m| ph2d_tool_registry::hash_node_id(m.name) == id)
    };
    let Some(id) = hero.store.take_command_pick_if(known) else {
        return;
    };
    let type_name = motion
        .registry
        .manifests()
        .map(|m| m.name)
        .find(|name| ph2d_tool_registry::hash_node_id(name) == id);
    if let Some(type_name) = type_name {
        let open = motion.library_open.take().unwrap_or_default();
        ph2d_panel_motion_graph::push_intent(ph2d_panel_motion_graph::library_pick(
            open.connect_from,
            open.splice,
            type_name,
            open.spawn,
        ));
    }
}

/// If a gesture asked (`OpenLibrary`, stashed in `open_library`), open the full-screen palette on the
/// live catalog — FILTERED to the compatible types for a smart-connect — and remember the wire context
/// in `library_open` for [`route_palette_pick`].
pub(super) fn open_pending_palette(hero: &mut HeroScreen, motion: &mut MotionState) {
    if let Some(open) = motion.open_library.take() {
        hero.store
            .open_command_palette(build_palette_model(&motion.registry, &open.compatible));
        motion.library_open = Some(open);
    }
}

/// Build the addable-node catalog from the registry (canonical name + English display label + category),
/// sorted by category then label so the menu groups by colour (the palette teaches the library map,
/// plan §2.4).
pub(super) fn build_catalog(
    registry: &ph2d_node_registry::NodeRegistry,
) -> Vec<ph2d_panel_motion_graph::NodeChoice> {
    use ph2d_node_registry::NodeUiCategory;
    use ph2d_panel_motion_graph::NodeChoice;
    let mut v: Vec<NodeChoice> = registry
        .manifests()
        .map(|m| {
            let ui = registry.ui_manifest(m.id);
            NodeChoice {
                type_name: m.name,
                display: ui.map(|u| u.display_name).unwrap_or(m.name),
                category: ui.map(|u| u.category).unwrap_or(NodeUiCategory::Utility),
                // Straight off the manifest (`&'static`), so the panel can filter
                // the smart-connect menu by what each type can actually take.
                inputs: m.inputs,
            }
        })
        .collect();
    v.sort_by(|a, b| (a.category as u8, a.display).cmp(&(b.category as u8, b.display)));
    v
}

/// Build the full-screen "Add Node" palette model from the live catalog: the 7 categories in display
/// order, each with the `node-cat-*` colour from the panel's single `cat_token`, and the two overloaded
/// categories (Transform, Utility) split into named sub-clusters so the layout stays scannable. Every
/// item id is `hash_node_id(type_name)` — the same hash the pick round-trips through.
///
/// `compatible` is the smart-connect allow-list: when non-empty, only those node types are shown (a wire
/// dropped in empty space opens the palette filtered to what it can feed, replacing the old dropdown's
/// filter). Empty = the whole catalog (plain `A` / R-click).
pub(super) fn build_palette_model(
    registry: &ph2d_node_registry::NodeRegistry,
    compatible: &[&'static str],
) -> ph2d_editor::widget::command_palette::PaletteModel {
    use ph2d_editor::widget::command_palette::{
        PaletteGroup, PaletteItem, PaletteModel, PaletteSub,
    };
    use ph2d_node_registry::NodeUiCategory;

    let mut cat = build_catalog(registry); // already sorted by (category, display)
    if !compatible.is_empty() {
        cat.retain(|nc| compatible.contains(&nc.type_name));
    }
    const ORDER: [(NodeUiCategory, &str); 7] = [
        (NodeUiCategory::Source, "Source"),
        (NodeUiCategory::Distribute, "Distribute"),
        (NodeUiCategory::Transform, "Transform"),
        (NodeUiCategory::Focus, "Focus"),
        (NodeUiCategory::Fx, "Fx"),
        (NodeUiCategory::Output, "Output"),
        (NodeUiCategory::Utility, "Utility"),
    ];
    let make_item = |nc: &ph2d_panel_motion_graph::NodeChoice| PaletteItem {
        label: nc.display.to_string(),
        id: ph2d_tool_registry::hash_node_id(nc.type_name),
    };
    let mut groups = Vec::new();
    for (c, title) in ORDER {
        let in_cat: Vec<&ph2d_panel_motion_graph::NodeChoice> = cat
            .iter()
            .filter(|nc| nc.category as u8 == c as u8)
            .collect();
        if in_cat.is_empty() {
            continue;
        }
        let sub_titles = palette_subgroups(c);
        let subs = if sub_titles.is_empty() {
            vec![PaletteSub {
                title: None,
                items: in_cat.iter().map(|nc| make_item(nc)).collect(),
            }]
        } else {
            sub_titles
                .iter()
                .filter_map(|&st| {
                    let items: Vec<PaletteItem> = in_cat
                        .iter()
                        .filter(|nc| palette_subgroup_of(c, nc.display) == Some(st))
                        .map(|nc| make_item(nc))
                        .collect();
                    (!items.is_empty()).then_some(PaletteSub {
                        title: Some(st.to_string()),
                        items,
                    })
                })
                .collect()
        };
        groups.push(PaletteGroup {
            title: title.to_string(),
            color: ph2d_panel_motion_graph::cat_token(c),
            subs,
        });
    }
    PaletteModel {
        title: "Add Node".to_string(),
        groups,
    }
}

/// The named sub-clusters for the two overloaded categories (empty = a flat category). Order is the
/// display order in the palette.
fn palette_subgroups(c: ph2d_node_registry::NodeUiCategory) -> &'static [&'static str] {
    use ph2d_node_registry::NodeUiCategory;
    match c {
        NodeUiCategory::Transform => &[
            "Basic Transforms",
            "Deformers",
            "Forces & Physics",
            "Rigging",
            "Behaviors & Timing",
        ],
        NodeUiCategory::Utility => &["Values & Math", "Time & Signal", "Data & Adapters"],
        _ => &[],
    }
}

/// Which sub-cluster a node belongs to, by display name. The last arm is a CATCH-ALL, so a node added to
/// the registry later lands in a sensible cluster instead of vanishing from the palette.
fn palette_subgroup_of(
    c: ph2d_node_registry::NodeUiCategory,
    display: &str,
) -> Option<&'static str> {
    use ph2d_node_registry::NodeUiCategory;
    match c {
        NodeUiCategory::Transform => Some(match display {
            "Move" | "Rotate" | "Scale" | "Transform" | "Mirror" | "Orbit" | "Look At" => {
                "Basic Transforms"
            }
            "Bend" | "Twist" | "Spherize" | "Four Point Warp" | "Kaleidoscope" | "Spline Wrap" => {
                "Deformers"
            }
            "Attractor" | "Vortex" | "Wind" | "Drag" | "Curl Noise" | "Noise" | "Spring"
            | "Integrate" | "Collide" | "Collider" | "Buoyancy" | "Simulation Step"
            | "Simulation Zone" => "Forces & Physics",
            "FABRIK" | "FK" | "IK 2-Bone" | "Rubber Hose" | "Skin" => "Rigging",
            _ => "Behaviors & Timing",
        }),
        NodeUiCategory::Utility => Some(match display {
            "Math" | "Unary" | "Compare" | "Gain" | "Mix" | "Normalize" | "Quantize"
            | "Threshold" | "Step" | "Wrap" | "Slope" | "Smooth" | "Median" | "Percentile"
            | "Reduce" | "Sort" | "Cull" => "Values & Math",
            "Time" | "Time Remap" | "LFO" | "Beat" | "Counter" | "On Change" | "Sample & Hold" => {
                "Time & Signal"
            }
            _ => "Data & Adapters",
        }),
        _ => None,
    }
}
