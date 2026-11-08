//! ⭐ **O recibo do Add Component** (plano `docs/GameEngineFuncional/25_add_component_plano.md` §2.4, maquete
//! A 4 da Crítica): depois de adicionar uma peça que traz outras, o Inspector diz o que veio e porquê, com
//! **Undo** — o mesmo pedido do botão da barra (`EditorAction::UndoStep`), que desfaz o passo inteiro.
//!
//! ⚠️ **Ele vive enquanto o passo dele é o TOPO do desfazer**: a shell chama [`vigia`] a cada quadro com a
//! profundidade da pilha e a seleção. Outro gesto, um undo ou outro objecto selecionado apagam-no — um
//! «Undo» que desfizesse outro passo mentiria.

use crate::sections::{
    Button, ButtonKind, ColorToken, HitIndex, Rect, TextSystem, Theme, TypeToken, VectorScene,
    WidgetStore, paint_button,
};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::property_row::{abaixo_do_botao, caixa_do_botao};
use std::cell::RefCell;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Recibo {
    entity_bits: u64,
    text: String,
    /// A profundidade do desfazer logo depois do passo dele (`None` = o passo ainda não fechou).
    passo: Option<usize>,
}

thread_local! {
    static RECIBO: RefCell<Option<Recibo>> = const { RefCell::new(None) };
}

/// A shell, depois de anexar: mostra `text` para o objecto `entity_bits`.
pub fn mostra(entity_bits: u64, text: String) {
    RECIBO.with(|r| {
        *r.borrow_mut() = Some(Recibo {
            entity_bits,
            text,
            passo: None,
        });
    });
}

/// A shell, a cada quadro: arma o recibo na profundidade do passo dele e apaga-o quando o topo do desfazer
/// muda ou o objecto deixa de ser o que o Inspector mostra.
pub fn vigia(depth: usize) {
    vigia_com(
        depth,
        crate::state::current_inspector_transform().map(|t| t.entity_bits),
    );
}

fn vigia_com(depth: usize, selected: Option<u64>) {
    RECIBO.with(|r| {
        let mut r = r.borrow_mut();
        let Some(rec) = r.as_mut() else {
            return;
        };
        let passo = *rec.passo.get_or_insert(depth);
        if passo != depth || selected != Some(rec.entity_bits) {
            *r = None;
        }
    });
}

/// O texto do recibo de `entity_bits`, se houver.
#[must_use]
pub fn texto(entity_bits: u64) -> Option<String> {
    RECIBO.with(|r| {
        r.borrow()
            .as_ref()
            .filter(|rec| rec.entity_bits == entity_bits)
            .map(|rec| rec.text.clone())
    })
}

/// O **×** do recibo: fecha-o já (dono 08/11), sem desfazer nada.
pub fn fecha() {
    RECIBO.with(|r| *r.borrow_mut() = None);
}

/// O recibo do objecto que o Inspector mostra agora.
pub(crate) fn do_inspector() -> Option<String> {
    texto(crate::state::current_inspector_transform()?.entity_bits)
}

/// Pinta a frase, o botão **Undo** e, à direita dele, o **×** que fecha; devolve o `y` de baixo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pinta(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    frase: &str,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let y = crate::sections::rows::aviso_em(
        scene,
        text,
        theme,
        x,
        w,
        y,
        frase,
        ColorToken::Success,
        TypeToken::Sm,
    );
    let rotulo = ph2d_i18n::tr("component_add.receipt.undo");
    let caixa: Rect = caixa_do_botao(text, x, w, y, rotulo);
    let lado = caixa.h;
    let gap = ph2d_tokens::control_gap_px();
    let x_fecha = x + w - lado;
    let caixa = Rect::new(caixa.x, caixa.y, caixa.w.min(x_fecha - gap - caixa.x), caixa.h);
    let fecha = Rect::new(x_fecha, caixa.y, lado, lado);
    hit.register(crate::ids::INSP_RECEIPT_CLOSE, fecha);
    let id = crate::ids::INSP_RECEIPT_CLOSE;
    let b = Button::new(id, ph2d_i18n::tr("component_add.receipt.close"))
        .kind(ButtonKind::IconOnly {
            icon: ph2d_editor_core::icons::IconId::Close,
        })
        .visual(store.button_visual(id));
    paint_button(&b, fecha, scene, text, theme);
    hit.register(crate::ids::INSP_RECEIPT_UNDO, caixa);
    let b = Button::new(crate::ids::INSP_RECEIPT_UNDO, rotulo.to_owned())
        .kind(ButtonKind::Default)
        .visual(store.button_visual(crate::ids::INSP_RECEIPT_UNDO));
    paint_button(&b, caixa, scene, text, theme);
    abaixo_do_botao(caixa)
}

/// O clique no **Undo** (o mesmo pedido do botão da barra) ou no **×** (só fecha).
pub(crate) fn clique(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    if do_inspector().is_none() {
        return false;
    }
    if ev == WidgetEvent::Click(crate::ids::INSP_RECEIPT_CLOSE) {
        fecha();
        return true;
    }
    if ev != WidgetEvent::Click(crate::ids::INSP_RECEIPT_UNDO) {
        return false;
    }
    host.bus_mut().push(EditorAction::UndoStep { redo: false });
    true
}

#[cfg(test)]
#[path = "recibo_tests.rs"]
mod tests;
