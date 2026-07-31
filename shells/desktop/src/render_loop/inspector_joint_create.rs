//! §12 Physics Joint — **o gesto que CRIA um joint** (W3, extraído em
//! W-JointCopy).
//!
//! Irmão do `inspector_joint` (que descreve e EDITA um que já existe), separado
//! dele quando a porta do paste passou o cap de 600 LOC do shell. O corte é o
//! que a própria UI já faz: **criar mora na §11** — um joint não existe ainda
//! quando você quer fazer um, então o botão fica onde você já está, olhando dois
//! corpos selecionados — e **editar mora na §12**.
//!
//! `inspector_joint` re-exporta as três funções, então todo chamador mantém o
//! caminho `inspector_joint::create_joint` que já usava.

use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{JointKind, PhysicsJoint};

/// Create a joint between two bodies — the gesture behind §11's *Join Selected
/// Bodies*.
///
/// **Both bodies are given a `Name` if they lack one.** The joint stores name
/// hashes, so an unnamed body is one a joint cannot refer to; naming it here is
/// not a side effect to be apologised for, it is how identity works in this
/// editor (the timeline's bindings have the same requirement).
///
/// The new joint lands at the **midpoint** of the two bodies. One rule for
/// every kind: for a Pin between two touching bodies — a chain link, the
/// common case — the midpoint IS the correct pivot, and for the others it is a
/// sensible place to start dragging from.
///
/// `kind` is the artist's choice from §11's join-kind selector — the gold
/// standard is to create the type you want, not to make a Pin and convert it.
/// `anchored` is left `false` (the default): the first reconcile seeds the
/// body-local anchors from the midpoint under THIS kind's policy (a Spring/Rope
/// anchors body B at its centre, a Pin/Weld at the shared point).
pub(crate) fn create_joint(
    sim: &mut SimWorld,
    a_bits: u64,
    b_bits: u64,
    kind: JointKind,
) -> Option<Entity> {
    create_joint_at(sim, a_bits, b_bits, kind, None)
}

