//! Gates 2 e 6 das macros (plano `docs/Macros/01_plano_onda1.md` §5): a SEMENTE e o `dt` irregular
//! reproduzem-se ao byte, e o mesmo traço numa tela 2× cai no mesmo lugar relativo.
//!
//! O relógio da água aqui é o FIXO (a dívida de `dt` vira passos síncronos): o que se mede é o pincel e
//! o tique, sem o worker — esse é o gate 1 (`macro_agua.rs`).

use super::macro_comum::{Gravador, cp, texels_diferentes, toca};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool, Tool};
use ph2d_macro::MacroDoc;
use ph2d_macro::rec::{self, Record};
use ph2d_tool_painter::macro_track::Passo;
use ph2d_tool_painter::macro_track::tocar::TocadorDoPainter;
use ph2d_tool_painter::{PaintMedia, PainterTool};

/// Tiques de quadro IRREGULARES (ms): o Airbrush emite pelo tempo, e a água fixa dá passos pela dívida.
const DTS: [f32; 7] = [7.0, 31.0, 16.0, 45.0, 3.0, 22.0, 12.5];

fn ferramenta(n: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (n * n * 4) as usize], n, n);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(6.0);
    t
}

/// Um traço de `de` a `ate` (fracções da tela), um evento e um tique irregular por quadro.
fn traco(t: &mut PainterTool, g: &mut Gravador, de: [f32; 2], ate: [f32; 2], k0: usize) {
    traco_com_cauda(t, g, de, ate, k0, 10);
}

/// [`traco`] com `cauda` quadros depois do `Up` (a água só corre depois dele).
fn traco_com_cauda(
    t: &mut PainterTool,
    g: &mut Gravador,
    de: [f32; 2],
    ate: [f32; 2],
    k0: usize,
    cauda: usize,
) {
    let (w, h) = t.source_size();
    let p = |f: f32| {
        [
            w as f32 * (de[0] + (ate[0] - de[0]) * f),
            h as f32 * (de[1] + (ate[1] - de[1]) * f),
        ]
    };
    const AMOSTRAS: usize = 24;
    for i in 0..=AMOSTRAS {
        let fase = match i {
            0 => PointerPhase::Down,
            AMOSTRAS => PointerPhase::Up,
            _ => PointerPhase::Move,
        };
        t.on_canvas_pointer(cp(p(i as f32 / AMOSTRAS as f32), fase));
        let _ = t.take_preview_arc();
        t.on_tick(DTS[(k0 + i) % DTS.len()]);
        g.quadro(t);
    }
    for i in 0..cauda {
        t.on_tick(DTS[(k0 + i) % DTS.len()]);
        g.quadro(t);
    }
}

/// O bake depois de +40 passos de água (a parte que a tela ainda não mostra).
fn fim(t: &mut PainterTool) -> Vec<u8> {
    for _ in 0..40 {
        t.on_tick(1000.0 / 40.0);
    }
    t.run_full().0
}

/// Grava o gate 2: devolve o tool que gravou (a verdade) e a macro.
fn grava_semente_e_dt() -> (PainterTool, MacroDoc) {
    let mut t = ferramenta(192);
    // O pincel com acaso (a semente decide) e o método que emite pelo TEMPO (o `dt` decide).
    t.set_brush_jitter_norm(0.7);
    t.set_brush_jitter_scale(0.6);
    t.set_brush_stroke_method(1);
    // Um traço ANTES de armar: a semente avança, e o estado inicial tem de levar a de AGORA (a de fábrica
    // de um tool novo esconderia uma semente que não viaja).
    let (w, h) = t.source_size();
    for (i, fase) in [PointerPhase::Down, PointerPhase::Move, PointerPhase::Up]
        .into_iter()
        .enumerate()
    {
        t.on_canvas_pointer(cp(
            [w as f32 * (0.1 + 0.1 * i as f32), h as f32 * 0.9],
            fase,
        ));
        t.on_tick(16.0);
    }
    let mut g = Gravador::arma(&mut t);
    t.set_brush_color_srgb8([200, 40, 30]);
    traco(&mut t, &mut g, [0.15, 0.3], [0.85, 0.35], 0);
    t.set_brush_color_srgb8([30, 70, 210]);
    traco(&mut t, &mut g, [0.2, 0.75], [0.8, 0.25], 3);
    let doc = g.para(&mut t);
    (t, doc)
}

/// O estado inicial com a `traco.seed` mexida (o JSON das configurações é a etiqueta `2`).
fn com_outra_semente(inicial: &[u8]) -> Vec<u8> {
    let mut r = Record::new();
    for (tag, corpo) in rec::fields(inicial).expect("registo") {
        if tag == 2 {
            let mut v: serde_json::Value =
                serde_json::from_str(rec::as_str(corpo).expect("utf8")).expect("json");
            let s = v["traco"]["seed"].as_u64().expect("a semente viaja");
            v["traco"]["seed"] = serde_json::json!(s.wrapping_add(7919));
            r.str(2, &v.to_string());
        } else {
            r.bytes(tag, corpo);
        }
    }
    r.finish()
}

