//! **L7 — o efeito do pincel compõe no SEU modo** (`docs/Painter/Pinceis/06_LEIS_DAS_LACUNAS.md` §2 e §10.3): o
//! Rebelle e o Krita compõem o Effect com a fórmula canónica do W3C *Compositing and Blending* em sRGB CODIFICADO
//! (o linear erra 2–56 níveis), misturada a `α` por passagem. Aqui o produto (o carimbo do Digital, pela porta do
//! ponteiro) contra uma escrita INDEPENDENTE das fórmulas do W3C, e contra as fotos do Rebelle a 10 passagens.
//! O Wet Paint compõe a sessão pela MESMA `blend_over` (`wetpaint/composite.rs`; `wetpaint/blend_tests.rs`).

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushBlend, BrushSpec, Falloff};

const AZUL: [u8; 3] = [23, 101, 179];

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// `n` toques do pincel AZUL no modo `m` com a força `alfa` sobre uma tela opaca da cor `base`; o texel do centro.
fn toques(base: [u8; 3], m: BrushBlend, alfa: f32, n: usize) -> [u8; 3] {
    let mut t = PainterTool::default();
    let (w, h) = (64u32, 64u32);
    let mut px = vec![0u8; (w * h * 4) as usize];
    for p in px.as_chunks_mut::<4>().0 {
        *p = [base[0], base[1], base[2], 255];
    }
    t.set_source(px, w, h);
    let b = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: AZUL.map(|c| f32::from(c) / 255.0),
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_brush_blend(m as u8);
    t.set_brush_strength(alfa);
    for _ in 0..n {
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    }
    let o = ((32 * w + 32) * 4) as usize;
    [t.canvas_rgba[o], t.canvas_rgba[o + 1], t.canvas_rgba[o + 2]]
}

// ── O W3C, escrito AQUI de novo (Compositing and Blending Level 1, §9) — nunca o do produto ──────────────────
fn lum(c: [f64; 3]) -> f64 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}
fn clip(c: [f64; 3]) -> [f64; 3] {
    let l = lum(c);
    let n = c[0].min(c[1]).min(c[2]);
    let x = c[0].max(c[1]).max(c[2]);
    let mut o = c;
    if n < 0.0 {
        o = o.map(|v| l + (v - l) * l / (l - n));
    }
    if x > 1.0 {
        o = o.map(|v| l + (v - l) * (1.0 - l) / (x - l));
    }
    o
}
fn set_lum(c: [f64; 3], l: f64) -> [f64; 3] {
    let d = l - lum(c);
    clip(c.map(|v| v + d))
}
fn sat(c: [f64; 3]) -> f64 {
    c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2])
}
fn set_sat(c: [f64; 3], s: f64) -> [f64; 3] {
    let (mx, mn) = (c[0].max(c[1]).max(c[2]), c[0].min(c[1]).min(c[2]));
    if mx <= mn {
        return [0.0; 3];
    }
    c.map(|v| (v - mn) * s / (mx - mn))
}

/// `B(cb, cs)` do W3C para os modos que o Effect dos oráculos usa (e os irmãos separáveis).
fn w3c(m: BrushBlend, b: [f64; 3], s: [f64; 3]) -> [f64; 3] {
    let sep = |f: &dyn Fn(f64, f64) -> f64| [f(b[0], s[0]), f(b[1], s[1]), f(b[2], s[2])];
    let hard = |b: f64, s: f64| {
        if s <= 0.5 {
            b * 2.0 * s
        } else {
            let s2 = 2.0 * s - 1.0;
            b + s2 - b * s2
        }
    };
    match m {
        BrushBlend::Multiply => sep(&|b, s| b * s),
        BrushBlend::Screen => sep(&|b, s| b + s - b * s),
        BrushBlend::Overlay => sep(&|b, s| hard(s, b)),
        BrushBlend::HardLight => sep(&hard),
        BrushBlend::Darken => sep(&|b, s| b.min(s)),
        BrushBlend::Lighten => sep(&|b, s| b.max(s)),
        BrushBlend::ColorDodge => sep(&|b, s| {
            if b == 0.0 {
                0.0
            } else if s >= 1.0 {
                1.0
            } else {
                (b / (1.0 - s)).min(1.0)
            }
        }),
        BrushBlend::ColorBurn => sep(&|b, s| {
            if b >= 1.0 {
                1.0
            } else if s <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - b) / s).min(1.0)
            }
        }),
        BrushBlend::Difference => sep(&|b, s| (b - s).abs()),
        BrushBlend::Exclusion => sep(&|b, s| b + s - 2.0 * b * s),
        // Os aditivos do Effect («Dodge» = ADDITIVE, «Burn» = LINEAR_BURN) — fora do W3C, as somas canónicas.
        BrushBlend::Add => sep(&|b, s| (b + s).min(1.0)),
        BrushBlend::LinearBurn => sep(&|b, s| (b + s - 1.0).max(0.0)),
        BrushBlend::Hue => set_lum(set_sat(s, sat(b)), lum(b)),
        BrushBlend::Saturation => set_lum(set_sat(b, sat(s)), lum(b)),
        BrushBlend::Color => set_lum(s, lum(b)),
        BrushBlend::Luminosity => set_lum(b, lum(s)),
        outro => unreachable!("fora da tabela: {outro:?}"),
    }
}

