//! O estado do **Envelope** (ADR-0129) publicado pela shell — irmão de `state.rs` pelo teto
//! de 600 LOC daquele arquivo, e coeso pelo mesmo critério que já separou os Effects: é a
//! família inteira de uma feature, com os seus statics ao lado dos seus acessores.

use std::cell::{Cell, RefCell};

thread_local! {
    static CURRENT_HAS_ENVELOPE: Cell<bool> = const { Cell::new(false) };
    /// O GESTO da gaiola: 0 = Perspective · 1 = Mesh · 2 = Pins. Um índice, e não um espelho do
    /// `ph2d_ecs::EnvelopeKind`: este painel não vê a crate do ECS, e a UI só precisa saber qual
    /// chip acender.
    static CURRENT_ENVELOPE_MODE: Cell<u8> = const { Cell::new(0) };
    /// Os rótulos dos presets de gaiola, PUBLICADOS pelo shell (a tabela mora no
    /// `ph2d_ecs::EnvelopeWarp`, que este painel não vê). Vazio = nenhum publicado ainda.
    static CURRENT_ENVELOPE_PRESETS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    /// O índice do preset ATIVO na lista acima; `None` = a gaiola é manual (a mão a promoveu).
    static CURRENT_ENVELOPE_WARP: Cell<Option<usize>> = const { Cell::new(None) };
    /// A força do preset ativo, `[-1, 1]`.
    static CURRENT_ENVELOPE_BEND: Cell<f64> = const { Cell::new(0.0) };
}

pub fn set_current_has_envelope(v: bool) {
    CURRENT_HAS_ENVELOPE.with(|c| c.set(v));
}

pub(crate) fn has_envelope() -> bool {
    CURRENT_HAS_ENVELOPE.with(Cell::get)
}

/// Publica em que **gesto** está o envelope da seleção: `0` Perspective · `1` Mesh · `2` Pins.
///
/// Só é lido quando [`has_envelope`] é `true` — sem envelope não há gesto a mostrar. Um índice e não
/// um espelho do enum: este painel não vê o `ph2d-ecs`, e a UI só precisa saber qual chip acender.
pub fn set_current_envelope_mode(v: u8) {
    CURRENT_ENVELOPE_MODE.with(|c| c.set(v));
}

pub(crate) fn envelope_mode() -> u8 {
    CURRENT_ENVELOPE_MODE.with(Cell::get)
}

/// Publica os presets de gaiola: os rótulos (na ordem de `ph2d_ecs::EnvelopeWarp::ALL`), qual está
/// ativo e a força dele.
///
/// **O painel se auto-popula desta lista** — o mesmo idioma da rack de áudio (`set_fx_kind_names`):
/// acrescentar um preset é uma linha na tabela do componente e **zero mudança de painel**. Uma lista
/// escrita à mão aqui driftaria da tabela no primeiro preset novo.
pub fn set_current_envelope_presets(labels: &[&'static str], active: Option<usize>, bend: f64) {
    CURRENT_ENVELOPE_PRESETS.with(|c| {
        let mut v = c.borrow_mut();
        v.clear();
        v.extend_from_slice(labels);
    });
    CURRENT_ENVELOPE_WARP.with(|c| c.set(active));
    CURRENT_ENVELOPE_BEND.with(|c| c.set(bend));
}

pub(crate) fn envelope_presets() -> Vec<&'static str> {
    CURRENT_ENVELOPE_PRESETS.with(|c| c.borrow().clone())
}

pub(crate) fn envelope_warp() -> Option<usize> {
    CURRENT_ENVELOPE_WARP.with(Cell::get)
}

pub(crate) fn envelope_bend() -> f64 {
    CURRENT_ENVELOPE_BEND.with(Cell::get)
}
