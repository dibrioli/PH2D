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

/// ⭐ **Material NEUTRO = a imagem de antes** (W2 E3-B): com `MaterialCelula::NEUTRO` em toda a célula e o
/// cartão no neutro, os QUATRO ambientes são BIT a BIT os de `5464db050b` (impressões gravadas antes do
/// material entrar no sombreado) com o alargamento do reflexo do ambiente de W1c (`1`); com o de fábrica
/// (`0`, a cura do brilho do Estúdio, `oraculo/README.md` §E3) o Clássico não muda e os HDRI são os de agora.
#[test]
fn o_material_neutro_e_bit_identico_a_de_antes() {
    let mut erros = Vec::new();
    for (ambiente, antes, agora) in [
        // §BUG-ressalto (27/10): regravadas — o relevo sai do papel sem pigmento e o papel é plano; a tinta coberta
        // é a mesma ao bit; o papel sob a orla parcial é plano. Antes: Clássico `0xd577_67fb_dd93_1d70` (os dois) · Estúdio `0x9f6c_0278_7b71_e6fd` ·
        // `0xc068_69bd_7caf_a607` · Interior `0x47f6_a218_9632_b47f` · `0xb01e_6c58_f436_56d4` · Pátio
        // `0xbb89_005a_1c61_c90d` · `0x3ae1_6642_c9fd_624e`.
        (
            Ambiente::Classico,
            0x47c1_f5e4_7399_15f8u64,
            0x47c1_f5e4_7399_15f8u64,
        ),
        (
            Ambiente::Estudio,
            0x00f8_7fc3_815d_244bu64,
            0x75ef_0c73_92b8_506au64,
        ),
        (
            Ambiente::Interior,
            0xa2f9_d113_c2ae_d806u64,
            0x6b3a_c14b_d041_dee3u64,
        ),
        (
            Ambiente::Patio,
            0x598b_2ac4_2f05_c2b9u64,
            0x537e_b935_9fbb_cc13u64,
        ),
    ] {
        // K10: com o rebote no papel desligado (ele muda só o papel, por desenho).
        let o = OleoReal {
            ambiente,
            af: Afinacao {
                rebote_papel: 0.0,
                ..Afinacao::default()
            },
            ..OleoReal::default()
        };
        let w1c = OleoReal {
            af: Afinacao {
                antisserrilhado_ambiente: 1.0,
                ..o.af
            },
            ..o
        };
        for (rot, o, alvo) in [("W1c", w1c, antes), ("fábrica", o, agora)] {
            let d = impressao_digital(&o);
            if d != alvo {
                erros.push(format!("{ambiente:?} {rot} {d:#018x}"));
            }
        }
    }
    assert!(erros.is_empty(), "{erros:?}");
}

/// O relevo variado com material NÃO neutro (metal num bloco, verniz e veludo numa faixa, IOR Level noutra)
/// e fios em metade da tela, com a crista de uma faca num quarto — a cena dos gates das duas curas de custo.
fn relevo_com_material() -> Grid {
    let mut g = relevo_variado();
    g.material = vec![MaterialCelula::NEUTRO; g.sett.len()];
    g.altura = vec![0.0; g.sett.len()];
    g.crista = vec![0.0; g.sett.len()];
    for y in 1..=W {
        for x in 1..=W {
            let i = x + y * g.s;
            let q = &mut g.material[i];
            match (x / 12, y / 16) {
                (0, _) => q.metal = 1.0,
                (1, 1 | 2) => {
                    q.verniz = 0.8;
                    q.veludo = 0.5;
                }
                (3, _) => q.nivel_ior = 0.8,
                _ => q.rugosidade = 0.2,
            }
            if y > W / 2 {
                g.altura[i] = 0.3 * ((x as f32) * 0.8).sin();
                if x > W / 2 {
                    g.crista[i] = g.altura[i];
                }
            }
        }
    }
    g
}

