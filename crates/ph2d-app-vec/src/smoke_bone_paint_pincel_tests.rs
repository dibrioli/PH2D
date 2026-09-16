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

/// ⏱️ **SONDA (`--ignored`) — o ANEL da Remoção de fundo na cena real: o de antes contra a porta.**
///
/// Cada ponto do anel volta à imagem pela porta do PONTEIRO (a que o pincel de protecção usa), e o
/// erro é a distância ao centro, em px de origem, contra o raio.
#[test]
#[ignore = "sonda: imprime a tabela, sem barra"]
fn sonda_o_anel_da_remocao_de_fundo() {
    for graus in [0.0_f32, super::super::super::super::DOBRA_GRAUS] {
        println!("-- dobra {graus} graus por junta --");
        mede_o_anel(graus);
    }
}

/// O corpo da [`sonda_o_anel_da_remocao_de_fundo`], com a dobra escolhida — a de `0°` é o CONTROLO:
/// sem dobra o anel de antes está certo, e a régua tem de o dizer.
fn mede_o_anel(graus: f32) {
    use ph2d_ecs::PresentWorld;
    let (sim, e) = cena_dobrada(
        super::super::super::super::ALTURA_PX,
        None,
        ph2d_poly2d::GridOptions::default(),
        graus,
    );
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let sprite = *sim.world().get::<ph2d_render::Sprite>(e).expect("sprite");
    let tr = ph2d_ecs::world_transform(sim.world(), e).expect("pose");
    assert!(
        tr.rotation == 0.0 && tr.scale.x == 1.0 && tr.scale.y == 1.0,
        "a sonda supoe a base identidade"
    );
    let (size, anchor) = (sprite.size, sprite.resolve_anchor(PPM));
    let origem = (sm.mesh.size[0], sm.mesh.size[1]);
    let (m, _) = ph2d_skeleton_live::skin_image::posed_sprite_mesh(
        sm.mesh.clone(),
        p2l,
        &pele,
        &sm.pesos,
        anchor,
        size,
        None,
    )
    .expect("malha posada");
    let mut present = PresentWorld::new();
    present.world_mut().spawn((
        ph2d_ecs::SimRef(e),
        ph2d_ecs::GlobalTransform::from_transform(tr),
        ph2d_render::RenderInstance {
            world_pos: [tr.translation.x, tr.translation.y],
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
    let janela = ph2d_host::WindowSize {
        width: 1600,
        height: 900,
    };
    let camera = ph2d_render::Camera2d::new(
        [tr.translation.x, tr.translation.y],
        900.0 / PX_POR_METRO as f32,
    );
    let bits = e.to_bits();
    let quad = ph2d_sprite_screen::sprite_image_to_screen_affine(
        origem.0, origem.1, tr, &sprite, None, &camera, janela,
    )
    .as_coeffs();
    let escala = quad[0].hypot(quad[1]);
    let volta = |pts: &[[f64; 2]], c: [f32; 2], raio: f32, present: &mut PresentWorld| -> f32 {
        pts.iter()
            .filter_map(|p| {
                match ph2d_sprite_screen::uv_sob_o_ponteiro(
                    &sim,
                    present.world_mut(),
                    &camera,
                    janela,
                    bits,
                    p[0] as f32,
                    p[1] as f32,
                ) {
                    ph2d_sprite_screen::UvSobOPonteiro::Uv(u, v) => Some(
                        (((u - c[0]) * origem.0 as f32).hypot((v - c[1]) * origem.1 as f32) / raio
                            - 1.0)
                            .abs(),
                    ),
                    _ => None,
                }
            })
            .fold(0.0_f32, f32::max)
    };
    for raio in [8.0_f32, 24.0] {
        let (mut antes, mut agora, mut n) = (Vec::new(), 0.0_f32, 0usize);
        for iu in 1..20 {
            for iv in 1..8 {
                let c = [iu as f32 / 20.0, iv as f32 / 8.0];
                let Some(l) = local_de(&m, c) else { continue };
                let w = [l[0] + tr.translation.x, l[1] + tr.translation.y];
                let cursor = camera.world_to_screen(w, janela);
                let Some(arcos) = ph2d_sprite_screen::anel_do_pincel(
                    &sim,
                    present.world_mut(),
                    &camera,
                    janela,
                    bits,
                    cursor,
                    raio,
                    origem,
                ) else {
                    continue;
                };
                if arcos.len() != 1
                    || arcos[0].len() != ph2d_sprite_screen::LADOS_DO_ANEL as usize + 1
                {
                    continue;
                }
                let r = f64::from(raio) * escala;
                let circulo: Vec<[f64; 2]> = (0..64)
                    .map(|i| {
                        let a = f64::from(i) / 64.0 * std::f64::consts::TAU;
                        [
                            f64::from(cursor.0) + r * a.cos(),
                            f64::from(cursor.1) + r * a.sin(),
                        ]
                    })
                    .collect();
                antes.push((volta(&circulo, c, raio, &mut present), c));
                agora = agora.max(volta(&arcos[0], c, raio, &mut present));
                n += 1;
            }
        }
        antes.sort_by(|a, b| a.0.total_cmp(&b.0));
        let q = |f: f32| antes[((antes.len() - 1) as f32 * f) as usize];
        println!(
            "raio {raio:>4} px | {n:>3} centros | anel de antes: p50 {:.1} % p90 {:.1} % pior {:.1} % \
             (em {:?}) | anel novo: {:.2e}",
            q(0.5).0 * 100.0,
            q(0.9).0 * 100.0,
            q(1.0).0 * 100.0,
            q(1.0).1,
            agora
        );
    }
}

/// ⏱️ **SONDA (`--ignored`) — a CAIXA do gizmo (o quad de repouso) contra o que a malha desenha.**
#[test]
#[ignore = "sonda: imprime a tabela, sem barra"]
fn sonda_a_caixa_do_gizmo_contra_a_malha() {
    for graus in [0.0_f32, super::super::super::super::DOBRA_GRAUS, 60.0] {
        let (sim, e) = cena_dobrada(
            super::super::super::super::ALTURA_PX,
            None,
            ph2d_poly2d::GridOptions::default(),
            graus,
        );
        let (sm, p2l, pele) = campo_da_cena(&sim, e);
        let sprite = *sim.world().get::<ph2d_render::Sprite>(e).expect("sprite");
        let (size, anchor) = (sprite.size, sprite.resolve_anchor(PPM));
        let (m, _) = ph2d_skeleton_live::skin_image::posed_sprite_mesh(
            sm.mesh.clone(),
            p2l,
            &pele,
            &sm.pesos,
            anchor,
            size,
            None,
        )
        .expect("malha posada");
        let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
        for p in &m.local {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        let quad_lo = [anchor[0] - size[0] / 2.0, anchor[1] - size[1] / 2.0];
        let quad_hi = [anchor[0] + size[0] / 2.0, anchor[1] + size[1] / 2.0];
        let px = |v: f32| v * PX_POR_METRO as f32;
        println!(
            "{graus:>4} graus | quad [{:.0},{:.0}]..[{:.0},{:.0}] px | malha [{:.0},{:.0}]..[{:.0},{:.0}] px",
            px(quad_lo[0]),
            px(quad_lo[1]),
            px(quad_hi[0]),
            px(quad_hi[1]),
            px(lo[0]),
            px(lo[1]),
            px(hi[0]),
            px(hi[1])
        );
    }
}
