//! Os portões da porta de bissecção da altura de câmara — as células da lei, sem o ambiente.

use super::altura_por;
use ph2d_ecs::camera_2d::{CAMERA_MAX_HEIGHT_WORLD, CAMERA_MIN_HEIGHT_WORLD};

/// ⭐ **A metade que importa: quem não pede nada recebe o arranque de sempre.** Ausente, vazio,
/// lixo, não finito, zero e negativo são todos `None` — e `None` é *não tocar na câmara*.
#[test]
fn sem_pedido_legivel_a_camara_nao_e_tocada() {
    for v in [
        None,
        Some(""),
        Some("  "),
        Some("abc"),
        Some("NaN"),
        Some("inf"),
        Some("0"),
        Some("-3"),
    ] {
        assert_eq!(
            altura_por(v),
            None,
            "{v:?} tinha de deixar a câmara como nasce"
        );
    }
}

/// O número pedido passa, e a faixa é a MESMA que o zoom do artista respeita — nunca uma vista
/// que a roda do rato não alcança.
#[test]
fn a_altura_pedida_passa_dentro_da_faixa_da_camara() {
    assert_eq!(altura_por(Some(" 24 ")), Some(24.0));
    assert_eq!(altura_por(Some("0.01")), Some(CAMERA_MIN_HEIGHT_WORLD));
    assert_eq!(altura_por(Some("1e6")), Some(CAMERA_MAX_HEIGHT_WORLD));
}
