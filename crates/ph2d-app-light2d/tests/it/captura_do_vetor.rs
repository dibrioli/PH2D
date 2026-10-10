//! **O vetor que bloqueia** (W3): um caminho do documento com `BloqueiaLuz`, desenhado pelo Vello na
//! grade (a MESMA porta do tile de uma forma), dá a sombra do polígono equivalente — a régua da C0 da
//! W2. ⛔ Sem a camada vetorial (o controlo) o caminho não bloqueia nada.

use bevy_ecs::world::World;
use ph2d_app_light2d::{Degrau, LuzDoQuadro};
use ph2d_ecs::{GlobalTransform, PresentWorld, Transform, VecPathRef};
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, Oclusor, TipoDeLuz};
use ph2d_light2d_gpu::Solver;
use ph2d_render::Camera2d;
use ph2d_vec_scene::{VecScene, VecXforms, Xform, rectangle};
use vello::kurbo::Affine;

use crate::comum::{fluencia_lida, gpu, renderer};

const ALVO: (u32, u32) = (256, 128);
const ALTURA_M: f32 = 4.0;

/// **Tolerância MEDIDA** (RTX 5060 Ti, Vulkan, 2026-10-09): RMSE relativo da fluência atrás do
/// caminho, medido `0,0000` no degrau `ALTO` das leis — a camada a 1 px por texel lida EXATA dá a
/// cobertura de área do Vello, a mesma que as 8×8 subamostras do polígono (o piso é o `f16`); sem a
/// camada: `1,38`. Lida bilinear (a 1.ª redacção) media `0,0074` a 2 px por texel.
/// ⚠️ Desde a luz DIRETA (C3, W4) mede `0,0043`: as cascatas borravam a diferença de captura entre o
/// Vello (cobertura de área) e as 8×8 subamostras do polígono; o caminho direto é nítido (sem a
/// camada: `1,39`, 320× acima).
const TOL: f64 = 0.006;

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
        let s = luz.solver().expect("solver");
        (fluencia_lida(gpu, s), *s.config(), luz.cena().clone())
    };
    let (com, cfg, cena) = correr(true);
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
        albedo: [0.0; 3],
    }];
    // A MESMA configuração do quadro (grade, grade fina): só a forma do oclusor muda.
    let mut sol = Solver::novo(gpu, cfg).expect("solver");
    let (ini, fim) = ph2d_app_light2d::regiao_da_janela(sol.grade_da_fluencia(), afim(), ALVO);
    sol.regiao_fina(ini, fim);
    sol.quadro(gpu, &poligono);
    let crate::comum::Fluencia {
        v: referencia,
        grade: g,
    } = fluencia_lida(gpu, &sol);

    let rmse = |a: &[[f32; 3]]| {
        let (mut e2, mut soma, mut n) = (0.0_f64, 0.0_f64, 0u32);
        for (k, (x, y)) in a.iter().zip(&referencia).enumerate() {
            let (i, j) = (k as u32 % g.largura, k as u32 / g.largura);
            // Só a região da janela (a única que o quadro calcula), atrás do caminho.
            if !(ini[0]..fim[0]).contains(&i) || !(ini[1]..fim[1]).contains(&j) {
                continue;
            }
            if g.origem_m[0] + (f64::from(i) + 0.5) * g.texel_m < 0.8 {
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
    (rmse(&com.v), rmse(&sem.v))
}

#[test]
fn um_caminho_vetorial_que_bloqueia_faz_a_sombra_do_seu_poligono() {
    let Some(gpu) = gpu() else {
        eprintln!("vetor: sem placa — saltado");
        return;
    };
    let (com, sem) = medir(&gpu, crate::comum::ALTO);
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
