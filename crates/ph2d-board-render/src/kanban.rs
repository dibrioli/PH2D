//! W4 do Kanban ([ADR-0195] §3) — **um Kanban no canvas**. O documento vive no `KanbanSet` do
//! projecto ([`install_kanbans`]); quem o pinta por inteiro e lhe trata o rato e as teclas é a família
//! Kanban, por um GANCHO que a camada de cima instala ([`install_kanban_painter`]; a entrada vive no
//! editor, `board_kanban.rs`) — o Quadro não depende da família. Sem gancho (ou se ele recusar), este
//! módulo pinta o NÍVEL BAIXO: a moldura com o fundo do Kanban, o título e uma coluna da cor de cada
//! lista com o título e a contagem de cartões.
//!
//! O gancho e a fonte são do FIO do desenho (`thread_local`): o `KanbanSet` é um `Rc` (o editor e a aba
//! partilham-no), e o export cria o seu próprio [`crate::RenderCache`].
//!
//! [ADR-0195]: ../../../docs/architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use ph2d_board_geom::to_world;
use ph2d_board_model::{BoardKanban, Element, ElementId, KanbanBoardView, KanbanId};
use ph2d_kanban_model::{Background, Hue, KanbanSet, Shade};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Color, Point, Rect, RoundedRect, Shape as _, VectorScene, VelloBlend};

use crate::{GREEK_ALPHA, READ_PX, RenderCache, token};

/// Os Kanbans do projecto, partilhados com o editor e a aba.
pub type KanbanSource = Rc<RefCell<KanbanSet>>;

/// A geometria da aba do Kanban a zoom 1 (`ph2d-app-kanban/src/metrics.rs`), em unidades do Kanban:
/// o nível baixo tem a MESMA silhueta que o Kanban inteiro (trocar de nível não salta).
pub const HEADER_H: f64 = 52.0;
const HEADER_PAD: f64 = 16.0;
const TITLE_PX: f64 = 18.0;
const BOARD_PAD: f64 = 12.0;
const LIST_GAP: f64 = 12.0;
const LIST_W: f64 = 272.0;
const COLLAPSED_W: f64 = 44.0;
const LIST_RADIUS: f64 = 12.0;
const LIST_HEAD: f64 = 36.0;
const LIST_PAD: f64 = 14.0;
const LIST_TITLE_PX: f64 = 15.0;
const CARD_STEP: f64 = 44.0;
const LIST_FOOT: f64 = 44.0;
/// O texto de um cartão na aba (px a zoom 1).
const CARD_TEXT_PX: f64 = 14.0;
/// A altura de uma linha de texto por px de letra.
const LINE_K: f64 = 1.3;
/// O traço de leitura de um texto que não se lê: a largura de uma letra e a faixa da linha que ocupa.
const GREEK_ADVANCE: f64 = 0.55;
const GREEK_BAND: [f64; 2] = [0.35, 0.65];

/// Quanto do Kanban se mostra, pelos px de ECRÃ de uma unidade do Kanban (zoom da câmara × da vista).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KanbanLod {
    /// Blocos de cor por lista e a contagem (o título da lista já não se lê).
    Low,
    /// Títulos e cartões só com título/capa (o texto do cartão não se lê).
    Mid,
    /// O Kanban inteiro, interactivo.
    High,
}

impl KanbanLod {
    /// O nível a `scale` px por unidade: baixo onde o título de lista fica abaixo do limiar de leitura
    /// do quadro ([`READ_PX`], medido 06/10); alto onde o texto de um cartão chega à menor letra da
    /// interface (`TypeToken::Xs`).
    #[must_use]
    pub fn at(scale: f64) -> Self {
        if LIST_TITLE_PX * scale < READ_PX {
            Self::Low
        } else if CARD_TEXT_PX * scale < f64::from(TypeToken::Xs.px()) {
            Self::Mid
        } else {
            Self::High
        }
    }
}

/// O que o gancho recebe para pintar um Kanban na moldura de um elemento.
pub struct KanbanPaint<'a> {
    pub scene: &'a mut VectorScene,
    pub ts: &'a mut TextSystem,
    pub theme: Theme,
    /// Local (`0..w × 0..h` do elemento, em unidades do mundo) → ecrã.
    pub to_screen: Affine,
    /// A moldura em coordenadas locais (o gancho recorta-se a ela; o board-render já recortou).
    pub frame: Rect,
    pub lod: KanbanLod,
    pub element: ElementId,
    pub kanban: KanbanId,
    /// A vista deste elemento (muda-se no ponteiro e nas teclas, nunca no desenho).
    pub view: &'a KanbanBoardView,
    pub set: &'a KanbanSet,
    /// A desenhar um export (sem estado de interacção: sem hover, sem cursor).
    pub export: bool,
}

