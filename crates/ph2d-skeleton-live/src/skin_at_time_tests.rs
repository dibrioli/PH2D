//! ⭐⭐⭐ **A PELE NOUTRO INSTANTE** — os gates da fonte de poses injectada
//! ([`crate::skin_live::skin_of_in`], [`crate::skin_image::deform_field_with`]).
//!
//! ⚠️ **O oráculo é o PRÓPRIO produto:** posar a pele sem tocar no mundo tem de dar exactamente o
//! que o mundo daria se o osso lá estivesse. Uma expectativa escrita à mão mediria a minha
//! aritmética; esta mede a **lei**, e reprova no dia em que as duas se separarem.
//!
//! É o que os fantasmas do onion precisam: eles desenham a arte deformada em `t ± k` **sem** mover
//! o objecto vivo (ADR-0142 — a pose-fantasma é não-destrutiva).

use super::*;

/// O quarto de volta que os dois lados deste gate aplicam ao osso.
const QUARTO: f32 = std::f32::consts::FRAC_PI_2;

/// A pele resolvida com a fonte `poses`, amostrada nos vértices de repouso — o gémeo do
/// [`posed_local`], com o instante injectado.
fn posed_local_with(
    sim: &SimWorld,
    e: Entity,
    sm: &SkinnedMesh,
    poses: &impl Fn(Entity) -> ph2d_skeleton::Xform,
) -> Option<Vec<[f64; 2]>> {
    let index = crate::skin_live::bone_index(sim);
    let (p2l, pele) = deform_field_with(sim, e, sm.mesh.size, PPM, &index, poses)?;
    let mut w = pele.scratch();
    Some(
        sm.mesh
            .rest
            .iter()
            .enumerate()
            .map(|(v, &p)| {
                let q = p2l.apply(p);
                let pesos = sm.pesos_de(v);
                if pesos.is_empty() {
                    pele.point(q, &mut w)
                } else {
                    pele.point_with(q, pesos, &mut w)
                }
            })
            .collect(),
    )
}

fn maior_desvio(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(p, q)| (p[0] - q[0]).hypot(p[1] - q[1]))
        .fold(0.0f64, f64::max)
}

/// Uma imagem presa a UM osso, com a malha guardada — a fixtura das duas metades.
fn braco() -> (SimWorld, Entity, Entity, SkinnedMesh) {
    let mut sim = SimWorld::default();
    let osso = crate::bone::create(&mut sim, None, [-2.0, 0.0], [2.0, 0.0]).expect("osso");
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, sprite(4.0, 2.0, 0.0, 0.0)))
        .id();
    let raiz = Entity::from_bits(osso);
    assert!(crate::skin_live::bind_image(
        &mut sim,
        e,
        &tinta(40, 20, 4, 4),
        [40, 20],
        PPM,
        GridOptions::default(),
        Some(raiz),
    ));
    let malha = skinned_mesh_of(&sim, e).expect("malha");
    (sim, e, raiz, malha)
}

/// ⭐⭐⭐ **Posar a pele SEM tocar no mundo dá o que o mundo daria** — e o mundo fica onde estava.
///
/// ⚠️ **As duas metades são uma lei só.** Um fantasma que deformasse certo mas mexesse no objecto
/// vivo destruiria a pose que o animador está a autorar — é a razão de o `pose_at` existir ao lado
/// do `apply`, um nível acima (ADR-0142).
#[test]
fn posing_the_skin_without_touching_the_world_is_what_the_world_would_give() {
    let (mut sim, e, raiz, malha) = braco();
    let repouso = posed_local(&sim, e, &malha).expect("pele");

    // O ORÁCULO: o que a pele dá com o osso REALMENTE girado.
    {
        let mut t = sim
            .world_mut()
            .get_mut::<Transform>(raiz)
            .expect("Transform");
        t.rotation = QUARTO;
    }
    let oraculo = posed_local(&sim, e, &malha).expect("pele");
    {
        let mut t = sim
            .world_mut()
            .get_mut::<Transform>(raiz)
            .expect("Transform");
        t.rotation = 0.0;
    }
    assert!(
        maior_desvio(&repouso, &posed_local(&sim, e, &malha).expect("pele")) < 1e-12,
        "a fixtura nao voltou ao repouso — o oraculo abaixo mede outra coisa"
    );

    // A FONTE INJECTADA: o mesmo quarto de volta, com o mundo parado.
    // ⚠️⚠️ **A pose injectada é a do OSSO com a rotação trocada, e não uma rotação solta.** A 1.ª
    // redacção usou `Transform::IDENTITY` e o gate leu `2e0` de desvio — que é exactamente a
    // translação da raiz do osso: *uma fixtura que perde a pose de base mede outro esqueleto.*
    let girado = ph2d_vec_entities::transform::xform_of_transform(Transform {
        rotation: QUARTO,
        ..ph2d_vec_entities::transform::world_transform(&sim, raiz)
    });
    let poses = |x: Entity| {
        if x == raiz {
            girado
        } else {
            ph2d_vec_entities::transform::xform_of_transform(
                ph2d_vec_entities::transform::world_transform(&sim, x),
            )
        }
    };
    let posado = posed_local_with(&sim, e, &malha, &poses).expect("pele");

    let desvio = maior_desvio(&oraculo, &posado);
    println!("pele posada vs mundo girado: maior desvio {desvio:e}");
    assert!(
        desvio < 1e-9,
        "a pele posada por fonte injectada nao e' a que o mundo daria (maior desvio {desvio:e}) — \
         o fantasma do onion desenharia outra dobra que a do quadro"
    );
    // ⛔ O CONTROLO que torna o de cima uma afirmação: o quarto de volta TEM de mover a malha.
    let moveu = maior_desvio(&repouso, &posado);
    assert!(
        moveu > 0.5,
        "a fonte injectada nao dobrou nada (maior deslocamento {moveu}) — o gate acima estaria a \
         comparar duas malhas em repouso"
    );
    // ⚠️ E o MUNDO ficou onde estava: a pele viva continua a ser a de repouso.
    assert!(
        maior_desvio(&repouso, &posed_local(&sim, e, &malha).expect("pele")) < 1e-12,
        "resolver a pele noutro instante MEXEU no mundo — um fantasma nao pode mover o objecto vivo"
    );
}

