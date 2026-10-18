//! **O lápis DEITADO** (W6, doc [47_medicao](../../../../docs/Painter/NewFeaturesWetPaint/47_medicao_a_caneta_no_pincel.md)
//! §3): o Graphite 8B de fábrica num traço recto à largura da foto do Rebelle, de pé e com a caneta a 55° AO
//! LONGO, ATRAVÉS e na DIAGONAL, contra o 8B do Rebelle nas mesmas inclinações
//! (`docs/Painter/NewFeaturesWetPaint/fixtures/rebelle_tilt/`, blocos A, B e D). A barra da W6 (05 §3): ao longo,
//! **largura ≥ 1,2×** e **textura menor**; o resto no VALE (05 §3) entre o Rebelle e o controlo (a caneta de pé).

use super::midias_de_fabrica::de_fabrica;
use super::regua_midia::{Medida, mede};
use super::sonda_gestos::{AZUL, W};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PainterTool;

/// O raio da cena: o do 8B em `midias_de_fabrica` (a largura da foto, 30 px).
const RAIO: f32 = 15.0;
const H: u32 = 400;
const CY: f32 = 200.0;

/// O traço recto (x 200 → 700, 3 px por evento, pressão 1) com a caneta a `tilt` (graus por eixo, o idioma do
/// Rebelle), 2 s de relógio depois. Devolve a medida (recorte de 240 px de altura) e o comprimento.
fn traco(preset: &ph2d_brush_presets::Preset, tilt_deg: [f32; 2]) -> (Medida, f64, Vec<u8>) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(preset).is_empty());
    t.set_brush_size_px(RAIO);
    t.set_brush_color_srgb8(AZUL);
    let tilt = [tilt_deg[0] / 90.0, tilt_deg[1] / 90.0];
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, CY],
        pressure: 1.0,
        tilt,
        phase,
    };
    t.on_canvas_pointer(cp(200.0, PointerPhase::Down));
    let mut x = 200.0;
    while x < 700.0 {
        x += 3.0;
        t.on_canvas_pointer(cp(x, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(x, PointerPhase::Up));
    for _ in 0..120 {
        t.on_tick(1000.0 / 60.0);
    }
    let (px, w, _) = t.run_full();
    let m = mede(&px, w, [0, 80, W, 240]);
    let tinta = |x: u32| (80..320).any(|y| px[((y * W + x) * 4) as usize] < 230);
    let a = (0..W).find(|&x| tinta(x)).unwrap_or(0);
    let b = (0..W).rfind(|&x| tinta(x)).unwrap_or(0);
    (m, f64::from(b - a + 1), px)
}

/// `n` do lado do Rebelle `r` do ponto médio entre ele e o controlo `c` (o vale, 05 §3).
fn no_vale(n: f64, r: f64, c: f64) -> bool {
    (n - r).abs() < 0.5 * (c - r).abs()
}

/// O Rebelle, bloco A/B/D (`fixtures/rebelle_tilt/README.md`): de pé 29 px · 8,5 % · 533 px; ao longo a 55°
/// 36 · 5,5 · 612; através a 55° 100 · 8,6 · 529; na diagonal (39, 39) 75 px.
const REBELLE_DE_PE: (f64, f64, f64) = (29.0, 8.5, 533.0);
const REBELLE_AO_LONGO: (f64, f64, f64) = (36.0, 5.5, 612.0);
const REBELLE_ATRAVES: (f64, f64) = (100.0, 8.6);
const REBELLE_DIAGONAL: f64 = 75.0;

#[test]
fn o_lapis_deitado_alarga_e_alisa_como_o_do_rebelle() {
    let p = &de_fabrica("Graphite 8B");
    let [
        (pe, cpe, _),
        (longo, clongo, _),
        (atraves, _, _),
        (diag, _, _),
    ] = std::thread::scope(|s| {
        [[0.0, 0.0], [55.0, 0.0], [0.0, 55.0], [39.0, 39.0]]
            .map(|t| s.spawn(move || traco(p, t)))
            .map(|h| h.join().unwrap())
    });
    println!(
        "[deitada] de pé {pe:?} {cpe} · ao longo {longo:?} {clongo} · através {atraves:?} · diagonal {diag:?}"
    );
    let (lr, tr, cr) = REBELLE_DE_PE;
    let mut falhas = Vec::new();
    let mut confere = |nome: &str, ok: bool, n: f64, r: f64| {
        if !ok {
            falhas.push(format!("{nome}: {n:.3} (Rebelle {r:.3})"));
        }
    };
    // A barra da W6: ao longo, ≥ 1,2× e mais liso; e no vale do Rebelle.
    let larg = longo.largura / pe.largura;
    let rl = REBELLE_AO_LONGO.0 / lr;
    confere(
        "largura ao longo ÷ de pé",
        larg >= 1.2 && no_vale(larg, rl, 1.0),
        larg,
        rl,
    );
    let tex = longo.textura / pe.textura;
    let rt = REBELLE_AO_LONGO.1 / tr;
    confere(
        "textura ao longo ÷ de pé",
        tex < 1.0 && no_vale(tex, rt, 1.0),
        tex,
        rt,
    );
    let comp = clongo - cpe;
    let rc = REBELLE_AO_LONGO.2 - cr;
    confere(
        "comprimento a mais ao longo (px)",
        no_vale(comp, rc, 0.0),
        comp,
        rc,
    );
    // Através: o lado inteiro do grafite, o grão de sempre.
    let lat = atraves.largura / pe.largura;
    let ra = REBELLE_ATRAVES.0 / lr;
    confere("largura através ÷ de pé", no_vale(lat, ra, 1.0), lat, ra);
    let tat = atraves.textura / pe.textura;
    confere(
        "textura através ÷ de pé",
        (tat - REBELLE_ATRAVES.1 / tr).abs() < 0.15,
        tat,
        REBELLE_ATRAVES.1 / tr,
    );
    let ldi = diag.largura / pe.largura;
    let rd = REBELLE_DIAGONAL / lr;
    confere(
        "largura na diagonal ÷ de pé",
        no_vale(ldi, rd, 1.0),
        ldi,
        rd,
    );
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// ⭐ **O rato, ao byte**: o 8B de fábrica com o rato (`tilt [0, 0]`) pinta o traço do mesmo pincel SEM as leis da
/// caneta (os knobs dela no neutro). E a caneta abaixo do início (30°) também.
#[test]
fn o_rato_e_a_caneta_de_pe_pintam_o_traco_de_antes() {
    let p = de_fabrica("Graphite 8B");
    let mut neutro = p.clone();
    for k in [
        "dabRotation",
        "tiltWidth",
        "tiltLength",
        "tiltSmooth",
        "tiltLag",
        "tiltStart",
        "tiltFull",
    ] {
        neutro.parametros.engine.remove(k);
    }
    let antes = traco(&neutro, [0.0, 0.0]).2;
    assert!(antes == traco(&p, [0.0, 0.0]).2, "o rato mudou");
    assert!(antes == traco(&p, [30.0, 0.0]).2, "a caneta de pé mudou");
}

/// **SONDA (W6): as razões do lápis deitado com knobs trocados** — `PH2D_SONDA_KNOBS="nome:k=v,k=v|…"` (chaves
/// do mapa `engine` do 8B de fábrica; a variante vazia é a de fábrica): de pé, ao longo, através e diagonal,
/// as mesmas razões de [`o_lapis_deitado_alarga_e_alisa_como_o_do_rebelle`].
#[test]
#[ignore = "sonda: as razões do lápis deitado por variante (W6); corre-se à mão"]
fn sonda_o_lapis_deitado_por_variante() {
    let variantes = std::env::var("PH2D_SONDA_KNOBS").unwrap_or_else(|_| "fabrica:".into());
    let base = de_fabrica("Graphite 8B");
    std::thread::scope(|s| {
        let hs: Vec<_> = variantes
            .split('|')
            .map(|v| {
                let (nome, kv) = v.split_once(':').unwrap_or((v, ""));
                let mut p = base.clone();
                for x in kv.split(',').filter(|x| !x.is_empty()) {
                    let (k, v) = x.split_once('=').unwrap();
                    p.parametros.engine.insert(k.to_owned(), v.parse().unwrap());
                }
                let hs = [[0.0, 0.0], [55.0, 0.0], [0.0, 55.0], [39.0, 39.0]].map(|t| {
                    let p = p.clone();
                    s.spawn(move || traco(&p, t))
                });
                (nome.to_owned(), hs)
            })
            .collect();
        for (nome, hs) in hs {
            let [(pe, cpe, _), (lo, clo, _), (at, _, _), (di, _, _)] =
                hs.map(|h| h.join().unwrap());
            println!(
                "[deitada] {nome}: ao longo {:.3} (R {:.3}) tex {:.3} (R {:.3}) +{:.0} px (R {:.0}) · através {:.3} (R {:.3}) tex {:.3} · diagonal {:.3} (R {:.3})",
                lo.largura / pe.largura,
                REBELLE_AO_LONGO.0 / REBELLE_DE_PE.0,
                lo.textura / pe.textura,
                REBELLE_AO_LONGO.1 / REBELLE_DE_PE.1,
                clo - cpe,
                REBELLE_AO_LONGO.2 - REBELLE_DE_PE.2,
                at.largura / pe.largura,
                REBELLE_ATRAVES.0 / REBELLE_DE_PE.0,
                at.textura / pe.textura,
                di.largura / pe.largura,
                REBELLE_DIAGONAL / REBELLE_DE_PE.0
            );
        }
    });
}