/// Os óleos dos gates de custo: o Clássico, um HDRI com o cartão a pôr verniz e Roughness, e a
/// sombra a zero (o ramo sem visibilidade).
fn oleos_de_custo() -> [OleoReal; 3] {
    let classico = OleoReal {
        ambiente: Ambiente::Classico,
        ..OleoReal::default()
    };
    let hdri = OleoReal {
        ambiente: Ambiente::Interior,
        material: MaterialCartao {
            verniz: 0.5,
            rugosidade: 0.4,
            ..MaterialCartao::NEUTRO
        },
        ..OleoReal::default()
    };
    [
        classico,
        hdri,
        OleoReal {
            sombra: 0.0,
            ..hdri
        },
    ]
}

/// ⭐ **Cura 1 de custo (27/10): a sombra no papel de uma célula PINTADA opaca sai dos termos da cor dela**, bit a
/// bit a conta por célula; sob a orla parcial é a do papel PLANO (§BUG-ressalto) — e a cor é a de [`ilumina`].
/// Mutações que sangram: a `vis` sem a força da fenda, o `dir` trocado por `None`, a `luz` de outra célula.
#[test]
fn a_sombra_dos_termos_e_bit_identica_a_do_papel() {
    let g = relevo_com_material();
    let mut celulas = 0;
    for o in oleos_de_custo() {
        let c = campo(&g, &o);
        for cy in 2..W {
            for cx in 2..W {
                let rgb = [215.0, 25.0 + cx as f64, 30.0 + cy as f64];
                let cob = 0.4 + 0.01 * cx as f64;
                let (cor, s) = ilumina_e_sombra(&o, &c, cx, cy, rgb, cob);
                // §BUG-ressalto: sob a orla parcial o papel é plano (a do papel); sob a tinta opaca, a dos termos.
                let antes = if cob < 1.0 {
                    sombra_no_papel(&o, &c, cx, cy)
                } else {
                    let a = rgb.map(srgb255_to_linear);
                    let t = termos_com(&o, &c, cx, cy, &c.material_em(cx, cy), a);
                    menos_rebote(&o, &c, cx, cy, sombra_dos_termos(&o, &t))
                };
                assert_eq!(
                    s.to_bits(),
                    antes.to_bits(),
                    "{:?} ({cx}, {cy}): {s} × {antes}",
                    o.ambiente
                );
                assert_eq!(
                    cor.map(f64::to_bits),
                    ilumina(&o, &c, cx, cy, rgb, cob).map(f64::to_bits),
                    "{:?} ({cx}, {cy})",
                    o.ambiente
                );
                celulas += usize::from(antes > 0.0);
            }
        }
    }
    assert!(celulas > 100, "a cena não tem sombra: {celulas}");
}

/// As linhas em FAIXAS de 5, da última para a primeira, cada faixa no seu fio.
struct EmFaixasAoContrario;

impl Reparte for EmFaixasAoContrario {
    fn por_linhas<A: Send, B: Send>(
        &self,
        w: usize,
        a: &mut [A],
        b: &mut [B],
        f: &(dyn Fn(usize, &mut [A], &mut [B]) + Sync),
    ) {
        const N: usize = 5;
        std::thread::scope(|s| {
            let faixas: Vec<_> = a
                .chunks_mut(w * N)
                .zip(b.chunks_mut(w * N))
                .enumerate()
                .collect();
            for (j, (fa, fb)) in faixas.into_iter().rev() {
                s.spawn(move || {
                    for (k, (la, lb)) in fa.chunks_mut(w).zip(fb.chunks_mut(w)).enumerate().rev() {
                        f(j * N + k, la, lb);
                    }
                });
            }
        });
    }
}

/// ⭐ **Cura 2 de custo (27/10): o campo de altura em faixas de linhas é o MESMO bit a bit** que o em série —
/// forma, detalhe, sombra (com fios) e material — numa janela inteira e numa sub-janela presa à borda.
#[test]
fn o_campo_em_faixas_e_bit_identico_ao_em_serie() {
    let g = relevo_com_material();
    let bits = |v: &[f32]| v.iter().map(|q| q.to_bits()).collect::<Vec<_>>();
    for o in oleos_de_custo() {
        for janela in [(1, 1, W, W), (3, 40, 20, W)] {
            let serie = CampoAltura::de_com(&EmSerie, &[(&g, true)], &o, 2, janela);
            let faixas = CampoAltura::de_com(&EmFaixasAoContrario, &[(&g, true)], &o, 2, janela);
            assert_eq!(
                (serie.x0, serie.y0, serie.w, serie.h),
                (faixas.x0, faixas.y0, faixas.w, faixas.h)
            );
            assert_eq!(bits(&serie.v), bits(&faixas.v), "forma {janela:?}");
            assert_eq!(
                bits(&serie.detalhe),
                bits(&faixas.detalhe),
                "detalhe {janela:?}"
            );
            assert_eq!(
                bits(&serie.sombra),
                bits(&faixas.sombra),
                "sombra {janela:?}"
            );
            assert_eq!(serie.material, faixas.material, "material {janela:?}");
            assert!(!serie.material.is_empty() && serie.v.iter().any(|&q| q != 0.0));
        }
    }
}

