//! Gates da lei do Óleo Real (filho de [`super`]): a tinta lisa é a cor dela, a fenda satura no tom da
//! tinta (M2), a sombra cai do lado contrário à luz e chega ao papel (M6); W1b: a espessura levanta o
//! relevo, a crista faz sombra sobre a própria tinta, o brilho da orla vai com a cobertura.

use super::*;

const W: usize = 64;
#[path = "oleo_real_borda_tests.rs"]
mod oleo_real_borda_tests;
#[path = "oleo_real_campo_tests.rs"]
mod oleo_real_campo_tests;
pub(super) use oleo_real_borda_tests::serrilhado_pub;

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
fn a_tinta_lisa_sai_com_a_cor_dela_em_todo_ambiente() {
    let g = grade(|_, _| 3000.0);
    for ambiente in Ambiente::TODOS {
        let o = OleoReal {
            brilho: 0.0,
            ambiente,
            ..OleoReal::default()
        };
        let c = campo(&g, &o);
        assert!(
            (termos(&o, &c, 32, 32).vis - 1.0).abs() < 1e-9,
            "{ambiente:?}"
        );
        for rgb in [
            [215.0, 25.0, 30.0],
            [240.0, 240.0, 236.0],
            [30.0, 25.0, 60.0],
        ] {
            let out = ilumina(&o, &c, 32, 32, rgb, 1.0);
            for k in 0..3 {
                assert!(
                    (out[k] - rgb[k]).abs() < 0.5,
                    "{ambiente:?} {rgb:?} → {out:?}"
                );
            }
        }
    }
}

#[test]
fn a_fenda_fica_no_tom_da_tinta_e_nunca_preta() {
    // Um sulco fundo de 3 células numa tinta alta, o vermelho saturado da captura do dono (W1b): o Cycles
    // sobre a mesma tinta põe o R da fenda em 171 de 255, a lei M2 punha-o em 76 (pontos pretos).
    let g = grade(|x, _| if (31..=33).contains(&x) { 0.0 } else { 6000.0 });
    let o = OleoReal {
        brilho: 0.0,
        sombra: 0.0,
        af: Afinacao {
            raio_suave: 0,
            raio_detalhe: 0,
            ..Afinacao::default()
        },
        ..OleoReal::default()
    };
    let c = campo(&g, &o);
    let t = termos(&o, &c, 32, 32);
    assert!(t.vis < 0.5, "o fundo do sulco vê pouco céu: {t:?}");
    let rgb = [215.0, 25.0, 30.0];
    let out = ilumina(&o, &c, 32, 32, rgb, 1.0);
    assert!(out[0] > 0.6 * rgb[0], "o vermelho não se apaga: {out:?}");
    // O multi-ricochete satura: o canal dominante perde menos que os outros.
    let lin = srgb255_to_linear;
    let (r, gg) = (lin(out[0]) / lin(rgb[0]), lin(out[1]) / lin(rgb[1]));
    assert!(r > gg, "R {r} G {gg}");
}

#[test]
fn o_multi_ricochete_bate_com_o_cycles() {
    // `oraculo/calibra_ao.py` (26/10): o ganho do canal R do vermelho sobre a visibilidade, no Cycles.
    for (v, ganho_cycles) in [
        (0.777, 1.156),
        (0.699, 1.228),
        (0.555, 1.358),
        (0.466, 1.467),
    ] {
        let a = srgb255_to_linear(215.0);
        let ganho = multi_ricochete(v, a) / v;
        assert!(
            (ganho / ganho_cycles - 1.0).abs() < 0.08,
            "V {v}: {ganho} × Cycles {ganho_cycles}"
        );
    }
}

#[test]
fn o_realce_enche_a_folga_sem_passar_de_um() {
    for base in [0.0, 0.2, 0.85, 0.999, 1.0] {
        assert!((rola(base, 0.0) - base).abs() < 1e-12, "neutro em {base}");
        let mut antes = base;
        for k in 1..=40 {
            let y = rola(base, f64::from(k) * 0.25);
            assert!(y >= antes && y <= 1.0, "{base} {k}: {y}");
            antes = y;
        }
    }
    // Pouco brilho soma-se quase inteiro.
    assert!((rola(0.2, 0.01) - 0.21).abs() < 1e-3);
}

