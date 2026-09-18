//! ⭐ **Os gates do modo RENDER** — a costura entre o G-buffer, a câmera, a lei do material e o olhar.
//!
//! ⚠️ A lei do material tem o oráculo dela (`ph2d-material`), e o olhar o dele
//! (`ph2d-view-transform`). O que só se prova AQUI é que o traçador os liga: a normal certa, a
//! direcção de vista certa, o olhar por amostra, e o fundo intocado.

use super::*;
use crate::Surfaces;
use crate::shade_render::view_direction;
use ph2d_material::{Environment, OpenPbr};
use ph2d_view_transform::{Look, ViewTransform};

struct Sky([f32; 3]);

impl Environment for Sky {
    fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        self.0
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        self.0
    }
}

/// Uma fita de três pixels: peça · fundo · peça.
fn strip(normals: [[f32; 3]; 3]) -> Gbuffer {
    Gbuffer {
        width: 3,
        height: 1,
        hit: vec![true, false, true],
        normal: normals.to_vec(),
        // ⚠️ A fita é sombreada com lente ORTOGRÁFICA e material único: o ponto de mundo não entra
        // em conta nenhuma destes gates. Ver [`ph2d_field_render::Gbuffer::point`].
        point: vec![[0.0; 3]; 3],
        curvature: Vec::new(),
        edges: Vec::new(),
    }
}

const BG: [u8; 4] = [12, 34, 56, 200];

/// ⭐⭐ **Todo pixel de peça é a lei do material sob o olhar**, e o de fundo sai com os bytes pedidos.
///
/// ⚠️ **Com lente ORTOGRÁFICA de propósito**: aí a direcção para o observador é `(0, 0, 1)` por
/// definição, e o gate escreve-a sem pedir nada à câmera — é o que o impede de partilhar a conta com
/// o produto e ficar verde sobre ela.
#[test]
fn every_part_pixel_is_the_material_law_under_the_look() {
    let cam = Orbit {
        lens: Lens::Ortho,
        ..Orbit::default()
    };
    let normals = [[0.0, 0.0, 1.0], [0.0; 3], [0.6, 0.0, 0.8]];
    let g = strip(normals);
    let surface = OpenPbr {
        base_color: [0.7, 0.3, 0.1],
        ..OpenPbr::default()
    }
    .prepare();
    let lamps = [Lamp {
        to_light: [0.0, 0.6, 0.8],
        radiance: [2.0, 1.8, 1.6],
    }];
    let sky = Sky([0.2, 0.25, 0.3]);
    let light = Lighting {
        lamps: &lamps,
        points: &[],
        sky: &sky,
        shadows: None,
    };
    for look in [
        Look::default(),
        Look {
            exposure_stops: -1.5,
            view: ViewTransform::Neutral,
        },
    ] {
        // ⚠️ **Um material só, e sem `owners`** — é o caso de UM, que é o que este gate afirma: a
        // lei do material por pixel tem gate próprio (`the_two_shapes_wear_their_own_material`).
        let so = [surface];
        let px = shade_render(
            &g,
            &cam,
            &Surfaces {
                all: &so,
                owners: None,
            },
            &light,
            look,
            BG,
        );
        assert_eq!(
            px[4..8],
            BG,
            "o pixel de fundo não saiu com os bytes pedidos"
        );
        for x in [0usize, 2] {
            let (n, v) = (normals[x], [0.0, 0.0, 1.0]);
            let i = surface.indirect(n, v, &sky);
            let d = surface.direct(n, v, lamps[0].to_light, lamps[0].radiance);
            let e = surface.emission(n, v);
            let c = look.apply([0, 1, 2].map(|k| i[k] + d[k] + e[k]));
            let want = [
                ph2d_color::srgb::linear_to_srgb_byte(c[0]),
                ph2d_color::srgb::linear_to_srgb_byte(c[1]),
                ph2d_color::srgb::linear_to_srgb_byte(c[2]),
                255,
            ];
            assert_eq!(px[x * 4..x * 4 + 4], want, "pixel {x} com {look:?}");
        }
    }
}

