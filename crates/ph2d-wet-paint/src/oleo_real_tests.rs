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
    let g = grade(|x, y| {
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
    });
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

/// ⭐ **Neutro = a imagem de antes** (W2 E1): com a força da fenda `1` e o alcance de fábrica a óptica é BIT
/// a BIT a de `c3365fb579` (a impressão digital gravada antes dos dois ajustes), no Clássico e no Estúdio.
#[test]
fn a_fenda_neutra_e_bit_identica_a_de_antes() {
    for (ambiente, antes) in [
        (Ambiente::Classico, 0xd577_67fb_dd93_1d70u64),
        (Ambiente::Estudio, 0xbdcc_ca7e_7461_db68),
    ] {
        let o = OleoReal {
            ambiente,
            fenda: 1.0,
            alcance_fenda: ALCANCE_FENDA_MAX,
            ..OleoReal::default()
        };
        assert_eq!(
            o,
            OleoReal {
                ambiente,
                ..OleoReal::default()
            },
            "o neutro é o de fábrica"
        );
        let d = impressao_digital(&o);
        assert_eq!(d, antes, "{ambiente:?} {d:#018x}");
    }
}
