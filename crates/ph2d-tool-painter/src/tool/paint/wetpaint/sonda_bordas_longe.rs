//! **Sonda (ignorada): um traço novo LONGE não pode mudar um byte do traço antigo** (fotos do dono 05/11: depois de um
//! pincel escuro de cerdas as bordas macias de todos os traços ficam duras). Varre B por todos os presets de fábrica
//! do Wet Paint, com um braço de controle na mesma sequência de tiques, e separa os momentos: aplicar, traçar, assentar.
//! `PH2D_SONDA_MATRIZ=A:papel:fora:seco,...`; `PH2D_SONDA_SO=nome` restringe B; `PH2D_SONDA_FOTOS=dir` grava as fotos.
use super::offthread::otica::forca_no_teste;
use crate::tool::PainterTool;
use ph2d_brush_presets::{Biblioteca, Preset};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_wet_paint::tuning::Knob;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

pub(super) const W: usize = 480;
pub(super) const H: usize = 340;
/// As linhas da região do A: o traço a `y = 70`, o B a `y = 270`, 200 px de papel.
pub(super) const REGIAO: usize = 160;
/// Só o lado longe do encosto (`x < 300`).
const XMAX: usize = 300;

#[derive(Clone)]
pub(super) struct Cfg {
    pub a: String,
    pub papel: bool,
    pub fora: bool,
    pub seco: bool,
    /// B passa antes a encostar em A (`y = 100`) e só então o traço longe.
    pub encosta: bool,
    /// A razão da grade (`1` = uma célula por píxel).
    pub razao: f64,
    /// Tiques de espera de A e depois de B (os do braço de controle são os mesmos).
    pub espera: usize,
    pub depois: usize,
}

fn p(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 0.8,
        tilt: [0.0, 0.0],
        phase,
    }
}

thread_local!(static SAIDA: std::cell::RefCell<Option<std::sync::Arc<Vec<u8>>>> = const { std::cell::RefCell::new(None) });

/// O dreno do editor: a saída que o ecrã mostra (a luz do papel e do relevo incluída).
fn drena(t: &mut PainterTool) {
    if let Some((a, _, _)) = t.take_preview_arc() {
        SAIDA.with(|s| *s.borrow_mut() = Some(a));
    }
}

/// O que o ecrã mostra agora.
pub(super) fn ecra(t: &mut PainterTool) -> Vec<u8> {
    drena(t);
    SAIDA.with(|s| s.borrow().as_ref().map(|a| a.to_vec()).unwrap_or_default())
}

pub(super) fn tiques(t: &mut PainterTool, n: usize, fora: bool) {
    for _ in 0..n {
        t.on_tick(1000.0 / 60.0);
        drena(t);
        if fora {
            std::thread::sleep(Duration::from_millis(8));
        }
    }
}

pub(super) fn traco(t: &mut PainterTool, y: f32, fora: bool) {
    t.on_canvas_pointer(p(60.0, y, PointerPhase::Down));
    for k in 1..=40 {
        let x = 60.0 + 9.0 * k as f32;
        t.on_canvas_pointer(p(x, y + (k as f32 * 0.4).sin() * 3.0, PointerPhase::Move));
        t.on_tick(8.0);
        drena(t);
        if fora {
            std::thread::sleep(Duration::from_millis(8));
        }
    }
    t.on_canvas_pointer(p(420.0, y, PointerPhase::Up));
}

/// O traço curto que encosta na ponta direita de A (`x` 370..420, a `y = 92`): o resto de A fica longe.
fn encosta(t: &mut PainterTool, fora: bool) {
    t.on_canvas_pointer(p(370.0, 92.0, PointerPhase::Down));
    for k in 1..=10 {
        t.on_canvas_pointer(p(370.0 + 5.0 * k as f32, 92.0, PointerPhase::Move));
        t.on_tick(8.0);
        drena(t);
        if fora {
            std::thread::sleep(Duration::from_millis(8));
        }
    }
    t.on_canvas_pointer(p(420.0, 92.0, PointerPhase::Up));
}