/// The same creation, with the anchors the GESTURE named (W-J4).
///
/// `at` is `Some((anchor_a_world, anchor_b_world))` for the canvas gesture —
/// press point and release point — and `None` for the selection route, which has
/// no points to offer and lets the seed policy place them (`anchored: false`).
///
/// ⚠️ **With points, the joint is born ALREADY anchored**: the locals are
/// converted here, against each body's authored pose, and `anchored: true` tells
/// the reconcile to READ them rather than re-derive from the policy. Going
/// through the seed instead would throw the gesture away — the policy puts a
/// spring's B end at the body's CENTRE, which is exactly the *"anchors are born
/// in centres, not where I dragged"* failure this wave exists to remove.
///
/// One function for both routes, so the naming, the two `a == b` guards, the
/// unique label and the `RootOrder` stamp cannot drift between them.
pub(crate) fn create_joint_at(
    sim: &mut SimWorld,
    a_bits: u64,
    b_bits: u64,
    kind: JointKind,
    at: Option<([f32; 2], [f32; 2])>,
) -> Option<Entity> {
    let (a, b) = (Entity::from_bits(a_bits), Entity::from_bits(b_bits));
    if a == b {
        return None;
    }
    // ⚠️ **MUNDO, não `Transform` cru.** `Transform` é LOCAL e compõe com o pai
    // (W5), então um corpo PARENTEADO devolve o próprio offset — e o "meio entre
    // os dois" saía entre a pose de mundo de um e o offset do outro, um lugar que
    // não é nem um nem outro. Medido na cena 67 antes do conserto: o pescoço de um
    // boneco nascia **1,65 m abaixo** da emenda, e o ragdoll esparramava porque
    // cada membro pendia de um ponto no ar.
    //
    // Sobreviveu desde o W3 porque toda fixture, cena e demo usava corpos-RAIZ,
    // onde local e mundo coincidem — a mesma frase que o W5 escreveu sobre si.
    let pa = ph2d_ecs::world_transform(sim.world(), a)?.translation;
    let pb = ph2d_ecs::world_transform(sim.world(), b)?.translation;

    let name_a = ensure_named(sim, a, "Body")?;
    let name_b = ensure_named(sim, b, "Body")?;

    // ⚠️ The `a == b` guard above compares ENTITIES; this compares the thing a
    // joint actually stores. Two bodies that happen to share a name resolve to
    // one id, so the joint could never bind — and it would report success.
    if ph2d_ecs::stable_name_id(&name_a) == ph2d_ecs::stable_name_id(&name_b) {
        return None;
    }
    // **A joint is named after what it joins** (W-J8) — the Unreal Constraints
    // Graph idiom. "Post : Plank" in the Hierarchy is a joint you can find; the
    // "Joint (3)" it replaces is a row you have to click to identify, in a rig
    // where every row looks the same.
    //
    // ⚠️ **A snapshot at creation, not a live binding, and that is deliberate.**
    // Renaming a body does not rewrite this label. The name is the artist's — the
    // Hierarchy lets them edit it — and a label that rewrote itself would fight a
    // rename it cannot know was intentional. (What DOES follow a rename is the
    // BINDING, which travels by name hash and re-attaches by itself.)
    let label = crate::name_unique::unique_name(sim, &format!("{name_a} : {name_b}"));
    // The authored poses of the two bodies — what a body-local anchor is
    // measured against (the seed uses the same `rest`, and using the LIVE pose
    // would bake a swing into the local; W-AnchorFollow).
    // A mesma correção do outro lado: a conversão mundo→local de uma âncora tem
    // de ser contra a pose de MUNDO do corpo, senão a rota do canvas planta o
    // mesmo erro que o ponto médio plantava.
    let pose = |e: Entity| {
        ph2d_ecs::world_transform(sim.world(), e)
            .map(|t| [t.translation.x, t.translation.y, t.rotation])
    };
    let anchored = at.and_then(|(wa, wb)| {
        // A shared-point kind is ONE place: the press point is the pivot, and
        // both bodies are anchored to it. A two-ended kind gets both points.
        let wb = if kind.shares_a_point() { wa } else { wb };
        Some((
            ph2d_physics_ecs::PhysicsWorld::local_anchor_at_pose(pose(a)?, wa),
            ph2d_physics_ecs::PhysicsWorld::local_anchor_at_pose(pose(b)?, wb),
            wa,
            wb,
        ))
    });
    // The display pivot: the A anchor when the gesture named one (the value
    // `sync_joint_pivots` will keep deriving), the midpoint otherwise.
    let origin = anchored.map_or((pa + pb) * 0.5, |(_, _, wa, _)| {
        ph2d_core::Vec2::new(wa[0], wa[1])
    });
    let base = PhysicsJoint::of_kind(kind);
    // ⚠️ **A geometria autorada de uma POLIA é MAIS que as duas âncoras** — ela
    // tem ROLDANAS —, e `anchored: true` diz ao reconcile *"leia o que está
    // guardado, não semeie"*. Uma polia criada por este gesto saía com as duas
    // roldanas na **origem do mundo**, com a corda indo de cada corpo até lá (o
    // que o artista fotografou). A rota por SELEÇÃO nunca teve o defeito, porque
    // ela deixa `anchored: false` e o semeio do reconcile roda.
    //
    // Um sentinela, duas perguntas — então o gesto estabelece as DUAS metades,
    // pela MESMA `pulley_rig` que o resto do sistema chama. Uma cópia da regra de
    // montagem aqui divergiria dela na primeira vez que qualquer uma mudasse.
    let rig = (kind == JointKind::Pulley).then(|| {
        ph2d_physics_ecs::pulley_rig(
            [pa.x, pa.y],
            [pb.x, pb.y],
            anchored.map(|(_, _, wa, wb)| (wa, wb)),
        )
    });
    let joint = sim
        .world_mut()
        .spawn((
            Name::new(label.clone()),
            PhysicsJoint {
                body_a: stable_name_id(&name_a),
                body_b: stable_name_id(&name_b),
                local_a: anchored.map_or([0.0, 0.0], |(la, _, _, _)| la),
                local_b: anchored.map_or([0.0, 0.0], |(_, lb, _, _)| lb),
                anchored: anchored.is_some(),
                // Só a rota do CANVAS traz comprimento: ela nasce `anchored`, e
                // aí o reconcile não semeia. A da seleção deixa o semeio rodar.
                max_length: rig.and_then(|r| r.1).unwrap_or(base.max_length),
                // ⚠️ `of_kind`, not `default()` + `kind` — the numbers that carry
                // a unit have to be seeded in THIS kind's unit. Built the other
                // way, "Join As = Slider" gave a rail +-0.785 METRES of stroke
                // (the Pin's +-45 deg read as a length) while making a Pin and
                // switching it to Slider gave +-0.5. Same door for both routes.
                ..base
            },
            Transform::from_translation(origin),
        ))
        .id();
    // ⚠️ **As roldanas são ENTIDADES**, então criá-las é SPAWNÁ-LAS — não
    // preencher dois campos. É por isso que este gesto ficou maior: ele passou a
    // montar objetos, e é o que dá ao artista o resto do que ele pediu (mais
    // roldanas, apagar uma, nomeá-la, desfazer) sem nada além do que a Hierarquia
    // já faz com qualquer objeto.
    if let Some((wheels, _)) = rig {
        let rope = stable_name_id(&label);
        for (i, w) in wheels.into_iter().enumerate() {
            let order = u16::try_from(i).unwrap_or(0);
            sim.world_mut().spawn((
                Name::new(format!("{label} Wheel {}", order + 1)),
                ph2d_physics_ecs::PulleyWheel {
                    rope,
                    order,
                    radius: w.radius,
                    // Comum: o tambor diferencial do W4 é um segundo gesto.
                    radius_out: 0.0,
                    wrap: ph2d_physics_ecs::WrapSide::Auto,
                    motor_speed: 0.0,
                    // Uma roldana nasce no CENÁRIO: montá-la num corpo (a cadernal
                    // móvel do W3) é um SEGUNDO gesto, e não um default. Nomeado e não
                    // `..default()` porque estes são sítios de PRODUTO — o campo da
                    // próxima wave nasceria neutro aqui em silêncio.
                    body: 0,
                    local: [0.0, 0.0],
                    mounted: false,
                    break_enabled: false,
                    break_force: ph2d_physics_ecs::PulleyWheel::DEFAULT_BREAK_FORCE,
                },
                Transform::from_translation(ph2d_core::Vec2::new(w.centre[0], w.centre[1])),
            ));
        }
    }
    // Every root object gets an explicit z, or the tree falls back to sorting
    // by entity bits — which the undo's respawn changes (the W3-era lesson
    // that `assign_missing_root_order` exists for).
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    Some(joint)
}

/// The entity's name, assigning a unique one first if it has none.
pub(crate) fn ensure_named(sim: &mut SimWorld, entity: Entity, base: &str) -> Option<String> {
    if let Some(n) = sim.world().get::<Name>(entity)
        && !n.as_str().is_empty()
    {
        return Some(n.as_str().to_string());
    }
    let fresh = crate::name_unique::unique_name(sim, base);
    sim.world_mut()
        .get_entity_mut(entity)
        .ok()?
        .insert(Name::new(fresh.clone()));
    Some(fresh)
}
