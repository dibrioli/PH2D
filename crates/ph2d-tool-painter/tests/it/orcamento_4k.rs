//! **B44.8 (GT-07, O1): o quadro de traço custa pelo TRAÇO, não pela tela** — o mesmo traço (r=40, 600 px,
//! 60 eventos, um dreno a cada 3, 6 ticks depois de soltar) numa tela de 1024² e numa de 3840×2160, nos 4 meios,
//! e os contadores por quadro que a ferramenta já expõe: área reivindicada pelo dreno (`preview_gpu_region`,
//! `None` = a tela inteira), bytes de undo retidos, texels do véu (`mistura_texels_do_veu`, B07.3) e escritas
//! de tela inteira do motor do Wet Paint (B44.9). Lei: cada contador a 4K ≤ 1,02× o de 1024².
//!
//! Os que reprovam HOJE por um mecanismo cuja cura é de outra pegada estão em [`A_ESPERA`] com o número medido,
//! e o portão AFIRMA que continuam a reprovar: quando a cura entra, o portão fica vermelho e pede que a linha
//! saia da lista (isenção afirmada, nunca silenciada). Sem relógio: corre na suíte normal.

use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{PaintMedia, PainterTool};

const TECTO: f64 = 1.02;
const TELAS: [(usize, usize); 2] = [(1024, 1024), (3840, 2160)];
const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::WetPaint,
    PaintMedia::Impasto,
];
const NOMES: [&str; 6] = [
    "área reivindicada (soma)",
    "área reivindicada (máx. por quadro)",
    "bytes de undo",
    "bytes de undo do 2.º traço",
    "texels do véu",
    "marcas de tela inteira",
];

/// `(meio, contador, 1024², 4K)` medidos 14/11 que reprovam a lei e esperam a cura de outra pegada.
// O 1.º traço de Impasto (13 457 984 → 100 407 872, o relevo que nascia gravava a tela) saiu no F1p: 1 528 256
// nas duas telas (`undo_delta_aparece`).
const A_ESPERA: &[(PaintMedia, usize, u64, u64, &str)] = &[];

#[derive(Debug, Clone, Copy, PartialEq)]
struct Contas([u64; 6]);

fn traco((w, h): (usize, usize), meio: PaintMedia) -> Contas {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; w * h * 4], w as u32, h as u32);
    t.set_paint_media(meio);
    t.set_wet_relogio_fixo(true);
    t.sonda_raio(40.0);
    t.set_brush_color_srgb8([20, 60, 200]);
    t.set_compor_por_quadro(true);
    let _ = t.take_preview_dirty();
    let marcas0 = t.wet_marcas_da_tela_inteira();
    let undo0 = t.undo_retained_bytes() as u64;
    let tela = (w * h) as u64;
    let (mut soma, mut maximo) = (0u64, 0u64);
    let mut drena = |t: &mut PainterTool| {
        if t.take_preview_dirty() {
            let a = t
                .preview_gpu_region()
                .map_or(tela, |(_, _, rw, rh)| u64::from(rw) * u64::from(rh));
            soma += a;
            maximo = maximo.max(a);
        }
    };
    let cp = |x: f32, y: f32, phase| CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let mut um_traco = |t: &mut PainterTool, y: f32| {
        t.on_canvas_pointer(cp(200.0, y, PointerPhase::Down));
        for k in 1..=60 {
            t.on_canvas_pointer(cp(200.0 + 10.0 * k as f32, y, PointerPhase::Move));
            if k % 3 == 0 {
                t.on_tick(1000.0 / 60.0);
                drena(t);
            }
        }
        t.on_canvas_pointer(cp(800.0, y, PointerPhase::Up));
        for _ in 0..6 {
            t.on_tick(1000.0 / 60.0);
            drena(t);
        }
    };
    um_traco(&mut t, 500.0);
    let undo1 = t.undo_retained_bytes() as u64;
    um_traco(&mut t, 700.0);
    Contas([
        soma,
        maximo,
        undo1 - undo0,
        t.undo_retained_bytes() as u64 - undo1,
        t.mistura_texels_do_veu(),
        t.wet_marcas_da_tela_inteira() - marcas0,
    ])
}

fn razao(p: u64, g: u64) -> f64 {
    match (p, g) {
        (0, 0) => 1.0,
        (0, _) => f64::INFINITY,
        _ => g as f64 / p as f64,
    }
}

#[test]
fn o_traco_a_4k_custa_o_mesmo_que_a_1024() {
    let mut falhas = Vec::new();
    for meio in MEIOS {
        let [p, g] = TELAS.map(|tela| traco(tela, meio));
        assert!(
            p.0[0] > 0 && p.0[2] > 0,
            "{meio:?}: o traço não pintou nem gravou undo ({p:?}) — régua morta"
        );
        for (i, nome) in NOMES.iter().enumerate() {
            let r = razao(p.0[i], g.0[i]);
            eprintln!(
                "[orcamento-4k] {meio:?} {nome}: 1024² {} · 4K {} · {r:.3}×",
                p.0[i], g.0[i]
            );
            let espera = A_ESPERA.iter().find(|e| e.0 == meio && e.1 == i);
            match (r <= TECTO, espera) {
                (true, None) | (false, Some(_)) => {}
                (true, Some(e)) => falhas.push(format!(
                    "{meio:?} {nome} passou a caber ({r:.3}×; esperado {} → {}, «{}»): tire a linha de A_ESPERA",
                    e.2, e.3, e.4
                )),
                (false, None) => falhas.push(format!(
                    "{meio:?} {nome}: 1024² {} → 4K {} = {r:.3}× > {TECTO}× — o quadro de traço paga pela tela",
                    p.0[i], g.0[i]
                )),
            }
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