pub(super) fn por_nome<'a>(b: &'a Biblioteca, n: &str) -> &'a Preset {
    &b.entradas
        .iter()
        .find(|e| e.preset.parametros.name == n)
        .unwrap_or_else(|| panic!("o pincel {n}"))
        .preset
}

pub(super) struct Fotos {
    /// f0 assentado · f1 aplicado · f2 traçado · f3 depois · f4 tudo recomposto.
    pub f: Vec<Vec<u8>>,
    /// O mesmo no `canvas_rgba`, antes da luz da saída.
    pub tela: Vec<Vec<u8>>,
    pub estado: String,
}

fn estado(t: &mut PainterTool) -> String {
    let w = &t.paint.wetpaint;
    let oleo_tela = w.oleo_na_tela;
    let s = w.session.as_ref().expect("sessão");
    let suave_cel = s.planos.suave.iter().filter(|v| **v != 0).count();
    let sombra = s.planos.sombra.iter().filter(|v| **v != 0).count();
    let grao = s.planos.grao.iter().filter(|v| **v != 0).count();
    format!(
        "oleoTela={oleo_tela} planoSuave={suave_cel} sombra={sombra} grao={grao} visivel={} ramo={:?}",
        t.impasto_visible(),
        t.last_drain_branch
    )
}

pub(super) fn corrida(lib: &Biblioteca, c: &Cfg, b: Option<&Preset>) -> Fotos {
    forca_no_teste(Some(c.fora));
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    if !c.fora {
        t.set_wet_relogio_fixo(true);
    }
    if c.razao > 1.0 {
        t.set_wet_grid_ratio(c.razao);
    }
    if c.papel {
        t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperCold.to_u8());
        t.set_substrate_depth(0.7);
    }
    t.preset_aplicar(por_nome(lib, &c.a)).expect("preset A");
    t.sonda_raio(34.0);
    t.set_brush_color_srgb8([120, 120, 125]);
    if let Some(v) = std::env::var("PH2D_SONDA_SE")
        .ok()
        .and_then(|v| v.parse().ok())
    {
        t.set_wet_smooth_edges(v);
    }
    traco(&mut t, 70.0, c.fora);
    tiques(&mut t, c.espera, c.fora);
    if c.seco {
        t.wetpaint_fast_dry();
        tiques(&mut t, 20, c.fora);
    }
    let mut f = vec![ecra(&mut t)];
    let mut tela = vec![t.canvas_rgba.to_vec()];
    if let Some(b) = b {
        t.preset_aplicar(b).expect("preset B");
        t.sonda_raio(30.0);
        t.set_brush_color_srgb8([18, 18, 20]);
    }
    tiques(&mut t, 4, c.fora);
    f.push(ecra(&mut t));
    tela.push(t.canvas_rgba.to_vec());
    if b.is_some() {
        if c.encosta {
            encosta(&mut t, c.fora);
            t.on_tick(8.0);
        }
        traco(&mut t, 270.0, c.fora);
    } else {
        for _ in 0..(if c.encosta { 80 } else { 40 }) {
            t.on_tick(8.0);
            drena(&mut t);
            if c.fora {
                std::thread::sleep(Duration::from_millis(8));
            }
        }
    }
    tiques(&mut t, 2, c.fora);
    f.push(ecra(&mut t));
    tela.push(t.canvas_rgba.to_vec());
    tiques(&mut t, c.depois, c.fora);
    f.push(ecra(&mut t));
    tela.push(t.canvas_rgba.to_vec());
    let est = estado(&mut t);
    t.wet_recomposite_full();
    tiques(&mut t, 4, c.fora);
    f.push(ecra(&mut t));
    tela.push(t.canvas_rgba.to_vec());
    forca_no_teste(None);
    Fotos {
        f,
        tela,
        estado: est,
    }
}

/// Píxeis da região que diferem e o Δ máximo; `y0..y1` são linhas.
pub(super) fn dif(a: &[u8], b: &[u8], y0: usize, y1: usize) -> (usize, u8) {
    let (mut n, mut m) = (0, 0u8);
    let (a, b) = (&a[y0 * W * 4..y1 * W * 4], &b[y0 * W * 4..y1 * W * 4]);
    for (i, (x, y)) in a
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .enumerate()
    {
        if i % W < XMAX && x != y {
            n += 1;
            for k in 0..4 {
                m = m.max(x[k].abs_diff(y[k]));
            }
        }
    }
    (n, m)
}

