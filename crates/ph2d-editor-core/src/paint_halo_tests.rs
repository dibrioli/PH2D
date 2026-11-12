//! O halo dos overlays: a tinta sai sempre depois do halo, mais larga, escura e com o mesmo tracejado.

use super::*;
use ph2d_tokens::ColorToken;
use ph2d_vector::{Circle, DrawOp};
use std::cell::RefCell;

const TEMAS: [Theme; 2] = [Theme::Forge, Theme::Sunstone];

fn cor(theme: Theme) -> Color {
    crate::paint::resolve(ColorToken::Warn, theme)
}

#[test]
fn o_traco_sai_sobre_o_halo_mais_largo_com_o_mesmo_tracejado() {
    for theme in TEMAS {
        let mut scene = VectorScene::new();
        scene.record();
        let traco = Stroke::new(2.0).with_dashes(0.0, [6.0, 3.0]);
        let c = cor(theme).with_alpha(0.5);
        stroke_com_halo(&mut scene, &traco, c, &Circle::new((5.0, 5.0), 4.0), theme);
        let ops = scene.take_record();
        let [
            DrawOp::Stroke {
                style: h,
                brush: Brush::Solid(hc),
                ..
            },
            DrawOp::Stroke {
                style: t,
                brush: Brush::Solid(tc),
                ..
            },
        ] = ops.as_slice()
        else {
            panic!("halo e traço, nesta ordem: {ops:?}");
        };
        assert_eq!(*tc, c);
        assert_eq!(*hc, cor_do_halo(c, theme));
        assert!(h.width > t.width, "o halo é mais largo");
        assert_eq!(h.dash_pattern, t.dash_pattern, "o mesmo tracejado");
        let (sombra, _) = crate::paint_flight::sombra_de_alca(theme);
        assert!(
            (hc.components[3] - sombra.components[3] * 0.5).abs() < 1e-6,
            "o alfa da tinta passa ao halo"
        );
    }
}

#[test]
fn o_ponto_cheio_sai_sobre_o_anel_do_halo() {
    for theme in TEMAS {
        let mut scene = VectorScene::new();
        scene.record();
        fill_com_halo(&mut scene, cor(theme), &Circle::new((5.0, 5.0), 3.0), theme);
        let ops = scene.take_record();
        assert!(
            matches!(ops.as_slice(), [DrawOp::Stroke { .. }, DrawOp::Fill { .. }]),
            "anel do halo e depois o ponto: {ops:?}"
        );
    }
}

thread_local! {
    static TEXTOS: RefCell<Vec<[f32; 4]>> = const { RefCell::new(Vec::new()) };
}

fn ouve(_t: &str, r: [f32; 4], _corpo: f32) {
    TEXTOS.with(|v| v.borrow_mut().push(r));
}

#[test]
fn o_texto_sai_por_cima_das_oito_copias_do_halo() {
    let mut ts = TextSystem::new();
    let mut scene = VectorScene::new();
    TEXTOS.with(|v| v.borrow_mut().clear());
    crate::paint::ouve_o_texto(Some(ouve));
    let rect = Rect::new(10.0, 10.0, 100.0, 14.0);
    paint_text_centered_com_halo(
        &mut ts,
        &mut scene,
        "Full jump",
        rect,
        11.0,
        cor(Theme::Sunstone),
        Theme::Sunstone,
    );
    crate::paint::ouve_o_texto(None);
    let r = TEXTOS.with(|v| v.borrow().clone());
    assert_eq!(r.len(), 9, "oito cópias do halo e o texto: {r:?}");
    let ultimo = r[8];
    let g = largura_do_halo() as f32;
    for (i, (dx, dy)) in DIRECOES_DO_HALO.iter().enumerate() {
        assert!(
            (r[i][0] - ultimo[0] - dx * g).abs() < 1e-3
                && (r[i][1] - ultimo[1] - dy * g).abs() < 1e-3,
            "a cópia {i} está deslocada {dx},{dy}: {:?} × {ultimo:?}",
            r[i]
        );
    }
}
