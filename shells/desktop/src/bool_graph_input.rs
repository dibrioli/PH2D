//! **O GESTO do diagrama da booleana viva** — arrastar entre círculos, e clicar numa ligação.
//!
//! Máquina da shell, irmã do [`crate::onion_modal`]: o card desenha e o motor resolve, mas quem
//! interpreta um *Down* sobre um círculo é quem sabe o que um `VecPathId` significa.
//!
//! # A DECISÃO está separada do encanamento, e é isso que a torna testável
//!
//! ⚠️ O irmão onion regista, no próprio arquivo de gates, que as suas fns `impl App` *"precisam de
//! uma janela"* e por isso só o smoke as cobre. Aqui a decisão — *o que este ponto, com este
//! modificador, SIGNIFICA* — vive em [`down_action`] e [`up_intent`], que são puras; as fns do
//! `App` são o encanamento por cima delas. É o antídoto da costura não-testada: o que decide é
//! exercitável sem abrir o app.
//!
//! # As três coisas que um ponteiro pode fazer no card
//!
//! | gesto | o que acontece |
//! |---|---|
//! | *Down* num círculo → *Up* noutro | **liga** os dois (`from` opera, `to` recebe) |
//! | clique numa ligação | **gira** a operação dela entre as quatro de conjunto |
//! | **Shift**+clique numa ligação | **corta** a ligação |
//!
//! ⚠️ **A rotação NÃO inclui um estado *"sem ligação"*.** Cortar por sobre-rodar seria o gesto mais
//! fácil de fazer por engano: quem quer ir de *Union* a *Subtract* e passa do ponto apagaria a
//! ligação em vez de continuar a rodar. O corte tem gesto próprio, e a dica no rodapé do card
//! nomeia os dois.
//!
//! # O acerto lê o rect que o PAINTER publicou
//!
//! ⚠️ `store.bool_graph_drawn()` é a porta única. Recalcular o retângulo a partir do canto pedido
//! repetiria a prisão ao viewport — duas contas que divergem no dia em que uma delas mudar, e o
//! artista clicando ao lado do que vê.

use std::cell::Cell;

use ph2d_editor::ids;
use ph2d_editor::widget::{
    BoolGraphIntent, BoolGraphView, bool_graph_drop_intent, bool_graph_link_at, bool_graph_node_at,
};
use ph2d_editor::zones::Rect;

use crate::App;

thread_local! {
    /// O último ponto do arrasto da banda de título (`None` = não está a arrastar).
    static TITLE_DRAG: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

/// **O que um *Down* no card significa.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DownAction {
    /// A banda de título: começa a mover o card.
    DragTitle,
    /// Um círculo: arma um arrasto de ligação a partir desta forma.
    ArmLink(u64),
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
    if let Some(i) = bool_graph_node_at(rect, view, p) {
        return Some(DownAction::ArmLink(view.nodes[i].id));
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

/// A decisão de um *Up*, tendo o arrasto partido de `from`. `None` = soltou em nada.
///
/// A operação de uma ligação NOVA é a que as outras já usam, ou `Union`. ⚠️ Herdar é o que faz
/// montar uma rede uniforme custar **um arrasto por ligação** em vez de um arrasto MAIS quatro
/// cliques a girar de volta ao mesmo verbo.
#[must_use]
pub(crate) fn up_intent(
    rect: Option<Rect>,
    view: &BoolGraphView,
    from: u64,
    p: (f32, f32),
) -> Option<BoolGraphIntent> {
    let i = bool_graph_node_at(rect?, view, p)?;
    let op = view.links.first().map_or(0, |l| l.op);
    bool_graph_drop_intent(view, from, view.nodes[i].id, op)
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
            Some(DownAction::ArmLink(from)) => hero.store.set_bool_graph_dragging(Some(from)),
            Some(DownAction::Intent(i)) => hero.store.push_bool_graph_intent(i),
            Some(DownAction::Swallow) => {}
            None => return false,
        }
        true
    }

    /// **Movimento**: arrasta a banda de título. Devolve `true` enquanto arrasta.
    ///
    /// ⚠️ O arrasto de LIGAÇÃO não precisa de nada aqui — ele só se decide no *Up*. Desenhar a
    /// linha elástica é a única coisa que este gesto não mostra, e a ausência fica registada em vez
    /// de fingir que não existe.
    pub(crate) fn bool_graph_pointer_move(&mut self, px: f32, py: f32) -> bool {
        let Some((lx, ly)) = TITLE_DRAG.with(Cell::get) else {
            return false;
        };
        TITLE_DRAG.with(|c| c.set(Some((px, py))));
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.move_bool_graph(px - lx, py - ly);
        }
        true
    }

    /// **Um *Up***: fecha o arrasto da banda, e resolve o arrasto de ligação.
    ///
    /// ⚠️ O arrasto é SEMPRE desarmado, mesmo quando o *Up* cai fora de um círculo. Um arrasto que
    /// sobrevive ao botão solto faria o próximo clique em qualquer sítio criar uma ligação que
    /// ninguém pediu.
    pub(crate) fn bool_graph_pointer_up(&mut self, px: f32, py: f32) -> bool {
        TITLE_DRAG.with(|c| c.set(None));
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        let Some(from) = hero.store.bool_graph_dragging() else {
            return false;
        };
        hero.store.set_bool_graph_dragging(None);
        let view = hero.store.bool_graph_view().clone();
        if let Some(i) = up_intent(hero.store.bool_graph_drawn(), &view, from, (px, py)) {
            hero.store.push_bool_graph_intent(i);
        }
        true
    }
}

#[cfg(test)]
#[path = "bool_graph_input_tests.rs"]
mod tests;
