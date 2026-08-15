//! **O MODO deste personagem** — irmão de `inspector_player_tests` por teto de
//! LOC, cortado por ASSUNTO: o pai responde *"como este corpo vira um player, e
//! o que ele guarda"*; aqui só se pergunta **como ele é movido**, e o que essa
//! escolha cala.

use super::*;

/// **O gesto do MODO leva a algum lugar, e VOLTA** (W-KinMove).
///
/// ⚠️ **As duas metades, e a segunda é a que quase shipou quebrada:** o
/// `build_player_info` recusava todo corpo que não fosse `Dynamic`, então clicar
/// `Kinematic` fazia a §14 inteira **DESAPARECER** — o artista escolhia o modo e
/// perdia o controle que o traria de volta. Um gate que só testasse a ida ficaria
/// verde sobre isso.
#[test]
fn switching_the_mode_writes_both_halves_and_the_section_survives_the_trip() {
    use ph2d_physics_ecs::PlayerMode;
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);

    // Ida: o componente E o corpo.
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Mode(1));
    let e = ph2d_ecs::Entity::from_bits(bits);
    assert_eq!(
        sim.world().get::<PlayerMode>(e).copied(),
        Some(PlayerMode::Kinematic),
        "o modo tem de ser escrito"
    );
    assert_eq!(
        sim.world().get::<RigidBody>(e).map(|b| b.kind),
        Some(BodyKind::Kinematic),
        "e o CORPO junto: pedir os dois em duas secoes e' a falha de duas-portas"
    );
    let info = build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG)
        .expect("a §14 tem de continuar VIVA num player cinematico");
    assert_eq!(info.mode_tag, 1, "e o chip tem de mostrar onde ele esta");

    // Volta: e o componente sai no neutro (o detach do `GravityScale`).
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Mode(0));
    assert_eq!(
        sim.world().get::<PlayerMode>(e).copied(),
        None,
        "no neutro o componente sai: um arquivo nao carrega um no-op"
    );
    assert_eq!(
        sim.world().get::<RigidBody>(e).map(|b| b.kind),
        Some(BodyKind::Dynamic)
    );
    assert_eq!(
        build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG)
            .unwrap()
            .mode_tag,
        0
    );
}

/// **Um corpo cinemático que NÃO é player continua fora da §14** — ele é dirigido
/// pela cena (um bake, uma curva), e oferecer *"Make Platform Player"* ali criaria
/// um player que a ponte não dirige.
#[test]
fn a_scene_driven_kinematic_body_is_not_offered_the_section() {
    let (sim, bits) = body(BodyKind::Kinematic, CAPSULE);
    assert!(build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).is_none());
}

/// **O EMPURRÃO autorado chega ao caixote** (W-KinPush) — a quarta condição.
///
/// ⚠️ **A ponta que nenhum outro gate cobre é o MEIO:** o seam prova que o
/// clique vira um `ReactionPush`, e o `player_push::a_walking_player_shoves_a_loose_crate`
/// prova que o componente move o caixote. Entre os dois há a linha que ESCREVE o
/// componente, e uma escrita no campo errado deixaria os dois verdes com o
/// slider inerte.
///
/// ⚠️ **O modo é CINEMÁTICO de propósito:** sob Spring o solver empurra sozinho e
/// o gate ficaria verde com o canal inteiro deletado — *um gate que passa no
/// controle está a medir a coisa errada*.
#[test]
fn authoring_the_push_reaches_the_crate() {
    fn crate_travel(push: f32) -> f32 {
        let mut sim = SimWorld::new();
        sim.world_mut().spawn((
            Name::new("Floor"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 40.0,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, -0.5)),
        ));
        let boxy = sim
            .world_mut()
            .spawn((
                Name::new("Crate"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.3,
                        half_y: 0.3,
                    },
                    ..Collider::default()
                },
                ph2d_physics_ecs::LockRotation,
                Transform::from_translation(Vec2::new(1.5, 0.3)),
            ))
            .id();
        let hero = sim
            .world_mut()
            .spawn((
                Name::new("Hero"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: CAPSULE,
                    ..Collider::default()
                },
                ph2d_physics_ecs::LockRotation,
                Transform::from_translation(Vec2::new(0.0, 0.9)),
            ))
            .id();
        let bits = hero.to_bits();
        // O gesto do artista, pela porta do Inspector e nada mais.
        apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
        apply_player_edit(&mut sim, bits, PlayerFieldEdit::FloatHeight(0.9));
        apply_player_edit(&mut sim, bits, PlayerFieldEdit::Mode(1));
        apply_player_edit(&mut sim, bits, PlayerFieldEdit::ReactionPush(push));
        assert!(
            (build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG)
                .unwrap()
                .reaction_push
                - push)
                .abs()
                < 1.0e-6,
            "a row tem de MOSTRAR o que foi autorado"
        );

        let mut bridge = ph2d_physics_ecs::PhysicsBridge::new();
        for t in 1..=60u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let x0 = sim.world().get::<Transform>(boxy).unwrap().translation.x;
        bridge.set_player_input(
            hero,
            ph2d_physics_ecs::PlayerInput {
                drive: 1.0,
                ..ph2d_physics_ecs::PlayerInput::default()
            },
        );
        for t in 61..=240u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        sim.world().get::<Transform>(boxy).unwrap().translation.x - x0
    }

    let off = crate_travel(0.0);
    let on = crate_travel(1.0);
    assert!(
        off.abs() < 0.01,
        "com o slider em zero o caixote fica: {off:.4}"
    );
    assert!(
        on > 1.0,
        "e com ele em um o gesto do artista move o caixote: {on:.4} (zero deu {off:.4})"
    );
}

