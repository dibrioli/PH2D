//! Os gates do componente do mover de vista de cima.

use super::*;

#[test]
fn a_ida_e_volta_lei_componente_e_exacta() {
    // ⚠️ Uma tradução perdida num campo lê-se como «o painel não guarda» e é o
    // tipo de defeito que só aparece depois de gravar e reabrir.
    for l in [
        TopDownLaw::default(),
        TopDownLaw {
            speed: 7.25,
            acceleration: 12.0,
            deceleration: 30.0,
            direction: DirectionMode::FourWay,
            viewpoint: Viewpoint::Custom,
            viewpoint_angle_deg: 33.75,
            rotation: RotationMode::Snap45,
            rotation_speed_deg: 180.0,
            min_slide_angle_deg: 22.5,
            max_slides: 7,
            default_controls: false,
        },
    ] {
        assert_eq!(TopDownPlayer::from_law(l).law(), l);
    }
}

#[test]
fn o_default_do_componente_e_o_default_da_LEI() {
    // ⛔ Uma segunda cópia dos defaults aqui divergiria da lei no dia em que um
    // deles mudasse — o defeito que o botão do remesh pagou noutro módulo.
    assert_eq!(TopDownPlayer::default().law(), TopDownLaw::default());
}

#[test]
fn os_defaults_medidos_sao_os_do_oraculo() {
    let d = TopDownPlayer::default();
    assert_eq!(d.min_slide_angle_deg, 15.0, "o limiar e' o do oraculo");
    assert_eq!(d.max_slides, 4, "o tecto de deslizes e' o do oraculo");
    assert!(d.default_controls, "ele anda no primeiro clique");
    assert_eq!(
        viewpoint::from_wire(d.viewpoint),
        Viewpoint::TopDown,
        "a isometria nasce DESLIGADA: liga-la por omissao mudaria toda cena"
    );
}

#[test]
fn o_componente_atravessa_o_postcard() {
    let a = TopDownPlayer {
        speed: 3.5,
        direction_mode: direction::to_wire(DirectionMode::AxisY),
        ..TopDownPlayer::default()
    };
    let bytes = postcard::to_stdvec(&a).expect("serializa");
    let b: TopDownPlayer = postcard::from_bytes(&bytes).expect("desserializa");
    assert_eq!(a, b);
}
