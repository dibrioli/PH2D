//! ⭐⭐⭐ **O oclusor FINO faz a sombra do gabarito** (F2 da fila do dono, `docs/Luz2D/01_plano.md`
//! §7.1: «em Low, objetos muito finos — ~5 cm na vista de fábrica — não fazem sombra»). Uma barra de
//! 5 cm (0,8 texel grosso no Low) nas DUAS orientações: fina no eixo PERPENDICULAR aos raios (a
//! sombra é uma lasca que alarga) e fina AO LONGO deles (a sombra é larga; o raio atravessa só 5 cm).
//! A régua: a MASSA da sombra (∫ 1 − luz com ÷ luz sem, através dela) e o fundo dela, placa ×
//! gabarito, pelo caminho do PRODUTO em cada nível.

use bevy_ecs::world::World;
use ph2d_app_light2d::{Degrau, LuzDoQuadro};
use ph2d_ecs::{GlobalTransform, Name, Transform};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, TipoDeLuz};
use ph2d_render::{Camera2d, Sprite, WHITE_TILE_KEY};
use vello::kurbo::Affine;

use crate::comum::{Fluencia, fluencia_lida};

const FHD: (u32, u32) = (1920, 1080);
const LAMPADA: [f32; 2] = [-3.0, 0.0];
/// A barra: o centro e os lados (`x` ao longo dos raios, `y` através deles).
const BARRA: [f32; 2] = [-1.5, 0.0];
const FINO_M: f32 = 0.05;

fn afim() -> Affine {
    Camera2d::default().world_to_screen_affine(WindowSize::new(FHD.0, FHD.1))
}

fn pose(x: f32, y: f32) -> (Transform, GlobalTransform) {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    (t, GlobalTransform::from_transform(t))
}

/// As lâmpadas da régua: o ponto e a pequena das cenas.
pub(crate) const LAMPADAS: [(&str, f32); 2] = [("ponto", 0.0), ("lâmpada pequena", 0.15)];

/// A lâmpada (`raio` 0 = ponto) e, com `lados`, a barra que bloqueia — nascidas como o PRODUTO as
/// cria (só `Transform`).
pub(crate) fn mundo(raio: f32, lados: Option<[f32; 2]>) -> World {
    let mut w = World::new();
    let (t, g) = pose(LAMPADA[0], LAMPADA[1]);
    w.spawn((
        Name::new("Lâmpada"),
        Luz {
            cor: [1.0; 3],
            intensidade: std::f32::consts::TAU * 2.0,
            tamanho_m: raio,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        g,
    ));
    if let Some(lados) = lados {
        let (t, g) = pose(BARRA[0], BARRA[1]);
        w.spawn((
            Name::new("Barra"),
            Sprite::atlas(WHITE_TILE_KEY, lados, [0.3, 0.3, 0.3, 1.0]),
            t,
            g,
            BloqueiaLuz { opacidade: 1.0 },
        ));
    }
    w
}

fn luz(gpu: &GpuContext, degrau: Degrau, w: &World) -> Fluencia {
    let r = crate::comum::renderer(gpu);
    let mut l = LuzDoQuadro::com_degrau(degrau);
    assert!(l.preparar(w, afim(), FHD));
    l.gravar(gpu, &mut ph2d_ecs::PresentWorld::new(), &r);
    fluencia_lida(gpu, l.solver().expect("solver"))
}

/// A sombra através de `x` (de `y0` a `y1`, a cada ¼ px): `(massa em m, fundo)`.
pub(crate) fn sombra(
    f: impl Fn([f64; 2]) -> f64,
    x: f64,
    (y0, y1): (f64, f64),
    ppm: f64,
) -> (f64, f64) {
    let dy = 0.25 / ppm;
    let n = ((y1 - y0) / dy) as usize;
    let mut massa = 0.0;
    let mut fundo = f64::INFINITY;
    for k in 0..=n {
        let v = f([x, y0 + k as f64 * dy]);
        massa += (1.0 - v).max(0.0) * dy;
        fundo = fundo.min(v);
    }
    (massa, fundo)
}

/// Uma orientação: os lados da barra e onde a régua corta a sombra.
pub(crate) struct Caso {
    pub nome: &'static str,
    pub lados: [f32; 2],
    pub xs: [f64; 3],
    pub ys: (f64, f64),
}

pub(crate) const CASOS: [Caso; 2] = [
    Caso {
        nome: "fina através dos raios (lasca)",
        lados: [1.0, FINO_M],
        xs: [-0.5, 0.5, 1.5],
        ys: (-0.4, 0.4),
    },
    Caso {
        nome: "fina ao longo dos raios (larga)",
        lados: [FINO_M, 0.6],
        xs: [-0.8, 0.0, 1.0],
        ys: (-1.2, 1.2),
    },
];

/// O MEDIDO + 10 % (rodada `medida_luz2d_w5a`, os três níveis): massa 0,026 · fundo 0,030. Antes (a
/// captura das cascatas) o Low dava 1,000 · 1,000 — a barra não fazia sombra; com a captura de 1 px a
/// massa errava 0,113 (a lasca do ponto alargava) e, sem o refino pela CURVATURA, o fundo 0,257.
const TETO_MASSA: f64 = 0.029;
const TETO_FUNDO: f64 = 0.033;

/// ⭐⭐⭐ **GATE F2 — o oclusor de 5 cm faz a sombra do gabarito em todo nível.**
#[test]
fn o_oclusor_fino_faz_a_sombra_do_gabarito() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let [a, b, ..] = afim().as_coeffs();
    let ppm = a.hypot(b);
    let mut falhas = Vec::new();
    for (lnome, raio) in LAMPADAS {
        let sem = mundo(raio, None);
        let gab_sem = cena_de_luz(&sem);
        for caso in &CASOS {
            let com = mundo(raio, Some(caso.lados));
            let gab_com = cena_de_luz(&com);
            let gab = |p: [f64; 2]| {
                ph2d_light2d_ref::fluencia(&gab_com, p)[1]
                    / ph2d_light2d_ref::fluencia(&gab_sem, p)[1].max(1e-12)
            };
            for (nome, degrau) in crate::a_escada_da_sombra::degraus() {
                let (fc, fs) = (luz(&gpu, degrau, &com), luz(&gpu, degrau, &sem));
                let placa = |p: [f64; 2]| fc.amostra(p) / fs.amostra(p).max(1e-12);
                for x in caso.xs {
                    let (mp, fp) = sombra(placa, x, caso.ys, ppm);
                    let (mg, fg) = sombra(gab, x, caso.ys, ppm);
                    eprintln!(
                        "F2 · {lnome} · {} · {nome} · x {x}: massa {mp:.4} × {mg:.4} m · fundo {fp:.3} × {fg:.3}",
                        caso.nome
                    );
                    assert!(
                        mg > 0.005,
                        "controlo: o gabarito tem sombra em x {x} ({mg})"
                    );
                    if (mp - mg).abs() > TETO_MASSA * mg || (fp - fg).abs() > TETO_FUNDO {
                        falhas.push(format!(
                            "{lnome} · {} · {nome} · x {x}: massa {mp:.4} × {mg:.4} m (teto {TETO_MASSA}) · fundo {fp:.3} × {fg:.3} (teto {TETO_FUNDO})",
                            caso.nome
                        ));
                    }
                }
            }
        }
    }
    assert!(
        falhas.is_empty(),
        "F2: a barra de 5 cm não faz a sombra do gabarito:\n{}",
        falhas.join("\n")
    );
}
