//! Stable string-keyed handle for tool icons.
//!
//! Per `docs/Migracao/2026-05-convention-by-discovery.md` §4.4 + Agente
//! 1's refinement: icons live **inside the tool's manifest** (as an
//! `icon_fn: fn() -> BezPath`), never in a central registry like the
//! legacy `enum IconId`. `IconHandle` is a typed wrapper for the
//! string id used to look up that fn through the [`Registry`].
//!
//! Adopted at PR 8 (chrome derivation) when `topbar_clusters()` /
//! `left_rail()` stop indexing into `enum IconId` and start asking the
//! registry by handle. Chrome-fixed icons (Save, Open, Settings, etc.)
//! remain on the legacy enum during the transition; only tool icons
//! migrate.

use crate::manifest::ToolId;
use crate::runtime::Registry;

/// Typed handle for an icon. Wraps a tool id string; resolves through
/// the [`Registry`] to the tool's `icon_fn`. `Copy` + `&'static str`
/// makes the handle cheap to pass everywhere and equivalent to a
/// `ToolId` at runtime — but the wrapping type prevents accidental
/// mixing with cluster ids, label keys, or other `&'static str` fields
/// on the manifest.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IconHandle(pub ToolId);

impl IconHandle {
    pub const fn new(id: ToolId) -> Self {
        Self(id)
    }

    /// Resolve the handle to the tool's `icon_fn` via the registry.
    /// Returns `None` if the tool isn't registered (forgotten line in
    /// `tools/registry_init.rs`) or the registry hasn't been built
    /// yet.
    pub fn resolve(self, reg: &Registry) -> Option<fn() -> crate::BezPath> {
        reg.by_id(self.0).map(|m| m.icon_fn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Zone;
    use crate::manifest::{HandlerFn, McpExposure, ToolHandler, ToolManifest};
    use ph2d_a11y::Role;
    use ph2d_core::MemoryBudget;

    fn icon_a() -> crate::BezPath {
        let mut p = crate::BezPath::new();
        p.move_to((1.0, 2.0));
        p
    }
    fn icon_b() -> crate::BezPath {
        let mut p = crate::BezPath::new();
        p.move_to((9.0, 9.0));
        p
    }
    fn click() {}

    const A: ToolManifest = ToolManifest {
        id: "alpha",
        label_key: "k",
        icon_fn: icon_a,
        zone: Zone::TopRight,
        cluster: "c",
        order: 0,
        a11y_role: Role::Button,
        handler: ToolHandler::OneShot {
            on_click: click as HandlerFn,
        },
        memory_budget: MemoryBudget::new(0, 0, 0),
        touches_sim: false,
        mcp: McpExposure::reserved(),
    };
    const B: ToolManifest = ToolManifest {
        id: "beta",
        icon_fn: icon_b,
        ..A
    };

    #[test]
    fn handle_resolves_to_registered_icon_fn() {
        let mut reg = Registry::default();
        reg.register(&A);
        reg.register(&B);
        reg.build().expect("should build");

        let h_a = IconHandle::new("alpha");
        let h_b = IconHandle::new("beta");

        let f_a = h_a.resolve(&reg).expect("alpha icon should resolve");
        let f_b = h_b.resolve(&reg).expect("beta icon should resolve");

        // Different fns; checking the bezpath shape confirms we routed
        // to the right one.
        let p_a = f_a();
        let p_b = f_b();
        assert_ne!(
            p_a.elements().len(),
            0,
            "icon_a should return non-empty path"
        );
        assert_ne!(
            p_b.elements().len(),
            0,
            "icon_b should return non-empty path"
        );
        // Distinct first elements (icon_a moves to (1,2); icon_b to (9,9)).
        assert_ne!(format!("{:?}", p_a), format!("{:?}", p_b));
    }

    #[test]
    fn handle_unknown_id_returns_none() {
        let mut reg = Registry::default();
        reg.register(&A);
        reg.build().expect("should build");
        assert!(IconHandle::new("nope").resolve(&reg).is_none());
    }

    #[test]
    fn handle_before_build_returns_none() {
        let mut reg = Registry::default();
        reg.register(&A);
        // Note: no .build() call.
        assert!(IconHandle::new("alpha").resolve(&reg).is_none());
    }
}
