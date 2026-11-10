//! ⭐ **Os ESTADOS de uma alça da Timeline nos cartões** (dono 10/11: «destaque nas alças no
//! arrastar, mouse down e hover»). Uma porta para todas as alças (cantos e fade das strips, chaves
//! do loop e a linha de mover, a ⟷ da duração, as chaves da folha):
//!
//! - **hover** — a alça aproxima-se da tinta da chave ([`ph2d_tokens::TIMELINE_ALCA_HOVER_MIX`]);
//! - **apertada** — o azul da seleção (`TimelineTinta::ChaveEscolhida`), a forma `Thin` maior;
//! - **arrasto** — o azul + um halo de acento sob a sombra; quem tem um tempo põe guia e leitura.
//!
//! O estado sai do store (`hot_id`, `active_id`, `timeline_dragging`) — o mesmo id que o clique
//! agarra, então o destaque não pode discordar do alvo. Fora dos cartões nada muda aqui: só os
//! cartões perguntam pelo estado; o hit rect nunca depende dele.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_elided};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::ferramenta::cards_on;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{Radius, Spacing, StrokeToken, Theme, TimelineTinta, TypeToken};
use ph2d_vector::{Affine, BezPath, Brush, Color, Fill, Stroke};

use crate::folha_tinta::tinta;

/// Um triângulo de uma alça, por três pontos.
pub(crate) type Tri = [(f32, f32); 3];

/// O estado de uma alça sob o ponteiro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EstadoAlca {
    Repouso,
    Hover,
    Apertada,
    Arrasto,
}

impl EstadoAlca {
    /// O estado de `id` no store: o aperto vence o hover, e enquanto algo está apertado nenhuma
    /// outra alça se acende (o ponteiro a passar por cima durante um arrasto não é um convite).
    #[must_use]
    pub(crate) fn de(store: &WidgetStore, id: NodeId) -> Self {
        match store.active_id() {
            Some(a) if a == id && store.timeline_dragging() => Self::Arrasto,
            Some(a) if a == id => Self::Apertada,
            Some(_) => Self::Repouso,
            None if store.hot_id() == Some(id) => Self::Hover,
            None => Self::Repouso,
        }
    }

    /// O estado nos cartões; fora deles, sempre o repouso (a saída de referência fica byte a byte).
    #[must_use]
    pub(crate) fn nos_cartoes(ctx: &PaintCtx, id: NodeId) -> Self {
        if cards_on() {
            Self::de(ctx.host.store(), id)
        } else {
            Self::Repouso
        }
    }

    const fn apertada(self) -> bool {
        matches!(self, Self::Apertada | Self::Arrasto)
    }

    /// Quanto a forma cresce de cada lado neste estado.
    #[must_use]
    pub(crate) fn cresce(self) -> f32 {
        if self.apertada() {
            StrokeToken::Thin.px()
        } else {
            0.0
        }
    }
}

/// A cor da alça neste estado, a partir da cor de repouso.
#[must_use]
pub(crate) fn cor(estado: EstadoAlca, repouso: Color, theme: Theme) -> Color {
    match estado {
        EstadoAlca::Repouso => repouso,
        EstadoAlca::Hover => mistura(
            repouso,
            tinta(TimelineTinta::Chave, theme),
            ph2d_tokens::TIMELINE_ALCA_HOVER_MIX,
        ),
        EstadoAlca::Apertada | EstadoAlca::Arrasto => acento(theme),
    }
}

/// O azul da seleção — a tinta da chave escolhida.
#[must_use]
pub(crate) fn acento(theme: Theme) -> Color {
    tinta(TimelineTinta::ChaveEscolhida, theme)
}

fn mistura(a: Color, b: Color, t: f32) -> Color {
    let (x, y) = (a.components, b.components);
    Color::new([
        x[0] + (y[0] - x[0]) * t,
        x[1] + (y[1] - x[1]) * t,
        x[2] + (y[2] - x[2]) * t,
        x[3],
    ])
}

fn cresce_rect(r: Rect, d: f32) -> Rect {
    Rect::new(r.x - d, r.y - d, r.w + d * 2.0, r.h + d * 2.0)
}

