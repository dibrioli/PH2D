//! **O GESTO do diagrama da booleana viva** — mover, ligar, selecionar.
//!
//! Máquina da shell, irmã do [`crate::onion_modal`]: o card desenha e o motor resolve, mas quem
//! interpreta um *Down* sobre um círculo é quem sabe o que um `VecPathId` significa.
//!
//! # A DECISÃO está separada do encanamento, e é isso que a torna testável
//!
//! ⚠️ O irmão onion regista, no próprio arquivo de gates, que as suas fns `impl App` *"precisam de
//! uma janela"* e por isso só o smoke as cobre. Aqui a decisão — *o que este ponto, nesta zona,
//! SIGNIFICA* — vive em [`down_action`] e [`up_intents`], que são puras; as fns do `App` são o
//! encanamento por cima delas. É o antídoto da costura não-testada.
//!
//! # Os quatro gestos
//!
//! | gesto | o que acontece |
//! |---|---|
//! | clique no **miolo** de um círculo | **seleciona** a forma no canvas |
//! | arrastar o **miolo** | **move** o círculo no plano |
//! | arrastar do **aro** até outro círculo | **liga** os dois (`from` opera, `to` recebe) |
//! | clique num traço | **gira** a operação · **Shift** corta |
//!
//! ⚠️ **O clique de selecionar não é conforto: é a única porta.** Um operando consumido desenha
//! VAZIO no canvas, e a lei de lá é *"nada desenhado, nada pego"* — sem o diagrama ele fica
//! inalcançável pelo ponteiro (Enio, 2026-08-22: *"só é possível selecionar e mover no canvas uma
//! shape"*).
//!
//! ⚠️ **A rotação NÃO inclui um estado *"sem ligação"*.** Cortar por sobre-rodar seria o gesto mais
//! fácil de fazer por engano: quem quer ir de *Union* a *Subtract* e passa do ponto apagaria a
//! ligação em vez de continuar a rodar.
//!
//! # O acerto lê o rect que o PAINTER publicou
//!
//! ⚠️ `store.bool_graph_drawn()` é a porta única. Recalcular o retângulo a partir do canto pedido
//! repetiria a prisão ao viewport — duas contas que divergem no dia em que uma delas mudar, e o
//! artista clicando ao lado do que vê.

use std::cell::Cell;

use ph2d_editor::ids;
use ph2d_editor::widget::{
    BoolGraphDrag, BoolGraphIntent, BoolGraphView, BoolGraphZone, bool_graph_clamp_to_plane,
    bool_graph_drop_intent, bool_graph_link_at, bool_graph_node_at,
};
use ph2d_editor::zones::Rect;

use crate::App;

thread_local! {
    /// O último ponto do arrasto da banda de título (`None` = não está a arrastar).
    static TITLE_DRAG: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

/// **O que um *Down* no card significa.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DownAction {
    /// A banda de título: começa a mover o card.
    DragTitle,
    /// Um círculo: arma um arrasto (mover o círculo, ou puxar uma ligação do aro).
    Arm(BoolGraphDrag),
    /// Uma ligação: gira a operação, ou corta.
    Intent(BoolGraphIntent),
    /// Caiu no card, mas em nada — engole o ponteiro e não faz mais nada.
    Swallow,
}

/// A decisão de um *Down*, dado o que o índice de acerto disse (`handle`/`body`) e a geometria.
///
/// `None` = o ponteiro não é do card, e quem chama deixa-o seguir.
///
/// ⚠️ O **X** não aparece aqui de propósito: ele já é um `Button` registado e o despacho genérico
/// fecha o card. Interceptá-lo daria duas portas para a mesma coisa.
#[must_use]
pub(crate) fn down_action(
    on_handle: bool,
    on_body: bool,
    rect: Option<Rect>,
    view: &BoolGraphView,
    p: (f32, f32),
    shift: bool,
) -> Option<DownAction> {
    if on_handle {
        return Some(DownAction::DragTitle);
    }
    if !on_body {
        return None;
    }
    let rect = rect?;
    if let Some((i, zone)) = bool_graph_node_at(rect, view, p) {
        return Some(DownAction::Arm(BoolGraphDrag {
            from: view.nodes[i].id,
            link: zone == BoolGraphZone::Ring,
            at: bool_graph_clamp_to_plane(rect, p),
            // ⚠️ Nasce `false`: até o ponteiro se mexer, este gesto ainda pode ser um CLIQUE.
            moved: false,
        }));
    }
    if let Some(k) = bool_graph_link_at(rect, view, p) {
        let l = view.links[k];
        return Some(DownAction::Intent(if shift {
            BoolGraphIntent::Unlink {
                from: l.from,
                to: l.to,
            }
        } else {
            BoolGraphIntent::Link {
                from: l.from,
                to: l.to,
                op: crate::bool_graph_ui::next_op(l.op),
            }
        }));
    }
    Some(DownAction::Swallow)
}

