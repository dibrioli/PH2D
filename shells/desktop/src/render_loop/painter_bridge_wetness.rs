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
    let veil = build_veil(wet, cwu, (rx0 as usize, ry0 as usize), (rw, rh), max_alpha);
    let affine = base * ph2d_vector::Affine::translate((f64::from(rx0), f64::from(ry0)));
    vector_scene.draw_image_rgba_transformed(
        &std::sync::Arc::new(veil),
        rw as u32,
        rh as u32,
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
) -> Vec<u8> {
    const TINT: [u8; 3] = [34, 31, 28]; // LITERAL-COLOR-OK: damp-paper darkening (near-neutral, faint warm)
    const BLUR_R: usize = 4; // LITERAL-PX-OK: gentle veil softening — wet paper has no 1-px hard edges
    let mut alpha = vec![0.0f32; rw * rh];
    for y in 0..rh {
        let src = (ry0 + y) * canvas_w + rx0;
        let dst = y * rw;
        for x in 0..rw {
            alpha[dst + x] = f32::from(wet[src + x]) / 255.0 * max_alpha;
        }
    }
    let alpha = box_blur_f32(&alpha, rw, rh, BLUR_R);
    let mut veil = vec![0u8; rw * rh * 4];
    for (i, &a) in alpha.iter().enumerate() {
        if a > 0.002 {
            let p = i * 4;
            veil[p] = TINT[0];
            veil[p + 1] = TINT[1];
            veil[p + 2] = TINT[2];
            veil[p + 3] = (a * 255.0).clamp(0.0, 255.0) as u8;
        }
    }
    veil
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
            "{:<16} {:>10} {:>11} {:>12}",
            "regiao", "M texels", "build ms", "ns/texel"
        );
        let cw = 4096usize;
        for (rw, rh) in [
            (512usize, 512usize),
            (1024, 1024),
            (2048, 2048),
            (4096, 4096),
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
                let veil = super::build_veil(&wet, cw, (0, 0), (rw, rh), 0.3 * 0.55);
                let ms = t0.elapsed().as_secs_f64() * 1e3;
                assert!(
                    veil.iter().any(|&b| b > 0),
                    "a fixture tem de MOLHAR — um véu todo zero mediria o ramo vazio"
                );
                best = best.min(ms);
            }
            let n = (rw * rh) as f64;
            println!(
                "{:<16} {:>10.2} {best:>11.3} {:>12.1}",
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
