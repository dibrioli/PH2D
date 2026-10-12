//! ⭐⭐ **As barras FLUTUANTES** — a coluna curta de ferramentas encostada à esquerda de uma área, a
//! grelha que um botão dela abre (flyout), e a barra de ações junto da seleção.
//!
//! **Um widget, várias tabelas:** o Quadro ([`super::board_bar`]) e o Vetor (as
//! [`crate::interaction::ToolBars`] que a crate do vetor publica, pintadas por [`super::tool_bars`]).
//! Aqui vive o que as duas partilham — a GEOMETRIA (onde cai cada botão), o FUNDO do painel e o
//! botão (estado, hover, alvo). A CARA de cada botão é da tabela: o Quadro desenha o contorno das
//! formas dele, o Vetor pinta ícones e amostras.
//!
//! ⛔ Nasceu da barra do Quadro e não ao lado dela: *copiar UM componente deixa para trás os IRMÃOS*,
//! e *«é a mesma conta que X» ao lado de uma cópia é o cheiro da lei duplicada* (memória da casa).
//!
//! ⚠️ **0 px de layout** (D10 decisão 1, recusa de 31/08): flutuam DENTRO da área; posições âncora
//! (D1), nunca soltas.

use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, resolve};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme};
use ph2d_vector::{Color, VectorScene};

/// O lado de um botão da COLUNA e do flyout.
#[must_use]
pub fn button_px() -> f32 {
    Spacing::Xl2.px()
}

/// O lado de um botão da barra junto da seleção — mais pequena, para tapar menos arte.
#[must_use]
pub fn strip_button_px() -> f32 {
    Spacing::Xl.px()
}

/// O espaço entre dois botões vizinhos.
#[must_use]
pub fn gap() -> f32 {
    ph2d_tokens::control_gap_px()
}

/// ⭐ **A margem de um ENQUADRAMENTO** (F8, px lógicos): a faixa que a barra de ações ④ ocupa por
/// cima da seleção (o afastamento `Lg`, uma linha de botões e o ar `Xs` do painel dela) mais o mesmo
/// ar até à borda — o que se enquadra nunca fica por baixo de uma barra flutuante. Derivada, não
/// escolhida: dá 44, o meio da faixa medida nos oráculos (Excalidraw 24, tldraw 64), e cobre a
/// coluna de ferramentas (gate `the_frame_margin_clears_both_floating_bars`).
#[must_use]
pub fn frame_margin() -> f32 {
    Spacing::Lg.px() + button_px() + 2.0 * Spacing::Xs.px()
}

/// O espaço entre dois GRUPOS.
#[must_use]
pub fn group_sep() -> f32 {
    Spacing::Sm.px()
}

/// Quantos botões um grupo da barra junto da seleção leva numa linha só (as 16 cores das notas do
/// Quadro vão em duas).
pub const ROW_MAX: usize = 8;

/// ⭐ **A coluna**: um rectângulo por botão, de cima para baixo, encostada ao canto de cima à
/// esquerda de `area`. `groups` são os tamanhos dos grupos, em ordem; entre dois grupos fica um
/// [`group_sep`].
#[must_use]
pub fn column_rects(area: Rect, groups: &[usize]) -> Vec<Rect> {
    let (s, g) = (button_px(), gap());
    let x = area.x + Spacing::Md.px();
    let mut y = area.y + Spacing::Md.px();
    let mut out = Vec::with_capacity(groups.iter().sum());
    for (gi, &n) in groups.iter().enumerate() {
        if gi > 0 && n > 0 {
            y += group_sep() - g;
        }
        for _ in 0..n {
            out.push(Rect::new(x, y, s, s));
            y += s + g;
        }
    }
    out
}

/// `r` espelhado na vertical do meio de `area` — a mão canhota (F5).
#[must_use]
pub fn mirrored(area: Rect, r: Rect) -> Rect {
    Rect::new(area.x + area.w - (r.x - area.x) - r.w, r.y, r.w, r.h)
}

