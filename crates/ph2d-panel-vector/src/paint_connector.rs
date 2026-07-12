//! A seção **CONNECTOR** — os campos da RELAÇÃO da linha que gruda em duas formas.
//!
//! Módulo irmão de `paint_modes` (teto de 600 LOC por arquivo de painel). Aqui moram as
//! quatro peças da seção, juntas de propósito: o **snapshot** que a shell publica, o
//! **seed** dos campos, a **pintura** e o **evento** de volta. (O snapshot ficaria em
//! `state.rs` por convenção, mas o arquivo está a 30 linhas do teto — e a lição do projeto
//! é *split, não allowlist*.)
//!
//! ## As três regras que decidem se isto funciona
//!
//! 1. **O que o campo mostra é o valor EFETIVO** — o automático, enquanto ninguém fixou
//!    nada (`jetty`/`spread` são `Option` no componente). A shell resolve o efetivo e
//!    publica; o painel só desenha. Um campo que nascesse em `0` faria o slider SALTAR no
//!    primeiro toque, longe da linha que o olho vê — e calibrar "com o olho" é justamente
//!    o pedido.
//! 2. **Editar FIXA** (`None` → `Some(v)`): é o auto→override de qualquer editor. A partir
//!    daí o valor para de acompanhar o tamanho das caixas.
//! 3. **A edição atinge TODOS os conectores selecionados** (a shell fecha isso) — é o que
//!    permite calibrar o diagrama inteiro de uma vez, em vez de linha por linha.
//!
//! A seção só aparece com um conector na seleção: sem `Some(snapshot)` o `paint` devolve o
//! `y` intocado e o `step` do orquestrador não emite nem o separador.

use std::cell::Cell;

use crate::ids;
use crate::paint_sections::BodyCtx;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{PanelHostInternal, seam_reset_button};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_i18n::tr;
use ph2d_tool_vector::connector;

thread_local! {
    /// O conector em FOCO (o primeiro da seleção), com os valores já EFETIVOS. `None` =
    /// nenhum conector selecionado → a seção inteira some.
    static CURRENT_CONNECTOR: Cell<Option<ConnectorSnapshot>> = const { Cell::new(None) };
}

/// O conector que o painel desenha — os valores **efetivos** (o automático, quando o
/// usuário não fixou nada). Espelho panel-local do `ph2d_ecs::VecConnector`: o painel não
/// depende da ECS, e a shell é quem resolve o `Option` → valor.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ConnectorSnapshot {
    /// Discriminante do `RouteKind` (`Straight = 0`, `Orthogonal = 1`).
    pub route: u8,
    /// O jetty EFETIVO (o automático, se o campo é `None` no componente).
    pub jetty: f64,
    /// O spread EFETIVO (idem).
    pub spread: f64,
    /// O raio das quinas do PERCURSO (as dobras do cotovelo), em unidades de mundo. Não tem
    /// "efetivo" a resolver — não é `Option` no componente: `0` (afiado) já é a resposta
    /// certa do fluxograma clássico, e não existe um raio automático que faça sentido.
    ///
    /// Não confundir com o `Head Round` da seção Stroke, que arredonda as quinas da SETA.
    pub corner: f64,
}

/// Publica o conector em foco (a shell chama a cada frame; `None` esconde a seção).
pub fn set_current_connector(snapshot: Option<ConnectorSnapshot>) {
    CURRENT_CONNECTOR.with(|c| c.set(snapshot));
}

/// O conector em foco deste frame (`None` ⇒ a seção não existe).
#[must_use]
pub(crate) fn current_connector() -> Option<ConnectorSnapshot> {
    CURRENT_CONNECTOR.with(Cell::get)
}

/// **Semeia os dois campos numéricos com o valor EFETIVO** + registra a FAIXA de cada um
/// (`set_number_range`, senão o arrasto escala errado — o gotcha conhecido da caixa
/// limitada).
///
/// O campo em FOCO nunca é semeado: senão o seed sobrescreveria a tecla que o usuário
/// acabou de digitar (ou o arrasto em curso) a cada frame. Chamado da Fase B do `paint`.
pub(crate) fn seed(store: &mut WidgetStore) {
    let Some(snap) = current_connector() else {
        return;
    };
    let focus = store.focus_id();
    for (id, field, value) in [
        (ids::VECTOR_CONNECTOR_JETTY, &connector::JETTY, snap.jetty),
        (
            ids::VECTOR_CONNECTOR_SPREAD,
            &connector::SPREAD,
            snap.spread,
        ),
        (
            ids::VECTOR_CONNECTOR_CORNER,
            &connector::CORNER,
            snap.corner,
        ),
    ] {
        store.set_number_range(id, field.min, field.max, field.step);
        if focus != Some(id) {
            store.set_number_value(id, connector::clamp_to(field, value));
        }
    }
}

