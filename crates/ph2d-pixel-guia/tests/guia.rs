//! Os gates do bloco G (P13 §7). Gabarito próprio (não há oráculo que se corre, §2): a lei é
//! geométrica e mede-se aqui, com a `line_cells` do núcleo como o rasterizador da Linha.

use ph2d_pixel_core::raster::{Cell, line_cells, snap_pixel_angle};
use ph2d_pixel_core::{Guias, Iso2x1};
use ph2d_pixel_guia::{centro, direcao_guiada, linhas_da_guia};

/// Distância de `p` à reta (infinita) pelos CENTROS das células `a` e `b`.
fn erro_na_reta(a: Cell, b: Cell, p: [f64; 2]) -> f64 {
    let (ca, cb) = (centro(a), centro(b));
    let (dx, dy) = (cb[0] - ca[0], cb[1] - ca[1]);
    ((p[0] - ca[0]) * dy - (p[1] - ca[1]) * dx).abs() / dx.hypot(dy)
}

fn rasteriza(a: Cell, b: Cell) -> Vec<Cell> {
    let mut v = Vec::new();
    line_cells(a.0, a.1, b.0, b.1, |x, y| v.push((x, y)));
    v
}

/// Guias de 2 PF no horizonte, à moda de uma casa em perspectiva.
fn dois_pf() -> Guias {
    Guias {
        pontos_de_fuga: [Some([-40.5, 20.5]), Some([140.5, 20.5]), None],
        horizonte: Some(20.5),
        iso: None,
    }
}

/// O PF cuja reta pela origem faz o menor ângulo (sem sentido) com o gesto `o → p`.
fn esperado(g: &Guias, o: Cell, p: Cell) -> [f64; 2] {
    let (gx, gy) = (f64::from(p.0 - o.0), f64::from(p.1 - o.1));
    let c = centro(o);
    let cos = |pf: &[f64; 2]| {
        let (vx, vy) = (pf[0] - c[0], pf[1] - c[1]);
        (gx * vx + gy * vy).abs() / (gx.hypot(gy) * vx.hypot(vy))
    };
    g.pontos_de_fuga
        .iter()
        .flatten()
        .map(|p| [f64::from(p[0]), f64::from(p[1])])
        .max_by(|a, b| cos(a).total_cmp(&cos(b)))
        .unwrap()
}

#[test]
fn a_linha_guiada_vai_ao_ponto_de_fuga() {
    let g = dois_pf();
    let mut casos = 0;
    let mut pior: f64 = 0.0;
    let mut razao: f64 = 1.0;
    let mut trocas = 0;
    let mut fora = 0;
    for ox in (10..90).step_by(7) {
        for oy in (0..64).step_by(5) {
            let o = (ox, oy);
            // O gesto aponta mais ou menos para cada PF (a mão erra uns graus), com 16..48 células.
            for (pi, pf) in g.pontos_de_fuga.iter().flatten().enumerate() {
                let (vx, vy) = (
                    f64::from(pf[0]) - centro(o)[0],
                    f64::from(pf[1]) - centro(o)[1],
                );
                let n = vx.hypot(vy);
                for len in [16.0, 23.0, 31.0, 48.0] {
                    for desvio in [-0.06_f64, 0.0, 0.05] {
                        let (s, c) = desvio.sin_cos();
                        let (ux, uy) = ((vx * c - vy * s) / n, (vx * s + vy * c) / n);
                        let p = (
                            ox + (ux * len).round() as i32,
                            oy + (uy * len).round() as i32,
                        );
                        let d = direcao_guiada(&g, o, p);
                        assert_ne!(d, o, "o destino não pode colapsar na origem ({o:?}→{p:?})");
                        // A Linha do núcleo de o→d: as células extremas são o e d, e a reta pelos
                        // seus centros, prolongada, passa pelo PF que a mão visava.
                        let cells = rasteriza(o, d);
                        assert!(cells.contains(&o) && cells.contains(&d));
                        // O PF esperado é o da reta (pela origem) de menor ângulo com o GESTO: com
                        // a mão desviada, pode ser o prolongamento da reta do outro PF.
                        let alvo = esperado(&g, o, p);
                        trocas += usize::from(alvo != [f64::from(pf[0]), f64::from(pf[1])]);
                        let e = erro_na_reta(o, d, alvo);
                        pior = pior.max(e);
                        assert!(e <= 0.5, "PF {pi}: {o:?}→{d:?} passa a {e:.3} px do PF");
                        // E o comprimento segue a mão (de ½ a 3×, a janela da lei).
                        let l = f64::from(d.0 - o.0).hypot(f64::from(d.1 - o.1));
                        assert!(l >= 0.5 * len - 1.5 && l <= 3.0 * len + 1.5, "{l} × {len}");
                        razao = razao.max(l / len).max(len / l);
                        fora += usize::from((l - len).abs() > 0.25 * len + 1.0);
                        casos += 1;
                    }
                }
            }
        }
    }
    assert!(casos > 1000, "{casos}");
    eprintln!(
        "a_linha_guiada_vai_ao_ponto_de_fuga: {casos} casos, pior {pior:.3} px, comprimento até {razao:.2}× a mão, {trocas} visam o outro PF, {fora} fora de ±25 % da mão"
    );
    // Medido: 88 de 3 744 (2,4 %) saem de ±25 % — só onde a reta curta não tem a direção.
    assert!(
        fora * 20 < casos,
        "o comprimento deixou de seguir a mão: {fora}"
    );
    assert!(
        trocas > 0 && trocas * 4 < casos,
        "o fixture tem de conter os dois regimes: {trocas}"
    );
}

