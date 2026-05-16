//! Registry — collects `ToolManifest` references and derives the
//! chrome / icon / dispatch tables from them.
//!
//! PR 1 ships the struct shape + an empty `build()` path. Collision
//! detection (NodeId hash) lands in PR 2; cluster validation (ADR-0023
//! cluster table) lands when the cluster table is canonicalized; full
//! aggregation (HR-13 memory budget sum + platform check) lands in
//! PR 3 alongside the CI lint.
//!
//! Lookup invariants:
//! - Order of iteration is deterministic cross-platform: tools are
//!   sorted by `(cluster, order, id)` at `build()` time (HR-5).
//! - No `HashMap` in the iteration path; `BTreeMap` instead (ADR-0022
//!   ban applies to sim crates; chrome is PresentWorld so technically
//!   exempt, but consistency reduces foot-gun surface).

use std::collections::BTreeMap;

use crate::manifest::{ClusterId, ToolId, ToolManifest};
use crate::node_id::{CollisionError, detect_collisions};

/// Errors that can fire from [`Registry::build`]. Variants will grow
/// as PR 3 (budget overrun) and PR 8 (cluster validation) add their
/// checks.
#[derive(Debug)]
pub enum BuildError {
    /// Two manifests share the same `id` after registration.
    DuplicateId(ToolId),
    /// Two distinct manifest ids hash to the same [`ph2d_a11y::NodeId`].
    /// Astronomically rare under FNV-1a 64-bit in the sub-200-id space
    /// of a mature editor, but defended for HR-5 determinism.
    NodeIdCollision(CollisionError),
    // Future variants land in subsequent PRs:
    //   UnknownCluster { id: ToolId, cluster: ClusterId },           // PR 8
    //   MemoryBudgetExceeded { platform: Platform, over_mb: u32 },   // PR 3
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId(id) => {
                write!(f, "duplicate tool id registered twice: {:?}", id)
            }
            Self::NodeIdCollision(c) => write!(f, "{c}"),
        }
    }
}

impl From<CollisionError> for BuildError {
    fn from(e: CollisionError) -> Self {
        Self::NodeIdCollision(e)
    }
}

impl std::error::Error for BuildError {}

/// The runtime registry. Built once at boot from the manifests handed
/// in by `register_all()` (see `crates/ph2d-editor/src/tools/registry_init.rs`).
/// Cheap to query during the frame loop — all lookups are O(log n).
#[derive(Default)]
pub struct Registry {
    /// All registered manifests, in registration order until `build()`
    /// runs (then sorted by `(cluster, order, id)`).
    manifests: Vec<&'static ToolManifest>,

    /// `id` → manifest. Built at `build()` time. Empty until then.
    by_id: BTreeMap<ToolId, &'static ToolManifest>,

    /// `cluster` → ordered manifest list. Built at `build()`. Used by
    /// chrome derivation (PR 8 — TopBar / LeftRail).
    by_cluster: BTreeMap<ClusterId, Vec<&'static ToolManifest>>,

    /// `true` after `build()` succeeded. Lookups before `build()`
    /// return empty / `None`.
    built: bool,
}

impl Registry {
    /// Register one manifest. Idempotent on identical `&'static`
    /// references (same const re-registered = no-op). Duplicate ids
    /// from *different* manifests trigger `BuildError::DuplicateId`
    /// when `build()` runs.
    pub fn register(&mut self, manifest: &'static ToolManifest) {
        // Skip exact pointer dup (same const cited twice). Real id
        // collisions are caught by build() so we can report all of
        // them at once with line-precise diagnostics.
        if self.manifests.iter().any(|m| std::ptr::eq(*m, manifest)) {
            return;
        }
        self.manifests.push(manifest);
    }

