//! Gates da lei do Óleo Real (filho de [`super`]): a tinta lisa é a cor dela, a fenda satura no tom da
//! tinta (M2), a sombra cai do lado contrário à luz e chega ao papel (M6); W1b: a espessura levanta o
//! relevo, a crista faz sombra sobre a própria tinta, o brilho da orla vai com a cobertura.

use super::*;

const W: usize = 64;

/// Uma grade `W×W` com massa `m(x, y)` na camada única.
fn grade(m: impl Fn(usize, usize) -> f32) -> Grid {
    let mut g = Grid::new(W, W);
    for y in 1..=W {
        for x in 1..=W {
            g.sett[x + y * g.s] = m(x, y);
        }
    }
    g
}

fn campo(g: &Grid, o: &OleoReal) -> CampoAltura {
    CampoAltura::de(&[(g, true)], o, 1, 1, 1, W, W)
}

#[test]
fn a_tinta_lisa_sai_com_a_cor_dela() {
    let g = grade(|_, _| 3000.0);
    let o = OleoReal {
        brilho: 0.0,
        ..OleoReal::default()
    };
    let c = campo(&g, &o);
    for rgb in [
        [20.0, 150.0, 230.0],
        [240.0, 240.0, 236.0],
        [30.0, 25.0, 60.0],
    ] {
        let out = ilumina(&o, &c, 32, 32, rgb, 1.0);
        for k in 0..3 {
            assert!((out[k] - rgb[k]).abs() < 0.5, "{rgb:?} → {out:?}");
        }
    }
}

#[test]
fn a_fenda_satura_no_tom_da_tinta_m2() {
    // Um sulco fundo de 3 células numa tinta alta: no fundo a cavidade é cheia.
    let g = grade(|x, _| if (31..=33).contains(&x) { 0.0 } else { 6000.0 });
    let o = OleoReal {
        brilho: 0.0,
        sombra: 0.0,
        af: Afinacao {
            raio_suave: 0,
            ..Afinacao::default()
        },
        ..OleoReal::default()
    };
    let c = campo(&g, &o);
    let t = termos(&o, &c, 32, 32);
    assert!(t.cav >= 0.999, "{t:?}");
    // Com a cavidade cheia, a razão entre canais em linear é (a_i/a_j)^(1+k): a fenda satura no tom.
    let rgb = [20.0, 150.0, 230.0];
    let out = ilumina(&o, &c, 32, 32, rgb, 1.0);
    let lin = |v: f64| srgb255_to_linear(v);
    let medido = lin(out[1]) / lin(out[2]);
    let lei = libm::pow(lin(rgb[1]) / lin(rgb[2]), 1.0 + o.af.k_fenda);
    assert!(
        (medido / lei - 1.0).abs() < 0.02,
        "medido {medido} lei {lei}"
    );
}

#[test]
fn a_sombra_cai_do_lado_contrario_a_luz_e_chega_ao_papel() {
    // Um bloco alto no meio (≈ 18 px de relevo à escala de fábrica); a luz vem do topo-esquerda (225°).
    let g = grade(|x, y| {
        if (24..=40).contains(&x) && (24..=40).contains(&y) {
            12_000.0
        } else {
            0.0
        }
    });
    let o = OleoReal::default();
    let c = campo(&g, &o);
    let atras = sombra_no_papel(&o, &c, 43, 43);
    let frente = sombra_no_papel(&o, &c, 21, 21);
    assert!(
        atras > 0.2,
        "atrás do bloco (baixo-direita) o papel escurece: {atras}"
    );
    assert!(frente == 0.0, "do lado da luz não: {frente}");
    let sem = sombra_no_papel(&OleoReal { sombra: 0.0, ..o }, &c, 43, 43);
    assert!(sem == 0.0, "Sombra 0 = nada no papel: {sem}");
}

#[test]
fn a_espessura_levanta_o_relevo_que_o_tecto_aparava() {
    // A mesma massa (o tecto de 3000); metade dela com o dobro da espessura (W1b).
    let mut g = grade(|_, _| 3000.0);
    let s = g.s;
    for y in 1..=W {
        for x in 33..=W {
            crate::grid::espessura::fixa(&mut g, x + y * s, 6000.0);
        }
    }
    let c = campo(&g, &OleoReal::default());
    let (fina, grossa) = (c.at_sombra(10, 32), c.at_sombra(56, 32));
    assert!(
        (grossa / fina - 2.0).abs() < 1e-3,
        "fina {fina} grossa {grossa}"
    );
}

#[test]
fn a_crista_projecta_sombra_dentro_da_tinta() {
    // Tinta em toda a parte e uma crista de espessura no meio; a luz vem do topo-esquerda (225°).
    let mut g = grade(|_, _| 3000.0);
    let s = g.s;
    for y in 24..=40 {
        for x in 24..=40 {
            crate::grid::espessura::fixa(&mut g, x + y * s, 12_000.0);
        }
    }
    let o = OleoReal::default();
    let c = campo(&g, &o);
    let atras = termos(&o, &c, 43, 43).luz;
    let frente = termos(&o, &c, 21, 21).luz;
    assert!(atras < 0.5, "atrás da crista, sobre a tinta: {atras}");
    assert!(frente == 1.0, "do lado da luz: {frente}");
}

#[test]
fn o_brilho_da_orla_vai_com_a_cobertura() {
    // Uma rampa virada para a luz: o brilho é o termo que satura (os pontos brancos da borda).
    let g = grade(|x, y| (x + y) as f32 * 200.0);
    let o = OleoReal::default();
    let c = campo(&g, &o);
    let rgb = [20.0, 150.0, 230.0];
    let sem_brilho = ilumina(&OleoReal { brilho: 0.0, ..o }, &c, 32, 32, rgb, 1.0);
    let orla_vazia = ilumina(&o, &c, 32, 32, rgb, 0.0);
    for k in 0..3 {
        assert!(
            (orla_vazia[k] - sem_brilho[k]).abs() < 1e-9,
            "{orla_vazia:?} vs {sem_brilho:?}"
        );
    }
}
