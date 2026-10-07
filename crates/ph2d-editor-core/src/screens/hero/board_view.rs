//! ⭐ **A VISTA de um quadro** (MiroClone) — com uma aba de quadro activa, a área central é dele:
//! a roda dá zoom à volta do cursor, o botão do meio (ou `Espaço`, ou a mão) arrasta a vista, e o
//! botão principal é do editor de formas ([`ph2d_board_edit::Editor`]). Nenhum destes gestos chega
//! à cena.
//!
//! A área é o `last_canvas` que o último `paint_hero_screen` publicou — a MESMA onde
//! [`ph2d_board_render::paint`] desenhou o quadro.

use super::HeroScreen;
use crate::zones::Rect;
use ph2d_board_edit::{Down, Editor, Metrics, Mods, Pointer};
use ph2d_board_model::{Area, Board, Rgba, Style};
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme};

/// Factor de zoom por linha de roda: o MESMO da câmara da cena (`input_dispatch::on_mouse_wheel`).
const WHEEL_ZOOM_PER_LINE: f64 = 0.9; // LITERAL-PX-OK: factor por linha, espelho da roda da cena
const WHEEL_LINE_PX: f64 = 16.0; // LITERAL-PX-OK: px por linha de roda, espelho da roda da cena
/// Janela do duplo-clique — a do despacho (`WidgetStore::record_pointer_down`).
const DOUBLE_CLICK_NS: u128 = 350_000_000;

/// O que a shell traz com cada evento do quadro.
pub struct Input<'a> {
    pub text: &'a mut TextSystem,
    pub mods: Modifiers,
    /// Relógio do evento (ns) — o duplo-clique.
    pub now_ns: u128,
}

/// ⭐ **A área de um quadro** — a de desenho MAIS a faixa da fila de ferramentas por cima dela (num
/// quadro a fila da cena não se pinta, e a faixa vazia mostraria a cena por baixo — foto de 05/10).
#[must_use]
pub fn area(layout: &super::HeroLayout) -> Rect {
    let d = layout.draw_area;
    let t = layout.tool_bar;
    if t.h <= 0.0 || t.w <= 0.0 {
        return d;
    }
    let top = t.y.min(d.y);
    Rect::new(d.x, top, d.w, d.y + d.h - top)
}

pub(crate) fn area_of(r: Rect) -> Area {
    [
        f64::from(r.x),
        f64::from(r.y),
        f64::from(r.w),
        f64::from(r.h),
    ]
}

fn inside(r: Rect, x: f32, y: f32) -> bool {
    x >= r.x && y >= r.y && x < r.x + r.w && y < r.y + r.h
}

/// As medidas de ECRÃ do editor, dos tokens.
#[must_use]
pub fn metrics() -> Metrics {
    Metrics {
        handle: f64::from(Spacing::Sm.px()),
        rotate_offset: f64::from(Spacing::Xl.px()),
        snap: f64::from(Spacing::Sm.px()),
        drag: f64::from(crate::interaction::TAB_DRAG_THRESHOLD_PX),
        click_size: ph2d_board_edit::CLICK_SIZE,
        paste_offset: ph2d_board_edit::PASTE_OFFSET,
        bind: f64::from(Spacing::Lg.px()),
        // Longe da pega de rodar (`Xl` acima do topo): o ponto de cima não lhe rouba o clique.
        dot: f64::from(Spacing::Xl3.px()),
    }
}

/// O estilo de nascença de uma forma: sem preenchimento, contorno e letra na tinta do tema.
#[must_use]
pub fn default_style(theme: Theme) -> Style {
    let c = ColorToken::Text1.resolve(theme);
    let ink = Rgba([c.r, c.g, c.b, c.a]);
    Style::new(None, Some(ink), ink)
}

/// O editor (criado no 1.º uso). Recebe só o CAMPO, para o resto do estado ficar livre.
pub(crate) fn editor(slot: &mut Option<Editor>, theme: Theme) -> &mut Editor {
    slot.get_or_insert_with(|| Editor::new(default_style(theme), metrics()))
}

/// Selecciona `ids` no quadro activo (as cenas de smoke abrem com a selecção à vista).
pub fn select(hero: &mut HeroScreen, ids: impl IntoIterator<Item = ph2d_board_model::ElementId>) {
    let theme = hero.theme;
    if let Some((board, live)) = hero.documents.active_parts() {
        editor(&mut live.editor, theme).select(&board.doc, ids);
    }
}

/// As formas e notas `ids` do quadro activo ajustam a altura ao texto no próximo desenho (uma cena
/// de smoke montada por código não tem o moldador).
pub fn fit_later(
    hero: &mut HeroScreen,
    ids: impl IntoIterator<Item = ph2d_board_model::ElementId>,
) {
    let theme = hero.theme;
    if let Some((_, live)) = hero.documents.active_parts() {
        editor(&mut live.editor, theme).fit_later(ids);
    }
}