/// O que o gancho recebe para pintar POR CIMA do quadro inteiro (o verso de um cartão do Kanban activo:
/// o modal do Trello, em ecrã, centrado na área do quadro).
pub struct KanbanOver<'a> {
    pub scene: &'a mut VectorScene,
    pub ts: &'a mut TextSystem,
    pub theme: Theme,
    /// A área do quadro no ecrã.
    pub area: Rect,
    /// O Kanban activo (o rato e as teclas são dele).
    pub active: Option<ElementId>,
}

/// ⭐ O gancho de pintura (a família Kanban implementa-o; a entrada é o `KanbanElementInput` do editor).
pub trait KanbanPainter {
    /// Pinta o Kanban. `false` = não pintou nada (o board-render pinta o nível baixo).
    fn paint(&mut self, p: KanbanPaint<'_>) -> bool;
    /// Pinta o que vai por cima de todo o quadro (o verso aberto). Sem nada a pintar, nada.
    fn paint_over(&mut self, _o: KanbanOver<'_>) {}
}

thread_local! {
    static SOURCE: RefCell<Option<KanbanSource>> = const { RefCell::new(None) };
    static PAINTER: RefCell<Option<Rc<RefCell<dyn KanbanPainter>>>> = const { RefCell::new(None) };
}

/// Os Kanbans que o desenho deste fio lê (o editor instala o `Rc` do projecto).
pub fn install_kanbans(set: KanbanSource) {
    SOURCE.with(|s| *s.borrow_mut() = Some(set));
}

/// O gancho de pintura deste fio (a família Kanban instala-o ao arrancar).
pub fn install_kanban_painter(p: Rc<RefCell<dyn KanbanPainter>>) {
    PAINTER.with(|s| *s.borrow_mut() = Some(p));
}

/// O nome do Kanban `id` nos Kanbans deste fio (o export para `.excalidraw` escreve-o).
#[must_use]
pub fn kanban_name(id: KanbanId) -> Option<String> {
    SOURCE.with(|s| {
        let set = s.borrow().clone()?;
        let set = set.borrow();
        set.get(id).map(|k| k.header.name.clone())
    })
}

/// Há gancho de pintura (sem ele o Kanban do quadro não é interactivo).
#[must_use]
pub fn kanban_painter_installed() -> bool {
    PAINTER.with(|s| s.borrow().is_some())
}

/// Onde um Kanban ficou no ecrã neste desenho.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KanbanPlacement {
    /// Local (`0..w × 0..h`) → ecrã.
    pub to_screen: Affine,
    pub lod: KanbanLod,
    /// O tamanho local (`el.w`, `el.h`).
    pub size: [f64; 2],
}

/// ⭐ O estado dos Kanbans no desenho (um por [`RenderCache`]): o activado (o rato e as teclas dentro
/// dele são do gancho) e onde cada um ficou.
#[derive(Default)]
pub struct KanbanFrames {
    pub active: Option<ElementId>,
    /// Um carregar que foi do Kanban activo (o largar também é dele).
    pub pressed: bool,
    pub on_screen: BTreeMap<ElementId, KanbanPlacement>,
    /// O tema do último desenho (o passe por cima não o recebe).
    pub theme: Theme,
}

impl KanbanFrames {
    /// O ponto de ecrã `p` em coordenadas locais de `id` (`None` = fora dele, ou não está à vista).
    #[must_use]
    pub fn local(&self, id: ElementId, p: [f64; 2]) -> Option<[f64; 2]> {
        let pl = self.on_screen.get(&id)?;
        let l = pl.to_screen.inverse() * Point::new(p[0], p[1]);
        let inside = l.x >= 0.0 && l.y >= 0.0 && l.x <= pl.size[0] && l.y <= pl.size[1];
        inside.then_some([l.x, l.y])
    }

    /// `p` (ecrã) cai no cabeçalho de `id` (o duplo clique abre-o na aba).
    #[must_use]
    pub fn on_header(&self, id: ElementId, zoom: f64, p: [f64; 2]) -> bool {
        self.local(id, p).is_some_and(|l| l[1] <= HEADER_H * zoom)
    }

