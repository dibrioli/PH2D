//! **Os intents da TRAJETÓRIA** (ADR-0141) — o K que ancora e o auto-orient.
//!
//! Split de `intent_apply.rs` sob o teto de 700 LOC, e uma unidade por direito próprio:
//! são os dois únicos intents cujo efeito não é sobre keys nem sobre strips, mas sobre
//! a GEOMETRIA que uma track de Position carrega ao lado.

use ph2d_anim::RationalTime;

use crate::intent_apply::edit;
use crate::state::TimelineState;

/// **O K do modo Path**: a âncora entra onde o objeto está, e o documento reescreve as
/// distâncias que TODAS as keys guardam — a porta única do
/// [`crate::TimelineDoc::add_path_key`].
pub(super) fn add_key(state: &mut TimelineState, entity: u64, t: RationalTime, at: [f32; 2]) {
    edit(state, |doc, _| {
        doc.key_the_path(entity, t, at);
    });
}

/// **Alterna o auto-orient.** Lê o estado RESOLVIDO para decidir o próximo — uma track
/// de Rotation o recusa, e alternar contra o flag AUTORADO faria o primeiro clique
/// depois de uma recusa não mudar nada visível.
pub(super) fn toggle_orient(state: &mut TimelineState, entity: u64) {
    edit(state, |doc, _| {
        let on = doc.auto_orient(entity) == crate::AutoOrient::Off;
        doc.set_auto_orient(entity, on);
    });
}
