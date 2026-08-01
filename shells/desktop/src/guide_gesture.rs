//! O **gesto das guias** (plano 25 §9, a W6.2): arrastar da régua para criar, arrastar para
//! mover, arrastar de volta para a régua para apagar.
//!
//! # As três decisões
//!
//! **1. Criar e mover são o MESMO caminho.** Um press na régua já EMPURRA a guia para o
//! documento e o gesto continua como um arrasto dela — não há um estado "guia em criação" ao
//! lado do estado "guia sendo movida". A consequência boa cai de graça: soltar sobre a régua
//! apaga, e isso vale tanto para desistir de uma guia nova quanto para descartar uma antiga,
//! sem que nada precise saber qual das duas era.
//!
//! **2. O gesto é TOOL-AGNÓSTICO**, e é o único jeito honesto: a régua é chrome de canvas e
//! aparece com qualquer ferramenta na mão. Um gesto que só respondesse no Vector deixaria uma
//! faixa visível e morta sob o mouse em todo o resto do app.
//!
//! ⚠️ **Mas o CONSUMO ainda é do Vector**: quem encaixa nas guias é o motor de snap vetorial.
//! Levá-las ao gizmo de sprite dos outros modos é wave própria, e está nomeada no handoff em
//! vez de meio-construída.
//!
//! **3. Com as réguas fora, as guias ficam INERTES** — visíveis e magnéticas, mas não
//! agarráveis. É o *lock de guias* que o Illustrator e o Photoshop escondem num booleano de
//! menu: aqui ele é o mesmo interruptor que já se vê na tela, então não há como o estado
//! travado ficar invisível.

use crate::app_state::App;
use ph2d_editor::ruler::{self, RulerAxis};
use ph2d_guides::{Guide, GuideAxis};

/// Tolerância para AGARRAR uma guia já posta, em pixels de tela. O mesmo alcance do ímã de
/// snap: se o ponteiro está perto o bastante para encaixar nela, está perto o bastante para
/// pegá-la.
const GRAB_PX: f64 = 6.0;

/// Um arrasto de guia em curso.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct GuideDrag {
    /// O índice no [`ph2d_guides::GuideSet`]. É a identidade da guia durante o gesto — e é
    /// por isso que a remoção do conjunto preserva a ordem.
    pub(crate) index: usize,
    /// Qual régua a governa (a de cima move uma horizontal, e vice-versa).
    pub(crate) ruler: RulerAxis,
}

/// O que um press DECIDE, dado o canvas resolvido e as guias que existem.
///
/// ⚠️ Puro de propósito — o padrão `hit_plan` desta codebase. A decisão de um gesto tem de
/// ser testável sem janela: no harness headless o `App` nasce sem `gfx`, então uma política
/// que morasse dentro dos métodos abaixo só poderia ser conferida abrindo o app.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum GuidePress {
    /// Nasce uma guia nova, governada por esta régua, nesta coordenada de mundo.
    Spawn(RulerAxis, f64),
    /// Pega a guia `índice`, governada por esta régua.
    Grab(usize, RulerAxis),
    /// Nada a fazer — o press segue o caminho de sempre.
    Pass,
}

/// A política do press. `rulers` é o interruptor das réguas: com ele desligado nada é
/// agarrável (as guias seguem visíveis e magnéticas — é o *lock*).
#[must_use]
pub(crate) fn press_plan(
    view: &ph2d_editor::GridView,
    guides: &ph2d_guides::GuideSet,
    rulers: bool,
    p: (f32, f32),
) -> GuidePress {
    if !rulers {
        return GuidePress::Pass;
    }
    let canvas = view.canvas;
    if let Some(r) = ruler::hit(canvas, p) {
        return GuidePress::Spawn(r, guide_pos_under(view, r, p));
    }
    if !contains(canvas, p) {
        return GuidePress::Pass;
    }
    let world = [
        ruler::world_at(view, p.0, RulerAxis::Top),
        ruler::world_at(view, p.1, RulerAxis::Left),
    ];
    // A tolerância é o MAIOR dos dois eixos: um pixel vale mundos diferentes em x e em y se a
    // projeção não for isotrópica, e agarrar tem de ser tão fácil nas duas réguas.
    let tol = ruler::world_per_px(view, RulerAxis::Top)
        .max(ruler::world_per_px(view, RulerAxis::Left))
        * GRAB_PX;
    match guides.nearest(world, tol) {
        Some(i) => {
            let axis = guides.get(i).map_or(GuideAxis::Vertical, |g| g.axis);
            GuidePress::Grab(i, ruler_for(axis))
        }
        None => GuidePress::Pass,
    }
}

