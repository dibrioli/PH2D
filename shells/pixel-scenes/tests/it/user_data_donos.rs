//! **O cartão User Data da camada, do cel e da tag, do pedido ao documento** (P12 U, O17/R5): cada
//! gesto é UM passo «User Data» pela porta `Op::UserData`, e o desfazer devolve o de antes.

use ph2d_editor_core::panel::cards::user_data::CardEdit;
use ph2d_editor_core::pixel_panel::{PixelFrameAction, PixelOwner, PixelRequest};
use ph2d_pixel_core::{FrameId, LayerId, Owner, PropValue};

use crate::comum::Palco;

fn dono(o: PixelOwner) -> Owner {
    match o {
        PixelOwner::Sprite => Owner::Sprite,
        PixelOwner::Layer(l) => Owner::Layer(LayerId(l)),
        PixelOwner::Cel { layer, frame } => Owner::Cel((LayerId(layer), FrameId(frame))),
        PixelOwner::Tag(i) => Owner::Tag(i),
        PixelOwner::Tileset(t) => Owner::Tileset(ph2d_pixel_core::TilesetId(t)),
    }
}

fn gestos(p: &mut Palco, o: PixelOwner) {
    let antes = p.app.doc(p.key).expect("doc").history.entries().count();
    let original = p.sprite().user_of(dono(o)).expect("dono").clone();
    for e in [
        CardEdit::Color([10, 20, 30, 255]),
        CardEdit::Text("hero".into()),
        CardEdit::Add,
        CardEdit::Kind("property".into()),
        CardEdit::Value("property".into(), "yes".into()),
        CardEdit::Value("property".into(), "talvez".into()),
    ] {
        p.pede(PixelRequest::UserData(o, e));
    }
    let d = p.app.doc(p.key).expect("doc");
    assert_eq!(
        d.history.entries().count(),
        antes + 5,
        "{o:?}: o valor que não é Bool não é passo"
    );
    assert_eq!(
        d.history.entries().last().expect("passo").0,
        ph2d_pixel_core::Label::new("pixel.history.user_data")
    );
    let u = p.sprite().user_of(dono(o)).expect("dono");
    assert_eq!(u.color, [10, 20, 30, 255]);
    assert_eq!(u.text, "hero");
    assert_eq!(u.props[""]["property"], PropValue::Bool(true));
    let info = p.quadro();
    let v = match o {
        PixelOwner::Layer(_) => info.user.layer,
        PixelOwner::Cel { .. } => info.user.cel,
        _ => info.user.tag,
    };
    assert_eq!(v.expect("o cartão vê o dono").text, "hero", "{o:?}");
    for _ in 0..5 {
        assert!(p.app.undo(p.key));
    }
    assert_eq!(p.sprite().user_of(dono(o)), Some(&original), "{o:?}");
}

#[test]
fn o_user_data_da_camada_do_cel_e_da_tag_e_um_passo_e_desfaz() {
    let mut p = Palco::limpo();
    let info = p.pede(PixelRequest::FrameAction(PixelFrameAction::NewTag));
    let (l, f) = (
        info.active_layer.expect("camada"),
        info.active_frame.expect("frame"),
    );
    assert!(info.user.tag.is_some(), "a tag nova cobre o frame activo");
    assert!(
        p.sprite().cel((LayerId(l), FrameId(f))).is_some(),
        "a cena tem cel na camada activa"
    );
    gestos(&mut p, PixelOwner::Layer(l));
    gestos(&mut p, PixelOwner::Cel { layer: l, frame: f });
    gestos(&mut p, PixelOwner::Tag(0));
}
