//! **A SEQUÊNCIA leva a algum lugar** (W5) — a quarta condição de UI que a
//! política deste módulo exige, e a que as outras três não implicam.
//!
//! O seam prova que o clique chega ao barramento; a paridade prova que o widget
//! é registrado; o `every_physics_component_is_authorable` prova que alguém o
//! escreve. **Nenhum dos três prova que o gesto INTEIRO produz um personagem que
//! anda** — foi essa a categoria que pegou o passo *"converta para Capsule"* que
//! quase entrou num roteiro de smoke: geometricamente correto, e destruía o
//! tronco.

use super::inspector_player::{apply_player_edit, build_player_info};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor::PlayerFieldEdit;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PlatformPlayer, RigidBody};

fn body(kind: BodyKind, shape: ColliderShape) -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Hero"),
            RigidBody { kind },
            Collider {
                shape,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    (sim, e.to_bits())
}

const CAPSULE: ColliderShape = ColliderShape::Capsule {
    half_height: 0.3,
    radius: 0.2,
};

/// **O gesto inteiro:** um corpo Dynamic vê a face vazia, o clique cria o
/// player, e os números vêm do ponto de partida da LEI.
#[test]
fn the_empty_face_becomes_a_player_with_the_laws_starting_point() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    let before = build_player_info(&sim, bits, 0.0, 0.0).expect("todo corpo Dynamic tem a §14");
    assert!(!before.has_player, "ele ainda nao e' um player");

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    let after = build_player_info(&sim, bits, 0.0, 0.0).expect("a secao continua viva");
    assert!(after.has_player);
    assert_eq!(after.speed, 6.0, "a velocidade do ponto de partida");
    assert_eq!(after.max_slope_deg, 45.0);
}

/// ⚠️ **E ele nasce PAIRANDO, não tangente.**
///
/// O ponto de partida do modelo (`0,5`) deixa esta cápsula exatamente tangente
/// ao chão — ela não flutua, e só uma rampa revela. O `Add` conhece a forma e
/// sobe a altura acima do piso; sem isso a primeira impressão do artista seria
/// um personagem encostado num app cuja tese é que ele paira.
#[test]
fn a_new_player_floats_over_its_own_collider() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    let info = build_player_info(&sim, bits, 0.0, 0.0).unwrap();
    assert!(
        info.min_float_known,
        "uma capsula tem piso computavel — sem isto o resto do gate nao diz nada"
    );
    assert!(
        info.float_height > info.min_float_height,
        "ele tem de nascer ACIMA do piso: {:.3} vs o minimo {:.3}",
        info.float_height,
        info.min_float_height
    );
}

/// O botão de ajuste conserta uma altura curta autorada à mão.
#[test]
fn fit_to_collider_raises_a_short_float_height() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FloatHeight(0.2));
    let short = build_player_info(&sim, bits, 0.0, 0.0).unwrap();
    assert!(
        short.float_height < short.min_float_height,
        "a fixture TEM de conter o defeito"
    );

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FitFloatHeight);
    let fixed = build_player_info(&sim, bits, 0.0, 0.0).unwrap();
    assert!(
        fixed.float_height > fixed.min_float_height,
        "o ajuste tem de passar do piso: {:.3} vs {:.3}",
        fixed.float_height,
        fixed.min_float_height
    );
}

/// ⚠️ **Um corpo que não é Dynamic não tem a §14, e a recusa é dupla.**
///
/// A mola é um impulso, e um impulso não move massa infinita. O pintor não a
/// oferece (o info é `None`) **e** o barramento não a honra — porque uma recusa
/// que mora só no laço de pintura não é recusa: os ids vivem no store a sessão
/// inteira, e um clique roteado por outra coisa chegaria aqui.
#[test]
fn a_static_body_gets_neither_the_section_nor_the_write() {
    for kind in [BodyKind::Static, BodyKind::Kinematic] {
        let (mut sim, bits) = body(kind, CAPSULE);
        assert!(
            build_player_info(&sim, bits, 0.0, 0.0).is_none(),
            "{kind:?} nao pode receber a secao"
        );
        apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
        assert!(
            sim.world()
                .get::<PlatformPlayer>(ph2d_ecs::Entity::from_bits(bits))
                .is_none(),
            "{kind:?} nao pode receber o componente nem por um clique roteado"
        );
    }
}

/// **Remover devolve o corpo a um corpo comum** — e a seção continua viva, com a
/// face vazia, para que ele possa voltar a ser um player.
#[test]
fn remove_gives_the_body_back_and_keeps_the_door_open() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Remove);
    let info = build_player_info(&sim, bits, 0.0, 0.0).expect("a secao NAO some com o componente");
    assert!(!info.has_player);
}

