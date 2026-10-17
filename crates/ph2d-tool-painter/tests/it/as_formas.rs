//! **As formas** (prompt 44, doc [44_medicao](../../../../docs/Painter/NewFeaturesWetPaint/44_medicao_as_formas.md)):
//! a ponta RETA do pincel (`dab_quina`) e a pressão do pincel (`pressureSize` · `pressureOpacity`).

use super::midias_de_fabrica::de_fabrica;
use super::sonda_gestos::{pinta_cena_ate, tela};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::RasterEditTool;

/// O CUSTO da ponta reta (doc 44 §5): a cena da bancada com o Wet Round e com o mesmo de quina 1, no MESMO
/// processo, intercaladas em 9 rodadas; o resumo é o MÍNIMO (`docs/DevOps/MEDIR_VELOCIDADE.md`).
#[test]
#[ignore = "medição: o custo da ponta reta (prompt 44); corre-se à mão em --profile smoke"]
fn measure_o_custo_da_ponta_reta() {
    let com = |q: f64| {
        let mut p = de_fabrica("Wet Round");
        p.parametros
            .brush
            .insert("dab_quina".to_owned(), serde_json::json!(q));
        p
    };
    let (redondo, reto) = (com(0.0), com(1.0));
    let mut tempos = [Vec::new(), Vec::new()];
    for rodada in 0..9 {
        for k in 0..2 {
            let i = (k + rodada) % 2;
            let p = if i == 0 { &redondo } else { &reto };
            let mut t = tela();
            t.set_wet_relogio_fixo(true);
            assert!(t.wet_preset_aplicar(p).is_empty());
            t.set_brush_size_px(17.0);
            let t0 = std::time::Instant::now();
            let _ = pinta_cena_ate(&mut t, 60);
            tempos[i].push(t0.elapsed().as_secs_f64());
        }
    }
    let min = |v: &Vec<f64>| v.iter().copied().fold(f64::INFINITY, f64::min);
    let (a, b) = (min(&tempos[0]), min(&tempos[1]));
    println!(
        "[custo] redondo {:.1} ms · reto {:.1} ms · razão {:.3} (loadavg {})",
        a * 1e3,
        b * 1e3,
        b / a,
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// Um só carimbo (Down + Up no mesmo ponto) do Wet Round DURO com a quina `q` e o ângulo `graus`, seco; devolve
/// a área pintada (cobertura > 0,08) em px.
fn area_de_um_carimbo(q: f64, graus: u16) -> f64 {
    let mut p = de_fabrica("Wet Round");
    for (k, v) in [
        ("dab_quina", serde_json::json!(q)),
        ("dab_angle_deg", serde_json::json!(graus)),
        ("hardness", serde_json::json!(1.0)),
    ] {
        p.parametros.brush.insert(k.to_owned(), v);
    }
    p.parametros
        .engine
        .insert("pigmentPerDab".to_owned(), 600.0);
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&p).is_empty());
    t.set_brush_size_px(60.0);
    super::sonda_gestos::gesto(&mut t, &[[500.0, 320.0], [500.0, 320.0]], 3.0, |_| 1.0);
    for _ in 0..60 {
        t.on_tick(1000.0 / 60.0);
    }
    let (px, _, _) = t.run_full();
    let papel = [px[0], px[1], px[2]];
    px.as_chunks::<4>()
        .0
        .iter()
        .filter(|c| (0..3).any(|k| f64::from(c[k].abs_diff(papel[k])) / 255.0 > 0.08))
        .count() as f64
}

/// ⭐ **A ponta reta chega ao CARIMBO molhado, e o canto não é cortado** (prompt 44): o quadrado tem `4/π ≈
/// 1,27×` a área do disco, a 0° E a 45° — onde o canto sai do disco do raio (a caixa do motor tem de o
/// conter: cortado pela caixa `±r`, o quadrado rodado daria `1,05×`). Sem a quina no `fp_raio`, 1,00×.
#[test]
fn a_ponta_reta_pinta_um_quadrado_inteiro() {
    let [disco, q0, q45] = std::thread::scope(|s| {
        [(0.0, 0), (1.0, 0), (1.0, 45)]
            .map(|(q, a)| s.spawn(move || area_de_um_carimbo(q, a)))
            .map(|h| h.join().unwrap())
    });
    for (nome, a) in [("0°", q0), ("45°", q45)] {
        let r = a / disco;
        assert!(
            r > 1.15,
            "a ponta reta a {nome} pinta {r:.3}× a área do disco ({a} px contra {disco}) — o quadrado é 1,27×"
        );
    }
}

