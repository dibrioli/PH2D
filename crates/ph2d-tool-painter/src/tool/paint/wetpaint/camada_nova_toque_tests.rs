//! **O 1.º toque Wet da camada nova** (dono 12/11, B18.4): o quadro do pen-down pede à pista GPU a janela do traço,
//! não a tela (a tela inteira = a dobra do relevo inteira, ~45 ms a 4K no app).

use super::agua_sobre_oleo_tests::ponteiro;
use crate::tool::PainterTool;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

const W: u32 = 640;
const H: u32 = 480;

/// A área (texels) que a pista GPU reivindica no quadro: `None` = a tela.
fn reivindicada(t: &mut PainterTool) -> u64 {
    t.on_tick(1000.0 / 60.0);
    if !t.take_preview_dirty() {
        return 0;
    }
    t.preview_gpu_region()
        .map_or(u64::from(W) * u64::from(H), |(_, _, w, h)| {
            u64::from(w) * u64::from(h)
        })
}

/// O pior quadro (área reivindicada) do toque e dos primeiros passos de um traço curto em `y`.
fn pior_do_toque(t: &mut PainterTool, y: f32) -> u64 {
    let mut pior = 0;
    t.on_canvas_pointer(ponteiro(200.0, y, PointerPhase::Down));
    pior = pior.max(reivindicada(t));
    for k in 1..=6 {
        t.on_canvas_pointer(ponteiro(200.0 + 6.0 * k as f32, y, PointerPhase::Move));
        pior = pior.max(reivindicada(t));
    }
    t.on_canvas_pointer(ponteiro(240.0, y, PointerPhase::Up));
    for _ in 0..3 {
        let _ = reivindicada(t);
    }
    pior
}

#[test]
fn o_primeiro_toque_da_camada_nova_reivindica_o_traco_e_nao_a_tela() {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_tool_mode("wetpaint");
    t.set_brush_size_px(20.0);
    for _ in 0..3 {
        let _ = reivindicada(&mut t);
    }
    let tela = u64::from(W) * u64::from(H);
    let base = pior_do_toque(&mut t, 100.0);
    let segundo = pior_do_toque(&mut t, 160.0);
    let nova = t.add_raster_layer("nova").expect("a camada nova");
    t.select_layer(nova);
    for _ in 0..3 {
        let _ = reivindicada(&mut t);
    }
    let primeiro_da_nova = pior_do_toque(&mut t, 260.0);
    let segundo_da_nova = pior_do_toque(&mut t, 320.0);
    let caixa = 8 * tela / 100;
    eprintln!(
        "área reivindicada (texels; tela {tela}): base 1.º {base} 2.º {segundo} · nova 1.º {primeiro_da_nova} 2.º {segundo_da_nova}"
    );
    assert!(
        segundo <= caixa,
        "controlo: o 2.º toque é confinado ({base} {segundo} {primeiro_da_nova} {segundo_da_nova} de {tela})"
    );
    assert!(
        primeiro_da_nova <= caixa,
        "o 1.º toque da camada nova reivindicou {primeiro_da_nova} de {tela} texels"
    );
    assert!(
        base <= caixa,
        "o 1.º toque da sessão reivindicou {base} de {tela} texels"
    );
}

/// O «+» no papel do dono (relevo aceso, sem tinta viva): os quadros seguintes não reivindicam nada à pista GPU
/// (a camada transparente não muda a figura nem o dente). O controlo é o caminho antigo (`TROCA_RECOMPOE_SEMPRE`):
/// a tela inteira. Com a tinta Wet viva o «+» ainda recompõe tudo (a paridade dá a tela ≠ oráculo: B18.4 aberto).
#[test]
fn no_papel_do_dono_o_mais_transparente_nao_reivindica_nada() {
    let mede = |antigo: bool| {
        let mut t = no_papel_do_dono();
        for _ in 0..3 {
            let _ = reivindicada(&mut t);
        }
        crate::tool::layers::TROCA_RECOMPOE_SEMPRE.with(|c| c.set(antigo));
        let nova = t.add_raster_layer("nova").expect("a camada nova");
        t.select_layer(nova);
        crate::tool::layers::TROCA_RECOMPOE_SEMPRE.with(|c| c.set(false));
        (0..3).map(|_| reivindicada(&mut t)).max().unwrap_or(0)
    };
    let (antes, depois) = (mede(true), mede(false));
    let tela = u64::from(W) * u64::from(H);
    eprintln!("o «+» (texels; tela {tela}): caminho antigo {antes} · cura {depois}");
    assert_eq!(antes, tela, "controlo: o caminho antigo reivindica a tela");
    assert_eq!(
        depois, 0,
        "o «+» transparente reivindicou {depois} de {tela} texels"
    );
}

/// O documento do dono: o «Textured Paper» do New Image (Relief 0,5).
pub(super) fn no_papel_do_dono() -> PainterTool {
    let mut t = PainterTool::default();
    t.agenda_papel_texturizado(7);
    t.bind_document(7, vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_tool_mode("wetpaint");
    t.set_brush_size_px(20.0);
    t
}

#[test]
fn no_papel_do_dono_o_primeiro_toque_da_camada_nova_reivindica_o_traco() {
    let mut t = no_papel_do_dono();
    for _ in 0..3 {
        let _ = reivindicada(&mut t);
    }
    let tela = u64::from(W) * u64::from(H);
    let base = pior_do_toque(&mut t, 100.0);
    let nova = t.add_raster_layer("nova").expect("a camada nova");
    t.select_layer(nova);
    for _ in 0..3 {
        let _ = reivindicada(&mut t);
    }
    let primeiro_da_nova = pior_do_toque(&mut t, 260.0);
    let segundo_da_nova = pior_do_toque(&mut t, 320.0);
    eprintln!(
        "papel do dono (texels; tela {tela}): base 1.º {base} · nova 1.º {primeiro_da_nova} 2.º {segundo_da_nova}"
    );
    let caixa = 8 * tela / 100;
    assert!(
        primeiro_da_nova <= caixa && segundo_da_nova <= caixa,
        "o 1.º toque da camada nova reivindicou {primeiro_da_nova} de {tela} texels"
    );
}