pub(super) fn grava_ppm(d: &str, nome: &str, f: &[u8]) {
    let mut v = format!("P6\n{W} {H}\n255\n").into_bytes();
    for px in f.as_chunks::<4>().0 {
        // sobre branco
        let a = u32::from(px[3]);
        for &c in &px[..3] {
            v.push(((u32::from(c) * a + 255 * (255 - a)) / 255) as u8);
        }
    }
    std::fs::create_dir_all(d).unwrap();
    std::fs::write(format!("{d}/{nome}.ppm"), v).unwrap();
}

fn flags(lib: &Biblioteca, b: &Preset) -> String {
    let _ = lib;
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; 64 * 64 * 4], 64, 64);
    t.set_paint_tool_mode("wetpaint");
    t.preset_aplicar(b).expect("preset");
    let w = &t.paint.wetpaint;
    format!(
        "oil={} glaze={} km={} tex={} se={:.2} ed={:.2} tilt={} modo={:?}",
        u8::from(w.oleo_real.is_some()),
        u8::from(w.km_glaze),
        u8::from(w.km_mixing),
        u8::from(w.paper_visual),
        w.knobs.get(Knob::SmoothEdges),
        w.knobs.get(Knob::EdgeDarkening),
        u8::from(w.tilt_on),
        b.parametros.paint_mode.as_deref().unwrap_or("-"),
    )
}

#[test]
#[ignore = "sonda: corre à mão"]
fn sonda_bordas_traco_longe() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let matriz =
        std::env::var("PH2D_SONDA_MATRIZ").unwrap_or_else(|_| "Wet Liner:1:0:1".to_owned());
    let so = std::env::var("PH2D_SONDA_SO").ok();
    let fotos = std::env::var("PH2D_SONDA_FOTOS").ok();
    let bs: Vec<&Preset> = lib
        .entradas
        .iter()
        .filter(|e| {
            e.preset.parametros.media == "WetPaint" || std::env::var("PH2D_SONDA_TODOS").is_ok()
        })
        .map(|e| &e.preset)
        .filter(|p| {
            so.as_ref()
                .is_none_or(|s| s.split(',').any(|n| n == p.parametros.name))
        })
        .collect();
    let fios: usize = std::env::var("PH2D_SONDA_FIOS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    eprintln!("B: {}", bs.len());
    for caso in matriz.split(',') {
        let v: Vec<&str> = caso.split(':').collect();
        let c = Cfg {
            a: v[0].to_owned(),
            papel: v[1] == "1",
            fora: v[2] == "1",
            seco: v[3] == "1",
            encosta: v.get(4) == Some(&"1"),
            razao: v.get(5).map_or(1.0, |x| x.parse().unwrap()),
            espera: v.get(6).map_or(120, |x| x.parse().unwrap()),
            depois: v.get(7).map_or(240, |x| x.parse().unwrap()),
        };
        let ctl = corrida(&lib, &c, None);
        let ctl2 = corrida(&lib, &c, None);
        let ruido: Vec<_> = (0..5)
            .map(|i| dif(&ctl.f[i], &ctl2.f[i], 0, REGIAO))
            .collect();
        println!(
            "=== A={} papel={} fora={} seco={} encosta={} razao={} espera={} depois={} | ruído do controle f0..f4 {:?} | deriva controle f4-f0 {:?} | {}",
            c.a,
            c.papel,
            c.fora,
            c.seco,
            c.encosta,
            c.razao,
            c.espera,
            c.depois,
            ruido,
            dif(&ctl.f[0], &ctl.f[4], 0, REGIAO),
            ctl.estado
        );
        let prox = AtomicUsize::new(0);
        let linhas = Mutex::new(Vec::new());
        std::thread::scope(|sc| {
            for _ in 0..fios {
                sc.spawn(|| {
                    loop {
                        let i = prox.fetch_add(1, Ordering::SeqCst);
                        let Some(b) = bs.get(i) else { break };
                        let nome = b.parametros.name.clone();
                        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            corrida(&lib, &c, Some(b))
                        }));
                        let l = match r {
                            Ok(o) => {
                                let d: Vec<_> = (0..5)
                                    .map(|k| dif(&o.f[k], &ctl.f[k], 0, REGIAO))
                                    .collect();
                                let dt = dif(&o.tela[3], &ctl.tela[3], 0, REGIAO);
                                if let Some(dir) = &fotos {
                                    let base = format!(
                                        "{}_{}_{}",
                                        nome.replace(' ', "_"),
                                        c.a.replace(' ', "_"),
                                        u8::from(c.fora)
                                    );
                                    for k in [0usize, 3, 4] {
                                        grava_ppm(dir, &format!("{base}_f{k}"), &o.f[k]);
                                    }
                                    grava_ppm(dir, &format!("{base}_ctl3"), &ctl.f[3]);
                                }
                                format!(
                                    "{:>6}|{:<22} {} | ECRA aplica {:?} traça {:?} depois {:?} recomp {:?} | TELA depois {:?} | {}",
                                    d[3].0,
                                    nome,
                                    flags(&lib, b),
                                    d[1],
                                    d[2],
                                    d[3],
                                    d[4],
                                    dt,
                                    o.estado
                                )
                            }
                            Err(_) => format!("PANIC |{nome}"),
                        };
                        linhas.lock().unwrap().push(l);
                    }
                });
            }
        });
        let mut l = linhas.into_inner().unwrap();
        l.sort();
        for x in &l {
            println!("{x}");
        }
    }
}

