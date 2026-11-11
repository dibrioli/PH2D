use super::*;
use crate::ColorMix;

/// O par do estudo (AP-13): o vermelho e o azul de fábrica do Wet.
const VERMELHO: [f64; 3] = [191.0, 0.0, 18.0];
const AZUL: [f64; 3] = [23.0, 101.0, 179.0];

/// Croma aproximado (`max − min` dos canais): o «5–16» do oliva medido no app; cada tinta sozinha dá 110–180.
fn croma(c: [f64; 3]) -> f64 {
    c.iter().copied().fold(f64::MIN, f64::max) - c.iter().copied().fold(f64::MAX, f64::min)
}

fn km(d: [f64; 3], s: [f64; 3], w: f64) -> [f64; 3] {
    let mut o = [0.0; 3];
    ColorMix::Km.mix(d[0], d[1], d[2], s[0], s[1], s[2], w, &mut o);
    o
}

fn espectral(d: [f64; 3], s: [f64; 3], w: f64) -> [f64; 3] {
    let mut o = [0.0; 3];
    ColorMix::Espectral.mix(d[0], d[1], d[2], s[0], s[1], s[2], w, &mut o);
    o
}

/// ⭐ B32.1 (decisão 7a): **vermelho + azul NÃO é oliva/quase preto.** O K–M de 3 canais (o Wet de antes) dá
/// croma baixo e verde ≥ vermelho; a lei espectral dá violeta: azul > verde + 20, croma ≥ 40, nos 3
/// pesos e nas 2 ordens.
#[test]
fn vermelho_mais_azul_no_wet_e_violeta_nao_oliva() {
    for w in [0.25, 0.5, 0.75] {
        for (d, s) in [(VERMELHO, AZUL), (AZUL, VERMELHO)] {
            let k = km(d, s, w);
            let e = espectral(d, s, w);
            eprintln!(
                "w {w}: K–M {k:.0?} croma {:.0} · espectral {e:.0?} croma {:.0}",
                croma(k),
                croma(e)
            );
            assert!(
                e[2] > e[1] + 20.0 && croma(e) >= 40.0,
                "w {w}: {e:?} não é violeta"
            );
        }
    }
}

/// O controlo: o K–M de 3 canais é o oliva/escuro do estudo (o vermelho antes da cura).
#[test]
fn o_km_de_tres_canais_da_o_escuro_sem_croma() {
    let k = km(VERMELHO, AZUL, 0.5);
    assert!(croma(k) < 40.0, "{k:?}");
}

/// Pontas e a mesma tinta, ao bit (a aguada parada não se move); a média de cantos iguais é o canto.
#[test]
fn as_pontas_e_a_mesma_tinta_sao_ao_bit() {
    assert_eq!(espectral(VERMELHO, AZUL, 0.0), VERMELHO);
    assert_eq!(espectral(VERMELHO, AZUL, 1.0), AZUL);
    assert_eq!(espectral(AZUL, AZUL, 0.4), AZUL);
    let c = [AZUL, AZUL, AZUL, AZUL].concat();
    let mut o = [0.0; 3];
    media_pesada_255(
        &c.try_into().unwrap(),
        &[0.1, 0.2, 0.3, 0.4],
        4,
        1.0,
        &mut o,
    );
    assert_eq!(o, AZUL);
}

/// A média de 2 cantos pesados ½·½ é a mistura de dois a ½ (a mesma lei nas duas portas).
#[test]
fn a_media_de_dois_cantos_e_a_mistura() {
    let mut c = [0.0; 12];
    c[..3].copy_from_slice(&VERMELHO);
    c[3..6].copy_from_slice(&AZUL);
    let mut o = [0.0; 3];
    media_pesada_255(&c, &[1.0, 1.0, 0.0, 0.0], 2, 0.5, &mut o);
    let m = espectral(VERMELHO, AZUL, 0.5);
    assert!((0..3).all(|k| (o[k] - m[k]).abs() < 0.01), "{o:?} × {m:?}");
}

#[test]
fn as_potencias_rapidas_casam_as_do_std() {
    for &x in &[1e-5f32, 0.003, 0.1, 0.5, 0.75, 0.99, 1.0] {
        assert!((log2(x) - x.log2()).abs() < 2e-6, "log2({x})");
    }
    for &y in &[-16.0f32, -9.3, -1.5, -0.49, -0.01, 0.0, 0.09, 1.3] {
        assert!((exp2(y) / y.exp2() - 1.0).abs() < 2e-6, "exp2({y})");
    }
}
