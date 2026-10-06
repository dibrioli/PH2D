//! O gate da porta das raízes soltas (A14).

use super::*;

fn osso(sim: &mut SimWorld, pos: [f32; 2], rot: f32, pai: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform {
                translation: ph2d_core::Vec2::new(pos[0], pos[1]),
                rotation: rot,
                ..Transform::IDENTITY
            },
            Bone {
                length: 10.0,
                ..Bone::default()
            },
        ))
        .id();
    match pai {
        Some(p) => {
            sim.world_mut().entity_mut(e).insert(ChildOf(p));
        }
        None => {
            sim.world_mut().entity_mut(e).insert(RootOrder(3));
        }
    }
    e
}

/// SONDA (A15) — o erro, ao bit, de pôr o esqueleto adoptado junto à raiz. Variantes sobre a mesma
/// varredura: `0` = na identidade (hoje); `1` = esqueleto em `T` = translação local da raiz e raiz
/// em `0`; `2` = esqueleto em `T = L + d` (`d` até 60: o repouso ou uma chave guardados, que não
/// são a pose de agora) e raiz em `L − T` — o rebase de um valor guardado.
#[test]
#[ignore = "sonda de medição (A15): --ignored --nocapture"]
fn diag_a15_o_erro_de_mover_o_esqueleto_para_a_raiz() {
    use ph2d_core::Vec2;
    fn bits(t: &Transform) -> [u32; 7] {
        [
            t.translation.x.to_bits(),
            t.translation.y.to_bits(),
            t.rotation.to_bits(),
            t.scale.x.to_bits(),
            t.scale.y.to_bits(),
            t.skew_x.to_bits(),
            t.skew_y.to_bits(),
        ]
    }
    fn ulp(a: u32, b: u32) -> u32 {
        (a as i32).wrapping_sub(b as i32).unsigned_abs()
    }
    for variante in 0..3 {
        let mut rng = 0x9E37_79B9_7F4A_7C15_u64;
        let mut f = |lo: f32, hi: f32| {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            lo + (hi - lo) * ((rng >> 40) as f32 / (1u64 << 24) as f32)
        };
        let (mut ossos, mut errados, mut pior) = (0usize, 0usize, 0u32);
        for k in 0..4000 {
            let mut sim = SimWorld::default();
            let pai = (k % 2 == 1).then(|| {
                let t = Transform {
                    translation: Vec2::new(f(-900.0, 900.0), f(-900.0, 900.0)),
                    rotation: f(-3.2, 3.2),
                    scale: Vec2::new(f(0.2, 3.0), f(0.2, 3.0)),
                    skew_x: if k % 4 == 3 { f(-0.5, 0.5) } else { 0.0 },
                    skew_y: if k % 4 == 3 { f(-0.5, 0.5) } else { 0.0 },
                };
                sim.world_mut().spawn(t).id()
            });
            let raiz = osso(&mut sim, [f(-2000.0, 2000.0), f(-2000.0, 2000.0)], f(-3.2, 3.2), pai);
            if k % 3 == 0 {
                let mut t = sim.world_mut().get_mut::<Transform>(raiz).expect("raiz");
                t.scale = Vec2::new(f(0.3, 2.5), f(0.3, 2.5));
                t.skew_x = f(-0.4, 0.4);
            }
            let o2 = osso(&mut sim, [f(1.0, 40.0), f(-5.0, 5.0)], f(-3.2, 3.2), Some(raiz));
            let o3 = osso(&mut sim, [f(1.0, 40.0), f(-5.0, 5.0)], f(-3.2, 3.2), Some(o2));
            let todos = [raiz, o2, o3];
            let antes: Vec<[u32; 7]> = todos
                .iter()
                .map(|e| bits(&ph2d_ecs::world_transform(sim.world(), *e).expect("mundo")))
                .collect();
            let novos = adopt_loose_roots(&mut sim);
            let esq = Entity::from_bits(novos[0]);
            let l = sim.world().get::<Transform>(raiz).expect("raiz").translation;
            let (t, resto) = match variante {
                0 => (Vec2::new(0.0, 0.0), l),
                1 => (l, Vec2::new(0.0, 0.0)),
                _ => {
                    let t = Vec2::new(l.x + f(-60.0, 60.0), l.y + f(-60.0, 60.0));
                    (t, Vec2::new(l.x - t.x, l.y - t.y))
                }
            };
            sim.world_mut().get_mut::<Transform>(esq).expect("esq").translation = t;
            sim.world_mut().get_mut::<Transform>(raiz).expect("raiz").translation = resto;
            for (e, a) in todos.iter().zip(&antes) {
                let d = bits(&ph2d_ecs::world_transform(sim.world(), *e).expect("mundo"));
                ossos += 1;
                if d != *a {
                    errados += 1;
                }
                pior = pior.max(d.iter().zip(a).map(|(x, y)| ulp(*x, *y)).max().unwrap_or(0));
            }
        }
        eprintln!("A15 variante {variante}: ossos {ossos} · fora do bit {errados} · pior ULP {pior}");
    }
}