/// **Negativo vira zero**, nunca um empurrão invertido: o personagem PUXANDO o
/// caixote em que esbarra seria a lei com o sinal trocado.
#[test]
fn a_negative_push_is_refused_not_inverted() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::ReactionPush(-2.0));
    let info = build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).unwrap();
    assert!((info.reaction_push).abs() < 1.0e-6, "{info:?}");
}

/// **O TERCEIRO CHIP leva a algum lugar** (W-KinPure) — a quarta condição de UI
/// deste módulo, pela porta que o artista de facto usa.
///
/// ⚠️ O gesto é UM clique, e ele tem de fazer três coisas de uma vez: pôr o
/// componente, virar o corpo em cinemático e calar a 3ª lei. Um teste por-edit
/// ficaria verde com qualquer uma das três faltando.
#[test]
fn choosing_pure_turns_the_world_into_scenery() {
    fn crate_travel(mode_tag: u8) -> f32 {
        let mut sim = SimWorld::new();
        sim.world_mut().spawn((
            Name::new("Floor"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 40.0,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, -0.5)),
        ));
        let boxy = sim
            .world_mut()
            .spawn((
                Name::new("Crate"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.3,
                        half_y: 0.3,
                    },
                    ..Collider::default()
                },
                ph2d_physics_ecs::LockRotation,
                Transform::from_translation(Vec2::new(1.5, 0.3)),
            ))
            .id();
        let hero = sim
            .world_mut()
            .spawn((
                Name::new("Hero"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: CAPSULE,
                    ..Collider::default()
                },
                ph2d_physics_ecs::LockRotation,
                Transform::from_translation(Vec2::new(0.0, 0.9)),
            ))
            .id();
        let bits = hero.to_bits();
        apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
        apply_player_edit(&mut sim, bits, PlayerFieldEdit::FloatHeight(0.9));
        apply_player_edit(&mut sim, bits, PlayerFieldEdit::Mode(mode_tag));

        // ⚠️ **A PONTE nasce ANTES do info, e é o que torna este gate um
        // ORÁCULO** — a resposta que a §14 pinta é PERGUNTADA ao mesmo
        // `pose_owner` que a lei consulta, em vez de declarada aqui a partir do
        // `mode_tag`. Com uma constante o gate afirmaria o valor que ele
        // próprio passou: verde para qualquer mapeamento, inclusive o errado.
        let mut bridge = ph2d_physics_ecs::PhysicsBridge::new();
        for t in 1..=60u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let law = bridge.player_liveness(sim.world(), hero);

        let info = build_player_info(&sim, bits, 0.0, 0.0, None, law).unwrap();
        assert_eq!(info.mode_tag, mode_tag, "o chip tem de MOSTRAR o escolhido");
        assert_eq!(
            info.reaction_is_live,
            mode_tag != 2,
            "e o card REACTION segue quem o mundo ouve"
        );
        // ⚠️ **A OFERTA fica ao lado da MEDIÇÃO, de propósito** (report do Enio,
        // 2026-08-09): o `push` é lido só pelo cinemático, e o número que este
        // helper devolve logo abaixo é o que prova a frase. Um gate de painel
        // sozinho afirmaria que a row some; este afirma que ela some **onde
        // não faz nada** — que é a razão.
        assert_eq!(
            info.push_is_live,
            mode_tag == 1,
            "o `Push on Bodies` so' e' oferecido a quem o le'"
        );
        // ⚠️ E a MESMA porta responde pela perna: só o dinâmico tem mola, e é
        // por isso que as três rows dela seguem o modo (auditoria de 15/08).
        assert_eq!(
            info.spring_is_live,
            mode_tag == 0,
            "a perna elastica e' do DINAMICO; os outros dois pousam"
        );

        let x0 = sim.world().get::<Transform>(boxy).unwrap().translation.x;
        bridge.set_player_input(
            hero,
            ph2d_physics_ecs::PlayerInput {
                drive: 1.0,
                ..ph2d_physics_ecs::PlayerInput::default()
            },
        );
        for t in 61..=240u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        sim.world().get::<Transform>(boxy).unwrap().translation.x - x0
    }

    let snap = crate_travel(1);
    let pure = crate_travel(2);
    assert!(snap > 1.0, "o CONTROLE empurra: {snap:.4} m");
    assert!(
        pure.abs() < 0.01,
        "e o puro sangue nao: {pure:.4} m (contra {snap:.4})"
    );
    // ⚠️ **E o DINÂMICO fecha o triângulo:** ele empurra sem o knob, porque quem
    // o empurra é o SOLVER (16,55 m medidos pela sonda da W-KinPush contra
    // 0,0000 do cinemático sem a wave). É por isso que oferecer-lhe o slider era
    // um controle morto, e é por isso que escondê-lo não lhe tira nada.
    let dynamic = crate_travel(0);
    assert!(
        dynamic > 1.0,
        "o dinamico empurra pelo solver, sem knob nenhum: {dynamic:.4} m"
    );
}

