//! **O que a TROCA e a CRIAÇÃO de tipo fazem com os números do joint.**
//!
//! Irmão de `inspector_joint_tests`, separado dele no cap de 600 LOC — e o corte
//! é por assunto: lá *o clique produz um joint que SEGURA*, aqui *o joint que
//! ele produz nasce (ou vira) com os números daquele TIPO*.
//!
//! É uma família própria porque cresce sozinha: cada tipo novo traz um número
//! que muda de significado ao ser reinterpretado — a unidade dos limites
//! (radianos ↔ metros), a do motor, e agora a ESCALA da mola (pendurar um corpo
//! ↔ suspender um veículo).

use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor::JointFieldEdit;
use ph2d_physics_ecs::{JointKind, PhysicsBridge, PhysicsJoint};

use super::inspector_joint::{
    apply_joint_edit, build_joint_info, create_joint, joint_with_edit, kind_of, tag_of,
};
use super::inspector_joint_tests::{registry, two_bodies};

/// **Creating a joint makes the KIND the artist chose** (gold standard: create
/// the type you want, not a Pin you convert in §12). Mutation-tested:
/// `create_joint` ignoring its `kind` and spawning the default Pin makes the
/// Spring/Rope/Weld iterations go red.
#[test]
fn create_joint_makes_the_requested_kind() {
    for kind in [
        JointKind::Pin,
        JointKind::Spring,
        JointKind::Rope,
        JointKind::Weld,
    ] {
        let (mut sim, hook, plank) = two_bodies(true);
        let joint = create_joint(&mut sim, hook.to_bits(), plank.to_bits(), kind).expect("join");
        let j = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
        assert_eq!(
            j.kind, kind,
            "create_joint ignored the chosen kind {kind:?}"
        );
    }
}

/// **Changing the kind re-seeds the anchor.** The anchor POLICY depends on the
/// kind — a Pin/Weld shares a point, a Spring/Rope anchors body B at its centre
/// — so a kind change marks the joint un-anchored, and the next reconcile
/// re-derives the body-local anchors under the new policy. Without it a Pin
/// turned into a Rope keeps the shared-point anchor and the rope hangs from the
/// wrong spot on body B. Mutation-tested: dropping `next.anchored = false` in
/// **Trocar para uma RODA re-semeia a MOLA, e voltar devolve o que era.**
///
/// `stiffness`/`damping` são um par de campos com dois donos: pendurar um corpo
/// (uma Spring, 30) e suspender um veículo (a suspensão de uma roda, 400 —
/// medido). Herdando o número da Spring, o carro senta no batente no primeiro
/// tick e nada na tela diz por quê; é o mesmo perigo que a unidade dos limites
/// e a do motor já resolvem, com ESCALA no lugar de unidade.
///
/// ⚠️ **A segunda metade é a que faz a regra ser "quando o PAPEL muda"** e não
/// "em toda troca": Wheel→Pin→Wheel tem de devolver o número que o artista
/// digitou, que é a promessa que este componente faz sobre trocar de tipo.
#[test]
fn switching_to_a_wheel_re_seeds_the_spring_and_switching_back_keeps_it() {
    let (mut sim, hook, plank) = two_bodies(true);
    let joint =
        create_joint(&mut sim, hook.to_bits(), plank.to_bits(), JointKind::Spring).expect("join");
    let reg = registry();

    let edit = |sim: &SimWorld, tag: u8| {
        let queue = EditorCommandQueue::default();
        apply_joint_edit(
            sim,
            joint.to_bits(),
            JointFieldEdit::Kind(tag),
            &queue,
            &reg,
        );
        queue
    };

    // Uma mola com um número AUTORADO — não o default, senão "preservou" e
    // "re-semeou" seriam indistinguíveis.
    {
        let queue = EditorCommandQueue::default();
        apply_joint_edit(
            &sim,
            joint.to_bits(),
            JointFieldEdit::Stiffness(77.0),
            &queue,
            &reg,
        );
        apply_editor_commands(sim.world_mut(), &queue, &reg).expect("commands apply");
    }
    assert_eq!(
        sim.world().get::<PhysicsJoint>(joint).unwrap().stiffness,
        77.0
    );

    // Spring → Wheel: o papel da mola mudou, então ela é re-semeada.
    let q = edit(&sim, 6);
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("commands apply");
    let j = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
    assert_eq!(j.kind, JointKind::Wheel, "o tipo não mudou");
    assert_eq!(
        j.stiffness,
        ph2d_physics_ecs::PhysicsJoint::default_spring(JointKind::Wheel)[0],
        "uma suspensão tinha de nascer com a rigidez MEDIDA dela, não com os 77 \
         que valiam para pendurar um corpo"
    );

    // Wheel → Pin → Wheel: o papel NÃO muda entre Pin e Wheel do ponto de vista
    // da mola de um Pin (ele não tem), então o número da suspensão sobrevive.
    let q = edit(&sim, 0);
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("commands apply");
    let q = edit(&sim, 6);
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("commands apply");
    assert_eq!(
        sim.world().get::<PhysicsJoint>(joint).unwrap().stiffness,
        ph2d_physics_ecs::PhysicsJoint::default_spring(JointKind::Wheel)[0],
        "ida e volta tinha de devolver o valor da roda"
    );
}

