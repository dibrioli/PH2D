//! Os gates do Fill no tutorial «fazer» (W13): a caixa — quatro traços, o azul do C&F dentro, um traço
//! por cima — gravada a 256², um aluno no estado inicial.

use super::super::{EnsinoDoPainter, TipoDoCriterio, mostra};
use super::{TOLERANCIA_DA_COR, rgb_parecida};
use crate::PaintMedia;
use crate::macro_track::inicial;
use crate::macro_track::roteiro::{DT_MS, grava_a_caixa_com_um_fill};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, Tool};
use ph2d_macro::Track;

const LADO: u32 = 256;
// LITERAL-COLOR-OK: a cor GRAVADA do Fill da caixa (`roteiro::AZUL_DA_CAIXA`).
const AZUL: [u8; 3] = [40, 110, 210];

fn caixa(meio: PaintMedia) -> Track {
    let (b, _) = grava_a_caixa_com_um_fill(LADO, 0, meio).expect("grava");
    ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0)
}

fn quadro(t: &mut PainterTool) {
    drop(t.take_preview_arc());
    t.on_tick(DT_MS);
}

/// O aluno solta `cor` do C&F em `(u, v)` e carrega no Done do modal — o gesto da shell.
pub(super) fn solta(t: &mut PainterTool, cor: [u8; 3], u: f32, v: f32) {
    t.set_brush_color_srgb8(cor);
    let volta = t.active_paint_mode_id();
    t.begin_colordrop_fill(volta);
    #[expect(clippy::cast_precision_loss, reason = "o lado da tela")]
    let pos = [u * LADO as f32, v * LADO as f32];
    for phase in [PointerPhase::Down, PointerPhase::Up] {
        t.on_canvas_pointer(CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        quadro(t);
    }
    t.handle_panel_event(PanelEvent::Click(
        ph2d_editor_core::ids::PAINTER_FILL_MODAL_DONE,
    ));
    quadro(t);
}

/// ⭐ **A caixa pelo tutorial**, em Digital e em Wet Paint: seis etapas — os quatro traços, o Fill, o
/// traço por cima do azul —, cada uma cumprida só quando feita, e o Ctrl+Z volta o Fill.
#[test]
fn a_caixa_ensina_os_tracos_o_fill_e_o_traco_por_cima() {
    use TipoDoCriterio::{Pincelada as P, Preenchimento as F};
    for meio in [PaintMedia::Digital, PaintMedia::WetPaint] {
        let track = caixa(meio);
        let e = EnsinoDoPainter::medido(&track).expect("ensino");
        let tipos: Vec<TipoDoCriterio> = e.criterios.iter().map(|c| c.regiao.tipo).collect();
        assert_eq!(tipos, [P, P, P, P, F, P], "{meio:?}");
        let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
        for (i, c) in e.criterios.iter().enumerate() {
            assert_eq!(e.por_cumprir(&t), Some(i), "{meio:?}: antes da {i}");
            mostra(&mut t, &track, c.regiao.k).expect("mostra");
            quadro(&mut t);
        }
        assert_eq!(e.por_cumprir(&t), None, "{meio:?}: a caixa feita");
        assert!(
            t.undo_last() && t.undo_last(),
            "o traço e o Fill desfazem-se"
        );
        assert_eq!(
            e.por_cumprir(&t),
            Some(4),
            "{meio:?}: o desfazer volta o Fill"
        );
    }
}

/// ⭐ **A cor do Fill na tolerância medida**: o aluno que solta dentro da caixa uma cor a
/// [`TOLERANCIA_DA_COR`] por canal cumpre; a um nível além, não. E o balão pede a cor enquanto a do
/// pincel não é parecida.
#[test]
fn a_cor_do_fill_cumpre_na_tolerancia_e_nao_alem() {
    let track = caixa(PaintMedia::Digital);
    let e = EnsinoDoPainter::medido(&track).expect("ensino");
    let fill = 4;
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    for c in &e.criterios[..fill] {
        mostra(&mut t, &track, c.regiao.k).expect("mostra");
    }
    let tol = TOLERANCIA_DA_COR;
    let dentro = [AZUL[0] + tol, AZUL[1] - tol, AZUL[2] + tol];
    let alem = [AZUL[0], AZUL[1] + tol + 1, AZUL[2]];
    assert!(rgb_parecida(dentro, AZUL) && !rgb_parecida(alem, AZUL));
    t.set_brush_color_srgb8(alem);
    assert_eq!(
        e.cor_que_falta(&t, fill),
        Some(AZUL),
        "a cor além pede a do Fill"
    );
    t.set_brush_color_srgb8(dentro);
    assert_eq!(e.cor_que_falta(&t, fill), None, "a cor dentro não a pede");
    for (cor, cumpre) in [(alem, false), (dentro, true)] {
        // Outro ponto do mesmo quadrado: a região é a mesma.
        solta(&mut t, cor, 0.6, 0.4);
        assert_eq!(
            e.por_cumprir(&t) != Some(fill),
            cumpre,
            "{cor:?}: {}",
            e.fracao_pintada(&t, &e.criterios[fill])
        );
        assert!(t.undo_last());
    }
}

/// A cor certa solta FORA da caixa inunda o papel de fora, não o quadrado: não cumpre.
#[test]
fn a_cor_certa_fora_da_caixa_nao_cumpre() {
    let track = caixa(PaintMedia::Digital);
    let e = EnsinoDoPainter::medido(&track).expect("ensino");
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    for c in &e.criterios[..4] {
        mostra(&mut t, &track, c.regiao.k).expect("mostra");
    }
    solta(&mut t, AZUL, 0.1, 0.1);
    assert_eq!(
        e.por_cumprir(&t),
        Some(4),
        "{}",
        e.fracao_pintada(&t, &e.criterios[4])
    );
}

/// SONDA (W13 doc §3): o custo de preparar e medir a caixa, o tamanho da região do Fill e a razão de
/// cada aluno do Fill, por meio.
#[test]
#[ignore = "sonda: imprime as medições do doc W13"]
fn sonda_a_caixa() {
    for meio in [PaintMedia::Digital, PaintMedia::WetPaint] {
        let track = caixa(meio);
        let t0 = std::time::Instant::now();
        let mut e = EnsinoDoPainter::de(&track).expect("de");
        let de = t0.elapsed();
        let t1 = std::time::Instant::now();
        let mut ms = Vec::new();
        e.mede(&track, &std::sync::atomic::AtomicBool::new(false), |m| {
            ms.push(m);
        })
        .expect("mede");
        let mede = t1.elapsed();
        for m in ms {
            e.com_medida(m);
        }
        let novos: Vec<usize> = e.criterios.iter().map(|c| c.novos.len()).collect();
        println!("{meio:?}: de {de:?} · mede {mede:?} · novos {novos:?}");
        for (nome, cor, u, v) in [
            ("exacta", AZUL, 0.45, 0.55),
            ("outro ponto", AZUL, 0.6, 0.4),
            ("cor a 26", [66, 84, 236], 0.45, 0.55),
            ("cor a 27", [40, 137, 210], 0.45, 0.55),
            ("fora", AZUL, 0.1, 0.1),
        ] {
            let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
            for c in &e.criterios[..4] {
                mostra(&mut t, &track, c.regiao.k).expect("mostra");
            }
            solta(&mut t, cor, u, v);
            for _ in 0..60 {
                quadro(&mut t);
            }
            println!(
                "  {nome}: razão {:?}",
                e.razao(&t, &e.criterios[4]).expect("medida")
            );
        }
    }
}