/// SONDA (prompt 44): a largura nos raios PEQUENOS de cada ingrediente do Flat sobre o Wet Round (a mesma tabela
/// de largura) — qual deles deixa o traço fino sem tinta.
#[test]
#[ignore = "sonda: a largura do Flat nos raios pequenos (prompt 44); corre-se à mão"]
fn sonda_os_raios_pequenos_do_flat() {
    let variantes: [(&str, &[(&str, serde_json::Value)]); 6] = [
        ("base", &[]),
        ("dura", &[("hardness", serde_json::json!(0.9))]),
        ("quina", &[("dab_quina", serde_json::json!(1.0))]),
        (
            "chato",
            &[
                ("dab_flatten", serde_json::json!(0.12)),
                ("dab_angle_deg", serde_json::json!(90)),
            ],
        ),
        (
            "dura+quina",
            &[
                ("hardness", serde_json::json!(0.9)),
                ("dab_quina", serde_json::json!(1.0)),
            ],
        ),
        (
            "flat",
            &[
                ("hardness", serde_json::json!(0.9)),
                ("dab_quina", serde_json::json!(1.0)),
                ("dab_flatten", serde_json::json!(0.12)),
                ("dab_angle_deg", serde_json::json!(90)),
            ],
        ),
    ];
    std::thread::scope(|s| {
        for (nome, campos) in variantes {
            s.spawn(move || {
                let mut p = de_fabrica("Wet Round");
                for (k, v) in campos {
                    p.parametros.brush.insert((*k).to_owned(), v.clone());
                }
                let l: Vec<String> = [1.5f32, 2.0, 2.5, 3.0, 4.0]
                    .iter()
                    .map(|&r| {
                        let (px, w, h) = super::sonda_gestos::traco_recto_com(r, 480, |t| {
                            t.set_wet_relogio_fixo(true);
                            assert!(t.wet_preset_aplicar(&p).is_empty());
                        });
                        format!("{r}→{}", super::sonda_gestos::largura_que_se_ve(&px, w, h))
                    })
                    .collect();
                println!("[pequenos] {nome:<11} {}", l.join(" "));
            });
        }
    });
}

/// A área de um traço curto em DIAGONAL (45°) do Wet Round duro de quina 1: `segue` = o Shape segue o traço (a
/// pegada roda por dab para 45°); senão o ângulo FIXO de 45° — o mesmo quadrado, pela caixa estática.
fn area_da_diagonal(segue: bool) -> f64 {
    let mut p = de_fabrica("Wet Round");
    let b = &mut p.parametros.brush;
    b.insert("dab_quina".to_owned(), serde_json::json!(1.0));
    b.insert("hardness".to_owned(), serde_json::json!(1.0));
    b.insert(
        "dab_angle_deg".to_owned(),
        serde_json::json!(if segue { 0 } else { 45 }),
    );
    if segue {
        b.get_mut("shape").unwrap()["rake"] = serde_json::json!(true);
    }
    p.parametros
        .engine
        .insert("pigmentPerDab".to_owned(), 600.0);
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&p).is_empty());
    t.set_brush_size_px(60.0);
    super::sonda_gestos::gesto(&mut t, &[[460.0, 280.0], [540.0, 360.0]], 3.0, |_| 1.0);
    for _ in 0..60 {
        t.on_tick(1000.0 / 60.0);
    }
    let (px, _, _) = t.run_full();
    let papel = [px[0], px[1], px[2]];
    px.as_chunks::<4>()
        .0
        .iter()
        .filter(|c| (0..3).any(|k| f64::from(c[k].abs_diff(papel[k])) / 255.0 > 0.08))
        .count() as f64
}

/// ⭐ **A ponta reta que RODA com o traço não perde os cantos** (mutação M6 do P44): com o Follow a pegada roda por
/// dab e o canto sai da caixa estática — o alcance do motor é o da pegada DESTE dab. O mesmo quadrado a 45° fixo é
/// a referência.
#[test]
fn a_ponta_reta_que_segue_o_traco_nao_perde_os_cantos() {
    let [segue, fixa] = std::thread::scope(|s| {
        [true, false]
            .map(|v| s.spawn(move || area_da_diagonal(v)))
            .map(|h| h.join().unwrap())
    });
    assert!(
        segue / fixa > 0.97,
        "a seguir o traço pinta {:.3}× a área do mesmo quadrado fixo ({segue} contra {fixa})",
        segue / fixa
    );
}

/// A cobertura da RAMPA de pressão (`regua_midia::rampa`) do Wet Flat com a `pressureOpacity` dada.
fn rampa_do_flat(opacidade: f64) -> (f64, f64) {
    rampa_de("Wet Flat", 17.0, &[("pressureOpacity", opacidade)])
}

/// A RAMPA de pressão (largura, cobertura) do preset `nome` com os knobs dados, só a linha 2 da bancada.
fn rampa_de(nome: &str, raio: f32, knobs: &[(&str, f64)]) -> (f64, f64) {
    let mut p = de_fabrica(nome);
    for &(k, v) in knobs {
        p.parametros.engine.insert(k.to_owned(), v);
    }
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&p).is_empty());
    t.set_brush_size_px(raio);
    t.set_brush_color_srgb8(super::sonda_gestos::AZUL);
    super::sonda_gestos::gesto(&mut t, &[[60.0, 300.0], [560.0, 300.0]], 3.0, |f| {
        0.15 + 0.85 * (f * std::f32::consts::PI).sin()
    });
    for _ in 0..480 {
        t.on_tick(1000.0 / 60.0);
    }
    let (px, w, h) = t.run_full();
    super::regua_midia::rampa(&px, w, h)
}