/// K4 ⭐ **A sombra no papel tira LUZ, não bytes** (`oraculo/README.md` §K4): o papel branco escurecido pelo plano
/// `u8` que o composite guarda devolve, em luz linear, `1 − s` (até meio passo do byte). Guardar `s·255` tirava
/// ~2,2× a luz: o halo cinza à volta de todo traço liso que o Rebelle e o Cycles não têm.
#[test]
fn a_sombra_no_papel_tira_a_luz_linear_e_nao_o_byte() {
    use crate::colorops::transfer::srgb255_to_linear;
    for k in 0..=100 {
        let s = f64::from(k) / 100.0;
        let papel = 255.0 - f64::from(sombra_em_u8(s));
        let luz = srgb255_to_linear(papel);
        assert!(
            (luz - (1.0 - s)).abs() <= 0.005,
            "s {s}: o papel recebe {luz} em vez de {}",
            1.0 - s
        );
    }
}

/// K10 ⭐ **O rebote das paredes no papel** (`oraculo/README.md` §K10): o papel entre duas paredes acesas recebe a
/// luz delas, na COR delas (parede preta não devolve nada); a sombra projectada atrás de um bloco alto vê a face de
/// COSTAS para a luz e recebe só o céu dela (a sombra do empaste alto do dono não se apaga, K4); o papel sem tinta a
/// alcance e o rebote a `0` ficam ao bit.
#[test]
fn o_rebote_das_paredes_acende_o_vao_e_nao_a_sombra_de_costas() {
    let pinta = |cor: [f32; 3]| {
        let mut g = grade(|x, _| {
            if (16..=24).contains(&x) || (28..=36).contains(&x) {
                12_000.0
            } else {
                0.0
            }
        });
        g.sett_rgb.fill(cor);
        g
    };
    let (azul, preto) = (pinta([23.0, 101.0, 179.0]), pinta([0.0; 3]));
    // A luz vem da ESQUERDA (x−): a face direita do 2.º bloco (x 36) está de costas para ela.
    let com = OleoReal {
        azimute: 180f64.to_radians(),
        ambiente: Ambiente::Classico,
        ..OleoReal::default()
    };
    let sem = OleoReal {
        af: Afinacao {
            rebote_papel: 0.0,
            ..com.af
        },
        ..com
    };
    let (ca, cp, c0) = (campo(&azul, &com), campo(&preto, &com), campo(&azul, &sem));
    let s = |o: &OleoReal, c: &CampoAltura, x| sombra_no_papel(o, c, x, 32);
    let r = |c: &CampoAltura, x| rebote_no_papel(&com, c, x, 32);
    // O vão (x 26) e o papel ao pé da face de costas (x 40) e da face acesa (x 12), a mesma distância.
    let (vao, costas, acesa) = (r(&ca, 26), r(&ca, 40), r(&ca, 12));
    assert!(vao > 0.005, "o vão recebe a luz das paredes: {vao}");
    assert!(
        s(&com, &ca, 26) < s(&sem, &c0, 26),
        "o vão fica mais claro: {} × {}",
        s(&com, &ca, 26),
        s(&sem, &c0, 26)
    );
    assert!(
        acesa > 2.0 * costas,
        "a face de costas para a luz devolve só o céu: acesa {acesa} × costas {costas}"
    );
    assert!(
        r(&cp, 26) < 0.1 * vao,
        "a parede preta não devolve luz: {}",
        r(&cp, 26)
    );
    // No papel PLANO (fora das saias da forma, a 8 células de cada bloco) só as paredes contam: a face acesa
    // devolve o sol, a de costas só o céu.
    let (plano_aceso, plano_costas) = (r(&ca, 8), r(&ca, 44));
    assert!(
        plano_aceso > 1.5 * plano_costas && plano_costas > 0.0,
        "papel plano: face acesa {plano_aceso} × de costas {plano_costas}"
    );
    // Só o PAPEL muda: a cor da tinta é a mesma ao bit.
    for x in [16, 20, 30, 36] {
        assert_eq!(
            ilumina(&com, &ca, x, 32, [23.0, 101.0, 179.0], 1.0).map(f64::to_bits),
            ilumina(&sem, &c0, x, 32, [23.0, 101.0, 179.0], 1.0).map(f64::to_bits)
        );
    }
    // Sem tinta a alcance (só um bloco no canto, x ≥ 60) e com o rebote a 0: ao bit.
    let canto = grade(|x, y| if x >= 60 && y >= 60 { 12_000.0 } else { 0.0 });
    let (cc, cc0) = (campo(&canto, &com), campo(&canto, &sem));
    for x in [5, 20, 30] {
        assert_eq!(rebote_no_papel(&com, &cc, x, 10), 0.0, "x {x}");
        assert_eq!(
            sombra_no_papel(&com, &cc, x, 10).to_bits(),
            sombra_no_papel(&sem, &cc0, x, 10).to_bits(),
            "x {x}"
        );
    }
    for x in [12, 26, 40] {
        assert_eq!(
            sombra_no_papel(&sem, &campo(&azul, &sem), x, 32).to_bits(),
            s(&sem, &c0, x).to_bits()
        );
    }
}

