//! Dono, 05/11: **girar de volta devolve o original.** O ângulo à mão era contínuo e a barra
//! arredondava: voltar a 0,4° lia «0» e reamostrava a imagem inteira.

use ph2d_pixel_core::Rect;
use ph2d_pixel_core::affine::{
    Affine, DragOpts, Handle, Pivot, Point, Resample, TURN_MAGNET_DEG, transform_image,
};
use ph2d_pixel_core::transform::Image;

const W: u32 = 23;
const H: u32 = 8;

/// A moldura levantada em (27, 36), com o pivô no centro, como `Solta::new`.
fn inicio() -> (Affine, Pivot) {
    (
        Affine::at(27.0, 36.0),
        Pivot {
            u: f64::from(W) / 2.0,
            v: f64::from(H) / 2.0,
        },
    )
}

/// Um arrasto da alça de giro de `graus` em torno do pivô, assentado como no app.
fn gira(m: Affine, p: Pivot, graus: f64) -> Affine {
    let c = m.apply(p.u, p.v);
    let braco = 200.0;
    let de = Point {
        x: c.x + braco,
        y: c.y,
    };
    let a = graus.to_radians();
    let para = Point {
        x: c.x + braco * a.cos(),
        y: c.y + braco * a.sin(),
    };
    let opt = DragOpts {
        keep_ratio: false,
        from_center: false,
        snap: false,
    };
    m.drag(p, (W, H), Handle::Rotate, (de, para), opt)
        .0
        .settle_turn(p)
}

fn imagem() -> Image {
    let data: Vec<u8> = (0..W * H)
        .flat_map(|i| {
            let v = u8::try_from((i * 37) % 251).expect("< 251");
            [v, 255 - v, v / 2, 255]
        })
        .collect();
    Image {
        w: W,
        h: H,
        bpp: 4,
        data,
    }
}

fn assenta(m: &Affine) -> (Rect, Image) {
    transform_image(&imagem(), &m.settle(), Resample::Fast, 0, None).expect("cabe")
}

#[test]
fn turning_back_by_hand_gives_back_the_original_cells() {
    let (m0, p) = inicio();
    let original = assenta(&m0);
    // A mão nunca volta exactamente: o erro cobre dentro da folga, nos dois sentidos.
    for erro in [0.0, 0.4, -0.4, 1.3, -1.3, 2.2, -2.2] {
        for ida in [7.0, 30.0, 45.0, 123.0, -60.0] {
            let longe = gira(m0, p, ida);
            assert_ne!(assenta(&longe), original, "{ida}°: o giro mudou a imagem");
            let volta = gira(longe, p, -ida + erro);
            assert_eq!(
                assenta(&volta),
                original,
                "ida {ida}°, volta com erro {erro}°: as células têm de ser as originais"
            );
        }
    }
}

#[test]
fn the_angle_by_hand_is_the_whole_degree_the_bar_shows() {
    let (m0, p) = inicio();
    let m = gira(m0, p, 30.4);
    let rot = m.decompose().rot;
    assert!((rot - 30.0).abs() < 1e-9, "30,4° vira 30°: {rot}");
    let m = gira(m0, p, 30.6);
    let rot = m.decompose().rot;
    assert!((rot - 31.0).abs() < 1e-9, "30,6° vira 31°: {rot}");
}

#[test]
fn a_right_angle_catches_the_hand_and_outside_the_margin_it_lets_go() {
    let (m0, p) = inicio();
    let m = gira(m0, p, 90.0 - TURN_MAGNET_DEG + 0.1);
    assert!((m.decompose().rot - 90.0).abs() < 1e-9, "cola nos 90°");
    let m = gira(m0, p, 90.0 - TURN_MAGNET_DEG - 0.6);
    assert!(
        (m.decompose().rot - 90.0).abs() > 1.0,
        "fora da folga não cola: {}",
        m.decompose().rot
    );
}

#[test]
fn the_pivot_stays_put_while_the_turn_settles() {
    let (m0, p) = inicio();
    let c0 = m0.apply(p.u, p.v);
    let c = gira(m0, p, 33.3).apply(p.u, p.v);
    assert!((c.x - c0.x).abs() < 1e-9 && (c.y - c0.y).abs() < 1e-9);
}