    /// Finalize the registry. Validates duplicates + NodeId hash
    /// collisions, sorts manifests deterministically, builds lookup
    /// indices. Call exactly once, at boot, after `register_all()` has
    /// run.
    ///
    /// Errors:
    /// - [`BuildError::DuplicateId`] — two manifests have the same id.
    /// - [`BuildError::NodeIdCollision`] — two distinct ids hash to
    ///   the same `NodeId` (astronomically rare under FNV-1a 64-bit
    ///   but defended).
    ///
    /// Future PRs will add: UnknownCluster (PR 8),
    /// MemoryBudgetExceeded (PR 3).
    pub fn build(&mut self) -> Result<(), BuildError> {
        // 1. Duplicate id check.
        let mut seen: BTreeMap<ToolId, ()> = BTreeMap::new();
        for m in &self.manifests {
            if seen.insert(m.id, ()).is_some() {
                return Err(BuildError::DuplicateId(m.id));
            }
        }

        // 2. NodeId hash collision check (HR-5 determinism). Detect
        //    here rather than at use site so the failure mode is
        //    "boot refuses with clear message", not "two unrelated
        //    chrome buttons silently share an a11y id".
        detect_collisions(&self.manifests)?;

        // 3. Sort by (cluster, order, id) for deterministic iteration
        //    (HR-5). Cluster first so cluster-grouped chrome
        //    derivation (PR 8) gets contiguous runs.
        self.manifests.sort_by_key(|m| (m.cluster, m.order, m.id));

        // 4. Build indices.
        self.by_id = self.manifests.iter().map(|m| (m.id, *m)).collect();
        self.by_cluster.clear();
        for m in &self.manifests {
            self.by_cluster.entry(m.cluster).or_default().push(*m);
        }

        self.built = true;
        Ok(())
    }