#[test]
fn a_linha_guiada_com_um_pf_tambem_vai_ao_horizontal_e_ao_vertical() {
    let g = Guias {
        pontos_de_fuga: [Some([50.5, 10.5]), None, None],
        ..Guias::default()
    };
    // Gesto quase horizontal, longe da direção do PF ⇒ a horizontal exacta.
    assert_eq!(direcao_guiada(&g, (0, 60), (30, 61)), (30, 60));
    assert_eq!(direcao_guiada(&g, (0, 60), (1, 30)), (0, 30));
    // Com 3 PF não há horizontal nem vertical automáticas: segue um PF.
    let g3 = Guias {
        pontos_de_fuga: [Some([-100.5, 0.5]), Some([200.5, 0.5]), Some([50.5, 400.5])],
        ..Guias::default()
    };
    let d = direcao_guiada(&g3, (50, 60), (52, 90));
    assert!(erro_na_reta((50, 60), d, [50.5, 400.5]) <= 0.5, "{d:?}");
}

#[test]
fn a_iso_e_dois_por_um_exacto() {
    let g = Guias {
        iso: Some(Iso2x1 { origem: [0.0, 0.0] }),
        ..Guias::default()
    };
    let o = (20, 20);
    let mut casos = 0;
    for px in -30..=30 {
        for py in -30..=30 {
            let p = (o.0 + px, o.1 + py);
            if p == o {
                continue;
            }
            let d = direcao_guiada(&g, o, p);
            let (dx, dy) = (d.0 - o.0, d.1 - o.1);
            let eixo_iso = dx != 0 && dx.abs() == 2 * dy.abs();
            let vertical = dx == 0;
            assert!(eixo_iso || vertical, "{p:?} ⇒ Δ ({dx},{dy}) não é eixo iso");
            if eixo_iso {
                // Degraus 2:1 sem erro: cada linha da Linha do núcleo leva exactamente 2 células.
                let cells = rasteriza(o, d);
                let mut por_linha = std::collections::BTreeMap::<i32, i32>::new();
                for (_, y) in &cells {
                    *por_linha.entry(*y).or_default() += 1;
                }
                let n = por_linha.len();
                for (i, (_, k)) in por_linha.iter().enumerate() {
                    // Os extremos podem ter 1 (o degrau começa/acaba numa célula).
                    let esperado = if i == 0 || i + 1 == n { 1..=2 } else { 2..=2 };
                    assert!(esperado.contains(k), "{o:?}→{d:?}: degrau de {k}");
                }
                casos += 1;
            }
        }
    }
    assert!(casos > 1000, "{casos}");
}

