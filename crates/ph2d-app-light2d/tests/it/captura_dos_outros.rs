//! **Os outros que bloqueiam** (W5e): a camada de uma família (o traço do Flip, as formas do Motion),
//! lida EXATA na resolução da captura, dá a sombra do polígono equivalente — a régua da camada do vetor
//! (`captura_do_vetor`). A família aqui é a de discos de ÁREA EXATA (`DiscosExatos`); que o Flip e o
//! Motion desenham essa área prova-o o contrato de cada um. ⛔ Sem a família (o controlo) nada bloqueia.

use bevy_ecs::world::World;
use ph2d_app_light2d::LuzDoQuadro;
use ph2d_ecs::{GlobalTransform, PresentWorld, Transform};
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, Oclusor, TipoDeLuz};
use ph2d_light2d_gpu::Solver;
use ph2d_render::Camera2d;
use vello::kurbo::Affine;

use crate::comum::{fluencia_lida, gpu, renderer};
use crate::o_traco_do_flip::DiscosExatos;

const ALVO: (u32, u32) = (256, 128);
const ALTURA_M: f32 = 4.0;
const CENTRO: [f64; 2] = [0.1, -0.1];
const RAIO: f64 = 0.4;

/// **Tolerância** — MEDIDA + 10 % (RTX 5060 Ti, `ALTO`): RMSE relativo `0,0033` atrás do disco; lida
/// BILINEAR a mesma camada mede `0,0066` (a mutação M11 da W5e); sem a família `0,92`.
const TOL: f64 = 0.0037;

fn afim() -> Affine {
    Camera2d::new([0.0, 0.0], ALTURA_M).world_to_screen_affine(WindowSize::new(ALVO.0, ALVO.1))
}

#[test]
fn a_camada_dos_outros_faz_a_sombra_do_seu_poligono() {
    let Some(gpu) = gpu() else {
        return;
    };
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
    let e = w
        .spawn((Transform::IDENTITY, BloqueiaLuz { opacidade: 1.0 }))
        .id();
    let r = renderer(&gpu);
    let mut exatos = DiscosExatos {
        entidade: e.to_bits(),
        discos: vec![(CENTRO, RAIO, [0, 0, 0])],
    };
    let mut correr = |com_familia: bool| {
        let mut luz = LuzDoQuadro::com_degrau(crate::comum::ALTO);
        assert!(luz.preparar(&w, afim(), ALVO));
        if com_familia {
            luz.gravar_com(&gpu, &mut PresentWorld::new(), &r, &mut [&mut exatos]);
        } else {
            luz.gravar(&gpu, &mut PresentWorld::new(), &r);
        }
        let s = luz.solver().expect("solver");
        (fluencia_lida(&gpu, s), *s.config(), luz.cena().clone())
    };
    let (com, cfg, cena) = correr(true);
    let (sem, _, _) = correr(false);

    // A régua: o MESMO disco como polígono de 256 lados, no mundo.
    let mut poligono = cena.clone();
    poligono.oclusores = vec![Oclusor {
        vertices: (0..256)
            .map(|k| {
                let a = std::f64::consts::TAU * f64::from(k) / 256.0;
                [
                    (CENTRO[0] + RAIO * a.cos()) as f32,
                    (CENTRO[1] + RAIO * a.sin()) as f32,
                ]
            })
            .collect(),
        opacidade: 1.0,
        albedo: [0.0; 3],
    }];
    let mut sol = Solver::novo(&gpu, cfg).expect("solver");
    let (ini, fim) = ph2d_app_light2d::regiao_da_janela(sol.grade_da_fluencia(), afim(), ALVO);
    sol.regiao_fina(ini, fim);
    sol.quadro(&gpu, &poligono);
    let crate::comum::Fluencia {
        v: referencia,
        grade: g,
    } = fluencia_lida(&gpu, &sol);
    let rmse = |a: &[[f32; 3]]| {
        let (mut e2, mut soma, mut n) = (0.0_f64, 0.0_f64, 0u32);
        for (k, (x, y)) in a.iter().zip(&referencia).enumerate() {
            let (i, j) = (k as u32 % g.largura, k as u32 / g.largura);
            if !(ini[0]..fim[0]).contains(&i) || !(ini[1]..fim[1]).contains(&j) {
                continue;
            }
            // Atrás do disco (a sombra e a penumbra dele).
            if g.origem_m[0] + (f64::from(i) + 0.5) * g.texel_m < CENTRO[0] + RAIO {
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
    let (com, sem) = (rmse(&com.v), rmse(&sem.v));
    eprintln!("outros: disco × polígono — RMSE relativo {com:.4} (sem a família: {sem:.4})");
    assert!(
        com <= TOL,
        "a camada dos outros não faz a sombra do polígono: {com:.4}"
    );
    assert!(
        sem > 5.0 * TOL,
        "controlo: sem a família nada devia bloquear ({sem:.4})"
    );
}