    pub(crate) fn begin_frame(&mut self) {
        self.on_screen.clear();
    }
}

fn hue_index(h: Hue) -> usize {
    Hue::ALL.iter().position(|x| *x == h).unwrap_or(0)
}

fn label(theme: Theme, hue: Hue, shade: Shade) -> Color {
    let s = Shade::ALL.iter().position(|x| *x == shade).unwrap_or(0);
    token(ph2d_tokens::kanban::LABELS[s][hue_index(hue)], theme)
}

/// ⭐ O fundo de um Kanban: o azul do Trello; outro matiz, a etiqueta forte dele.
#[must_use]
pub fn kanban_board_bg(theme: Theme, bg: Background) -> Color {
    match bg {
        Background::Color(Hue::Blue) | Background::Gradient(_) | Background::Image(_) => {
            token(T::KanbanBoardBg, theme)
        }
        Background::Color(h) => label(theme, h, Shade::Bold),
    }
}

/// ⭐ Uma lista: (fundo, tinta do título, tinta da contagem). Com cor, a intensidade SUTIL do matiz.
#[must_use]
pub fn kanban_list_colors(theme: Theme, hue: Option<Hue>) -> (Color, Color, Color) {
    match hue {
        None => (
            token(T::KanbanList, theme),
            token(T::KanbanListFg, theme),
            token(T::KanbanListMeta, theme),
        ),
        Some(h) => (
            label(theme, h, Shade::Subtle),
            token(T::KanbanLabelInk, theme),
            token(T::KanbanTintedMeta, theme),
        ),
    }
}

/// Pinta o Kanban de `el` (`v` = mundo → ecrã): pelo gancho, ou o nível baixo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    cache: &mut RenderCache,
    el: &Element,
    k: &BoardKanban,
    v: Affine,
    zoom: f64,
    theme: Theme,
    board: u64,
) {
    let t = v * to_world(el);
    // Um export leva o nível MÉDIO (os títulos e os cartões; sem o estado de interacção).
    let lod = if cache.export.is_some() {
        KanbanLod::Mid
    } else {
        KanbanLod::at(zoom * k.view.zoom)
    };
    if cache.export.is_none() {
        cache.kanbans.theme = theme;
        let size = [el.w, el.h];
        let pl = KanbanPlacement { to_screen: t, lod, size };
        cache.kanbans.on_screen.insert(el.id, pl);
    }
    let st = &k.style;
    let alpha = f32::from(st.opacity.min(100)) / 100.0;
    let outer = crate::image::box_path(el, st.round);
    if alpha < 1.0 {
        scene.push_layer_shape(VelloBlend::default(), alpha, t, &outer);
    }
    scene.push_layer_shape(VelloBlend::default(), 1.0, t, &outer);
    let frame = Rect::new(0.0, 0.0, el.w, el.h);
    let source = SOURCE.with(|s| s.borrow().clone());
    let painter = PAINTER.with(|s| s.borrow().clone());
    let set = source.as_ref().map(|s| s.borrow());
    let painted = match (&painter, &set) {
        (Some(p), Some(set)) => p.borrow_mut().paint(KanbanPaint {
            scene,
            ts,
            theme,
            to_screen: t,
            frame,
            lod,
            element: el.id,
            kanban: k.kanban,
            view: &k.view,
            set,
            export: cache.export.is_some(),
        }),
        _ => false,
    };
    if !painted {
        let kb = set.as_ref().and_then(|s| s.get(k.kanban));
        let bg = kb.map_or_else(
            || token(T::KanbanBoardBg, theme),
            |kb| kanban_board_bg(theme, kb.header.background),
        );
        scene.fill_path(&outer, &Brush::Solid(bg), t);
        if let Some(kb) = kb {
            low(scene, ts, cache, (el, &k.view, t, zoom), kb, theme, board);
        }
    }
    scene.pop_layer();
    if alpha < 1.0 {
        scene.pop_layer();
    }
    crate::image_frame::paint(scene, cache, el, st, t, board);
}

/// ⭐ O passe por cima do quadro (o verso aberto de um Kanban activo), depois de tudo o resto.
pub(crate) fn paint_over(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    cache: &crate::RenderCache,
    area: ph2d_board_model::Area,
) {
    if cache.export.is_some() || cache.kanbans.on_screen.is_empty() {
        return;
    }
    let Some(p) = PAINTER.with(|s| s.borrow().clone()) else {
        return;
    };
    let [x, y, w, h] = area;
    p.borrow_mut().paint_over(KanbanOver {
        scene,
        ts,
        theme: cache.kanbans.theme,
        area: Rect::new(x, y, x + w, y + h),
        active: cache.kanbans.active,
    });
}