#[test]
fn o_ambiente_poe_o_brilho_do_lado_da_luz() {
    // Duas rampas iguais, uma virada para a luz (topo-esquerda) e outra de costas: com o estúdio o reflexo
    // do lado claro é maior (o ambiente rodou-se até à luz).
    let g = grade(|x, _| x as f32 * 300.0);
    let o = OleoReal {
        sombra: 0.0,
        ..OleoReal::default()
    };
    let c = campo(&g, &o);
    let de_frente = termos(&o, &c, 32, 32).espec[1];
    let g2 = grade(|x, _| (64 - x) as f32 * 300.0);
    let c2 = campo(&g2, &o);
    let de_costas = termos(&o, &c2, 32, 32).espec[1];
    assert!(
        de_frente > de_costas,
        "frente {de_frente} costas {de_costas}"
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
    // Do lado da luz só o céu que a parede tapa ao pé (W1c): a lâmpada chega inteira.
    assert!(
        atras > frente + 0.15 && frente < 0.15,
        "do lado da luz quase nada: {frente} × atrás {atras}"
    );
    assert!(
        luz_directa(&o, &c, 21, 21, c.at_sombra(21, 21)) == 1.0,
        "a lâmpada chega inteira à frente"
    );
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

/// A largura (células) da parede de cima de um planalto na coluna `x`: as células entre 10 % e 90 % do miolo `y=32`.
fn largura_da_parede(c: &CampoAltura, x: isize) -> usize {
    let topo = c.at(x, 32);
    (1..32)
        .filter(|&y| {
            let f = c.at(x, y) / topo;
            (0.1..=0.9).contains(&f)
        })
        .count()
}

/// ⭐ **A parede nivelada fica viva** (W3 K7): o planalto que a faca NIVELOU ([`Grid::nivelado`]) sai do desfoque
/// largo da forma — a parede cai em ≤ 2 células (o Rebelle Knife Smooth 2: ~2 px) contra as ~10 da forma de sempre
/// —, o miolo fica à altura dele, e a tinta por marcar longe do planalto é a de antes AO BIT (a marca toda a zero
/// também). Mutações que sangram: a marca ignorada na forma · a média sem normalizar · a parte presa sem o `mu`.
#[test]
fn a_parede_nivelada_fica_viva() {
    // Um planalto (x 8..=30, y 20..=44) e, longe dele, uma faixa de outra tinta (x 50..=60) sem marca.
    let tinta = |x: usize, y: usize| (8..=30).contains(&x) && (20..=44).contains(&y);
    let outra = |x: usize, y: usize| (50..=60).contains(&x) && (20..=44).contains(&y);
    let base = grade(|x, y| {
        if tinta(x, y) || outra(x, y) {
            3000.0
        } else {
            0.0
        }
    });
    let mut marcada = base.clone();
    let mut zerada = base.clone();
    crate::grid::nivelado::garante(&mut zerada);
    let s = marcada.s;
    let mu = crate::grid::nivelado::garante(&mut marcada);
    for y in 1..=W {
        for x in 1..=W {
            if tinta(x, y) {
                mu[x + y * s] = 1.0;
            }
        }
    }
    let o = OleoReal::default();
    let (c0, c1, cz) = (campo(&base, &o), campo(&marcada, &o), campo(&zerada, &o));
    let (antes, depois) = (largura_da_parede(&c0, 19), largura_da_parede(&c1, 19));
    // §BUG-ressalto: a forma de sempre só alisa o ombro DENTRO da tinta (4 células; eram ~10 com a saia no papel).
    assert!(
        antes >= 4,
        "a forma de sempre devia espalhar a parede: {antes}"
    );
    assert!(
        depois <= 2,
        "a parede nivelada espalhou-se: {depois} células"
    );
    assert!(
        (c1.at(19, 32) / c0.at(19, 32) - 1.0).abs() < 0.01,
        "o miolo do planalto mudou: {} × {}",
        c1.at(19, 32),
        c0.at(19, 32)
    );
    // Fora do planalto a parede não sobe: a 2 células do último marcado, nada.
    assert!(
        c1.at(19, 17) < 1e-6 * c1.at(19, 32),
        "o planalto vazou: {}",
        c1.at(19, 17)
    );
    let bits = |c: &CampoAltura, x: isize| -> Vec<u64> {
        (1..=W as isize).map(|y| c.at(x, y).to_bits()).collect()
    };
    assert_eq!(bits(&c1, 55), bits(&c0, 55), "a tinta por marcar mudou");
    assert!(
        cz.v.iter()
            .map(|v| v.to_bits())
            .eq(c0.v.iter().map(|v| v.to_bits())),
        "a marca a zero mudou a forma"
    );
    // A orla FINA do planalto (as duas últimas linhas a pouca tinta, cobertura < 1, como a borda do Smooth 2) não
    // puxa o miolo para baixo: a média pesa pela cobertura (1,3 %; pesada só pela marca, 14 %) — a parede é a da orla,
    // não uma rampa para dentro.
    let mut fina = marcada.clone();
    for y in 43..=44 {
        for x in 8..=30 {
            fina.sett[x + y * s] = 40.0;
        }
    }
    let cf = campo(&fina, &o);
    let cobre = crate::opacity::alpha_of_mass(40.0);
    assert!(cobre < 0.9, "a orla de teste devia ser fina: {cobre}");
    assert!(
        (cf.at(19, 40) / cf.at(19, 32) - 1.0).abs() < 0.03,
        "a orla fina puxou o miolo: {} × {} (cobertura da orla {cobre})",
        cf.at(19, 40),
        cf.at(19, 32)
    );
}

/// ⭐ **A crista da faca faz sombra** (W3 K6): o rebordo da lâmina ([`Grid::crista`]) entra na altura da SOMBRA e
/// projecta a banda escura atrás dele, por dentro da tinta (`49_realshader/E4_lamina.md` §Rebordo); os fios de um
/// pincel de cerdas (a `altura` sem crista) não — a sombra deles fica a de antes ao bit. Mutações que sangram: a
/// crista fora da sombra, os fios na sombra, a crista sem o `cobre`.
#[test]
fn a_crista_da_faca_faz_sombra() {
    let lisa = grade(|_, _| 3000.0);
    // O mesmo rebordo (4 px de relevo, 3 células) nos FIOS e na CRISTA de uma faca; luz do topo-esquerda (225°).
    let mut fios = grade(|_, _| 3000.0);
    fios.altura = vec![0.0; fios.cells];
    for y in 1..=W {
        for x in 30..=32 {
            fios.altura[x + y * fios.s] = 4.0;
        }
    }
    let mut faca = fios.clone();
    faca.crista.clone_from(&faca.altura);
    let o = OleoReal::default();
    let (c_lisa, c_fios, c_faca) = (campo(&lisa, &o), campo(&fios, &o), campo(&faca, &o));
    assert!(
        c_fios
            .sombra
            .iter()
            .map(|v| v.to_bits())
            .eq(c_lisa.sombra.iter().map(|v| v.to_bits())),
        "os fios entraram na altura da sombra"
    );
    let atras = |c: &CampoAltura| termos(&o, c, 34, 32).luz;
    assert!(
        atras(&c_fios) == 1.0,
        "sem crista, atrás dos fios: {}",
        atras(&c_fios)
    );
    assert!(
        atras(&c_faca) < 0.5,
        "atrás da crista da faca: {}",
        atras(&c_faca)
    );
    assert!(termos(&o, &c_faca, 28, 32).luz == 1.0, "do lado da luz");
    // A crista vai a zero com a cobertura, como os fios: no papel nu não levanta nada.
    let mut nua = Grid::new(W, W);
    nua.crista = vec![2.0; nua.cells];
    assert!(
        campo(&nua, &o).sombra.iter().all(|&v| v == 0.0),
        "crista sem tinta"
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

/// Um relevo VARIADO (cristas, um sulco fundo, um bloco alto e o papel livre) e a impressão digital de
/// tudo o que a óptica devolve nele — termos, cor e sombra no papel, bit a bit.
fn impressao_digital(o: &OleoReal) -> u64 {
    let g = relevo_variado();
    let c = campo(&g, o);
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut mistura = |v: f64| {
        h ^= v.to_bits();
        h = h.wrapping_mul(0x0100_0000_01b3);
    };
    for cy in 2..W {
        for cx in 2..W {
            let t = termos(o, &c, cx, cy);
            for v in [t.vis, t.luz, t.sol, t.espec[0], t.espec[1], t.espec[2]] {
                mistura(v);
            }
            for v in ilumina(o, &c, cx, cy, [215.0, 25.0, 30.0], 1.0) {
                mistura(v);
            }
            mistura(sombra_no_papel(o, &c, cx, cy));
        }
    }
    h
}

/// A força da fenda multiplica as CHANCES de oclusão: `V' / (1 − V') = (V / (1 − V)) / f`, nos extremos
/// `f = 0` (céu inteiro) e `V = 1` (nada a tapar). Fecha a mutação sobrevivente M1a (`f × 1,001`).
#[test]
fn a_forca_da_fenda_multiplica_as_chances_de_oclusao() {
    assert_eq!(forca_da_fenda(0.5, 0.0), 1.0);
    assert_eq!(forca_da_fenda(1.0, 3.0), 1.0);
    assert_eq!(forca_da_fenda(0.5, 3.0), 0.25);
    assert_eq!(forca_da_fenda(0.75, 2.0), 0.6);
    for v in [0.1, 0.3, 0.55, 0.9] {
        for f in [0.25, 0.5, 2.0, 3.0] {
            let w = forca_da_fenda(v, f);
            let chances = |x: f64| x / (1.0 - x);
            assert!(
                (chances(w) - chances(v) / f).abs() < 1e-12,
                "v {v} f {f}: {w}"
            );
        }
    }
}

/// ⭐ **Neutro = a imagem de antes** (W2 E1 + E2): com a força da fenda `1`, o alcance de fábrica e a força
/// do ambiente de fábrica, o CLÁSSICO é BIT a BIT o de `c3365fb579` (a impressão digital gravada antes dos
/// ajustes). O Estúdio saiu daqui na E2 por DESENHO (o difuso passou a vir do sector do HDRI; a impressão
/// dele em `c3365fb579` era `0xbdcc_ca7e_7461_db68`): as leis dele são as de
/// [`o_hdri_uniforme_da_o_ganho_um_em_toda_a_celula`] e [`o_papel_ao_pe_da_parede_perde_o_ceu_do_lado_dela`].
#[test]
fn a_fenda_neutra_e_bit_identica_a_de_antes() {
    // §BUG-ressalto: o papel ao pé da tinta mudou por desenho (sem saia; plano) — era `0xd577_67fb_dd93_1d70`.
    let (ambiente, antes) = (Ambiente::Classico, 0x47c1_f5e4_7399_15f8u64);
    // K10: o rebote da tinta no papel mudou o PAPEL por desenho; desligado, o de antes ao bit.
    let af = Afinacao {
        rebote_papel: 0.0,
        ..Afinacao::default()
    };
    let o = OleoReal {
        ambiente,
        fenda: 1.0,
        alcance_fenda: ALCANCE_FENDA_MAX,
        forca_ambiente: FORCA_AMBIENTE_FABRICA,
        af,
        ..OleoReal::default()
    };
    assert_eq!(
        o,
        OleoReal {
            ambiente,
            af,
            ..OleoReal::default()
        },
        "o neutro é o de fábrica"
    );
    let d = impressao_digital(&o);
    assert_eq!(d, antes, "{ambiente:?} {d:#018x}");
}

/// O relevo variado da [`impressao_digital`] (cristas, um sulco fundo, um bloco alto e o papel livre).
fn relevo_variado() -> Grid {
    grade(|x, y| {
        let (fx, fy) = (x as f32, y as f32);
        let ondas = 2500.0 + 1800.0 * (fx * 0.37).sin() * (fy * 0.23).cos();
        if (30..=32).contains(&x) {
            0.0
        } else if (40..=52).contains(&x) && (10..=22).contains(&y) {
            12_000.0
        } else if x > 56 {
            0.0
        } else {
            ondas.max(0.0)
        }
    })
}

/// ⭐ **O HDRI UNIFORME é o Clássico** (W2 E2): a integral por sector usa o MESMO arco de cada fatia que a
/// visibilidade, por isso um céu de radiância constante dá ganho `1` em toda a célula (tinta, fenda, papel) —
/// a prova de que as somas acumuladas, o lado pelo sinal do arco e a normalização pelo plano aberto estão
/// certos. Mutações que sangram: o `sinal` do `sinθ·sinγ` trocado, `u + 0,5` em falta no lado negativo
/// (só com céu não uniforme — ver o gate do lado), `cima` sem o `÷ FATIAS`.
#[test]
fn o_hdri_uniforme_da_o_ganho_um_em_toda_a_celula() {
    let s = Sectores::uniforme();
    let o = OleoReal::default();
    let g = relevo_variado();
    let c = campo(&g, &o);
    let ceu = CeuDoMapa::novo(&o, 0.3, &s);
    for k in 0..3 {
        assert!((ceu.cima[k] - 1.0).abs() < 1e-3, "{:?}", ceu.cima);
    }
    let mut pior = 0.0f64;
    for cy in 2..W {
        for cx in 2..W {
            let (x, y) = (cx as isize, cy as isize);
            let n = normal(&c, x, y);
            let (v, d) = visibilidade_e_ceu(&c, x, y, n, 1.0, Some(&ceu));
            let (v0, _) = visibilidade_e_ceu(&c, x, y, n, 1.0, None);
            assert_eq!(
                v.to_bits(),
                v0.to_bits(),
                "a visibilidade não muda com o céu"
            );
            if v > 0.02 {
                for dk in d.expect("com mapa") {
                    pior = pior.max((dk - 1.0).abs());
                }
            }
        }
    }
    assert!(pior < 2e-3, "ganho do céu uniforme ≠ 1: {pior}");
}

/// O `u` do mapa de uma direcção da tela pela forma fechada de [`u_do_mapa`] é o mesmo que o
/// [`para_o_mapa`] + a convenção de `Mapa::amostra` (`atan2(y, −x)/2π + ½`).
#[test]
fn o_u_da_fatia_e_o_do_mapa() {
    use core::f64::consts::TAU;
    for az in [0.0f64, 40.0, 135.0, 225.0, 300.0] {
        let o = OleoReal {
            azimute: az.to_radians(),
            ..OleoReal::default()
        };
        for claro in [-2.2, 0.0, 1.3, 2.95] {
            let u0 = u_do_mapa(&o, claro);
            for k in 0..16 {
                let phi = TAU * k as f64 / 16.0;
                let b = para_o_mapa(&o, claro, [libm::cos(phi), libm::sin(phi), 0.0]);
                let u_ref = libm::atan2(b[1], -b[0]) / TAU + 0.5;
                let dif = (u0 + phi / TAU - u_ref).rem_euclid(1.0);
                assert!(
                    dif.min(1.0 - dif) < 1e-9,
                    "az {az} claro {claro} φ {phi}: {dif}"
                );
            }
        }
    }
}

/// ⭐ **O papel ao pé da parede perde o céu do LADO dela** (W2 E2): uma parede alta numa tela; o ambiente roda
/// o lado claro para a luz (esquerda). O papel à DIREITA da parede tem o lado claro tapado — com o Estúdio
/// escurece mais que com o céu uniforme; o da ESQUERDA tem o lado escuro tapado — escurece menos. A mesma
/// luz directa nos dois modos (`sombra` só do céu: luz a pino). Mutações que sangram: o `u + 0,5` do lado
/// negativo em falta, o `dir` ignorado no papel.
#[test]
fn o_papel_ao_pe_da_parede_perde_o_ceu_do_lado_dela() {
    let g = grade(|x, _| {
        if (30..=34).contains(&x) {
            12_000.0
        } else {
            0.0
        }
    });
    let base = OleoReal {
        azimute: 180f64.to_radians(),
        altitude: 89f64.to_radians(),
        ..OleoReal::default()
    };
    let c = campo(&g, &base);
    let com = |ambiente| OleoReal { ambiente, ..base };
    let (classico, estudio) = (com(Ambiente::Classico), com(Ambiente::Estudio));
    let s = |o: &OleoReal, x| sombra_no_papel(o, &c, x, 32);
    // O papel PLANO à esquerda da parede (do lado da luz) e à direita, fora das encostas da forma (a altura
    // desfocada vai de 24 a 40). Medido: Clássico 0,089 · 0,090; Estúdio 0,042 · 0,354.
    let (esq, dir) = (20, 43);
    assert!(
        s(&classico, dir) > 0.01,
        "a parede tapa o céu: {}",
        s(&classico, dir)
    );
    assert!(
        s(&estudio, dir) > s(&classico, dir) + 0.01,
        "à direita o lado claro está tapado: {} × {}",
        s(&estudio, dir),
        s(&classico, dir)
    );
    assert!(
        s(&estudio, esq) < s(&classico, esq) - 0.01,
        "à esquerda só o lado escuro: {} × {}",
        s(&estudio, esq),
        s(&classico, esq)
    );
}
