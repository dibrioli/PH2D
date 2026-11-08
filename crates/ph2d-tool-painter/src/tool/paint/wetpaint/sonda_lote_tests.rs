//! Lote de medição de 07/11, um processo, uma compilação ([MEDIR_VELOCIDADE](../../../../../../../docs/DevOps/MEDIR_VELOCIDADE.md)):
//! por que o óleo é mais rápido que a aguada dentro do Wet Paint, e a folha do pincel acima de ~60 px. As linhas `LOTE\t…`
//! são tabuladas e resumidas fora; `PH2D_SONDA_DIR` recebe as imagens da folha.

use super::PainterTool;
use super::agua_sobre_oleo_tests::{mostrada, ponteiro};
use super::undo_tinta_morre_tests::{self as m, razoes};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_wet_paint::tuning::Knob;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const QUADRO: Duration = Duration::from_micros(16_667);

/// O CPU de cada fio deste processo (ns): o fio da tela à parte, os outros pelo nome. Lido entre as fases, nunca dentro.
fn cpu_por_fio() -> BTreeMap<String, u64> {
    let eu = std::fs::read_link("/proc/thread-self")
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_default();
    let mut v = BTreeMap::new();
    for e in std::fs::read_dir("/proc/self/task")
        .into_iter()
        .flatten()
        .flatten()
    {
        let tid = e.file_name().to_string_lossy().into_owned();
        let comm = std::fs::read_to_string(e.path().join("comm")).unwrap_or_default();
        let ns: u64 = std::fs::read_to_string(e.path().join("schedstat"))
            .ok()
            .and_then(|s| s.split(' ').next().and_then(|n| n.parse().ok()))
            .unwrap_or(0);
        let nome = if tid == eu {
            "tela".to_owned()
        } else {
            comm.trim()
                .trim_end_matches(|c: char| c.is_ascii_digit())
                .to_owned()
        };
        *v.entry(nome).or_insert(0) += ns;
    }
    v
}

fn zera_diag() {
    let _ = crate::wet_diag::take_otica();
    let _ = crate::wet_diag::take_worker();
    let _ = crate::wet_diag::take_cells();
    let _ = crate::wet_diag::take_window();
}

/// O que a grade tem, com o motor em casa: células com massa e com água, caixa da água e passos dados. Fora do cronómetro.
fn grade(t: &mut PainterTool) -> (usize, usize, i64, u64, bool) {
    t.wet_bring_home();
    let Some(s) = t.paint.wetpaint.session.as_ref() else {
        return (0, 0, 0, 0, false);
    };
    let g = &s.engine.layers[0].grid;
    let massa = g
        .sett
        .iter()
        .zip(&g.susp)
        .filter(|(a, b)| **a + **b > 0.0)
        .count();
    let agua = g.active.iter().filter(|a| **a != 0).count();
    let caixa = i64::from((g.bx1 - g.bx0 + 1).max(0)) * i64::from((g.by1 - g.by0 + 1).max(0));
    (massa, agua, caixa, s.engine.sim.frame, g.has_fluid)
}

struct Cena<'a> {
    variante: &'a str,
    res: (usize, usize),
    razao: (u8, u8),
    rodada: usize,
}

