//! **Prancha B19.2** (Chromatic Aberration no modelo da placa + a janela suja sem costura), 3 fundos ×
//! 1080p/4K, em `pranchas/B19a/`. O «antes» no mesmo processo:
//! - CA: `ca_lei_antiga` é o `apply_chromatic_aberration` de d469f3100d verbatim (centro da JANELA
//!   `(w−1)/2`, normalizador do canto, gather para fora, `round`), sem o premultiplicar: o fundo é
//!   opaco, e com `α = 1` o premultiplicar é a identidade. A lei de hoje pelo mesmo caminho dá o
//!   `composite` AO BIT (a montagem é a do compositor).
//! - Costura: o caminho antigo da janela suja = `compor_regiao` sem halo (o que o `composite_region`
//!   fazia), em ladrilhos de 96 px colados sobre a tela; o depois é o `composite_region` de hoje.

use super::prancha_b19a_tests::{camada, fundo, monta};
use super::*;
use ph2d_painter_effects::adjustments::{
    AdjustWindow, AdjustmentKind, AdjustmentParams, ChromaticAberrationParams, GaussianBlurParams,
    apply_chromatic_aberration,
};

fn ca_lei_antiga(p: &ChromaticAberrationParams, acc: &mut [[f32; 4]], w: i32, h: i32) {
    let cx = (w as f32 - 1.0) * 0.5;
    let cy = (h as f32 - 1.0) * 0.5;
    let corner = (cx * cx + cy * cy).sqrt().max(1e-3);
    let scale = |shift: f32| 1.0 + shift / corner;
    let (sr, sg, sb) = (
        scale(p.red_shift),
        scale(p.green_shift),
        scale(p.blue_shift),
    );
    let src = acc.to_vec();
    let gather = |dx: f32, dy: f32, s: f32, ch: usize| -> f32 {
        let sx = ((cx + dx * s).round() as i32).clamp(0, w - 1);
        let sy = ((cy + dy * s).round() as i32).clamp(0, h - 1);
        src[(sy * w + sx) as usize][ch]
    };
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let o = &mut acc[(y * w + x) as usize];
            o[0] = gather(dx, dy, sr, 0);
            o[1] = gather(dx, dy, sg, 1);
            o[2] = gather(dx, dy, sb, 2);
            o[3] = gather(dx, dy, 1.0, 3);
        }
    }
}

/// Anéis de cor (tinta sobre o fundo): a franja da lente vê-se nas bordas.
fn pilha(w: u32, h: u32, bg: &[u8]) -> (LayerStack, MapPixelSource) {
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
    let aneis = camada(w, h, |x, y| {
        let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
        let k = (d / (h as f32 / 24.0)) as u32;
        k.is_multiple_of(2)
            .then_some([[30, 30, 30, 255], [250, 250, 250, 255]][(k / 2 % 2) as usize])
    });
    let mut s = LayerStack::new();
    let mut src = MapPixelSource::default();
    let f = s.add_raster("fundo", w, h).unwrap();
    src.insert(
        f,
        LayerImage {
            width: w,
            height: h,
            rgba8: bg.to_vec(),
        },
    );
    let a = s.add_raster("aneis", w, h).unwrap();
    src.insert(a, aneis);
    (s, src)
}

fn aberracao(w: u32, h: u32, bg: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let (mut s, src) = pilha(w, h, bg);
    let por_baixo = composite(&s, &src, w, h);
    let p = ChromaticAberrationParams {
        red_shift: 24.0,
        green_shift: 0.0,
        blue_shift: -24.0,
        falloff_center: 0.0,
    };
    let adj = s
        .add_adjustment(AdjustmentKind::ChromaticAberration)
        .unwrap();
    s.adjustment_mut(adj).unwrap().params = AdjustmentParams::ChromaticAberration(p);
    let depois = composite(&s, &src, w, h);
    let acc0: Vec<[f32; 4]> = por_baixo
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| [0, 1, 2, 3].map(|k| decode_byte(c[k])))
        .collect();
    let mut hoje = acc0.clone();
    apply_chromatic_aberration(&p, &mut hoje, AdjustWindow::full(w, h));
    assert_eq!(
        encode(&hoje),
        depois,
        "a montagem do oráculo é a do compositor"
    );
    let mut antes = acc0;
    ca_lei_antiga(&p, &mut antes, w as i32, h as i32);
    (encode(&antes), depois)
}

fn costura(w: u32, h: u32, bg: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let (mut s, src) = pilha(w, h, bg);
    let adj = s.add_adjustment(AdjustmentKind::GaussianBlur).unwrap();
    s.adjustment_mut(adj).unwrap().params =
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 12.0 });
    let tela = composite(&s, &src, w, h);
    let (mut antes, mut depois) = (tela.clone(), tela);
    let lado = 96u32;
    let (x0, y0) = ((w - h / 2) / 2, h / 4);
    for ty in (y0..y0 + h / 2).step_by(lado as usize) {
        for tx in (x0..x0 + h / 2).step_by(lado as usize) {
            let r = Region {
                x: tx,
                y: ty,
                w: lado,
                h: lado,
            };
            let (velho, _) = compor_regiao(&s, &src, w, h, r, None);
            let novo = composite_region(&s, &src, w, h, r);
            for (dst, v) in [(&mut antes, encode(&velho)), (&mut depois, novo)] {
                for y in 0..lado {
                    let i = (((ty + y) * w + tx) * 4) as usize;
                    let j = (y * lado * 4) as usize;
                    dst[i..i + (lado * 4) as usize].copy_from_slice(&v[j..j + (lado * 4) as usize]);
                }
            }
        }
    }
    (antes, depois)
}

#[test]
#[ignore = "prancha B19.2: grava as PNG em docs/Painter/missao_4k/pranchas/B19a/"]
fn prancha_b19_2_lente_e_costura() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B19a"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let bg = fundo(w, h, qual);
            let casos = [
                ("lente_centro_da_tela", aberracao(w, h, &bg)),
                ("desfoque_janela_sem_costura", costura(w, h, &bg)),
            ];
            for (caso, (a, d)) in casos {
                assert_ne!(
                    a, d,
                    "{res} {nome} {caso}: a lei antiga tem de dar outra imagem"
                );
                monta(&a, &d, w, h)
                    .save(format!("{dir}/{res}_{nome}_{caso}_antes_depois.png"))
                    .expect("png");
            }
        }
    }
}
