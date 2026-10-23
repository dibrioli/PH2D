//! **A régua do custo dos cantos nas 7 câmeras** (handoff W6g §55 b / §56.1 — o alvo «C3 ≤ +5 % sobre a lei
//! W6d» em TODA câmera das cenas 5 e 6). À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_cantos_custo -- --ignored --nocapture --test-threads=1
//! ```
//!
//! O molde do `custo` da `medida_luz2d_w6d` (FHD, `DEGRAU_PADRAO`, sem rebote, a cena com os distantes que
//! tiver): por câmera, as [`VARIANTES`] num processo, [`RODADAS`] intercaladas em ordem rodada, o MÍNIMO da C3
//! (preenche + visibilidade + fina) e as parcelas da rodada mínima. A lei W6d entra duas vezes — o 2.º
//! solver é o controlo do ruído. As câmeras correm uma de cada vez (7 solvers vivos, não 49).

use ph2d_app_light2d::smoke;
use ph2d_light2d_gpu::{Canto, Config, VERTICE_PADRAO};
use ph2d_render::Camera2d;
use vello::kurbo::Affine;

use crate::as_formas_redondas::config;
use crate::comum::gpu;
use crate::medida_luz2d_w4b::loadavg;
use crate::medida_luz2d_w5d::{FHD, solver, vista};

const RODADAS: usize = 9;

#[derive(Clone, Copy)]
enum Camera {
    Zoom(f64),
    /// O centro em metros e a altura do mundo à vista (a de fábrica: 10 m).
    Centro([f32; 2], f32),
}

/// As 7 do §55 b: (cena, câmera).
const CAMERAS: [(u32, Camera); 7] = [
    (5, Camera::Centro([-6.0, -1.0], 2.5)),
    (5, Camera::Centro([6.0, -1.0], 2.5)),
    (5, Camera::Zoom(1.6)),
    (5, Camera::Zoom(1.0)),
    (6, Camera::Centro([0.0, -3.0], 1.5)),
    (6, Camera::Zoom(4.0)),
    (6, Camera::Zoom(1.0)),
];

const VARIANTES: [&str; 7] = [
    "lei W6d",
    "cunha",
    "cunha + V2",
    "cunha + canto",
    "padrão",
    "padrão sem a luz do canto",
    "lei W6d (controlo)",
];

fn afim(c: Camera) -> Affine {
    match c {
        Camera::Zoom(z) => vista(z),
        Camera::Centro(centro, h) => Camera2d::new(centro, h)
            .world_to_screen_affine(ph2d_host::WindowSize::new(FHD.0, FHD.1)),
    }
}

fn nome(c: Camera) -> String {
    match c {
        Camera::Zoom(z) => format!("zoom {z}"),
        Camera::Centro([x, y], h) => format!("[{x}, {y}] h {h}"),
    }
}

fn variante(c: Config, v: usize) -> Config {
    let lei = Config {
        borda_vertice: None,
        ..c.sem_os_cantos()
    };
    let cunha = Config {
        borda_vertice: Some(VERTICE_PADRAO),
        ..c.sem_os_cantos()
    };
    match v {
        0 | 6 => lei,
        1 => cunha,
        2 => Config {
            borda_cantos: true,
            ..cunha
        },
        3 => Config {
            canto: Canto::LIGADO,
            ..cunha
        },
        4 => Config {
            borda_cantos: true,
            canto: Canto::LIGADO,
            ..cunha
        },
        _ => Config {
            borda_cantos: true,
            canto: Canto {
                luz: false,
                ..Canto::LIGADO
            },
            ..cunha
        },
    }
}

/// Os blocos 8×8 da captura com alguma cunha na `V2` (θ₁ < 50), o censo do `custo` da W6d.
fn blocos_com_cunha(gpu: &ph2d_gpu::GpuContext, s: &ph2d_light2d_gpu::Solver) -> Option<usize> {
    let (b, [lx, ly]) = s.cantos_lidos(gpu)?;
    let (bx, by) = (lx.div_ceil(8), ly.div_ceil(8));
    let mut com = vec![false; (bx * by) as usize];
    for j in 0..ly {
        for i in 0..lx {
            let k = (j * lx + i) as usize * 8;
            if half::f16::from_le_bytes([b[k], b[k + 1]]).to_f32() < 50.0 {
                com[(j / 8 * bx + i / 8) as usize] = true;
            }
        }
    }
    Some(com.iter().filter(|&&c| c).count())
}

#[test]
#[ignore = "a régua do custo dos cantos: corre-se à mão (cabeçalho)"]
fn medida_luz2d_cantos_custo() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    eprintln!(
        "# Régua do custo dos cantos · {} · perfil {} · loadavg antes {}",
        gpu.adapter.get_info().name,
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "otimizado"
        },
        loadavg()
    );
    for (n, cam) in CAMERAS {
        let mut w = bevy_ecs::world::World::new();
        assert!(smoke::encenar(&mut w, n));
        let cena = ph2d_light2d_ecs::coleta::cena_de_luz(&w);
        let a = afim(cam);
        let base = config(&gpu, a, false);
        let mut solvers: Vec<_> = (0..VARIANTES.len())
            .map(|v| solver(&gpu, a, variante(base, v)))
            .collect();
        let mut minimo = vec![f64::INFINITY; solvers.len()];
        let mut partes_min = vec![[0.0; 4]; solvers.len()];
        for rodada in 0..RODADAS {
            for j in 0..solvers.len() {
                let i = (j + rodada) % solvers.len();
                for _ in 0..3 {
                    solvers[i].quadro(&gpu, &cena);
                }
                let t = solvers[i].tempos(&gpu).expect("TIMESTAMP_QUERY");
                let c3 = t.c3_preenche + t.c3_visibilidade + t.c3_fina;
                if c3 < minimo[i] {
                    minimo[i] = c3;
                    partes_min[i] = [t.c3_preenche, t.c3_borda, t.c3_visibilidade, t.c3_fina];
                }
            }
        }
        let distantes = solvers[0].quadro(&gpu, &cena).distantes;
        let blocos = blocos_com_cunha(&gpu, &solvers[4]);
        let recusados = solvers[4].cantos_recusados(&gpu);
        eprintln!(
            "CAMERA · cena {n} · {} · distantes {distantes} · blocos com cunha {blocos:?} · recusados {recusados:?} · loadavg {}",
            nome(cam),
            loadavg()
        );
        for (v, nome_v) in VARIANTES.iter().enumerate() {
            let [p, b, vis, f] = partes_min[v];
            eprintln!(
                "CUSTO · cena {n} · {} · {nome_v} · C3 {:.3} ms ({:+.1} % sobre a lei W6d) · preenche {p:.3} [borda {b:.3}] · visibilidade {vis:.3} · fina {f:.3}",
                nome(cam),
                minimo[v],
                100.0 * (minimo[v] / minimo[0] - 1.0)
            );
        }
    }
    eprintln!("# loadavg depois {}", loadavg());
}
