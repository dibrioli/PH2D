//! §BUG-bordas-W3b (smoke do dono 27/10, `oraculo/README.md` §BUG-bordas-W3b): os pontinhos ESCUROS soltos na orla
//! rasgada da tinta grossa — a régua (a de `oraculo/pontinhos_regua.py`, célula a célula) e os gates.

use super::*;

/// ⭐ A FIXTURA: a orla de BAIXO de um traço real do Dabber (fábrica, 40 px, Óleo Real ligado, cena
/// `medida_planos_da_borda_w3b` de `ph2d-tool-painter`, 27/10): a massa efectiva (`u16`, × o factor da espessura) e os
/// fios (`f32`) de 256 × 64 células — `fixtures/orla_dabber.bin` (`ORLA`, w, h, massa, fios).
fn orla_dabber() -> Grid {
    let b = include_bytes!("../fixtures/orla_dabber.bin");
    assert_eq!(&b[..4], b"ORLA");
    let (w, h) = (
        usize::from(u16::from_le_bytes([b[4], b[5]])),
        usize::from(u16::from_le_bytes([b[6], b[7]])),
    );
    let mut g = Grid::new(w, h);
    g.altura = vec![0.0; g.sett.len()];
    let (massa, fios) = b[8..].split_at(w * h * 2);
    for y in 0..h {
        for x in 0..w {
            let k = x + y * w;
            let i = (x + 1) + (y + 1) * g.s;
            g.sett[i] = f32::from(u16::from_le_bytes([massa[2 * k], massa[2 * k + 1]]));
            g.altura[i] = f32::from_le_bytes(fios[4 * k..4 * k + 4].try_into().expect("4"));
        }
    }
    g.sett_rgb.fill([30.0, 70.0, 210.0]);
    g
}

/// A tela pela porta do composite (a de `serrilhado`): `(luminância, cobertura)` de cada célula, `(w + 2)` por linha.
fn tela(g: &Grid, o: &OleoReal) -> Vec<[f64; 2]> {
    let (w, h) = (g.w, g.h);
    let c = CampoAltura::de(&[(g, true)], o, 1, 1, 1, w, h);
    let camadas = [crate::render::RenderLayer {
        grid: g,
        opacity: 1.0,
        visible: true,
    }];
    let visual = crate::render::PigmentVisual {
        paper: false,
        relevo: true,
    };
    // As linhas do pigmento e o plano da sombra (`w × h`, como o da sessão), depois a tela — pela leitura dele.
    let mut linhas = vec![[0u8; 4]; w * h];
    let mut plano = vec![0u8; w * h];
    for y in 1..=h {
        let (mut out, mut sombra, mut feita) = (vec![0u8; w * 4], vec![0u8; w], vec![false; w]);
        crate::render::render_pigment_row_oleo_com_sombra(
            None,
            &camadas,
            visual,
            o,
            &c,
            y,
            1,
            w,
            &mut out,
            &mut sombra,
            &mut feita,
        );
        for x in 1..=w {
            if !feita[x - 1] {
                sombra[x - 1] = sombra_em_u8(sombra_no_papel(o, &c, x, y));
            }
            plano[x - 1 + (y - 1) * w] = sombra[x - 1];
            linhas[x - 1 + (y - 1) * w].copy_from_slice(&out[(x - 1) * 4..x * 4]);
        }
    }
    let mut px = vec![[255.0, 0.0]; (w + 2) * (h + 2)];
    for y in 1..=h {
        for x in 1..=w {
            let p = linhas[x - 1 + (y - 1) * w];
            let s = sombra_sem_furo(&plano, (w, h), (x - 1, y - 1));
            let pa = f64::from(p[3]) / 255.0;
            let l = 0.299 * f64::from(p[0]) + 0.587 * f64::from(p[1]) + 0.114 * f64::from(p[2]);
            px[x + y * (w + 2)] = [l * pa + f64::from(255 - s) * (1.0 - pa), pa];
        }
    }
    px
}