/// O rectângulo do painel que contém `rects`, com o respiro do fundo.
#[must_use]
pub fn panel_of(rects: impl IntoIterator<Item = Rect>) -> Option<Rect> {
    let mut it = rects.into_iter();
    let first = it.next()?;
    let (mut x0, mut y0, mut x1, mut y1) = (first.x, first.y, first.x + first.w, first.y + first.h);
    for r in it {
        x0 = x0.min(r.x);
        y0 = y0.min(r.y);
        x1 = x1.max(r.x + r.w);
        y1 = y1.max(r.y + r.h);
    }
    let pad = Spacing::Xs.px();
    Some(Rect::new(
        x0 - pad,
        y0 - pad,
        x1 - x0 + 2.0 * pad,
        y1 - y0 + 2.0 * pad,
    ))
}

/// ⭐ **A grelha de um flyout**, à direita de `anchor` (o botão da coluna que a abre), `cols`
/// colunas, alinhada pelo topo dele. ⚠️ Presa dentro de `area`: a grelha que passasse do fundo
/// da área ficaria cortada pelo recorte da área e os botões dela inalcançáveis.
#[must_use]
pub fn grid_rects(area: Rect, anchor: Rect, n: usize, cols: usize) -> Vec<Rect> {
    let (s, g) = (button_px(), gap());
    let cols = cols.max(1);
    let rows = n.div_ceil(cols) as f32;
    let h = rows * s + (rows - 1.0).max(0.0) * g;
    let x0 = anchor.x + anchor.w + Spacing::Sm.px();
    let y0 = anchor
        .y
        .min(area.y + area.h - h - Spacing::Md.px())
        .max(area.y);
    (0..n)
        .map(|i| {
            let (col, row) = ((i % cols) as f32, (i / cols) as f32);
            Rect::new(x0 + col * (s + g), y0 + row * (s + g), s, s)
        })
        .collect()
}

/// Colunas de um grupo da barra junto da seleção (`> ROW_MAX` vai em duas linhas).
fn strip_cols(len: usize) -> usize {
    if len > ROW_MAX { len.div_ceil(2) } else { len }
}

/// A largura e a altura da barra junto da seleção para estes grupos, com botões de lado `s`.
fn strip_extent(groups: &[usize], s: f32) -> (f32, f32) {
    let g = gap();
    let rows = if groups.iter().any(|&n| n > ROW_MAX) {
        2.0
    } else {
        1.0
    };
    let live = groups.iter().filter(|&&n| n > 0).count();
    let w = groups.iter().map(|&n| strip_cols(n)).sum::<usize>() as f32 * (s + g)
        + live.saturating_sub(1) as f32 * group_sep();
    (w, rows * s + (rows - 1.0) * g)
}

/// Os rectângulos dos grupos a partir do canto `(x, y)` de uma barra de altura `bar_h`.
fn strip_from(x: f32, y: f32, bar_h: f32, groups: &[usize], s: f32) -> Vec<Rect> {
    let g = gap();
    let mut x = x;
    let mut out = Vec::with_capacity(groups.iter().sum());
    for &n in groups.iter().filter(|&&n| n > 0) {
        let c = strip_cols(n);
        let y0 = if c < n { y } else { y + (bar_h - s) / 2.0 };
        for i in 0..n {
            let (col, row) = ((i % c) as f32, (i / c) as f32);
            out.push(Rect::new(x + col * (s + g), y0 + row * (s + g), s, s));
        }
        x += c as f32 * (s + g) + group_sep();
    }
    out
}

/// ⭐ **A barra junto da seleção**: por cima de `sel` (a seleção no ecrã), presa dentro de
/// `area`; por baixo quando não cabe por cima. Um rectângulo por botão (de lado `s`), grupo a grupo.
///
/// ⚠️ O lado é do CLIENTE: as caras do Quadro (amostras, traços, letras) lêem-se a
/// [`strip_button_px`]; os ÍCONES da ④ do vetor não (medido na foto da cena 9, 07/10: `12 px` de
/// ícone ilegíveis) e usam o [`button_px`] da coluna.
#[must_use]
pub fn strip_rects(area: Rect, sel: Rect, groups: &[usize], s: f32) -> Vec<Rect> {
    strip_rects_at(area, sel, groups, s, Spacing::Lg.px())
}

