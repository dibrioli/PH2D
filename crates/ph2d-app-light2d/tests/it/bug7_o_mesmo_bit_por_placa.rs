//! Diagnóstico (à mão) do bug 7, família 1 (`docs/Luz2D/BUGS_luz2d.md`): os dois «não muda um bit» que só a
//! NVIDIA cumpre — o produto contra ELE MESMO (o controlo do determinismo), contra a descida que lê sempre o R
//! de cima e contra todos os níveis por um cone; para os texels diferentes, o tamanho da diferença em ULPs do
//! f16 da fluência. Corre-se nas duas placas e, na Radeon, também com o otimizador do compilador desligado
//! (`ACO_DEBUG=noopt`): se a diferença morre aí, quem a faz é a reescrita da conta pelo compilador.
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-light2d --test it bug7_ -- --ignored --nocapture
//! ```

use bevy_ecs::world::World;
use half::f16;
use ph2d_app_light2d::{
    config_da_vista, config_do_entorno, grade_do_quadro, regiao_da_janela, smoke,
};
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_gpu::Solver;
use ph2d_render::Camera2d;

use crate::comum::{fluencia_lida, gpu};

const FHD: (u32, u32) = (1920, 1080);

#[test]
#[ignore = "diagnóstico do bug 7: corre-se à mão"]
fn bug7_o_mesmo_bit_por_placa() {
    use ph2d_app_light2d::qualidade::degrau_do;
    use ph2d_app_light2d::teto::{Tetos, fina_que_cabe};
    use ph2d_editor_core::luz_edits::NivelDaLuz;
    let Some(gpu) = gpu() else { return };
    let i = gpu.adapter.get_info();
    eprintln!(
        "bug7 · {} ({:?}) · ACO_DEBUG={:?}",
        i.name,
        i.backend,
        std::env::var("ACO_DEBUG").ok()
    );
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    let cena = cena_de_luz(&w);
    let (centro, alvo) = ([-4.2, -1.8], FHD);
    let a = Camera2d::new(centro, 3.8).world_to_screen_affine(WindowSize::new(alvo.0, alvo.1));
    let base = degrau_do(NivelDaLuz::Baixa);
    let k = fina_que_cabe(
        a,
        alvo,
        base,
        Tetos {
            lado: 8192,
            recusou: None,
        },
    )
    .expect("cabe");
    // (o nome, a fina engrossada?, salta o T nulo, um cone)
    let variantes: [(&str, bool, bool, Option<u32>); 6] = [
        ("T nulo · produto", true, true, None),
        ("T nulo · produto de novo", true, true, None),
        ("T nulo · lê sempre", true, false, None),
        ("cone · produto", false, true, None),
        ("cone · produto de novo", false, true, None),
        ("cone · todos por um cone", false, true, Some(u32::MAX)),
    ];
    let luz = |grossa: bool, salta: bool, um: Option<u32>| {
        let d = if grossa {
            base.com_fina_engrossada(k)
        } else {
            base
        };
        let g = grade_do_quadro(a, alvo, d).expect("grade");
        let mut cv = config_da_vista(&gpu.device, g, a, d);
        let mut ce = config_do_entorno(&gpu.device, a, alvo, d).expect("o entorno");
        cv.hrc_salta_o_t_nulo = salta;
        ce.hrc_salta_o_t_nulo = salta;
        let mut s = Solver::novo(&gpu, cv).expect("vista");
        let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, alvo);
        s.regiao_fina(i, f);
        let mut e = Solver::novo(&gpu, ce).expect("entorno");
        e.regiao_fina([0; 2], [0; 2]);
        assert!(e.mascarar_miolo(
            &gpu,
            Some([
                g.origem_m[0],
                g.origem_m[1],
                g.origem_m[0] + f64::from(g.largura) * g.texel_m,
                g.origem_m[1] + f64::from(g.altura) * g.texel_m,
            ])
        ));
        if let Some(um) = um {
            s.diag_cones_um(&gpu, um);
            e.diag_cones_um(&gpu, um);
        }
        for _ in 0..30 {
            e.quadro(&gpu, &cena);
            assert!(s.somar_entorno(&gpu, Some(&e)));
            s.quadro(&gpu, &cena);
        }
        (fluencia_lida(&gpu, &s).v, g.largura)
    };
    let corridas: Vec<_> = variantes
        .iter()
        .map(|&(_, grossa, salta, um)| luz(grossa, salta, um))
        .collect();
    for (ref_k, outras) in [(0, [1, 2]), (3, [4, 5])] {
        let (r, largura) = &corridas[ref_k];
        for o in outras {
            let x = &corridas[o].0;
            let mut n = 0;
            let mut ulps = [0usize; 4]; // 1 · 2 · 3–8 · mais
            let mut pior = (0u32, 0.0f32, [0usize; 2]);
            for (t, (p, q)) in r.iter().zip(x).enumerate() {
                if p == q {
                    continue;
                }
                n += 1;
                let u = (0..3)
                    .map(|c| {
                        let (a, b) = (f16::from_f32(p[c]).to_bits(), f16::from_f32(q[c]).to_bits());
                        u32::from(a.abs_diff(b))
                    })
                    .max()
                    .unwrap_or(0);
                ulps[match u {
                    1 => 0,
                    2 => 1,
                    3..=8 => 2,
                    _ => 3,
                }] += 1;
                if u > pior.0 {
                    let rel = (0..3)
                        .map(|c| (p[c] - q[c]).abs() / p[c].abs().max(1e-6))
                        .fold(0.0, f32::max);
                    pior = (u, rel, [t % *largura as usize, t / *largura as usize]);
                }
            }
            eprintln!(
                "  {} × {}: {n} texels · ULPs do f16 [1, 2, 3–8, >8] = {ulps:?} · pior {} ULP (rel {:.2e}) em {:?}",
                variantes[ref_k].0, variantes[o].0, pior.0, pior.1, pior.2
            );
        }
    }
}
