//! **O vetor que bloqueia** (W3): um caminho do documento com `BloqueiaLuz`, desenhado pelo Vello na
//! grade (a MESMA porta do tile de uma forma), dá a sombra do polígono equivalente — a régua da C0 da
//! W2. ⛔ Sem a camada vetorial (o controlo) o caminho não bloqueia nada.

use bevy_ecs::world::World;
use ph2d_app_light2d::{DEGRAU_PADRAO, Degrau, LuzDoQuadro};
use ph2d_ecs::{GlobalTransform, PresentWorld, Transform, VecPathRef};
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, Oclusor, TipoDeLuz};
use ph2d_light2d_gpu::{Config, Solver};
use ph2d_render::Camera2d;
use ph2d_vec_scene::{VecScene, VecXforms, Xform, rectangle};
use vello::kurbo::Affine;

use crate::comum::{fluencia_lida, gpu, renderer};

const ALVO: (u32, u32) = (256, 128);
const ALTURA_M: f32 = 4.0;

/// **Tolerância MEDIDA + 10 %** (RTX 5060 Ti, Vulkan, 2026-10-09): RMSE relativo da fluência atrás
/// do caminho, medido `0,0074` com o degrau padrão (sem a camada: `1,38`).
const TOL: f64 = 0.0082;

fn afim() -> Affine {
    Camera2d::new([0.0, 0.0], ALTURA_M).world_to_screen_affine(WindowSize::new(ALVO.0, ALVO.1))
}

/// `(rmse com a camada, rmse sem ela)` contra o polígono, para o degrau `d`.
pub fn medir(gpu: &ph2d_gpu::GpuContext, d: Degrau) -> (f64, f64) {
    let mut doc = VecScene::new();
    let id = doc.push_path(rectangle([-0.25, -0.75], [0.25, 0.75]));
    // Rodado 0,4 rad e posto em (0,1; −0,1): o afim do mundo que a shell publica.
    let (s, c) = 0.4_f64.sin_cos();
    let xf = [c, s, -s, c, 0.1, -0.1];
    let mut xforms = VecXforms::new();
    xforms.insert(id, Xform(xf));
    let mut w = World::new();
    let mut t = Transform::IDENTITY;
    t.translation.x = -2.5;
    t.translation.y = 0.1;
    w.spawn((
        Luz {
            cor: [1.0, 0.95, 0.85],
            intensidade: 8.0,
            tamanho_m: 0.2,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        GlobalTransform::from_transform(t),
    ));
    w.spawn((VecPathRef(id), BloqueiaLuz { opacidade: 1.0 }));
    let r = renderer(gpu);

    let correr = |com_camada: bool| {
        let mut luz = LuzDoQuadro::com_degrau(d);
        assert!(luz.preparar(&w, afim(), ALVO));
        if com_camada {
            luz.desenhar_vetores(gpu, &doc, &xforms);
        }
        luz.gravar(gpu, &mut PresentWorld::new(), &r);
        (
            fluencia_lida(gpu, luz.solver().expect("solver")),
            luz.grade().expect("acesa"),
            luz.cena().clone(),
        )
    };
    let (com, g, cena) = correr(true);
    let (sem, _, _) = correr(false);

    // A régua: o MESMO retângulo como polígono, no mundo.
    let canto = |x: f64, y: f64| [(c * x - s * y + 0.1) as f32, (s * x + c * y - 0.1) as f32];
    let mut poligono = cena.clone();
    poligono.oclusores = vec![Oclusor {
        vertices: vec![
            canto(-0.25, -0.75),
            canto(0.25, -0.75),
            canto(0.25, 0.75),
            canto(-0.25, 0.75),
        ],
        opacidade: 1.0,
    }];
    let mut sol = Solver::novo(gpu, Config::padrao_para(&gpu.device, g)).expect("solver");
    sol.quadro(gpu, &poligono);
    let referencia = fluencia_lida(gpu, &sol);

    let rmse = |a: &[[f32; 3]]| {
        let (mut e2, mut soma, mut n) = (0.0_f64, 0.0_f64, 0u32);
        for (k, (x, y)) in a.iter().zip(&referencia).enumerate() {
            let i = f64::from(k as u32 % g.largura);
            if g.origem_m[0] + (i + 0.5) * g.texel_m < 0.8 {
                continue;
            }
            for ch in 0..3 {
                let d = f64::from(x[ch]) - f64::from(y[ch]);
                e2 += d * d;
                soma += f64::from(y[ch]);
                n += 1;
            }
        }
        (e2 / f64::from(n)).sqrt() / (soma / f64::from(n))
    };
    (rmse(&com), rmse(&sem))
}

#[test]
fn um_caminho_vetorial_que_bloqueia_faz_a_sombra_do_seu_poligono() {
    let Some(gpu) = gpu() else {
        eprintln!("vetor: sem placa — saltado");
        return;
    };
    let (com, sem) = medir(&gpu, DEGRAU_PADRAO);
    eprintln!("vetor: caminho × polígono — RMSE relativo {com:.4} (sem a camada: {sem:.4})");
    assert!(
        com <= TOL,
        "o caminho vetorial não faz a sombra do polígono: {com:.4}"
    );
    assert!(
        sem > 5.0 * TOL,
        "controlo: sem a camada o caminho não devia bloquear ({sem:.4})"
    );
}
