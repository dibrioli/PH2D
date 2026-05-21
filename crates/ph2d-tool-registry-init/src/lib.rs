#![forbid(unsafe_code)]
//! ph2d-tool-registry-init — APPEND-ONLY tool registration point.
//!
//! Single function: [`register_all`]. Adding a new tool = adding one
//! line below, in alphabetical order, plus the matching `[dependencies]`
//! entry in this crate's `Cargo.toml`. Coordenador-only edits — the
//! tool author owns the tool crate; this crate owns the wiring list.
//!
//! See `docs/Migracao/2026-05-convention-by-discovery.md` Appendix G
//! for the canonical shape and PR 6.0 for the extraction rationale.

use ph2d_tool_registry::Registry;

/// Register all tool manifests with the runtime registry. Called once
/// at boot from `shells/<plat>/src/init.rs`.
///
/// Order of calls doesn't matter for correctness — [`Registry::build`]
/// sorts by `(cluster, order, id)` for deterministic iteration (HR-5).
/// Alphabetical order in source is a merge-conflict-hygiene
/// convention for the multi-agent operating model.
pub fn register_all(reg: &mut Registry) {
    ph2d_tool_bgremoval::register(reg);
    ph2d_tool_grid_snap::register(reg);
    ph2d_tool_make_square::register(reg);
    ph2d_tool_padding::register(reg);
    ph2d_tool_real_size::register(reg);
    ph2d_tool_trim_transparency::register(reg);
    //   ph2d_tool_brush::register(reg);         // future
    //   ph2d_tool_move::register(reg);          // future
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_all_wires_migrated_tools() {
        let mut reg = Registry::default();
        register_all(&mut reg);
        reg.build()
            .expect("registry should build cleanly with all wired tools");
        assert!(
            reg.by_id("make_square").is_some(),
            "make_square should be registered after PR 4"
        );
        assert!(
            reg.by_id("grid_snap").is_some(),
            "grid_snap should be registered after PR 6"
        );
        assert!(
            reg.by_id("bgremoval").is_some(),
            "bgremoval should be registered after PR 7"
        );
        assert!(
            reg.by_id("trim_transparency").is_some(),
            "trim_transparency should be registered after Wave 2 PR 11.4"
        );
    }

    #[test]
    fn image_tools_cluster_in_canonical_order() {
        // Wave 2 PR 11.4 contract: every Image Tools action pill comes
        // from the `image_tools` cluster, ordered by manifest
        // `order` field. `Registry::cluster` sorts by `(order, id)`,
        // so this is the canonical paint order: trim (40), make_square
        // (50), bgremoval (60), real_size (70), padding (80).
        let mut reg = Registry::default();
        register_all(&mut reg);
        reg.build().expect("registry should build");
        let cluster = reg.cluster("image_tools");
        let ids: Vec<&str> = cluster.iter().map(|m| m.id).collect();
        assert_eq!(
            ids,
            vec![
                "trim_transparency",
                "make_square",
                "bgremoval",
                "real_size",
                "padding"
            ]
        );
    }
}
