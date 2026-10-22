//! SONDA (ignored): o custo por QUADRO de um preset no caminho que o app percorre — o `PainterTool` com o preset
//! de fábrica (o tamanho dele), um traço a `vel` px por evento, `PH2D_SONDA_EVQ` eventos por quadro (3: a caneta a
//! ~180 Hz), o tique de 60 Hz e a drenagem da prévia, ao ritmo da parede — separado por fase: o depósito dos dabs,
//! o composite, o tique, a drenagem. Variantes no MESMO processo, intercaladas por rodada; o resumo é o MÍNIMO
//! das medianas das rodadas (a mediana das rodadas ao lado). `docs/Painter/Pinceis/exames/custo_no_app.md`.
//!
//! ```text
//! PH2D_SONDA_PRESETS="Filbert|Hard Round|Oil Bristle|Wet Round" PH2D_SONDA_VELS=3,30 PH2D_SONDA_PISOS=35000,max \
//!   cargo test -p ph2d-tool-painter --profile smoke --test it sonda_custo_no_app -- --ignored --nocapture
//! ```
//! `PH2D_SONDA_PISOS`: o piso de tempo do depósito (`ph2d_wet_paint::par::PISO_NS_DEPOSITO`) de cada variante —
//! `max` = a rota de antes (abaixo do piso de células, sempre em série). `PH2D_SONDA_TAMS` = diâmetros (0 = o do
//! preset). `PH2D_SONDA_ABLA="k=v;k=v,k2=v2"` = variantes do 1.º pincel com chaves do motor trocadas
//! (`brush.<campo>` para o pincel).

use crate::exame_bancada::{Fonte, Pincel, pinceis};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PainterTool;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

fn lista(var: &str, omissao: &str) -> Vec<String> {
    std::env::var(var)
        .unwrap_or_else(|_| omissao.to_owned())
        .split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Os ms de cada quadro de UM bloco, por fase.
#[derive(Default, Clone)]
struct Fases {
    dep: Vec<f64>,
    comp: Vec<f64>,
    tick: Vec<f64>,
    dren: Vec<f64>,
    tot: Vec<f64>,
}

fn q(v: &[f64], f: f64) -> f64 {
    let mut v = v.to_vec();
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * f) as usize]
}

