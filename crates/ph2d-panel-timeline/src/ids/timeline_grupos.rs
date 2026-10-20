//! Os ids da LINHA DE GRUPO de canais do dope-sheet (`crate::linhas`) — irmão de
//! [`super::timeline`], que está no tecto de LOC dos painéis.

use ph2d_a11y::NodeId;

use super::timeline::dynamic_id;

/// A diamond on a channel-group row, keyed by the group's handle (its first member's
/// `AnimTarget`) and the column time as `f64::to_bits`.
#[must_use]
pub fn timeline_group_key_id(group: u64, t_bits: u64) -> NodeId {
    dynamic_id("timeline.group_key", &[group, t_bits])
}

/// A channel-group row's twirl, keyed like [`timeline_group_key_id`].
#[must_use]
pub fn timeline_group_twirl_id(group: u64) -> NodeId {
    dynamic_id("timeline.group_twirl", &[group])
}
