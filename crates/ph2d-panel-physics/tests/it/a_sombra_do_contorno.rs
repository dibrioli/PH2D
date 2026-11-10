//! ⭐ Dono 07/11 (fecho da GEF02): sombra escura por baixo do contorno da camada no canvas — o contorno laranja do
//! Crate sumia sobre a arte laranja. A lei mora em `tinta_canvas`; a pintura (`ph2d-app-physics::overlay`) pinta
//! TODAS as sombras antes das cores.

use ph2d_panel_physics::tinta_canvas::{Marca, Tinta, sombra_do_contorno, tinta_do_contorno};
use ph2d_physics_ecs::BodyKind;
use ph2d_tokens::Theme;

/// ⭐ Dono 07/11: sombra escura por baixo do contorno da camada (o Crate laranja sobre arte laranja).
/// Mutações: sombra da largura do traço; sem o padrão; cor da camada; opacidade fora da marca; sombra
/// nas anotações de cor fixa.
#[test]
fn cada_contorno_da_camada_tem_uma_sombra_escura_mais_larga_com_o_mesmo_padrao() {
    let luz = |c: [f32; 4]| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    let fino = f64::from(ph2d_tokens::StrokeToken::Thin.px());
    for theme in [Theme::Forge, Theme::Sunstone] {
        for corpo in [
            Some(BodyKind::Static),
            Some(BodyKind::Dynamic),
            Some(BodyKind::Kinematic),
            None,
        ] {
            for marca in [
                Marca::Solida,
                Marca::SensorVazio,
                Marca::Atravessavel,
                Marca::Solta,
            ] {
                let tinta = Tinta::Camada {
                    camada: 3,
                    marca,
                    corpo,
                    tom: ph2d_panel_physics::tinta_canvas::TOM_NEUTRO,
                };
                let (traco, cor) = tinta_do_contorno(tinta, theme);
                let (sombra, escura) =
                    sombra_do_contorno(tinta, theme).expect("toda camada tem sombra");
                assert!(
                    (sombra.width - (traco.width + 2.0 * fino)).abs() < 1e-9,
                    "a sombra sai Thin de cada lado: {} × {}",
                    sombra.width,
                    traco.width
                );
                assert_eq!(
                    sombra.dash_pattern, traco.dash_pattern,
                    "o mesmo padrão, {corpo:?} {marca:?}"
                );
                assert!(
                    luz(escura.components) < 0.1 && luz(escura.components) < luz(cor.components),
                    "a sombra é ESCURA em {theme:?}: {:?}",
                    escura.components
                );
                assert!(
                    escura.components[3] < cor.components[3] && escura.components[3] > 0.0,
                    "a sombra segue a opacidade da marca"
                );
            }
        }
        assert!(sombra_do_contorno(Tinta::Fixa([1.0, 0.0, 0.0, 1.0]), theme).is_none());
    }
}