/// ⭐ **A folga entre a seleção e a ④ do vetor**: uma ALÇA inteira (os widgets da borda da caixa
/// ficam à vista) mais `Spacing::Lg`. O dono (11/10): *«abrem muito próximos das formas»* — eram
/// `Lg` (12 px); doc 17 §3.
#[must_use]
pub fn selection_gap() -> f32 {
    crate::HANDLE_SIZE_PX + Spacing::Lg.px()
}

/// A [`strip_rects`] a `gap` px da seleção — a ④ do vetor afasta-se a [`selection_gap`] (F11); o
/// Quadro fica na de sempre.
#[must_use]
pub fn strip_rects_at(area: Rect, sel: Rect, groups: &[usize], s: f32, gap: f32) -> Vec<Rect> {
    let (width, bar_h) = strip_extent(groups, s);
    let above = sel.y - bar_h - gap;
    let y = if above >= area.y {
        above
    } else {
        (sel.y + sel.h + gap).min(area.y + area.h - bar_h)
    };
    let x = (sel.x + sel.w / 2.0 - width / 2.0)
        .max(area.x)
        .min(area.x + area.w - width);
    strip_from(x, y, bar_h, groups, s)
}

/// ⭐ **A mesma barra, FIXADA**: no topo da área, ao centro. Deixa de seguir a seleção — e por
/// isso nunca tapa a arte que se está a editar (a lição da *Contextual Task Bar*, estudo §4).
#[must_use]
pub fn pinned_strip_rects(area: Rect, groups: &[usize], s: f32) -> Vec<Rect> {
    let (width, bar_h) = strip_extent(groups, s);
    let x = (area.x + area.w / 2.0 - width / 2.0).max(area.x);
    strip_from(x, area.y + Spacing::Md.px(), bar_h, groups, s)
}

/// O fundo (elevado, com moldura) por baixo de um conjunto de botões.
pub fn paint_panel(scene: &mut VectorScene, rects: impl IntoIterator<Item = Rect>, theme: Theme) {
    let Some(panel) = panel_of(rects) else {
        return;
    };
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, panel, radius, resolve(ColorToken::BgElev, theme));
    crate::paint::stroke_frame(
        scene,
        panel,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        resolve(ColorToken::Border, theme),
    );
}

/// O miolo de um botão — onde a cara se pinta.
#[must_use]
pub fn inner(r: Rect) -> Rect {
    let inset = Spacing::Xs.px();
    Rect::new(
        r.x + inset,
        r.y + inset,
        r.w - 2.0 * inset,
        r.h - 2.0 * inset,
    )
}

/// Um botão das barras: o id, onde cai, se está aceso e a tinta da cara em repouso (a família).
#[derive(Clone, Copy, Debug)]
pub struct Btn {
    pub id: NodeId,
    pub rect: Rect,
    pub on: bool,
    pub tint: ColorToken,
}

/// ⭐ **O botão**: o fundo do estado (ligado · hover), o ALVO no `HitIndex`, e a tinta da cara —
/// a `tint` em repouso, o acento quando ligado.
pub fn paint_button(
    scene: &mut VectorScene,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    theme: Theme,
    b: Btn,
) -> Color {
    let hover = matches!(
        store.button_state(b.id),
        Some(ButtonState::Hovered | ButtonState::Pressed | ButtonState::Focused)
    );
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    if b.on {
        fill_rounded_rect(
            scene,
            b.rect,
            radius,
            resolve(ColorToken::AccentSoft, theme),
        );
    } else if hover {
        fill_rounded_rect(scene, b.rect, radius, resolve(ColorToken::Bg2, theme));
    }
    hit_index.register(b.id, b.rect);
    resolve(if b.on { ColorToken::Accent } else { b.tint }, theme)
}

