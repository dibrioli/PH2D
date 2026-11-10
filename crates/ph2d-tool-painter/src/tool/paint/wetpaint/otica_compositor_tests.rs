//! B11.4: a cola por faixas é a serial ao bit, e o custo das duas (a medida que fixa `COLA_EM_FAIXAS_BYTES`).

use super::{COLA_EM_FAIXAS_BYTES, cola, cola_em_faixas, cola_serial, recorta};

/// Bytes deterministas, diferentes por posição e por semente.
fn bytes(n: usize, semente: u32) -> Vec<u8> {
    let mut s = semente.wrapping_mul(2_654_435_761).max(1);
    (0..n)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            s as u8
        })
        .collect()
}

#[test]
fn a_cola_em_faixas_e_a_serial_ao_bit() {
    let (gw, gh) = (613usize, 401usize);
    let caixas = [
        (1, 1, gw, gh),
        (1, 1, 1, 1),
        (gw, gh, gw, gh),
        (7, 3, 300, 400),
        (200, 1, gw, 9),
        (1, 200, 17, gh),
    ];
    for bpp in [1usize, 4] {
        for (k, &(x0, y0, x1, y1)) in caixas.iter().enumerate() {
            let fundo = bytes(gw * gh * bpp, 11 + k as u32);
            let b = bytes((x1 + 1 - x0) * (y1 + 1 - y0) * bpp, 97 + k as u32);
            let mut serial = fundo.clone();
            cola_serial(&mut serial, bpp, gw, (x0, y0, x1, y1), &b);
            for f in [cola_em_faixas, cola] {
                let mut nova = fundo.clone();
                f(&mut nova, bpp, gw, (x0, y0, x1, y1), &b);
                assert!(nova == serial, "bpp {bpp} caixa {:?}", (x0, y0, x1, y1));
            }
            assert_eq!(recorta(&serial, bpp, gw, (x0, y0, x1, y1)), b);
        }
    }
}

/// A cola de uma imagem 2880² (7 B por célula: pigmento 4 + sombra, grão, Smooth Edges) e de caixas menores: a
/// serial e a por faixas INTERCALADAS no mesmo processo, o mínimo de 9. Medida: `--ignored`, `--profile smoke`.
#[test]
#[ignore = "medida"]
fn mede_cola_por_faixa() {
    let g = 2880usize;
    let mut plano4 = vec![0u8; g * g * 4];
    for lado in [64usize, 128, 256, 384, 512, 1024, 2880] {
        let caixa = (1, 1, lado, lado);
        let b = bytes(lado * lado * 4, 5);
        let (mut s_ms, mut f_ms) = (f64::MAX, f64::MAX);
        for _ in 0..9 {
            let t = std::time::Instant::now();
            cola_serial(&mut plano4, 4, g, caixa, &b);
            s_ms = s_ms.min(t.elapsed().as_secs_f64() * 1e3);
            let t = std::time::Instant::now();
            cola_em_faixas(&mut plano4, 4, g, caixa, &b);
            f_ms = f_ms.min(t.elapsed().as_secs_f64() * 1e3);
        }
        println!(
            "caixa {lado}² células × 4 B = {:.2} MB: serial {s_ms:.3} ms · faixas {f_ms:.3} ms · {:.1}× (limiar {} B)",
            b.len() as f64 / 1e6,
            s_ms / f_ms,
            COLA_EM_FAIXAS_BYTES,
        );
    }
}
