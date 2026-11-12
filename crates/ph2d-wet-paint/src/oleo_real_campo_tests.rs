//! Gates do Óleo Real (filho de [`super`]): o material neutro e a sombra dos termos ao bit, o campo em faixas = em
//! série, a sombra no papel em luz linear, o rebote das paredes (partido de `oleo_real_tests.rs`, 29/10).

use super::*;

/// ⭐ **Material NEUTRO = a imagem de antes** (W2 E3-B): com `MaterialCelula::NEUTRO` em toda a célula e o
/// cartão no neutro, os QUATRO ambientes são BIT a BIT os de `5464db050b` (impressões gravadas antes do
/// material entrar no sombreado) com o alargamento do reflexo do ambiente de W1c (`1`) — desde 12/11 (o brilho
/// picotado, dono) o de fábrica outra vez: as duas colunas são iguais.
#[test]
fn o_material_neutro_e_bit_identico_a_de_antes() {
    let mut erros = Vec::new();
    for (ambiente, antes, agora) in [
        // §BUG-ressalto (27/10): regravadas — o relevo sai do papel sem pigmento e o papel é plano; a tinta coberta
        // é a mesma ao bit; o papel sob a orla parcial é plano. Antes: Clássico `0xd577_67fb_dd93_1d70` (os dois) · Estúdio `0x9f6c_0278_7b71_e6fd` ·
        // `0xc068_69bd_7caf_a607` · Interior `0x47f6_a218_9632_b47f` · `0xb01e_6c58_f436_56d4` · Pátio
        // `0xbb89_005a_1c61_c90d` · `0x3ae1_6642_c9fd_624e`.
        // Dono 31/10 (pontos pretos): o piso do ganho do HDRI (`piso_do_ganho`) muda os três HDRI por desenho; o
        // Clássico fica ao bit. Antes: Estúdio `0x00f8_7fc3_815d_244b` · `0x75ef_0c73_92b8_506a` · Interior
        // `0xa2f9_d113_c2ae_d806` · `0x6b3a_c14b_d041_dee3` · Pátio `0x598b_2ac4_2f05_c2b9` · `0x537e_b935_9fbb_cc13`.
        // Cura 02/11c: o lobo da lâmpada passa ao microfacet físico (`G/(4·n·v)` × a irradiância) — os quatro por
        // desenho. Antes: Clássico `0x47c1_f5e4_7399_15f8` · Estúdio `0x03c8_d0c8_5dc1_62fd` · `0x1b0c_d582_4ea7_dfc3` ·
        // Interior `0x098a_d848_eafe_0558` · `0x1539_eaf7_a31b_5abd` · Pátio `0x39ae_7d6d_e887_a7ca` · `0x155b_c697_578a_d5ad`.
        // Decisão do dono 12/11: a luz do traço é a única — os quatro por desenho. Antes (a Cheia, `acb4e537f3`):
        // Clássico `0xc628_2478_0671_3a32` · Estúdio `0xf8b3_5f85_223d_cfb4` · `0xc66f_56f2_6140_3359` · Interior
        // `0x28bd_f703_1fff_08e4` · `0xa11d_ff5c_d937_0c9e` · Pátio `0xcff4_3d4f_54e3_fca9` · `0x2e48_98cb_6899_0c1e`.
        // Dono 12/11 (o brilho picotado) e os portões da luz do traço: o céu lido inteiro (`CUNHA = 1/(2·FATIAS)`),
        // 4 fatias × 9 distâncias de novo (a marcha da sombra fica a do traço) e o reflexo do ambiente com o
        // antisserrilhado da lâmpada (`1`: W1c = fábrica). Antes: Clássico `0x4dbc_404d_8074_9c67` · Estúdio
        // `0x8048_ab52_5c1f_f63d` · `0xfff8_d2b6_63e2_75fd` · Interior `0xdbc1_c829_d7dd_66ad` · `0x28ff_94f2_3b75_64e0` ·
        // Pátio `0x8ac7_bed7_0e26_0f61` · `0xebd6_2df9_5c74_a0d2`.
        (
            Ambiente::Classico,
            0x38d9_b798_ba26_97adu64,
            0x38d9_b798_ba26_97adu64,
        ),
        (
            Ambiente::Estudio,
            0xc428_4436_e0c5_209cu64,
            0xc428_4436_e0c5_209cu64,
        ),
        (
            Ambiente::Interior,
            0xe4e2_9f2b_215b_9e71u64,
            0xe4e2_9f2b_215b_9e71u64,
        ),
        (
            Ambiente::Patio,
            0xa665_430c_8069_6dfdu64,
            0xa665_430c_8069_6dfdu64,
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
pub(super) fn relevo_com_material() -> Grid {
    let mut g = relevo_variado();
    g.material = crate::grid::material::campo_de(&vec![MaterialCelula::NEUTRO; g.sett.len()]);
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
pub(super) fn oleos_de_custo() -> [OleoReal; 3] {
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
pub(super) struct EmFaixasAoContrario;

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