/// **O que um movimento faz com o arrasto em curso.** `None` = nada a atualizar.
///
/// ⚠️ `moved` liga assim que o ponteiro sai do ponto de partida, e nunca volta a desligar: é o que
/// separa um clique de um arrasto, e um gesto que oscilasse entre os dois faria o *Up* significar
/// coisas diferentes conforme o último pixel.
#[must_use]
pub(crate) fn drag_move(
    rect: Option<Rect>,
    drag: BoolGraphDrag,
    p: (f32, f32),
) -> Option<BoolGraphDrag> {
    let at = bool_graph_clamp_to_plane(rect?, p);
    let moved = drag.moved || at != drag.at;
    Some(BoolGraphDrag { at, moved, ..drag })
}

/// **O que um *Up* produz**, dado o arrasto em curso e onde ele foi solto.
///
/// - arrasto do **aro** solto noutro círculo ⇒ uma ligação nova;
/// - arrasto do **miolo** que se mexeu ⇒ a posição nova (UMA escrita, no fim);
/// - arrasto do **miolo** que NÃO se mexeu ⇒ um clique: seleciona a forma no canvas.
#[must_use]
pub(crate) fn up_intents(
    rect: Option<Rect>,
    view: &BoolGraphView,
    drag: BoolGraphDrag,
    p: (f32, f32),
) -> Vec<BoolGraphIntent> {
    if drag.link {
        let Some(rect) = rect else { return Vec::new() };
        let Some((i, _)) = bool_graph_node_at(rect, view, p) else {
            return Vec::new();
        };
        // A operação de uma ligação NOVA: a que as outras já usam, ou `Union`. ⚠️ Herdar é o que
        // faz montar uma rede uniforme custar um arrasto por ligação, em vez de um arrasto MAIS
        // quatro cliques a girar de volta ao mesmo verbo.
        let op = view.links.first().map_or(0, |l| l.op);
        return bool_graph_drop_intent(view, drag.from, view.nodes[i].id, op)
            .into_iter()
            .collect();
    }
    if drag.moved {
        return vec![BoolGraphIntent::Move {
            id: drag.from,
            at: drag.at,
        }];
    }
    vec![BoolGraphIntent::Select { id: drag.from }]
}

impl App {
    /// **Um *Down* dentro do card.** Devolve `true` quando o consome — e ele consome tudo o que cai
    /// no corpo, senão o ponteiro atravessaria para a arte por baixo e o arrasto moveria as FORMAS.
    pub(crate) fn bool_graph_pointer_down(&mut self, px: f32, py: f32, shift: bool) -> bool {
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        let hit = hero.hit_index.hit(px, py);
        let view = hero.store.bool_graph_view().clone();
        let action = down_action(
            hit == Some(ids::VECTOR_BOOL_GRAPH_HANDLE),
            hit == Some(ids::VECTOR_BOOL_GRAPH_BODY),
            hero.store.bool_graph_drawn(),
            &view,
            (px, py),
            shift,
        );
        match action {
            Some(DownAction::DragTitle) => TITLE_DRAG.with(|c| c.set(Some((px, py)))),
            Some(DownAction::Arm(d)) => hero.store.set_bool_graph_dragging(Some(d)),
            Some(DownAction::Intent(i)) => hero.store.push_bool_graph_intent(i),
            Some(DownAction::Swallow) => {}
            None => return false,
        }
        true
    }

    /// **Movimento**: arrasta a banda de título, ou atualiza a pré-visualização do círculo.
    pub(crate) fn bool_graph_pointer_move(&mut self, px: f32, py: f32) -> bool {
        if let Some((lx, ly)) = TITLE_DRAG.with(Cell::get) {
            TITLE_DRAG.with(|c| c.set(Some((px, py))));
            if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                hero.store.move_bool_graph(px - lx, py - ly);
            }
            return true;
        }
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        let Some(drag) = hero.store.bool_graph_dragging() else {
            return false;
        };
        let next = drag_move(hero.store.bool_graph_drawn(), drag, (px, py));
        hero.store.set_bool_graph_dragging(next);
        true
    }

    /// **Um *Up***: fecha o arrasto da banda, e resolve o gesto do diagrama.
    ///
    /// ⚠️ O arrasto é SEMPRE desarmado, mesmo quando o *Up* cai fora de um círculo. Um arrasto que
    /// sobrevive ao botão solto faria o próximo clique em qualquer sítio criar uma ligação que
    /// ninguém pediu.
    pub(crate) fn bool_graph_pointer_up(&mut self, px: f32, py: f32) -> bool {
        TITLE_DRAG.with(|c| c.set(None));
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        let Some(drag) = hero.store.bool_graph_dragging() else {
            return false;
        };
        hero.store.set_bool_graph_dragging(None);
        let view = hero.store.bool_graph_view().clone();
        for i in up_intents(hero.store.bool_graph_drawn(), &view, drag, (px, py)) {
            hero.store.push_bool_graph_intent(i);
        }
        true
    }
}

#[cfg(test)]
#[path = "bool_graph_input_tests.rs"]
mod tests;
