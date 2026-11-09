//! ⭐ **Quem está na outra ponta de um nome de sinal** (GEF02 onda 1): o sufixo `1 listens` /
//! `nobody listens` / `a script may listen` dentro de cada caixa onde o artista digita um sinal.
//!
//! O censo chega da shell uma vez por quadro ([`set_signal_census`]); a tabela [`CAIXAS`] diz o
//! papel de cada caixa; o pintor da caixa (`ph2d_editor_core::widget::field_suffix`) pergunta ao
//! [`fornecedor`] e reserva a largura — o texto digitado nunca fica por baixo do sufixo.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::widget::field_suffix::{self, Sufixo};
use ph2d_i18n::{tr, tr_with};
use ph2d_signal_ends::{Hint, Role, SignalCensus};
use ph2d_tokens::ColorToken;
use std::cell::RefCell;

pub use ph2d_signal_ends::Role as SignalRole;

thread_local! {
    static CURRENT_SIGNAL_CENSUS: RefCell<SignalCensus> = RefCell::new(SignalCensus::default());
}

/// A shell publica o censo do quadro (uma vez, em `publish_global`).
pub fn set_signal_census(c: SignalCensus) {
    CURRENT_SIGNAL_CENSUS.with(|s| *s.borrow_mut() = c);
}

/// ⭐ **As caixas de nome de sinal do Inspector** e o papel de cada uma (censo de 09/11: 6 ouvem, 25
/// emitem). Caixa nova de sinal = uma linha aqui; o gate `cada_caixa_de_sinal_tem_o_seu_sufixo`
/// confere que cada id é pintado numa secção.
pub const CAIXAS: &[(&[NodeId], Role)] = &[
    (&[ids::INSP_ACTION_ON], Role::Listen),
    (&[ids::INSP_WEAPON_ON_SIGNAL], Role::Listen),
    (&[ids::INSP_WEAPON_RELOAD_ON], Role::Listen),
    (&[ids::INSP_FACTORY_ON_SIGNAL], Role::Listen),
    (&[ids::INSP_SM_TRANS_ON], Role::Listen),
    (&[ids::INSP_EMITTER_ON], Role::Listen),
    (&[ids::INSP_TRIGGER_SIGNAL], Role::Emit),
    (&[ids::INSP_WATCH_SIGNAL], Role::Emit),
    (&[ids::INSP_TIMER_SIGNAL], Role::Emit),
    (&[ids::INSP_WEAPON_ON_FIRE], Role::Emit),
    (&[ids::INSP_WEAPON_ON_EMPTY], Role::Emit),
    (&[ids::INSP_WEAPON_ON_RELOADED], Role::Emit),
    (&[ids::INSP_FACTORY_ON_SPAWNED], Role::Emit),
    (&[ids::INSP_FACTORY_ON_EXHAUSTED], Role::Emit),
    (&[ids::INSP_LIFE_ON_DEATH], Role::Emit),
    (&[ids::INSP_VIDA_ON_DAMAGE], Role::Emit),
    (&[ids::INSP_VIDA_ON_HEAL], Role::Emit),
    (&[ids::INSP_VIDA_ON_DEATH], Role::Emit),
    (&[ids::INSP_NAV_ON_ARRIVED], Role::Emit),
    (&[ids::INSP_NAV_ON_NO_PATH], Role::Emit),
    (&[ids::INSP_NAV_ON_STUCK], Role::Emit),
    (&[ids::INSP_NAV_LINK_ON_CROSSED], Role::Emit),
    (&[ids::INSP_RAY_ON_ENTER], Role::Emit),
    (&[ids::INSP_RAY_ON_EXIT], Role::Emit),
    (&[ids::INSP_PHYS_SIGNAL], Role::Emit),
    (&[ids::INSP_PHYS_SIGNAL_LEAVE], Role::Emit),
    (&[ids::INSP_SM_STATE_ON_ENTER], Role::Emit),
    (&[ids::INSP_SM_STATE_ON_EXIT], Role::Emit),
    (&[ids::INSP_ANIM_SIGNAL_FINISH], Role::Emit),
    (&[ids::INSP_ANIM_SIGNAL_LOOP], Role::Emit),
    (&ids::INSP_ANIM_MARKER_SIGNAL, Role::Emit),
];

/// O papel da caixa `id`, se ela for uma caixa de sinal.
#[must_use]
pub fn role_of(id: NodeId) -> Option<Role> {
    CAIXAS
        .iter()
        .find(|(ids, _)| ids.contains(&id))
        .map(|(_, r)| *r)
}

/// O texto e a tinta do sufixo de `hint` numa caixa de papel `role`; `None` = sem sufixo.
#[must_use]
pub fn signal_hint(hint: Hint, role: Role) -> Option<Sufixo> {
    let (texto, tinta) = match (hint, role) {
        (Hint::Empty, _) => return None,
        (Hint::Count(n), Role::Emit) => (
            tr_with("panel.inspector.signal.n_listen", &[("n", &n)]),
            ColorToken::Text3,
        ),
        (Hint::Count(n), Role::Listen) => (
            tr_with("panel.inspector.signal.n_send", &[("n", &n)]),
            ColorToken::Text3,
        ),
        (Hint::ScriptMay, Role::Emit) => (
            tr("panel.inspector.signal.script_may_listen").to_owned(),
            ColorToken::Text3,
        ),
        (Hint::ScriptMay, Role::Listen) => (
            tr("panel.inspector.signal.script_may_send").to_owned(),
            ColorToken::Text3,
        ),
        (Hint::Nobody, Role::Emit) => (
            tr("panel.inspector.signal.nobody_listens").to_owned(),
            ColorToken::Warn,
        ),
        (Hint::Nobody, Role::Listen) => (
            tr("panel.inspector.signal.nobody_sends").to_owned(),
            ColorToken::Warn,
        ),
    };
    Some(Sufixo { texto, tinta })
}

/// ⭐ O fornecedor que o pintor da caixa chama: o sufixo da caixa `id` com o nome `name`.
#[must_use]
pub fn fornecedor(id: NodeId, name: &str) -> Option<Sufixo> {
    let role = role_of(id)?;
    let hint = CURRENT_SIGNAL_CENSUS.with(|c| c.borrow().hint(name, role));
    signal_hint(hint, role)
}

/// Liga o fornecedor neste fio (o `paint` do Inspector, cada quadro).
pub(crate) fn instala() {
    field_suffix::instala(Some(fornecedor));
}
