//! B25.8 (missão 4K, N10): quantos pincéis de FÁBRICA sobem um lote à placa — por preset, pela porta do ponteiro.
//!
//! Uma ponte espiã que DECLINA (a CPU pinta o mesmo byte, `a_bridge_that_declines_…`) conta quem lá chega; o predicado
//! estático (`eligible`) diz porquê os outros ficam. A lista sai em `--nocapture`; o número vive em
//! `docs/Painter/missao_4k/medicoes/N10_carimbo.md`.

use super::{DeviceStamp, eligible};
use ph2d_editor_core::Tool as _;
use ph2d_editor_core::tool::{
    CanvasPaintTool as _, CanvasPointer, PointerPhase, RasterEditTool as _,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const LADO: u32 = 256;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um traço reto à mão, `passo` px por evento (um quadro por evento), a 18 px de raio. `None` = a elipse do
/// `figure_tool` (a figura empilhada que os gates da ponte usam): o controlo que TEM de subir.
fn traco(t: &mut crate::tool::PainterTool, passo: Option<f32>) {
    let Some(passo) = passo else {
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
        t.on_canvas_pointer(cp([128.0, 128.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([200.0, 180.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([200.0, 180.0], PointerPhase::Up));
        return;
    };
    let n = (160.0 / passo) as usize;
    t.on_canvas_pointer(cp([40.0, 128.0], PointerPhase::Down));
    for k in 1..=n {
        t.on_canvas_pointer(cp([40.0 + passo * k as f32, 128.0], PointerPhase::Move));
        t.on_tick(16.0);
    }
    t.on_canvas_pointer(cp([200.0, 128.0], PointerPhase::Up));
    t.on_tick(16.0);
}

/// Pinta `traco` com o pincel que `prepara` deixa; devolve (lotes na placa, o `eligible` do pincel, pintou?).
fn conta(
    prepara: impl FnOnce(&mut crate::tool::PainterTool),
    passo: Option<f32>,
) -> (usize, bool, bool) {
    let mut t = crate::tool::PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    prepara(&mut t);
    t.set_brush_size_px(18.0);
    let chamadas = Arc::new(AtomicUsize::new(0));
    let c = Arc::clone(&chamadas);
    let ponte: DeviceStamp = Box::new(move |_| {
        c.fetch_add(1, Ordering::Relaxed);
        None
    });
    t.set_device_stamp(Some(ponte));
    let elegivel = eligible(&t.paint.brush);
    traco(&mut t, passo);
    let pintou = t.canvas_rgba.iter().any(|&b| b != 255);
    (chamadas.load(Ordering::Relaxed), elegivel, pintou)
}

/// Os passos do traço à mão (px por quadro): o rato lento e o rápido.
const PASSOS: [f32; 2] = [4.0, 24.0];

#[test]
#[ignore = "sonda B25.8 (9,7 s): a contagem por preset; corre à mão com --nocapture"]
fn quantos_pinceis_de_fabrica_sobem_a_placa() {
    // O controlo: o redondo de sempre na figura empilhada TEM de subir — sem isto a contagem 0 seria a régua morta.
    let (lotes, elegivel, pintou) = conta(|_| {}, None);
    assert!(elegivel && pintou && lotes > 0, "o controlo: {lotes} lotes");
    let mao: Vec<usize> = PASSOS.iter().map(|&p| conta(|_| {}, Some(p)).0).collect();
    println!(
        "B25.8 controlo (redondo de fábrica): figura {lotes} lotes · à mão {mao:?} (passos {PASSOS:?})"
    );

    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    assert!(bib.erros.is_empty(), "{:?}", bib.erros);
    // Por meio: [presets, elegíveis, sobem na figura, sobem à mão em algum passo].
    let mut por_meio: std::collections::BTreeMap<String, [usize; 4]> = Default::default();
    for e in &bib.entradas {
        let p = &e.preset;
        let aplica = |t: &mut crate::tool::PainterTool| {
            t.set_semente_dos_tracos(0);
            t.set_wet_relogio_fixo(true);
            let falhas = t.preset_aplicar(p).expect("meio conhecido");
            assert!(falhas.is_empty(), "{}: {falhas:?}", p.parametros.name);
        };
        let (fig, elegivel, _) = conta(aplica, None);
        let mao: Vec<usize> = PASSOS.iter().map(|&q| conta(aplica, Some(q)).0).collect();
        let n = por_meio.entry(p.parametros.media.clone()).or_default();
        n[0] += 1;
        n[1] += usize::from(elegivel);
        n[2] += usize::from(fig > 0);
        n[3] += usize::from(mao.iter().any(|&l| l > 0));
        println!(
            "B25.8\t{}\t{}\telegivel={elegivel}\tfigura={fig}\tmao={mao:?}",
            p.parametros.media, p.parametros.name
        );
        // `elegivel` é o do pincel VIVO; o lote sobe com o spec do meio (`authored_spec`), que pode diferir.
        let subiu = fig > 0 || mao.iter().any(|&l| l > 0);
        if subiu && !elegivel {
            println!(
                "B25.8 ⚠ {} sobe com o pincel vivo fora do predicado",
                p.parametros.name
            );
        }
    }
    for (meio, [n, el, fig, mao]) in &por_meio {
        println!(
            "B25.8 {meio}: {n} presets · {el} elegíveis · {fig} sobem na figura · {mao} sobem à mão"
        );
    }
    assert!(!por_meio.is_empty(), "a biblioteca de fábrica veio vazia");
}