/// A medição das variantes do [`ENTALHE_MIN`] no MESMO processo (∞ = a lei desligada).
#[test]
#[ignore = "medicao — corre-se à mão"]
fn medida_entalhe_min() {
    let g = orla_dabber();
    for ambiente in [Ambiente::Classico, Ambiente::Estudio] {
        let o = OleoReal {
            ambiente,
            ..OleoReal::default()
        };
        for (fora, v, furo) in [
            (0.0, f64::INFINITY, 255u8),
            (ORLA_FORA_MIN, ENTALHE_MIN, FURO_DA_SOMBRA),
            (0.0, ENTALHE_MIN, FURO_DA_SOMBRA),
            (ORLA_FORA_MIN, f64::INFINITY, FURO_DA_SOMBRA),
            (ORLA_FORA_MIN, ENTALHE_MIN, 255),
            (ORLA_FORA_MIN, ENTALHE_MIN, 64),
        ] {
            ORLA_FORA_DA_MEDIDA.with(|c| c.set(Some(fora)));
            FURO_DA_MEDIDA.with(|c| c.set(Some(furo)));
            {
                ENTALHE_DA_MEDIDA.with(|c| c.set(Some(v)));
                let (s, t) = pontinhos_escuros(&tela(&g, &o), (g.w, g.h));
                let (e, l) = (
                    super::oleo_real_tests::serrilhado_pub(&o).0,
                    super::oleo_real_tests::serrilhado_pub(&o).1,
                );
                println!(
                    "[entalhe] {ambiente:?} fora {fora} min {v} furo {furo}: soltos na orla {s} · em toda a parte {t} · escada {e:.3} luz {l:.2}"
                );
            }
        }
        ENTALHE_DA_MEDIDA.with(|c| c.set(None));
        ORLA_FORA_DA_MEDIDA.with(|c| c.set(None));
        FURO_DA_MEDIDA.with(|c| c.set(None));
    }
}

/// ⭐ §BUG-bordas-W3b: **a orla rasgada do Dabber não tem pontinhos escuros soltos** (foto do dono 7.png), nos dois
/// ambientes. Sem as três leis o Estúdio dava 10 na orla e 59 em toda a parte; cada uma desligada: o entalhe 10 · o
/// contorno à volta dos furos 6 · a sombra com furos 1 na orla e 3.
#[test]
fn a_orla_rasgada_nao_tem_pontinhos_escuros() {
    let g = orla_dabber();
    for ambiente in [Ambiente::Classico, Ambiente::Estudio] {
        let o = OleoReal {
            ambiente,
            ..OleoReal::default()
        };
        let (soltos, todos) = pontinhos_escuros(&tela(&g, &o), (g.w, g.h));
        println!("[pontinhos] {ambiente:?}: soltos na orla {soltos} · em toda a parte {todos}");
        assert_eq!(soltos, 0, "{ambiente:?}: {soltos} pontinhos soltos na orla");
        // Também à volta dos furos da tinta junto à orla (a ≤ 3 de QUALQUER contorno): o entalhe e o corte do
        // contorno à volta de um furo eram a outra metade dos pontinhos (sem as leis: 59).
        assert_eq!(
            todos, 0,
            "{ambiente:?}: {todos} pontinhos soltos em toda a parte"
        );
    }
}

/// O mapa da orla (para olhar): `#` papel escuro, `%` tinta escura, `X` pontinho solto.
#[test]
#[ignore = "medicao — corre-se à mão"]
fn medida_quem_sao_os_pontinhos() {
    let o = OleoReal {
        ambiente: Ambiente::Estudio,
        ..OleoReal::default()
    };
    let g = orla_dabber();
    let px = tela(&g, &o);
    let s = g.w + 2;
    for y in 25..56 {
        let linha: String = (1..=110)
            .map(|x| {
                let (l, a) = (px[x + y * s][0], px[x + y * s][1]);
                match (a >= 0.5, l < 45.0) {
                    (false, true) => '#',
                    (true, true) => '%',
                    (true, false) => 'o',
                    (false, false) if a > 0.05 => '+',
                    _ => '.',
                }
            })
            .collect();
        println!("{y:3} {linha}");
    }
    let c = CampoAltura::de(&[(&g, true)], &o, 1, 1, 1, g.w, g.h);
    let lista: Vec<(usize, usize)> = std::env::var("PH2D_PONTINHOS_EM")
        .unwrap_or_default()
        .split(';')
        .filter_map(|p| p.split_once(','))
        .map(|(a, b)| (a.parse().expect("x"), b.parse().expect("y")))
        .collect();
    for y in 1..g.h {
        for x in 1..g.w {
            let (l, a) = (px[x + y * s][0], px[x + y * s][1]);
            if lista.contains(&(x, y)) {
                for dy in -2isize..=2 {
                    let v: Vec<String> = (-2isize..=2)
                        .map(|dx| {
                            let (xx, yy) = ((x as isize + dx) as usize, (y as isize + dy) as usize);
                            format!(
                                "{:5.0}/{:3.0}/{:4.1}/{:4.2}",
                                g.sett[xx + yy * g.s],
                                px[xx + yy * s][0],
                                c.at_fina(xx as isize, yy as isize),
                                c.cobertura_da_borda(xx, yy)
                            )
                        })
                        .collect();
                    println!("  ({x},{y}) {dy:+} {}", v.join(" "));
                }
            }
            if l < 45.0 && y >= 25 && y < 40 && x <= 110 {
                let m = |dx: isize, dy: isize| {
                    g.sett[(x as isize + dx) as usize + (y as isize + dy) as usize * g.s]
                };
                println!(
                    "({x},{y}) l {l:.0} a {a:.2} massa {:.0} viz [{:.0} {:.0} {:.0} {:.0}] h {:.1} hviz [{:.1} {:.1} {:.1} {:.1}] borda {:.2}",
                    m(0, 0),
                    m(-1, 0),
                    m(1, 0),
                    m(0, -1),
                    m(0, 1),
                    c.at_fina(x as isize, y as isize),
                    c.at_fina(x as isize - 1, y as isize),
                    c.at_fina(x as isize + 1, y as isize),
                    c.at_fina(x as isize, y as isize - 1),
                    c.at_fina(x as isize, y as isize + 1),
                    c.cobertura_da_borda(x, y)
                );
            }
        }
    }
}