/// **A FRAÇÃO da 3ª lei é uma fração, e a caixa de texto tem de o honrar**
/// (auditoria de 2026-08-15).
///
/// ⚠️ **O teto `1,0` desta família é de RECURSO, não de gosto** — o próprio
/// registro das faixas o diz: *"acima de 1 o personagem devolveria mais do que
/// recebeu"*, quantidade de movimento inventada do nada. O slider parava em 1 e
/// a **escrita** só impunha o piso (`v.max(0.0)`), então digitar `5` na caixa
/// guardava um chão a levar cinco vezes o peso do personagem.
///
/// ⚠️ Isto é o oposto do slider dual: ali a caixa passa da faixa **de propósito**
/// (o arrasto é conforto, o disfuncional começa depois). Aqui não há faixa
/// confortável e um teto físico — há **um** número, e ele é o físico.
#[test]
fn the_reaction_fractions_are_fractions_in_the_text_box_too() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    let e = ph2d_ecs::Entity::from_bits(bits);

    for (edit, read) in [
        (
            PlayerFieldEdit::ReactionSupport(5.0),
            (|p: &PlatformPlayer| p.reaction_support) as fn(&PlatformPlayer) -> f32,
        ),
        (PlayerFieldEdit::ReactionMovement(5.0), |p| {
            p.reaction_movement
        }),
        (PlayerFieldEdit::ReactionPush(5.0), |p| p.reaction_push),
    ] {
        apply_player_edit(&mut sim, bits, edit);
        let v = read(sim.world().get::<PlatformPlayer>(e).unwrap());
        assert!(
            (v - 1.0).abs() < 1.0e-6,
            "uma fracao acima de 1 INVENTA quantidade de movimento: {edit:?} guardou {v}"
        );
    }

    // E o piso continua lá — negativo seria o personagem PUXANDO o chão.
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::ReactionSupport(-2.0));
    assert_eq!(
        sim.world()
            .get::<PlatformPlayer>(e)
            .unwrap()
            .reaction_support,
        0.0
    );

    // ⚠️ **E um NaN vira ZERO, não o teto** — `NaN.clamp(0,1)` devolve `NaN` em
    // Rust, e um NaN aqui viaja pelo impulso até à pose que o `readback`
    // escreve: o personagem desaparece do mundo com todos os números na tela a
    // parecerem certos. Zero é a resposta segura; a máxima seria uma escolha
    // inventada num caminho de erro.
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::ReactionPush(f32::NAN));
    assert_eq!(
        sim.world().get::<PlatformPlayer>(e).unwrap().reaction_push,
        0.0,
        "um NaN tem de virar ZERO -- clamp() sozinho o deixa passar"
    );
}