/// `apply_joint_edit`'s Kind arm leaves it anchored and this goes red.
#[test]
fn changing_the_kind_re_seeds_the_anchor() {
    let (mut sim, hook, plank) = two_bodies(true);
    let joint =
        create_joint(&mut sim, hook.to_bits(), plank.to_bits(), JointKind::Pin).expect("join");

    // Seed the anchors: the first reconcile flips `anchored` to true.
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    assert!(
        sim.world().get::<PhysicsJoint>(joint).unwrap().anchored,
        "the pin should be seeded after a dispatch"
    );

    let reg = registry();
    let queue = EditorCommandQueue::default();
    apply_joint_edit(
        &sim,
        joint.to_bits(),
        JointFieldEdit::Kind(2), // Rope
        &queue,
        &reg,
    );
    apply_editor_commands(sim.world_mut(), &queue, &reg).expect("commands apply");

    let j = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
    assert_eq!(j.kind, JointKind::Rope, "the kind did not change");
    assert!(
        !j.anchored,
        "a kind change must mark the joint for re-seed, so the two-ended Rope \
         anchors body B at its centre instead of keeping the Pin's shared point"
    );
}

/// **A parameter edit only LANDS after the queue is flushed — the render loop's
/// job, and the one it forgot (W-JointParams, 2026-07-25).**
///
/// The user's report was in TWO parts: the bridge gated the re-describe on
/// `at_rest` (fixed in `bridge/joints.rs`), AND the shell's §12 edit block
/// pushed a `SetComponent` without flushing it — so a joint slider did nothing
/// until some OTHER Inspector edit happened to drain the queue ("às vezes
/// funciona"). This pins the fact that block must honour: `apply_joint_edit`
/// only QUEUES; the component changes on `apply_editor_commands`. The arch-gate
/// `the_joint_edit_loop_flushes_the_command_queue` proves the render loop calls
/// it; this proves that call is load-bearing.
#[test]
fn a_joint_param_edit_lands_only_when_the_queue_is_flushed() {
    let (mut sim, hook, plank) = two_bodies(true);
    let joint =
        create_joint(&mut sim, hook.to_bits(), plank.to_bits(), JointKind::Spring).expect("join");
    {
        let mut j = sim
            .world_mut()
            .get_mut::<PhysicsJoint>(joint)
            .expect("joint");
        j.stiffness = 30.0;
    }
    let reg = registry();
    let queue = EditorCommandQueue::default();

    // The edit the panel emits: stiffen the spring.
    apply_joint_edit(
        &sim,
        joint.to_bits(),
        JointFieldEdit::Stiffness(300.0),
        &queue,
        &reg,
    );
    // ⚠️ Before the flush the component is UNCHANGED — the edit is only queued.
    // This is the whole reason the render loop must flush; skipping it is what
    // made the slider inert.
    assert_eq!(
        sim.world().get::<PhysicsJoint>(joint).unwrap().stiffness,
        30.0,
        "apply_joint_edit must only QUEUE — if it wrote the component directly \
         the flush would not be load-bearing and the render-loop bug would be \
         invisible here"
    );

    apply_editor_commands(sim.world_mut(), &queue, &reg).expect("commands apply");
    assert_eq!(
        sim.world().get::<PhysicsJoint>(joint).unwrap().stiffness,
        300.0,
        "after the flush the component must carry the new stiffness — this is the \
         edit the render loop failed to flush, so the joint sat at k=30. The \
         bridge picking a flushed component change up and tightening the spring \
         is proven end to end in ph2d-physics-ecs/tests/joint_live_edit.rs"
    );
}

