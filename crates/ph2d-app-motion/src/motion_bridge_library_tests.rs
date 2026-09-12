//! Gates for the "Add Node" palette wiring (the shell half). `super` is `render_loop::motion_bridge`.
//! The editor-core chrome half (scrim closes / item records the pick) is gated in
//! `ph2d-editor-core::screens::hero::chrome::command_palette`; here we prove the Motion-specific glue:
//! the model built from the live catalog, and the `OpenLibrary` intent stashing the spawn.

use crate::motion_state::MotionState;
use ph2d_panel_motion_graph::GraphIntent;

/// **The palette model groups the live catalog by category, sub-clustering the two overloaded ones.**
/// FALSIFIED by leaving Transform flat (its subs would be one `None`-titled cluster), or by dropping the
/// grouping (the model would be empty / mis-ordered).
#[test]
fn the_palette_model_groups_the_catalog_by_category_with_subclusters() {
    let m = MotionState::new();
    let model = super::library::build_palette_model(&m.registry, &[]);

    let titles: Vec<&str> = model.groups.iter().map(|g| g.title.as_str()).collect();
    assert_eq!(
        titles.first(),
        Some(&"Source"),
        "Source leads (display order)"
    );
    assert!(
        titles.contains(&"Transform") && titles.contains(&"Utility"),
        "the overloaded categories are present"
    );
    assert!(
        model.item_count() >= 100,
        "the full catalog is present (~111 nodes), got {}",
        model.item_count()
    );

    // Transform is split into NAMED sub-clusters; a plain category (Source) is one flat, un-titled list.
    let tx = model
        .groups
        .iter()
        .find(|g| g.title == "Transform")
        .expect("Transform group");
    assert!(
        tx.subs.iter().all(|s| s.title.is_some()),
        "Transform is split into named sub-clusters"
    );
    assert!(
        tx.subs
            .iter()
            .any(|s| s.title.as_deref() == Some("Forces & Physics")),
        "the Forces & Physics sub-cluster exists"
    );
    let src = model
        .groups
        .iter()
        .find(|g| g.title == "Source")
        .expect("Source group");
    assert_eq!(src.subs.len(), 1);
    assert!(
        src.subs[0].title.is_none(),
        "Source is a flat category (no sub-headers)"
    );

    // Every item id is `hash_node_id(type_name)` — the round-trip the pick closes in the bridge. Prove it
    // for one known node so a change to the id scheme is caught.
    let boids_id = ph2d_tool_registry::hash_node_id("motion.boids");
    assert!(
        model
            .groups
            .iter()
            .flat_map(|g| &g.subs)
            .flat_map(|s| &s.items)
            .any(|it| it.id == boids_id),
        "an item carries the hash of its type_name (motion.boids), so the pick maps back"
    );
}

/// **A non-empty `compatible` filters the palette to the smart-connect allow-list.** A loose end dropped
/// in space opens the palette showing ONLY the types that wire can feed. FALSIFIED by ignoring
/// `compatible` (the whole catalog would show, and the artist would pick something the shell then
/// refuses).
#[test]
fn the_palette_model_is_filtered_to_the_compatible_types() {
    let m = MotionState::new();
    let full = super::library::build_palette_model(&m.registry, &[]);
    let filtered = super::library::build_palette_model(&m.registry, &["motion.boids"]);
    assert!(
        full.item_count() > filtered.item_count(),
        "the allow-list narrows the catalog"
    );
    assert_eq!(
        filtered.item_count(),
        1,
        "only the one allowed type is shown"
    );
    let boids = ph2d_tool_registry::hash_node_id("motion.boids");
    assert!(
        filtered
            .groups
            .iter()
            .flat_map(|g| &g.subs)
            .flat_map(|s| &s.items)
            .any(|it| it.id == boids),
        "and it is the allowed type"
    );
}

/// **The `OpenLibrary` intent stashes the spawn AND the wire context for the bridge to open the palette.**
/// The bridge — which owns the editor `WidgetStore` — reads `open_library` after the drain, filters the
/// model to `compatible` and opens the full-screen palette. FALSIFIED by not handling `OpenLibrary` (the
/// stash stays `None`, the gesture opens nothing) or by dropping the wire context (smart-connect / splice
/// would fall back to a plain add).
#[test]
fn open_library_intent_stashes_the_spawn_and_wire_context() {
    use crate::motion_state::LibraryOpen;
    let mut m = MotionState::new();
    assert!(
        m.open_library.is_none(),
        "nothing stashed before the intent"
    );

    let _ = ph2d_panel_motion_graph::drain_intents(); // clear any intent the boot left behind
    ph2d_panel_motion_graph::push_intent(GraphIntent::OpenLibrary {
        x: 12.0,
        y: 34.0,
        connect_from: Some((7, 1)),
        splice: None,
        compatible: vec!["motion.grid"],
    });
    super::apply_graph_intents(
        &mut m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor::ToastQueue::default(),
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );

    assert_eq!(
        m.open_library,
        Some(LibraryOpen {
            spawn: (12.0, 34.0),
            connect_from: Some((7, 1)),
            splice: None,
            compatible: vec!["motion.grid"],
        }),
        "OpenLibrary stashes the spawn + the wire context; the bridge opens the palette with them"
    );
}