/// O CUSTO das leis da W3b na orla real (a fixtura do Dabber, 256 × 64), o mínimo de 9: o campo inteiro, o contorno
/// (`borda_suave`), o entalhe (`sem_entalhes`) e a leitura da sombra sem furos (por célula da janela).
#[test]
#[ignore = "medicao — o custo (perfil smoke)"]
fn medida_custo_das_leis_w3b() {
    let g = orla_dabber();
    let (w, h) = (g.w, g.h);
    let o = OleoReal::default();
    let cob: Vec<f64> = (0..w * h)
        .map(|k| cobertura_da_celula(&[(&g, true)], (k % w + 1) + (k / w + 1) * g.s))
        .collect();
    let plano: Vec<u8> = (0..w * h).map(|k| ((k * 37) % 256) as u8).collect();
    let mut m = [f64::MAX; 4];
    for _ in 0..9 {
        let t = std::time::Instant::now();
        let c = CampoAltura::de_com(&EmSerie, &[(&g, true)], &o, 1, (1, 1, w, h));
        m[0] = m[0].min(t.elapsed().as_secs_f64());
        std::hint::black_box(c.at(5, 5));
        let t = std::time::Instant::now();
        let b = borda_suave(&EmSerie, &cob, w, h);
        m[1] = m[1].min(t.elapsed().as_secs_f64());
        let t = std::time::Instant::now();
        let e = sem_entalhes(&EmSerie, &cob, b, w, h);
        m[2] = m[2].min(t.elapsed().as_secs_f64());
        std::hint::black_box(e.len());
        let t = std::time::Instant::now();
        let mut s = 0u64;
        for y in 0..h {
            for x in 0..w {
                s += u64::from(sombra_sem_furo(&plano, (w, h), (x, y)));
            }
        }
        m[3] = m[3].min(t.elapsed().as_secs_f64());
        std::hint::black_box(s);
    }
    let n = (w * h) as f64;
    println!(
        "[custo W3b] campo {:.1} ns/célula · contorno {:.2} · entalhe {:.2} · sombra sem furos {:.2} ns/célula (pior caso: plano em serra)",
        m[0] * 1e9 / n,
        m[1] * 1e9 / n,
        m[2] * 1e9 / n,
        m[3] * 1e9 / n
    );
}

/// C1 (auditoria W3b): **a sombra sem furos só enche a célula MAIS ESCURA que a mediana** — a mais clara fica como
/// está (antes: `s − mediana` estourava em dev e, sem verificação de transbordo, escurecia-a até à mediana).
#[test]
fn a_sombra_sem_furo_nao_escurece_a_celula_clara() {
    let com = |centro: u8, viz: u8| {
        let mut p = vec![viz; 9];
        p[4] = centro;
        sombra_sem_furo(&p, (3, 3), (1, 1))
    };
    assert_eq!(com(40, 200), 40, "a célula clara (s ≥ 32) entre vizinhas escuras escureceu");
    assert_eq!(com(FURO_DA_SOMBRA, 255), FURO_DA_SOMBRA, "no limiar");
    assert_eq!(com(200, 40), 40, "o furo escuro não foi cheio");
    assert_eq!(com(60, 40), 60, "abaixo do furo fica");
}

