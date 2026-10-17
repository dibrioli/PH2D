//! ⭐⭐ **QUE ABA do painel está aberta** — «Drawing» ou «Interface» (F17, decisão do dono de 16/10,
//! `docs/UI_New_and_Simple/vetor_nova_ui/28_plano_F17_abas_desenho_interface.md` §2).
//!
//! ⚠️ **Uma porta de escrita, três portas de clique**: as duas abas do título e o interruptor da
//! coluna ① chegam todos ao `apply_event` do painel e acabam em [`set_aba`]. Leitores: o plano do
//! corpo, a face das abas, o `on` do interruptor e o tinto do palco (`ToolBars::stage_tint`). É uma
//! VISTA (D3), não um `DrawMode`: não muda o que o clique no canvas faz. Sessão; nasce em Drawing.

use std::cell::Cell;

/// A aba do painel do vetor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Aba {
    /// A geometria, a cor e os efeitos da forma.
    #[default]
    Drawing,
    /// Componente, moldura, layout, âncoras, pele e estados — a criação de UI.
    Interface,
}

thread_local! {
    static ABA: Cell<Aba> = const { Cell::new(Aba::Drawing) };
}

/// A aba aberta agora.
#[must_use]
pub fn aba() -> Aba {
    ABA.with(Cell::get)
}

/// Abre `aba`. A ÚNICA escrita.
pub(crate) fn set_aba(aba: Aba) {
    ABA.with(|c| c.set(aba));
}

/// ⭐ **A seleção tem algo que só a aba Interface edita** — a marca na aba quando a aberta é a
/// outra (NN/g: o que vive só num modo esconde-se). Lê só o que a shell já publicou por quadro.
///
/// ⚠️ Conta o que a seleção JÁ É, nunca o que ela pode vir a ser: a Component oferece *Create* a
/// qualquer forma e a States tem hospedeiro em qualquer forma — contá-las acenderia a marca sempre.
#[must_use]
pub fn seleccao_tem_interface() -> bool {
    let componente = super::component_state().is_some_and(|c| c.is_main || c.is_instance);
    let pele = super::widget_skin_state().is_some_and(|w| w.selected.is_some() || w.unknown);
    let estados =
        super::ui_states_state().is_some_and(|s| s.host.is_some() && s.recorded.contains(&true));
    let morph = super::morph_states_state().is_some_and(|m| !m.rows.is_empty());
    componente
        || pele
        || estados
        || morph
        || super::frame_present()
        || super::layout_item().is_some()
}
