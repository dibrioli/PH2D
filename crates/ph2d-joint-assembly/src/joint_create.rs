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

use ph2d_ecs::{Entity, Name, SimWorld, Transform};
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
pub fn create_joint(
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
pub fn create_joint_at(
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
    // ⚠️ MUNDO, não `Transform` cru (LOCAL, compõe com o pai — W5): na cena 67 o pescoço nascia
    // 1,65 m abaixo da emenda.
    let pa = ph2d_ecs::world_transform(sim.world(), a)?.translation;
    let pb = ph2d_ecs::world_transform(sim.world(), b)?.translation;

    let name_a = ensure_named(sim, a, "Body")?;
    let name_b = ensure_named(sim, b, "Body")?;

    // ⭐ A guarda «nomes iguais ⇒ um id» dissolveu com o `StableId` (ADR-0164 F1): homónimos têm
    // ids diferentes. Pela ENTIDADE, não pelo nome (`stable_id_for_name` é para as cenas).
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let id_a = ph2d_ecs::stable_id_of(sim.world(), a).map_or(0, |s| s.0);
    let id_b = ph2d_ecs::stable_id_of(sim.world(), b).map_or(0, |s| s.0);
    // **A joint is named after what it joins** (W-J8) — the Unreal Constraints
    // Graph idiom. "Post : Plank" in the Hierarchy is a joint you can find; the
    // "Joint (3)" it replaces is a row you have to click to identify, in a rig
    // where every row looks the same.
    //
    // ⚠️ A snapshot at creation, not a live binding: the name is the artist's (the BINDING follows
    // a rename; the label does not).
    let label = ph2d_unique_name::unique_name(sim, &format!("{name_a} : {name_b}"));
    // The authored poses of the two bodies — what a body-local anchor is
    // measured against (the seed uses the same `rest`, and using the LIVE pose
    // would bake a swing into the local; W-AnchorFollow).
    // A pose de MUNDO, a mesma correção do ponto médio.
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
    // **A EMENDA**, quando o gesto não nomeou um ponto (W-Rig): onde as SILHUETAS (da forma
    // autorada, olho aberto ou não — opção B) se encontram na linha entre os centros — o meio punha o
    // pivô de uma cabeça pequena dentro do tronco. Formas iguais reduzem ao meio; sem collider, o meio.
    let silhueta = |e: Entity| {
        let w = sim.world();
        let f = ph2d_physics_ecs::forma_para_editar(w, e)?;
        w.get::<ph2d_physics_ecs::Collider>(f)
            .map(|c| ph2d_physics_ecs::forma_resolvida(w, f, c))
            .zip(ph2d_ecs::world_transform(w, f))
    };
    let (sa, sb) = (silhueta(a), silhueta(b));
    let seam = ph2d_physics_ecs::seam_between(
        sa.as_ref().map(|(c, t)| (c, *t)),
        sb.as_ref().map(|(c, t)| (c, *t)),
    )
    .unwrap_or((pa + pb) * 0.5);
    // The display pivot: the A anchor when the gesture named one (the value
    // `sync_joint_pivots` will keep deriving), the seam otherwise.
    let origin = anchored.map_or(seam, |(_, _, wa, _)| ph2d_core::Vec2::new(wa[0], wa[1]));
    let base = PhysicsJoint::of_kind(kind);
    // ⚠️ Uma POLIA autora também as ROLDANAS (`anchored: true` diz ao reconcile «não semeie»; sem
    // isto nasciam na origem do mundo) — pela MESMA `pulley_rig` do resto do sistema.
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
                body_a: id_a,
                body_b: id_b,
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
        // ⚠️ A corda é apontada pela IDENTIDADE do joint que acabou de nascer — e por isso
        // a varredura corre AQUI, depois do spawn: sem ela o joint ainda não tem id, e as
        // roldanas nasceriam a apontar para `0` (nenhuma corda).
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        let rope = ph2d_ecs::stable_id_of(sim.world(), joint).map_or(0, |s| s.0);
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
pub fn ensure_named(sim: &mut SimWorld, entity: Entity, base: &str) -> Option<String> {
    if let Some(n) = sim.world().get::<Name>(entity)
        && !n.as_str().is_empty()
    {
        return Some(n.as_str().to_string());
    }
    let fresh = ph2d_unique_name::unique_name(sim, base);
    sim.world_mut()
        .get_entity_mut(entity)
        .ok()?
        .insert(Name::new(fresh.clone()));
    Some(fresh)
}

/// Re-bind slot A (or B, if `slot_b`) of the joint at `joint_bits` to `target`
/// — the resolve half of the §12 eyedropper pick, called from `input_dispatch`
/// when the armed pick's next canvas click lands on a body. Returns whether the
/// bind took (so the caller clears the armed pick only on success).
///
/// Written in place (the diff undo sees it), through `clamped()`. ⚠️ Refuses a self-joint:
/// `false`, and the pick stays armed.
#[must_use]
pub fn set_joint_body(sim: &mut SimWorld, joint_bits: u64, slot_b: bool, target: Entity) -> bool {
    let joint_e = Entity::from_bits(joint_bits);
    let Some(&current) = sim.world().get::<PhysicsJoint>(joint_e) else {
        return false;
    };
    let Some(name) = ensure_named(sim, target, "Body") else {
        return false;
    };
    // ⚠️ A IDENTIDADE do alvo, pela entidade que este sítio tem em mãos. O `name` continua
    // garantido (um corpo sem nome é ilegível na Hierarquia), mas não é ele que se guarda.
    let _ = &name;
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let hash = ph2d_ecs::stable_id_of(sim.world(), target).map_or(0, |s| s.0);
    let mut next = current;
    if slot_b {
        next.body_b = hash;
    } else {
        next.body_a = hash;
    }
    if next.body_a == next.body_b {
        return false; // a self-joint is dormant — keep the pick armed instead
    }
    // Outro corpo: a âncora local perde o sentido; o reconcile re-deriva-a do pivô visível.
    next.anchored = false;
    let next = next.clamped();
    if next != current
        && let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(joint_e)
    {
        *j = next;
    }
    // Escolher o corpo B É escolher «Body»: o rótulo diria «World» sobre um corpo escolhido, e o
    // clique em «Body» sem corpo B (recusado por `anchor_to_world`) só termina aqui.
    if slot_b {
        sim.world_mut()
            .entity_mut(joint_e)
            .remove::<ph2d_physics_ecs::JointWorldAnchor>();
    }
    true
}
