//! B32.4: as rotas em cache (carimbo em rampa e carimbo colorido) compõem pela MESMA mistura de pigmento do caminho
//! por píxel (`blend_over_pigment_modo`): com Pigment > 0 o azul sobre o vermelho é a espectral nas três.

use crate::blend_vazio::blend_over_pigment_modo;
use crate::spec::BrushSpec;
use crate::stamp::sample_mask;
use crate::texture::{ImageMask, TextureKind};

const W: u32 = 64;
const LADO: u32 = 64;
const C: [f32; 2] = [32.5, 32.5];
const VERMELHO: [u8; 4] = [191, 0, 18, 255];
const AZUL: [f32; 3] = [23.0 / 255.0, 101.0 / 255.0, 179.0 / 255.0];
const FORCA: f32 = 0.5;

fn pincel(mix: f32) -> BrushSpec {
    let mut s = BrushSpec::default();
    s.shape.kind = TextureKind::Image;
    s.color = AZUL;
    s.pigment = true;
    s.pigment_mix = mix;
    s
}

fn tela() -> Vec<u8> {
    VERMELHO.repeat((W * W) as usize)
}

fn centro(buf: &[u8]) -> [u8; 4] {
    let i = (32 * W as usize + 32) * 4;
    [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
}

/// A lei por píxel, com a cobertura `a` do texel do centro.
fn lei(s: &BrushSpec, a: f32) -> [u8; 4] {
    let dst = VERMELHO.map(|v| f32::from(v) / 255.0);
    let o = blend_over_pigment_modo(s.blend, dst, AZUL, a, s.pigmento(), s.blend_sobre_vazio);
    o.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
}

fn perto(a: [u8; 4], b: [u8; 4]) -> bool {
    (0..4).all(|k| a[k].abs_diff(b[k]) <= 1)
}

#[test]
fn as_rotas_em_cache_misturam_o_pigmento_como_o_caminho_por_pixel() {
    let lum = vec![255u8; (LADO * LADO) as usize];
    let camada = ImageMask {
        lum: &lum,
        width: LADO,
        height: LADO,
    };
    for mix in [0.0f32, 0.5, 1.0] {
        let s = pincel(mix);
        let mask = crate::stamp::render_stamp_mask(&s, None, Some(&camada), None, LADO);
        let a = sample_mask(mask.nivel_para(2.0 * 10.0 * mask.reach()), 0.0, 0.0) * FORCA;
        assert!(a > 0.45, "o centro do carimbo cobre: {a}");
        let esperado = lei(&s, a);

        let mut rampa = tela();
        let azul = vec![[AZUL[0], AZUL[1], AZUL[2], 1.0]; 256];
        let _ = crate::blit_stamp_ramped(
            &mut rampa,
            W,
            W,
            C,
            10.0,
            &mask,
            &s,
            FORCA,
            false,
            &azul,
            crate::RampAlphaMode::None,
            None,
        );
        let cor = crate::render_color_stamp_mask(
            &s,
            &[camada],
            &[AZUL],
            &[None],
            &[1.0],
            None,
            None,
            LADO,
        );
        let mut colorido = tela();
        let _ = crate::blit_color_stamp(
            &mut colorido,
            W,
            W,
            C,
            10.0,
            &cor,
            &s,
            FORCA,
            false,
            [0.0; 3],
        );
        let (r, c) = (centro(&rampa), centro(&colorido));
        assert!(
            perto(r, esperado),
            "rampa mix {mix}: {r:?} × lei {esperado:?}"
        );
        assert!(
            perto(c, esperado),
            "colorido mix {mix}: {c:?} × lei {esperado:?}"
        );
        if mix > 0.0 {
            let simples = lei(&pincel(0.0), a);
            assert!(
                !perto(esperado, simples),
                "o controlo: a mistura tem de diferir do over ({esperado:?})"
            );
        }
    }
}
