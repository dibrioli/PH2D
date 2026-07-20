//! **O cursor do pincel do Flip** — o anel que mostra, no canvas, o tamanho da coisa
//! que vai acontecer (smoke do Enio 2026-07-13: *"não vemos o círculo da ferramenta e
//! não é possível saber o tamanho no canvas"*).
//!
//! **O Size mede o MUNDO** (§4.C.6 — Enio 2026-07-17 reverteu o brush absoluto em px de
//! tela de 2026-07-11), então o anel se PROJETA: `raio = size_to_world(Size)/2 ·
//! px_per_world`. Dar zoom engrossa a tinta, e o anel engrossa junto — é essa a única
//! coisa que ele tem de fazer: mostrar o tamanho do que vai acontecer. Um anel de tamanho
//! fixo na tela sobre um traço que segue o zoom seria o anel mentindo, e é para não ter
//! essa mentira que este módulo existe.
//!
//! (Ele segue mais simples que o do Painter: não passa por afim de objeto nem espaço de
//! imagem — só pelo zoom.)
//!
//! Modos: **Draw** (a espessura do traço), **Erase** (o raio da borracha), **Sculpt** (o raio
//! do pincel de escultura) e **Colorize** (a espessura do rabisco — o mesmo Size do pincel, e
//! como o rabisco SEMEIA pela cápsula, o anel mostra o quanto ele vai pegar). Em **Select** e
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
    // §4.C.6: o Size mede o MUNDO, então o anel precisa do zoom para se projetar.
    px_per_world: f64,
) {
    if !active {
        return;
    }
    let Some(r) = ring_radius(style, px_per_world) else {
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
pub(crate) fn ring_radius(style: Option<FlipStyleSnapshot>, px_per_world: f64) -> Option<f64> {
    let style = style?;
    // **A borracha usa o raio EFETIVO dela** (§4.C) — `erase_px` já vem com o link
    // resolvido pela tool. Com o link ligado (default) esse número É o `width_px`, então
    // o anel não muda; deslinkado, o anel mostra o tamanho com que se vai APAGAR, que é
    // o ponto inteiro de ter um anel.
    let size = match style.mode {
        FlipMode::Erase => style.erase_px,
        // Colorize usa o MESMO Size do pincel (a espessura do rabisco); o anel mostra o
        // tamanho da cápsula que vai semear — sem ele o pincel de Colorize não tinha círculo
        // no canvas (report do Enio 2026-07-19).
        FlipMode::Draw | FlipMode::Reshape | FlipMode::Colorize => style.width_px,
        _ => return None,
    };
    // **Mundo → TELA** (§4.C.6): o Size mede o MUNDO, e o anel promete o que vai
    // acontecer — então ele acompanha o zoom, como a tinta. Um anel de tamanho fixo na
    // tela sobre um traço que engrossa com o zoom seria o anel MENTINDO, que é o defeito
    // que este arquivo existe para não ter.
    let r = f64::from(ph2d_tool_flip::size_to_world(size)) * 0.5 * px_per_world;
    Some(r.max(MIN_RING_R_PX))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O zoom de referência**: com `px_per_world == SIZE_PX_PER_WORLD`, uma unidade de
    /// Size projeta exatamente 1 px — então o anel vale `Size/2` e os números aqui leem
    /// como liam quando o pincel era absoluto em tela. É só a vista onde as duas leis
    /// coincidem; o gate `the_ring_follows_the_zoom` cobre o resto.
    const REF_PPW: f64 = ph2d_tool_flip::SIZE_PX_PER_WORLD as f64;

    /// Compara raio de anel com tolerância: a conversão passa por `f32`
    /// (`size_to_world` devolve a largura que o `Point` guarda), então 40 px viram
    /// 20,0000003. Erro de 3e-7 px num cursor — comparar exato seria testar o `f32`.
    #[track_caller]
    fn assert_ring(got: Option<f64>, want: f64, msg: &str) {
        let g = got.unwrap_or_else(|| panic!("sem anel: {msg}"));
        assert!((g - want).abs() < 1e-4, "{msg}: esperado {want}, veio {g}");
    }

    /// O anel aparece nos três modos que TÊM raio, e em nenhum outro. Um anel no Fill
    /// (que não tem raio) seria um controle mentindo — o mesmo princípio do painel modal.
    #[test]
    fn the_ring_is_drawn_only_in_the_modes_that_have_a_radius() {
        for (mode, want) in [
            (FlipMode::Draw, true),
            (FlipMode::Erase, true),
            (FlipMode::Reshape, true),
            (FlipMode::Colorize, true),
            (FlipMode::Fill, false),
            (FlipMode::Select, false),
        ] {
            let r = ring_radius(
                Some(FlipStyleSnapshot {
                    mode,
                    width_px: 40.0,
                    // A borracha lê o raio EFETIVO dela; com o link LIGADO (o default) ele
                    // é o próprio Size do pincel, então os três modos concordam aqui.
                    erase_px: 40.0,
                    ..Default::default()
                }),
                REF_PPW,
            );
            assert_eq!(r.is_some(), want, "modo {mode:?}: deveria ter anel? {want}");
            if want {
                assert_ring(r, 20.0, "no zoom de referencia o anel e METADE do Size");
            }
        }
        assert!(
            ring_radius(None, REF_PPW).is_none(),
            "sem estilo publicado, sem anel"
        );
    }

    /// 🔴 **O anel SEGUE o zoom** (§4.C.6) — porque o Size mede o MUNDO e o anel promete
    /// o que a tinta vai fazer. Aproximar 2× engrossa o traço 2× na tela; um anel de
    /// tamanho fixo ali seria o anel MENTINDO, que é o defeito que este módulo existe
    /// para não ter.
    ///
    /// Mutação que sangra: ignorar o `px_per_world` (voltar a `Size·0,5`, o brush absoluto
    /// de 2026-07-11) — os três zooms passam a devolver o mesmo número.
    #[test]
    fn the_ring_follows_the_zoom() {
        let style = FlipStyleSnapshot {
            mode: FlipMode::Draw,
            width_px: 40.0,
            ..Default::default()
        };
        assert_ring(
            ring_radius(Some(style), REF_PPW),
            20.0,
            "zoom de referencia",
        );
        assert_ring(
            ring_radius(Some(style), REF_PPW * 2.0),
            40.0,
            "2x de zoom tem de DOBRAR o anel — o traço dobra",
        );
        assert_ring(
            ring_radius(Some(style), REF_PPW * 0.5),
            10.0,
            "afastar a camera encolhe o anel junto com a tinta",
        );
    }

    /// 🔴 **Deslinkada, a borracha desenha o anel DELA** (§4.C) — e o pincel segue com o
    /// dele. O anel existe pra dizer o tamanho do que vai acontecer; se ele mostrasse o
    /// Size do pincel enquanto a borracha apaga noutro raio, seria uma mentira na tela.
    ///
    /// Mutação que sangra: o braço `Erase` ler `width_px` (o comportamento pré-§4.C) —
    /// o anel volta a 20 e este teste cai. O irmão acima cobre o caso LINKADO, onde os
    /// dois números coincidem e a mutação NÃO sangra: é preciso o par.
    #[test]
    fn an_unlinked_eraser_draws_its_own_ring() {
        let r = ring_radius(
            Some(FlipStyleSnapshot {
                mode: FlipMode::Erase,
                width_px: 40.0,  // o pincel
                erase_px: 120.0, // a borracha, deslinkada (efetivo resolvido pela tool)
                link_size: false,
                ..Default::default()
            }),
            REF_PPW,
        );
        assert_ring(r, 60.0, "o anel da borracha e METADE do raio DELA");
        // E o pincel não foi contaminado: no Draw o anel volta a ser o do Size.
        let r = ring_radius(
            Some(FlipStyleSnapshot {
                mode: FlipMode::Draw,
                width_px: 40.0,
                erase_px: 120.0,
                link_size: false,
                ..Default::default()
            }),
            REF_PPW,
        );
        assert_ring(r, 20.0, "o anel do PINCEL nao segue a borracha");
    }

    /// **O anel nunca some**: um pincel de 1 px ainda desenha um alvo visível (o piso).
    /// Um cursor que desaparece justo quando o traço fica fino é pior que nenhum.
    #[test]
    fn a_hairline_brush_still_shows_a_visible_ring() {
        let r = ring_radius(
            Some(FlipStyleSnapshot {
                mode: FlipMode::Draw,
                width_px: 1.0,
                ..Default::default()
            }),
            REF_PPW,
        )
        .unwrap();
        assert!(r >= MIN_RING_R_PX, "o anel sumiu num pincel fino: {r}");
    }

    /// O Sculpt usa o MESMO Size do pincel, e no zoom de referência `Size = 200` desenha
    /// 100 px de raio.
    #[test]
    fn the_sculpt_ring_is_half_the_size_at_the_reference_zoom() {
        let r = ring_radius(
            Some(FlipStyleSnapshot {
                mode: FlipMode::Reshape,
                width_px: 200.0,
                ..Default::default()
            }),
            REF_PPW,
        );
        assert_ring(r, 100.0, "Sculpt no zoom de referencia");
    }
}
