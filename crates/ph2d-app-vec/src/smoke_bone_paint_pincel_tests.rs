//! ⏱️ **O PINCEL SOBRE A ARTE DOBRADA PRECISA DO `Smooth`?** — filho da [`super`] (a silhueta) para
//! herdar as fixturas; a pergunta é outra: não o que se DESENHA, mas o que se PINTA.
//!
//! ⚠️ A nota do `SkinDeform::Smooth` dizia que sim (a cura da curvatura do pincel só ligava com as
//! peças do `Smooth`), e a medição era de quando a malha do bind guardava `~200` peças. Hoje ela
//! guarda `2 430`, e esta sonda remede na cena real — a resposta está no doc daquele enum.

use super::*;

/// O ponto LOCAL posado de uma UV de repouso, pela malha desenhada (`None` fora dela).
fn local_de(m: &ph2d_render::SpriteMesh, uv: [f32; 2]) -> Option<[f32; 2]> {
    m.tris.iter().find_map(|t| {
        let [a, b, c] = t.map(|i| m.uv[i as usize]);
        let det = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
        if det == 0.0 {
            return None;
        }
        let l1 = ((uv[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (uv[1] - a[1])) / det;
        let l2 = ((b[0] - a[0]) * (uv[1] - a[1]) - (uv[0] - a[0]) * (b[1] - a[1])) / det;
        let l0 = 1.0 - l1 - l2;
        if l0 < -1e-6 || l1 < -1e-6 || l2 < -1e-6 {
            return None;
        }
        let [pa, pb, pc] = t.map(|i| m.local[i as usize]);
        Some([
            l0 * pa[0] + l1 * pb[0] + l2 * pc[0],
            l0 * pa[1] + l1 * pb[1] + l2 * pc[1],
        ])
    })
}

/// ⏱️ **SONDA (`--ignored`) — o PINCEL sobre a arte dobrada precisa do `Smooth`?**
#[test]
#[ignore = "sonda: imprime a tabela, sem barra"]
fn sonda_o_pincel_precisa_do_smooth() {
    use ph2d_ecs::PresentWorld;
    let (sim, e) = cena(super::super::super::super::ALTURA_PX, None);
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let sprite = *sim.world().get::<ph2d_render::Sprite>(e).expect("sprite");
    let (size, anchor) = (sprite.size, sprite.resolve_anchor(PPM));
    let [iw, ih] = sm.mesh.size;
    let malha = |refine| {
        ph2d_skeleton_live::skin_image::posed_sprite_mesh(
            sm.mesh.clone(),
            p2l,
            &pele,
            &sm.pesos,
            anchor,
            size,
            refine,
        )
        .expect("malha posada")
        .0
    };
    let modos = [
        ("Fast", None),
        ("Smooth z1", Some(opcoes(true, 1.0))),
        ("Smooth z4", Some(opcoes(true, 4.0))),
        ("Smooth z8", Some(opcoes(true, 8.0))),
    ];
    for (nome, refine) in modos {
        let m = malha(refine);
        let mut present = PresentWorld::new();
        present.world_mut().spawn((
            ph2d_ecs::SimRef(e),
            ph2d_ecs::GlobalTransform::from_transform(ph2d_ecs::Transform::IDENTITY),
            ph2d_render::RenderInstance {
                world_pos: [0.0, 0.0],
                size,
                atlas_uv: [0.0, 0.0, 1.0, 1.0],
                tint: [1.0; 4],
                basis: ph2d_render::RenderInstance::IDENTITY_BASIS,
                texture_id: 0,
                premultiplied: 0.0,
                anchor,
                per_corner_tint: [[1.0; 4]; 4],
                opacity: 1.0,
                flip_uv: 0,
                z_order: 0,
                sampling: 0,
                uv_xform: ph2d_render::RenderInstance::IDENTITY_UV_XFORM,
                clip_group: ph2d_render::RenderInstance::CLIP_GROUP_NONE,
                clip_meta: 0,
                sub_order: 0,
            },
            m.clone(),
        ));
        for raio_px in [4.0_f32, 8.0, 16.0, 32.0, 64.0] {
            let fp = [raio_px / iw as f32, raio_px / ih as f32];
            let (mut pior, mut soma, mut n, mut sem) = (1.0_f32, 0.0_f32, 0usize, 1.0_f32);
            for iu in 1..20 {
                for iv in 1..8 {
                    let centro = [iu as f32 / 20.0, iv as f32 / 8.0];
                    let Some(c0) = local_de(&m, centro) else {
                        continue;
                    };
                    let ph2d_render::MeshUv::Use { warp, .. } =
                        ph2d_render::mesh_uv(present.world_mut(), e.to_bits(), c0, true, fp)
                    else {
                        continue;
                    };
                    let d = ph2d_painter_brush::canvas_warp::warped_dab(
                        ph2d_painter_brush::canvas_warp::CanvasWarp {
                            linear: warp.linear,
                            curve: warp.curve,
                        },
                        0.0,
                        0,
                    );
                    let pegada = ph2d_painter_brush::FootprintDeform::new(d.flatten, d.angle_deg)
                        .with_curve(d.curve);
                    let mede = |corrigido: bool| -> Option<f32> {
                        let (mut lo, mut hi) = (f32::INFINITY, 0.0_f32);
                        for k in 0..360 {
                            let (p, s) = if corrigido {
                                (pegada.outline_at(k as f32 / 360.0), d.radius_scale)
                            } else {
                                let a = k as f32 / 360.0 * std::f32::consts::TAU;
                                ([a.cos(), a.sin()], 1.0)
                            };
                            let q = [centro[0] + p[0] * fp[0] * s, centro[1] + p[1] * fp[1] * s];
                            let l = local_de(&m, q)?;
                            let r = (l[0] - c0[0]).hypot(l[1] - c0[1]);
                            lo = lo.min(r);
                            hi = hi.max(r);
                        }
                        Some(hi / lo)
                    };
                    let (Some(com), Some(crua)) = (mede(true), mede(false)) else {
                        continue;
                    };
                    pior = pior.max(com);
                    sem = sem.max(crua);
                    soma += com;
                    n += 1;
                }
            }
            println!(
                "{nome:>10} ({:>5} pecas) | raio {raio_px:>4} px | {n:>3} dabs | redondeza media \
                 {:.4} pior {pior:.4} | sem correcao pior {sem:.4}",
                m.tris.len(),
                soma / n.max(1) as f32
            );
        }
    }
}
