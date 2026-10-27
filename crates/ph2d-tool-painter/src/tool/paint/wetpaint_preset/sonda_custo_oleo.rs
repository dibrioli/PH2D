//! **O custo de PINTAR com o Óleo Real** (W2, 27/10): a cena `gestos` da bancada (`ferramentas/bateria.json`) sem
//! óleo, com óleo ANTES das duas curas de custo (o atalho do papel liso de `wetpaint/oleo_plano.rs` ligado; o
//! campo em série e a sombra da tinta pela conta própria) e DEPOIS delas, intercaladas, no mesmo
//! processo. Imprime, por fase (traços × `simula`), o tempo, as chamadas do composite, as células do dirty do
//! MOTOR e as que o composite renderiza (com o óleo, crescidas pelo alcance), e o tempo do pigmento × o da sombra.
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_custo_oleo -- --ignored --nocapture`
//! `PH2D_SONDA_PRESET` (o `relativo`, por omissão o Filbert) · `PH2D_SONDA_TRACOS` (quantos traços, 8) ·
//! `PH2D_SONDA_SIMULA` (quadros da `simula`, 240) · `PH2D_SONDA_RONDAS` (2).
use crate::PainterTool;
use crate::tool::paint::wetpaint::composite_contagem as contagem;
use crate::tool::paint::wetpaint::{oleo_curas_desligadas, oleo_plano_desligado};
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use serde_json::Value;
use std::time::Instant;

fn raiz() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn env_n(k: &str, d: usize) -> usize {
    std::env::var(k)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(d)
}

fn filbert(b: &Biblioteca, nome: &str) -> ph2d_brush_presets::Preset {
    b.entradas
        .iter()
        .find(|e| e.relativo == nome)
        .unwrap_or_else(|| panic!("preset {nome}"))
        .preset
        .clone()
}

fn cp(p: &Value, phase: PointerPhase) -> CanvasPointer {
    let grau = |i: usize| p.get(i).and_then(Value::as_f64).unwrap_or(0.0) as f32 / 90.0;
    CanvasPointer {
        pos: [p[0].as_f64().unwrap() as f32, p[1].as_f64().unwrap() as f32],
        pressure: p[2].as_f64().unwrap() as f32,
        tilt: [grau(3), grau(4)],
        phase,
    }
}

type Fase = (f64, u64, u64, u64, f64, f64, f64);

/// Por fase (traços, `simula`): `(ms, composites, células do motor, células renderizadas, ms pigmento, ms
/// sombra)`, e a tela final.
fn corre(preset: &ph2d_brush_presets::Preset, oleo: bool, curas: bool) -> ([Fase; 2], Vec<u8>) {
    oleo_curas_desligadas::set(!curas, !curas);
    let bat: Value = serde_json::from_slice(
        &std::fs::read(raiz().join("docs/Painter/Pinceis/ferramentas/bateria.json")).unwrap(),
    )
    .unwrap();
    let [w, h] = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let cena = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "gestos")
        .unwrap();
    let mut t = PainterTool::default();
    t.set_source(vec![240u8; (w * h * 4) as usize], w, h);
    t.set_wet_oleo_real_ligado(oleo);
    t.set_semente_dos_tracos(0);
    t.set_wet_relogio_fixo(true);
    assert!(t.preset_aplicar(preset).unwrap().is_empty());
    t.set_brush_color_srgb8([23, 101, 179]);
    let (n_tracos, n_simula) = (
        env_n("PH2D_SONDA_TRACOS", 8),
        env_n("PH2D_SONDA_SIMULA", 240),
    );
    let mut fase = [(0.0, 0, 0, 0, 0.0, 0.0, 0.0); 2];
    let fecha = |t0: Instant| {
        let (n, m, r) = contagem::take();
        let (tp, ts) = contagem::take_tempos();
        let tc = contagem::take_campo();
        (t0.elapsed().as_secs_f64() * 1e3, n, m, r, tp, ts, tc)
    };
    let _ = (
        contagem::take(),
        contagem::take_tempos(),
        contagem::take_campo(),
    );
    let t0 = Instant::now();
    let tracos = cena["ops"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["op"] == "traco");
    for op in tracos.take(n_tracos) {
        let pts = op["pts"].as_array().unwrap();
        t.set_brush_size_px(op["size_px"].as_f64().unwrap() as f32 * 0.5);
        t.on_canvas_pointer(cp(&pts[0], PointerPhase::Down));
        for p in &pts[1..] {
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(cp(&pts[pts.len() - 1], PointerPhase::Up));
    }
    fase[0] = fecha(t0);
    let t1 = Instant::now();
    for _ in 0..n_simula {
        t.on_tick(1000.0 / 60.0);
    }
    fase[1] = fecha(t1);
    oleo_curas_desligadas::set(false, false);
    (fase, t.canvas_rgba.to_vec())
}

#[test]
#[ignore = "sonda de custo — corre-se à mão no perfil smoke"]
fn sonda_custo_oleo() {
    let b = Biblioteca::carregar(&raiz().join("assets/brushes"), None);
    let nome = std::env::var("PH2D_SONDA_PRESET")
        .unwrap_or_else(|_| "WetPainter/Oil/Flat/Filbert.ph2d-brush.png".into());
    let preset = filbert(&b, &nome);
    let variantes = [
        ("sem oleo", false, false),
        ("oleo antes", true, false),
        ("oleo depois", true, true),
    ];
    let mut melhor = [[f64::INFINITY; 2]; 3];
    let mut telas: Vec<Option<Vec<u8>>> = vec![None; 3];
    for ronda in 0..env_n("PH2D_SONDA_RONDAS", 2) {
        for j in 0..3 {
            let v = (j + ronda) % 3;
            let (nome_v, oleo, atalho) = variantes[v];
            let (f, tela) = corre(&preset, oleo, atalho);
            telas[v] = Some(tela);
            let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
            for (k, (ms, n, m, r, tp, ts, tc)) in f.iter().enumerate() {
                let nomef = ["tracos", "simula"][k];
                melhor[v][k] = melhor[v][k].min(*ms);
                let n = (*n).max(1) as f64;
                println!(
                    "[sonda] ronda {ronda} {nome_v:15} {nomef}: {ms:8.0} ms · {n} composites · motor {:.0} · render {:.0} \
                     cél/composite · campo {tc:.0} ms · pigmento {tp:.0} ms · sombra {ts:.0} ms · load {}",
                    *m as f64 / n,
                    *r as f64 / n,
                    load.trim()
                );
            }
        }
    }
    for (v, (nome_v, ..)) in variantes.iter().enumerate() {
        println!(
            "[sonda] MÍNIMO {nome_v:15}: tracos {:.0} ms · simula {:.0} ms",
            melhor[v][0], melhor[v][1]
        );
    }
    println!(
        "[sonda] tela depois == antes das curas: {}",
        telas[1] == telas[2]
    );
}

/// **O papel liso da sombra é bit a bit a conta por célula**: traços que se cruzam, a água a correr, um traço por
/// cima da tinta parada e o Apply — a tela de cada passo igual com e sem o atalho de `oleo_plano`.
#[test]
fn o_papel_liso_da_sombra_e_bit_identico_a_conta_por_celula() {
    let b = Biblioteca::carregar(&raiz().join("assets/brushes"), None);
    let preset = filbert(&b, "WetPainter/Oil/Flat/Filbert.ph2d-brush.png");
    let pinta = |atalho: bool| {
        oleo_plano_desligado::set(!atalho);
        let (w, h) = (240u32, 160u32);
        let mut t = PainterTool::default();
        t.set_source(vec![240u8; (w * h * 4) as usize], w, h);
        t.set_wet_oleo_real_ligado(true);
        t.set_semente_dos_tracos(0);
        t.set_wet_relogio_fixo(true);
        assert!(t.preset_aplicar(&preset).unwrap().is_empty());
        t.set_brush_size_px(14.0);
        let mut fotos = Vec::new();
        let traco = |t: &mut PainterTool, a: [f32; 2], b: [f32; 2], cor: [u8; 3]| {
            t.set_brush_color_srgb8(cor);
            let p = |k: f32, phase| CanvasPointer {
                pos: [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k],
                pressure: 0.8,
                tilt: [0.0, 0.0],
                phase,
            };
            t.on_canvas_pointer(p(0.0, PointerPhase::Down));
            for i in 1..=40 {
                t.on_canvas_pointer(p(i as f32 / 40.0, PointerPhase::Move));
                t.on_tick(8.0);
            }
            t.on_canvas_pointer(p(1.0, PointerPhase::Up));
        };
        // Um traço a tocar a borda da grade (a caixa da óptica presa a ela) e dois que se cruzam.
        traco(&mut t, [2.0, 40.0], [220.0, 60.0], [200, 30, 30]);
        traco(&mut t, [60.0, 150.0], [120.0, 10.0], [30, 60, 200]);
        fotos.push(t.canvas_rgba.to_vec());
        for _ in 0..20 {
            t.on_tick(1000.0 / 60.0);
        }
        fotos.push(t.canvas_rgba.to_vec());
        traco(&mut t, [10.0, 100.0], [230.0, 90.0], [240, 220, 40]);
        for _ in 0..10 {
            t.on_tick(1000.0 / 60.0);
        }
        fotos.push(t.canvas_rgba.to_vec());
        fotos.push(t.run_full().0);
        oleo_plano_desligado::set(false);
        fotos
    };
    let (sem, com) = (pinta(false), pinta(true));
    for (k, (a, b)) in sem.iter().zip(&com).enumerate() {
        let difs = a.iter().zip(b).filter(|(x, y)| x != y).count();
        assert_eq!(
            difs, 0,
            "foto {k}: {difs} bytes diferentes entre o atalho e a conta por célula"
        );
    }
    assert!(sem[0].iter().any(|&v| v != 240), "a cena não pintou nada");
}

/// A cena das curas de custo: dois traços que se cruzam, a água a correr, um traço por cima e o Apply, com o
/// óleo `o`; `(sombra da tinta pela conta própria, campo em série)` desligam cada cura.
fn pinta_oleo(
    preset: &ph2d_brush_presets::Preset,
    o: ph2d_wet_paint::oleo_real::OleoReal,
    antigo: (bool, bool),
) -> Vec<Vec<u8>> {
    oleo_curas_desligadas::set(antigo.0, antigo.1);
    let (w, h) = (200u32, 140u32);
    let mut t = PainterTool::default();
    t.set_source(vec![240u8; (w * h * 4) as usize], w, h);
    t.set_semente_dos_tracos(0);
    t.set_wet_relogio_fixo(true);
    assert!(t.preset_aplicar(preset).unwrap().is_empty());
    t.set_wet_oleo_real(Some(o));
    t.set_wet_oleo_real_ligado(true);
    t.set_brush_size_px(14.0);
    let traco = |t: &mut PainterTool, a: [f32; 2], b: [f32; 2], cor: [u8; 3]| {
        t.set_brush_color_srgb8(cor);
        let p = |k: f32, phase| CanvasPointer {
            pos: [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k],
            pressure: 0.8,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(p(0.0, PointerPhase::Down));
        for i in 1..=30 {
            t.on_canvas_pointer(p(i as f32 / 30.0, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(p(1.0, PointerPhase::Up));
    };
    let mut fotos = Vec::new();
    traco(&mut t, [2.0, 40.0], [190.0, 60.0], [200, 30, 30]);
    traco(&mut t, [60.0, 130.0], [110.0, 10.0], [30, 60, 200]);
    fotos.push(t.canvas_rgba.to_vec());
    for _ in 0..12 {
        t.on_tick(1000.0 / 60.0);
    }
    fotos.push(t.canvas_rgba.to_vec());
    traco(&mut t, [10.0, 100.0], [190.0, 90.0], [240, 220, 40]);
    fotos.push(t.canvas_rgba.to_vec());
    fotos.push(t.run_full().0);
    oleo_curas_desligadas::set(false, false);
    fotos
}

/// O Clássico, um HDRI e um HDRI com o material do cartão NÃO neutro (metal, verniz, veludo, anisotropia).
fn oleos_das_curas() -> [ph2d_wet_paint::oleo_real::OleoReal; 3] {
    use ph2d_wet_paint::oleo_real::{Ambiente, MaterialCartao, OleoReal};
    [
        OleoReal {
            ambiente: Ambiente::Classico,
            ..OleoReal::default()
        },
        OleoReal {
            ambiente: Ambiente::Interior,
            ..OleoReal::default()
        },
        OleoReal {
            ambiente: Ambiente::Estudio,
            material: MaterialCartao {
                metal: 0.4,
                verniz: 0.6,
                veludo: 0.3,
                subsuperficie: 0.5,
                ..MaterialCartao::NEUTRO
            },
            ..OleoReal::default()
        },
    ]
}

fn compara_curas(antigo: (bool, bool), nome: &str) {
    let b = Biblioteca::carregar(&raiz().join("assets/brushes"), None);
    let preset = filbert(&b, "WetPainter/Oil/Flat/Filbert.ph2d-brush.png");
    for o in oleos_das_curas() {
        let (antes, depois) = (
            pinta_oleo(&preset, o, antigo),
            pinta_oleo(&preset, o, (false, false)),
        );
        for (k, (a, d)) in antes.iter().zip(&depois).enumerate() {
            let difs = a.iter().zip(d).filter(|(x, y)| x != y).count();
            assert_eq!(
                difs, 0,
                "{nome} {:?} foto {k}: {difs} bytes diferentes",
                o.ambiente
            );
        }
        assert!(antes[0].iter().any(|&v| v != 240), "a cena não pintou nada");
    }
}

/// ⭐ **Cura 1 de custo (27/10): a sombra no papel das células pintadas vem da linha da tinta** — a tela de cada
/// passo bit a bit a da conta própria por célula, no Clássico, num HDRI e com material não neutro.
#[test]
fn a_sombra_da_tinta_pelos_termos_e_bit_identica_ao_caminho_antigo() {
    compara_curas((true, false), "sombra");
}

/// ⭐ **Cura 2 de custo (27/10): o campo de altura em faixas pelo rayon** — a tela de cada passo bit a bit a do
/// campo em série, no Clássico, num HDRI e com material não neutro.
#[test]
fn o_campo_do_oleo_em_faixas_e_bit_identico_ao_caminho_antigo() {
    compara_curas((false, true), "campo");
}
