// ph2d-chrome-sync:z=73 (dispatch priority, ADR-0107; lower = earlier)
//! **O pill UI** — o abridor visível do painel que o ARTISTA desenhou (plano UI/UX W8b.3).
//!
//! ⚠️ Terceiro irmão do [`super::physics_toggle`] e do [`super::tokens_toggle`], e a queixa que o
//! justifica é a MESMA que o Enio já fez duas vezes (física em 2026-07-27, tokens em 2026-08-04):
//! *"onde fica o botão para abrir esse painel?"*. Aqui o abridor que existia era o chip **Show as
//! Panel** da seção Frame — e ele exige a ferramenta Vector em mãos **e** a moldura selecionada.
//! Fechar pelo X e perder a seleção deixava o artista sem caminho de volta que não fosse
//! re-encontrar a própria moldura na Hierarquia: uma feature que só existe para quem já sabe que
//! ela existe.
//!
//! ⚠️ **É a MESMA visibilidade que o chip escreve** (`panel_visibility` com a chave do painel), e
//! não um segundo estado que precisa concordar com ela — os TRÊS abridores (o chip, este pill e o
//! X do próprio painel) escrevem um fato só. O estado do botão é **DERIVADO** do mesmo bool, pelo
//! mesmo motivo: um bool próprio é como o pill passa a dizer *fechado* sobre um painel aberto pelo
//! chip.
//!
//! # Por que o rótulo é `UI`, e não o título do painel
//!
//! ⚠️ O painel gerado carrega o título que o **artista** digitou (hoje `"Color"`), e a lista de
//! clusters do topbar é `const`. Um pill chamado *"Color"* seria uma segunda cópia de um literal
//! que vive em código GERADO, num sítio que não pode acompanhá-lo — ele mentiria no instante em
//! que o artista renomeasse a moldura. O pill nomeia a ESPÉCIE (*a UI que você desenhou*); quem
//! mostra o título é a barra do próprio painel, que o lê da tabela.
//!
//! ⚠️ **A chave é um literal aqui de propósito, e não por descuido:** o chrome não pode depender de
//! `ph2d-panel-authored` (os painéis dependem desta crate, nunca o contrário), então a única
//! resposta possível é a mesma que o `tokens_toggle` dá. O que impede as duas de divergirem é um
//! gate na SHELL, que alcança os dois lados e dirige o pill até a porta do painel
//! (`the_ui_pill_opens_the_authored_panel`).

use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

/// A chave de visibilidade do painel autorado — o `Panel::ID` dele.
///
/// ⚠️ Ver o § do cabeçalho: o literal é obrigatório nesta camada, e quem o mantém honesto é o gate
/// da shell, que compara este caminho com `ph2d_panel_authored::visibility_key()`.
const AUTHORED: &str = "authored";

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id != ids::TOPBAR_AUTHORED {
        return false;
    }
    let visible = !hero.is_panel_visible(AUTHORED);
    hero.panel_visibility.insert(AUTHORED, visible);
    if let Some(InteractiveState::Button { state }) = hero.store.get_mut(ids::TOPBAR_AUTHORED) {
        *state = if visible {
            ButtonState::Pressed
        } else {
            ButtonState::Normal
        };
    }
    true
}
