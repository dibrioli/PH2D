//! Painter on-canvas WETNESS sheen overlay — the damp-paper veil drawn over the wet region while the
//! Watercolor render-path is active (Wetness card). Split from `painter_bridge_overlays.rs` for the
//! HR-18 file-LOC cap. Pure draw: reads the active `PainterTool` moisture view + selection + camera and
//! writes a translucent veil image into the overlay `VectorScene`; mutates no tool or model state.
//! Called once per frame by `painter_bridge_overlays::draw_overlays`.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::PainterTool;
use ph2d_vector::VectorScene;

/// #12a (doc 14): the on-canvas WETNESS sheen — a subtle cool tint over the wet paper (Rebelle
/// "show wetness"), alpha ∝ the local moisture byte. Built over the moisture RECT only (transient,
/// session-scoped) and drawn via the same image→screen affine as the sprite. Read-only.
pub(super) fn draw_wetness_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
) {
    // Wetness-preview strength (Wetness card slider): 0 ⇒ no preview.
    let intensity = painter.wet_preview_intensity();
    if intensity <= 0.0 {
        return;
    }
    let Some((wet, cw, ch, [rx0, ry0, rx1, ry1])) = painter.canvas_wet_view() else {
        return;
    };
    let Some(bits) = hero.gizmo.selection else {
        return;
    };
    let entity = ph2d_ecs::Entity::from_bits(bits);
    let (Some(tr), Some(sprite)) = (
        ph2d_ecs::world_transform(sim.world(), entity),
        ph2d_sprite_screen::sprite_mostrada(sim.world(), entity),
    ) else {
        return;
    };
    // A grelha desta sprite (ADR-0164 F1 passo 6) — ausente = uma célula, e aí o quad do
    // afim é o de sempre, byte-idêntico.
    let sprite_grid = sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
    // `base` maps FULL image-px → screen; the sub-image rides it after a translate to the rect origin.
    let base = ph2d_sprite_screen::sprite_image_to_screen_affine(
        cw,
        ch,
        tr,
        sprite,
        sprite_grid,
        camera,
        window_size,
    );
    // ⚠️ **O véu é recortado à VIEWPORT, e isto é o que o torna barato sem mudar um pixel.** A região
    // de umidade é o rect CUMULATIVO da sessão — ele só cresce, e o log do produto de 2026-08-02 mediu
    // este build subindo `2,13 → 9,67 → 42,64 ms/quadro` (`CHROME wet`), 60% do quadro inteiro, porque
    // ele reconstruía a região INTEIRA a cada frame. O que está fora da janela não é visto por
    // ninguém: recortar é **livre de mudança de aparência por construção**, e troca um custo que
    // cresce com a PINTURA por um limitado pela TELA.
    // O passo vem ANTES do recorte: a grelha de blocos é absoluta (B16.A5), o pan não lhe muda a fase.
    let step = veil_downscale(base);
    let (rx0, ry0, rx1, ry1) =
        clip_to_viewport(base, window_size, (rx0, ry0, rx1, ry1), step, (cw, ch));
    let (rw, rh) = ((rx1 - rx0) as usize, (ry1 - ry0) as usize);
    if rw == 0 || rh == 0 {
        return;
    }
    // Straight-alpha DAMP-PAPER darkening over the wet region (Enio 2026-07-11: "sem tonalidade
    // azulada" — wet paper just darkens, no water-blue sheen). Near-neutral dark tint, a hair warm so it
    // reads as damp paper, not a grey wash; the slider (`intensity`) scales the max veil alpha. Vello
    // premultiplies on draw.
    let max_alpha = intensity * 0.55; // LITERAL-COLOR-OK: slider 0..1 → veil alpha 0..0.55 at full wetness
    // Local moisture → veil alpha, softened (o build mora no tool: `watercolor_backdrop/veu.rs`). O
    // véu é construído na densidade em que vai ser VISTO (`veil_downscale`) e GUARDADO pela revisão da
    // humidade + recorte + passo + força: um quadro parado redesenha o mesmo `StableImage` (B16.10).
    let Some((veil, _, _)) = painter.wetness_veil([rx0, ry0, rx1, ry1], step, max_alpha) else {
        return;
    };
    // O sub-véu cavalga o `base` depois de um translate à origem do rect, e de uma escala que desfaz
    // o downscale — a imagem é `step` vezes menor, então cada texel dela mede `step` px de imagem.
    #[allow(clippy::cast_precision_loss)]
    let affine = base
        * ph2d_vector::Affine::translate((f64::from(rx0), f64::from(ry0)))
        * ph2d_vector::Affine::scale(step as f64);
    vector_scene.draw_stable_image_transformed(&veil, affine, veil_quality(base, step));
}

