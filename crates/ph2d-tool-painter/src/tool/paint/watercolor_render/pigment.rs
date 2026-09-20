//! **QUE COR TEM O PIGMENTO NESTE PIXEL** — o assunto que o composite resolve antes de o entregar
//! a Beer–Lambert, cortado do [`super`] pelo tecto de LOC e por RESPONSABILIDADE: ali o laço
//! responde *quanta densidade há aqui*, e este ficheiro responde *de que cor ela é*.
//!
//! Três camadas, nesta ordem, cada uma com um dono diferente no modelo:
//! 1. **o dono** — a cor do traço a que o texel pertence, já SUAVIZADA na junção;
//! 2. **o depósito** — o que a brocha de facto largou ali (o mixer molhado), que sobrepõe o dono
//!    na proporção do pigmento depositado;
//! 3. **a água** — o que a re-molhagem dissolveu por cima (mistura SUBTRACTIVA) e o que o anel de
//!    backrun concentrou.

use super::super::watercolor_lut::Luts;

/// Quanto o anel do backrun CONCENTRA o pigmento empurrado (EDGE-2) — re-exportado do pai para
/// manter a lei num sítio só.
use super::BACKRUN_CONC;

/// A cor do pigmento em `(x, y)`, das três camadas acima.
///
/// `dono` já vem suavizado pelo campo por-dono ([`super::super::watercolor_rewet_px::StyleField::sample_color`]);
/// `deposito` é `(buffer, índice do texel × 4)` quando o buffer de cor depositada existe.
///
/// ⚠️ **A ponta do lerp do depósito é o `dono` SUAVIZADO e nunca o discreto** (report do dono
/// 2026-09-20, *"trocar a cor do pincel também criou pixelamento da borda"*): com o mixer armado
/// sobre papel virgem o depósito escreve alfa ZERO e a cor vem INTEIRA do dono — lido nearest no
/// mapa de posse, ele imprimia um degrau de UM pixel na junção entre dois traços de cores
/// diferentes. Cada canal lê o próprio valor antes de o escrever.
#[inline]
pub(super) fn pigmento_do_pixel(
    dono: [u8; 3],
    deposito: Option<(&[u8], usize)>,
    dissolve: f32,
    bleed: [f32; 3],
    backrun: f32,
    lut: &Luts,
) -> [u8; 3] {
    // ── 1+2. Depositado vs dono, por lerp PROPORCIONAL (`ca8/255`, a fração de pigmento do
    // depósito). A janela fixa `COL_LO..COL_HI` (take 6) cruzava em ~1px na borda do footprint =
    // a LINHA DURA da junção (take 10, sonda [maps]); depositado == raw (mixer-off) ⇒ byte-idêntico.
    let mut pig = dono;
    if let Some((buf, ci)) = deposito {
        let ca8 = buf[ci + 3];
        if ca8 == u8::MAX {
            pig = [buf[ci], buf[ci + 1], buf[ci + 2]];
        } else if ca8 > 0 {
            let w = f32::from(ca8) / 255.0;
            for (c, p) in pig.iter_mut().enumerate() {
                let b = f32::from(*p);
                *p = (b + (f32::from(buf[ci + c]) - b) * w + 0.5) as u8;
            }
        }
    }
    // ── 3a. Wet-on-wet DISSOLVE: the lifted paint's colour (diffused through the wet region) tints
    // the wash's pigment — the old colour bleeds into and beyond its own footprint. SUBTRACTIVE mix
    // (absorbance-space geometric mean, via the ln/exp LUTs): paints mix like pigments, not light —
    // the linear sRGB lerp desaturated the blend toward the paper's cream ("pálida e amarelada sem
    // Pigment", Enio 2026-07-06). Pigment ON still adds its full RYB pass on top, unchanged.
    if dissolve > 0.0 {
        for (p, &bl) in pig.iter_mut().zip(bleed.iter()) {
            let a = -lut.lnl[*p as usize];
            let bi = (bl + 0.5).clamp(0.0, 255.0) as usize;
            let b = -lut.lnl[bi];
            let mag = a + (b - a) * dissolve;
            *p = lut.l2s_byte(lut.exp_mag(mag));
        }
    }
    // ── 3b. Backrun CONCENTRATION (EDGE-2): the ring's pigment is the pushed paint CONCENTRATED —
    // Beer–Lambert saturates at the pigment colour, so density alone can never render darker than
    // the wash the pigment came from; the "severely darkened edge" needs a darker floor.
    if backrun > 0.0 {
        for p in &mut pig {
            let a = -lut.lnl[*p as usize] * (1.0 + BACKRUN_CONC * backrun);
            *p = lut.l2s_byte(lut.exp_mag(a));
        }
    }
    pig
}