/// A varredura inversa: A por todos os presets, B fixo (`PH2D_SONDA_B`, Flat Bristle), longe.
#[test]
#[ignore = "sonda: corre à mão"]
fn sonda_bordas_varre_a() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let nome_b = std::env::var("PH2D_SONDA_B").unwrap_or_else(|_| "Flat Bristle".to_owned());
    let fora = std::env::var("PH2D_SONDA_FORA").is_ok_and(|v| v == "1");
    let seco = !std::env::var("PH2D_SONDA_MOLHADO").is_ok_and(|v| v == "1");
    let fios: usize = std::env::var("PH2D_SONDA_FIOS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let b = por_nome(&lib, &nome_b);
    let fotos = std::env::var("PH2D_SONDA_FOTOS").ok();
    let so = std::env::var("PH2D_SONDA_SO").ok();
    let bs: Vec<String> = lib
        .entradas
        .iter()
        .filter(|e| e.preset.parametros.media == "WetPaint")
        .map(|e| e.preset.parametros.name.clone())
        .filter(|n| so.as_ref().is_none_or(|s| s.split(',').any(|x| x == n)))
        .collect();
    let prox = AtomicUsize::new(0);
    let linhas = Mutex::new(Vec::new());
    std::thread::scope(|sc| {
        for _ in 0..fios {
            sc.spawn(|| {
                loop {
                    let i = prox.fetch_add(1, Ordering::SeqCst);
                    let Some(a) = bs.get(i) else { break };
                    let c = Cfg {
                        a: a.clone(),
                        papel: true,
                        fora,
                        seco,
                        encosta: false,
                        razao: 1.0,
                        espera: 120,
                        depois: 240,
                    };
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let ctl = corrida(&lib, &c, None);
                        let o = corrida(&lib, &c, Some(b));
                        (ctl, o)
                    }));
                    let l = match r {
                        Ok((ctl, o)) => {
                            let d: Vec<_> =
                                (0..5).map(|k| dif(&o.f[k], &ctl.f[k], 0, REGIAO)).collect();
                            let dt = dif(&o.tela[3], &ctl.tela[3], 0, REGIAO);
                            if let Some(dir) = &fotos {
                                let base = a.replace(' ', "_");
                                grava_ppm(dir, &format!("{base}_antes"), &ctl.f[3]);
                                grava_ppm(dir, &format!("{base}_depois"), &o.f[3]);
                            }
                            format!(
                                "{:>6}|A={:<22} {} | aplica {:?} traça {:?} depois {:?} recomp {:?} | TELA {:?}",
                                d[3].0,
                                a,
                                flags(&lib, por_nome(&lib, a)),
                                d[1],
                                d[2],
                                d[3],
                                d[4],
                                dt
                            )
                        }
                        Err(_) => format!("PANIC |{a}"),
                    };
                    linhas.lock().unwrap().push(l);
                }
            });
        }
    });
    let mut l = linhas.into_inner().unwrap();
    l.sort();
    for x in &l {
        println!("{x}");
    }
}

