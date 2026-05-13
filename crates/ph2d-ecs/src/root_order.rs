//! `RootOrder` — explicit ordering index for root-level entities.
//!
//! M14.7 polish: root entities (no `ChildOf`) had no stable order in
//! [`crate::scene::build_hierarchy_snapshot`] beyond
//! `Entity::to_bits()`. The hierarchy panel's drag-reorder could
//! change the dispatch's `hierarchy_order`, but the host's snapshot
//! rebuild on the next frame restored the bits-sort — so dropping a
//! root sprite "above" another silently snapped back to id order.
//!
//! This component is read by the snapshot's root sort: roots are
//! ordered by `(RootOrder.0, entity.to_bits())`, with absent =
//! `u32::MAX` so new (untouched) entities collate after every
//! explicitly-ordered one. The editor's `pending_reparent` drain
//! assigns sequential indices when the user drops a root before /
//! after another root.
//!
//! Non-root entities (with `ChildOf`) ignore this component — sibling
//! order is already controlled by `Children`'s insertion order, which
//! the same drain rewrites via the re-insert-ChildOf trick.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootOrder(pub u32);
