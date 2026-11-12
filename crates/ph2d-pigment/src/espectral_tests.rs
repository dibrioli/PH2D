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

/// Gerador determinístico (splitmix64): as mesmas entradas em toda corrida.
struct Sm(u64);

impl Sm {
    fn u(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    /// `[0, 1)` com 24 bits.
    fn f(&mut self) -> f32 {
        (self.u() >> 40) as f32 / (1u32 << 24) as f32
    }
    fn de<T: Copy>(&mut self, v: &[T]) -> T {
        v[(self.u() % v.len() as u64) as usize]
    }
}

/// Canais do teste: uniformes, `k/255` (o que o motor de fluido manda), as fronteiras do `clamp` e do `EPS`, fora de
/// `[0, 1]`, subnormais e o NaN canónico.
fn canal(g: &mut Sm) -> f32 {
    match g.u() % 8 {
        0 => g.de(&[
            0.0,
            -0.0,
            1.0,
            EPS,
            EPS * 0.5,
            1.0 - f32::EPSILON,
            1.0 + f32::EPSILON,
            -1e-7,
            2.0,
            -3.0,
            f32::MIN_POSITIVE,
            1e-45,
            1e-40,
            f32::NAN,
            f32::INFINITY,
        ]),
        1 | 2 => ((g.u() % 256) as f64 / 255.0) as f32,
        _ => g.f(),
    }
}

fn cor(g: &mut Sm) -> [f32; 3] {
    [canal(g), canal(g), canal(g)]
}

fn fac(g: &mut Sm) -> f32 {
    match g.u() % 4 {
        0 => g.de(&[
            0.0,
            -0.0,
            1.0,
            1e-30,
            1e-45,
            f32::EPSILON,
            1.0 - f32::EPSILON,
            -0.5,
            1.5,
            f32::NAN,
        ]),
        _ => g.f(),
    }
}

fn ao_bit32(a: [f32; 3], b: [f32; 3]) -> bool {
    a.map(f32::to_bits) == b.map(f32::to_bits)
}

/// ⭐ A mistura em pistas é a escalar de antes ([`super::oraculo`]), ao bit, em 4 milhões de entradas: o memo quente
/// (a base e a tinta repetidas), cores iguais, `fac` nas pontas e fora delas.
#[test]
fn a_mistura_vetorial_e_a_escalar_ao_bit() {
    let mut g = Sm(0x5eed_0001);
    let (mut base, mut tinta) = (cor(&mut g), cor(&mut g));
    for i in 0..4_000_000u32 {
        if g.u().is_multiple_of(2) {
            base = cor(&mut g);
        }
        if g.u().is_multiple_of(4) {
            tinta = cor(&mut g);
        }
        if g.u().is_multiple_of(32) {
            tinta = base;
        }
        let f = fac(&mut g);
        let (novo, velho) = (mistura(base, tinta, f), oraculo::mistura(base, tinta, f));
        assert!(
            ao_bit32(novo, velho),
            "#{i}: mistura({base:?}, {tinta:?}, {f:?}) = {novo:?} × {velho:?}"
        );
    }
}

/// A porta `0..255` do motor de fluido, ao bit (os canais inteiros e os de `f64` qualquer).
#[test]
fn a_mistura_255_vetorial_e_a_escalar_ao_bit() {
    let mut g = Sm(0x5eed_0255);
    let c = |g: &mut Sm| -> [f64; 3] {
        [0; 3].map(|_| {
            if g.u().is_multiple_of(2) {
                (g.u() % 256) as f64
            } else {
                (g.u() >> 11) as f64 / (1u64 << 53) as f64 * 255.0
            }
        })
    };
    for i in 0..2_000_000u32 {
        let (d, s) = (c(&mut g), c(&mut g));
        let s = if g.u().is_multiple_of(32) { d } else { s };
        let w = match g.u() % 4 {
            0 => g.de(&[
                0.0,
                1.0,
                1e-300,
                1e-9,
                0.5,
                1.0 - 1e-12,
                -1.0,
                2.0,
                f64::NAN,
            ]),
            _ => (g.u() >> 11) as f64 / (1u64 << 53) as f64,
        };
        let (novo, velho) = (mistura_255(d, s, w), oraculo::mistura_255(d, s, w));
        assert!(
            novo.map(f64::to_bits) == velho.map(f64::to_bits),
            "#{i}: mistura_255({d:?}, {s:?}, {w:?}) = {novo:?} × {velho:?}"
        );
    }
}

/// A média de até 4 cantos, ao bit: pesos nulos, negativos, cantos iguais, `count` de 0 a 4.
#[test]
fn a_media_pesada_vetorial_e_a_escalar_ao_bit() {
    let mut g = Sm(0x5eed_0004);
    for i in 0..1_000_000u32 {
        let mut colors = [0.0f64; 12];
        for v in &mut colors {
            *v = if g.u().is_multiple_of(2) {
                (g.u() % 256) as f64
            } else {
                f64::from(g.f()) * 255.0
            };
        }
        if g.u().is_multiple_of(8) {
            let p = [colors[0], colors[1], colors[2]];
            for k in 1..4 {
                colors[k * 3..k * 3 + 3].copy_from_slice(&p);
            }
        }
        let weights = [0; 4].map(|_| match g.u() % 6 {
            0 => 0.0,
            1 => -0.25,
            _ => f64::from(g.f()),
        });
        let count = (g.u() % 5) as usize;
        let total: f64 = weights[..count].iter().filter(|w| **w > 0.0).sum();
        let inv_total = if g.u().is_multiple_of(8) || total <= 0.0 {
            f64::from(g.f()) * 2.0
        } else {
            1.0 / total
        };
        let (mut novo, mut velho) = ([7.0; 3], [7.0; 3]);
        media_pesada_255(&colors, &weights, count, inv_total, &mut novo);
        oraculo::media_pesada_255(&colors, &weights, count, inv_total, &mut velho);
        assert!(
            novo.map(f64::to_bits) == velho.map(f64::to_bits),
            "#{i}: media({colors:?}, {weights:?}, {count}, {inv_total}) = {novo:?} × {velho:?}"
        );
    }
}

/// As potências de antes, ao bit: o `log2` num passo primo sobre os 2³² bits, o `exp2` em todo o `f32` de
/// `[−126, 4]` num passo menor, e as fronteiras (meios inteiros, onde o `floor(y + ½)` vira).
#[test]
fn o_log2_e_o_exp2_sao_os_de_antes_ao_bit() {
    for b in (0..=u32::MAX).step_by(4093) {
        let x = f32::from_bits(b);
        assert_eq!(log2(x).to_bits(), oraculo::log2(x).to_bits(), "log2({x:e})");
    }
    for b in (0..=u32::MAX).step_by(1021) {
        let y = f32::from_bits(b);
        assert_eq!(exp2(y).to_bits(), oraculo::exp2(y).to_bits(), "exp2({y:e})");
    }
    for k in -130i32..=8 {
        let m = k as f32 + 0.5;
        for y in [
            m,
            m.next_down(),
            m.next_up(),
            k as f32,
            (k as f32).next_down(),
            (k as f32).next_up(),
        ] {
            assert_eq!(exp2(y).to_bits(), oraculo::exp2(y).to_bits(), "exp2({y:e})");
        }
    }
}

/// ⛔ A PROVA da troca do `floor` pelo [`MAGICO`]: todos os 2³² bits, no `exp2` e no `log2` (~10 s em 16 fios;
/// `--ignored`, corre-se quando estas potências mudam).
#[test]
#[ignore = "varredura exaustiva dos 2³² bits: corre-se quando log2/exp2 mudam"]
fn o_log2_e_o_exp2_sao_os_de_antes_em_todos_os_bits() {
    const FIOS: u64 = 16;
    let pedaco = (1u64 << 32) / FIOS;
    std::thread::scope(|s| {
        for f in 0..FIOS {
            s.spawn(move || {
                for b in f * pedaco..(f + 1) * pedaco {
                    let v = f32::from_bits(b as u32);
                    assert_eq!(exp2(v).to_bits(), oraculo::exp2(v).to_bits(), "exp2({v:e})");
                    assert_eq!(log2(v).to_bits(), oraculo::log2(v).to_bits(), "log2({v:e})");
                }
            });
        }
    });
}
