//! ⭐ **Navegar um quadro grande** (MiroClone W5): os botões do canto (Frames · Busca · Minimapa ·
//! Enquadrar tudo), o minimapa (`M`), o painel de Frames, a busca (`Ctrl+F`) com o filtro por
//! etiqueta, o campo de pôr uma etiqueta nas notas, os ícones de link e as teclas `Alt+1`/`Alt+2`
//! (enquadrar tudo / a selecção — as do Miro).
//!
//! Tudo pintado e despachado AQUI e nos filhos (`board_nav_*`): a shell só compõe (a catraca).

use super::HeroScreen;
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, resolve};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_board_model::{Board, ElementId};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// A facilitação (W7): o cartão do cronómetro e o painel da votação.
#[path = "board_facilitate.rs"]
pub mod facilitate;
#[path = "board_nav_frames.rs"]
mod frames;
#[path = "board_nav_minimap.rs"]
mod minimap;
#[path = "board_nav_search.rs"]
mod search;
/// A equipa e a saída (W8): comentários, versões, apresentação, exportar.
#[path = "board_team.rs"]
pub mod team;

pub(crate) use frames::panel_width as frames_panel_width;
pub use frames::{Part as RowPart, row_node};
pub use minimap::minimap_rect;
pub use search::{board_tags, chip_node, find, open_tag_field};

/// Abre o painel de Frames e o minimapa e enquadra o quadro inteiro na próxima pintura (as cenas
/// de smoke abrem assim).
pub fn show(hero: &mut HeroScreen, frames: bool, minimap: bool) {
    let n = &mut hero.documents.live.nav;
    n.frames_open = frames;
    n.minimap_open = minimap;
    n.fit_pending = true;
}

/// Os botões do canto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavButton {
    /// W8 — apresentar (P1).
    Present,
    /// W8 — a lista dos comentários (K8).
    Comments,
    Frames,
    Search,
    Minimap,
    Fit,
}

const BUTTONS: [NavButton; 6] = [
    NavButton::Present,
    NavButton::Comments,
    NavButton::Frames,
    NavButton::Search,
    NavButton::Minimap,
    NavButton::Fit,
];

/// ⛔ Os saltos diferem uns dos outros nos bits ALTOS (40 a 47) e cada descodificador exige os bits
/// acima do que carrega a zero: medido nos testes (07/10), saltos que só diferiam nos bits 32–33
/// faziam o clique num ícone de link ler-se como uma linha do painel de Frames.
const SALT: u64 = 0x4e61_10b0_a2d0_0000;
const SALT_FRAME_ROW: u64 = 0x4e61_2000_0000_0000;
const SALT_LINK: u64 = 0x4e61_3000_0000_0000;
const SALT_TAG_CHIP: u64 = 0x4e61_4000_0000_0000;

#[must_use]
pub fn button_node(b: NavButton) -> NodeId {
    let i = BUTTONS.iter().position(|x| *x == b).expect("na tabela") as u64;
    NodeId(SALT ^ (i + 1))
}

fn button_of(id: NodeId) -> Option<NavButton> {
    let i = (id.0 ^ SALT).checked_sub(1)?;
    BUTTONS.get(usize::try_from(i).ok()?).copied()
}

pub(crate) fn link_node(id: ElementId) -> NodeId {
    debug_assert!(id.0 >> 32 == 0, "id de elemento além de 32 bits");
    NodeId(SALT_LINK ^ id.0)
}

fn link_of(id: NodeId) -> Option<ElementId> {
    let e = id.0 ^ SALT_LINK;
    (e >> 32 == 0 && e != 0).then_some(ElementId(e))
}