/// §BUG-ressalto ⭐ **O relevo nunca sai do pigmento** (smoke do dono 27/10, `oraculo/README.md` §BUG-ressalto): a
/// forma desfocada fazia uma SAIA de altura ~3–4 células no papel à volta da tinta grossa; o papel na saia,
/// inclinado para fora, via o céu inteiro (faixa branca em relevo) e a sombra só começava no pé dela. A célula
/// sem pigmento tem altura 0 nas três escalas, e a sombra no papel é a MAIOR ao pé da parede, nos quatro lados.
#[test]
fn o_relevo_nao_sai_do_pigmento() {
    let g = grade(|x, y| {
        let fora = (20..=44).contains(&x) && (20..=44).contains(&y);
        let janela = (27..=37).contains(&x) && (27..=37).contains(&y);
        if fora && !janela { 12_000.0 } else { 0.0 }
    });
    let o = OleoReal::default();
    let c = campo(&g, &o);
    for y in 1..=W {
        for x in 1..=W {
            if g.sett[x + y * g.s] > 0.0 {
                continue;
            }
            let (xi, yi) = (x as isize, y as isize);
            let h = [c.at(xi, yi), c.at_fina(xi, yi), c.at_sombra(xi, yi)];
            assert!(h == [0.0; 3], "papel ({x}, {y}) com relevo {h:?}");
        }
    }
    // A sombra no papel cai a partir do contorno: a célula ao pé da tinta escurece pelo menos tanto como as de
    // fora (antes: 0 ao pé e a sombra só a 4 células — a faixa branca).
    let s = |x, y| sombra_no_papel(&o, &c, x, y);
    for (rot, pe, longe) in [
        ("fora, esquerda", (19, 32), (15, 32)),
        ("fora, direita", (45, 32), (49, 32)),
        ("fora, cima", (32, 19), (32, 15)),
        ("fora, baixo", (32, 45), (32, 49)),
        ("janela, esquerda", (27, 32), (30, 32)),
        ("janela, direita", (37, 32), (34, 32)),
    ] {
        let (a, b) = (s(pe.0, pe.1), s(longe.0, longe.1));
        // Folga de 0,01: no fundo da janela funda as duas já estão na sombra cheia (0,998 · 0,998).
        assert!(a + 0.01 >= b, "{rot}: ao pé {a} × mais longe {b}");
    }
    // Sob a orla PARCIAL (uma coluna de tinta fina ao pé da parede, cobertura < 1) o papel continua plano: a
    // sombra dele é a do papel ao pé, não a de uma encosta virada para o céu (o fio claro da folha).
    let mut orla = g.clone();
    for y in 20..=44 {
        orla.sett[19 + y * orla.s] = 40.0;
    }
    assert!(crate::opacity::alpha_of_mass(40.0) < 0.9);
    let co = campo(&orla, &o);
    let (_, sob) = ilumina_e_sombra(
        &o,
        &co,
        19,
        32,
        [30.0, 70.0, 210.0],
        crate::opacity::alpha_of_mass(40.0),
    );
    let longe = sombra_no_papel(&o, &co, 15, 32);
    assert!(
        sob + 0.01 >= longe,
        "o papel sob a orla parcial acende-se: {sob} × mais longe {longe}"
    );
    // Com a Subsurface a luz do céu de cada célula fica guardada com a normal DELA: o papel sob a orla não a lê.
    let sss = OleoReal {
        material: MaterialCartao {
            subsuperficie: 1.0,
            raio_subsuperficie: 4.0,
            ..MaterialCartao::NEUTRO
        },
        ..o
    };
    let cs = campo(&orla, &sss);
    let (_, sob) = ilumina_e_sombra(
        &sss,
        &cs,
        19,
        32,
        [30.0, 70.0, 210.0],
        crate::opacity::alpha_of_mass(40.0),
    );
    assert!(
        sob + 0.01 >= sombra_no_papel(&sss, &cs, 15, 32),
        "com a Subsurface o papel sob a orla acende-se: {sob}"
    );
}

