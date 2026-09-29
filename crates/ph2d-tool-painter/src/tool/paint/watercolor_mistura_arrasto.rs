//! **O Smudge arrasta a tinta MOLHADA da sessão** (report do dono, 2026-09-29: *«o Smudge não afeta
//! a mancha de tinta quando a tinta está molhada»*; medido: `0` texels mudados — doc 44 §3).
//!
//! O `smear_wet_base` arrasta a tinta SECA de antes da sessão; a molhada vive noutro plano — a cor
//! depositada da sessão (`stroke_color`) — e ninguém a arrastava. Aqui, a cada dab e ANTES do
//! depósito dele, a tinta que a sessão tinha antes deste traço (o `antes` da mistura) é arrastada do
//! dab anterior para este pela MESMA lei de levantar e pesar do Smudge seco, e o depósito
//! ([`super::watercolor_mistura::deposita`]) recompõe cada texel a partir dela.
//!
//! ## As três escolhas, cada uma medida
//!
//! * ⛔ **Nunca o plano da cor em si.** O depósito recompõe cada texel a partir do `antes` e do
//!   `proprio`, e um arrasto escrito no plano era deitado fora no mesmo dab: a fila saía às riscas,
//!   com texels ao bit o amarelo entre os dabs.
//! * **Em alfa PRÉ-MULTIPLICADO** ([`ph2d_painter_brush::smear_dab_premultiplicado`]): o papel daquele
//!   plano é `0,0,0,0`, e o arrasto dele em alfa recto é média com PRETO — o rasto saía `152,152,129`,
//!   um oliva sujo, contra `219,235,141`.
//! * **O destino é fotografado ANTES de ser arrastado**: sem isso o 1.º depósito ali fotografava o
//!   plano por cima do arrasto. A ORIGEM já está guardada — é a pegada do dab anterior, que passou
//!   por esta mesma porta; ela só sai dessa caixa quando o raio CRESCE entre dabs, e o anel que sobra
//!   fica onde a queda é ~0 (com um taper de `3` diâmetros, capturá-la também deixava a saída ao bit a
//!   mesma — e uma linha que mutação nenhuma mata não é lei).
//!
//! ⚠️ **A seleção, a proteção e o alpha-lock pesam o arrasto** como pesam o seco (`smear_wet_base`
//! repõe a base fora deles pelo mesmo `keep`): sem isto, o artista arrastava tinta molhada para
//! dentro da zona protegida.

use super::watercolor_mistura::PlanosDaMistura;
use ph2d_painter_brush::BrushSpec;

impl PlanosDaMistura {
    /// Um dab do arrasto molhado: `para` é o centro deste dab, `passo` o deslocamento desde o dab
    /// ORIGINAL anterior (`None` no primeiro: só fotografa), `forca` o Smudge e `guarda` o `keep`
    /// dos portões de pintura por texel (`None` = sem portões).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn arrasta(
        &mut self,
        buf: &[u8],
        (fw, fh): (usize, usize),
        para: [f32; 2],
        passo: Option<[f32; 2]>,
        raio: &BrushSpec,
        forca: f32,
        tiling: [bool; 2],
        guarda: Option<&dyn Fn(usize) -> f32>,
    ) {
        let r = raio.radius_px;
        let lo = |v: f32, n: usize| ((v - r).floor().max(0.0) as usize).min(n);
        let hi = |v: f32, n: usize| ((v + r).ceil().max(0.0) as usize).min(n);
        let [x0, y0, x1, y1] = [
            lo(para[0], fw),
            lo(para[1], fh),
            hi(para[0], fw),
            hi(para[1], fh),
        ];
        self.captura(buf, fw, [x0, y0, x1, y1]);
        let Some(s) = passo else {
            return;
        };
        let largura = x1 - x0;
        // Os portões: o `antes` da caixa, para repor por `1 − keep` depois do arrasto.
        let antes_do_arrasto: Option<Vec<u8>> = guarda.map(|_| {
            let mut v = Vec::with_capacity(largura * (y1 - y0) * 4);
            for y in y0..y1 {
                v.extend_from_slice(&self.antes[(y * fw + x0) * 4..(y * fw + x1) * 4]);
            }
            v
        });
        let de = [para[0] - s[0], para[1] - s[1]];
        let _ = ph2d_painter_brush::smear_dab_premultiplicado(
            &mut self.antes,
            fw as u32,
            fh as u32,
            de,
            para,
            raio,
            forca,
            tiling,
        );
        let (Some(keep), Some(v)) = (guarda, antes_do_arrasto) else {
            return;
        };
        for y in y0..y1 {
            for x in x0..x1 {
                let k = keep(y * fw + x).clamp(0.0, 1.0);
                if k >= 1.0 {
                    continue;
                }
                let o = ((y - y0) * largura + (x - x0)) * 4;
                let i = (y * fw + x) * 4;
                let mut repor = [v[o], v[o + 1], v[o + 2], v[o + 3]];
                mistura_premultiplicada(&mut repor, &self.antes[i..i + 4], k);
                self.antes[i..i + 4].copy_from_slice(&repor);
            }
        }
    }
}

/// `dst ← lerp(dst, src, w)` em alfa pré-multiplicado, de volta a recto — a mesma mistura do arrasto,
/// para que repor fora dos portões não escureça o que o arrasto não escureceu.
fn mistura_premultiplicada(dst: &mut [u8; 4], src: &[u8], w: f32) {
    let da = f32::from(dst[3]) / 255.0;
    let sa = f32::from(src[3]) / 255.0;
    let na = da + (sa - da) * w;
    if na > 0.0 {
        for c in 0..3 {
            let pd = f32::from(dst[c]) * da;
            let ps = f32::from(src[c]) * sa;
            dst[c] = ((pd + (ps - pd) * w) / na).round().clamp(0.0, 255.0) as u8;
        }
    }
    dst[3] = (na * 255.0).round().clamp(0.0, 255.0) as u8;
}