/// `n` quadros a 60 Hz, com `gesto(k)` no início de cada um; imprime a fase.
fn fase(
    t: &mut PainterTool,
    c: &Cena<'_>,
    nome: &str,
    n: usize,
    gesto: &mut dyn FnMut(&mut PainterTool, usize),
) {
    zera_diag();
    let cpu0 = cpu_por_fio();
    let frame0 = t
        .paint
        .wetpaint
        .session
        .as_ref()
        .map_or(0, |s| s.engine.sim.frame);
    let _ = frame0;
    let inicio = Instant::now();
    let mut ms = Vec::with_capacity(n);
    let (mut env, mut area, mut prox) = (0.0, 0u64, Instant::now());
    for k in 0..n {
        let a = Instant::now();
        gesto(t, k);
        t.on_tick(1000.0 / 60.0);
        let b = Instant::now();
        let saiu = t.take_preview_arc().is_some();
        env += b.elapsed().as_secs_f64() * 1e3;
        if saiu {
            if let Some((_, _, w, h)) = t.take_preview_upload_bbox() {
                area += u64::from(w) * u64::from(h);
            } else {
                area += (t.source_size.0 * t.source_size.1) as u64;
            }
        }
        ms.push(a.elapsed().as_secs_f64() * 1e3);
        prox += QUADRO;
        if let Some(d) = prox.checked_duration_since(Instant::now()) {
            std::thread::sleep(d);
        }
    }
    let parede = inicio.elapsed().as_secs_f64() * 1e3;
    let cpu1 = cpu_por_fio();
    let (copia, otica) = crate::wet_diag::take_otica();
    let (busy, away, sleep) = crate::wet_diag::take_worker();
    let celulas = crate::wet_diag::take_cells();
    let (janela_sim, janela_comp, janela_espera) = crate::wet_diag::take_window();
    let (massa, agua, caixa, frame, fluido) = grade(t);
    ms.sort_by(f64::total_cmp);
    let p = |q: f64| ms[((ms.len() - 1) as f64 * q) as usize];
    let fios: Vec<String> = cpu1
        .iter()
        .map(|(k, v)| {
            format!(
                "{k}={:.0}",
                v.saturating_sub(cpu0.get(k).copied().unwrap_or(0)) as f64 / 1e6
            )
        })
        .collect();
    eprintln!(
        "LOTE\tv\t{}\t{}x{}\t{}x{}\t{}\t{nome}\t{parede:.0}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{env:.1}\t{area}\t{busy:.1}\t{away:.1}\t{sleep:.1}\t{:.1}\t{:.1}\t{}\t{:.1}\t{}\t{:.1}\t{:.1}\t{:.1}\t{celulas}\t{massa}\t{agua}\t{caixa}\t{frame}\t{fluido}\t{}",
        c.variante,
        c.res.0,
        c.res.1,
        c.razao.0,
        c.razao.1,
        c.rodada,
        p(0.0),
        p(0.5),
        p(0.95),
        p(1.0),
        copia.0,
        otica.0,
        otica.2,
        janela_sim.0,
        janela_sim.2,
        janela_comp.0,
        janela_espera.0,
        copia.1,
        fios.join(",")
    );
}

/// Uma corrida com uma ferramenta nova (as rodadas são réplicas: J13 §5 recusa o mínimo sobre água acumulada).
fn corrida(c: &Cena<'_>, caminho: &str, relevo: bool) {
    let _r = razoes::com(c.razao.0, c.razao.1);
    let (w, h) = c.res;
    let k = w as f32 / 1920.0;
    let tamanho: f32 = std::env::var("PH2D_LOTE_TAM")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(120.0 * k);
    let mut t = m::tela(w, h, caminho, tamanho);
    t.set_wet_relogio_fixo(false);
    t.set_compor_por_quadro(true);
    if !relevo {
        t.set_wet_oleo_real_ligado(false);
    }
    fase(&mut t, c, "aquece", 5, &mut |_, _| {});
    const MOVES: usize = 120;
    let ponto = |i: usize| {
        let u = i as f32 / MOVES as f32;
        (
            (200.0 + 1500.0 * u) * k,
            (540.0 + 300.0 * (u * 9.0).sin()) * k,
        )
    };
    let (x0, y0) = ponto(0);
    t.on_canvas_pointer(ponteiro(x0, y0, PointerPhase::Down));
    fase(&mut t, c, "traco", MOVES, &mut |t, i| {
        let (x, y) = ponto(i + 1);
        t.on_canvas_pointer(ponteiro(x, y, PointerPhase::Move));
    });
    let (x1, y1) = ponto(MOVES);
    fase(&mut t, c, "soltar", 1, &mut |t, _| {
        t.on_canvas_pointer(ponteiro(x1, y1, PointerPhase::Up));
    });
    fase(&mut t, c, "assenta", 120, &mut |_, _| {});
    fase(&mut t, c, "espera", 240, &mut |_, _| {});
    fase(&mut t, c, "parado", 60, &mut |_, _| {});
}