/// ⭐⭐⭐ **Um projecto de antes do A14 abre com um esqueleto por raiz, e a pose AO BIT** — duas
/// correntes soltas (uma de topo, outra dentro de um grupo posado) e uma que já tem esqueleto.
/// Controlo: a 2.ª passagem não cria nada, e a corrente que já tinha esqueleto fica onde estava.
#[test]
fn an_old_project_opens_with_one_skeleton_per_root_and_the_pose_to_the_bit() {
    let mut sim = SimWorld::default();
    let a = osso(&mut sim, [3.0, 4.0], 0.7, None);
    let _a2 = osso(&mut sim, [10.0, 0.0], -1.3, Some(a));
    let grupo = sim
        .world_mut()
        .spawn(Transform {
            translation: ph2d_core::Vec2::new(-7.0, 2.5),
            rotation: 0.4,
            ..Transform::IDENTITY
        })
        .id();
    let b = osso(&mut sim, [1.0, 1.0], 2.1, Some(grupo));
    let _b2 = osso(&mut sim, [10.0, 0.0], 0.9, Some(b));
    let ja = sim.world_mut().spawn((Transform::IDENTITY, Skeleton)).id();
    let c = osso(&mut sim, [0.0, 0.0], 0.0, Some(ja));
    let antes = ph2d_skeleton_live::skin_live::bone_polylines(&sim);

    let novos = adopt_loose_roots(&mut sim);
    assert_eq!(novos.len(), 2, "uma raiz solta, um esqueleto");
    let w = sim.world();
    let dono = |e: Entity| ph2d_skeleton_ecs::skeleton_of(w, e).map(Entity::to_bits);
    assert!(novos.contains(&dono(a).expect("A ganhou esqueleto")));
    assert!(novos.contains(&dono(b).expect("B ganhou esqueleto")));
    assert_eq!(
        dono(c),
        Some(ja.to_bits()),
        "controlo: a que já tinha esqueleto"
    );
    let esq_b = Entity::from_bits(dono(b).expect("dono"));
    assert_eq!(
        w.get::<ChildOf>(esq_b).map(|p| p.parent()),
        Some(grupo),
        "o esqueleto fica no lugar da raiz (dentro do grupo)"
    );
    assert_eq!(
        w.get::<RootOrder>(Entity::from_bits(dono(a).expect("dono"))),
        Some(&RootOrder(3)),
        "o esqueleto de topo herda a ordem da raiz"
    );
    // (sobrevivente M10) a raiz deixou de ser raiz: a ordem dela fica com o esqueleto, não em dobro.
    assert_eq!(
        w.get::<RootOrder>(a),
        None,
        "a raiz adoptada guardou uma RootOrder obsoleta"
    );
    assert_eq!(
        ph2d_skeleton_live::skin_live::bone_polylines(&sim),
        antes,
        "a pose de algum osso mudou — a migração mexeu no desenho"
    );
    assert!(
        adopt_loose_roots(&mut sim).is_empty(),
        "controlo: a 2.ª passagem não cria nada"
    );
}
