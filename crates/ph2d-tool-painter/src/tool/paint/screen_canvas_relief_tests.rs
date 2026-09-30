//! Os gates do RELEVO na tela da vista 3D (`docs/3D/29`, D2–D4). A cor sai
//! SEM a luz 2D e a espessura sai crua, em píxeis, para a escultura a pousar
//! na peça. ⚠️ Mora em `paint/` (declarado no `impasto_live.rs`) porque arma o
//! pincel pelo `PaintState`, que só é visível daqui.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

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

/// Um pincel de impasto opaco — o `impasto_canvas` dos gates do corpo, sem a
/// fonte (quem a dá é a tela da vista ou o `set_source`).
fn impasto(t: &mut PainterTool) {
    let b = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.3],
        space_attenuation: false,
        impasto: true,
        impasto_depth: 0.5,
        impasto_smoothing: 0.0,
        impasto_body: 1.0,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
}

fn traco(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([10.0, 24.0], PointerPhase::Down));
    for k in 1..=10 {
        t.on_canvas_pointer(cp([10.0 + 3.0 * k as f32, 24.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([40.0, 24.0], PointerPhase::Up));
}

/// ⭐⭐⭐ **GATE — na tela da vista a cor sai SEM a luz 2D; numa sprite, com.**
///
/// ⚠️ O CONTROLO é a mesma pincelada numa sprite: lá o passe de luz corre e a
/// drenagem DIFERE dos píxeis que o pincel pousou. Sem ele, um relevo que nem
/// chegasse a existir passaria a metade da tela da vista.
#[test]
fn na_tela_da_vista_a_cor_sai_sem_a_luz_2d() {
    // (1) CONTROLO: numa sprite o impasto acende a drenagem.
    let mut s = PainterTool::default();
    s.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    impasto(&mut s);
    traco(&mut s);
    assert!(s.impasto_visible(), "CONTROLO: numa sprite o relevo acende");
    let (lit, _, _) = s.drain_preview_arc().expect("o traço sujou a sprite");
    assert_ne!(
        lit.as_slice(),
        s.canvas_rgba.as_slice(),
        "CONTROLO: numa sprite a drenagem leva a luz 2D"
    );

    // (2) Na tela da vista, a drenagem É o que o pincel pousou.
    let mut t = PainterTool::default();
    impasto(&mut t);
    assert!(t.bind_screen_canvas(W, H));
    traco(&mut t);
    assert!(!t.impasto_visible(), "a luz 2D ligou na tela da vista");
    let f = t.take_screen_canvas().expect("o traço sujou a tela");
    assert_eq!(
        f.rgba.as_slice(),
        t.canvas_rgba.as_slice(),
        "a tela da vista levou a luz 2D para a peça"
    );
}

/// ⭐⭐⭐ **GATE — a espessura sai em PÍXEIS, na janela pedida, pela lei do
/// passe de luz** (comprometido + traço × `impasto_depth` × `16`).
#[test]
fn a_espessura_da_tela_sai_em_pixeis_na_janela() {
    let mut t = PainterTool::default();
    impasto(&mut t);
    assert!(
        t.screen_canvas_heights_in((0, 0, W, H)).is_none(),
        "sem tela da vista não há espessura a pousar"
    );
    assert!(t.bind_screen_canvas(W, H));
    assert!(
        t.screen_canvas_heights_in((0, 0, W, H)).is_none(),
        "CONTROLO: antes do traço nada tem relevo"
    );
    traco(&mut t);
    let id = t.layers.active().expect("a tela tem uma camada");
    let cheia = t.layer_height_view(id).expect("o traço deixou relevo");
    let prof = t.layers.get(id).expect("a camada existe").impasto_depth;

    let janela = (20, 20, 10, 6);
    let h = t.screen_canvas_heights_in(janela).expect("há relevo");
    assert_eq!(h.len(), 60, "a janela é 10×6");
    let mut algum = false;
    for j in 0..6u32 {
        for i in 0..10u32 {
            let k = ((20 + j) * W + 20 + i) as usize;
            let esperado = cheia[k] * prof * 16.0;
            assert_eq!(
                h[(j * 10 + i) as usize].to_bits(),
                esperado.to_bits(),
                "({i},{j})"
            );
            algum |= esperado > 0.0;
        }
    }
    assert!(algum, "a fixtura não contém relevo na janela");

    // Uma janela a sair da tela é recortada, nunca recusada.
    let borda = t
        .screen_canvas_heights_in((60, 44, 10, 10))
        .expect("há relevo");
    assert_eq!(borda.len(), 4 * 4);
}

/// ⭐⭐ **GATE — a espessura vai no MESMO quadro que a cor, à volta do
/// rectângulo mudado** — a janela cobre o rectângulo com a margem, e o que ela
/// leva é a porta da espessura sobre essa janela, ao bit.
#[test]
fn a_drenagem_leva_a_espessura_a_volta_do_rectangulo() {
    let mut t = PainterTool::default();
    impasto(&mut t);
    assert!(t.bind_screen_canvas(W, H));
    traco(&mut t);
    let f = t.take_screen_canvas().expect("o traço sujou a tela");
    let r = f.rect.expect("uma tela de uma camada devolve o rectângulo");
    let rel = f
        .relief
        .as_ref()
        .expect("um traço de impasto leva espessura");
    let (x, y, w, h) = rel.window;
    assert!(
        x + 2 <= r.0.max(2) && y + 2 <= r.1.max(2),
        "a janela {:?} não cobre a margem à esquerda/topo de {r:?}",
        rel.window
    );
    assert!(
        x + w >= (r.0 + r.2 + 2).min(W) && y + h >= (r.1 + r.3 + 2).min(H),
        "a janela {:?} não cobre a margem à direita/baixo de {r:?}",
        rel.window
    );
    assert_eq!(rel.px.len(), (w * h) as usize);
    assert_eq!(
        Some(&rel.px),
        t.screen_canvas_heights_in(rel.window).as_ref(),
        "a drenagem e a porta da espessura divergem"
    );

    // CONTROLO: sem impasto, a drenagem não leva espessura nenhuma.
    let mut c = PainterTool::default();
    impasto(&mut c);
    c.paint.brush.impasto = false;
    c.paint
        .brush_by_mode
        .iter_mut()
        .for_each(|b| b.impasto = false);
    assert!(c.bind_screen_canvas(W, H));
    traco(&mut c);
    let g = c.take_screen_canvas().expect("o traço sujou a tela");
    assert!(g.relief.is_none(), "um traço sem impasto levou espessura");
}

/// ⭐⭐ **GATE — A MEIO do traço a espessura já sai** — o artista vê o relevo
/// crescer enquanto pinta, antes de largar. ⚠️ Até largar, a espessura vive no
/// envelope do traço aberto e NÃO no comprometido; uma janela que lesse só o
/// comprometido entregava zero a pincelada inteira (foi uma mutação que
/// SOBREVIVEU ao gate de cima, que só lê depois do pen-up).
#[test]
fn a_meio_do_traco_a_espessura_ja_sai() {
    let mut t = PainterTool::default();
    impasto(&mut t);
    assert!(t.bind_screen_canvas(W, H));
    t.on_canvas_pointer(cp([10.0, 24.0], PointerPhase::Down));
    for k in 1..=6 {
        t.on_canvas_pointer(cp([10.0 + 3.0 * k as f32, 24.0], PointerPhase::Move));
    }
    let id = t.layers.active().expect("a tela tem uma camada");
    assert!(
        t.heights
            .get(&id)
            .is_none_or(|c| c.iter().all(|&h| h == 0.0)),
        "a fixtura não contém o fenómeno: o traço ainda aberto já comprometeu relevo"
    );
    let cheia = t.layer_height_view(id).expect("o traço aberto tem relevo");
    let prof = t.layers.get(id).expect("a camada existe").impasto_depth;
    let h = t
        .screen_canvas_heights_in((0, 0, W, H))
        .expect("há relevo a meio");
    let mut algum = false;
    for (k, (&got, &raw)) in h.iter().zip(&cheia).enumerate() {
        assert_eq!(got.to_bits(), (raw * prof * 16.0).to_bits(), "píxel {k}");
        algum |= got > 0.0;
    }
    assert!(algum, "a meio do traço a janela entregou zero");
}