/// ⭐ **Quanto a cara de um botão do MÓDULO cresce sobre o miolo**, sem mexer no botão nem na barra:
/// `1,5×` (Enio, 08/10: *«vamos aumentar o tamanho deles em 50%»*) e mais `15 %` (o meio do 2.º
/// pedido, *«mais 10 a 20%»*). Num botão de `24 px` o desenho vai de `12,8` a `22,1 px`.
pub const MODULE_FACE_GROWTH: f32 = 1.725; // LITERAL-PX-OK: 1,5 × 1,15 — a RAZÃO pedida pelo dono, não um tamanho

/// O quadrado, centrado no botão `r`, em que a cara de um botão do módulo se pinta: o ícone desenha
/// [`MODULE_FACE_GROWTH`] vezes o que desenharia no [`inner`].
///
/// ⚠️ O QUADRADO pode passar do botão — é só o respiro invisível do pintor de ícones; o que nunca
/// passa é o DESENHO com a espessura do traço (preso a `lado − traço`).
#[must_use]
pub fn module_face_rect(r: Rect) -> Rect {
    let m = inner(r);
    let max_glyph = r.w.min(r.h) - StrokeToken::Default.px();
    let glyph = (crate::paint::icon_glyph_side(m.w.min(m.h)) * MODULE_FACE_GROWTH).min(max_glyph);
    let side = crate::paint::icon_side_for_glyph(glyph);
    Rect::new(
        r.x + (r.w - side) / 2.0,
        r.y + (r.h - side) / 2.0,
        side,
        side,
    )
}

/// ⭐ **Uma amostra de cor** no miolo `inner`: o quadrado cheio, ou o vazio cortado pela diagonal
/// quando não há tinta; `ring` deixa só o ANEL (o contorno lê-se como contorno).
pub fn paint_swatch(
    scene: &mut VectorScene,
    theme: Theme,
    inner: Rect,
    rgba: Option<[u8; 4]>,
    ring: bool,
    fg: Color,
) {
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    let line = f64::from(StrokeToken::Default.px());
    match rgba {
        Some([r, g, b, a]) => {
            let doc = Color::from_rgba8(r, g, b, a); // LITERAL-COLOR-OK: cor do DOCUMENTO (do artista), não da UI
            fill_rounded_rect(scene, inner, radius, doc);
        }
        None => {
            crate::paint::stroke_frame(
                scene,
                inner,
                radius,
                theme,
                ph2d_tokens::visuals::Feel::Rest,
                StrokeToken::Default.px(),
                fg,
            );
            let mut p = ph2d_vector::BezPath::new();
            p.move_to((f64::from(inner.x), f64::from(inner.y + inner.h)));
            p.line_to((f64::from(inner.x + inner.w), f64::from(inner.y)));
            scene.inner_mut().stroke(
                &ph2d_vector::Stroke::new(line),
                ph2d_vector::Affine::IDENTITY,
                &ph2d_vector::Brush::Solid(fg),
                None,
                &p,
            );
        }
    }
    if ring && rgba.is_some() {
        let k = ((inner.w - Spacing::Sm.px()) / 2.0).max(0.0);
        let hole = Rect::new(
            inner.x + k,
            inner.y + k,
            inner.w - 2.0 * k,
            inner.h - 2.0 * k,
        );
        fill_rounded_rect(scene, hole, radius, resolve(ColorToken::BgElev, theme));
    }
}

#[path = "flash.rs"]
pub mod flash;
/// O cliente do Vetor (e de qualquer módulo que publique [`crate::interaction::ToolBars`]).
#[path = "tool_bars.rs"]
pub mod tool_bars;
#[path = "touch_mods.rs"]
pub mod touch_mods;

#[cfg(test)]
#[path = "float_bar_tests.rs"]
mod tests;
