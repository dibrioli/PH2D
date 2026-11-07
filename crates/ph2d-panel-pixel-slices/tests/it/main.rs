//! Os cartões da janela SLICES por gesto real, cada controlo até ao pedido. A tela inteira com todos os
//! painéis prova-se em `the_pixel_slices_answers_a_real_pointer`; o documento, em `shells/pixel-scenes`.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_slices::canal::{self, Contexto, Linha, Pedido, Vista};
use ph2d_panel_pixel_slices::{PixelSlicesPanel, PixelSlicesPanelState, ids};
use ph2d_pixel_core::SliceId;
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect::new(0.0, 0.0, 1600.0, 1400.0);

fn contexto() -> Contexto {
    Contexto {
        na_mao: true,
        linhas: vec![Linha {
            id: SliceId(3),
            nome: "button".to_owned(),
            selecionada: true,
        }],
        vista: Some(Vista {
            nome: Some("button".to_owned()),
            cor: Some([0, 0, 255, 255]),
            bounds: [Some(1), Some(2), Some(3), Some(4)],
            ..Vista::default()
        }),
        transform: false,
    }
}

fn mesa(x: Contexto) -> Mesa<PixelSlicesPanel> {
    let _ = canal::take_pedidos();
    canal::publish_contexto(x);
    let mut m = Mesa::nova(PixelSlicesPanelState::default());
    m.host.set_panel_visible(PixelSlicesPanel::ID, true);
    m.assenta_em(JANELA);
    m
}

fn clica(m: &mut Mesa<PixelSlicesPanel>, id: NodeId) -> Vec<Pedido> {
    assert!(m.rect(id).is_some(), "o controlo não se pintou");
    let _ = m.clica(id);
    m.assenta_em(JANELA);
    canal::take_pedidos()
}

#[test]
fn cada_controlo_chega_ao_pedido() {
    let mut m = mesa(contexto());
    for (id, p) in [
        (ids::linha(0), Pedido::Seleciona(SliceId(3))),
        (ids::ALL, Pedido::Todas),
        (ids::NONE, Pedido::Nenhuma),
        (ids::TRANSFORM, Pedido::Transform),
        (ids::NINE, Pedido::Nove(true)),
        (ids::PIVOT, Pedido::Pivot(true)),
        (ids::DUPLICATE, Pedido::Duplica),
        (ids::DELETE, Pedido::Apaga),
    ] {
        assert_eq!(clica(&mut m, id), vec![p]);
    }
}

/// Sem a ferramenta na mão a janela fecha-se e não fica rect nenhum a roubar o ponteiro.
#[test]
fn sem_a_ferramenta_a_janela_fecha() {
    let mut m = mesa(Contexto::default());
    m.assenta_em(JANELA);
    assert!(m.rect(ids::ALL).is_none());
    assert!(!m.host.panel_visible(PixelSlicesPanel::ID));
}

/// Sem slices a janela ensina a criar uma; não há All nem None para carregar.
#[test]
fn sem_slices_so_o_transform_responde() {
    let x = Contexto {
        na_mao: true,
        ..Contexto::default()
    };
    let mut m = mesa(x);
    assert!(m.rect(ids::ALL).is_none());
    assert_eq!(clica(&mut m, ids::TRANSFORM), vec![Pedido::Transform]);
}
