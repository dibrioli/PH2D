//! [`BrushSpec::default`] — the brush the app boots with, and the baseline every "byte-identical"
//! claim in the Painter is measured against.
//!
//! Split out of `spec.rs` (the workspace LOC cap). It earns its own file: this is not merely a `Default`
//! impl, it is the DEFINITION of the neutral brush — the one whose stroke must be indistinguishable
//! from a build in which Impasto, the watercolor optics and the material never existed. Every default
//! here is load-bearing, and several of them are load-bearing in a way a reader would not guess (the
//! neutral `impasto_roughness` is the geometric midpoint that reproduces the old hard-coded exponent,
//! to the float).

use crate::blend::BrushBlend;
use crate::falloff::Falloff;
use crate::falloff_curve::FalloffCurve;
use crate::height::{DepthSource, DrawTo};
use crate::line_kind::LineKind;
use crate::spec::BrushSpec;
use crate::stroke_method::{JitterUnit, StrokeMethod};
use crate::symmetry::SymmetrySettings;
use crate::texture::TextureSettings;

impl Default for BrushSpec {
    /// A soft round black brush, matching Blender's default "TexDraw": smooth falloff, full
    /// strength/flow, 10% spacing.
    fn default() -> Self {
        Self {
            radius_px: 25.0,
            hardness: 0.0,
            strength: 1.0,
            flow: 1.0,
            spacing: 0.10,
            blend: BrushBlend::Mix,
            falloff: Falloff::Smooth,
            jitter: 0.0,
            color: [0.0, 0.0, 0.0],
            custom_falloff: FalloffCurve::default(),
            stroke_method: StrokeMethod::Space,
            // Grid Stamp: 32x32 px, sem deslocamento. 32 e' a celula de tile mais comum do 2D
            // (e o dobro do 16 de sprite classico), entao o primeiro carimbo cai numa grade que o
            // artista reconhece antes de tocar num slider.
            grid_cell_px: [32.0, 32.0],
            grid_offset_px: [0.0, 0.0],
            grid_fit: 0.0,
            space_attenuation: false, // Adjust Strength off by default (Enio 2026-06-24)
            accumulate: false,
            style_solid: false,
            line_kind: LineKind::None,
            // Sketchy: o alcance e a densidade são o ponto de operação do Krita traduzido para as
            // unidades desta casa (alcance em DIÂMETROS; densidade no teto ORÇADO da W0.3).
            sketchy_reach: 1.0,
            // Metade do teto medido: uma teia que se vê sem cobrir o desenho. ⚠️ É um default de
            // LOOK — o smoke é quem o julga, e o teto (o que o produto SUSTENTA) é outra pergunta.
            sketchy_density: 0.2,
            thread_width_px: 1.0,
            thread_opacity: 0.25,
            sketchy_magnetify: true,
            // Wire: a janela tem de ENCHER dentro de um gesto normal, senão o artista vê o arame
            // *"formar tarde"* — que é a consequência que o próprio manual do Krita descreve para uma
            // história longa. Num pincel de fábrica (diâmetro 24 px) seis diâmetros são 144 px de
            // arco, e um gesto de algumas centenas de px mostra o laço inteiro. ⚠️ É um default de
            // LOOK, e o teto (o que o produto SUSTENTA, medido em 24) é outra pergunta.
            wire_history: 6.0,
            wire_connection_line: true,
            // ⚠️ Estes três já nascem VIVOS, ao contrário do `spray_count`, e a razão é a mesma que
            // fez o Spray precisar de armar um default: **um tipo escolhido tem de FAZER alguma
            // coisa**. Aqui os campos são exclusivos do Ribbon (nenhum outro modo os lê), então
            // escrever o ponto de operação no default não impõe política a ninguém — o que seria
            // errado é o artista escolher `Ribbon` e ver o traço de sempre.
            //
            // ⚠️ **E são decisão de LOOK; o SMOKE é quem os julga.** `0,45` são ~0,11 s de atraso
            // (~260 px atrás num gesto ligeiro de 2 340 px/s, a régua medida do `SPEED_LOOKAHEAD_S`)
            // e `0,30` põe o `ζ` em ~0,70 — sub-amortecido, que é o chicote na saída da curva.
            ribbon_weight: 0.45,
            ribbon_friction: 0.30,
            // A gravidade nasce em ZERO: uma fita que atrasa já é uma fita, e pender é a segunda
            // descoberta. Um slider cujo neutro é 0 não é controle morto — é o `jitter`.
            ribbon_gravity: 0.0,
            dash_ratio: 1.0,
            dash_samples: 20,
            jitter_unit: JitterUnit::Brush,
            jitter_absolute_px: 0.0,
            input_samples: 1,
            stabilizer: 0.5,
            airbrush_rate_s: 0.1,
            edge_to_edge: false,
            // Off: no taper window, so `Taper::width` returns exactly 1.0 and no dab is touched.
            taper: crate::taper::Taper::default(),
            texture: TextureSettings::default(),
            grain_depth: 1.0,
            shape: TextureSettings::default(),
            // Nothing reads `Dab::dir` by default (no rake, no chisel), so the stroke need not warm up.
            needs_heading: false,
            dab_flatten: 0.0,
            dab_angle_deg: 0,
            color_jitter_enabled: false,
            color_jitter_hue: 0.0,
            color_jitter_sat: 0.0,
            color_jitter_val: 0.0,
            jitter_scale: 0.0,
            jitter_rotate: 0.0,
            jitter_spacing: 0.0,
            // Uma marca por ponto do caminho — o neutro do spray, byte-idêntico ao mundo pré-W5.
            spray_count: 1,
            symmetry: SymmetrySettings::default(),
            // Watercolor: the `watercolor` gate (OFF) is what guarantees a byte-identical default
            // brush — so the params carry sensible *when-enabled* values, not neutral zeros, and
            // toggling "Wet edges" on shows an effect immediately.
            watercolor: false,
            edge_gain: 1.5,
            edge_spread: 7.0,
            smooth_edges: true,
            granulation: 0.3,
            pigment: false,
            pigment_mix: 0.5,
            // Render-path optics (wet_edges defaults); inert unless `watercolor` is on.
            fill: 0.12,
            depth: 1.2,
            // Pigment body: lifts light-valued pigments so they deposit at their hue (not near-invisible
            // over white). Inert unless `watercolor` is on → a plain brush is byte-identical regardless.
            opacity: 0.4,
            warp: 6.0,
            wet_smudge: 0.0,   // off → byte-identical (the smear path is skipped)
            wet_rewet: 0.0,    // off → byte-identical (the rewet path is skipped)
            wet_charge: 1.0,   // full fresh paint → mixer skipped → byte-identical
            wet_dilution: 0.0, // full-strength deposit → byte-identical
            wet_pull: 0.0,     // no colour carry (inert unless charge < 1)
            // Paper slot inactive by default (the render-path falls back to its built-in paper noise);
            // granulation follows the paper's tooth until the artist points it at the Grain slot map.
            paper: TextureSettings::default(),
            granulation_use_paper: true,
            paper_depth: 1.0,
            watercolor_shape_auto: true, // built-in feather silhouette (byte-identical default)
            // Impasto: the `impasto` gate (OFF) is what guarantees the byte-identical default — the
            // params below carry sensible *when-enabled* values (a visible ridge the moment the
            // artist ticks the box), not neutral zeros. `impasto_off_is_byte_identical` locks that.
            impasto: false,
            impasto_smooth_edges: true,
            // Enio's dialled-in defaults (2026-07-12, after the smoke): thick paint (Depth 1) whose
            // relief OBEYS the falloff (Body 0 — the rounded ridge he asked back for), settled soft
            // (Smoothing 1). They are the artist's numbers, not the engine's: the `impasto` gate below
            // is what keeps the brush byte-identical until he ticks the box.
            impasto_depth: 1.0,
            // O filme do substrato nasce DESLIGADO, como tudo o que este pincel só faz sob pedido — e é
            // isso que mantém o depósito byte-idêntico até o artista subir o Paint na seção Paper.
            film_depth: 0.0,
            impasto_source: DepthSource::Uniform,
            impasto_draw_to: DrawTo::ColorAndDepth,
            impasto_smoothing: 1.0,
            impasto_body: 0.0,
            // **1.0 — a faca leva a MASSA junto** (Enio, 2026-07-18). Era `0.0`, com o racional "o Smear
            // arrasta a COR e deixa o corpo onde está" — e o preço disso, medido, é que o pigmento viaja e
            // o relevo **para na fronteira exata do traço original**: numa faca cruzando um traço grosso,
            // pigmento alcança x=99 e relevo x=41, que é a borda de onde o traço foi pintado. O artista vê
            // tinta empurrada que não tem corpo. Tinta tem massa; empurrar tinta move as duas coisas.
            impasto_plow: 1.0,
            impasto_push: 0.0, // sem deslocamento: um traço empilha sobre o que já estava (byte-idêntico)
            // O material NEUTRO — o passe de luz de antes deste módulo, à risca. `roughness: 0.5` cai
            // EXATAMENTE no expoente 24 que estava cravado (a média geométrica de 6 e 96), então um
            // pincel default é byte-idêntico ao build pré-material. `shine: 0.7` era o default global.
            impasto_shine: 0.7,
            impasto_roughness: crate::material::Material::NEUTRAL.roughness,
            impasto_metallic: crate::material::Material::NEUTRAL.metallic,
            impasto_wax: crate::material::Material::NEUTRAL.wax,
            impasto_wax_color: crate::material::Material::NEUTRAL.wax_color,
        }
    }
}