/// **O retângulo de umidade recortado ao que a JANELA de fato mostra.**
///
/// `base` leva px de imagem → tela; a inversa leva os quatro cantos da janela de volta ao espaço da
/// imagem, e o bbox disso — dilatado por uma margem — é tudo o que pode aparecer. Interseção com o
/// rect de umidade.
///
/// ⚠️ **A margem existe por causa do BLUR:** o véu é borrado por `BLUR_R`, então um texel logo fora da
/// janela contribui para um que está dentro dela. Sem a margem a borda do véu mudaria conforme o
/// artista panha — e uma mudança de aparência que depende da posição da câmera é exatamente o tipo de
/// defeito que ninguém reproduz.
///
/// ⚠️ **Afim degenerada devolve o rect inteiro** (o comportamento de antes): uma inversa que não
/// existe não é motivo para o véu sumir.
fn clip_to_viewport(
    base: ph2d_vector::Affine,
    window_size: WindowSize,
    (rx0, ry0, rx1, ry1): (u32, u32, u32, u32),
    step: usize,
    (cw, ch): (u32, u32),
) -> (u32, u32, u32, u32) {
    /// O raio do desfoque do véu em texels a passo 1 (o `BLUR_R` de `watercolor_backdrop/veu.rs`).
    const BLUR_R: usize = 4; // LITERAL-PX-OK: geometria de recorte
    let step = step.max(1);
    // Folga em px de imagem: o desfoque do passo (`max(BLUR_R/passo, 1)` texels de `passo` px) e um
    // bloco, nunca abaixo de 8 — para a borda do véu não depender do pan.
    #[allow(clippy::cast_precision_loss)]
    let margin = (step * (BLUR_R / step).max(1) + step).max(2 * BLUR_R) as f64;
    // B16.A5: o recorte começa e acaba na grelha ABSOLUTA do passo (ou na borda da tela), então a
    // média de blocos do build não muda de fase quando a vista anda 1 px. A passo 1 é a identidade.
    #[allow(clippy::cast_possible_truncation)]
    let s = step as u32;
    let alinha = |(x0, y0, x1, y1): (u32, u32, u32, u32)| {
        let (ax0, ay0) = (x0 / s * s, y0 / s * s);
        let cima = |v: u32, max: u32, lo: u32| v.div_ceil(s).saturating_mul(s).min(max).max(lo);
        (
            ax0,
            ay0,
            cima(x1, cw.max(x1), ax0),
            cima(y1, ch.max(y1), ay0),
        )
    };
    let inv = base.inverse();
    if !inv.as_coeffs().iter().all(|c| c.is_finite()) {
        return alinha((rx0, ry0, rx1, ry1));
    }
    let (w, h) = (f64::from(window_size.width), f64::from(window_size.height));
    let mut lo = (f64::INFINITY, f64::INFINITY);
    let mut hi = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for (sx, sy) in [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)] {
        let p = inv * ph2d_vector::Point::new(sx, sy);
        lo = (lo.0.min(p.x), lo.1.min(p.y));
        hi = (hi.0.max(p.x), hi.1.max(p.y));
    }
    if !lo.0.is_finite() || !hi.0.is_finite() {
        return alinha((rx0, ry0, rx1, ry1));
    }
    let clamp = |v: f64, max: u32| v.clamp(0.0, f64::from(max)) as u32;
    let cx0 = clamp((lo.0 - margin).floor(), rx1).max(rx0);
    let cy0 = clamp((lo.1 - margin).floor(), ry1).max(ry0);
    let cx1 = clamp((hi.0 + margin).ceil(), rx1).max(cx0);
    let cy1 = clamp((hi.1 + margin).ceil(), ry1).max(cy0);
    alinha((cx0, cy0, cx1, cy1))
}

/// **Quantos px de IMAGEM cabem num px de TELA** — o passo em que o véu é amostrado.
///
/// `base` leva px de imagem → tela, então `sqrt(|det|)` é quantos px de tela um px de imagem ocupa.
/// Com o artista afastado esse número é menor que 1, e construir o véu em resolução de imagem produz
/// detalhe que a GPU **descarta ao reduzir**. O passo é o inverso, truncado.
///
/// ⚠️ **Nunca menor que 1:** aproximar não é motivo para SUPERAMOSTRAR — 1:1 já é toda a densidade que
/// a tela mostra, e o véu é um campo borrado desenhado em `ImageQuality::Low`.
///
/// ⚠️ **E é capeado**, porque um zoom muito longe levaria o véu a um punhado de texels e a borda dele
/// passaria a piscar entre passos vizinhos conforme a câmera se move — trocar custo por cintilação é
/// o negócio errado. O teto vem da MEDIÇÃO: a 4096² um passo de 8 já leva o build a 3,2 ms.
fn veil_downscale(base: ph2d_vector::Affine) -> usize {
    /// Teto do passo (ver acima).
    const MAX_STEP: usize = 8; // LITERAL-PX-OK: geometria de amostragem
    let [a, b, c, d, _, _] = base.as_coeffs();
    let det = (a * d - b * c).abs();
    if !det.is_finite() || det <= 0.0 {
        return 1;
    }
    //  já foi provado finito e positivo acima, então a raiz é finita e positiva.
    let screen_px_per_image_px = det.sqrt();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let step = (1.0 / screen_px_per_image_px).floor() as usize;
    step.clamp(1, MAX_STEP)
}