/// ⭐⭐ **A direcção de vista segue o raio que traçou o pixel** — em perspectiva ela muda de pixel
/// para pixel, e é por isso que não pode ser uma constante.
///
/// ⚠️ O gate afirma a GEOMETRIA, e não os números da conta: no centro da imagem o olho está de
/// frente, e um pixel à esquerda vê o olho à sua DIREITA (e vice-versa).
#[test]
fn the_view_direction_follows_the_ray_that_traced_the_pixel() {
    let cam = Orbit::default();
    assert!(
        matches!(cam.lens, Lens::Perspective { .. }),
        "o controlo: a câmera de omissão é convergente"
    );
    let screen = Screen::new(5, 5, cam.half_extent);
    let unit = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let centre = view_direction(&cam, &screen, 2, 2);
    assert!(
        centre[0].abs() < 1.0e-5 && centre[1].abs() < 1.0e-5,
        "no centro o olho está de frente: {centre:?}"
    );
    let left = view_direction(&cam, &screen, 0, 2);
    let right = view_direction(&cam, &screen, 4, 2);
    let top = view_direction(&cam, &screen, 2, 0);
    assert!(
        left[0] > 0.0 && right[0] < 0.0,
        "esquerda {left:?} · direita {right:?}"
    );
    assert!(
        top[1] < 0.0,
        "um pixel de cima vê o olho por BAIXO: {top:?}"
    );
    for v in [centre, left, right, top] {
        assert!(
            (unit(v) - 1.0).abs() < 1.0e-5,
            "a direcção não é unitária: {v:?}"
        );
    }
    // E com a lente paralela a direcção é a mesma em todo pixel.
    let ortho = Orbit {
        lens: Lens::Ortho,
        ..cam
    };
    for (x, y) in [(0, 0), (4, 2), (2, 4)] {
        let v = view_direction(&ortho, &screen, x, y);
        assert!(
            v[0].abs() < 1.0e-6 && v[1].abs() < 1.0e-6 && (v[2] - 1.0).abs() < 1.0e-6,
            "{v:?}"
        );
    }
}

// ⛔⛔ **A SONDA DO CUSTO E DA LUZ MUDOU-SE PARA A `ph2d-app-field3d`** (auditoria de 2026-09-13),
// e a mudança é a correcção: daqui ela não alcança o [`crate::Lighting`] que o produto de facto
// monta (o rig e o céu vivem na `ph2d-app-field3d::render_light`), então escrevia a luz **à mão** —
// uma lâmpada com a direcção em literal e um céu **CONSTANTE**. ⇒ a tabela de luz do
// `docs/Render3d/05` §6 media um programa que o produto não corre. *Uma segunda cópia escrita à
// mão do valor que uma porta produz é a forma canónica de um número envelhecer sem ninguém ver.*
//
// Ela é agora `render_light_tests::measure_what_the_render_mode_costs_and_paints`.

/// ⭐ **O matcap responde ao MESMO olhar** — e o olhar de omissão é o quadro de sempre, ao byte.
#[test]
fn the_matcap_answers_the_same_look() {
    let texels = vec![0.5_f32; 2 * 2 * 3];
    let m = Matcap {
        side: 2,
        rgb_linear: &texels,
    };
    let g = strip([[0.0, 0.0, 1.0], [0.0; 3], [0.0, 0.0, 1.0]]);
    let always = shade(&g, &m, BG);
    assert_eq!(shade_with(&g, &m, Look::default(), BG), always);
    assert_eq!(
        always[0],
        ph2d_color::srgb::linear_to_srgb_byte(0.5),
        "o controlo: o matcap de omissão pinta a fotografia tal como ela é"
    );
    let darker = shade_with(
        &g,
        &m,
        Look {
            exposure_stops: -1.0,
            view: ViewTransform::Standard,
        },
        BG,
    );
    assert_eq!(
        darker[0],
        ph2d_color::srgb::linear_to_srgb_byte(0.25),
        "um stop abaixo é metade da luz"
    );
    assert_eq!(darker[4..8], BG, "o fundo não obedece ao olhar");
}
