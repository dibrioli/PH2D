//! B14.2 (decisão 2 do dono): a luz SIMPLES do gesto — metade das amostras no mesmo alcance — e a CHEIA de fábrica.

use super::*;

const W: usize = 64;

/// Um bloco alto no meio (o da `a_sombra_cai_do_lado_contrario_a_luz_e_chega_ao_papel`).
fn bloco() -> Grid {
    let mut g = Grid::new(W, W);
    for y in 24..=40 {
        for x in 24..=40 {
            g.sett[x + y * g.s] = 12_000.0;
        }
    }
    g
}

#[test]
fn a_luz_simples_do_gesto_corta_as_amostras_e_a_cheia_e_a_de_fabrica() {
    assert_eq!(OleoReal::default().qualidade, QualidadeDaLuz::Cheia);
    let (c, s) = (QualidadeDaLuz::Cheia, QualidadeDaLuz::Simples);
    assert_eq!(
        (c.fatias(), c.horizonte(), c.marcha().count()),
        (FATIAS, &HORIZONTE[..], PASSOS_SOMBRA)
    );
    assert_eq!(
        (s.fatias(), s.horizonte().len(), s.marcha().count()),
        (2, 5, 12)
    );
    // O mesmo alcance: a região suja e a costura do recompor não mudam.
    assert!(s.marcha().all(|t| t <= PASSOS_SOMBRA));
    assert_eq!(s.horizonte().last(), Some(&ALCANCE_FENDA_MAX));

    let g = bloco();
    let cheia = OleoReal::default();
    let simples = OleoReal {
        qualidade: s,
        ..cheia
    };
    let campo = CampoAltura::de(&[(&g, true)], &cheia, 1, 1, 1, W, W);
    let (mut sombra, mut luz) = (0, 0);
    for y in 12..=52 {
        for x in 12..=52 {
            let (a, b) = (
                sombra_no_papel(&cheia, &campo, x, y),
                sombra_no_papel(&simples, &campo, x, y),
            );
            sombra += usize::from(a.to_bits() != b.to_bits());
            let (ta, tb) = (termos(&cheia, &campo, x, y), termos(&simples, &campo, x, y));
            luz += usize::from(
                ta.luz.to_bits() != tb.luz.to_bits() || ta.vis.to_bits() != tb.vis.to_bits(),
            );
        }
    }
    assert!(
        sombra > 0 && luz > 0,
        "a Simples tem de ser usada: sombra {sombra} luz {luz} células diferentes"
    );
    // A sombra continua a cair atrás do bloco, e o papel longe dele e a lâmpada à frente ficam como na cheia.
    assert!(sombra_no_papel(&simples, &campo, 43, 43) > 0.2);
    assert_eq!(
        sombra_no_papel(&simples, &campo, 4, 4).to_bits(),
        sombra_no_papel(&cheia, &campo, 4, 4).to_bits()
    );
    assert!(luz_directa(&simples, &campo, 21, 21, campo.at_sombra(21, 21)) == 1.0);
}

/// Sonda de custo (B14.2; `--profile smoke`, `--ignored`): a luz por célula (`termos` + `sombra_no_papel`) de uma folha
/// de tinta com relevo, Cheia × Simples intercaladas em 10 blocos; o mínimo de cada.
#[test]
#[ignore = "sonda de custo: corre com --profile smoke --ignored"]
fn sonda_a_luz_simples_custa_menos_que_a_cheia() {
    const L: usize = 256;
    let mut g = Grid::new(L, L);
    let mut h: u64 = 0x9E37_79B9_7F4A_7C15;
    for y in 2..L - 2 {
        for x in 2..L - 2 {
            h ^= h << 13;
            h ^= h >> 7;
            h ^= h << 17;
            g.sett[x + y * g.s] = 4_000.0 + (h % 8_000) as f32;
        }
    }
    let cheia = OleoReal::default();
    let simples = OleoReal {
        qualidade: QualidadeDaLuz::Simples,
        ..cheia
    };
    let campo = CampoAltura::de(&[(&g, true)], &cheia, 1, 1, 1, L, L);
    let passa = |o: &OleoReal| {
        let t = std::time::Instant::now();
        let mut acc = 0.0;
        for y in 1..L - 1 {
            for x in 1..L - 1 {
                acc += sombra_no_papel(o, &campo, x, y) + termos(o, &campo, x, y).luz;
            }
        }
        std::hint::black_box(acc);
        t.elapsed().as_secs_f64() * 1e3
    };
    let (mut mc, mut ms) = (f64::MAX, f64::MAX);
    for b in 0..10 {
        if b % 2 == 0 {
            mc = mc.min(passa(&cheia));
            ms = ms.min(passa(&simples));
        } else {
            ms = ms.min(passa(&simples));
            mc = mc.min(passa(&cheia));
        }
    }
    eprintln!(
        "[sonda B14.2] {L}x{L} células: Cheia {mc:.2} ms · Simples {ms:.2} ms · {:.2}x",
        mc / ms
    );
    assert!(ms < mc, "a Simples não custa menos: {ms:.2} × {mc:.2} ms");
}