/// ⭐ Item 2. `PH2D_LOTE_RODADAS` (omissão 3).
#[test]
#[ignore = "sonda: --ignored --nocapture --test-threads=1, perfil smoke"]
fn sonda_oleo_x_aguada_por_fase() {
    const V: [(&str, &str, bool); 7] = [
        ("Dabber", "Oil/Dabber/Dabber.ph2d-brush.png", true),
        (
            "Dabber sem relevo",
            "Oil/Dabber/Dabber.ph2d-brush.png",
            false,
        ),
        ("Flat Bristle", "Oil/Flat/Flat Bristle.ph2d-brush.png", true),
        (
            "Flat Bristle sem relevo",
            "Oil/Flat/Flat Bristle.ph2d-brush.png",
            false,
        ),
        (
            "Wet Round",
            "Watercolor/Fluid/Wet Round.ph2d-brush.png",
            true,
        ),
        ("Wet Flat", "Watercolor/Fluid/Wet Flat.ph2d-brush.png", true),
        ("Wet Wash", "Watercolor/Fluid/Wet Wash.ph2d-brush.png", true),
    ];
    let rodadas: usize = std::env::var("PH2D_LOTE_RODADAS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);
    eprintln!(
        "LOTE\tcab\trayon={}\tcarga={}",
        rayon::current_num_threads(),
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    for rodada in 0..rodadas {
        for res in [(1920, 1080), (3840, 2160)] {
            let razoes_do_lote: &[(u8, u8)] = if std::env::var_os("PH2D_LOTE_TAM").is_some() {
                &[(1, 1)]
            } else {
                &[(1, 1), (1, 2)]
            };
            for &razao in razoes_do_lote {
                for i in 0..V.len() {
                    let (nome, caminho, relevo) = V[(i + rodada * 3) % V.len()];
                    corrida(
                        &Cena {
                            variante: nome,
                            res,
                            razao,
                            rodada,
                        },
                        caminho,
                        relevo,
                    );
                }
            }
        }
    }
    eprintln!(
        "LOTE\tfim\tcarga={}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// Os pincéis da folha.
const FOLHA: [(&str, &str); 7] = [
    ("flat_bristle", "Oil/Flat/Flat Bristle.ph2d-brush.png"),
    ("thick_impasto", "Oil/Flat/Thick Impasto.ph2d-brush.png"),
    ("knife_textured", "Oil/Knife/Knife Textured.ph2d-brush.png"),
    ("fan", "Acrylic/Fan/Fan Brush.ph2d-brush.png"),
    ("dabber4", "Oil/Dabber/Dabber 4.ph2d-brush.png"),
    ("wet_wash", "Watercolor/Fluid/Wet Wash.ph2d-brush.png"),
    ("wet_round", "Watercolor/Fluid/Wet Round.ph2d-brush.png"),
];

/// A lei em simulação só vale acima do raio aprovado: abaixo dele, o pincel de fábrica ao bit.
const RAIO_APROVADO: f32 = 60.0;

/// As variantes, por override em execução: T textura, B fios, E relevo, G grade da água proporcional.
const VARIANTES: [&str; 8] = ["F", "T", "B", "E", "G", "TB", "TBE", "TBEG"];

fn traco_da_folha(px: f32, caminho: &str, variante: &str) -> (Vec<u8>, usize, usize, f64) {
    let (w, h) = ((12.0 * px) as usize, (4.0 * px) as usize);
    let d = 2.0 * RAIO_APROVADO as f64;
    let grade = if variante.contains('G') {
        ((px / RAIO_APROVADO).round() as u8).max(1)
    } else {
        1
    };
    let _r = razoes::com(grade, 1);
    let mut t = m::tela(w, h, caminho, px);
    t.set_brush_color_srgb8([40, 90, 180]);
    let k = &mut t.paint.wetpaint.knobs;
    if variante.contains('T') {
        k.set(Knob::TextureSizeRef, d);
        k.set(Knob::TextureSizeExp, 1.0);
    }
    if variante.contains('B') {
        k.set(Knob::TextureSizeRef, d);
        k.set(Knob::BristleLaneSizeExp, 1.0);
    }
    if variante.contains('E') {
        k.set(Knob::EmbossSizeRef, d);
        k.set(Knob::EmbossSizeExp, 1.0);
    }
    let a = Instant::now();
    t.on_canvas_pointer(ponteiro(1.5 * px, 2.0 * px, PointerPhase::Down));
    for i in 1..=60 {
        let u = i as f32 / 60.0;
        t.on_canvas_pointer(ponteiro(
            (1.5 + 9.0 * u) * px,
            (2.0 + 0.5 * (u * std::f32::consts::TAU).sin()) * px,
            PointerPhase::Move,
        ));
        t.ticks_em_blocos(1);
    }
    t.on_canvas_pointer(ponteiro(10.5 * px, 2.0 * px, PointerPhase::Up));
    t.ticks_em_blocos(90);
    let ms = a.elapsed().as_secs_f64() * 1e3;
    (mostrada(&mut t), w, h, ms)
}

fn grava_ppm(nome: &str, px: &[u8], w: usize, h: usize) {
    let dir = std::env::var("PH2D_SONDA_DIR").expect("PH2D_SONDA_DIR");
    let mut v = format!("P6\n{w} {h}\n255\n").into_bytes();
    v.reserve(w * h * 3);
    for p in px.as_chunks::<4>().0 {
        v.extend_from_slice(&p[..3]);
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), v).expect("gravar");
}

/// ⭐ Item 4: `pincel@tamanho.variante.ppm`, e o tempo de cada um.
#[test]
#[ignore = "sonda: --ignored --nocapture --test-threads=1, PH2D_SONDA_DIR"]
fn sonda_folha_de_tamanhos() {
    for (nome, caminho) in FOLHA {
        for px in [20.0f32, 40.0, 60.0, 90.0, 120.0, 200.0] {
            let vs: &[&str] = if px <= RAIO_APROVADO {
                &VARIANTES[..1]
            } else {
                &VARIANTES
            };
            for v in vs {
                let (img, w, h, ms) = traco_da_folha(px, caminho, v);
                grava_ppm(&format!("{nome}@{px}.{v}"), &img, w, h);
                eprintln!("LOTE\tfolha\t{nome}\t{px}\t{v}\t{ms:.0}");
            }
        }
    }
    // O custo em tamanho grande, intercalado.
    for rodada in 0..3 {
        for i in 0..4 {
            let v = ["F", "TBE", "G", "TBEG"][(i + rodada) % 4];
            let (_, _, _, ms) = traco_da_folha(200.0, FOLHA[0].1, v);
            eprintln!("LOTE\tcusto\t{v}\t{rodada}\t{ms:.0}");
        }
    }
}

/// O Dabber com um ajuste por override, no mesmo gesto da folha: a régua da «escovada» (dono 07/11).
fn traco_ajustado(
    px: f32,
    caminho: &str,
    ajusta: &dyn Fn(&mut PainterTool),
) -> (Vec<u8>, usize, usize, f64) {
    let (w, h) = ((12.0 * px) as usize, (4.0 * px) as usize);
    let _r = razoes::com(1, 1);
    let mut t = m::tela(w, h, caminho, px);
    t.set_brush_color_srgb8([40, 90, 180]);
    ajusta(&mut t);
    let a = Instant::now();
    t.on_canvas_pointer(ponteiro(1.5 * px, 2.0 * px, PointerPhase::Down));
    for i in 1..=60 {
        let u = i as f32 / 60.0;
        t.on_canvas_pointer(ponteiro(
            (1.5 + 9.0 * u) * px,
            (2.0 + 0.5 * (u * std::f32::consts::TAU).sin()) * px,
            PointerPhase::Move,
        ));
        t.ticks_em_blocos(1);
    }
    t.on_canvas_pointer(ponteiro(10.5 * px, 2.0 * px, PointerPhase::Up));
    t.ticks_em_blocos(90);
    let ms = a.elapsed().as_secs_f64() * 1e3;
    (mostrada(&mut t), w, h, ms)
}

/// ⭐ Por que o Dabber fica escovado ao crescer: cada knob fora do padrão volta ao padrão, um de cada vez, em dois
/// tamanhos, mais os sliders do Shape, o jitter e os fios que crescem com o pincel. `dabber@tamanho.variante.ppm`.
#[test]
#[ignore = "sonda: --ignored --nocapture --test-threads=1, PH2D_SONDA_DIR"]
fn sonda_a_escovada_do_dabber() {
    use ph2d_wet_paint::tuning::KNOB_DEFS;
    const DABBER: &str = m::DABBER;
    const REF: f64 = 102.0;
    let dir = std::env::var("PH2D_SONDA_DIR").expect("PH2D_SONDA_DIR");
    let base = m::tela(64, 64, DABBER, 51.0);
    let padrao = crate::tool::paint::WetKnobs::DEFAULT;
    let fora: Vec<_> = KNOB_DEFS
        .iter()
        .filter(|d| base.paint.wetpaint.knobs.get(d.knob) != padrao.get(d.knob))
        .collect();
    for d in &fora {
        eprintln!(
            "LOTE\tknob\t{}\t{}\t{}",
            d.key,
            base.paint.wetpaint.knobs.get(d.knob),
            padrao.get(d.knob)
        );
    }
    let b = &base.paint.brush;
    eprintln!(
        "LOTE\tbrush\tjitter {} forma {:?} params {:?} size {:?} flow {} mapa {:?}",
        b.jitter,
        b.shape.kind,
        &b.shape.params[..4],
        b.shape.size,
        b.shape.flow,
        b.shape.mapping
    );
    let mut vs: Vec<(String, Box<dyn Fn(&mut PainterTool)>)> =
        vec![("F".to_owned(), Box::new(|_| {}))];
    for d in fora {
        let (kn, v) = (d.knob, padrao.get(d.knob));
        vs.push((
            format!("sem_{}", d.key),
            Box::new(move |t| t.paint.wetpaint.knobs.set(kn, v)),
        ));
    }
    let fios = [
        Knob::BristlePressure,
        Knob::BristleLift,
        Knob::BristleNoiseSat,
        Knob::BristleNoiseLight,
        Knob::BristleHeight,
    ];
    vs.push((
        "sem_fios_todos".to_owned(),
        Box::new(move |t| {
            for k in fios {
                t.paint.wetpaint.knobs.set(k, 0.0);
            }
        }),
    ));
    vs.push((
        "jitter0".to_owned(),
        Box::new(|t| t.paint.brush.jitter = 0.0),
    ));
    for (nome, slot, u) in [
        ("shape_count_min", 0, 0.1f32),
        ("shape_strength_max", 1, 1.0),
        ("shape_size_min", 2, 0.0),
        ("shape_size_max", 2, 1.0),
        ("shape_felt_max", 3, 1.0),
    ] {
        vs.push((
            nome.to_owned(),
            Box::new(move |t| t.paint.brush.shape.params[slot] = u),
        ));
    }
    for (nome, tex, fio) in [
        ("cresce_tapete", true, false),
        ("cresce_fios", false, true),
        ("cresce_ambos", true, true),
    ] {
        vs.push((
            nome.to_owned(),
            Box::new(move |t| {
                let k = &mut t.paint.wetpaint.knobs;
                k.set(Knob::TextureSizeRef, REF);
                if tex {
                    k.set(Knob::TextureSizeExp, 1.0);
                }
                if fio {
                    k.set(Knob::BristleLaneSizeExp, 1.0);
                }
            }),
        ));
    }
    for px in [25.0f32, 51.0, 100.0, 200.0] {
        for (i, (nome, f)) in vs.iter().enumerate() {
            if i > 0 && px != 51.0 && px != 200.0 {
                continue;
            }
            let arq = format!("dabber@{px}.{nome}");
            if std::path::Path::new(&format!("{dir}/{arq}.ppm")).exists() {
                continue;
            }
            let (img, w, h, ms) = traco_ajustado(px, DABBER, f.as_ref());
            grava_ppm(&arq, &img, w, h);
            eprintln!("LOTE\tescovada\t{px}\t{nome}\t{ms:.0}");
        }
    }
}

/// ⭐ O relevo gravado na grade cresce com o pincel? O desvio de `altura` no miolo e o declive ao longo e através, com a
/// dureza da ponta como a outra alavanca que depende do raio.
#[test]
#[ignore = "sonda: --ignored --nocapture --test-threads=1, PH2D_SONDA_DIR"]
fn sonda_o_relevo_gravado_por_tamanho() {
    for (px, dureza) in [
        (25.0f32, None),
        (51.0, None),
        (100.0, None),
        (200.0, None),
        (51.0, Some(0.2f32)),
        (51.0, Some(2.0)),
        (200.0, Some(0.2)),
        (200.0, Some(2.0)),
    ] {
        let (w, h) = ((12.0 * px) as usize, (4.0 * px) as usize);
        let _r = razoes::com(1, 1);
        let mut t = m::tela(w, h, m::DABBER, px);
        t.set_brush_color_srgb8([40, 90, 180]);
        if let Some(d) = dureza {
            t.paint.brush.hardness = d;
        }
        t.on_canvas_pointer(ponteiro(1.5 * px, 2.0 * px, PointerPhase::Down));
        // Recto: o declive «ao longo» é o x da grade, sem a curva a misturar os eixos.
        for i in 1..=60 {
            let u = i as f32 / 60.0;
            t.on_canvas_pointer(ponteiro((1.5 + 9.0 * u) * px, 2.0 * px, PointerPhase::Move));
            t.ticks_em_blocos(1);
        }
        t.on_canvas_pointer(ponteiro(10.5 * px, 2.0 * px, PointerPhase::Up));
        t.ticks_em_blocos(90);
        let nome = dureza.map_or("F".to_owned(), |d| format!("dur{d}"));
        let (img, iw, ih) = (mostrada(&mut t), w, h);
        grava_ppm(&format!("recto@{px}.{nome}"), &img, iw, ih);
        let s = t.paint.wetpaint.session.as_mut().expect("a sessão");
        s.bring_home();
        let g = s.engine.active_grid_mut();
        let cel = |x: usize, y: usize| f64::from(g.altura[y * g.s + x]);
        let (x0, x1) = ((4.0 * px) as usize, (8.0 * px) as usize);
        let (y0, y1) = ((1.8 * px) as usize, (2.2 * px) as usize);
        let (mut n, mut soma, mut soma2, mut ao_longo, mut atraves) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for y in y0..y1 {
            for x in x0..x1 {
                let v = cel(x, y);
                n += 1.0;
                soma += v;
                soma2 += v * v;
                ao_longo += (cel(x + 1, y) - v).powi(2);
                atraves += (cel(x, y + 1) - v).powi(2);
            }
        }
        let var = soma2 / n - (soma / n) * (soma / n);
        eprintln!(
            "LOTE\taltura\t{px}\t{nome}\tdesvio {:.3}\tao_longo {:.4}\tatraves {:.4}\tcomprimento {:.1}",
            var.max(0.0).sqrt(),
            (ao_longo / n).sqrt(),
            (atraves / n).sqrt(),
            (atraves / ao_longo.max(1e-12)).sqrt(),
        );
    }
}

/// O preço de um passo do undo vivo com o fluxo do dono.
#[test]
#[ignore = "sonda: --ignored --nocapture"]
fn sonda_o_preco_de_um_passo_no_fluxo_do_dono() {
    let _r = razoes::com(1, 2);
    m::sonda_o_preco_de_um_passo();
}
