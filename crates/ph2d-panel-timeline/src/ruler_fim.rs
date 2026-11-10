//! ⭐ **O fim da animação nos cartões** (dono 10/11: «a linha que marca o fim desapareceu e o
//! rachurado só não dá contraste suficiente»). Filho de `ruler` (tecto de LOC), lê a geometria dele
//! por `super::*`. Duas peças, ambas pintadas pela régua DEPOIS das faixas (nunca tapadas por peça):
//!
//! - **além do fim** — o véu da fita funda apaga o que passa do fim, e listras claras a 45° (`\`;
//!   a mistura vai a 135°, `/`) dizem «fora» sobre peça e sobre fita vazia;
//! - **a linha do fim** — a da ⟷ da duração, régua e faixas inteiras, na tinta e nos estados dela.
//!
//! Fora dos cartões nada disto pinta: o véu clássico (`BgScrim`) fica em `ruler.rs`.

use super::*;
use ph2d_editor_core::paint::{rect_to_vello, stroke_polyline};
use ph2d_tokens::TimelineTinta;
use ph2d_tokens::chrome::{
    TIMELINE_ALEM_LISTRA_ALPHA, TIMELINE_ALEM_LISTRA_PX, TIMELINE_ALEM_PASSO_PX,
    TIMELINE_ALEM_VEU_ALPHA, TIMELINE_FIM_LINHA_PX,
};
use ph2d_vector::Color;

/// `(véu, listra)` de «além do fim» — a porta que o pintor e o portão leem.
pub(crate) fn tons_alem(theme: Theme) -> (Color, Color) {
    let fita = crate::folha_tinta::tinta(TimelineTinta::FitaPropriedade, theme);
    let claro = crate::folha_tinta::tinta(TimelineTinta::TextoCabecalho, theme);
    (
        fita.multiply_alpha(TIMELINE_ALEM_VEU_ALPHA),
        claro.multiply_alpha(TIMELINE_ALEM_LISTRA_ALPHA),
    )
}

/// O véu e as listras `\` de «além do fim» dentro de `shade`.
pub(super) fn pinta_alem(ctx: &mut PaintCtx, theme: Theme, shade: Rect) {
    if shade.w <= 0.0 || shade.h <= 0.0 {
        return;
    }
    let (veu, listra) = tons_alem(theme);
    fill_rounded_rect(ctx.scene, shade, 0.0, veu);
    ctx.scene.push_clip(&rect_to_vello(shade));
    let (y0, y1) = (shade.y, shade.y + shade.h);
    let mut x = shade.x - shade.h;
    while x < shade.x + shade.w {
        stroke_polyline(
            ctx.scene,
            &[(x, y0), (x + shade.h, y1)],
            TIMELINE_ALEM_LISTRA_PX,
            listra,
        );
        x += TIMELINE_ALEM_PASSO_PX;
    }
    ctx.scene.pop_layer();
}

/// O rect da linha do fim em `region` (régua + faixas, altura cheia), ou `None` sem fim autorado ou
/// com o fim fora da tela — o mesmo portão do véu e da ⟷.
pub(crate) fn linha_do_fim(
    region: Rect,
    view_start: f64,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
) -> Option<Rect> {
    if !snap.view_length_explicit {
        return None;
    }
    let x = region.x + ((snap.view_length_seconds - view_start) * px_per_s) as f32;
    let dentro = x >= region.x && x <= region.x + region.w;
    let w = TIMELINE_FIM_LINHA_PX;
    dentro.then(|| Rect::new(x - w * 0.5, region.y, w, region.h))
}

/// A linha do fim nos cartões: a tinta e o estado da ⟷ da duração (é a borda que ela arrasta), com
/// a sombra das alças por baixo para se ler sobre qualquer peça e em todo tema.
pub(super) fn pinta_linha_do_fim(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    (view_start, px_per_s): (f64, f64),
    snap: &TimelineViewSnapshot,
) {
    let Some(linha) = linha_do_fim(region, view_start, px_per_s, snap) else {
        return;
    };
    let cor = resolve(ColorToken::TimelinePlayhead, theme);
    let e = EstadoAlca::nos_cartoes(ctx, ids::TIMELINE_DUR_HANDLE);
    crate::alca_estado::pinta(ctx, theme, e, cor, &[linha], &[]);
}

#[cfg(test)]
#[path = "ruler_fim_tests.rs"]
mod tests;