#[test]
fn sem_guia_e_o_snap_de_hoje() {
    let g = Guias::default();
    for ax in [-3, 0, 7] {
        for px in -25..=25 {
            for py in -25..=25 {
                let (a, b) = ((ax, 2), (ax + px, 2 + py));
                assert_eq!(
                    direcao_guiada(&g, a, b),
                    snap_pixel_angle(a, b),
                    "{a:?}→{b:?}"
                );
            }
        }
    }
    // Um horizonte sozinho não é guia de direção.
    let h = Guias {
        horizonte: Some(4.0),
        ..Guias::default()
    };
    assert_eq!(
        direcao_guiada(&h, (0, 0), (9, 4)),
        snap_pixel_angle((0, 0), (9, 4))
    );
}

#[test]
fn pf_no_infinito_ou_na_origem_nao_da_nan() {
    let o = (10, 10);
    let casos = [
        [f32::INFINITY, 10.5],
        [f32::NEG_INFINITY, f32::INFINITY],
        [f32::NAN, 3.0],
        [10.5, 10.5], // o centro da origem
        [10.7, 10.3], // dentro da célula da origem
        [f32::MAX, f32::MAX],
        [-f32::MAX, 10.5],
    ];
    for pf in casos {
        let g = Guias {
            pontos_de_fuga: [Some(pf), None, None],
            ..Guias::default()
        };
        for p in [(30, 12), (10, 40), (11, 10), (10, 10), (-5, -20)] {
            let d = direcao_guiada(&g, o, p);
            // Sem NaN: o destino é uma célula plausível (perto do ponteiro, não i32::MIN/MAX).
            assert!(
                (d.0 - o.0).abs() <= 64 && (d.1 - o.1).abs() <= 64,
                "PF {pf:?}, {p:?} ⇒ {d:?}"
            );
            if p == o {
                assert_eq!(d, o);
            }
            // PF inutilizável (não finito ou na célula da origem) = PF ausente: o snap de hoje.
            let longe =
                pf.iter().all(|v| v.is_finite()) && (pf[0] - 10.5).hypot(pf[1] - 10.5) > 0.5;
            if !longe {
                assert_eq!(d, snap_pixel_angle(o, p), "PF {pf:?}, {p:?}");
            }
        }
        for (a, b) in linhas_da_guia(&g, [0.0, 0.0, 64.0, 48.0]) {
            assert!(
                a.iter().chain(&b).all(|v| v.is_finite()),
                "{pf:?}: {a:?} {b:?}"
            );
        }
    }
}

#[test]
fn o_overlay_e_o_feixe_e_a_grade_dentro_do_retangulo() {
    let r = [0.0_f32, 0.0, 128.0, 96.0];
    let g = dois_pf();
    let linhas = linhas_da_guia(&g, r);
    assert!(!linhas.is_empty());
    let dentro = |p: [f32; 2]| {
        p[0] >= r[0] - 1e-3 && p[0] <= r[2] + 1e-3 && p[1] >= r[1] - 1e-3 && p[1] <= r[3] + 1e-3
    };
    let mut raios = [0; 2];
    for (a, b) in &linhas {
        assert!(dentro(*a) && dentro(*b), "{a:?} {b:?} fora de {r:?}");
        // Cada raio, prolongado, passa por um PF (ou é o horizonte).
        let (dx, dy) = (f64::from(b[0] - a[0]), f64::from(b[1] - a[1]));
        let n = dx.hypot(dy);
        assert!(n > 0.0);
        let mut algum = (dy / n).abs() < 1e-9 && (f64::from(a[1]) - 20.5).abs() < 1e-3;
        for (i, pf) in g.pontos_de_fuga.iter().flatten().enumerate() {
            let e = ((f64::from(pf[0] - a[0])) * dy - (f64::from(pf[1] - a[1])) * dx).abs() / n;
            if e < 1e-2 {
                raios[i] += 1;
                algum = true;
            }
        }
        assert!(algum, "{a:?}→{b:?} não aponta a nenhum PF");
    }
    assert!(raios.iter().all(|&k| k >= 4), "{raios:?}");

    let iso = Guias {
        iso: Some(Iso2x1 { origem: [3.0, 5.0] }),
        ..Guias::default()
    };
    let grade = linhas_da_guia(&iso, r);
    assert!(grade.len() >= 8, "{}", grade.len());
    for (a, b) in &grade {
        assert!(dentro(*a) && dentro(*b));
        let incl = (b[1] - a[1]) / (b[0] - a[0]);
        assert!((incl.abs() - 0.5).abs() < 1e-4, "inclinação {incl}");
    }
}