/// Um traço de `quadros` quadros: `por_quadro` eventos de `vel` px, em zigue-zague na tela.
fn traco(
    t: &mut PainterTool,
    lado: u32,
    vel: f32,
    por_quadro: usize,
    quadros: usize,
    y0: f32,
) -> Fases {
    let mut f = Fases::default();
    let (mut x, mut dir, mut y) = (40.0f32, 1.0f32, y0);
    t.on_canvas_pointer(cp([x, y], PointerPhase::Down));
    let _ = ph2d_tool_painter::wet_diag::take_stamp();
    let periodo = Duration::from_secs_f64(1.0 / 60.0);
    let mut prazo = Instant::now();
    for _ in 0..quadros {
        let q0 = Instant::now();
        for _ in 0..por_quadro {
            x += dir * vel;
            if x > lado as f32 - 40.0 || x < 40.0 {
                dir = -dir;
                y = (y + 60.0).min(lado as f32 - 40.0);
                x = x.clamp(40.0, lado as f32 - 40.0);
            }
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        let q1 = Instant::now();
        t.on_tick(1000.0 / 60.0);
        let q2 = Instant::now();
        let _ = t.take_preview_arc();
        let _ = t.take_preview_upload_bbox();
        let q3 = Instant::now();
        let (_, dep, comp, _) = ph2d_tool_painter::wet_diag::take_stamp();
        let ptr = (q1 - q0).as_secs_f64() * 1e3;
        // O Digital não passa pelo `note_stamp`: o ponteiro inteiro conta como depósito.
        f.dep.push(if dep + comp > 0.0 { dep } else { ptr });
        f.comp.push(comp);
        f.tick.push((q2 - q1).as_secs_f64() * 1e3);
        f.dren.push((q3 - q2).as_secs_f64() * 1e3);
        f.tot.push((q3 - q0).as_secs_f64() * 1e3);
        prazo += periodo;
        match prazo.checked_duration_since(Instant::now()) {
            Some(d) => std::thread::sleep(d),
            None => prazo = Instant::now(),
        }
    }
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
    f
}

fn ablacoes(ps: &mut Vec<Pincel>) {
    let Ok(abla) = std::env::var("PH2D_SONDA_ABLA") else {
        return;
    };
    let Fonte::Preset { preset, relativo } = &ps[0].fonte else {
        panic!("ablação só sobre preset")
    };
    let (preset, relativo) = (preset.clone(), relativo.clone());
    for var in abla.split(';').filter(|s| !s.is_empty()) {
        let mut p = preset.clone();
        for kv in var.split(',') {
            let (k, v) = kv.split_once('=').expect("k=v");
            if let Some(b) = k.strip_prefix("brush.") {
                p.parametros
                    .brush
                    .insert(b.into(), serde_json::from_str(v).expect("json"));
            } else {
                p.parametros
                    .engine
                    .insert(k.into(), v.parse().expect("f64"));
            }
        }
        ps.push(Pincel {
            nome: format!("[{var}]"),
            fonte: Fonte::Preset {
                preset: p,
                relativo: relativo.clone(),
            },
        });
    }
}

/// O piso de tempo do depósito do motor; `u64::MAX` = a rota de antes.
fn pisos_do_motor(ns: u64) {
    ph2d_wet_paint::par::PISO_NS_DEPOSITO.store(ns, Ordering::Relaxed);
}

/// Uma ferramenta por (pincel, tamanho), aquecida fora da régua; `blocos[vel][piso][rodada]`.
struct Caso {
    nome: String,
    diam: f32,
    t: PainterTool,
    blocos: Vec<Vec<Vec<Fases>>>,
}

#[test]
#[ignore = "sonda: o custo por quadro no caminho do app, por fase, intercalado"]
fn sonda_custo_no_app() {
    let spec = std::env::var("PH2D_SONDA_PRESETS").unwrap_or_else(|_| "Filbert".into());
    let lado: u32 = std::env::var("PH2D_SONDA_TELA").map_or(1024, |v| v.parse().unwrap());
    let num = |var: &str, omissao: &str| -> Vec<f32> {
        lista(var, omissao)
            .iter()
            .map(|v| v.parse().unwrap())
            .collect()
    };
    let (vels, tams) = (num("PH2D_SONDA_VELS", "3,30"), num("PH2D_SONDA_TAMS", "0"));
    let pisos: Vec<u64> = lista("PH2D_SONDA_PISOS", "35000")
        .iter()
        .map(|v| {
            if v == "max" {
                u64::MAX
            } else {
                v.parse().unwrap()
            }
        })
        .collect();
    let por_quadro: usize = std::env::var("PH2D_SONDA_EVQ").map_or(3, |v| v.parse().unwrap());
    let quadros: usize = std::env::var("PH2D_SONDA_QUADROS").map_or(40, |v| v.parse().unwrap());
    let rodadas: usize = std::env::var("PH2D_SONDA_RODADAS").map_or(3, |v| v.parse().unwrap());
    let mut ps = pinceis(&spec);
    ablacoes(&mut ps);
    let carga = || {
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
            .to_owned()
    };
    println!(
        "[sonda] load {} · tela {lado}² · {por_quadro} eventos/quadro",
        carga()
    );
    let piso_de_antes = ph2d_wet_paint::par::PISO_NS_DEPOSITO.load(Ordering::Relaxed);
    let mut casos: Vec<Caso> = Vec::new();
    for p in &ps {
        for &tam in &tams {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (lado * lado * 4) as usize], lado, lado);
            p.aplica(&mut t, false);
            if tam > 0.0 {
                t.set_brush_size_px(tam * 0.5);
            }
            let diam = t.brush_size_px() * 2.0;
            let _ = traco(&mut t, lado, 6.0, por_quadro, 10, 60.0);
            casos.push(Caso {
                nome: p.nome.clone(),
                diam,
                t,
                blocos: vec![vec![Vec::new(); pisos.len()]; vels.len()],
            });
        }
    }
    let variantes: Vec<(usize, usize)> = (0..vels.len())
        .flat_map(|v| (0..pisos.len()).map(move |p| (v, p)))
        .collect();
    let n = casos.len();
    for r in 0..rodadas {
        for k in 0..n {
            let c = &mut casos[(k + r) % n];
            for j in 0..variantes.len() {
                let (vi, pi) = variantes[(j + r) % variantes.len()];
                pisos_do_motor(pisos[pi]);
                let y0 = 100.0 + (37.0 * (r * variantes.len() + j) as f32) % (lado as f32 - 200.0);
                let f = traco(&mut c.t, lado, vels[vi], por_quadro, quadros, y0);
                c.blocos[vi][pi].push(f);
            }
        }
    }
    pisos_do_motor(piso_de_antes);
    println!(
        "[sonda] {:<16} {:>5} {:>6} {:>6} | {:>7} {:>6} {:>6} {:>6} | {:>7} {:>7} {:>7} {:>7}",
        "pincel",
        "diam",
        "px/qd",
        "piso",
        "dep",
        "comp",
        "tick",
        "dren",
        "TOTAL",
        "med.rod",
        "p95",
        ">16,7"
    );
    for c in &casos {
        for (vi, v) in vels.iter().enumerate() {
            for (pi, piso) in pisos.iter().enumerate() {
                let rs = &c.blocos[vi][pi];
                let min_de = |sel: fn(&Fases) -> &Vec<f64>| {
                    rs.iter()
                        .map(|f| q(sel(f), 0.5))
                        .fold(f64::INFINITY, f64::min)
                };
                let tots: Vec<f64> = rs.iter().map(|f| q(&f.tot, 0.5)).collect();
                let todos: Vec<f64> = rs.iter().flat_map(|f| f.tot.iter().copied()).collect();
                let acima = todos.iter().filter(|&&x| x > 1000.0 / 60.0).count();
                let rotulo = if *piso == u64::MAX {
                    "antes".to_owned()
                } else {
                    format!("{}us", piso / 1000)
                };
                println!(
                    "[sonda] {:<16} {:>5.0} {:>6.0} {:>6} | {:>7.2} {:>6.2} {:>6.2} {:>6.2} | {:>7.2} {:>7.2} {:>7.2} {:>3}/{:<3}",
                    c.nome,
                    c.diam,
                    v * por_quadro as f32,
                    rotulo,
                    min_de(|f| &f.dep),
                    min_de(|f| &f.comp),
                    min_de(|f| &f.tick),
                    min_de(|f| &f.dren),
                    tots.iter().copied().fold(f64::INFINITY, f64::min),
                    q(&tots, 0.5),
                    q(&todos, 0.95),
                    acima,
                    todos.len()
                );
            }
        }
    }
    println!("[sonda] load {}", carga());
}
