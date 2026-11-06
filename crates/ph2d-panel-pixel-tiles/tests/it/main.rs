//! Os cartões dos tilemaps por gesto real: clique, número e Cancel, cada um até ao pedido.

use std::sync::Arc;

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_tiles::canal::{self, Alvo, Contexto, Eixo, Mao, Pedido, Tela};
use ph2d_panel_pixel_tiles::{PixelTilesPanel, PixelTilesPanelState, ids};
use ph2d_pixel_tiles::desenho::Modo;
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect::new(0.0, 0.0, 1600.0, 1400.0);
const LADO: u16 = 4;

fn contexto(lado_h: u16) -> Contexto {
    let px = vec![200; usize::from(LADO) * usize::from(lado_h) * 4];
    Contexto {
        mao: Some(Mao::default()),
        tile_w: LADO,
        tile_h: lado_h,
        imagens: Arc::new(vec![px.clone(), px.clone(), px]),
        ..Contexto::default()
    }
}

fn mesa(x: Contexto) -> Mesa<PixelTilesPanel> {
    let _ = canal::take_pedidos();
    canal::publish_contexto(x);
    let mut m = Mesa::nova(PixelTilesPanelState::default());
    m.host.set_panel_visible(PixelTilesPanel::ID, true);
    m.assenta_em(JANELA);
    m
}

fn clica(m: &mut Mesa<PixelTilesPanel>, id: NodeId) -> Vec<Pedido> {
    assert!(m.rect(id).is_some(), "o controlo não se pintou");
    let _ = m.clica(id);
    m.assenta_em(JANELA);
    canal::take_pedidos()
}

#[test]
fn cada_controlo_do_cartao_chega_ao_pedido() {
    let mut m = mesa(contexto(LADO));
    for (id, p) in [
        (ids::TARGET_TILES, Pedido::Alvo(Alvo::Tiles)),
        (ids::TARGET_PIXELS, Pedido::Alvo(Alvo::Pixels)),
        (ids::MODE_MANUAL, Pedido::Modo(Modo::Manual)),
        (ids::MODE_AUTO, Pedido::Modo(Modo::Auto)),
        (ids::MODE_STACK, Pedido::Modo(Modo::Stack)),
        (ids::FLIP_X, Pedido::Flip(Eixo::X)),
        (ids::FLIP_Y, Pedido::Flip(Eixo::Y)),
        (ids::FLIP_D, Pedido::Flip(Eixo::D)),
        (ids::ADD_TILE, Pedido::NovoTile),
        (ids::DELETE_TILE, Pedido::ApagaTile),
        (ids::tile(2), Pedido::Tile(2)),
    ] {
        assert_eq!(clica(&mut m, id), vec![p]);
    }
}

#[test]
fn com_o_alvo_em_tiles_os_modos_nao_se_pintam_e_a_flip_d_so_em_tile_quadrado() {
    let mut x = contexto(LADO);
    x.mao = Some(Mao {
        alvo: Alvo::Tiles,
        ..Mao::default()
    });
    let m = mesa(x);
    assert!(m.rect(ids::MODE_AUTO).is_none());
    let m = mesa(contexto(LADO * 2));
    assert!(m.rect(ids::FLIP_X).is_some());
    assert!(
        m.rect(ids::FLIP_D).is_none(),
        "tile não quadrado não tem diagonal"
    );
}

#[test]
fn a_janela_escreve_os_lados_e_devolve_o_verbo() {
    let mut m = mesa(Contexto::default());
    canal::abrir(Tela::NovaCamada);
    m.assenta_em(JANELA);
    let ev = m.host.type_into_number(ids::TILE_W, "24");
    let _ = m.aplica(ev);
    let ev = m.host.type_into_number(ids::TILE_H, "8");
    let _ = m.aplica(ev);
    m.assenta_em(JANELA);
    assert_eq!(
        clica(&mut m, ids::TILES_OK),
        vec![Pedido::NovaCamada {
            conjunto: None,
            w: 24,
            h: 8
        }]
    );
    assert_eq!(m.st.tela(), Tela::Tileset);
}

#[test]
fn o_convert_so_oferece_flips_em_tile_quadrado_e_o_cancel_fecha() {
    let mut m = mesa(Contexto::default());
    canal::abrir(Tela::Converter);
    m.assenta_em(JANELA);
    assert!(clica(&mut m, ids::MATCH_FLIPS).is_empty());
    assert_eq!(
        clica(&mut m, ids::TILES_OK),
        vec![Pedido::Converte {
            conjunto: None,
            w: 16,
            h: 16,
            flips: true
        }]
    );
    canal::abrir(Tela::Converter);
    m.assenta_em(JANELA);
    let ev = m.host.type_into_number(ids::TILE_H, "8");
    let _ = m.aplica(ev);
    m.assenta_em(JANELA);
    assert!(m.rect(ids::MATCH_FLIPS).is_none());
    assert_eq!(clica(&mut m, ids::TILES_CANCEL), vec![Pedido::Fechou]);
}