/// ⭐⭐ **A MALHA que o fantasma desenha é a arte dobrada** — a porta [`posed_sprite_mesh`] com a
/// pele de outro instante, que é exactamente o que o onion monta.
///
/// ⚠️ **A UV não se mexe**, e é a metade que se esquece: ela é a do quad no ponto de REPOUSO, logo
/// dobrar a arte não pode remapear a textura — se ela seguisse a pose, o desenho escorregaria por
/// dentro da silhueta.
#[test]
fn the_ghost_mesh_bends_the_art_and_keeps_the_rest_uv() {
    let (sim, e, raiz, malha) = braco();
    let sprite = sim.world().get::<Sprite>(e).copied().expect("sprite");
    let inst = instancia_de(&sprite);
    let index = crate::skin_live::bone_index(&sim);
    let vivo = |x: Entity| {
        ph2d_vec_entities::transform::xform_of_transform(
            ph2d_vec_entities::transform::world_transform(&sim, x),
        )
    };
    // ⚠️⚠️ **A pose injectada é a do OSSO com a rotação trocada, e não uma rotação solta.** A 1.ª
    // redacção usou `Transform::IDENTITY` e o gate leu `2e0` de desvio — que é exactamente a
    // translação da raiz do osso: *uma fixtura que perde a pose de base mede outro esqueleto.*
    let girado = ph2d_vec_entities::transform::xform_of_transform(Transform {
        rotation: QUARTO,
        ..ph2d_vec_entities::transform::world_transform(&sim, raiz)
    });
    let em_t = |x: Entity| if x == raiz { girado } else { vivo(x) };

    let (p2l, pele_repouso) =
        deform_field_with(&sim, e, malha.mesh.size, PPM, &index, &vivo).expect("pele");
    let (_, pele_t) =
        deform_field_with(&sim, e, malha.mesh.size, PPM, &index, &em_t).expect("pele");
    let a = posed_sprite_mesh(
        malha.mesh.clone(),
        p2l,
        &pele_repouso,
        &malha.pesos,
        inst.anchor,
        inst.size,
    )
    .expect("malha de repouso");
    let b = posed_sprite_mesh(
        malha.mesh,
        p2l,
        &pele_t,
        &malha.pesos,
        inst.anchor,
        inst.size,
    )
    .expect("malha em t");

    assert_eq!(
        a.uv, b.uv,
        "a UV do fantasma seguiu a pose em vez do repouso"
    );
    assert_eq!(a.tris, b.tris, "a topologia mudou entre os dois instantes");
    let moveu = a
        .local
        .iter()
        .zip(&b.local)
        .map(|(p, q)| f64::from(p[0] - q[0]).hypot(f64::from(p[1] - q[1])))
        .fold(0.0f64, f64::max);
    assert!(
        moveu > 0.1,
        "a malha do fantasma nao dobrou (maior deslocamento {moveu}) — ele desenharia o quad de \
         repouso, que e' o defeito que esta wave cura"
    );
}
