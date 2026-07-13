//! **O cursor do pincel do Flip** — o anel que mostra, no canvas, o tamanho da coisa
//! que vai acontecer (smoke do Enio 2026-07-13: *"não vemos o círculo da ferramenta e
//! não é possível saber o tamanho no canvas"*).
//!
//! Ele é bem mais simples que o do Painter, e por um motivo que vale registrar: **o
//! pincel do Flip é ABSOLUTO em px de tela** (Enio 2026-07-11 — a largura não escala
//! com o zoom). Então o raio do anel é `Size/2` em pixels, direto, sem passar por
//! câmera, afim de objeto ou espaço de imagem. O que o anel mostra é exatamente o que o
//! traço vai medir.
//!
//! Modos: **Draw** (a espessura do traço), **Erase** (o raio da borracha) e **Sculpt**
//! (o raio do pincel de escultura) — os três usam o mesmo Size. Em **Select** e
//! **Fill** não há anel: não há raio nenhum em jogo, e um anel ali seria uma mentira.

use ph2d_editor::HeroScreen;
use ph2d_tool_flip::{FlipMode, FlipStyleSnapshot};
use ph2d_vector::VectorScene;

/// Segmentos do anel — o bastante para um círculo ler liso em qualquer tamanho.
const RING_SEGS: u32 = 64;
/// Espessura do traço do anel (px de tela).
const RING_STROKE_PX: f64 = 1.5; // LITERAL-PX-OK: overlay cursor (espelha o anel do Painter)
/// Raio mínimo desenhado (px): abaixo disso o anel vira um ponto e some — um cursor que
/// desaparece quando o pincel é fino é pior que nenhum.
const MIN_RING_R_PX: f64 = 3.0; // LITERAL-PX-OK: piso de visibilidade do overlay

/// O anel do pincel, se o modo atual tiver um raio. Desenha no `vector_scene` (a cena
/// de overlay, composta sobre o canvas neste frame — como o anel do Painter).
pub(super) fn draw_flip_cursor(
    active: bool,
    style: Option<FlipStyleSnapshot>,
    hero: &HeroScreen,
    vector_scene: &mut VectorScene,
    cursor: (f32, f32),
) {
    if !active {
        return;
    }
    let Some(r) = ring_radius(style) else {
        return;
    };
    let (cx, cy) = cursor;
    // Sobre um painel, o cursor é do painel: não desenha.
    if hero.store.panel_at(cx, cy).is_some() {
        return;
    }

    use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke};
    use std::f64::consts::TAU;
    let mut path = BezPath::new();
    for i in 0..RING_SEGS {
        let (s, c) = (f64::from(i) * TAU / f64::from(RING_SEGS)).sin_cos();
        let p = Point::new(f64::from(cx) + c * r, f64::from(cy) + s * r);
        if i == 0 {
            path.move_to(p);
        } else {
            path.line_to(p);
        }
    }
    path.close_path();
    let color = Color::new([0.78, 0.78, 0.78, 0.85]); // LITERAL-COLOR-OK: overlay cursor
    vector_scene.inner_mut().stroke(
        &Stroke::new(RING_STROKE_PX),
        Affine::IDENTITY,
        &Brush::Solid(color),
        None,
        &path,
    );
}

/// **O raio do anel, em px de TELA** — ou `None` se o modo não tem raio nenhum.
///
/// A política mora aqui (e não no meio do desenho) porque é ela que se testa: um anel
/// no modo Fill seria uma mentira (não há raio em jogo), e um anel que some quando o
/// pincel é fino é pior que nenhum — daí o piso.
#[must_use]
pub(crate) fn ring_radius(style: Option<FlipStyleSnapshot>) -> Option<f64> {
    let style = style?;
    matches!(
        style.mode,
        FlipMode::Draw | FlipMode::Erase | FlipMode::Reshape
    )
    .then(|| (style.width_px * 0.5).max(MIN_RING_R_PX))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O anel aparece nos três modos que TÊM raio, e em nenhum outro. Um anel no Fill
    /// (que não tem raio) seria um controle mentindo — o mesmo princípio do painel modal.
    #[test]
    fn the_ring_is_drawn_only_in_the_modes_that_have_a_radius() {
        for (mode, want) in [
            (FlipMode::Draw, true),
            (FlipMode::Erase, true),
            (FlipMode::Reshape, true),
            (FlipMode::Fill, false),
            (FlipMode::Select, false),
        ] {
            let r = ring_radius(Some(FlipStyleSnapshot {
                mode,
                width_px: 40.0,
                ..Default::default()
            }));
            assert_eq!(r.is_some(), want, "modo {mode:?}: deveria ter anel? {want}");
            if want {
                assert_eq!(r, Some(20.0), "o anel e METADE do Size, em px de TELA");
            }
        }
        assert!(
            ring_radius(None).is_none(),
            "sem estilo publicado, sem anel"
        );
    }

    /// **O anel nunca some**: um pincel de 1 px ainda desenha um alvo visível (o piso).
    /// Um cursor que desaparece justo quando o traço fica fino é pior que nenhum.
    #[test]
    fn a_hairline_brush_still_shows_a_visible_ring() {
        let r = ring_radius(Some(FlipStyleSnapshot {
            mode: FlipMode::Draw,
            width_px: 1.0,
            ..Default::default()
        }))
        .unwrap();
        assert!(r >= MIN_RING_R_PX, "o anel sumiu num pincel fino: {r}");
    }

    /// E o anel é ABSOLUTO: `Size = 200 px` desenha 100 px de raio na tela, ponto —
    /// nenhuma câmera entra na conta (o pincel do Flip não escala com o zoom).
    #[test]
    fn the_ring_is_half_the_size_in_screen_pixels() {
        let r = ring_radius(Some(FlipStyleSnapshot {
            mode: FlipMode::Reshape,
            width_px: 200.0,
            ..Default::default()
        }));
        assert_eq!(r, Some(100.0));
    }
}