/// **O curso vai e volta na unidade do TIPO** (W-J5).
///
/// `limit_min/max` carregam radianos num Pin e metros num Slider — o modelo do
/// próprio rapier (um campo `limits`, pertencente ao grau de liberdade que o
/// joint deixou livre). Este gate pina o par de portas: o que o artista digita na
/// row Min é o que a row Min mostra no frame seguinte, nos DOIS tipos.
///
/// Mutação: `limit_in`/`limit_out` ignorarem o tipo (converter sempre) ⇒ o
/// Slider volta 0,5 m como 28,6 e isto fica vermelho.
#[test]
fn a_limit_round_trips_in_its_kinds_own_unit() {
    for (kind_tag, typed) in [(0u8, 45.0_f32), (4, 0.5)] {
        let base = PhysicsJoint {
            kind: kind_of(kind_tag),
            limits_enabled: true,
            ..PhysicsJoint::default()
        };
        let after =
            joint_with_edit(base, JointFieldEdit::LimitMax(typed)).expect("a limit edit lands");
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((Name::new("J"), after, Transform::default()))
            .id();
        let info = build_joint_info(&mut sim, e.to_bits(), 0).expect("info");
        assert!(
            (info.limit_max_ui - typed).abs() < 1e-3,
            "kind {kind_tag}: digitou {typed}, a row mostra {}",
            info.limit_max_ui
        );
        // ⚠️ **E o número GUARDADO, que é o que chega ao solver.** O round-trip
        // sozinho é oráculo fraco: um par de conversões consistentemente ERRADO
        // (converter sempre, ignorando o tipo) vai e volta perfeitamente enquanto
        // o trilho fica 57x curto. A mutação que apaga o `if limits_in_metres`
        // sobreviveu à asserção acima e é esta que a mata.
        let stored = if kind_tag == 4 {
            typed // metros, verbatim
        } else {
            typed.to_radians()
        };
        assert!(
            (after.limit_max - stored).abs() < 1e-4,
            "kind {kind_tag}: {typed} tem de VIRAR {stored} no componente, got {}",
            after.limit_max
        );
    }
}

/// **Trocar entre dobradiça e trilho RE-SEMEIA o alcance** — e trocar entre dois
/// tipos da mesma unidade NÃO.
///
/// Sem a re-semeadura os ±45° de um Pin (±0,785 rad) viram ±0,785 **metros** de
/// curso, um número que ninguém digitou. Com ela sempre, Pin→Weld→Pin jogaria
/// fora os ângulos do artista — que é a promessa que o componente faz sobre
/// trocar de tipo.
///
/// Mutação: re-semear em toda troca ⇒ a 2ª metade fica vermelha; nunca re-semear
/// ⇒ a 1ª.
#[test]
fn changing_the_limit_unit_re_seeds_the_range_and_nothing_else_does() {
    let pin = PhysicsJoint {
        kind: JointKind::Pin,
        limit_min: -0.4,
        limit_max: 0.4,
        ..PhysicsJoint::default()
    };
    // Pin -> Slider: a unidade muda, o alcance é re-semeado em METROS.
    let slider = joint_with_edit(pin, JointFieldEdit::Kind(4)).expect("kind edit");
    let want = PhysicsJoint::default_limits(JointKind::Slider);
    assert!(
        (slider.limit_max - want[1]).abs() < 1e-6,
        "Pin->Slider tem de re-semear o curso, got {}",
        slider.limit_max
    );
    // Pin -> Weld: MESMA unidade, o alcance do artista sobrevive.
    let weld = joint_with_edit(pin, JointFieldEdit::Kind(3)).expect("kind edit");
    assert!(
        (weld.limit_max - 0.4).abs() < 1e-6,
        "Pin->Weld tem de PRESERVAR os angulos, got {}",
        weld.limit_max
    );
}

/// **Todo chip que o painel oferece resolve para um tipo DISTINTO.**
///
/// O painel fala *tags* (ele nunca vê `ph2d-physics-ecs`), então `kind_of` e
/// `tag_of` são a única conversão — e enquanto ninguém as percorre inteiras, um
/// tipo novo pode chegar em UM dos lados e o chip entrega outra coisa **em
/// silêncio**. Foi exatamente o que uma mutação mostrou: mapear o tag do Rod
/// para `Pin` deixava a workspace inteira verde.
///
/// ⚠️ **O comprimento vem de `INSP_JOINT_KIND`, o array que o painel PINTA** —
/// não de uma lista escrita aqui, que nasceria desatualizada no dia em que o
/// sétimo tipo chegasse. É essa amarração que faz o gate crescer sozinho.
#[test]
fn every_kind_chip_the_panel_offers_round_trips_to_a_distinct_kind() {
    let chips = ph2d_editor::ids::INSP_JOINT_KIND.len();
    let mut seen: Vec<JointKind> = Vec::new();
    for tag in 0..chips {
        let tag = u8::try_from(tag).expect("a lista de chips cabe num u8");
        let kind = kind_of(tag);
        assert_eq!(
            tag_of(kind),
            tag,
            "o chip {tag} resolve para {kind:?}, que o shell devolve como tag {}",
            tag_of(kind)
        );
        assert!(
            !seen.contains(&kind),
            "o chip {tag} resolve para {kind:?}, que outro chip já entrega —              dois botões para o mesmo tipo, e um tipo sem botão nenhum"
        );
        seen.push(kind);
    }
}

