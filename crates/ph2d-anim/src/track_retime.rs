//! The retime verbs on a [`Track`] (plano 18) — a CHILD module of `track` (file cap), so it reaches
//! the private `keys`.

use super::{Key, KeyId, Track};
use crate::retime::Retime;

impl Track {
    /// [`Track::upsert_key`] for a WHOLE key — its retime included (duplicate, paste).
    pub fn upsert_whole(&mut self, k: Key) -> KeyId {
        let id = self.upsert_key(k.t, k.value, k.interp);
        self.set_retime(id, k.retime);
        id
    }

    /// Set the retime of the segment leaving the key with `id` (`None` = reset the rhythm). Returns
    /// `true` if it existed.
    pub fn set_retime(&mut self, id: KeyId, retime: Option<Retime>) -> bool {
        match self.index_of(id) {
            Some(i) => {
                self.keys[i].retime = retime;
                true
            }
            None => false,
        }
    }
}
