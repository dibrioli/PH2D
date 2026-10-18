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
use ph2d_tokens::{Spacing, Theme};

/// Factor de zoom por linha de roda: o MESMO da câmara da cena (`input_dispatch::on_mouse_wheel`).
const WHEEL_ZOOM_PER_LINE: f64 = 0.9; // LITERAL-PX-OK: factor por linha, espelho da roda da cena
const WHEEL_LINE_PX: f64 = 16.0; // LITERAL-PX-OK: px por linha de roda, espelho da roda da cena
/// Janela do duplo-clique — a do despacho (`WidgetStore::record_pointer_down`).
const DOUBLE_CLICK_NS: u128 = crate::interaction::DOUBLE_CLICK_WINDOW_NS;

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
        // 8 px de ecrã em qualquer zoom: medido no Excalidraw 0.18.1 e no tldraw 5.5.2 (W5b,
        // `docs/MiroClone/03_guias_e_alinhamento.md` §3).
        snap: f64::from(Spacing::Md.px()),
        drag: f64::from(crate::interaction::TAB_DRAG_THRESHOLD_PX),
        click_size: ph2d_board_edit::CLICK_SIZE,
        paste_offset: ph2d_board_edit::PASTE_OFFSET,
        bind: f64::from(Spacing::Lg.px()),
        // Longe da pega de rodar (`Xl` acima do topo): o ponto de cima não lhe rouba o clique.
        dot: f64::from(Spacing::Xl3.px()),
        eraser: f64::from(Spacing::Md.px()),
        frame_title: f64::from(Spacing::Xl.px()),
    }
}

/// O estilo de nascença de uma forma: sem preenchimento, contorno e letra na TINTA DO DOCUMENTO
/// ([`ph2d_board_model::DEFAULT_INK`]) — nunca a do tema em que nasceu: quem desenha resolve-a pelo
/// tema de cada momento ([`ph2d_board_render::ThemeInk`]; smoke do dono, 07/10).
#[must_use]
pub fn default_style() -> Style {
    let ink = Rgba(ph2d_board_model::DEFAULT_INK);
    Style::new(None, Some(ink), ink)
}