/// **Uma POLIA nasce ARMADA pelas DUAS rotas de criação** — a por seleção e a do
/// canvas.
///
/// ## O defeito que este gate reproduz
///
/// O gesto do canvas (press em A → arrasta → solta em B) nomeia as âncoras, e
/// por isso nasce `anchored: true` — deliberado, senão a política de semeio
/// jogaria fora o ponto apontado. Mas o mesmo sentinela gateia o semeio do
/// RIG da polia, então uma polia criada assim ficava com `wheel_a == wheel_b ==
/// [0, 0]`: **as duas roldanas na origem do mundo**, com a corda saindo de cada
/// corpo até lá. Era o que o artista via, e a rota por seleção — que deixa
/// `anchored: false` — funcionava, o que fez o defeito parecer aleatório.
///
/// ⚠️ **O gate anterior (`a_fresh_pulley_seeds_its_wheels…`) monta o joint À MÃO
/// com `anchored: false`**: a fixture não continha o fenômeno, porque não
/// passava pelo gesto. Este passa pelos dois.
#[test]
fn a_pulley_is_rigged_by_both_creation_routes() {
    // A pose que a fixture monta: `Hook` em (0, 6), `Plank` em (0, 5).
    let (pa, pb) = ([0.0f32, 6.0], [0.0f32, 5.0]);
    for at in [None, Some((pa, pb))] {
        let route = if at.is_some() { "canvas" } else { "seleção" };
        let (mut sim, hook, plank) = two_bodies(true);
        let joint = super::inspector_joint::create_joint_at(
            &mut sim,
            hook.to_bits(),
            plank.to_bits(),
            JointKind::Pulley,
            at,
        )
        .expect("join");
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 1);
        let j = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
        let rope_id = ph2d_ecs::stable_name_id(
            sim.world()
                .get::<ph2d_ecs::Name>(joint)
                .expect("a corda tem nome")
                .as_str(),
        );

        // 1. As roldanas EXISTEM como objetos — o sintoma, direto: elas eram dois
        //    campos que ficavam em `[0, 0]`, e agora são duas entidades que o
        //    gesto tem de SPAWNAR.
        let wheels: Vec<([f32; 2], f32)> = {
            let mut q = sim
                .world_mut()
                .query::<(&ph2d_physics_ecs::PulleyWheel, &ph2d_ecs::Transform)>();
            let mut v: Vec<_> = q
                .iter(sim.world())
                .filter(|(w, _)| w.rope == rope_id)
                .map(|(w, t)| (w.order, [t.translation.x, t.translation.y], w.radius))
                .collect();
            v.sort_by_key(|(o, _, _)| *o);
            v.into_iter().map(|(_, c, r)| (c, r)).collect()
        };
        assert_eq!(
            wheels.len(),
            2,
            "{route}: o gesto tem de criar duas roldanas"
        );
        // 2. Cada uma fica ACIMA do seu corpo, que é o que uma roldana faz, e tem
        //    tamanho VISÍVEL — um raio zero seria o modelo de ponto de volta.
        for ((w, r), body, tag) in [(wheels[0], pa, "A"), (wheels[1], pb, "B")] {
            assert!(
                (w[0] - body[0]).abs() < 1.0e-4 && w[1] > body[1],
                "{route}: a roldana {tag} ({w:?}) não está acima do corpo {body:?}"
            );
            assert!(r > 0.0, "{route}: a roldana {tag} nasceu sem raio");
        }
        // 3. E a corda nasce EXATAMENTE esticada — medida aqui pela rota, dos
        //    números que ficaram guardados, sem re-chamar a `pulley_rig`.
        let world = |p: [f32; 2], local: [f32; 2]| [p[0] + local[0], p[1] + local[1]];
        let route_wheels: Vec<_> = wheels
            .iter()
            .map(
                |&(centre, radius)| ph2d_physics_ecs::rope_route::RopeWheel {
                    centre,
                    radius,
                    side: 1,
                },
            )
            .collect();
        let mut segs = Vec::new();
        let mut w = route_wheels.clone();
        let (ea, eb) = (world(pa, j.local_a), world(pb, j.local_b));
        ph2d_physics_ecs::rope_route::resolve_sides(ea, eb, &mut w, &mut segs);
        let taut = ph2d_physics_ecs::rope_route::route(ea, eb, &w, &mut segs)
            .expect("a rota existe")
            .length;
        assert!(
            (taut - j.max_length).abs() < 1.0e-3,
            "{route}: a corda mede {:.4} e a rota montada é {taut:.4}",
            j.max_length
        );
    }
}