/// Ecrã → ponteiro no mundo do quadro.
fn world_pointer(board: &Board, area: Area, x: f32, y: f32, mods: Modifiers) -> Pointer {
    Pointer {
        world: board.camera.to_world(area, [f64::from(x), f64::from(y)]),
        px: 1.0 / board.camera.zoom,
        mods: Mods {
            shift: mods.shift,
            ctrl: mods.ctrl || mods.meta,
            alt: mods.alt,
        },
    }
}

/// O rectângulo do mundo à vista `[x0, y0, x1, y1]`.
fn view_rect(board: &Board, area: Area) -> [f64; 4] {
    let [x, y, w, h] = area;
    let a = board.camera.to_world(area, [x, y]);
    let b = board.camera.to_world(area, [x + w, y + h]);
    [a[0], a[1], b[0], b[1]]
}

/// A roda sobre a área de um quadro activo. `true` = consumida (a cena não a vê).
pub fn wheel(hero: &mut HeroScreen, x: f32, y: f32, dy: f32) -> bool {
    let area = hero.last_canvas;
    if !inside(area, x, y) {
        return false;
    }
    let Some(board) = hero.documents.active_board_mut() else {
        return false;
    };
    let factor = WHEEL_ZOOM_PER_LINE.powf(-f64::from(dy) / WHEEL_LINE_PX);
    board
        .camera
        .zoom_about(area_of(area), [f64::from(x), f64::from(y)], factor);
    true
}

/// Um botão sobre a área de um quadro activo. `on_canvas` = nenhum painel nem controlo por baixo.
/// `true` = consumido.
#[allow(clippy::too_many_arguments)]
pub fn pointer(
    hero: &mut HeroScreen,
    input: Input<'_>,
    kind: PointerKind,
    button: PointerButton,
    x: f32,
    y: f32,
    on_canvas: bool,
) -> bool {
    if hero.documents.active().is_none() {
        return false;
    }
    let area = area_of(hero.last_canvas);
    let theme = hero.theme;
    match kind {
        PointerKind::Down if on_canvas && inside(hero.last_canvas, x, y) => {
            // ⚠️ Tomar o Down antes do despacho herda a obrigação dele de largar o foco: sem isto o
            // campo de renomear uma aba (ou qualquer campo do chrome) ficava com o teclado.
            let arena = bumpalo::Bump::new();
            for e in hero.blur_focus(&arena).to_vec() {
                hero.apply_event(e);
            }
            let space = hero.documents.live.space;
            let Some((board, live)) = hero.documents.active_parts() else {
                return false;
            };
            let pan = |hero: &mut HeroScreen| {
                hero.documents.pan_from = Some([f64::from(x), f64::from(y)]);
                true
            };
            match button {
                PointerButton::Middle => return pan(hero),
                PointerButton::Primary if space => return pan(hero),
                PointerButton::Primary => {}
                _ => return true,
            }
            let p = world_pointer(board, area, x, y, input.mods);
            let view = view_rect(board, area);
            let double = live.last_down.is_some_and(|(t, at)| {
                input.now_ns.saturating_sub(t) < DOUBLE_CLICK_NS
                    && (at[0] - x).hypot(at[1] - y) <= crate::interaction::TAB_DRAG_THRESHOLD_PX
            });
            live.last_down = (!double).then_some((input.now_ns, [x, y]));
            let history = live.histories.entry(board.id).or_default();
            let ed = editor(&mut live.editor, theme);
            if double && ed.double_click(&mut board.doc, history, input.text, p) {
                return true;
            }
            if ed.pointer_down(&mut board.doc, history, input.text, p, view) == Down::Pan {
                return pan(hero);
            }
            true
        }
        PointerKind::Up => {
            if hero.documents.pan_from.take().is_some() {
                return true;
            }
            let Some((board, live)) = hero.documents.active_parts() else {
                return false;
            };
            let p = world_pointer(board, area, x, y, input.mods);
            let history = live.histories.entry(board.id).or_default();
            let Some(ed) = live.editor.as_mut().filter(|e| e.is_busy()) else {
                return false;
            };
            ed.pointer_up(&mut board.doc, history, p);
            true
        }
        _ => false,
    }
}

/// O cursor mexeu-se. `true` = um arrasto do quadro (vista ou gesto) está em curso e consumiu-o.
pub fn pointer_move(hero: &mut HeroScreen, input: Input<'_>, x: f32, y: f32) -> bool {
    if let Some(from) = hero.documents.pan_from {
        let to = [f64::from(x), f64::from(y)];
        if let Some(board) = hero.documents.active_board_mut() {
            board.camera.pan_by_screen(to[0] - from[0], to[1] - from[1]);
        }
        hero.documents.pan_from = Some(to);
        return true;
    }
    let area = area_of(hero.last_canvas);
    let over = inside(hero.last_canvas, x, y);
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    let p = world_pointer(board, area, x, y, input.mods);
    let Some(ed) = live.editor.as_mut() else {
        return false;
    };
    if !ed.is_busy() {
        // Sem botão: só os pontos azuis da forma por baixo (o movimento segue para a interface).
        if over {
            ed.hover(&board.doc, p);
        }
        return false;
    }
    ed.pointer_move(&mut board.doc, input.text, p);
    true
}

#[cfg(test)]
#[path = "board_view_tests.rs"]
mod tests;
