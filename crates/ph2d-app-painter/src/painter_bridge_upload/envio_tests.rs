//! B20.6: o envio inteiro fundido é o `clone` + `premultiply_rgba8` de antes, ao bit, e o que ele poupa.

use super::envio_fundido;
use ph2d_render::premultiply_rgba8;

fn de_antes(src: &[u8]) -> Vec<u8> {
    let mut v = src.to_vec();
    premultiply_rgba8(&mut v);
    v
}

/// Os 65 536 pares (canal, alfa), em telas de larguras ímpares (a faixa corta a meio de uma linha), e o buffer
/// reusado com lixo de um envio anterior de outro tamanho.
#[test]
fn envio_fundido_e_o_extract_mais_premul() {
    let mut pares = Vec::with_capacity(65_536 * 4);
    for a in 0..=255u8 {
        for c in 0..=255u8 {
            pares.extend_from_slice(&[c, 255 - c, c ^ 0x5a, a]);
        }
    }
    let mut dst = vec![0xAB; 17];
    for (w, h) in [(65_536usize, 1usize), (257, 255), (1021, 67), (3, 21_846)] {
        let n = w * h * 4;
        let src: Vec<u8> = pares.iter().copied().cycle().take(n).collect();
        envio_fundido(&src, &mut dst);
        assert_eq!(dst, de_antes(&src), "{w}x{h}");
    }
}

/// SONDA (ignorada): 4K, o envio de antes × o fundido, intercalados, o mínimo de 9.
#[test]
#[ignore = "sonda: corre à mão, no perfil smoke"]
fn sonda_envio_inteiro_4k() {
    let n = 3840 * 2160 * 4;
    let src: Vec<u8> = (0..n).map(|i| (i * 37 % 251) as u8).collect();
    let mut dst = Vec::new();
    let (mut a, mut b) = (f64::INFINITY, f64::INFINITY);
    for _ in 0..9 {
        let t = std::time::Instant::now();
        let v = de_antes(&src);
        std::hint::black_box(&v);
        drop(v);
        a = a.min(t.elapsed().as_secs_f64() * 1e3);
        let t = std::time::Instant::now();
        envio_fundido(&src, &mut dst);
        std::hint::black_box(&dst);
        b = b.min(t.elapsed().as_secs_f64() * 1e3);
    }
    println!("[envio 4K] antes {a:.2} ms · fundido {b:.2} ms ({:.1}×)", a / b);
}
