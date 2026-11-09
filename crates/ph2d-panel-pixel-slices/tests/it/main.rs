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
        com_sprite: true,
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
        user: None,
        collider: None,
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

/// Aberta sem a ferramenta na mão (menu *Window*), a janela fica e ensina; sem slices não há All.
#[test]
fn sem_a_ferramenta_a_janela_aberta_fica() {
    let mut m = mesa(Contexto::default());
    m.assenta_em(JANELA);
    assert!(m.rect(ids::ALL).is_none());
    assert!(m.host.panel_visible(PixelSlicesPanel::ID));
}

/// Sem slices a janela ensina a criar uma; não há All nem None para carregar.
#[test]
fn sem_slices_so_o_transform_responde() {
    let x = Contexto {
        com_sprite: true,
        na_mao: true,
        ..Contexto::default()
    };
    let mut m = mesa(x);
    assert!(m.rect(ids::ALL).is_none());
    assert_eq!(clica(&mut m, ids::TRANSFORM), vec![Pedido::Transform]);
}

/// O cartão User Data da slice (P12 U): nasce fechado atrás do botão ao lado do nome; aberto, cada
/// controlo da tabela chega ao pedido pelo ponteiro, e o tipo que a tela não edita só se lê.
#[test]
fn o_cartao_user_data_chega_ao_pedido() {
    use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
    use ph2d_pixel_core::user::edit::{PropKind, UserEdit};
    use ph2d_pixel_core::{PropMap, PropValue, UserData};
    let mut mapa = PropMap::new();
    mapa.insert("hp".into(), PropValue::I32(3));
    mapa.insert("raw".into(), PropValue::U8(7));
    let u = UserData {
        props: [(String::new(), mapa)].into(),
        ..UserData::default()
    };
    let mut x = contexto();
    x.user = Some((SliceId(3), u));
    let mut m = mesa(x);
    let ud = ids::user_data();
    assert!(m.rect(ud.name(0)).is_none(), "nasce fechado");
    assert!(clica(&mut m, ud.toggle).is_empty());
    assert!(m.rect(ud.name(0)).is_some(), "aberto, a tabela pinta-se");
    assert!(m.rect(ud.name(1)).is_none(), "UInt8 só se lê: sem caixa");
    let pede = |e| vec![Pedido::UserData(SliceId(3), e)];
    assert_eq!(clica(&mut m, ud.add), pede(UserEdit::Add));
    assert_eq!(
        clica(&mut m, ud.kind(0)),
        pede(UserEdit::Kind("hp".into(), PropKind::Float))
    );
    assert_eq!(
        clica(&mut m, ud.remove(0)),
        pede(UserEdit::Remove("hp".into()))
    );
    if let Some(InteractiveState::TextInput { text, .. }) = m.host.store_mut().get_mut(ud.value(0))
    {
        "12".clone_into(text);
    }
    let _ = m.aplica(vec![WidgetEvent::Submit(ud.value(0))]);
    assert_eq!(
        canal::take_pedidos(),
        pede(UserEdit::Value("hp".into(), "12".into()))
    );
    let _ = m.aplica(vec![WidgetEvent::Submit(ud.value(0))]);
    m.assenta_em(JANELA);
    let _ = canal::take_pedidos();
}