/// A fórmula iterada `n` vezes em sRGB codificado, a tela de 8 bits a arredondar a cada passagem.
fn previsto(base: [u8; 3], m: BrushBlend, alfa: f64, n: usize) -> [f64; 3] {
    let s = AZUL.map(|c| f64::from(c) / 255.0);
    let mut c = base.map(f64::from);
    for _ in 0..n {
        let b = c.map(|v| v / 255.0);
        let f = w3c(m, b, s);
        c = [0, 1, 2].map(|k| ((b[k] + alfa * (f[k] - b[k])) * 255.0).round());
    }
    c
}

const MODOS: [BrushBlend; 16] = [
    BrushBlend::Multiply,
    BrushBlend::Screen,
    BrushBlend::Overlay,
    BrushBlend::HardLight,
    BrushBlend::Darken,
    BrushBlend::Lighten,
    BrushBlend::ColorDodge,
    BrushBlend::ColorBurn,
    BrushBlend::Difference,
    BrushBlend::Exclusion,
    BrushBlend::Add,
    BrushBlend::LinearBurn,
    BrushBlend::Hue,
    BrushBlend::Saturation,
    BrushBlend::Color,
    BrushBlend::Luminosity,
];

/// ⭐ **`o_efeito_do_pincel_compoe_no_seu_modo`**: base (o vermelho do Marker Bullet do Rebelle, o cinza 50 %, o
/// pastel) + o efeito AZUL nos α por passagem medidos (0,056 · 0,088 · 0,216) e o do Krita (0,90), 1× e 10×:
/// cada canal a ≤ 2/255 da fórmula W3C em sRGB codificado. A mesma conta em luz LINEAR erra 2–56 níveis (06 §2).
#[test]
fn o_efeito_do_pincel_compoe_no_seu_modo() {
    let bases = [[191u8, 0, 18], [128, 128, 128], [230, 184, 161]];
    let mut falhas = Vec::new();
    for m in MODOS {
        for base in bases {
            for (alfa, n) in [(0.056, 1), (0.088, 1), (0.216, 1), (0.9, 1), (0.056, 10), (0.216, 10)] {
                let nosso = toques(base, m, alfa, n);
                let p = previsto(base, m, f64::from(alfa), n);
                let erro = (0..3)
                    .map(|k| (f64::from(nosso[k]) - p[k]).abs())
                    .fold(0.0, f64::max);
                if erro > 2.0 {
                    falhas.push(format!("{m:?} base {base:?} α {alfa} ×{n}: {nosso:?} × W3C {p:?}"));
                }
            }
        }
    }
    assert!(falhas.is_empty(), "fora da fórmula:\n{}", falhas.join("\n"));
}

/// ⭐ **Contra as FOTOS do Rebelle** (06 §10.3: 10 passagens do Effect sobre cinza e pastel, o α ajustado por
/// eles): Hue, Luminosity e Overlay batem com a fórmula; a nossa tinta a 10× fica a ≤ erro do ajuste + 2 níveis
/// da foto. (Saturation, Linear Burn e Dodge do Rebelle NÃO são as fórmulas do nome quando acumulam — registado
/// no 03 §4 L7, não se persegue.)
#[test]
fn o_efeito_a_dez_passagens_e_o_do_rebelle() {
    // (modo, α/passagem, foto 10× sobre cinza, foto 10× sobre pastel, erro do ajuste do 06)
    let fotos = [
        (BrushBlend::Hue, 0.055, [128u8, 128, 128], [208u8, 191, 186], 0.45),
        (BrushBlend::Luminosity, 0.055, [111, 111, 111], [182, 136, 113], 1.07),
        (BrushBlend::Overlay, 0.21, [19, 83, 203], [133, 148, 217], 1.84),
    ];
    for (m, alfa, cinza, pastel, ajuste) in fotos {
        for (base, foto) in [([128u8, 128, 128], cinza), ([230, 184, 161], pastel)] {
            let nosso = toques(base, m, alfa, 10);
            let erro = (0..3)
                .map(|k| nosso[k].abs_diff(foto[k]))
                .max()
                .unwrap_or(0);
            assert!(
                f64::from(erro) <= ajuste + 2.0,
                "{m:?} sobre {base:?}: nosso {nosso:?} × Rebelle {foto:?} (erro {erro})"
            );
        }
    }
}