/// Qual knob do preset B muda A molhado: sem pincelada, troca-se um knob de cada vez (e o preset todo menos ele).
#[test]
#[ignore = "sonda: corre à mão"]
fn sonda_bordas_qual_knob() {
    use ph2d_wet_paint::tuning::KNOB_DEFS;
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let a_n = std::env::var("PH2D_SONDA_A").unwrap_or_else(|_| "Wet Liner".into());
    let b_n = std::env::var("PH2D_SONDA_B").unwrap_or_else(|_| "Wet Dry Brush".into());
    let espera: usize = std::env::var("PH2D_SONDA_ESPERA")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);
    let c = Cfg {
        a: a_n,
        papel: true,
        fora: false,
        seco: false,
        encosta: false,
        razao: 1.0,
        espera,
        depois: 600,
    };
    let b = por_nome(&lib, &b_n);
    let ka = {
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; 64 * 64 * 4], 64, 64);
        t.set_paint_tool_mode("wetpaint");
        t.preset_aplicar(por_nome(&lib, &c.a)).unwrap();
        t.paint.wetpaint.knobs
    };
    let kb = {
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; 64 * 64 * 4], 64, 64);
        t.set_paint_tool_mode("wetpaint");
        t.preset_aplicar(b).unwrap();
        t.paint.wetpaint.knobs
    };
    // Sem pincelada: `troca` escreve nos knobs vivos, depois de A assentar, e os tiques correm.
    let corre = |troca: &(dyn Fn(&mut PainterTool) + Sync)| -> Vec<u8> {
        forca_no_teste(Some(false));
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperCold.to_u8());
        t.set_substrate_depth(0.7);
        t.preset_aplicar(por_nome(&lib, &c.a)).expect("preset A");
        t.sonda_raio(34.0);
        t.set_brush_color_srgb8([120, 120, 125]);
        traco(&mut t, 70.0, false);
        tiques(&mut t, c.espera, false);
        troca(&mut t);
        tiques(&mut t, c.depois, false);
        forca_no_teste(None);
        ecra(&mut t)
    };
    let ctl = corre(&|_| {});
    let todo = corre(&|t| {
        t.preset_aplicar(b).unwrap();
    });
    println!(
        "A={} B={} espera={espera}: só o preset, sem traço {:?}",
        c.a,
        b_n,
        dif(&ctl, &todo, 0, REGIAO)
    );
    let difs: Vec<_> = KNOB_DEFS
        .iter()
        .filter(|d| ka.get(d.knob) != kb.get(d.knob))
        .collect();
    if std::env::var("PH2D_SONDA_SO_PRESET").is_ok() {
        return;
    }
    let prox = AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..12 {
            sc.spawn(|| {
                while let Some(d) = difs.get(prox.fetch_add(1, Ordering::SeqCst)) {
                    let so = corre(&|t| t.paint.wetpaint.knobs.set(d.knob, kb.get(d.knob)));
                    let sem = corre(&|t| {
                        t.preset_aplicar(b).unwrap();
                        t.paint.wetpaint.knobs.set(d.knob, ka.get(d.knob));
                    });
                    println!(
                        "{:?}: A {:.3} B {:.3} | só ele {:?} | B sem ele {:?}",
                        d.knob,
                        ka.get(d.knob),
                        kb.get(d.knob),
                        dif(&ctl, &so, 0, REGIAO),
                        dif(&ctl, &sem, 0, REGIAO)
                    );
                }
            });
        }
    });
}