fn register(store: &mut WidgetStore, id: NodeId) {
    if store.button_state(id).is_none() {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// Regista os botões do canto (pintado ⇒ clicável). Chamado pelo `document_tabs::populate`.
pub fn populate(store: &mut WidgetStore) {
    for b in BUTTONS {
        register(store, button_node(b));
        store.set_tooltip(button_node(b), ph2d_i18n::tr(button_key(b)));
    }
    for (id, key) in search::field_buttons() {
        register(store, id);
        store.set_tooltip(id, ph2d_i18n::tr(key));
    }
    facilitate::populate(store);
    team::populate(store);
}

fn button_key(b: NavButton) -> &'static str {
    match b {
        NavButton::Present => "board.nav.present",
        NavButton::Comments => "board.nav.comments",
        NavButton::Frames => "board.nav.frames",
        NavButton::Search => "board.nav.search",
        NavButton::Minimap => "board.nav.minimap",
        NavButton::Fit => "board.nav.fit",
    }
}

/// Os botões do canto, da esquerda para a direita, encostados ao canto de baixo à direita.
#[must_use]
pub fn button_rects(area: Rect) -> Vec<(NavButton, Rect)> {
    let (s, g) = (Spacing::Xl2.px(), ph2d_tokens::control_gap_px());
    let n = BUTTONS.len() as f32;
    let y = area.y + area.h - Spacing::Md.px() - s;
    let x0 = area.x + area.w - Spacing::Md.px() - n * s - (n - 1.0) * g;
    BUTTONS
        .iter()
        .enumerate()
        .map(|(i, b)| (*b, Rect::new(x0 + i as f32 * (s + g), y, s, s)))
        .collect()
}

/// A margem do «enquadrar» (px de ecrã).
fn fit_margin() -> f64 {
    f64::from(Spacing::Xl4.px())
}

/// `Alt+1`: o quadro inteiro à vista.
pub fn fit_all(board: &mut Board, area: [f64; 4]) {
    if let Some(e) = board.doc.extent() {
        board.camera.fit(area, e, fit_margin());
    }
}

/// Enquadra o elemento `id` com o que está à volta dele (três vezes a caixa dele).
pub(crate) fn go_to(board: &mut Board, area: [f64; 4], id: ElementId, context: f64) {
    let Some(el) = board.doc.get(id) else {
        return;
    };
    let [x0, y0, x1, y1] = el.aabb();
    let (cx, cy, hw, hh) = (
        (x0 + x1) / 2.0,
        (y0 + y1) / 2.0,
        (x1 - x0) / 2.0,
        (y1 - y0) / 2.0,
    );
    let bx = [
        cx - hw * context,
        cy - hh * context,
        cx + hw * context,
        cy + hh * context,
    ];
    board.camera.fit(area, bx, fit_margin());
}

/// O resto do caminho até um elemento: a caixa dele e mais duas à volta (o contexto).
const GO_CONTEXT: f64 = 3.0; // LITERAL-PX-OK: três caixas (a do elemento e duas à volta)

/// ⭐ **As teclas da navegação** — antes das do editor: `M` o minimapa, `Ctrl+F` a busca, `Alt+1`
/// enquadrar tudo, `Alt+2` a selecção, `Ctrl+Alt+K` o link. `Some` = tomada.
pub fn key(
    hero: &mut HeroScreen,
    k: super::board_keys::BoardKey,
    mods: ph2d_host::Modifiers,
) -> Option<super::board_keys::KeyOutcome> {
    use super::board_keys::BoardKey;
    let ctrl = mods.ctrl || mods.meta;
    let area = super::board_view::area_of(hero.last_canvas);
    let theme = hero.theme;
    let editing = hero
        .documents
        .live
        .editor
        .as_ref()
        .is_some_and(ph2d_board_edit::Editor::is_editing_text);
    if editing {
        return None;
    }
    if let Some(out) = team::key(hero, k, mods) {
        return Some(out);
    }
    let taken = Some(super::board_keys::KeyOutcome {
        consumed: true,
        copy: None,
    });
    match (k, ctrl, mods.alt) {
        (BoardKey::Char('m'), false, false) => {
            // Com uma nota seleccionada a letra escreve nela (o idioma do Miro).
            let (board, live) = hero.documents.active_parts()?;
            if live
                .editor
                .as_ref()
                .is_some_and(|e| e.would_type_into_note(&board.doc))
            {
                return None;
            }
            let n = &mut hero.documents.live.nav;
            n.minimap_open = !n.minimap_open;
            taken
        }
        (BoardKey::Char('f'), true, false) => {
            search::open(hero);
            taken
        }
        (BoardKey::Char('1'), false, true) => {
            fit_all(hero.documents.active_board_mut()?, area);
            taken
        }
        (BoardKey::Char('2'), false, true) => {
            let (board, live) = hero.documents.active_parts()?;
            let ed = super::board_view::editor(&mut live.editor, theme);
            let f = ed.frame(&board.doc)?;
            let bx = [
                f.center[0] - f.w / 2.0,
                f.center[1] - f.h / 2.0,
                f.center[0] + f.w / 2.0,
                f.center[1] + f.h / 2.0,
            ];
            board.camera.fit(area, bx, fit_margin());
            taken
        }
        (BoardKey::Char('k'), true, true) => {
            let (board, live) = hero.documents.active_parts()?;
            let history = live.histories.entry(board.id).or_default();
            let ed = super::board_view::editor(&mut live.editor, theme);
            ed.organize(&mut board.doc, history, ph2d_board_edit::Command::Link);
            taken
        }
        _ => None,
    }
}

/// ⭐ Os cliques da navegação. Corre no pré-despacho (ids derivados).
pub fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    if search::apply_event(hero, event)
        || frames::apply_event(hero, event)
        || facilitate::apply_event(hero, event)
        || team::apply_event(hero, event)
    {
        return true;
    }
    let (WidgetEvent::Click(id) | WidgetEvent::DoubleClick(id)) = event else {
        return false;
    };
    let area = super::board_view::area_of(hero.last_canvas);
    if let Some(b) = button_of(id) {
        let n = &mut hero.documents.live.nav;
        match b {
            NavButton::Present => team::show::start(hero),
            NavButton::Comments => team::toggle_list(hero),
            NavButton::Frames => n.frames_open = !n.frames_open,
            NavButton::Minimap => n.minimap_open = !n.minimap_open,
            NavButton::Search => {
                if n.search_open {
                    search::close(hero);
                } else {
                    search::open(hero);
                }
            }
            NavButton::Fit => {
                if let Some(board) = hero.documents.active_board_mut() {
                    fit_all(board, area);
                }
            }
        }
        return true;
    }
    if let Some(src) = link_of(id) {
        let theme = hero.theme;
        if let Some((board, live)) = hero.documents.active_parts() {
            let ed = super::board_view::editor(&mut live.editor, theme);
            if let Some(to) = ed.follow_link(&board.doc, src) {
                go_to(board, area, to, GO_CONTEXT);
            }
        }
        return true;
    }
    false
}

