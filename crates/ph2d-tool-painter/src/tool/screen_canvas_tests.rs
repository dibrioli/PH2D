//! Os gates da [`super`] — a tela da vista 3D presa no Painter.
//!
//! ⚠️ Eles passam pela porta PÚBLICA do ponteiro (`on_canvas_pointer`), que é a
//! mesma que a escultura usa: um traço conduzido por dentro afirmaria sobre o
//! motor e nunca sobre a costura.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: u32 = 64;
const H: u32 = 48;

fn cp(p: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: p,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([10.0, 20.0], PointerPhase::Down));
    for k in 1..=10 {
        t.on_canvas_pointer(cp([10.0 + 3.0 * k as f32, 20.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([40.0, 20.0], PointerPhase::Up));
}

fn pintor() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_brush_color_srgb8([255, 0, 0]);
    t.set_brush_strength(1.0);
    t.set_brush_size_px(6.0);
    t
}

/// ⭐⭐ **O motor do Painter pinta a tela da vista, e a drenagem dela entrega a
/// tinta com o rectângulo que mudou** — o elo de que a escultura depende.
#[test]
fn um_traco_na_tela_da_vista_sai_na_drenagem_com_o_rectangulo() {
    let mut t = pintor();
    assert!(t.bind_screen_canvas(W, H), "a tela nasceu");
    assert!(t.on_screen_canvas());
    traco(&mut t);
    let f = t.take_screen_canvas().expect("o traço sujou a tela");
    assert_eq!((f.w, f.h), (W, H));
    let pintados = f.rgba.as_chunks::<4>().0.iter().filter(|p| p[3] > 0).count();
    assert!(pintados > 20, "o traço deixou tinta: {pintados} píxeis");
    let r = f
        .rect
        .expect("uma tela de uma camada devolve o rectângulo sujo");
    assert!(
        r.2 > 0 && r.3 > 0 && r.2 < W,
        "o rectângulo é o do traço: {r:?}"
    );
    assert!(
        t.take_screen_canvas().is_none(),
        "drenar outra vez não entrega nada"
    );
}

/// ⭐⭐⭐ **Com a tela presa, a PONTE DA SPRITE não lhe toca** — as quatro portas
/// que ela usa a cada quadro. Sem isto a ponte drenava a tela antes da escultura
/// (o rectângulo sujo perdia-se), ligava a sprite escolhida por cima dela, e o
/// `Apply` assava a vista numa sprite.
#[test]
fn com_a_tela_presa_a_ponte_da_sprite_nao_lhe_toca() {
    let mut t = pintor();
    t.bind_screen_canvas(W, H);
    traco(&mut t);
    assert!(t.take_preview_arc().is_none(), "a drenagem da ponte");
    assert!(!t.take_preview_dirty(), "a pista GPU da ponte");
    assert!(
        !t.needs_document_bind(7),
        "a sprite escolhida não desloca a tela"
    );
    t.request_commit();
    assert!(
        !t.take_pending_commit(),
        "o Apply não assa a vista numa sprite"
    );
    // O CONTROLO: a escultura continua a drenar a mesma tinta.
    assert!(t.take_screen_canvas().is_some());
}

/// **Limpar devolve a tela transparente** — o fim de um traço, depois de ele ter
/// sido pousado; e **soltar devolve a ponte à sprite**.
#[test]
fn limpar_esvazia_e_soltar_devolve_a_ponte_a_sprite() {
    let mut t = pintor();
    t.bind_screen_canvas(W, H);
    traco(&mut t);
    let _ = t.take_screen_canvas();
    t.clear_screen_canvas();
    assert!(t.on_screen_canvas(), "limpar não solta");
    assert!(
        t.canvas_rgba.iter().all(|&b| b == 0),
        "a tela voltou transparente"
    );
    t.release_screen_canvas();
    assert!(!t.on_screen_canvas());
    assert!(
        t.needs_document_bind(7),
        "a ponte volta a poder ligar a sprite"
    );
}

/// **A vista muda de tamanho ⇒ a tela acompanha**; o mesmo tamanho não a refaz.
#[test]
fn a_tela_segue_o_tamanho_da_vista() {
    let mut t = pintor();
    assert!(t.bind_screen_canvas(W, H));
    assert!(!t.bind_screen_canvas(W, H), "o mesmo tamanho é idempotente");
    assert!(t.bind_screen_canvas(2 * W, H), "a vista cresceu");
    assert_eq!(t.canvas_size(), (2 * W, H));
}

/// ⭐⭐ **Prender a tela GUARDA a sprite de várias camadas que estava ligada**, e
/// soltar deixa-a voltar inteira — a tela entra como documento, nunca por um
/// `set_source` que achataria as camadas.
#[test]
fn prender_a_tela_nao_achata_a_sprite_que_estava_ligada() {
    let mut t = pintor();
    t.bind_document(1, vec![255u8; 32 * 32 * 4], 32, 32);
    t.layers.add_raster("Layer 2", 32, 32);
    let camadas = t.layers.root().len();
    assert!(camadas >= 2);
    t.bind_screen_canvas(W, H);
    assert!(t.is_trivial_stack(), "a tela é um documento de uma camada");
    t.release_screen_canvas();
    t.bind_document(1, vec![0u8; 4], 1, 1);
    assert_eq!(
        t.layers.root().len(),
        camadas,
        "a sprite voltou com as camadas dela"
    );
    assert_eq!(t.source_size, (32, 32));
}