/// O dono de um texto do nível baixo na cache do texto: o espaço alto do quadro (os elementos usam o
/// baixo) e o índice do texto dentro do elemento.
fn owner(board: u64, el: ElementId, i: u64) -> (u64, u64) {
    (board | 1 << 63, el.0 << 12 | i)
}

/// Um texto numa linha, a `px` locais, com o topo em `at` (local) — um traço de leitura se fica
/// abaixo do limiar de leitura no ecrã.
#[allow(clippy::too_many_arguments)]
fn text(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    cache: &mut RenderCache,
    who: (u64, u64),
    s: &str,
    px: f64,
    at: [f64; 2],
    right: Option<f64>,
    t: Affine,
    screen: f64,
    ink: Color,
) {
    if s.is_empty() {
        return;
    }
    if px * screen < READ_PX {
        let w = px * GREEK_ADVANCE * s.chars().count() as f64;
        let x = right.map_or(at[0], |r| r - w);
        let [b0, b1] = GREEK_BAND;
        let bar = Rect::new(x, at[1] + px * b0, x + w, at[1] + px * b1).to_path(0.1);
        scene.fill_path(&bar, &Brush::Solid(ink.multiply_alpha(GREEK_ALPHA)), t);
        return;
    }
    let layout = cache.text.get(ts, who, s, &[], px as f32, f32::MAX, false);
    let x = right.map_or(at[0], |r| r - f64::from(layout.width()));
    ph2d_board_layout::paint(scene, layout, s, t * Affine::translate((x, at[1])), ink);
}

/// ⭐ O nível baixo: o cabeçalho com o nome e uma coluna por lista (a cor dela, o título em
/// maiúsculas e a contagem de cartões), na geometria da aba à escala da vista.
fn low(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    cache: &mut RenderCache,
    (el, view, t, zoom): (&Element, &KanbanBoardView, Affine, f64),
    kb: &ph2d_kanban_model::Kanban,
    theme: Theme,
    board: u64,
) {
    let z = view.zoom;
    let screen = zoom * z;
    let head = Rect::new(0.0, 0.0, el.w, HEADER_H * z).to_path(0.1);
    scene.fill_path(&head, &Brush::Solid(token(T::KanbanHeaderBg, theme)), t);
    let title_at = [HEADER_PAD * z, (HEADER_H - TITLE_PX * LINE_K) * z / 2.0];
    let fg = token(T::KanbanHeaderFg, theme);
    let name = &kb.header.name;
    text(scene, ts, cache, owner(board, el.id, 0), name, TITLE_PX * z, title_at, None, t, screen, fg);
    let top = (HEADER_H + BOARD_PAD) * z;
    let bottom = el.h - BOARD_PAD * z;
    let mut x = BOARD_PAD * z - view.scroll[0];
    for (i, l) in kb.lists_in_order().into_iter().enumerate() {
        if x > el.w {
            break;
        }
        let collapsed = view.collapsed.contains(&l.id);
        let w = if collapsed { COLLAPSED_W } else { LIST_W } * z;
        let cards = kb.cards_in(l.id).len();
        let tall = (LIST_HEAD + cards as f64 * CARD_STEP + LIST_FOOT) * z;
        let h = if collapsed { bottom - top } else { tall.min(bottom - top) };
        if x + w > 0.0 && h > 0.0 {
            let (fill, ink, meta) = kanban_list_colors(theme, l.color);
            let r = RoundedRect::new(x, top, x + w, top + h, LIST_RADIUS * z).to_path(0.1);
            scene.fill_path(&r, &Brush::Solid(fill), t);
            if !collapsed {
                scene.push_layer_shape(VelloBlend::default(), 1.0, t, &r);
                let (px, pad) = (LIST_TITLE_PX * z, LIST_PAD * z);
                let at = [x + pad, top + (LIST_HEAD - LIST_TITLE_PX * LINE_K) * z / 2.0];
                let n = i as u64 * 2 + 1;
                let up = l.title.to_uppercase();
                text(scene, ts, cache, owner(board, el.id, n), &up, px, at, None, t, screen, ink);
                let count = cards.to_string();
                let right = Some(x + w - pad);
                text(scene, ts, cache, owner(board, el.id, n + 1), &count, px, at, right, t, screen, meta);
                scene.pop_layer();
            }
        }
        x += w + LIST_GAP * z;
    }
}

#[cfg(test)]
#[path = "kanban_tests.rs"]
mod tests;
