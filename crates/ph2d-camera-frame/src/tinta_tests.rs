//! A tinta da câmera no canvas.

use super::*;

fn moldura(selecionada: bool) -> Moldura {
    Moldura {
        camera: 1,
        nome: "Camera".into(),
        pose: [0.0, 0.0],
        centro: [0.0, 0.0],
        meia: [4.0, 2.25],
        ativa: true,
        selecionada,
        limites: None,
        mira: [0.0, 0.0],
        offset: [0.0, 0.0],
        segue: None,
    }
}

/// Quantas operações da cena pintam na cor da câmera (`ObjCamera`).
fn na_cor_da_camera(selecionada: bool) -> usize {
    use ph2d_vector::{Brush, DrawOp};
    let theme = Theme::default();
    let cor = ph2d_editor_core::paint::resolve(ColorToken::ObjCamera, theme);
    let mut cena = VectorScene::new();
    cena.record();
    desenhar(
        &[moldura(selecionada)],
        Pintura {
            mostrar: CameraFrameShow::Always,
            unidade: DisplayUnit::Meters,
            pixels_per_meter: 100.0,
            icone_px: 72.0,
            em_edicao: false,
            canvas: [0.0, 0.0, 1280.0, 720.0],
            ponteiro: (0.0, 0.0),
            escurecer: false,
            agarrada: None,
        },
        theme,
        &Camera2d::new([0.0, 0.0], 10.0),
        WindowSize {
            width: 1280,
            height: 720,
        },
        &mut cena,
        &mut TextSystem::without_system_fonts(),
    );
    cena.take_record()
        .into_iter()
        .filter(|op| {
            matches!(op,
                DrawOp::Stroke { brush: Brush::Solid(c), .. } | DrawOp::Fill { brush: Brush::Solid(c), .. }
                    if *c == cor)
        })
        .count()
}

/// ⭐ **A câmera selecionada acende o ícone na cor dela** (dono 08/11: «a câmera perdeu o destaque de cor do
/// ícone ao ser selecionada»): o laranja da Hierarchy; sem seleção, a tinta de sempre.
///
/// (Mutação: `cor = selecao` sempre ⇒ RED.)
#[test]
fn a_camera_selecionada_pinta_o_icone_na_cor_dela() {
    assert!(
        na_cor_da_camera(true) > 0,
        "selecionada sem a cor da câmera"
    );
    assert_eq!(
        na_cor_da_camera(false),
        0,
        "sem seleção, já na cor da câmera"
    );
}