/// ⭐ Pinta a navegação do quadro activo, por cima do quadro e debaixo das barras.
pub fn paint(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let theme = hero.theme;
    let a = super::board_view::area_of(area);
    if std::mem::take(&mut hero.documents.live.nav.fit_pending)
        && let Some(board) = hero.documents.active_board_mut()
    {
        fit_all(board, a);
    }
    search::paint_highlight(hero, area, scene, theme);
    paint_links(hero, a, scene, theme);
    minimap::paint(hero, area, scene, theme);
    frames::paint(hero, area, scene, ts, theme);
    facilitate::paint(hero, area, scene, ts);
    team::paint(hero, area, scene, ts);
    search::paint_fields(hero, area, scene, ts, theme);
    let open = {
        let n = &hero.documents.live.nav;
        let list = hero.documents.live.team.list_open;
        [false, list, n.frames_open, n.search_open, n.minimap_open, false]
    };
    let rects = button_rects(area);
    panel(
        scene,
        &rects.iter().map(|(_, r)| *r).collect::<Vec<_>>(),
        theme,
    );
    for ((b, r), on) in rects.into_iter().zip(open) {
        let icon = match b {
            NavButton::Present => IconId::Presentation,
            NavButton::Comments => IconId::MessageSquare,
            NavButton::Frames => IconId::Layers,
            NavButton::Search => IconId::Search,
            NavButton::Minimap => IconId::Grid,
            NavButton::Fit => IconId::Maximize,
        };
        button(
            scene,
            &mut hero.hit_index,
            &hero.store,
            button_node(b),
            icon,
            r,
            on,
            theme,
        );
    }
}

