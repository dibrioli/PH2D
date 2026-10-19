//! **SONDA: o TRAÇO RÁPIDO** (auditoria de 18/10, «um traço rápido o FPS cai para 30»): o Oil Bristle de fábrica a
//! 103 px numa tela 1024², um traço de 800 px com `PH2D_SONDA_PASSO` px (25) entre eventos e um tick de 60 Hz a cada
//! `PH2D_SONDA_EVENTOS_POR_QUADRO` eventos (3): o tempo de cada evento (o carimbo + o compositor) e o do quadro.
//! Variantes no MESMO processo (`PH2D_SONDA_KNOBS="nome:k=v,-k|…"`, a vazia = a fábrica), intercaladas, o MÍNIMO de 7.
//! Só usa a API pública estável (corre igual em versões antigas da linha, para a bissecção).

use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

fn preset(nome: &str, kv: &str) -> ph2d_brush_presets::Preset {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let mut p = ph2d_brush_presets::Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == nome)
        .unwrap_or_else(|| panic!("o preset {nome:?}"))
        .preset;
    for x in kv.split(',').filter(|x| !x.is_empty()) {
        if let Some(k) = x.strip_prefix('-') {
            p.parametros.engine.remove(k);
        } else if let Some((k, v)) = x.split_once('=') {
            p.parametros.engine.insert(k.to_owned(), v.parse().unwrap());
        }
    }
    p
}

/// `(evento mediana, evento pior, quadro mediana, quadro pior)` em ms.
pub(super) fn mede(
    p: &ph2d_brush_presets::Preset,
    raio: f32,
    passo: f32,
    por_quadro: usize,
) -> [f64; 4] {
    let mut t = ph2d_tool_painter::PainterTool::default();
    t.set_source(vec![255u8; 1024 * 1024 * 4], 1024, 1024);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_brush_size_px(raio);
    t.set_brush_color_srgb8([30, 90, 200]);
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 500.0 - 0.14 * (x - 100.0)],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(100.0, PointerPhase::Down));
    let (mut ev, mut qd) = (Vec::new(), Vec::new());
    let n = (800.0 / passo) as usize;
    let mut q0 = std::time::Instant::now();
    for k in 1..=n {
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(cp(100.0 + passo * k as f32, PointerPhase::Move));
        ev.push(t0.elapsed().as_secs_f64() * 1e3);
        if k % por_quadro == 0 {
            t.on_tick(1000.0 / 60.0);
            qd.push(q0.elapsed().as_secs_f64() * 1e3);
            q0 = std::time::Instant::now();
        }
    }
    t.on_canvas_pointer(cp(900.0, PointerPhase::Up));
    let m = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        (v[v.len() / 2], v[v.len() - 1])
    };
    let ((a, b), (c, d)) = (m(&mut ev), m(&mut qd));
    [a, b, c, d]
}

#[test]
#[ignore = "sonda: o traço rápido; corre-se à mão (--release, PH2D_RELEASE=\"mede velocidade\")"]
fn sonda_o_traco_rapido() {
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_owned());
    let raio: f32 = env("PH2D_SONDA_RAIO", "51.5").parse().unwrap();
    let passo: f32 = env("PH2D_SONDA_PASSO", "25").parse().unwrap();
    let por_quadro: usize = env("PH2D_SONDA_EVENTOS_POR_QUADRO", "3").parse().unwrap();
    let nome = env("PH2D_SONDA_PRESET", "Oil Bristle");
    let variantes: Vec<(String, ph2d_brush_presets::Preset)> = env("PH2D_SONDA_KNOBS", "fabrica:")
        .split('|')
        .map(|s| {
            let (n, kv) = s.split_once(':').unwrap_or((s, ""));
            (n.to_owned(), preset(&nome, kv))
        })
        .collect();
    let mut melhor = vec![[f64::MAX; 4]; variantes.len()];
    for rodada in 0..7 {
        for j in 0..variantes.len() {
            let i = (rodada + j) % variantes.len();
            let m = mede(&variantes[i].1, raio, passo, por_quadro);
            for (a, b) in melhor[i].iter_mut().zip(m) {
                *a = a.min(b);
            }
        }
    }
    println!(
        "[rapido] load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    for ((n, _), [a, b, c, d]) in variantes.iter().zip(melhor) {
        println!(
            "[rapido] {nome} r {raio} passo {passo} ×{por_quadro}/quadro · {n:<16} evento {a:.3} · pior {b:.3} ms | quadro {c:.3} · pior {d:.3} ms"
        );
    }
}
