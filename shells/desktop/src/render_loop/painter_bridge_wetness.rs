//! Painter on-canvas WETNESS sheen overlay — the damp-paper veil drawn over the wet region while the
//! Watercolor render-path is active (Wetness card). Split from `painter_bridge_overlays.rs` for the
//! HR-18 file-LOC cap. Pure draw: reads the active `PainterTool` moisture view + selection + camera and
//! writes a translucent veil image into the overlay `VectorScene`; mutates no tool or model state.
//! Called once per frame by `painter_bridge_overlays::draw_overlays`.

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
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
        sim.world().get::<crate::Transform>(entity),
        sim.world().get::<ph2d_render::Sprite>(entity),
    ) else {
        return;
    };
    // `base` maps FULL image-px → screen; the sub-image rides it after a translate to the rect origin.
    let base = super::bgremoval_preview::sprite_image_to_screen_affine(
        cw,
        ch,
        tr,
        sprite,
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
    let cwu = cw as usize;
    // Local moisture → veil alpha, then a GENTLE box blur so the damp reads with a soft organic fringe
    // (a stroke's footprint edge / the fresh-vs-decayed step at a junction is ~1 px hard otherwise). The
    // moisture MAP is untouched — this is preview-only. (Safe now that the pour is per-footprint, Bug #9:
    // an earlier over-blur only spread the union-pour RECTANGLE — that root is fixed, so this just softens.)
    // ⚠️ **E ele é construído na densidade em que vai ser VISTO.** O recorte acima resolve o zoom
    // PARA DENTRO; este passo resolve o zoom PARA FORA, que é o caso do log de 2026-08-02: com a
    // pintura de 4096² cabendo numa janela de ~1000 px, o véu era montado em resolução de IMAGEM para
    // ser exibido em resolução de TELA — 16× de trabalho que a GPU descarta ao reduzir. Construir
    // acima da densidade de exibição não é qualidade, é desperdício por definição.
    let step = veil_downscale(base);
    let (veil, vw, vh) = build_veil(
        wet,
        cwu,
        (rx0 as usize, ry0 as usize),
        (rw, rh),
        max_alpha,
        step,
    );
    // O sub-véu cavalga o `base` depois de um translate à origem do rect, e de uma escala que desfaz
    // o downscale — a imagem é `step` vezes menor, então cada texel dela mede `step` px de imagem.
    #[allow(clippy::cast_precision_loss)]
    let affine = base
        * ph2d_vector::Affine::translate((f64::from(rx0), f64::from(ry0)))
        * ph2d_vector::Affine::scale(step as f64);
    vector_scene.draw_image_rgba_transformed(
        &std::sync::Arc::new(veil),
        vw as u32,
        vh as u32,
        affine,
        ph2d_vector::ImageQuality::Low,
    );
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

/// **O BUILD do véu, separado do DRAW** — extraído para poder ser MEDIDO (a sonda
/// `measure_the_wetness_veil`) e, principalmente, para que a decisão *"com que frequência isto é
/// reconstruído?"* tenha um lugar onde ser feita.
///
/// ⚠️ **O custo é do tamanho do RECT CUMULATIVO de umidade**, que só cresce enquanto o artista pinta —
/// e o log do produto de 2026-08-02 o mediu subindo **2,13 → 9,67 → 42,64 ms/quadro** numa sessão
/// (`CHROME wet`), o que a essa altura era 60% do quadro inteiro. Quatro alocações e quatro passadas
/// sobre a região, TODO quadro.
pub(super) fn build_veil(
    wet: &[u8],
    canvas_w: usize,
    (rx0, ry0): (usize, usize),
    (rw, rh): (usize, usize),
    max_alpha: f32,
    step: usize,
) -> (Vec<u8>, usize, usize) {
    const TINT: [u8; 3] = [34, 31, 28]; // LITERAL-COLOR-OK: damp-paper darkening (near-neutral, faint warm)
    const BLUR_R: usize = 4; // LITERAL-PX-OK: gentle veil softening — wet paper has no 1-px hard edges
    let step = step.max(1);
    let (vw, vh) = (rw.div_ceil(step), rh.div_ceil(step));
    if vw == 0 || vh == 0 {
        return (Vec::new(), 0, 0);
    }
    // A MÉDIA do bloco, não uma amostra dele: um `nearest` num campo de umidade produz cintilação na
    // borda conforme a câmera anda, e a média já é metade do desfoque que o véu quer.
    #[allow(clippy::cast_precision_loss)]
    let mut alpha = vec![0.0f32; vw * vh];
    for vy in 0..vh {
        for vx in 0..vw {
            let (y0, x0) = (vy * step, vx * step);
            let (y1, x1) = ((y0 + step).min(rh), (x0 + step).min(rw));
            let mut acc = 0.0f32;
            for y in y0..y1 {
                let src = (ry0 + y) * canvas_w + rx0;
                for x in x0..x1 {
                    acc += f32::from(wet[src + x]);
                }
            }
            let n = ((y1 - y0) * (x1 - x0)).max(1) as f32;
            alpha[vy * vw + vx] = acc / n / 255.0 * max_alpha;
        }
    }
    // O raio do desfoque é em px de IMAGEM; na grade reduzida ele mede `BLUR_R / step`, e o piso de 1
    // mantém a suavização que dá ao véu a franja orgânica (a média do bloco cobre o resto).
    let alpha = box_blur_f32(&alpha, vw, vh, (BLUR_R / step).max(1));
    let mut veil = vec![0u8; vw * vh * 4];
    for (i, &a) in alpha.iter().enumerate() {
        if a > 0.002 {
            let p = i * 4;
            veil[p] = TINT[0];
            veil[p + 1] = TINT[1];
            veil[p + 2] = TINT[2];
            veil[p + 3] = (a * 255.0).clamp(0.0, 255.0) as u8;
        }
    }
    (veil, vw, vh)
}

/// Separable box blur of a `w×h` f32 map (sliding window, O(w·h) — safe on a full-canvas wet map). Edge
/// pixels normalize by the FULL window, so the field FADES softly at the boundary (the damp fringe we want).
fn box_blur_f32(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    if r == 0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    let win = (2 * r + 1) as f32;
    let mut tmp = vec![0.0f32; w * h];
    for y in 0..h {
        let base = y * w;
        let mut acc = 0.0f32;
        for x in 0..(r + 1).min(w) {
            acc += src[base + x];
        }
        for x in 0..w {
            tmp[base + x] = acc / win;
            if x + r + 1 < w {
                acc += src[base + x + r + 1];
            }
            if x >= r {
                acc -= src[base + x - r];
            }
        }
    }
    let mut out = vec![0.0f32; w * h];
    for x in 0..w {
        let mut acc = 0.0f32;
        for y in 0..(r + 1).min(h) {
            acc += tmp[y * w + x];
        }
        for y in 0..h {
            out[y * w + x] = acc / win;
            if y + r + 1 < h {
                acc += tmp[(y + r + 1) * w + x];
            }
            if y >= r {
                acc -= tmp[(y - r) * w + x];
            }
        }
    }
    out
}

#[cfg(test)]
mod measure {
    use std::time::Instant;

    /// **Quanto custa o VÉU, e como ele escala com a região molhada.**
    ///
    /// O log do produto de 2026-08-02 (`PH2D_PAINT_PERF`, sessão do Enio) mediu `CHROME wet` subindo
    /// **2,13 → 9,67 → 42,64 ms/quadro** — a essa altura **60% do quadro inteiro**, com
    /// `frame p50 = 72,3 ms` (~14 fps). ⚠️ Nenhuma sonda de bancada do TOOL podia ver isto: o véu é
    /// desenhado pelo SHELL. Foi o log que o achou, não uma suspeita minha.
    ///
    /// ⚠️ **É o slider `Preview` do card Wetness**: em `0` a função sai na primeira linha, e o default
    /// é 0 — este custo só existe para quem LIGA o preview (estava em 0,300 na sessão medida).
    #[test]
    #[ignore = "measurement, not a gate"]
    fn measure_the_wetness_veil() {
        println!("\no VÉU de umidade — build por quadro\n");
        println!(
            "{:<16} {:>6} {:>10} {:>11} {:>12}",
            "regiao", "passo", "M texels", "build ms", "ns/texel"
        );
        let cw = 4096usize;
        for (rw, rh, step) in [
            (512usize, 512usize, 1usize),
            (2048, 2048, 1),
            (4096, 4096, 1),
            (4096, 4096, 4),
            (4096, 4096, 8),
        ] {
            // Mapa PLAUSÍVEL (molhado no miolo, seco nas bordas): um mapa chapado deixaria o
            // `if a > 0.002` do preenchimento decidir tudo por um ramo só.
            let mut wet = vec![0u8; cw * cw];
            for y in 0..rh {
                for x in 0..rw {
                    let d =
                        ((x as f32 / rw as f32) - 0.5).abs() + ((y as f32 / rh as f32) - 0.5).abs();
                    wet[y * cw + x] = ((1.0 - d).clamp(0.0, 1.0) * 255.0) as u8;
                }
            }
            let mut best = f64::MAX;
            for _ in 0..3 {
                let t0 = Instant::now();
                let (veil, _, _) = super::build_veil(&wet, cw, (0, 0), (rw, rh), 0.3 * 0.55, step);
                let ms = t0.elapsed().as_secs_f64() * 1e3;
                assert!(
                    veil.iter().any(|&b| b > 0),
                    "a fixture tem de MOLHAR — um véu todo zero mediria o ramo vazio"
                );
                best = best.min(ms);
            }
            let n = (rw * rh) as f64;
            println!(
                "{:<16} {step:>6} {:>10.2} {best:>11.3} {:>12.1}",
                format!("{rw}x{rh}"),
                n / 1e6,
                best * 1e6 / n
            );
        }
        println!();
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
    use super::{build_veil, veil_downscale};
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

    /// **O véu grosso cobre a MESMA região e continua molhando** — um passo que perdesse a última
    /// linha/coluna deixaria uma faixa seca na borda do desenho.
    #[test]
    fn a_coarser_veil_still_covers_the_whole_region() {
        let (cw, rw, rh) = (64usize, 50usize, 30usize);
        let wet = vec![200u8; cw * cw];
        let (fine, fw, fh) = build_veil(&wet, cw, (0, 0), (rw, rh), 0.3, 1);
        assert_eq!((fw, fh), (rw, rh));
        for step in [2usize, 3, 4, 7] {
            let (coarse, vw, vh) = build_veil(&wet, cw, (0, 0), (rw, rh), 0.3, step);
            // `div_ceil`: a última coluna parcial TEM de existir, senão a borda fica sem véu.
            assert_eq!((vw, vh), (rw.div_ceil(step), rh.div_ceil(step)));
            assert!(
                vw * step >= rw && vh * step >= rh,
                "o véu grosso não cobre o rect"
            );
            assert!(
                coarse.iter().any(|&b| b > 0),
                "o véu grosso saiu SECO no passo {step}"
            );
            assert!(
                coarse.len() < fine.len(),
                "o passo {step} não economizou nada"
            );
        }
    }
}