/// A fixtura do Dabber com o Smooth Edges da tela de `px` (Estúdio).
fn oleo_suave(px: f64) -> OleoReal {
    OleoReal {
        ambiente: Ambiente::Estudio,
        suave_px: px,
        ..OleoReal::default()
    }
}

/// ⭐ D1 W3b (dono 27/10: «Smooth Edges só atua na cor e não no relevo»): **com o Smooth Edges da tela o RELEVO
/// segue a silhueta amaciada** — na orla real do Dabber a cobertura da borda (o alfa do composite) e as três
/// alturas (a forma, a fina que dá a normal, a da sombra) mudam nas MESMAS células; longe da orla a cobertura fica ao
/// bit; o papel sem pigmento continua sem relevo; e a sombra no papel muda com elas.
#[test]
fn o_smooth_edges_da_tela_amacia_o_relevo_pela_mesma_silhueta() {
    let g = orla_dabber();
    let (w, h) = (g.w, g.h);
    let campo = |px: f64| CampoAltura::de(&[(&g, true)], &oleo_suave(px), 1, 1, 1, w, h);
    let (c0, c6) = (campo(0.0), campo(6.0));
    let cob = |x: usize, y: usize| cobertura_da_celula(&[(&g, true)], x + y * g.s);
    let (mut borda, mut altura, mut fina, mut sombra, mut longe, mut fora) = (0, 0, 0, 0, 0, 0);
    let mut papel_sombra = 0;
    for y in 1..=h {
        for x in 1..=w {
            let (xi, yi) = (x as isize, y as isize);
            let mudou_borda = c0.cobertura_da_borda(x, y) != c6.cobertura_da_borda(x, y);
            borda += usize::from(mudou_borda);
            if mudou_borda {
                altura += usize::from(c0.at(xi, yi) != c6.at(xi, yi));
                fina += usize::from(c0.at_fina(xi, yi) != c6.at_fina(xi, yi));
                sombra += usize::from(c0.at_sombra(xi, yi) != c6.at_sombra(xi, yi));
            }
            // Longe (> 8 células) de qualquer troca papel/mancha a cobertura não mexe.
            let m = |x: usize, y: usize| cob(x, y) >= MANCHA_DA_SILHUETA;
            let me = m(x, y);
            let perto = (y.saturating_sub(8).max(1)..=(y + 8).min(h))
                .any(|yy| (x.saturating_sub(8).max(1)..=(x + 8).min(w)).any(|xx| m(xx, yy) != me));
            if !perto && mudou_borda {
                longe += 1;
            }
            if cob(x, y) <= 0.0 && (c6.at(xi, yi) != 0.0 || c6.at_fina(xi, yi) != 0.0) {
                fora += 1;
            }
            if cob(x, y) <= 0.0 && sombra_no_papel(&oleo_suave(0.0), &c0, x, y) != sombra_no_papel(&oleo_suave(6.0), &c6, x, y)
            {
                papel_sombra += 1;
            }
        }
    }
    println!(
        "[smooth no campo] borda {borda} · altura {altura} · fina {fina} · sombra {sombra} · longe {longe} · relevo fora {fora} · sombra no papel {papel_sombra}"
    );
    assert!(borda > 100, "o Smooth Edges não mexeu na silhueta ({borda} células)");
    for (nome, n) in [("forma", altura), ("fina (a normal)", fina), ("sombra", sombra)] {
        assert!(
            n * 10 >= borda * 9,
            "a altura da {nome} só seguiu {n} das {borda} células da silhueta amaciada"
        );
    }
    assert_eq!(longe, 0, "a cobertura mudou longe da orla");
    assert_eq!(fora, 0, "o relevo saiu do pigmento");
    assert!(papel_sombra > 0, "a sombra no papel não seguiu a silhueta amaciada");
}

/// ⭐ D1 W3b: **o neutro `0` é o campo de antes, ao bit** (as alturas, a cobertura e o alcance), e o raio entra na
/// chave do campo guardado (`oleo_cache`).
#[test]
fn o_smooth_edges_zero_e_o_campo_de_antes_ao_bit() {
    let g = orla_dabber();
    let (w, h) = (g.w, g.h);
    let o = OleoReal::default();
    assert_eq!(o.suave_px, 0.0, "a fábrica liga o Smooth Edges no campo");
    let a = CampoAltura::de(&[(&g, true)], &o, 1, 20, 10, w - 20, h - 10);
    // A lei de antes: o contorno e o entalhe, sem amaciar.
    let cob: Vec<f64> = (0..a.w * a.h)
        .map(|k| cobertura_da_celula(&[(&g, true)], (a.x0 + k % a.w) + (a.y0 + k / a.w) * g.s))
        .collect();
    let antes = sem_entalhes(&EmSerie, &cob, borda_suave(&EmSerie, &cob, a.w, a.h), a.w, a.h);
    assert_eq!(a.borda, antes, "a cobertura da borda com o Smooth Edges 0 não é a de antes");
    assert_eq!(alcance(&o), MARGEM + 2 * o.af.raio_suave + 1, "o alcance de fábrica mudou");
    let com = |px: f64| ChaveCampo::de(&oleo_suave(px), &[(&g, true)], 1);
    assert_ne!(com(0.0), com(3.0), "o raio do Smooth Edges não está na chave do campo");
}