/// Os vértices afastados do centróide por `d` (o triângulo maior, com o mesmo centro).
fn cresce_tri(pts: Tri, d: f32) -> Tri {
    let cx = (pts[0].0 + pts[1].0 + pts[2].0) / 3.0; // LITERAL-PX-OK: a média de três vértices
    let cy = (pts[0].1 + pts[1].1 + pts[2].1) / 3.0; // LITERAL-PX-OK: a média de três vértices
    pts.map(|(x, y)| {
        let (dx, dy) = (x - cx, y - cy);
        let n = dx.hypot(dy).max(f32::EPSILON);
        (x + dx / n * d, y + dy / n * d)
    })
}

fn tri_path(pts: Tri) -> BezPath {
    let mut p = BezPath::new();
    p.move_to((f64::from(pts[0].0), f64::from(pts[0].1)));
    p.line_to((f64::from(pts[1].0), f64::from(pts[1].1)));
    p.line_to((f64::from(pts[2].0), f64::from(pts[2].1)));
    p.close_path();
    p
}

/// ⭐ **A porta**: pinta uma alça feita de `rects` e `tris` no `estado` dela — halo (arrasto),
/// sombra (cartões), a forma (maior quando apertada) na cor do estado.
pub(crate) fn pinta(
    ctx: &mut PaintCtx,
    theme: Theme,
    estado: EstadoAlca,
    repouso: Color,
    rects: &[Rect],
    tris: &[Tri],
) {
    let d = estado.cresce();
    let rects: Vec<Rect> = rects.iter().map(|r| cresce_rect(*r, d)).collect();
    let tris: Vec<Tri> = tris.iter().map(|t| cresce_tri(*t, d)).collect();
    if estado == EstadoAlca::Arrasto {
        let halo = acento(theme).multiply_alpha(ph2d_tokens::TIMELINE_ALCA_HALO_ALPHA);
        let g = StrokeToken::Heavy.px();
        for r in &rects {
            fill_rounded_rect(ctx.scene, cresce_rect(*r, g), Radius::Xs.px() + g, halo);
        }
        for t in &tris {
            let s = Stroke::new(f64::from(g * 2.0));
            let b = Brush::Solid(halo);
            ctx.scene
                .stroke_path(&s, Affine::IDENTITY, &b, &tri_path(*t));
        }
    }
    crate::strip_paint::pinta_sombra_de_alca(ctx, &rects, &tris);
    let c = cor(estado, repouso, theme);
    for r in rects {
        fill_rounded_rect(ctx.scene, r, Radius::Xs.px(), c);
    }
    let b = Brush::Solid(c);
    for t in tris {
        ctx.scene
            .fill_shape(Fill::NonZero, Affine::IDENTITY, &b, &tri_path(t));
    }
}

/// A guia do arrasto: um fio de acento vertical em `x`, de `y0` a `y1`. Só no arrasto.
pub(crate) fn pinta_guia(
    ctx: &mut PaintCtx,
    theme: Theme,
    estado: EstadoAlca,
    x: f32,
    y0: f32,
    y1: f32,
) {
    if estado != EstadoAlca::Arrasto {
        return;
    }
    let w = StrokeToken::Thin.px();
    let cor = acento(theme).multiply_alpha(ph2d_tokens::TIMELINE_ALCA_GUIA_ALPHA);
    fill_rounded_rect(ctx.scene, Rect::new(x - w * 0.5, y0, w, y1 - y0), 0.0, cor);
}

/// O texto da leitura: segundos com duas casas, sem «-0.00».
#[must_use]
pub(crate) fn leitura_str(segundos: f64) -> String {
    ph2d_editor_core::widget::zero_sem_sinal(&format!("{segundos:.2}")).into_owned()
}

/// O rect da etiqueta de leitura: à direita de `x` (à esquerda quando `a_esquerda`), com o centro
/// vertical em `cy`, a largura do texto mais `Xs` de cada lado.
#[must_use]
pub(crate) fn leitura_rect(texto_w: f32, x: f32, cy: f32, a_esquerda: bool) -> Rect {
    let pad = Spacing::Xs.px();
    let (w, h) = (texto_w + pad * 2.0, TypeToken::Xs.px() + pad * 2.0);
    let x0 = if a_esquerda { x - pad - w } else { x + pad };
    Rect::new(x0, cy - h * 0.5, w, h)
}