/// ⭐ **A pressão → a OPACIDADE do pincel esmaece a rampa** (mutação M8 do P44): o Wet Flat com `pressureOpacity`
/// 1 contra o mesmo a 0 — a cobertura na pressão ~0,46 cai (medido 0,54 contra ~0,7); a largura não muda com ela.
#[test]
fn a_pressao_do_pincel_esmaece_a_rampa() {
    let [com, sem] = std::thread::scope(|s| {
        [1.0, 0.0]
            .map(|o| s.spawn(move || rampa_do_flat(o)))
            .map(|h| h.join().unwrap())
    });
    assert!(
        com.1 < sem.1 - 0.08,
        "a opacidade da pressão não esmaece: cobertura {:.3} com ela, {:.3} sem",
        com.1,
        sem.1
    );
}

/// ⭐ **A opacidade da pressão MORDE no pó** (prompt 45, C28): o Airbrush com os números do Spray 2 (tamanho 0,
/// opacidade 1, curva linear) esmaece na pressão ~0,46 — antes do 45 a pressão entrava antes do tecto do
/// carimbo e o pó pousava o mesmo (medido 1,02; com a opacidade que pousa, 0,61).
#[test]
fn a_opacidade_da_pressao_morde_no_po() {
    let (_, cob) = rampa_de(
        "Airbrush",
        15.0,
        &[
            ("pressureSize", 0.0),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
        ],
    );
    assert!(cob < 0.75, "o Airbrush não esmaece com a pressão: {cob:.3}");
}

/// **A curva da pressão → opacidade** (prompt 45, C27): a curva 2 esmaece o Wet Flat mais que a linear no
/// toque leve (medido 0,27 contra 0,54) e não lhe mexe na largura.
#[test]
fn a_curva_esmaece_mais_no_toque_leve() {
    let [linear, curva] = std::thread::scope(|s| {
        [1.0, 2.0]
            .map(|g| {
                s.spawn(move || {
                    rampa_de(
                        "Wet Flat",
                        17.0,
                        &[("pressureOpacity", 1.0), ("pressureOpacityCurve", g)],
                    )
                })
            })
            .map(|h| h.join().unwrap())
    });
    assert!(
        curva.1 < linear.1 - 0.1,
        "a curva não esmaece: {:.3} contra {:.3} linear",
        curva.1,
        linear.1
    );
    assert!(
        (curva.0 - linear.0).abs() < 0.08,
        "a curva mexeu na largura: {curva:?} × {linear:?}"
    );
}

/// **Sonda (ignorada) do prompt 45**: a rampa (largura, cobertura) de cada pincel com pressão na cena seca da
/// bancada, por curva `pressureOpacityCurve` (`PH2D_SONDA_CURVAS`, vírgulas; omissão `1`). Doc 45 §2.
#[test]
#[ignore = "sonda: a rampa de pressão por curva; corre-se à mão"]
fn sonda_o_esmaecer_da_pressao() {
    type Caso = (
        &'static str,
        &'static str,
        f32,
        &'static [(&'static str, f64)],
    );
    const CASOS: [Caso; 7] = [
        ("flat", "Wet Flat", 17.0, &[]),
        ("gouache", "Gouache Flat", 19.5, &[]),
        ("air", "Airbrush", 15.0, &[]),
        ("air_spray2", "Airbrush", 15.0, &[("pressureSize", 0.0)]),
        ("ctl17", "Wet Round", 17.0, &[]),
        ("ctl19", "Wet Round", 19.5, &[]),
        ("ctl15", "Wet Round", 15.0, &[]),
    ];
    let curvas: Vec<f64> = std::env::var("PH2D_SONDA_CURVAS")
        .map(|s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect())
        .unwrap_or_else(|_| vec![1.0]);
    let r: Vec<_> = std::thread::scope(|s| {
        let hs: Vec<_> = curvas
            .iter()
            .flat_map(|&g| CASOS.map(move |c| (g, c)))
            .filter(|(g, (id, ..))| *g == curvas[0] || !id.starts_with("ctl"))
            .map(|(g, (id, nome, raio, knobs))| {
                s.spawn(move || {
                    let mut p = de_fabrica(nome);
                    for (k, v) in knobs {
                        p.parametros.engine.insert((*k).to_owned(), *v);
                    }
                    if !id.starts_with("ctl") {
                        p.parametros
                            .engine
                            .insert("pressureOpacityCurve".to_owned(), g);
                    }
                    let mut t = tela();
                    t.set_wet_relogio_fixo(true);
                    assert!(t.wet_preset_aplicar(&p).is_empty());
                    t.set_brush_size_px(raio);
                    let (px, w, h) = pinta_cena_ate(&mut t, 1800);
                    (g, id, super::regua_midia::rampa(&px, w, h))
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for (g, id, (l, c)) in r {
        println!("[rampa] curva={g} {id} larg={l:.3} cob={c:.3}");
    }
}
