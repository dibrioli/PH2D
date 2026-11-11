//! B39.4: a colour pick goes to the tool ONCE. The published colour comes back a frame after the tool
//! applies it; comparing only with it sent the same pick twice (two undo steps for one gesture).

use ph2d_a11y::NodeId;
use std::cell::RefCell;
use std::collections::BTreeMap;

thread_local! {
    static ENVIADA: RefCell<BTreeMap<NodeId, String>> = const { RefCell::new(BTreeMap::new()) };
}

/// `true` when `payload` must go out for swatch `id`: not yet `aplicada` and not the last one sent.
pub(crate) fn deve_enviar(id: NodeId, payload: &str, aplicada: bool) -> bool {
    ENVIADA.with(|m| {
        let mut m = m.borrow_mut();
        if aplicada {
            m.remove(&id);
            return false;
        }
        if m.get(&id).is_some_and(|p| p == payload) {
            return false;
        }
        m.insert(id, payload.to_owned());
        true
    })
}

/// A new picker gesture on `id` (the picker opened on it again): what it picks goes out, even the
/// colour sent last time — the tool may hold another colour now (undo, another document).
pub(crate) fn esquece(id: NodeId) {
    ENVIADA.with(|m| m.borrow_mut().remove(&id));
}