/// O fundo de um grupo de controlos (o das barras do quadro).
pub(crate) fn panel(scene: &mut VectorScene, rects: &[Rect], theme: Theme) {
    let Some(first) = rects.first() else {
        return;
    };
    let (mut x0, mut y0, mut x1, mut y1) = (first.x, first.y, first.x + first.w, first.y + first.h);
    for r in rects {
        x0 = x0.min(r.x);
        y0 = y0.min(r.y);
        x1 = x1.max(r.x + r.w);
        y1 = y1.max(r.y + r.h);
    }
    let p = Spacing::Xs.px();
    let bg = Rect::new(x0 - p, y0 - p, x1 - x0 + 2.0 * p, y1 - y0 + 2.0 * p);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, bg, radius, resolve(ColorToken::BgElev, theme));
}

/// Um botão de ícone (aceso = aberto), registado no índice.
#[allow(clippy::too_many_arguments)]
pub(crate) fn button(
    scene: &mut VectorScene,
    hit: &mut HitIndex,
    store: &WidgetStore,
    id: NodeId,
    icon: IconId,
    r: Rect,
    on: bool,
    theme: Theme,
) {
    let hover = matches!(
        store.button_state(id),
        Some(ButtonState::Hovered | ButtonState::Pressed | ButtonState::Focused)
    );
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    if on {
        fill_rounded_rect(scene, r, radius, resolve(ColorToken::AccentSoft, theme));
    } else if hover {
        fill_rounded_rect(scene, r, radius, resolve(ColorToken::Bg2, theme));
    }
    let fg = resolve(
        if on {
            ColorToken::Accent
        } else {
            ColorToken::Text2
        },
        theme,
    );
    let i = Spacing::Xs.px();
    let inner = Rect::new(r.x + i, r.y + i, r.w - 2.0 * i, r.h - 2.0 * i);
    paint_icon(scene, icon, inner, fg, StrokeToken::Default.px());
    hit.register(id, r);
}