/// ⚠️ **O amortecimento é clampado no TETO MEDIDO da lei** — acima dele o boost
/// inverte a velocidade em vez de matá-la, e o personagem pipoca.
///
/// Duas camadas: a porta da lei também clampa. Esta existe para o número
/// AUTORADO nunca guardar algo que o motor não vai honrar — um campo que mente
/// sobre si mesmo é pior que um clamp invisível.
#[test]
fn the_damping_is_clamped_to_the_measured_ceiling() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::SpringDamping(5.0));
    let info = build_player_info(&sim, bits, 0.0, 0.0).unwrap();
    assert_eq!(
        info.spring_damping,
        ph2d_physics_ecs::RideConfig::MAX_DAMPING
    );
}

/// ⚠️ **Uma CAIXA não tem piso computável, e o info o diz** em vez de devolver
/// a fórmula da cápsula.
///
/// A extensão de uma cápsula ao longo de uma normal é `radius + hh·cos θ` (o
/// raio é isotrópico) e a de uma caixa é `half_x·sin θ + half_y·cos θ` — outra
/// fórmula, outro piso. Um número errado apresentado como certo é pior que a
/// ausência dele.
#[test]
fn a_box_reports_no_known_floor() {
    let (sim, bits) = body(
        BodyKind::Dynamic,
        ColliderShape::Cuboid {
            half_x: 0.3,
            half_y: 0.5,
        },
    );
    let info = build_player_info(&sim, bits, 0.0, 0.0).unwrap();
    assert!(!info.min_float_known);
}

/// ⚠️ **As duas assistências da W10 chegam ao COMPONENTE, e o clamp é o do
/// barramento** — a quarta condição para a wave nova.
///
/// O seam prova que o número levanta a edição; este prova que a edição pousa no
/// personagem, e que um valor negativo (que o chip aceita digitar) vira zero em
/// vez de um alcance ao contrário.
#[test]
fn the_two_w10_assists_land_on_the_component_and_are_clamped() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::CornerReach(0.2));
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::LiftMomentum(0.8));
    let info = build_player_info(&sim, bits, 0.0, 0.0).unwrap();
    assert!((info.corner_reach - 0.2).abs() < 1.0e-6, "{info:?}");
    assert!((info.lift_momentum - 0.8).abs() < 1.0e-6, "{info:?}");

    // Negativo não é uma direção nem uma janela: vira o desligado.
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::CornerReach(-1.0));
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::LiftMomentum(-1.0));
    let clamped = build_player_info(&sim, bits, 0.0, 0.0).unwrap();
    assert_eq!((clamped.corner_reach, clamped.lift_momentum), (0.0, 0.0));
}

/// **O `Fit Crouch` semeia a altura agachada pelo MESMO piso da perna de pé**
/// (W18).
///
/// ⚠️ **A mesma função, e é o desenho inteiro:** o piso é da FORMA e da rampa, não
/// de qual perna está em uso. Uma segunda fórmula para a de baixo divergiria no
/// dia em que a caixa ganhasse a dela.
///
/// ⚠️ **A PERNA DE PÉ é deliberadamente ALTA na fixture, e é isso que dá dentes
/// ao gate.** A primeira versão fazia `FitFloatHeight` antes, então a perna já
/// estava no valor ajustado — e *"semear do piso"* e *"copiar a perna de pé"*
/// davam **o mesmo número**: a mutação `p.crouch_height = p.float_height`
/// **sobreviveu**, sobre um gate escrito exatamente para a pegar. Com a perna em
/// `1,50` as duas respostas ficam a um metro de distância.
///
/// ⚠️ **Mutações medidas:** `p.float_height` no lugar do `fitted_float` dá `1,500`
/// contra os `0,600` do piso; e um `fitted_float` sem o `min` deixa o agachar
/// passar da perna (gate irmão).
#[test]
fn fitting_the_crouch_seeds_it_from_the_floor_not_from_the_standing_leg() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    // Uma perna de pé BEM acima do que o piso pede — a premissa que separa as
    // duas respostas possíveis.
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FloatHeight(1.50));
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::CrouchHeight(0.10));

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FitCrouchHeight);
    let info = build_player_info(&sim, bits, 0.0, 0.0).unwrap();

    assert!(
        info.crouch_height > info.min_float_height,
        "o Fit deixou o agachar ABAIXO do piso ({:.3} <= {:.3}): e' exatamente o \
         estado que ele existe para consertar",
        info.crouch_height,
        info.min_float_height
    );
    assert!(
        info.crouch_height < info.float_height * 0.5,
        "o Fit copiou a PERNA DE PE ({:.3} contra {:.3}) em vez de derivar o piso -- \
         um agachar que nao agacha",
        info.crouch_height,
        info.float_height
    );
}

