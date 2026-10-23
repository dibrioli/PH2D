//! **A mistura de volta** dos núcleos que levantam e pesam (Smear, Clone, Blur): `dest = lerp(dest,
//! src, w)` em alfa **pré-multiplicado**, de volta a recto.
//!
//! ⚠️ Em alfa recto o transparente guarda `0,0,0,0`, e `lerp(amarelo, 0,0,0,0, ½)` é um amarelo a
//! METADE da luz com metade do alfa: uma orla ESCURA onde o pincel passa entre tinta e vazio numa
//! camada transparente (smoke do dono 23/10, o Smudge da aquarela sobre a poça seca). Pré-multiplicado
//! dá o amarelo com metade do alfa. Sobre opaco (`α = 255` dos dois lados) é a recta, byte a byte.

/// `dst ← lerp(dst, src, w)` pré-multiplicado; `src` recto na escala `0..=255`.
#[inline]
pub(crate) fn mistura_premultiplicada(dst: &mut [u8], src: [f32; 4], w: f32) {
    let da = f32::from(dst[3]) / 255.0;
    let sa = src[3] / 255.0;
    let na = da + (sa - da) * w;
    if na > 0.0 {
        for c in 0..3 {
            let pd = f32::from(dst[c]) * da;
            let ps = src[c] * sa;
            dst[c] = ((pd + (ps - pd) * w) / na).round().clamp(0.0, 255.0) as u8;
        }
    }
    dst[3] = (na * 255.0).round().clamp(0.0, 255.0) as u8;
}

/// A mesma mistura com a fonte em bytes.
#[inline]
pub(crate) fn mistura_premultiplicada_u8(dst: &mut [u8], src: [u8; 4], w: f32) {
    mistura_premultiplicada(dst, src.map(f32::from), w);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BlurKernel, BrushSpec, blur_dab, clone_dab, smear_dab_premultiplicado};

    const AMARELO: [u8; 4] = [250, 225, 20, 255];
    const W: u32 = 24;
    const H: u32 = 16;

    /// Transparente à esquerda (`x < 12`), amarelo opaco à direita: a camada transparente com tinta.
    fn camada() -> Vec<u8> {
        let mut b = vec![0u8; (W * H * 4) as usize];
        for y in 0..H {
            for x in 12..W {
                let i = ((y * W + x) * 4) as usize;
                b[i..i + 4].copy_from_slice(&AMARELO);
            }
        }
        b
    }

    /// Texels meio-transparentes que a mistura deixou mais escuros que a tinta (tolerância de 1).
    fn escurecidos(b: &[u8]) -> (usize, usize) {
        let mut meio = 0;
        let mut escuros = 0;
        for p in b.chunks_exact(4) {
            if p[3] > 0 && p[3] < 255 {
                meio += 1;
                if (0..3).any(|c| i32::from(p[c]) < i32::from(AMARELO[c]) - 1) {
                    escuros += 1;
                }
            }
        }
        (meio, escuros)
    }

    /// ⭐ Nenhum dos três núcleos escurece a tinta que leva para o vazio de uma camada transparente.
    #[test]
    fn levantar_e_pesar_numa_camada_transparente_nao_escurece_a_tinta() {
        let spec = BrushSpec {
            radius_px: 6.0,
            ..Default::default()
        };
        let sem = [false, false];
        let mut casos: Vec<(&str, Vec<u8>)> = Vec::new();
        for off in [-6.0f32, 6.0] {
            let mut b = camada();
            let _ = clone_dab(&mut b, W, H, [12.0, 8.0], [off, 0.0], &spec, 0.6, sem);
            casos.push(("clone", b));
        }
        for k in [BlurKernel::Binomial, BlurKernel::Caixa] {
            let mut b = camada();
            let _ = blur_dab(&mut b, W, H, [12.0, 8.0], &spec, 0.6, sem, k);
            casos.push(("blur", b));
        }
        let mut b = camada();
        let _ = smear_dab_premultiplicado(&mut b, W, H, [16.0, 8.0], [10.0, 8.0], &spec, 0.6, sem);
        casos.push(("smear", b));
        let mut maus = Vec::new();
        for (nome, b) in casos {
            let (meio, escuros) = escurecidos(&b);
            assert!(
                meio > 0,
                "{nome}: CONTROLO — a mistura levou tinta ao vazio"
            );
            if escuros > 0 {
                maus.push(format!("{nome}: {escuros} de {meio}"));
            }
        }
        assert!(maus.is_empty(), "texels que escureceram — {maus:?}");
    }

    /// Opaco dos dois lados, a pré-multiplicada é a recta ao byte (o caminho de sempre não muda).
    #[test]
    fn sobre_opaco_e_a_recta_ao_byte() {
        for d in (0..=255u16).step_by(17) {
            for s in (0..=255u16).step_by(15) {
                for w in [0.0f32, 0.13, 0.5, 0.77, 1.0] {
                    let (d, s) = (d as u8, s as u8);
                    let mut a = [d, 255 - d, d / 2, 255];
                    let src = [s, s / 3, 255 - s, 255];
                    let recta: Vec<u8> = (0..4)
                        .map(|c| {
                            let (x, y) = (f32::from(a[c]), f32::from(src[c]));
                            (x + (y - x) * w).round().clamp(0.0, 255.0) as u8
                        })
                        .collect();
                    mistura_premultiplicada_u8(&mut a, src, w);
                    assert_eq!(a.as_slice(), recta.as_slice(), "d {d} s {s} w {w}");
                }
            }
        }
    }
}