/// O CUSTO do Smooth Edges no campo (fixtura do Dabber, 256 × 64, mínimo de 9 intercalado): o campo inteiro a 0, a
/// 3 e a 6 px, em ns por célula.
#[test]
#[ignore = "medicao — o custo (perfil smoke)"]
fn medida_custo_do_smooth_edges_no_campo() {
    let g = orla_dabber();
    let (w, h) = (g.w, g.h);
    let raios = [0.0, 3.0, 6.0];
    let mut m = [f64::MAX; 3];
    for _ in 0..9 {
        for (k, &r) in raios.iter().enumerate() {
            let o = oleo_suave(r);
            let t = std::time::Instant::now();
            let c = CampoAltura::de_com(&EmSerie, &[(&g, true)], &o, 1, (1, 1, w, h));
            m[k] = m[k].min(t.elapsed().as_secs_f64());
            std::hint::black_box(c.at(5, 5));
        }
    }
    let n = (w * h) as f64;
    println!(
        "[custo Smooth Edges no campo] 0 px {:.1} · 3 px {:.1} · 6 px {:.1} ns/célula",
        m[0] * 1e9 / n,
        m[1] * 1e9 / n,
        m[2] * 1e9 / n
    );
}

/// A tinta com o GRÃO de 1 px (a Knife Stain: células cheias, ralas e vazias sorteadas por célula) numa faixa de
/// 64 × 40 com orlas rectas: `(grade, cobertura crua de cada célula)`.
fn grao_de_um_px() -> (Grid, Vec<f64>) {
    let (w, h) = (96, 64);
    let mut g = Grid::new(w, h);
    let mut crua = vec![0.0; (w + 2) * (h + 2)];
    for y in 12..52 {
        for x in 16..80 {
            let k = (x * 7_919 + y * 104_729) ^ (x * y * 31);
            let m: f32 = [0.0, 1_500.0, 6_000.0, 12_000.0, 12_000.0][k % 5];
            let i = x + y * g.s;
            g.sett[i] = m;
            crua[x + y * (w + 2)] = crate::opacity::alpha_of_mass(f64::from(m));
        }
    }
    g.sett_rgb.fill([30.0, 70.0, 210.0]);
    (g, crua)
}

/// ⭐ J6 (dono 28/10, Knife Stain com o Real Oil 368 → 162/454 na régua do Rebelle; bisseção em
/// as leis do óleo uma a uma desligadas, só sem o contorno 194 → 278): **o grão de 1 px da tinta sobrevive ao óleo.** O contorno sub-célula
/// ([`borda_suave`]) via cada célula rala do grão como a escada de uma borda e cortava-lhe a cobertura pela recta
/// média — o grão derretia em manchas. Só a silhueta que é um MEIO PLANO amostrado ([`ESCADA_ERRO_MAX`]) é escada:
/// a cobertura de composição do miolo granulado é a crua (RMS 0,0006 ≤ 0,01; sem a lei 0,323), nos dois ambientes.
#[test]
fn o_grao_de_um_px_sobrevive_ao_oleo() {
    let (g, crua) = grao_de_um_px();
    for ambiente in [Ambiente::Classico, Ambiente::Estudio] {
        let o = OleoReal {
            ambiente,
            ..OleoReal::default()
        };
        let px = tela(&g, &o);
        let s = g.w + 2;
        let (mut soma, mut n) = (0.0, 0usize);
        for y in 15..49 {
            for x in 19..77 {
                let d = px[x + y * s][1] - crua[x + y * s];
                soma += d * d;
                n += 1;
            }
        }
        let rms = (soma / n as f64).sqrt();
        println!("[grão] {ambiente:?}: RMS da cobertura contra a crua {rms:.4}");
        assert!(
            rms <= 0.01,
            "{ambiente:?}: o óleo muda a cobertura do grão de 1 px (RMS {rms:.4}) — derrete-o"
        );
    }
}
