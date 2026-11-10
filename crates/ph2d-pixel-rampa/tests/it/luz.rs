//! P13 L: a luz na rampa — o gabarito CPU (`docs/PixelArt/P13_plano.md` §4.L, §7).
//! Fixture `paleta_rampas`: pele 5 degraus, água 4, folha 6 (geradas pela `gera_rampa`) e 3 cores
//! soltas; o cel tem todas as cores, mais um texel transparente.

use ph2d_pixel_core::{PaletteRamps, Ramp, RampId};
use ph2d_pixel_rampa::luz::{LutDaLuz, degrau_aceso};
use ph2d_pixel_rampa::{ConfigRampa, gera_rampa};

fn paleta_rampas() -> (Vec<[u8; 4]>, PaletteRamps) {
    let cfg = |passos| ConfigRampa {
        passos,
        desvio_matiz_graus: 20.0,
        curva_croma: 0.6,
        faixa_l: [0.25, 0.92],
    };
    let mut pal = vec![[0, 0, 0, 0]];
    let mut ramps = Vec::new();
    for (id, (base, n)) in [
        ([0xd9, 0xa0, 0x66, 255], 5),
        ([0x5b, 0x6e, 0xe1, 255], 4),
        ([0x6a, 0xbe, 0x30, 255], 6),
    ]
    .into_iter()
    .enumerate()
    {
        let first = pal.len();
        pal.extend(gera_rampa(base, cfg(n)));
        ramps.push(Ramp {
            id: RampId(id as u32 + 1),
            name: format!("r{id}"),
            indices: (first..pal.len()).map(|i| i as u8).collect(),
        });
    }
    pal.extend([[200, 30, 40, 255], [250, 250, 250, 255], [20, 20, 24, 255]]);
    (
        pal,
        PaletteRamps {
            ramps,
            cycles: Vec::new(),
        },
    )
}

fn luzes() -> Vec<[f32; 3]> {
    let mut v = Vec::new();
    for i in 0..=40 {
        let k = i as f32 * 0.05;
        v.extend([[k, k, k], [k, k * 0.6, k * 0.3], [k * 0.2, k * 0.5, k]]);
    }
    v
}

#[test]
fn a_luz_neutra_na_rampa_e_o_quadro_de_hoje() {
    let (pal, ramps) = paleta_rampas();
    let lut = LutDaLuz::da_paleta(&pal, &ramps);
    let mut em_rampa = 0;
    for &cor in &pal {
        match lut.acende(cor, [1.0, 1.0, 1.0]) {
            Some(c) => {
                assert_eq!(c, cor, "a luz neutra mudou o degrau de {cor:?}");
                em_rampa += 1;
            }
            None => assert!(
                cor[3] == 0
                    || ramps
                        .ramps
                        .iter()
                        .all(|r| r.indices.iter().all(|&i| pal[usize::from(i)] != cor)),
                "cor de rampa sem degrau: {cor:?}"
            ),
        }
    }
    assert_eq!(em_rampa, 15, "a fixture tem 15 degraus");
}

#[test]
fn mais_luz_nunca_escurece_o_degrau() {
    let (pal, ramps) = paleta_rampas();
    let lut = LutDaLuz::da_paleta(&pal, &ramps);
    let mut subiu = 0;
    for r in &lut.rampas {
        for pos in 0..r.len() {
            for tinta in [[1.0, 1.0, 1.0], [1.0, 0.6, 0.3], [0.2, 0.5, 1.0]] {
                let mut antes = 0;
                for i in 0..=200 {
                    let k = i as f32 * 0.01;
                    let p = degrau_aceso(r, pos, tinta.map(|t: f32| t * k));
                    assert!(
                        p >= antes,
                        "k={k} {tinta:?}: o degrau {pos} desceu {antes}→{p}"
                    );
                    subiu += usize::from(p > antes);
                    antes = p;
                }
                assert_eq!(
                    degrau_aceso(r, pos, [0.0; 3]),
                    0,
                    "sem luz = a ponta escura"
                );
            }
        }
    }
    assert!(subiu > 30, "a varredura quase não muda o degrau ({subiu})");
}

#[test]
fn a_luz_na_rampa_fica_na_paleta() {
    let (pal, ramps) = paleta_rampas();
    let lut = LutDaLuz::da_paleta(&pal, &ramps);
    let mut trocas = 0;
    for &cor in &pal {
        let Some((s, _)) = lut.degrau_de(cor) else {
            continue;
        };
        for l in luzes() {
            let c = lut.acende(cor, l).expect("em rampa");
            assert!(
                lut.rampas[s].iter().any(|d| d.cor == c),
                "{cor:?} sob {l:?} saiu da rampa: {c:?}"
            );
            trocas += usize::from(c != cor);
        }
    }
    assert!(trocas > 100, "a luz quase nunca troca o degrau ({trocas})");
}

#[test]
fn a_lut_da_placa_e_a_da_luz() {
    let (pal, ramps) = paleta_rampas();
    let lut = LutDaLuz::da_paleta(&pal, &ramps);
    let t = lut.texeis();
    assert_eq!((lut.largura(), t.len()), (6, 18));
    assert_eq!(t[4][0], lut.rampas[0][4].l);
    assert_eq!(t[5][0], -1.0, "fim da rampa de 5");
    assert_eq!(t[6 + 4][0], -1.0, "fim da rampa de 4");
}

/// A lei escrita: o degrau é o de L mais próximo de `L(cor) · lum(luz)` (claridades recalculadas
/// das cores da paleta, não lidas da LUT).
#[test]
fn o_degrau_e_o_da_claridade_mais_proxima() {
    use ph2d_pixel_rampa::luz::{claridade, lum};
    let (pal, ramps) = paleta_rampas();
    let lut = LutDaLuz::da_paleta(&pal, &ramps);
    for r in &lut.rampas {
        let ls: Vec<f32> = r.iter().map(|d| claridade(d.cor)).collect();
        for pos in 0..r.len() {
            for l in luzes() {
                let alvo = ls[pos] * lum(l);
                let perto = (0..ls.len())
                    .min_by(|&a, &b| (ls[a] - alvo).abs().total_cmp(&(ls[b] - alvo).abs()))
                    .unwrap();
                assert_eq!(degrau_aceso(r, pos, l), perto, "pos {pos} luz {l:?}");
            }
        }
    }
    assert!((lum([1.0; 3]) - 1.0).abs() < 1e-5, "a luz neutra tem lum 1");
}
