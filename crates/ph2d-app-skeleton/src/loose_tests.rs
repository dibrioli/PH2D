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

/// Os sete números do mundo de `e`, com o zero sem sinal (o único desvio que o `compose` admite).
fn mundo(sim: &SimWorld, e: Entity) -> [u32; 7] {
    let t = ph2d_ecs::world_transform(sim.world(), e).expect("mundo");
    [
        t.translation.x,
        t.translation.y,
        t.rotation,
        t.scale.x,
        t.scale.y,
        t.skew_x,
        t.skew_y,
    ]
    .map(|v| (v + 0.0).to_bits())
}

/// ⭐⭐⭐ GATE (A15) — **o esqueleto adoptado tem a origem na cabeça da raiz e a pose AO BIT** —
/// 4 000 raízes (de topo e dentro de grupos com escala e skew, raiz com escala e skew), três ossos
/// cada. Medido antes da cura (sonda de `16abdf7cb`): esqueleto em `T = L` e raiz em `0` = 0 de
/// 12 000 fora do bit; rebasear por outro `T` (`L − T`) = 5 055 de 12 000, pior 2 560 ULP.
#[test]
fn the_adopted_skeleton_sits_on_the_root_and_the_pose_stays_to_the_bit() {
    use ph2d_core::Vec2;
    let mut rng = 0x9E37_79B9_7F4A_7C15_u64;
    let mut f = |lo: f32, hi: f32| {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        lo + (hi - lo) * ((rng >> 40) as f32 / (1u64 << 24) as f32)
    };
    let doc = TimelineDoc::default();
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
        let l = [f(-2000.0, 2000.0), f(-2000.0, 2000.0)];
        let raiz = osso(&mut sim, l, f(-3.2, 3.2), pai);
        if k % 3 == 0 {
            let mut t = sim.world_mut().get_mut::<Transform>(raiz).expect("raiz");
            t.scale = Vec2::new(f(0.3, 2.5), f(0.3, 2.5));
            t.skew_x = f(-0.4, 0.4);
        }
        let o2 = osso(&mut sim, [f(1.0, 40.0), f(-5.0, 5.0)], f(-3.2, 3.2), Some(raiz));
        let o3 = osso(&mut sim, [f(1.0, 40.0), f(-5.0, 5.0)], f(-3.2, 3.2), Some(o2));
        let antes = [raiz, o2, o3].map(|e| mundo(&sim, e));
        let esq = Entity::from_bits(adopt_loose_roots(&mut sim, &doc)[0]);
        assert_eq!(
            sim.world().get::<Transform>(esq).expect("esq").translation,
            Vec2::new(l[0], l[1]),
            "a origem do esqueleto não ficou na raiz ({k})"
        );
        assert_eq!(
            [raiz, o2, o3].map(|e| mundo(&sim, e)),
            antes,
            "a pose saiu do bit ({k})"
        );
    }
}

/// ⭐⭐ GATE (A15) — **onde mover a origem não seria exacto, o esqueleto fica na identidade**: o
/// repouso guardado noutro sítio e a posição na timeline. Controlo: o repouso IGUAL à pose vai com
/// ela (fica `0` na raiz) e repô-lo devolve o mesmo mundo.
#[test]
fn a_root_whose_place_is_stored_elsewhere_keeps_the_skeleton_at_identity() {
    let adopta = |rest: Option<[f32; 2]>, animada: bool| {
        let mut sim = SimWorld::default();
        let raiz = osso(&mut sim, [30.0, -12.5], 0.3, None);
        if let Some(r) = rest {
            let mut b = BoneRest::de(&Transform::IDENTITY);
            b.translation = r;
            b.rotation = 0.3;
            sim.world_mut().entity_mut(raiz).insert(b);
        }
        let mut doc = TimelineDoc::default();
        if animada {
            doc.bind(raiz.to_bits(), ph2d_timeline::PropKind::TranslationY);
        }
        let esq = Entity::from_bits(adopt_loose_roots(&mut sim, &doc)[0]);
        let t = sim.world().get::<Transform>(esq).expect("esq").translation;
        (sim, raiz, [t.x, t.y])
    };
    assert_eq!(adopta(None, false).2, [30.0, -12.5], "controlo: sem nada guardado");
    assert_eq!(adopta(Some([1.0, 2.0]), false).2, [0.0, 0.0], "repouso noutro sítio");
    assert_eq!(adopta(None, true).2, [0.0, 0.0], "posição na timeline");
    let (mut sim, raiz, t) = adopta(Some([30.0, -12.5]), false);
    assert_eq!(t, [30.0, -12.5], "o repouso igual à pose vai com ela");
    let antes = mundo(&sim, raiz);
    let rest = *sim.world().get::<BoneRest>(raiz).expect("repouso");
    assert_eq!(rest.translation, [0.0, 0.0]);
    let mut tr = sim.world_mut().get_mut::<Transform>(raiz).expect("raiz");
    rest.aplica(&mut tr);
    assert_eq!(mundo(&sim, raiz), antes, "repor o repouso moveu a raiz");
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

    let novos = adopt_loose_roots(&mut sim, &TimelineDoc::default());
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
        adopt_loose_roots(&mut sim, &TimelineDoc::default()).is_empty(),
        "controlo: a 2.ª passagem não cria nada"
    );
}