/// §BUG-bordas: a tela (luminância e cobertura) de uma borda RECTA inclinada `1/8` (tinta grossa por cima de
/// `y = 20,3 + (x − 1)/8`), pela óptica do composite: a cor da tinta pela cobertura da célula (× a da orla) sobre o
/// papel com a sombra dele. Devolve `(escada, luz)`: o RMS do resíduo da cobertura somada por coluna contra a recta
/// (px) e, em cada distância `d` à recta (−3..3 px, passo ½), o RMS ao longo da borda da luminância menos a média
/// dela — o maior `d` (níveis). A borda recta é invariante ao longo dela: tudo o que varia é a grade.
pub(super) fn serrilhado_pub(o: &OleoReal) -> (f64, f64) {
    serrilhado(o)
}

fn serrilhado(o: &OleoReal) -> (f64, f64) {
    let recta = |x: usize| 20.3 + (x as f64 - 1.0) / 8.0;
    let mut g = grade(|x, y| {
        if (y as f64) <= recta(x) {
            12_000.0
        } else {
            0.0
        }
    });
    g.sett_rgb.fill([30.0, 70.0, 210.0]);
    let c = campo(&g, o);
    let camadas = [crate::render::RenderLayer {
        grid: &g,
        opacity: 1.0,
        visible: true,
    }];
    let visual = crate::render::PigmentVisual {
        paper: false,
        relevo: true,
    };
    // A tela pela porta do composite: a linha de pigmento com a sombra das células pintadas, e a do papel nu.
    let mut px = vec![[0.0f64; 2]; (W + 1) * (W + 1)];
    for y in 1..=W {
        let (mut out, mut sombra, mut feita) = (vec![0u8; W * 4], vec![0u8; W], vec![false; W]);
        crate::render::render_pigment_row_oleo_com_sombra(
            None,
            &camadas,
            visual,
            o,
            &c,
            y,
            1,
            W,
            &mut out,
            &mut sombra,
            &mut feita,
        );
        for x in 1..=W {
            if !feita[x - 1] {
                sombra[x - 1] = sombra_em_u8(sombra_no_papel(o, &c, x, y));
            }
            let p = &out[(x - 1) * 4..x * 4];
            let pa = f64::from(p[3]) / 255.0;
            let l = 0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2]);
            px[x + y * (W + 1)] = [l * pa + f64::from(255 - sombra[x - 1]) * (1.0 - pa), pa];
        }
    }
    let tela = |x: usize, y: usize| (px[x + y * (W + 1)][0], px[x + y * (W + 1)][1]);
    let colunas = 9..=56;
    let mut res = Vec::new();
    let mut perfis = Vec::new();
    for x in colunas.clone() {
        let col: Vec<(f64, f64)> = (1..=W).map(|y| tela(x, y)).collect();
        let soma: f64 = col.iter().map(|p| p.1).sum();
        res.push(soma - (recta(x) + 0.5));
        perfis.push((recta(x), col));
    }
    let mu = res.iter().sum::<f64>() / res.len() as f64;
    let escada = (res.iter().map(|r| (r - mu).powi(2)).sum::<f64>() / res.len() as f64).sqrt();
    let mut luz = 0.0f64;
    for k in -6..=6 {
        let d = f64::from(k) * 0.5;
        let v: Vec<f64> = perfis
            .iter()
            .map(|(r, col)| {
                // A célula `y` (1..) tem o centro em `y`; interpolação linear em `y`.
                let yy = r + d;
                let i = libm::floor(yy) as usize;
                let t = yy - i as f64;
                col[i - 1].0 * (1.0 - t) + col[i].0 * t
            })
            .collect();
        let m = v.iter().sum::<f64>() / v.len() as f64;
        let r = (v.iter().map(|a| (a - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt();
        luz = luz.max(r);
    }
    (escada, luz)
}

/// ⭐ §BUG-bordas (smoke do dono 27/10, `oraculo/README.md` §BUG-bordas): **a borda inclinada não tem escada.** A
/// silhueta de uma recta a 1/8 era uma escada de células e a parede, a sombra e o brilho caíam nela (escada 0,286 px
/// = o dente de serra do binário, luz 29,9 níveis no Clássico); com o contorno sub-célula a cobertura somada segue a
/// recta (0,066) e a luz ao longo da borda fica no piso do antisserrilhado IDEAL (a área exacta amostrada assim dá
/// 8,3 níveis): 8,38 · 6,32 Clássico · Estúdio. Mutações vermelhas: sem o contorno · a cobertura do composite sem ele
/// (`render.rs`) · o contorno arredondado à célula inteira.
#[test]
fn a_borda_inclinada_nao_tem_escada() {
    for ambiente in [Ambiente::Classico, Ambiente::Estudio] {
        let o = OleoReal {
            ambiente,
            ..OleoReal::default()
        };
        let (escada, luz) = serrilhado(&o);
        println!("[serrilhado] {ambiente:?}: escada {escada:.3} px · luz {luz:.2} níveis");
        assert!(
            escada <= 0.12,
            "{ambiente:?}: a silhueta em escada, {escada:.3} px"
        );
        assert!(
            luz <= 10.0,
            "{ambiente:?}: a luz em degraus ao longo da borda, {luz:.2} níveis"
        );
    }
}

#[test]
#[ignore = "medicao — o custo do contorno sub-célula dentro do campo (perfil smoke)"]
fn medida_custo_da_borda_suave() {
    const N: usize = 768;
    let mut g = Grid::new(N, N);
    for y in 1..=N {
        for x in 1..=N {
            let (dx, dy) = (x as f64 - 384.0, y as f64 - 384.0);
            if dx.hypot(dy) < 250.0 + 12.0 * (dy.atan2(dx) * 7.0).sin() {
                g.sett[x + y * g.s] = 12_000.0;
            }
        }
    }
    let o = OleoReal::default();
    let celulas = (N * N) as f64;
    let (mut campo_min, mut borda_min) = (f64::MAX, f64::MAX);
    let cob: Vec<f64> = (0..N * N)
        .map(|k| cobertura_da_celula(&[(&g, true)], (k % N + 1) + (k / N + 1) * g.s))
        .collect();
    let orla = cob.iter().filter(|&&c| c >= 0.5).count();
    for _ in 0..5 {
        let t = std::time::Instant::now();
        let c = CampoAltura::de_com(&EmSerie, &[(&g, true)], &o, 1, (1, 1, N, N));
        campo_min = campo_min.min(t.elapsed().as_secs_f64());
        std::hint::black_box(c.at(5, 5));
        let t = std::time::Instant::now();
        let b = borda_suave(&EmSerie, &cob, N, N);
        borda_min = borda_min.min(t.elapsed().as_secs_f64());
        std::hint::black_box(b.len());
    }
    println!(
        "[custo] campo {:.1} ns/célula · contorno {:.2} ns/célula ({:.1} %) · {} células de tinta",
        campo_min * 1e9 / celulas,
        borda_min * 1e9 / celulas,
        100.0 * borda_min / campo_min,
        orla
    );
}
