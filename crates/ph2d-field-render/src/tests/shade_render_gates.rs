//! ⭐ **Os gates do modo RENDER** — a costura entre o G-buffer, a câmera, a lei do material e o olhar.
//!
//! ⚠️ A lei do material tem o oráculo dela (`ph2d-material`), e o olhar o dele
//! (`ph2d-view-transform`). O que só se prova AQUI é que o traçador os liga: a normal certa, a
//! direcção de vista certa, o olhar por amostra, e o fundo intocado.

use super::*;
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
        sky: &sky,
    };
    for look in [
        Look::default(),
        Look {
            exposure_stops: -1.5,
            view: ViewTransform::Neutral,
        },
    ] {
        let px = shade_render(&g, &cam, &surface, &light, look, BG);
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

/// ⏱️ **SONDA (`--ignored`): o que o modo RENDER custa, e que luz ele devolve.**
///
/// ⚠️ Ela **não é um gate** — não há barra aqui. É a medição que responde *«isto ainda é
/// interactivo?»* e *«a peça sai preta ou estourada?»* antes de o dono olhar para ela.
#[test]
#[ignore = "sonda de medição"]
fn measure_what_the_render_mode_costs_and_paints() {
    use std::time::Instant;
    let (w, h) = (640, 360);
    let cam = Orbit::default();
    let doc = sphere(0.6);
    let reg = Registry::new();
    let t = Instant::now();
    let g = trace(&doc, &reg, &cam, w, h);
    let trace_ms = t.elapsed().as_secs_f64() * 1e3;

    let texels = vec![0.5_f32; 2 * 2 * 3];
    let m = Matcap {
        side: 2,
        rgb_linear: &texels,
    };
    let t = Instant::now();
    let matcap_px = shade(&g, &m, BG);
    let matcap_ms = t.elapsed().as_secs_f64() * 1e3;

    // O rig e o céu de omissão do modelador, como o `render_light` os traduz: a chave a `π`, o céu
    // do estúdio aproximado pela média dele.
    let surface = OpenPbr::default().prepare();
    let lamps = [Lamp {
        to_light: [-0.558, 0.663, 0.5],
        radiance: [std::f32::consts::PI; 3],
    }];
    let sky = Sky([0.35, 0.37, 0.42]);
    let light = Lighting {
        lamps: &lamps,
        sky: &sky,
    };
    let t = Instant::now();
    let render_px = shade_render(&g, &cam, &surface, &light, Look::default(), BG);
    let render_ms = t.elapsed().as_secs_f64() * 1e3;

    let stats = |px: &[u8]| -> (f64, u32, usize) {
        let (mut sum, mut saturated, mut n) = (0.0_f64, 0_u32, 0_usize);
        for (i, p) in px.as_chunks::<4>().0.iter().enumerate() {
            if !g.hit[i] {
                continue;
            }
            n += 1;
            sum += f64::from(p[1]);
            if p[1] == 255 {
                saturated += 1;
            }
        }
        (sum / n as f64, saturated, n)
    };
    let (m_mean, m_sat, pixels) = stats(&matcap_px);
    let (r_mean, r_sat, _) = stats(&render_px);
    println!("esfera {w}x{h} · {pixels} pixels de peça · traçado {trace_ms:.1} ms");
    println!("matcap: {matcap_ms:.2} ms · verde médio {m_mean:.1} · saturados {m_sat}");
    println!("render: {render_ms:.2} ms · verde médio {r_mean:.1} · saturados {r_sat}");
    for stops in [-2.0, -1.0, 0.0, 1.0, 2.0] {
        let px = shade_render(
            &g,
            &cam,
            &surface,
            &light,
            Look {
                exposure_stops: stops,
                view: ViewTransform::Standard,
            },
            BG,
        );
        let neutral = shade_render(
            &g,
            &cam,
            &surface,
            &light,
            Look {
                exposure_stops: stops,
                view: ViewTransform::Neutral,
            },
            BG,
        );
        let (s_mean, s_sat, _) = stats(&px);
        let (n_mean, n_sat, _) = stats(&neutral);
        println!(
            "  {stops:+.0} stop · Standard média {s_mean:6.1} saturados {s_sat:6} · \
             Neutral média {n_mean:6.1} saturados {n_sat:6}"
        );
    }
}

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
