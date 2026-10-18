//! **O RELEVO de cada fio** (W3, prompt 49; 05 §3): o óleo do Rebelle tem fios que CONSTROEM e fios que
//! ESCAVAM (o Bristle Flat: `particle_thin_remove_impasto_strength` 0,65), e os fios escuros da foto são a
//! sombra deles. A régua lê a imagem (a textura através × ao longo, a borda) e o CAMPO do motor (a massa e o
//! relevo da grelha, não o píxel), na cena seca da bancada.

use super::midias_de_fabrica::{QUADROS_SECOS, de_fabrica};
use super::regua_midia::{Medida, RECTO, mede};
use super::sonda_gestos::{pinta_cena_ate, tela};

/// O campo do motor no recto da bancada (y 120, colunas 160–460: fora das pontas), no miolo de cada coluna
/// (a massa ≥ metade do máximo dela, menos 3 linhas de cada lado).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Campo {
    /// A massa média do miolo (`sett + susp`).
    pub massa: f64,
    /// A variação da massa ATRAVÉS (de linha para linha, resíduo a 5 px na vertical), em % da média.
    pub fios: f64,
    /// A variação AO LONGO (resíduo a 5 px na horizontal), em % da média.
    pub longo: f64,
    /// O relevo médio que as células guardaram (`Grid::relevo`).
    pub relevo: f64,
    /// O emboss que o render soma à cor (o mesmo cálculo de `render.rs`): o desvio e a fracção que bate no
    /// tecto de ±40.
    pub emboss: f64,
    pub saturado: f64,
    /// A altura dos fios (`Grid::altura`, px): o desvio no miolo.
    pub altura: f64,
}

/// A cena seca (`midias_de_fabrica::cena_seca`) com o preset `p` ao raio `raio`: a régua da imagem e o campo.
pub(super) fn cena_e_campo(p: &ph2d_brush_presets::Preset, raio: f32) -> (Medida, Campo) {
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_brush_size_px(raio);
    let (px, w, _) = pinta_cena_ate(&mut t, QUADROS_SECOS);
    let mut campo = Campo::default();
    t.wet_sonda_motor(|e| {
        let g = &e.layers[e.active_layer].grid;
        let r = (w as usize / g.w).max(1);
        let i = |x: usize, y: usize| (y / r + 1) * g.s + x / r + 1;
        let m = |x: usize, y: usize| f64::from(g.sett[i(x, y)] + g.susp[i(x, y)]);
        let (mut sm, mut sr, mut n) = (0.0, 0.0, 0.0);
        let (mut fios, mut longo, mut emb) = (Vec::new(), Vec::new(), Vec::new());
        let mut alt = Vec::new();
        for x in 160..460 {
            let col: Vec<f64> = (60..180).map(|y| m(x, y)).collect();
            let mx = col.iter().copied().fold(0.0, f64::max);
            let (Some(a), Some(b)) = (
                col.iter().position(|&v| v >= 0.5 * mx),
                col.iter().rposition(|&v| v >= 0.5 * mx),
            ) else {
                continue;
            };
            for y in (60 + a + 3)..(60 + b).saturating_sub(2) {
                let v = m(x, y);
                let eixo = |dx: isize, dy: isize| {
                    (-2..=2)
                        .map(|k| {
                            m(
                                (x as isize + k * dx) as usize,
                                (y as isize + k * dy) as usize,
                            )
                        })
                        .sum::<f64>()
                        / 5.0
                };
                fios.push(v - eixo(0, 1));
                longo.push(v - eixo(1, 0));
                let rel = f64::from(g.relevo[i(x, y)]);
                alt.push(f64::from(ph2d_wet_paint::grid::altura::em(g, i(x, y))));
                let d = (m(x + r, y) - m(x - r, y)) * 0.5 + (m(x, y + r) - m(x, y - r));
                emb.push((d * 0.008 * rel).clamp(-40.0, 40.0));
                sm += v;
                sr += rel;
                n += 1.0;
            }
        }
        let desvio = |v: &[f64]| {
            let k = v.len().max(1) as f64;
            let a = v.iter().sum::<f64>() / k;
            (v.iter().map(|x| (x - a).powi(2)).sum::<f64>() / k).sqrt()
        };
        let media = sm / f64::max(n, 1.0);
        campo = Campo {
            massa: media,
            fios: 100.0 * desvio(&fios) / media.max(1e-9),
            longo: 100.0 * desvio(&longo) / media.max(1e-9),
            relevo: sr / f64::max(n, 1.0),
            emboss: desvio(&emb),
            saturado: emb.iter().filter(|e| e.abs() >= 40.0).count() as f64
                / emb.len().max(1) as f64,
            altura: desvio(&alt),
        };
    });
    if let Ok(dir) = std::env::var("PH2D_SONDA_OUT") {
        let mut f = format!("P6\n{w} 640\n255\n").into_bytes();
        px.as_chunks::<4>()
            .0
            .iter()
            .for_each(|q| f.extend_from_slice(&q[..3]));
        std::fs::create_dir_all(&dir).unwrap();
        let n = &p.parametros.name;
        std::fs::write(format!("{dir}/relevo_{n}_{raio}.ppm"), &f).unwrap();
    }
    (mede(&px, w, RECTO), campo)
}