/// ⭐ **O ícone de link** no canto de cima à direita de cada elemento ligado e à vista — clicar
/// nele leva ao destino (o Miro: *«To go to the linked URL - just click the link icon»*).
fn paint_links(hero: &mut HeroScreen, area: [f64; 4], scene: &mut VectorScene, theme: Theme) {
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let rev = board.doc.rev();
    if live.nav.links.as_ref().is_none_or(|(r, _)| *r != rev) {
        let ids = board
            .doc
            .live()
            .filter(|e| e.link().is_some() || e.locked())
            .map(|e| e.id)
            .collect();
        live.nav.links = Some((rev, ids));
    }
    let s = Spacing::Xl.px();
    let far = live.render_cache.detail.frame_title_only_px;
    let mut drawn = Vec::new();
    let mut locks = Vec::new();
    for id in live.nav.links.as_ref().map_or(&[][..], |(_, v)| v) {
        let Some(el) = board.doc.get(*id) else {
            continue;
        };
        // Dentro de um frame visto de longe o elemento não se pinta: o ícone também não.
        let hidden = board.doc.frame_chain(*id).iter().any(|f| {
            board
                .doc
                .get(*f)
                .is_some_and(|f| f.h * board.camera.zoom < far)
        });
        if hidden {
            continue;
        }
        let [x0, y0, x1, _] = el.aabb();
        // O cadeado (só um sinal: destrava-se pela barra ou por Ctrl+Shift+L) no canto esquerdo.
        let inside = |r: &Rect| {
            r.x + r.w >= area[0] as f32
                && r.y + r.h >= area[1] as f32
                && r.x <= (area[0] + area[2]) as f32
                && r.y <= (area[1] + area[3]) as f32
        };
        if el.locked() {
            let p = board.camera.to_screen(area, [x0, y0]);
            let r = Rect::new(p[0] as f32 - s / 2.0, p[1] as f32 - s / 2.0, s, s);
            if inside(&r) {
                locks.push(r);
            }
        }
        if el.link().is_none() {
            continue;
        }
        let p = board.camera.to_screen(area, [x1, y0]);
        let r = Rect::new(p[0] as f32 - s / 2.0, p[1] as f32 - s / 2.0, s, s);
        if !inside(&r) {
            continue;
        }
        drawn.push((*id, r));
    }
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    for r in locks {
        fill_rounded_rect(scene, r, radius, resolve(ColorToken::BgElev, theme));
        let fg = resolve(ColorToken::Text2, theme);
        paint_icon(scene, IconId::Lock, r, fg, StrokeToken::Default.px());
    }
    for (id, r) in drawn {
        register(&mut hero.store, link_node(id));
        let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
        fill_rounded_rect(scene, r, radius, resolve(ColorToken::BgElev, theme));
        let fg = resolve(ColorToken::Accent, theme);
        paint_icon(scene, IconId::Link, r, fg, StrokeToken::Default.px());
        hero.hit_index.register(link_node(id), r);
    }
}

/// O campo de texto `id` em `r`, com o mesmo pintor dos outros campos.
pub(crate) fn paint_field(
    id: NodeId,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
) {
    let Some(InteractiveState::TextInput {
        text,
        caret,
        selection_anchor,
        state,
    }) = store.get(id)
    else {
        return;
    };
    hit.register(id, r);
    let field = crate::widget::TextInput::new(id, "").visual((*state, store.hover_live(id)));
    scene.push_clip(&crate::paint::rect_to_vello(r));
    crate::widget::paint_text_input_with_buffer(
        &field,
        Some(text.as_str()),
        Some(*caret),
        *selection_anchor,
        r,
        scene,
        ts,
        theme,
    );
    scene.pop_layer();
}

/// Abre o campo `id` com `text` todo seleccionado e o teclado nele.
pub(crate) fn open_field(store: &mut WidgetStore, id: NodeId, text: String) {
    let caret = text.len();
    store.register(
        id,
        InteractiveState::TextInput {
            state: crate::widget::TextInputState::Focused,
            text,
            caret,
            selection_anchor: Some(0),
        },
    );
    store.set_focus(Some(id));
    store.mark_cancel_on_escape(id);
}

/// O texto do campo `id`.
pub(crate) fn field_text(store: &WidgetStore, id: NodeId) -> String {
    match store.get(id) {
        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
        _ => String::new(),
    }
}

/// O texto de um campo (os testes da busca).
#[cfg(test)]
pub(crate) fn field_text_for_tests(hero: &HeroScreen, id: NodeId) -> String {
    field_text(&hero.store, id)
}

/// Minimapa: o carregar no quadro, antes do editor. `true` = era no minimapa.
pub fn pointer_down(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    minimap::down(hero, x, y) || team::pointer_down(hero, x, y)
}

/// Minimapa: o arrasto. `true` = consumido.
pub fn pointer_move(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    minimap::drag(hero, x, y)
}

/// Minimapa: o largar. `true` = era dele.
pub fn pointer_up(hero: &mut HeroScreen) -> bool {
    std::mem::take(&mut hero.documents.live.nav.minimap_drag)
}