/// Soltar aqui APAGA a guia? Qualquer régua serve — devolver uma horizontal pela faixa da
/// esquerda é um gesto que ninguém faz por engano, e recusá-lo obrigaria o artista a
/// descobrir que a lixeira tem lado.
#[must_use]
pub(crate) fn release_deletes(view: &ph2d_editor::GridView, p: (f32, f32)) -> bool {
    ruler::hit(view.canvas, p).is_some()
}

/// Onde a guia pousa: a coordenada de mundo sob o cursor, ao longo da régua que a governa.
///
/// ⚠️ **A régua e a coordenada são CRUZADAS**, e é a inversão que o `spawns()` existe para
/// tornar explícita: a régua de CIMA cria uma HORIZONTAL — uma linha de `y` constante — e esse
/// `y` vem da posição VERTICAL do cursor, que se lê pela régua da ESQUERDA. Perguntar pela
/// régua de origem daria a coordenada ao longo da linha, que é justamente a que ela não fixa.
#[must_use]
pub(crate) fn guide_pos_under(view: &ph2d_editor::GridView, r: RulerAxis, p: (f32, f32)) -> f64 {
    match r {
        RulerAxis::Top => ruler::world_at(view, p.1, RulerAxis::Left),
        RulerAxis::Left => ruler::world_at(view, p.0, RulerAxis::Top),
    }
}

impl App {
    /// A vista de projeção com o canvas RESOLVIDO — a que o `paint_hero_screen` de fato usou.
    ///
    /// ⚠️ O `hero.grid.view` carrega um canvas de fachada (`0,0,0,0`) porque quem o resolve é o
    /// layout, dentro do paint. Ler aquele valor daria faixas de régua num retângulo vazio e o
    /// gesto nunca dispararia; espelhar a aritmética do layout aqui seria a segunda porta.
    fn ruler_view(&self) -> Option<ph2d_editor::GridView> {
        let hero = self.gfx.as_ref()?.hero_screen.as_ref()?;
        let view = hero.grid.view?;
        Some(ph2d_editor::GridView {
            canvas: hero.last_canvas,
            ..view
        })
    }

    /// As réguas estão à mostra? É o mesmo interruptor que decide se as guias são agarráveis.
    pub(crate) fn rulers_visible(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.view.rulers_visible)
    }

    /// Pen-down: começa um arrasto de guia, se houver um a começar. Devolve `true` quando
    /// consome o evento.
    pub(crate) fn guide_pointer_down(&mut self, x: f32, y: f32) -> bool {
        let rulers = self.rulers_visible();
        let Some(view) = self.ruler_view() else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        match press_plan(&view, &gfx.guides, rulers, (x, y)) {
            GuidePress::Spawn(r, pos) => {
                let index = gfx.guides.push(Guide {
                    axis: r.spawns(),
                    pos,
                });
                self.guide_drag = Some(GuideDrag { index, ruler: r });
                true
            }
            GuidePress::Grab(index, r) => {
                self.guide_drag = Some(GuideDrag { index, ruler: r });
                true
            }
            GuidePress::Pass => false,
        }
    }

    /// Pen-move: leva a guia para debaixo do cursor.
    pub(crate) fn guide_pointer_move(&mut self, x: f32, y: f32) -> bool {
        let Some(drag) = self.guide_drag else {
            return false;
        };
        let Some(view) = self.ruler_view() else {
            return false;
        };
        let pos = guide_pos_under(&view, drag.ruler, (x, y));
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.guides.set_pos(drag.index, pos);
        }
        true
    }

    /// Pen-up: solta a guia — ou a apaga, se o dedo a devolveu à régua.
    pub(crate) fn guide_pointer_up(&mut self, x: f32, y: f32) -> bool {
        let Some(drag) = self.guide_drag.take() else {
            return false;
        };
        let delete = self
            .ruler_view()
            .is_some_and(|v| release_deletes(&v, (x, y)));
        if delete && let Some(gfx) = self.gfx.as_mut() {
            gfx.guides.remove(drag.index);
        }
        true
    }
}

/// Qual régua governa uma guia já existente.
fn ruler_for(axis: GuideAxis) -> RulerAxis {
    match axis {
        GuideAxis::Horizontal => RulerAxis::Top,
        GuideAxis::Vertical => RulerAxis::Left,
    }
}

fn contains(r: ph2d_editor::zones::Rect, p: (f32, f32)) -> bool {
    p.0 >= r.x && p.0 < r.x + r.w && p.1 >= r.y && p.1 < r.y + r.h
}

#[cfg(test)]
#[path = "guide_gesture_tests.rs"]
mod tests;