/// A largura do recto molhado (2 s depois dos gestos, como a foto `_2s` do Rebelle) e seco (30 s): a
/// tinta que alastra alarga-se entre os dois.
pub(super) fn largura_molhada_e_seca(p: &ph2d_brush_presets::Preset, raio: f32) -> (f64, f64) {
    let l = |quadros: u32| {
        let mut t = tela();
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(p).is_empty());
        t.set_brush_size_px(raio);
        let (px, w, _) = pinta_cena_ate(&mut t, quadros);
        mede(&px, w, RECTO).largura
    };
    (l(120), l(QUADROS_SECOS))
}

/// Aplica as variantes `k=v,b.k=v,-k` a um preset.
pub(super) fn com_knobs(mut p: ph2d_brush_presets::Preset, kv: &str) -> ph2d_brush_presets::Preset {
    for x in kv.split(',').filter(|x| !x.is_empty()) {
        if let Some(k) = x.strip_prefix('-') {
            p.parametros.engine.remove(k);
        } else if let Some((k, v)) = x.split_once('=') {
            let v: f64 = v.parse().unwrap();
            if let Some(b) = k.strip_prefix("b.") {
                p.parametros
                    .brush
                    .insert(b.to_owned(), serde_json::json!(v));
            } else {
                p.parametros.engine.insert(k.to_owned(), v);
            }
        }
    }
    p
}