/// O editor (criado no 1.º uso). Recebe só o CAMPO, para o resto do estado ficar livre.
pub(crate) fn editor(slot: &mut Option<Editor>, theme: Theme) -> &mut Editor {
    slot.get_or_insert_with(|| {
        let mut ed = Editor::new(default_style(), metrics());
        ed.pen = super::board_bar::pen_box(theme);
        ed.frame_name = |n| ph2d_i18n::tr_with("board.frame.default_name", &[("n", &n)]);
        ed
    })
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

/// Põe a ferramenta `tool` na mão no quadro activo (a cena de smoke da caneta abre com ela).
pub fn set_tool(hero: &mut HeroScreen, tool: ph2d_board_edit::Tool) {
    let theme = hero.theme;
    if let Some((_, live)) = hero.documents.active_parts() {
        editor(&mut live.editor, theme).tool = tool;
    }
}

pub use ph2d_board_edit::BoardCursor;

/// W23 B5 — o cursor que o quadro pede em `(x, y)` (ecrã): sobre uma pega da selecção, a seta do eixo
/// dela ou a de rodar. A janela traduz para o cursor do sistema.
#[must_use]
pub fn cursor(hero: &HeroScreen, x: f32, y: f32) -> Option<BoardCursor> {
    if hero.documents.surface_active().is_some() || !inside(hero.last_canvas, x, y) {
        return None;
    }
    let board = hero.documents.active_board()?;
    let ed = hero.documents.live.editor.as_ref()?;
    let p = world_pointer(board, area_of(hero.last_canvas), x, y, Modifiers::default());
    ed.cursor_at(&board.doc, p)
}

/// Ecrã → ponteiro no mundo do quadro.
pub(crate) fn world_pointer(board: &Board, area: Area, x: f32, y: f32, mods: Modifiers) -> Pointer {
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

/// W23 B5 — o que a roda e o trackpad pedem ao quadro (o idioma do Miro).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoardScroll {
    /// A roda do rato (px, 16 por linha): zoom no ponteiro.
    Wheel(f32),
    /// Dois dedos no trackpad (px): a vista anda com eles.
    Pan([f32; 2]),
    /// A pinça (o factor de escala): zoom no ponteiro.
    Pinch(f64),
}

/// A roda sobre a área de um quadro activo. `true` = consumida (a cena não a vê).
pub fn wheel(hero: &mut HeroScreen, x: f32, y: f32, dy: f32) -> bool {
    scroll(hero, x, y, BoardScroll::Wheel(dy))
}

/// ⭐ A roda, os dois dedos e a pinça sobre a área de um quadro activo. `true` = consumida.
pub fn scroll(hero: &mut HeroScreen, x: f32, y: f32, s: BoardScroll) -> bool {
    let dy = match s {
        BoardScroll::Wheel(dy) | BoardScroll::Pan([_, dy]) => dy,
        BoardScroll::Pinch(_) => 0.0,
    };
    if let Some(taken) = super::surface_input::wheel(hero, x, y, dy) {
        return taken;
    }
    // W8 — durante a apresentação a vista é a da viagem: a roda não a mexe (W19: nem no ecrã inteiro do
    // vídeo, que tapa o quadro).
    if super::board_nav::team::show::showing(hero) || super::board_video::fullscreen(hero) {
        return true;
    }
    let area = hero.last_canvas;
    if !inside(area, x, y) {
        return false;
    }
    if hero.documents.active().is_none() {
        return false;
    }
    if !matches!(s, BoardScroll::Pinch(_)) && super::board_nav::facilitate::wheel(hero, x, y, dy) {
        return true;
    }
    // c21 — o MEU zoom pára de seguir.
    super::board_nav::team::live_board::unfollow(hero);
    let Some(board) = hero.documents.active_board_mut() else {
        return false;
    };
    let about = [f64::from(x), f64::from(y)];
    match s {
        BoardScroll::Wheel(dy) => {
            let factor = WHEEL_ZOOM_PER_LINE.powf(-f64::from(dy) / WHEEL_LINE_PX);
            board.camera.zoom_about(area_of(area), about, factor);
        }
        BoardScroll::Pinch(factor) => board.camera.zoom_about(area_of(area), about, factor),
        BoardScroll::Pan([dx, dy]) => {
            let z = board.camera.zoom;
            board.camera.center_x -= f64::from(dx) / z;
            board.camera.center_y -= f64::from(dy) / z;
        }
    }
    true
}

/// O carregar no quadro que é do editor (ou do pan). Também o de um carregar num vídeo com outra ferramenta
/// na mão, entregue depois, quando o gesto deixou de ser um clique (`board_video::deferred_*`, W21).
fn canvas_down(
    hero: &mut HeroScreen,
    input: Input<'_>,
    button: PointerButton,
    x: f32,
    y: f32,
) -> bool {
    let area = area_of(hero.last_canvas);
    let theme = hero.theme;
    // Um clique no quadro fecha a galeria dos modelos (W6).
    hero.documents.live.templates_open = false;
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
        // W23 B5 — o botão direito numa forma abre o menu dela.
        PointerButton::Secondary => {
            super::board_menu::open(hero, x, y);
            return true;
        }
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
    // Um gesto pendurado fecha-se antes de o novo começar.
    ed.finish_gesture(&mut board.doc, history);
    // O que nasce num quadro em rascunho nasce à mão (W4).
    ed.style.sketch = board.sketch;
    if double && ed.double_click(&mut board.doc, history, input.text, p) {
        live.gesture = ed.is_busy();
        return true;
    }
    if ed.pointer_down(&mut board.doc, history, input.text, p, view) == Down::Pan {
        live.gesture = false;
        return pan(hero);
    }
    live.gesture = ed.is_busy();
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
    if hero.documents.surface_active().is_some() {
        // Um carregar sobre o chrome desenhado por cima da aba (o menu File, um popover, a paleta)
        // é do chrome; o largar segue a lei do quadro: só é da aba se o carregar foi.
        if matches!(kind, PointerKind::Down) && !on_canvas {
            return false;
        }
        return super::surface_input::pointer(hero, input, kind, button, x, y).unwrap_or(false);
    }
    if hero.documents.active().is_none() {
        return false;
    }
    let area = area_of(hero.last_canvas);
    // c21 — carregar no quadro (clicar, arrastar, pan com o meio ou o espaço) pára de seguir.
    if matches!(kind, PointerKind::Down) && on_canvas && inside(hero.last_canvas, x, y) {
        super::board_nav::team::live_board::unfollow(hero);
    }
    match kind {
        // K2 — uma bolha de comentário (um controlo, não o quadro) arrasta-se para outro sítio.
        PointerKind::Down if super::board_nav::team::bubble_down(hero, x, y, button) => true,
        PointerKind::Down
            if on_canvas && super::board_nav::pointer_down(hero, x, y, button, input.now_ns) =>
        {
            true
        }
        PointerKind::Down if on_canvas && inside(hero.last_canvas, x, y) => {
            canvas_down(hero, input, button, x, y)
        }
        PointerKind::Down => {
            // Um carregar FORA do quadro (um botão da barra, um painel): um gesto pendurado fecha-se
            // aqui — senão o largar deste clique ia para ele e o botão nunca recebia o clique.
            if let Some((board, live)) = hero.documents.active_parts()
                && let Some(ed) = live.editor.as_mut()
                && ed.is_busy()
            {
                let history = live.histories.entry(board.id).or_default();
                ed.finish_gesture(&mut board.doc, history);
                live.gesture = false;
            }
            false
        }
        PointerKind::Up => {
            if hero.documents.pan_from.take().is_some() {
                return true;
            }
            // W21 — segurado parado num vídeo com outra ferramenta: não foi clique, é da ferramenta.
            if let Some([ax, ay]) = super::board_video::deferred_hold(hero, input.now_ns) {
                let replay = Input {
                    text: &mut *input.text,
                    mods: input.mods,
                    now_ns: input.now_ns,
                };
                canvas_down(hero, replay, PointerButton::Primary, ax as f32, ay as f32);
            } else if super::board_nav::pointer_up(hero, x, y, input.now_ns) {
                return true;
            }
            let Some((board, live)) = hero.documents.active_parts() else {
                return false;
            };
            let p = world_pointer(board, area, x, y, input.mods);
            let history = live.histories.entry(board.id).or_default();
            // O largar só é do quadro se o carregar foi.
            if !std::mem::take(&mut live.gesture) {
                return false;
            }
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
    if hero.documents.surface_active().is_some() {
        return super::surface_input::pointer_move(hero, input, x, y).unwrap_or(false);
    }
    // W11 — o meu cursor no quadro da sala (a família manda-o aos outros).
    super::board_nav::team::live_board::cursor(hero, x, y);
    // W21 — um carregar num vídeo com outra ferramenta virou arrasto: a ferramenta começa onde carregou.
    if let Some([ax, ay]) = super::board_video::deferred_drag(hero, x, y) {
        let replay = Input {
            text: &mut *input.text,
            mods: input.mods,
            now_ns: input.now_ns,
        };
        canvas_down(hero, replay, PointerButton::Primary, ax as f32, ay as f32);
    }
    if super::board_nav::pointer_move(hero, x, y) {
        return true;
    }
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
    let gesture = live.gesture;
    let Some(ed) = live.editor.as_mut() else {
        return false;
    };
    if !ed.is_busy() || !gesture {
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

/// W23 B5 — o rascunho EM MASSA vazio diz o que fazer: a frase onde a escrita começa, na tinta da
/// nota a meia força (some à 1.ª letra).
pub(crate) fn paint_bulk_hint(
    board: &Board,
    ed: &Editor,
    area: Area,
    text: Option<&ph2d_board_edit::TextOverlay>,
    scene: &mut ph2d_vector::VectorScene,
    ts: &mut TextSystem,
) {
    let Some(t) = text.filter(|t| ed.bulk_draft() == Some(t.element)) else {
        return;
    };
    let Some(el) = board.doc.get(t.element) else {
        return;
    };
    if el.shape().is_none_or(|s| !s.text.is_empty()) {
        return;
    }
    let z = board.camera.zoom;
    let [x, y] = board
        .camera
        .to_screen(area, [el.x + t.origin[0], el.y + t.origin[1]]);
    let w = (el.w - 2.0 * t.origin[0]).max(0.0) * z;
    let Rgba([r, g, b, a]) = el.style().text_color;
    let ink = ph2d_vector::Color::from_rgba8(r, g, b, a / 2);
    let font = (el.style().font_size * z) as f32;
    crate::paint::paint_text_block(
        ts,
        scene,
        ph2d_i18n::tr("board.note.bulk_hint"),
        x as f32,
        y as f32,
        font,
        w as f32,
        ink,
    );
}