#[test]
fn a_semente_e_os_tiques_irregulares_reproduzem_se_ao_byte() {
    let (mut original, doc) = grava_semente_e_dt();
    let mut toc = toca(&doc).expect("a reprodução não diverge");
    let (a, b) = (fim(&mut original), fim(toc.tool_mut()));
    assert_eq!(
        texels_diferentes(&a, &b),
        0,
        "a reprodução difere do tool que gravou"
    );
}

/// Os controlos do gate 2: a fixture DEPENDE do que se grava — a semente mexida e um tique a menos
/// mudam a tela.
#[test]
fn sem_a_semente_ou_sem_um_tique_a_reproducao_muda() {
    let (_, doc) = grava_semente_e_dt();
    let verdade = fim(toca(&doc).expect("toca").tool_mut());

    let mut semente = doc.clone();
    semente.tracks[0].initial = com_outra_semente(&doc.tracks[0].initial);
    let d = texels_diferentes(&fim(toca(&semente).expect("toca").tool_mut()), &verdade);
    assert!(
        d > 0,
        "a semente mexida deu a mesma tela: a fixture não depende da semente"
    );

    let mut sem_tique = doc.clone();
    let i = sem_tique.tracks[0]
        .events
        .iter()
        .position(|e| matches!(Passo::descodifica(&e.payload), Ok(Passo::Tique(dt)) if dt > 30.0))
        .expect("há um tique longo");
    sem_tique.tracks[0].events.remove(i);
    let d = texels_diferentes(&fim(toca(&sem_tique).expect("toca").tool_mut()), &verdade);
    assert!(
        d > 0,
        "um tique a menos deu a mesma tela: a fixture não depende do dt"
    );
}

/// O centro de massa da tinta (o quanto cada pixel se afasta do branco), em fracções da tela.
fn centro(px: &[u8], n: u32) -> [f64; 2] {
    let (mut sx, mut sy, mut sw) = (0.0, 0.0, 0.0);
    for (i, p) in px.chunks(4).enumerate() {
        let w = f64::from(255 - p[0].min(p[1]).min(p[2]));
        let (x, y) = ((i as u32 % n) as f64 + 0.5, (i as u32 / n) as f64 + 0.5);
        sx += w * x;
        sy += w * y;
        sw += w;
    }
    assert!(sw > 0.0, "nada pintado");
    [sx / sw / f64::from(n), sy / sw / f64::from(n)]
}

#[test]
fn o_mesmo_traco_numa_tela_2x_cai_no_mesmo_lugar_relativo() {
    let n = 96;
    let mut t = ferramenta(n);
    let mut g = Gravador::arma(&mut t);
    t.set_brush_color_srgb8([20, 20, 120]);
    // Sem cauda: mede-se onde o traço POUSA. A água escorre depois em células de tamanho absoluto — na
    // tela 1× desce mais, relativamente (medido: 1,13 px no centro de massa depois de 10 + 40 passos).
    traco_com_cauda(&mut t, &mut g, [0.25, 0.3], [0.7, 0.45], 0, 0);
    let doc = g.para(&mut t);
    let um = centro(&t.run_full().0, n);

    let tr = &doc.tracks[0];
    let toc = TocadorDoPainter::sobre_tela(
        &tr.initial,
        tr.events.iter().map(|e| e.payload.as_slice()),
        vec![255u8; (4 * n * n * 4) as usize],
        2 * n,
        2 * n,
    )
    .expect("toca sobre a tela 2×");
    let mut toc = toca_com_doc(&doc, toc);
    let dois = centro(&toc.tool_mut().run_full().0, 2 * n);
    let erro_px = ((dois[0] - um[0]).hypot(dois[1] - um[1])) * f64::from(n);
    assert!(
        erro_px < 1.0,
        "na tela 2× o traço caiu {erro_px:.2} px (da tela 1×) longe do lugar relativo: {um:?} × {dois:?}"
    );
    // Controlo: sem a escala, o traço cairia na METADE do lugar relativo — a régua distingue os dois.
    let metade = [um[0] / 2.0, um[1] / 2.0];
    let longe = (dois[0] - metade[0]).hypot(dois[1] - metade[1]) * f64::from(n);
    assert!(longe > 10.0, "a régua não distingue a escala da falta dela");
}

fn toca_com_doc(doc: &MacroDoc, toc: TocadorDoPainter) -> TocadorDoPainter {
    super::macro_comum::toca_com(doc, toc).expect("a reprodução não diverge")
}