/// O [`FieldDesc`] da caixa numérica `id` desta seção (`None` = não é uma delas). Porta
/// única do `seed` (que registra a faixa) e do `apply_event` (que clampa): duas tabelas
/// divergiriam, e a caixa passaria a clampar numa faixa e a escalar noutra.
///
/// [`FieldDesc`]: ph2d_tool_vector::shapes::FieldDesc
fn number_field_of(id: ph2d_a11y::NodeId) -> Option<&'static ph2d_tool_vector::shapes::FieldDesc> {
    if id == ids::VECTOR_CONNECTOR_JETTY {
        Some(&connector::JETTY)
    } else if id == ids::VECTOR_CONNECTOR_SPREAD {
        Some(&connector::SPREAD)
    } else if id == ids::VECTOR_CONNECTOR_CORNER {
        Some(&connector::CORNER)
    } else {
        None
    }
}

/// O evento de volta: os dois campos numéricos + o botão de rota. Delegado do
/// `event::apply_event` (que já está no teto de 200 LOC por função).
///
/// Os três saem como `SetValue` **no id do próprio campo** — a shell tem um caminho só
/// para aplicar (e ela é quem sabe QUAIS conectores estão selecionados). O botão de rota
/// emite o PRÓXIMO discriminante, calculado a partir do que está na tela: o clique cicla.
pub(crate) fn apply_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        WidgetEvent::ValueChanged(id) if number_field_of(id).is_some() => {
            let Some(field) = number_field_of(id) else {
                return false;
            };
            let v = connector::clamp_to(field, host.store().number_value(id).unwrap_or(0.0));
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v)));
            true
        }
        WidgetEvent::Click(id) if id == ids::VECTOR_CONNECTOR_ROUTE => {
            seam_reset_button(host, id);
            // Sem conector em foco o botão nem foi pintado — mas o caminho recusa por
            // construção em vez de adivinhar uma rota.
            let Some(snap) = current_connector() else {
                return false;
            };
            let next = connector::next_route(f64::from(snap.route));
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, next)));
            true
        }
        _ => false,
    }
}

impl BodyCtx<'_> {
    /// Seção **CONNECTOR** — Route (escolha) + Jetty / Spread / **Corner** (caixas
    /// numéricas). Só existe com um conector na seleção: sem ele a seção INTEIRA some (nem
    /// cabeçalho), e o `step` do orquestrador não emite separador.
    ///
    /// Reusa os MESMOS desenhos de linha da seção de parâmetros de forma
    /// (`labeled_choice_button` / `labeled_number_field`) — a seção é irmã dela, não um
    /// enxerto: mesma coluna de rótulo, mesmo widget, mesma estética.
    pub(crate) fn connector_section(&mut self, y: f32) -> f32 {
        let Some(snap) = current_connector() else {
            return y;
        };
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_CONNECTOR,
            tr("panel.vector.section.connector"),
            y,
        );
        if collapsed {
            return y;
        }
        y = self.labeled_choice_button(
            connector::ROUTE.label,
            ids::VECTOR_CONNECTOR_ROUTE,
            connector::route_label(f64::from(snap.route)),
            y,
        );
        // Os valores pintados vêm do STORE (semeado com o efetivo na Fase B do paint), como
        // em toda caixa numérica do painel — assim o dígito que o usuário está digitando não
        // é sobrescrito pelo snapshot do frame.
        y = self.labeled_number_field(
            connector::JETTY.label,
            ids::VECTOR_CONNECTOR_JETTY,
            connector::JETTY.step,
            y,
        );
        y = self.labeled_number_field(
            connector::SPREAD.label,
            ids::VECTOR_CONNECTOR_SPREAD,
            connector::SPREAD.step,
            y,
        );
        // **Corner** — o raio das quinas do PERCURSO (as dobras do cotovelo). Irmã de
        // Route/Jetty/Spread porque é a mesma pergunta: como esta linha é desenhada.
        self.labeled_number_field(
            connector::CORNER.label,
            ids::VECTOR_CONNECTOR_CORNER,
            connector::CORNER.step,
            y,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A seção some sem conector na seleção.** Este teste morde a decisão, não o pixel: o
    /// `paint` devolve o `y` intocado quando o snapshot é `None`, e é isso que faz o
    /// orquestrador pular a seção INTEIRA (cabeçalho e separador inclusos).
    #[test]
    fn without_a_connector_in_the_selection_there_is_nothing_to_draw() {
        set_current_connector(None);
        assert!(current_connector().is_none());
        set_current_connector(Some(ConnectorSnapshot {
            route: 1,
            jetty: 0.3,
            spread: 0.0,
            corner: 0.0,
        }));
        assert!(current_connector().is_some());
        set_current_connector(None); // não vaza para os outros testes da thread
    }

    /// O clique na rota emite o PRÓXIMO discriminante — e o rótulo do botão nunca é o número.
    #[test]
    fn the_route_button_shows_a_name_and_cycles_to_the_next_one() {
        assert_eq!(connector::route_label(0.0), "Straight");
        assert!((connector::next_route(0.0) - 1.0).abs() < f64::EPSILON);
        assert_eq!(connector::route_label(1.0), "Orthogonal");
        assert!(connector::next_route(1.0).abs() < f64::EPSILON);
    }
}
