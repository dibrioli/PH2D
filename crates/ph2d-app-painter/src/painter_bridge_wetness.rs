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
    let (rx0, ry0, rx1, ry1) = clip_to_viewport(base, window_size, (rx0, ry0, rx1, ry1));
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
    let step = veil_downscale(base);
    let Some((veil, _, _)) = painter.wetness_veil([rx0, ry0, rx1, ry1], step, max_alpha) else {
        return;
    };
    // O sub-véu cavalga o `base` depois de um translate à origem do rect, e de uma escala que desfaz
    // o downscale — a imagem é `step` vezes menor, então cada texel dela mede `step` px de imagem.
    #[allow(clippy::cast_precision_loss)]
    let affine = base
        * ph2d_vector::Affine::translate((f64::from(rx0), f64::from(ry0)))
        * ph2d_vector::Affine::scale(step as f64);
    vector_scene.draw_stable_image_transformed(&veil, affine, ph2d_vector::ImageQuality::Low);
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
) -> (u32, u32, u32, u32) {
    /// Folga em px de imagem: o raio do blur mais um, para a borda do véu não depender do pan.
    const MARGIN: f64 = 8.0; // LITERAL-PX-OK: BLUR_R (4) com folga — geometria de recorte
    let inv = base.inverse();
    if !inv.as_coeffs().iter().all(|c| c.is_finite()) {
        return (rx0, ry0, rx1, ry1);
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
        return (rx0, ry0, rx1, ry1);
    }
    let clamp = |v: f64, max: u32| v.clamp(0.0, f64::from(max)) as u32;
    let cx0 = clamp((lo.0 - MARGIN).floor(), rx1).max(rx0);
    let cy0 = clamp((lo.1 - MARGIN).floor(), ry1).max(ry0);
    let cx1 = clamp((hi.0 + MARGIN).ceil(), rx1).max(cx0);
    let cy1 = clamp((hi.1 + MARGIN).ceil(), ry1).max(cy0);
    (cx0, cy0, cx1, cy1)
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
        let (x0, y0, x1, y1) = clip_to_viewport(Affine::IDENTITY, win(800, 600), full);
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
        let a = clip_to_viewport(Affine::translate((-500.0, -500.0)), win(800, 600), full);
        assert!(
            a.0 < 500 && a.1 < 500,
            "o recorte não deixou folga ANTES do que a janela mostra: {a:?}"
        );
        assert!(
            a.2 > 500 + 800 && a.3 > 500 + 600,
            "o recorte não deixou folga DEPOIS do que a janela mostra: {a:?}"
        );
    }

    /// **Afim degenerada devolve a região inteira** — uma inversa que não existe não é motivo para o
    /// véu sumir. (Controle: o caminho de recusa é alcançável e não é o comum.)
    #[test]
    fn a_degenerate_affine_keeps_the_whole_rect() {
        let full = (10u32, 20u32, 300u32, 400u32);
        let squashed = Affine::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(clip_to_viewport(squashed, win(800, 600), full), full);
    }
}

#[cfg(test)]
mod downscale_tests {
    use super::veil_downscale;
    use ph2d_vector::Affine;

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
