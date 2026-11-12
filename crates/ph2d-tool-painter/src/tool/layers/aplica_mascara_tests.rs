//! B01.A1: o `apply_mask` em faixas de linhas contra o laço serial antigo (verbatim), ao bit, a 1, 2, 7 e
//! 32 fios; e a sonda 4096² (`--ignored --nocapture`, `--profile smoke`).

use super::estrutura::mascara_no_alfa;
use crate::compositor::FIOS_FORCADOS;

/// O laço de `apply_mask` antes de B01.A1, verbatim.
fn oraculo_serial(mask_px: &[u8], parent_px: &mut [u8], n: usize, inverted: bool) {
    for idx in 0..n {
        let v = crate::compositor::mask_value(mask_px, idx);
        let cov = if inverted { 1.0 - v } else { v };
        let a = parent_px[idx * 4 + 3] as f32;
        parent_px[idx * 4 + 3] = (a * cov).round().clamp(0.0, 255.0) as u8;
    }
}

fn ruido(len: usize, semente: u32) -> Vec<u8> {
    let mut s = semente | 1;
    (0..len)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            (s >> 24) as u8
        })
        .collect()
}

#[test]
fn apply_mask_em_faixas_e_o_laco_serial_ao_bit() {
    let (w, h) = (97usize, 61usize);
    let mask = ruido(w * h * 4, 3);
    let base = ruido(w * h * 4, 5);
    for inverted in [false, true] {
        let mut esperado = base.clone();
        oraculo_serial(&mask, &mut esperado, w * h, inverted);
        for fios in [1usize, 2, 7, 32] {
            let mut got = base.clone();
            FIOS_FORCADOS.with(|c| c.set(Some(fios)));
            mascara_no_alfa(&mask, &mut got, w, inverted);
            FIOS_FORCADOS.with(|c| c.set(None));
            assert!(
                got == esperado,
                "{fios} fios, inverted={inverted}: bytes diferem do laço serial"
            );
        }
    }
}

/// Oráculo serial × faixas intercalados, mín. de 9, a 4096².
#[test]
#[ignore = "sonda de tempo"]
fn sonda_apply_mask_4096() {
    let (w, h) = (4096usize, 4096usize);
    let mask = ruido(w * h * 4, 3);
    let base = ruido(w * h * 4, 5);
    let (mut serial, mut faixas) = (f64::MAX, f64::MAX);
    let mut a = base.clone();
    let mut b = base.clone();
    for _ in 0..9 {
        a.copy_from_slice(&base);
        let t0 = std::time::Instant::now();
        oraculo_serial(&mask, &mut a, w * h, false);
        serial = serial.min(t0.elapsed().as_secs_f64() * 1e3);
        b.copy_from_slice(&base);
        let t0 = std::time::Instant::now();
        mascara_no_alfa(&mask, &mut b, w, false);
        faixas = faixas.min(t0.elapsed().as_secs_f64() * 1e3);
        assert!(a == b);
    }
    println!("apply_mask 4096²: serial {serial:.2} ms · faixas {faixas:.2} ms (mín. de 9)");
}