/// O filtro do véu na tela: aproximado a passo 1 cada texel ocupa vários px e o `Low` (vizinho mais
/// próximo) mostra-o em ESCADA (B20.12) — aí `Medium`. Afastado ou a 1:1 fica o `Low`.
fn veil_quality(base: ph2d_vector::Affine, step: usize) -> ph2d_vector::ImageQuality {
    let [a, b, c, d, _, _] = base.as_coeffs();
    // `|det| > 1` ⇔ `sqrt(|det|) > 1`: um px de imagem ocupa mais que um px de tela.
    let det = (a * d - b * c).abs();
    if step == 1 && det.is_finite() && det > 1.0 {
        ph2d_vector::ImageQuality::Medium
    } else {
        ph2d_vector::ImageQuality::Low
    }
}

#[cfg(test)]
mod clip_tests {
    use super::clip_to_viewport;
    use ph2d_host::WindowSize;
    use ph2d_vector::Affine;

    fn win(w: u32, h: u32) -> WindowSize {
        WindowSize::new(w, h)
    }

    /// **O véu é construído só sobre o que a janela mostra.**
    ///
    /// Mutação que sangra: devolver `(rx0, ry0, rx1, ry1)` sem recortar — a região volta a ser a
    /// pintura inteira, que é o custo que o log do produto mediu em 42,64 ms/quadro.
    #[test]
    fn the_veil_is_clipped_to_what_the_window_shows() {
        // Identidade: 1 px de imagem = 1 px de tela, janela de 800×600 sobre uma pintura de 4096².
        let full = (0u32, 0u32, 4096u32, 4096u32);
        let (x0, y0, x1, y1) =
            clip_to_viewport(Affine::IDENTITY, win(800, 600), full, 1, (4096, 4096));
        let clipped = u64::from(x1 - x0) * u64::from(y1 - y0);
        let whole = 4096u64 * 4096;
        assert!(
            clipped * 20 < whole,
            "o recorte devolveu {clipped} texels de {whole} — a região do véu ainda segue a PINTURA \
             e não a TELA (o custo que o log mediu em 42,64 ms/quadro)"
        );
        // E ele cobre TUDO o que a janela mostra, com a folga do blur: perder um texel visível seria
        // trocar um custo por um buraco no desenho.
        assert!(
            x0 == 0 && y0 == 0 && x1 >= 800 && y1 >= 600,
            "o recorte comeu parte do que a janela mostra: ({x0},{y0})..({x1},{y1})"
        );
    }

    /// **Panhar não pode mudar a BORDA do véu** — daí a margem do blur.
    ///
    /// A janela deslocada por 100 px tem de trazer 100 px novos e manter a folga nos dois lados; sem a
    /// margem, o texel da borda perderia vizinhos e o véu mudaria de aparência conforme a câmera.
    #[test]
    fn panning_keeps_the_blur_margin_on_both_sides() {
        let full = (0u32, 0u32, 4096u32, 4096u32);
        let a = clip_to_viewport(
            Affine::translate((-500.0, -500.0)),
            win(800, 600),
            full,
            1,
            (4096, 4096),
        );
        assert!(
            a.0 < 500 && a.1 < 500,
            "o recorte não deixou folga ANTES do que a janela mostra: {a:?}"
        );
        assert!(
            a.2 > 500 + 800 && a.3 > 500 + 600,
            "o recorte não deixou folga DEPOIS do que a janela mostra: {a:?}"
        );
    }

