//! ⭐ Dono 10/11: Dash e a largura do stroke no Look da forma. O traço próprio substitui o do tipo do corpo no contorno
//! e a sombra segue-o; sem ele, o do corpo continua.

use ph2d_panel_physics::tinta_canvas::{
    Marca, Tinta, Tom, espessura, sombra_do_contorno, tinta_do_contorno, traco_do_corpo,
};
use ph2d_physics_ecs::{BodyKind, Collider, ShapeLook};
use ph2d_tokens::Theme;

fn tinta(corpo: Option<BodyKind>, tom: Tom) -> Tinta {
    Tinta::Camada {
        camada: 2,
        marca: Marca::Solida,
        corpo,
        tom,
    }
}

#[test]
fn o_traco_proprio_substitui_o_do_corpo_e_a_sombra_o_segue() {
    let proprio = ShapeLook {
        largura: 5.0,
        tracejado: true,
        traco: [2.0, 9.0],
        traco_proprio: true,
        ..ShapeLook::default()
    };
    let fino = f64::from(ph2d_tokens::StrokeToken::Thin.px());
    for corpo in [Some(BodyKind::Static), Some(BodyKind::Kinematic), None] {
        let t = tinta(corpo, Tom::from(&proprio));
        let (traco, _) = tinta_do_contorno(t, Theme::Forge);
        assert_eq!(
            (traco.width, &traco.dash_pattern[..]),
            (5.0, &[2.0, 9.0][..])
        );
        let (sombra, _) = sombra_do_contorno(t, Theme::Forge).expect("sombra");
        assert_eq!(sombra.width, 5.0 + 2.0 * fino);
        assert_eq!(
            &sombra.dash_pattern[..],
            &[2.0, 9.0][..],
            "o mesmo tracejado"
        );
        // Contínuo: o próprio sem Dashed não herda o pontilhado do Kinematic.
        let liso = ShapeLook {
            tracejado: false,
            ..proprio
        };
        let (traco, _) = tinta_do_contorno(tinta(corpo, Tom::from(&liso)), Theme::Forge);
        assert!(traco.dash_pattern.is_empty(), "{corpo:?}");
    }
    // Sem traço próprio (os números semeados não contam), o do corpo: Static grosso.
    let sem = ShapeLook {
        traco_proprio: false,
        ..proprio
    };
    let (traco, _) =
        tinta_do_contorno(tinta(Some(BodyKind::Static), Tom::from(&sem)), Theme::Forge);
    assert_eq!(traco.width, espessura(Some(BodyKind::Static)));
    assert!(traco.dash_pattern.is_empty());
}

/// O que o cartão mostra sem traço próprio é o que o canvas pinta: o Kinematic pontilhado, a solta tracejada.
#[test]
fn o_traco_do_corpo_e_o_que_o_contorno_pinta() {
    let col = Collider::default();
    for corpo in [Some(BodyKind::Static), Some(BodyKind::Kinematic), None] {
        let d = traco_do_corpo(&col, corpo);
        let marca = if corpo.is_some() {
            Marca::Solida
        } else {
            Marca::Solta
        };
        let neutro = ph2d_panel_physics::tinta_canvas::TOM_NEUTRO;
        let t = Tinta::Camada {
            camada: 0,
            marca,
            corpo,
            tom: neutro,
        };
        let (traco, _) = tinta_do_contorno(t, Theme::Forge);
        assert_eq!(d.largura, traco.width, "{corpo:?}");
        assert_eq!(
            d.padrao.map(Vec::from),
            (!traco.dash_pattern.is_empty()).then(|| traco.dash_pattern.to_vec())
        );
    }
}
