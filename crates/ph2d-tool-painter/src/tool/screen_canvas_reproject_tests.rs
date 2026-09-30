//! Os gates da tinta molhada que muda de VISTA ([`super::PainterTool::reproject_screen_canvas`]) —
//! report do dono, 29/09: *«rotacionar e pintar em seguida está pausando a simulação»*.
//!
//! ⚠️ A «vista nova» destes gates é um DESLOCAMENTO de `D` píxeis para a direita: a porta não sabe
//! o que é uma malha, só recebe o mapa de onde cada píxel estava, e um deslocamento é o mapa mais
//! simples que tem as duas metades — sítios que a vista de antes via e sítios que não via.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

const W: u32 = 64;
const H: u32 = 48;
const D: usize = 8;

fn cp(p: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: p,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um traço vermelho de Wet Paint a `y = 20`, de `x = 10` a `40`, sobre uma tela cinzenta.
fn molhada() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_brush_color_srgb8([255, 0, 0]);
    t.set_brush_strength(1.0);
    t.set_brush_size_px(6.0);
    t.set_paint_media(PaintMedia::WetPaint);
    assert!(t.bind_screen_canvas(W, H));
    assert!(t.seed_screen_canvas([128u8, 128, 128, 255].repeat((W * H) as usize)));
    t.on_canvas_pointer(cp([10.0, 20.0], PointerPhase::Down));
    for k in 1..=10 {
        t.on_canvas_pointer(cp([10.0 + 3.0 * k as f32, 20.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([40.0, 20.0], PointerPhase::Up));
    t.on_tick(16.7);
    assert!(
        t.screen_canvas_is_flowing(),
        "a fixture tem de ter água a correr"
    );
    t
}

/// A vista nova vê a de antes deslocada `D` píxeis para a direita.
fn deslocada() -> Vec<Option<[f32; 2]>> {
    (0..(W * H) as usize)
        .map(|p| {
            let (x, y) = (p % W as usize, p / W as usize);
            (x >= D).then(|| [(x - D) as f32 + 0.5, y as f32 + 0.5])
        })
        .collect()
}

/// O retrato da vista nova: AZUL, para se ver onde ele entra.
fn azul() -> Vec<u8> {
    [0u8, 0, 255, 255].repeat((W * H) as usize)
}

fn px(rgba: &[u8], x: usize, y: usize) -> [u8; 4] {
    let o = (y * W as usize + x) * 4;
    [rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3]]
}

/// ⭐⭐ **Mudar de vista NÃO pára a água** — o CONTROLO é a porta de antes (semear a tela com o
/// retrato novo), que a pára: é essa a frase do report.
#[test]
fn mudar_de_vista_nao_para_a_agua() {
    let mut controlo = molhada();
    assert!(controlo.seed_screen_canvas(azul()));
    // O guarda de identidade corre no tique: é ali que a sessão vê a tela trocada.
    controlo.on_tick(16.7);
    assert!(
        !controlo.screen_canvas_is_flowing(),
        "o CONTROLO: semear a tela pára a água"
    );

    let mut t = molhada();
    assert!(t.reproject_screen_canvas(&azul(), &deslocada()));
    assert!(
        t.screen_canvas_is_flowing(),
        "a água parou ao mudar de vista"
    );
    assert!(t.screen_canvas_is_wet(), "a sessão da água morreu");
    t.on_tick(16.7);
    assert!(
        t.screen_canvas_is_flowing(),
        "depois de um quadro a água parou"
    );
}

/// ⭐ **A tinta vai com a superfície** — o vermelho aparece `D` píxeis à direita; onde a vista de
/// antes não via entra o retrato novo; e onde via e não havia tinta fica a base de antes.
#[test]
fn a_tinta_vai_com_a_superficie() {
    let mut t = molhada();
    assert!(t.reproject_screen_canvas(&azul(), &deslocada()));
    let f = t.take_screen_canvas().expect("a tela mudou");
    // O píxel 46 vê o sítio que o 38 via — dentro do traço (10..40) —, e só tem
    // tinta se ela viajou; o 12 vê o 4, antes do traço, e só fica sem ela se
    // viajou (sem viagem, o 12 é o traço de antes no mesmo píxel).
    let tinta = px(&f.rgba, 38 + D, 20);
    assert!(
        tinta[0] > tinta[1].saturating_add(60),
        "o traço não foi com a superfície: {tinta:?}"
    );
    assert_eq!(
        px(&f.rgba, 4 + D, 20),
        [0, 0, 255, 255],
        "a tinta ficou onde estava no ecrã em vez de ir com a superfície"
    );
    assert_eq!(
        px(&f.rgba, 2, 40),
        [0, 0, 255, 255],
        "onde não se via entra o retrato"
    );
    assert_eq!(
        px(&f.rgba, 60, 44),
        [0, 0, 255, 255],
        "onde se via e não há água a tela é a semente"
    );
}

/// ⭐⭐⭐ **A base da água na vista nova é a SEMENTE, em todo píxel** (report do
/// dono, 30/09: *«melhorou mas não curou perfeitamente»*). Quem chama entrega a
/// peça SEM a água; levar a base de antes pelo píxel mais perto (a cura de
/// 29/09, debaixo da água) era adivinhar o que ele já sabe. A régua: a base de
/// antes é CINZENTA (`G = 128`) e a semente AZUL (`G = 0`), e o pigmento é
/// vermelho (`G = 0`) — um píxel com verde tem a base de antes nele.
///
/// O CONTROLO: há água (vermelho) e há borda meio transparente (vermelho E
/// azul no mesmo píxel) — sem ela o pigmento opaco tapava qualquer base.
#[test]
fn a_base_da_agua_e_a_semente_em_todo_pixel() {
    let mut t = molhada();
    assert!(t.reproject_screen_canvas(&azul(), &deslocada()));
    let f = t.take_screen_canvas().expect("a tela mudou");
    let (mut molhados, mut meio) = (0, 0);
    for p in 0..(W * H) as usize {
        let c = px(&f.rgba, p % W as usize, p / W as usize);
        assert_eq!(c[1], 0, "o píxel {p} tem a base de antes nele: {c:?}");
        if c[0] > 20 {
            molhados += 1;
            if c[2] > 20 {
                meio += 1;
            }
        }
    }
    assert!(molhados > 100, "a fixture não tem água: {molhados}");
    assert!(
        meio > 10,
        "a fixture não tem borda meio transparente: {meio}"
    );
}

/// **Sem água viva a porta recusa e nada muda** — o pincel Digital, e tamanhos que não são os da
/// tela. Quem chama semeia como antes.
#[test]
fn sem_agua_viva_ou_com_tamanhos_errados_nada_muda() {
    let mut d = PainterTool::default();
    assert!(d.bind_screen_canvas(W, H));
    assert!(!d.reproject_screen_canvas(&azul(), &deslocada()));

    let mut t = molhada();
    let _ = t.take_screen_canvas();
    assert!(!t.reproject_screen_canvas(&azul()[4..], &deslocada()));
    assert!(!t.reproject_screen_canvas(&azul(), &deslocada()[1..]));
    assert!(t.screen_canvas_is_flowing());
}
