//! **The Add Node catalog + the full-screen palette model** — split from `motion_bridge` for the shell
//! LOC cap. `super` is `render_loop::motion_bridge`. Gated by the parent's `#[cfg(feature =
//! "panel-motion-graph")]` mod declaration, so nothing here needs its own cfg.
//!
//! Two concerns, one place: `build_catalog` is the flat list the graph menu publishes; `build_palette_model`
//! groups that same list into the coloured, sub-clustered model the shell's "Add Node" palette renders.
//! Both derive from the registry — the single source of what a node is called and which category it wears.

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
pub(super) fn build_palette_model(
    registry: &ph2d_node_registry::NodeRegistry,
) -> ph2d_editor::widget::command_palette::PaletteModel {
    use ph2d_editor::widget::command_palette::{
        PaletteGroup, PaletteItem, PaletteModel, PaletteSub,
    };
    use ph2d_node_registry::NodeUiCategory;

    let cat = build_catalog(registry); // already sorted by (category, display)
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
