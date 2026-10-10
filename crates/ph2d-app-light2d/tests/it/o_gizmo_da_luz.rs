//! ⭐ **O gizmo da luz** (smoke do dono, 2026-10-09: *«luz sem gizmo»*) — a geometria que se pinta.

use ph2d_app_light2d::gizmo::{LAMPADA_MIN_PX, glifo};
use ph2d_light2d_ecs::Luz;

/// A lâmpada no tamanho REAL (o que decide a penumbra), com piso; os raios FORA do anel e da
/// lâmpada; o alcance só na selecionada e finita; a Ambiente sem lâmpada nem alcance.
#[test]
fn o_glifo_mostra_o_tamanho_os_raios_e_o_alcance() {
    let mut luz = Luz {
        alcance_m: 3.0,
        ..Luz::default()
    };
    let g = glifo(&luz, [100.0, 50.0], 100.0, 24.0, false);
    assert!(
        (g.lampada_px - 25.0).abs() < 1e-4,
        "0,25 m a 100 px/m = 25 px: {}",
        g.lampada_px
    );
    assert_eq!(g.raios.len(), 8);
    for (a, b) in &g.raios {
        let (ra, rb) = (
            (a[0] - 100.0).hypot(a[1] - 50.0),
            (b[0] - 100.0).hypot(b[1] - 50.0),
        );
        assert!(
            ra > 25.0 && rb > ra,
            "o raio começa fora da lâmpada e do anel: {ra} {rb}"
        );
    }
    assert_eq!(g.alcance_px, None, "o alcance só aparece na selecionada");
    assert_eq!(
        glifo(&luz, [0.0; 2], 100.0, 24.0, true).alcance_px,
        Some(300.0)
    );
    luz.alcance_m = f32::INFINITY;
    assert_eq!(
        glifo(&luz, [0.0; 2], 100.0, 24.0, true).alcance_px,
        None,
        "∞ não se desenha"
    );
    luz.tamanho_m = 0.001;
    assert_eq!(
        glifo(&luz, [0.0; 2], 100.0, 24.0, false).lampada_px,
        LAMPADA_MIN_PX
    );
    let amb = glifo(&Luz::ambiente(), [0.0; 2], 100.0, 24.0, true);
    assert_eq!((amb.lampada_px, amb.alcance_px), (0.0, None));
    assert!(amb.raios.len() > 8, "a Ambiente lê-se diferente da lâmpada");
    assert_eq!(g.cor, [255; 3], "a cor da luz em sRGB");
    // Uma lâmpada MAIOR que o anel: os raios começam fora dela, não dentro.
    let grande = Luz {
        tamanho_m: 0.6,
        ..Luz::default()
    };
    let gg = glifo(&grande, [0.0; 2], 100.0, 24.0, false);
    assert!(
        gg.raios.iter().all(|(a, _)| a[0].hypot(a[1]) > 60.0),
        "os raios de uma lâmpada de 60 px começam dentro dela"
    );
}