/// ⭐ A leitura do arrasto (a etiqueta de tempo à Figma): azul, o texto na tinta de mais contraste.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pinta_leitura(
    ctx: &mut PaintCtx,
    theme: Theme,
    estado: EstadoAlca,
    segundos: f64,
    x: f32,
    cy: f32,
    a_esquerda: bool,
) {
    if estado != EstadoAlca::Arrasto {
        return;
    }
    let texto = leitura_str(segundos);
    let font = TypeToken::Xs.px();
    let tw = ctx.text_system.prefix_width(&texto, font);
    let r = leitura_rect(tw, x, cy, a_esquerda);
    let fundo = acento(theme);
    fill_rounded_rect(ctx.scene, r, Radius::Xs.px(), fundo);
    let pad = Spacing::Xs.px();
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &texto,
        r.x + pad,
        r.y + pad,
        font,
        tw,
        tinta_sobre(fundo, theme),
    );
}

/// A tinta de texto de mais contraste sobre `fundo`: a do nome do objecto ou a da fita.
#[must_use]
pub(crate) fn tinta_sobre(fundo: Color, theme: Theme) -> Color {
    let lum = |c: Color| {
        let k = c.components;
        0.2126 * k[0] + 0.7152 * k[1] + 0.0722 * k[2] // LITERAL-PX-OK: os pesos de luminância Rec. 709
    };
    let (a, b) = (
        tinta(TimelineTinta::TextoCabecalho, theme),
        tinta(TimelineTinta::FitaPropriedade, theme),
    );
    if (lum(a) - lum(fundo)).abs() >= (lum(b) - lum(fundo)).abs() {
        a
    } else {
        b
    }
}

/// A chave (losango) nos cartões, no estado dela: o anel da fita, o losango na cor do estado.
pub(crate) fn pinta_chave(
    ctx: &mut PaintCtx,
    theme: Theme,
    cx: f32,
    cy: f32,
    escolhida: bool,
    estado: EstadoAlca,
) {
    let d = estado.cresce();
    let h = ph2d_tokens::TIMELINE_KEY_HALF_PX + d;
    let anel = h + ph2d_tokens::TIMELINE_KEY_RING_PX;
    let t = if escolhida {
        TimelineTinta::ChaveEscolhida
    } else {
        TimelineTinta::Chave
    };
    if estado == EstadoAlca::Arrasto {
        let halo = acento(theme).multiply_alpha(ph2d_tokens::TIMELINE_ALCA_HALO_ALPHA);
        crate::tracks::paint_diamond(ctx, cx, cy, anel + StrokeToken::Heavy.px(), halo);
    }
    let repouso = tinta(t, theme);
    let mut fundo = tinta(TimelineTinta::FitaPropriedade, theme);
    // A chave em repouso já é a tinta mais clara: o hover dela acende também o ANEL (a borda).
    if estado == EstadoAlca::Hover {
        fundo = mistura(fundo, repouso, ph2d_tokens::TIMELINE_ALCA_HOVER_MIX);
    }
    crate::tracks::paint_diamond(ctx, cx, cy, anel, fundo);
    crate::tracks::paint_diamond(ctx, cx, cy, h, cor(estado, repouso, theme));
}

/// Uma chave da folha no estado dela, com a leitura do tempo durante o arrasto.
pub(crate) fn chave_da_folha(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: NodeId,
    (kx, cy): (f32, f32),
    tok: ph2d_tokens::ColorToken,
    segundos: f64,
) {
    let estado = EstadoAlca::nos_cartoes(ctx, id);
    if !cards_on() {
        crate::tracks::paint_chave(ctx, theme, kx, cy, tok);
        return;
    }
    let escolhida = tok == ph2d_tokens::ColorToken::TimelineKeySelected;
    pinta_chave(ctx, theme, kx, cy, escolhida, estado);
    let alcance = ph2d_tokens::TIMELINE_KEY_HALF_PX + ph2d_tokens::TIMELINE_KEY_RING_PX;
    pinta_leitura(ctx, theme, estado, segundos, kx + alcance, cy, false);
}

#[cfg(test)]
#[path = "alca_estado_tests.rs"]
mod tests;