/// **SONDA (W3): a largura molhada × seca** dos três óleos e das variantes (`PH2D_SONDA_KNOBS`).
#[test]
#[ignore = "sonda: a largura molhada e seca dos óleos; corre-se à mão"]
fn sonda_a_largura_molhada_e_seca() {
    let variantes: Vec<(String, String)> = std::env::var("PH2D_SONDA_KNOBS").map_or_else(
        |_| vec![(String::new(), String::new())],
        |v| {
            v.split('|')
                .map(|s| {
                    let (n, kv) = s.split_once(':').unwrap_or((s, ""));
                    (n.to_owned(), kv.to_owned())
                })
                .collect()
        },
    );
    let casos: Vec<(&str, f32, &(String, String))> = OLEOS
        .iter()
        .flat_map(|&(n, r)| variantes.iter().map(move |v| (n, r, v)))
        .collect();
    let linhas: Vec<String> = std::thread::scope(|s| {
        let hs: Vec<_> = casos
            .iter()
            .map(|&(n, r, (vn, kv))| {
                s.spawn(move || {
                    let (a, b) = largura_molhada_e_seca(&com_knobs(de_fabrica(n), kv), r);
                    format!("[largura] {n:<14} {vn:<12} r {r:4.1} molhada {a:5.1} seca {b:5.1} B−A {:+.1}", b - a)
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for l in linhas {
        println!("{l}");
    }
}

/// Os três óleos e o raio da foto de cada um (a largura dela): Oil Bristle (Bristle Flat), Acrylic Round
/// (Round Hard), Express Oil (Hairy).
pub(super) const OLEOS: [(&str, f32); 3] = [
    ("Oil Bristle", 20.5),
    ("Acrylic Round", 19.5),
    ("Express Oil", 21.0),
];

/// **SONDA (W3, passo 1): a barra da W3 HOJE** — os três óleos, à largura da foto e a 40 px, com as
/// variantes de `PH2D_SONDA_KNOBS` (`nome:k=v,b.k=v|…`; a vazia = o preset de fábrica; `-k` tira o knob):
/// a régua da imagem (textura, através, ao longo, borda, carris) e o campo do motor.
#[test]
#[ignore = "sonda: a barra da W3 nos óleos de fábrica; corre-se à mão"]
fn sonda_a_barra_da_w3() {
    let variantes: Vec<(String, String)> = std::env::var("PH2D_SONDA_KNOBS").map_or_else(
        |_| vec![(String::new(), String::new())],
        |v| {
            v.split('|')
                .map(|s| {
                    let (n, kv) = s.split_once(':').unwrap_or((s, ""));
                    (n.to_owned(), kv.to_owned())
                })
                .collect()
        },
    );
    let raios: Option<Vec<f32>> = std::env::var("PH2D_SONDA_RAIOS")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect());
    let casos: Vec<(&str, f32, &(String, String))> = OLEOS
        .iter()
        .flat_map(|&(n, rf)| {
            raios
                .clone()
                .unwrap_or_else(|| vec![rf, 20.0])
                .into_iter()
                .map(move |r| (n, r))
        })
        .flat_map(|(n, r)| variantes.iter().map(move |v| (n, r, v)))
        .collect();
    let linhas: Vec<String> = std::thread::scope(|s| {
        let hs: Vec<_> = casos
            .iter()
            .map(|&(n, raio, (vn, kv))| {
                s.spawn(move || {
                    let (m, c) = cena_e_campo(&com_knobs(de_fabrica(n), kv), raio);
                    format!(
                        "[relevo] {:<28} r {raio:4.1} tex {:5.2} atr {:5.2} long {:5.2} borda {:.3} carril {:4.2} cob {:.3} cv {:.3} | massa {:8.0} fios {:5.2}% longo {:5.2}% relevo {:.2} emb {:5.2} sat {:.3} alt {:.2}",
                        if vn.is_empty() {
                            n.to_owned()
                        } else {
                            format!("{n}@{vn}")
                        },
                        m.textura,
                        m.atraves,
                        m.ao_longo,
                        m.borda,
                        m.carril,
                        m.cobertura,
                        m.cv,
                        c.massa,
                        c.fios,
                        c.longo,
                        c.relevo,
                        c.emboss,
                        c.saturado,
                        c.altura,
                    )
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for l in linhas {
        println!("{l}");
    }
}

/// O controlo da W3: o preset sem o relevo de cada fio e com o emboss de antes (0,7).
fn sem_relevo(nome: &str) -> ph2d_brush_presets::Preset {
    com_knobs(de_fabrica(nome), "-bristleHeight,-bristleRemove,emboss=0.7")
}

/// `n` do lado do Rebelle `r` do ponto médio entre ele e o controlo `c`, e o controlo longe dele (3× o ruído).
fn no_vale(n: f64, r: f64, c: f64, ruido: f64) -> Result<(), String> {
    if (c - r).abs() <= 3.0 * ruido {
        return Err(format!("o controlo {c:.3} não se distingue do Rebelle {r}"));
    }
    if (n - r).abs() < 0.5 * (c - r).abs() {
        Ok(())
    } else {
        Err(format!(
            "{n:.3} fora do vale do Rebelle {r} (controlo {c:.3})"
        ))
    }
}

/// ⭐ **A barra da W3** (05 §3; doc 49_medicao) no Oil Bristle (o Bristle Flat do Rebelle, cujos fios constroem e
/// escavam), seco, à largura da foto e a 40 px, contra o preset SEM o relevo de cada fio:
/// - a **textura** no vale do Rebelle (5,86; ruído 0,25 — o recto × o rápido da foto, 5,9 · 6,1);
/// - o **FORTE** da rampa no vale (5,04; o controlo 3,31: as sombras são da tinta que pesa);
/// - **borda/miolo 0,8–0,9** (a barra de 08/10; Rebelle 0,87);
/// - a textura ATRAVÉS do traço (os fios), não ao longo: através ≥ 4× ao longo (Rebelle 5,84 · 1,09);
/// - o CAMPO: a altura existe no traço (desvio ≥ 0,5 px) e no controlo nem nasce.
#[test]
fn o_oleo_tem_o_relevo_de_cada_fio() {
    let casos = [20.5f32, 20.0];
    let medidas: Vec<_> = std::thread::scope(|s| {
        let hs: Vec<_> = casos
            .iter()
            .map(|&r| {
                s.spawn(move || {
                    let (p, c) = (de_fabrica("Oil Bristle"), sem_relevo("Oil Bristle"));
                    let lf = |p: &ph2d_brush_presets::Preset, f: &str| {
                        super::os_fios::cena_seca_com_rampa(p, r, f).1
                    };
                    (
                        cena_e_campo(&p, r),
                        lf(&p, &format!("relevo_{r}")),
                        cena_e_campo(&c, r),
                        lf(&c, &format!("relevo@sem_{r}")),
                    )
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut falhas = Vec::new();
    for (&r, ((m, campo), lf, (c, ccampo), clf)) in casos.iter().zip(&medidas) {
        println!(
            "[relevo] r {r}: {m:?} {campo:?} leve/forte {lf:?} · SEM {c:?} {ccampo:?} {clf:?}"
        );
        let mut f = |ok: Result<(), String>, o: &str| {
            if let Err(e) = ok {
                falhas.push(format!("r {r}: {o} {e}"));
            }
        };
        f(no_vale(m.textura, 5.86, c.textura, 0.25), "textura");
        f(no_vale(lf.1, 5.04, clf.1, 0.25), "forte");
        f(
            ((0.8..=0.9).contains(&m.borda))
                .then_some(())
                .ok_or(format!("{:.3}", m.borda)),
            "borda/miolo",
        );
        f(
            (m.atraves >= 4.0 * m.ao_longo)
                .then_some(())
                .ok_or(format!("{:.2} × {:.2}", m.atraves, m.ao_longo)),
            "através × ao longo",
        );
        f(
            (campo.altura >= 0.5 && ccampo.altura == 0.0)
                .then_some(())
                .ok_or(format!("{:.2} × {:.2}", campo.altura, ccampo.altura)),
            "o campo da altura",
        );
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// ⭐ **Cada knob do relevo MORDE, e o neutro não chama a lei** (W3): sobre o Oil Bristle sem o relevo, a
/// altura sozinha muda o traço; a fracção que escava muda-o com a altura ligada; e a fracção SEM altura dá o
/// traço de antes AO BYTE.
#[test]
fn cada_knob_do_relevo_morde() {
    use super::os_fios::traco_com_rampa;
    let base = sem_relevo("Oil Bristle");
    let com = |kv: &str| com_knobs(base.clone(), kv);
    let (p_alt, p_cava, p_neutro) = (
        com("bristleHeight=1.25"),
        com("bristleHeight=1.25,bristleRemove=0.2"),
        com("bristleRemove=0.2"),
    );
    let (antes, alt, cava, neutro) = std::thread::scope(|s| {
        let a = s.spawn(|| traco_com_rampa(&base));
        let b = s.spawn(|| traco_com_rampa(&p_alt));
        let c = s.spawn(|| traco_com_rampa(&p_cava));
        let d = s.spawn(|| traco_com_rampa(&p_neutro));
        (
            a.join().unwrap(),
            b.join().unwrap(),
            c.join().unwrap(),
            d.join().unwrap(),
        )
    });
    assert_ne!(alt, antes, "bristleHeight não morde");
    assert_ne!(cava, alt, "bristleRemove não morde");
    assert_eq!(neutro, antes, "a fracção sem altura mudou o traço");
}

/// Um traço recto (y 100, x 60 → 460) a pressão 1 e 2 s de tinta, numa tela 520×200.
fn traco(t: &mut ph2d_tool_painter::PainterTool) {
    super::sonda_gestos::gesto(t, &[[60.0, 100.0], [460.0, 100.0]], 3.0, |_| 1.0);
    for _ in 0..120 {
        ph2d_editor_core::Tool::on_tick(t, 1000.0 / 60.0);
    }
}

/// ⭐ **O relevo é só da LUZ** (W3): com as lâmpadas do Impasto apagadas o relevo não aparece — o traço do Oil
/// Bristle é o do mesmo pincel sem relevo, ao byte (a luz é a única porta do campo para a tela). O campo, esse,
/// nasce com o 1.º traço e não antes. (O undo do campo é do motor: `grid/altura_tests.rs`.)
#[test]
fn o_relevo_e_so_da_luz() {
    use super::sonda_gestos::{AZUL, fnv1a};
    use ph2d_editor_core::tool::RasterEditTool;
    let pinta = |p: &ph2d_brush_presets::Preset, luz: bool| {
        let mut t = ph2d_tool_painter::PainterTool::default();
        t.set_source(vec![255u8; 520 * 200 * 4], 520, 200);
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(p).is_empty());
        t.set_brush_size_px(20.0);
        t.set_brush_color_srgb8(AZUL);
        if !luz {
            t.set_impasto_light_intensity(0.0);
        }
        let mut campo = Vec::new();
        t.wet_sonda_motor(|e| campo.clone_from(&e.layers[e.active_layer].grid.altura));
        assert!(campo.is_empty(), "o campo nasceu antes do traço");
        traco(&mut t);
        t.wet_sonda_motor(|e| campo.clone_from(&e.layers[e.active_layer].grid.altura));
        (fnv1a(&t.run_full().0), campo.iter().any(|&h| h != 0.0))
    };
    let p = de_fabrica("Oil Bristle");
    let sem = com_knobs(p.clone(), "-bristleHeight");
    let ((apagada, campo), (sem_apagada, sem_campo)) = (pinta(&p, false), pinta(&sem, false));
    assert!(campo && !sem_campo, "o campo nasce só com o relevo");
    assert_eq!(apagada, sem_apagada, "o relevo apareceu sem luz");
    assert_ne!(
        pinta(&p, true).0,
        pinta(&sem, true).0,
        "a luz não acendeu o relevo"
    );
}

/// **A borracha tira o relevo** (W3): o Oil Bristle de fábrica e, por cima de metade do traço, a borracha do
/// mesmo pincel — onde ela passou a altura vai a zero (o mesmo pente, para o alvo 0: ~10⁻⁵ px); na outra
/// metade fica.
#[test]
fn a_borracha_tira_o_relevo() {
    use ph2d_editor_core::tool::RasterEditTool;
    let mut t = ph2d_tool_painter::PainterTool::default();
    t.set_source(vec![255u8; 520 * 200 * 4], 520, 200);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&de_fabrica("Oil Bristle")).is_empty());
    t.set_brush_size_px(20.0);
    t.set_brush_color_srgb8(super::sonda_gestos::AZUL);
    traco(&mut t);
    t.toggle_brush_eraser();
    assert!(t.brush_eraser());
    t.set_brush_size_px(30.0);
    super::sonda_gestos::gesto(&mut t, &[[40.0, 100.0], [240.0, 100.0]], 3.0, |_| 1.0);
    let mut alt = |x0: usize, x1: usize| {
        let mut m = 0.0f32;
        t.wet_sonda_motor(|e| {
            let g = &e.layers[e.active_layer].grid;
            for y in 95..106 {
                for x in x0..x1 {
                    m = m.max(ph2d_wet_paint::grid::altura::em(g, (y + 1) * g.s + x + 1).abs());
                }
            }
        });
        m
    };
    let (apagada, intacta) = (alt(80, 200), alt(300, 440));
    assert!(apagada < 1e-3, "a borracha deixou relevo ({apagada} px)");
    assert!(
        intacta > 0.1,
        "o resto do traço perdeu o relevo ({intacta})"
    );
}

/// **SONDA (W3): o que o relevo CUSTA** — o move (o carimbo + o compositor, onde a luz corre) e o tick do Oil
/// Bristle com e sem o relevo, numa tela `PH2D_SONDA_LADO` (2048) ao raio 40: as duas variantes no MESMO
/// processo, intercaladas em 7 rodadas de ordem rodada, o resumo é o MÍNIMO (MEDIR_VELOCIDADE.md).
#[test]
#[ignore = "sonda: o custo do relevo; corre-se à mão"]
fn sonda_o_custo_do_relevo() {
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
    let lado: u32 = std::env::var("PH2D_SONDA_LADO").map_or(2048, |v| v.parse().unwrap());
    let p = de_fabrica("Oil Bristle");
    let variantes = [
        ("sem", com_knobs(p.clone(), "-bristleHeight"), true),
        ("com, luz apagada", p.clone(), false),
        ("com", p, true),
    ];
    let mede = |p: &ph2d_brush_presets::Preset, luz: bool| -> (f64, f64, f64) {
        let mut t = ph2d_tool_painter::PainterTool::default();
        t.set_source(vec![255u8; (lado * lado * 4) as usize], lado, lado);
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(p).is_empty());
        t.set_brush_size_px(40.0);
        if !luz {
            t.set_impasto_light_intensity(0.0);
        }
        let y = lado as f32 * 0.5;
        let cp = |x: f32, phase| CanvasPointer {
            pos: [x, y],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(cp(100.0, PointerPhase::Down));
        let mut moves = Vec::new();
        let mut ticks = Vec::new();
        for k in 1..=150 {
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp(100.0 + 6.0 * k as f32, PointerPhase::Move));
            moves.push(t0.elapsed().as_secs_f64() * 1e3);
            let t1 = std::time::Instant::now();
            t.on_tick(1000.0 / 60.0);
            ticks.push(t1.elapsed().as_secs_f64() * 1e3);
        }
        t.on_canvas_pointer(cp(1000.0, PointerPhase::Up));
        moves.sort_by(f64::total_cmp);
        ticks.sort_by(f64::total_cmp);
        (
            moves[moves.len() / 2],
            moves[moves.len() - 1],
            ticks[ticks.len() - 1],
        )
    };
    let mut melhor = [(f64::MAX, f64::MAX, f64::MAX); 3];
    for rodada in 0..7 {
        for j in 0..3 {
            let i = (rodada + j) % 3;
            let (a, b, c) = mede(&variantes[i].1, variantes[i].2);
            let m = &mut melhor[i];
            *m = (m.0.min(a), m.1.min(b), m.2.min(c));
        }
    }
    for (i, (n, _, _)) in variantes.iter().enumerate() {
        let (a, b, c) = melhor[i];
        println!(
            "[custo] {lado}² {n}: move mediana {a:.3} ms · move pior {b:.3} ms · tick pior {c:.3} ms"
        );
    }
}