    /// Lookup by tool id. `None` if not registered or `build()` not
    /// yet called.
    pub fn by_id(&self, id: ToolId) -> Option<&'static ToolManifest> {
        if !self.built {
            return None;
        }
        self.by_id.get(id).copied()
    }

    /// All manifests in a cluster, sorted by `(order, id)`. Empty
    /// vec if no manifests in that cluster or before `build()`.
    pub fn cluster(&self, cluster: ClusterId) -> &[&'static ToolManifest] {
        if !self.built {
            return &[];
        }
        self.by_cluster
            .get(cluster)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// All registered manifests, in sorted order. Used by PR 8 chrome
    /// derivation + CI lint tests (PR 3 aggregates budgets etc.).
    pub fn manifests(&self) -> &[&'static ToolManifest] {
        &self.manifests
    }

    /// True if `build()` has completed successfully.
    pub fn is_built(&self) -> bool {
        self.built
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Zone;
    use crate::manifest::{HandlerFn, McpExposure, ToolHandler};
    use ph2d_a11y::Role;
    use ph2d_core::MemoryBudget;

    fn icon() -> crate::BezPath {
        crate::BezPath::new()
    }
    fn click() {}

    const MAKE_SQUARE: ToolManifest = ToolManifest {
        id: "make_square",
        label_key: "tool.make_square.label",
        icon_fn: icon,
        zone: Zone::TopRight,
        cluster: "image_tools",
        order: 50,
        a11y_role: Role::Button,
        handler: ToolHandler::OneShot {
            on_click: click as HandlerFn,
        },
        memory_budget: MemoryBudget::new(0, 0, 0),
        touches_sim: false,
        mcp: McpExposure::reserved(),
    };

    const TRIM: ToolManifest = ToolManifest {
        id: "trim_transparency",
        label_key: "tool.trim_transparency.label",
        icon_fn: icon,
        zone: Zone::TopRight,
        cluster: "image_tools",
        order: 40,
        a11y_role: Role::Button,
        handler: ToolHandler::OneShot {
            on_click: click as HandlerFn,
        },
        memory_budget: MemoryBudget::new(0, 0, 0),
        touches_sim: false,
        mcp: McpExposure::reserved(),
    };

    #[test]
    fn empty_registry_builds_clean() {
        let mut reg = Registry::default();
        reg.build().expect("empty registry should build");
        assert!(reg.is_built());
        assert!(reg.manifests().is_empty());
        assert!(reg.by_id("make_square").is_none());
    }

    #[test]
    fn lookups_before_build_return_none() {
        let mut reg = Registry::default();
        reg.register(&MAKE_SQUARE);
        // Pre-build: lookups are inert.
        assert!(reg.by_id("make_square").is_none());
        assert!(reg.cluster("image_tools").is_empty());
        assert!(!reg.is_built());
    }

    #[test]
    fn build_indexes_by_id_and_cluster() {
        let mut reg = Registry::default();
        reg.register(&TRIM);
        reg.register(&MAKE_SQUARE);
        reg.build().expect("should build cleanly");

        // by_id
        assert_eq!(reg.by_id("make_square").unwrap().id, "make_square");
        assert_eq!(
            reg.by_id("trim_transparency").unwrap().id,
            "trim_transparency"
        );
        assert!(reg.by_id("nope").is_none());

        // by_cluster — order respected (trim:40 before make_square:50).
        let image_tools = reg.cluster("image_tools");
        assert_eq!(image_tools.len(), 2);
        assert_eq!(image_tools[0].id, "trim_transparency");
        assert_eq!(image_tools[1].id, "make_square");
    }

    #[test]
    fn duplicate_id_is_rejected_at_build() {
        const TRIM_DUP: ToolManifest = ToolManifest {
            id: "trim_transparency", // same id as TRIM
            label_key: "tool.dup.label",
            icon_fn: icon,
            zone: Zone::TopRight,
            cluster: "image_tools",
            order: 60,
            a11y_role: Role::Button,
            handler: ToolHandler::OneShot {
                on_click: click as HandlerFn,
            },
            memory_budget: MemoryBudget::new(0, 0, 0),
            touches_sim: false,
            mcp: McpExposure::reserved(),
        };
        let mut reg = Registry::default();
        reg.register(&TRIM);
        reg.register(&TRIM_DUP);
        match reg.build() {
            Err(BuildError::DuplicateId(id)) => assert_eq!(id, "trim_transparency"),
            other => panic!("expected DuplicateId, got {other:?}"),
        }
    }

    #[test]
    fn registering_same_static_twice_is_idempotent() {
        let mut reg = Registry::default();
        reg.register(&MAKE_SQUARE);
        reg.register(&MAKE_SQUARE); // identical &'static — should dedupe
        reg.build().expect("idempotent registration should build");
        assert_eq!(reg.manifests().len(), 1);
    }

    #[test]
    fn iteration_order_is_deterministic() {
        // Two clusters, two manifests each, registered in scrambled
        // order. Build must reorder them stably.
        const A_HIGH: ToolManifest = ToolManifest {
            id: "a_high",
            label_key: "k",
            icon_fn: icon,
            zone: Zone::TopRight,
            cluster: "alpha",
            order: 200,
            a11y_role: Role::Button,
            handler: ToolHandler::OneShot { on_click: click },
            memory_budget: MemoryBudget::new(0, 0, 0),
            touches_sim: false,
            mcp: McpExposure::reserved(),
        };
        const B_LOW: ToolManifest = ToolManifest {
            id: "b_low",
            label_key: "k",
            icon_fn: icon,
            zone: Zone::TopRight,
            cluster: "beta",
            order: 10,
            a11y_role: Role::Button,
            handler: ToolHandler::OneShot { on_click: click },
            memory_budget: MemoryBudget::new(0, 0, 0),
            touches_sim: false,
            mcp: McpExposure::reserved(),
        };
        const A_LOW: ToolManifest = ToolManifest {
            id: "a_low",
            label_key: "k",
            icon_fn: icon,
            zone: Zone::TopRight,
            cluster: "alpha",
            order: 5,
            a11y_role: Role::Button,
            handler: ToolHandler::OneShot { on_click: click },
            memory_budget: MemoryBudget::new(0, 0, 0),
            touches_sim: false,
            mcp: McpExposure::reserved(),
        };

        let mut reg = Registry::default();
        reg.register(&A_HIGH);
        reg.register(&B_LOW);
        reg.register(&A_LOW);
        reg.build().expect("should build");

        let ids: Vec<&str> = reg.manifests().iter().map(|m| m.id).collect();
        // Cluster "alpha" comes before "beta" alphabetically; within
        // alpha, order 5 < 200.
        assert_eq!(ids, vec!["a_low", "a_high", "b_low"]);
    }
}