/// **E o `Fit Crouch` nunca passa da perna de pé** — a metade que só morde numa
/// cápsula cujo piso já está acima da altura autorada.
///
/// ⚠️ Sem o `min`, um corpo gordo (piso alto) com uma perna curta receberia um
/// agachar mais alto do que estar em pé, e o `crouch_step` passaria a LEVANTAR o
/// personagem quando ele segurasse BAIXO.
#[test]
fn a_fitted_crouch_never_rises_above_the_standing_leg() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    // Uma perna deliberadamente MAIS CURTA que o piso da forma.
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FloatHeight(0.20));
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::CrouchHeight(0.10));
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FitCrouchHeight);
    let info = build_player_info(&sim, bits, 0.0, 0.0).unwrap();
    assert!(
        info.crouch_height <= info.float_height,
        "o agachar ({:.3}) passou da perna de pe ({:.3})",
        info.crouch_height,
        info.float_height
    );
}

/// **A §14 NÃO É CASA PARA A CORRIDA GRAVADA** (W25) — a medição que abriu a
/// wave, e ela afirma um DEFEITO do desenho antigo, não uma cura.
///
/// A fita de entrada é um fato do **DOCUMENTO**: ela viaja no arquivo (W17) e é
/// o que o Bake replaya (W16). Os dois botões que a governam moravam só aqui,
/// numa seção que **só existe sobre um corpo Dynamic selecionado** — e este gate
/// mede exatamente isso: apagar o personagem, ou selecionar qualquer outra
/// coisa, e a corrida fica presa no documento sem gesto que a alcance.
///
/// ⚠️ **O oráculo é a AUSÊNCIA da seção**, não a ausência do botão: com
/// `build_player_info` em `None` o painel não pinta nada, então nenhum readout e
/// nenhum verbo existem — é por isso que a cura teve de ser uma segunda VISTA
/// noutro painel, e não um botão a mais dentro deste.
///
/// **Mutação que deve sangrar:** fazer o `build_player_info` devolver `Some`
/// para qualquer entidade — o que consertaria o alcance e traria de volta a §14
/// sobre um corpo Static, onde a mola (um impulso) não move massa infinita.
#[test]
fn the_player_section_is_no_home_for_a_document_wide_run() {
    // 1. O corpo não é Dynamic: a §14 não existe, e com ela nenhum verbo de fita.
    let (sim, bits) = body(BodyKind::Static, CAPSULE);
    assert!(
        build_player_info(&sim, bits, 4.0, 0.0).is_none(),
        "a §14 nasceu sobre um corpo Static -- a mola e' um impulso e nao move \
         massa infinita"
    );

    // 2. E o personagem foi APAGADO, que é o caso que fecha o argumento: a
    //    corrida sobrevive a ele (está no arquivo), a seção não.
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    assert!(
        build_player_info(&sim, bits, 4.0, 0.0).is_some(),
        "o controle: enquanto o player existe a §14 mostra a corrida"
    );
    sim.world_mut().despawn(ph2d_ecs::Entity::from_bits(bits));
    assert!(
        build_player_info(&sim, bits, 4.0, 0.0).is_none(),
        "apagar o personagem tinha de levar a §14 embora -- e' isso que prendia \
         a corrida"
    );
}

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
    let info = build_player_info(&sim, bits, 0.0, 0.0)
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
    assert_eq!(build_player_info(&sim, bits, 0.0, 0.0).unwrap().mode_tag, 0);
}

/// **Um corpo cinemático que NÃO é player continua fora da §14** — ele é dirigido
/// pela cena (um bake, uma curva), e oferecer *"Make Platform Player"* ali criaria
/// um player que a ponte não dirige.
#[test]
fn a_scene_driven_kinematic_body_is_not_offered_the_section() {
    let (sim, bits) = body(BodyKind::Kinematic, CAPSULE);
    assert!(build_player_info(&sim, bits, 0.0, 0.0).is_none());
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
            (build_player_info(&sim, bits, 0.0, 0.0)
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
    let info = build_player_info(&sim, bits, 0.0, 0.0).unwrap();
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

        let info = build_player_info(&sim, bits, 0.0, 0.0).unwrap();
        assert_eq!(info.mode_tag, mode_tag, "o chip tem de MOSTRAR o escolhido");
        assert_eq!(
            info.reaction_is_live,
            mode_tag != 2,
            "e o card REACTION segue quem o mundo ouve"
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

    let snap = crate_travel(1);
    let pure = crate_travel(2);
    assert!(snap > 1.0, "o CONTROLE empurra: {snap:.4} m");
    assert!(
        pure.abs() < 0.01,
        "e o puro sangue nao: {pure:.4} m (contra {snap:.4})"
    );
}