    /// **A grelha de blocos do véu não muda de fase com o pan** (B16.A5): afastado (passo 4) o véu é a
    /// média de blocos de 4 px contados a partir do recorte; se a origem acompanhasse a câmera px a px,
    /// a franja «borbulharia» ao arrastar. O recorte começa num múltiplo do passo, acaba noutro (ou na
    /// borda da tela) e cobre a janela com a folga do desfoque do passo.
    #[test]
    fn o_veu_nao_depende_da_fase_do_recorte() {
        let full = (0u32, 0u32, 4096u32, 4096u32);
        for step in [2usize, 4, 8] {
            for pan in 0..8 {
                let k = 1.0 / step as f64;
                let base =
                    Affine::scale(k) * Affine::translate((-(1000.0 + f64::from(pan)), -700.0));
                let (x0, y0, x1, y1) =
                    clip_to_viewport(base, win(200, 150), full, step, (4096, 4096));
                let s = step as u32;
                assert!(
                    x0 % s == 0 && y0 % s == 0 && (x1 % s == 0 || x1 == 4096) && y1 % s == 0,
                    "passo {step}, pan {pan}: o recorte ({x0},{y0})..({x1},{y1}) não está na grelha \
                     absoluta do passo — a franja do véu muda de fase quando a vista anda"
                );
                // A folga cobre o desfoque do passo (raio max(4/passo,1) texels de `passo` px).
                let margem = (step * (4 / step).max(1)) as u32;
                let (lo_x, hi_x) = (1000 + pan, 1000 + pan + 200 * s);
                assert!(
                    x0 + margem <= lo_x && x1 >= hi_x + margem,
                    "passo {step}, pan {pan}: a folga não cobre o desfoque: ({x0}..{x1}) para {lo_x}..{hi_x}"
                );
            }
        }
        // Controle: o recorte alinhado não passa da tela.
        let longe = Affine::scale(0.25) * Affine::translate((-4000.0, -4000.0));
        let r = clip_to_viewport(longe, win(400, 300), full, 4, (4096, 4096));
        assert!(
            r.2 <= 4096 && r.3 <= 4096,
            "o recorte passou da tela: {r:?}"
        );
    }

    /// **Afim degenerada devolve a região inteira** — uma inversa que não existe não é motivo para o
    /// véu sumir. (Controle: o caminho de recusa é alcançável e não é o comum.)
    #[test]
    fn a_degenerate_affine_keeps_the_whole_rect() {
        let full = (10u32, 20u32, 300u32, 400u32);
        let squashed = Affine::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(
            clip_to_viewport(squashed, win(800, 600), full, 1, (4096, 4096)),
            full
        );
    }
}

#[cfg(test)]
mod downscale_tests {
    use super::{veil_downscale, veil_quality};
    use ph2d_vector::{Affine, ImageQuality};

    /// **Aproximado, o véu é filtrado; a 1:1 e afastado, não** (B20.12). Mutação que sangra: `Low`
    /// sempre — a 400 % cada texel do véu vira um degrau de 4 px.
    #[test]
    fn o_veu_aproximado_e_filtrado() {
        let q = |a: Affine| veil_quality(a, veil_downscale(a));
        assert!(
            matches!(q(Affine::scale(4.0)), ImageQuality::Medium),
            "a 400 % o véu em vizinho-mais-próximo é uma escada"
        );
        assert!(
            matches!(q(Affine::scale(1.5)), ImageQuality::Medium),
            "a 150 % também"
        );
        assert!(
            matches!(q(Affine::IDENTITY), ImageQuality::Low),
            "a 1:1 cada texel é um px: nada a filtrar"
        );
        assert!(
            matches!(q(Affine::scale(0.25)), ImageQuality::Low),
            "afastado o véu já é a média dos blocos"
        );
        assert!(
            matches!(
                q(Affine::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0])),
                ImageQuality::Low
            ),
            "afim degenerada não escolhe filtro"
        );
    }

    /// **Afastado, o véu é amostrado mais grosso; a 1:1 ou aproximado, nunca.**
    ///
    /// Mutação que sangra: devolver `1` sempre — o véu volta a ser construído em resolução de IMAGEM
    /// para ser exibido em resolução de TELA, os 220 ms/quadro que o log mediu a 4096².
    #[test]
    fn the_veil_is_sampled_at_the_density_it_is_shown_at() {
        assert_eq!(
            veil_downscale(Affine::IDENTITY),
            1,
            "a 1:1 não se subamostra"
        );
        assert_eq!(
            veil_downscale(Affine::scale(4.0)),
            1,
            "aproximar não SUPERAMOSTRA"
        );
        assert_eq!(
            veil_downscale(Affine::scale(0.25)),
            4,
            "4 px de imagem por px de tela"
        );
        assert!(
            veil_downscale(Affine::scale(0.001)) <= 8,
            "o passo é capeado — trocar custo por cintilação de borda é o negócio errado"
        );
        assert_eq!(
            veil_downscale(Affine::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0])),
            1,
            "afim degenerada não pode escolher um passo"
        );
    }
}
