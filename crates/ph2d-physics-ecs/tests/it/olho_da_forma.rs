//! **O olho da forma esconde E desliga** (GEF02 B8, smoke do dono 08/11).
//!
//! Um corpo com DUAS formas filhas: o olho de uma (a principal ou a peça) desliga só ela; a outra
//! continua a colidir. Ligar o olho devolve a forma.

use ph2d_core::Vec2;
use ph2d_ecs::row_flag::{RowFlag, flip};
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, Visibility};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody, forma_desligada, main_shape,
};

fn caixa(h: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: h,
            half_y: h,
        },
        density: 1.0,
        ..Collider::default()
    }
}

/// Chão estático (topo em `y = 0,5`) e o corpo «Duo» com as formas filhas A (esquerda, a principal)
/// e B (direita, a peça), de olho aberto.
fn cena() -> (SimWorld, Entity, Entity, Entity) {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let chao = w
        .spawn((
            Name::new("Floor"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    w.spawn((
        Name::new("Collision Shape"),
        caixa(20.0).with_half_y(0.5),
        Transform::IDENTITY,
        ChildOf(chao),
    ));
    let duo = w
        .spawn((
            Name::new("Duo"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Transform::from_translation(Vec2::new(0.0, 3.0)),
        ))
        .id();
    let a = w
        .spawn((
            Name::new("Collision Shape"),
            caixa(0.3),
            Transform::from_translation(Vec2::new(-1.0, 0.0)),
            ChildOf(duo),
        ))
        .id();
    let b = w
        .spawn((
            Name::new("Collision Shape"),
            caixa(0.3),
            Transform::from_translation(Vec2::new(1.0, 0.0)),
            ChildOf(duo),
        ))
        .id();
    (sim, duo, a, b)
}

trait MeiaAltura {
    fn with_half_y(self, hy: f32) -> Self;
}
impl MeiaAltura for Collider {
    fn with_half_y(mut self, hy: f32) -> Self {
        if let ColliderShape::Cuboid { half_y, .. } = &mut self.shape {
            *half_y = hy;
        }
        self
    }
}

fn corre(sim: &mut SimWorld, bridge: &mut PhysicsBridge, de: u64, ate: u64) {
    for t in de..=ate {
        bridge.dispatch(sim, true, t);
    }
}

fn y(sim: &SimWorld, e: Entity) -> f32 {
    ph2d_ecs::world_transform(sim.world(), e)
        .expect("transform")
        .translation
        .y
}

fn olho(sim: &mut SimWorld, e: Entity) {
    flip(sim.world_mut(), e, RowFlag::Visible);
}

/// **(2) do smoke:** desligar UMA forma — a principal ou a peça, antes do play ou a meio — nunca
/// desliga a outra: ela continua em cima do chão (o centro de uma caixa de `0,3` pousada fica em
/// `~0,8`; afundada passa de `0,5`).
#[test]
fn desligar_uma_forma_nao_desliga_a_outra() {
    for desliga_a in [true, false] {
        for a_meio in [false, true] {
            let (mut sim, _duo, a, b) = cena();
            let mut bridge = PhysicsBridge::new();
            let (off, on) = if desliga_a { (a, b) } else { (b, a) };
            // A meio = o relógio já andou (o corpo caiu e pousou nas duas): a principal muda com o
            // corpo vivo no solver.
            let inicio = if a_meio { 120 } else { 0 };
            if a_meio {
                corre(&mut sim, &mut bridge, 0, inicio - 1);
            }
            olho(&mut sim, off);
            assert!(forma_desligada(sim.world(), off));
            corre(&mut sim, &mut bridge, inicio, inicio + 240);
            // A ligada é a PRINCIPAL nos dois casos (a A desligada passa-lhe o papel; a B desligada
            // deixa a A onde estava): viaja no corpo, nunca como peça.
            assert_eq!(main_shape(sim.world(), _duo), Some(on));
            assert!(bridge.body_pose(_duo).is_some(), "o corpo saiu do solver");
            assert!(bridge.part_handle(on).is_none(), "a principal virou peça");
            assert!(bridge.part_handle(off).is_none(), "a desligada não é peça");
            // A caixa do corpo NO SOLVER cobre só a ligada (cada uma tem meia largura 0,3).
            let (mins, maxs) = bridge.body_extent(_duo).expect("o corpo está no solver");
            let x_on = ph2d_ecs::world_transform(sim.world(), on)
                .unwrap()
                .translation
                .x;
            assert!(
                (mins[0] - (x_on - 0.3)).abs() < 0.05 && (maxs[0] - (x_on + 0.3)).abs() < 0.05,
                "({}) o solver tem {mins:?}..{maxs:?}, não só a ligada em x = {x_on:.2}",
                if a_meio { "a meio" } else { "em repouso" },
            );
            assert!(
                y(&sim, on) > 0.6,
                "desligar {} ({}) fez a outra afundar até {:.3}",
                if desliga_a {
                    "A, a principal"
                } else {
                    "B, a peça"
                },
                if a_meio { "a meio" } else { "em repouso" },
                y(&sim, on)
            );
        }
    }
}

/// **(1) do smoke:** o olho fechado tira a forma do solver (ela afunda no chão, a outra não); aberto
/// de novo, ela volta. A principal passa à forma seguinte enquanto a primeira está desligada.
#[test]
fn o_olho_da_forma_desliga_e_liga_o_collider() {
    let (mut sim, duo, a, b) = cena();
    let mut bridge = PhysicsBridge::new();
    assert_eq!(main_shape(sim.world(), duo), Some(a));
    olho(&mut sim, a);
    assert_eq!(
        main_shape(sim.world(), duo),
        Some(b),
        "a principal passa à seguinte"
    );
    assert!(sim.world().get::<Visibility>(a).is_some_and(|v| v.hidden));
    corre(&mut sim, &mut bridge, 0, 0);
    assert!(
        bridge.part_handle(a).is_none(),
        "a forma desligada não é peça"
    );
    assert!(
        bridge.part_handle(b).is_none(),
        "B é a principal: viaja no corpo"
    );
    olho(&mut sim, a);
    assert_eq!(main_shape(sim.world(), duo), Some(a));
    corre(&mut sim, &mut bridge, 0, 0);
    assert!(
        bridge.part_handle(b).is_some(),
        "com A de volta, B volta a ser peça"
    );
    // As duas desligadas (dono 09/11): o corpo FICA no solver, não toca nada e cai com a gravidade.
    olho(&mut sim, a);
    olho(&mut sim, b);
    assert_eq!(main_shape(sim.world(), duo), None);
    corre(&mut sim, &mut bridge, 0, 30);
    assert!(
        bridge.body_pose(duo).is_some(),
        "sem forma ligada o corpo saiu do solver"
    );
    let caindo = y(&sim, duo);
    assert!(
        caindo < 2.9 && caindo > 1.0,
        "meio segundo de queda livre: {caindo:.3}"
    );
    // O olho de A reabre-se A MEIO da queda: a forma volta e o corpo pousa no chão.
    olho(&mut sim, a);
    corre(&mut sim, &mut bridge, 31, 300);
    let pousado = y(&sim, duo);
    assert!(
        (pousado - 0.8).abs() < 0.05,
        "com A de volta o corpo pousa no topo do chão: {pousado:.3}"
    );
}

/// **Sem forma ligada nenhuma, atravessa o chão** — a outra metade: de olhos fechados desde o
/// repouso, o corpo passa pelo topo do chão (`0,5`) e segue a cair.
#[test]
fn as_formas_todas_desligadas_atravessam_o_chao() {
    let (mut sim, duo, a, b) = cena();
    let mut bridge = PhysicsBridge::new();
    olho(&mut sim, a);
    olho(&mut sim, b);
    corre(&mut sim, &mut bridge, 0, 120);
    assert!(bridge.body_pose(duo).is_some(), "o corpo saiu do solver");
    let fundo = y(&sim, duo);
    assert!(fundo < -2.0, "o corpo não atravessou o chão: {fundo:.3}");
}

/// Um corpo dinâmico de UMA forma filha, de olho fechado: cai, e a massa é a de um corpo sem
/// forma — `MASS_WITHOUT_SHAPE_KG` (1 kg, Box2D/Godot/Unity) — salvo um `MassOverride`, que manda.
#[test]
fn a_unica_forma_desligada_cai_com_a_massa_de_um_corpo_sem_forma() {
    for manual in [None, Some(5.0)] {
        let mut sim = SimWorld::new();
        let w = sim.world_mut();
        let rb = RigidBody {
            kind: BodyKind::Dynamic,
        };
        let solo = w
            .spawn((
                Name::new("Solo"),
                rb,
                Transform::from_translation(Vec2::new(0.0, 3.0)),
            ))
            .id();
        if let Some(m) = manual {
            w.entity_mut(solo).insert(ph2d_physics_ecs::MassOverride(m));
        }
        let f = w
            .spawn((caixa(0.3), Transform::IDENTITY, ChildOf(solo)))
            .id();
        olho(&mut sim, f);
        let mut bridge = PhysicsBridge::new();
        corre(&mut sim, &mut bridge, 0, 60);
        let massa = bridge.body_mass(solo).expect("o corpo está no solver");
        let esperada = manual.unwrap_or(ph2d_physics::MASS_WITHOUT_SHAPE_KG);
        assert!(
            (massa - esperada).abs() < 1e-5,
            "massa {massa} contra {esperada}"
        );
        assert!(
            y(&sim, solo) < 0.0,
            "um segundo de queda: {}",
            y(&sim, solo)
        );
    }
}

/// **O olho de um ANCESTRAL abaixo do corpo desliga as formas debaixo dele** (GEF02 fecho): um grupo
/// escondido entre o corpo e a forma, ou a filha visível de uma forma de olho fechado, não colidem.
/// E a chave/edição de um campo de forma vai à 1.ª forma mesmo desligada, nunca ao corpo.
#[test]
fn o_olho_de_um_ancestral_desliga_as_formas_debaixo_dele() {
    let mut w = ph2d_ecs::World::new();
    let rb = RigidBody {
        kind: BodyKind::Dynamic,
    };
    let corpo = w.spawn((Name::new("Body"), rb)).id();
    let grupo = w.spawn((Name::new("Group"), ChildOf(corpo))).id();
    let dentro = w.spawn((caixa(0.3), ChildOf(grupo))).id();
    w.entity_mut(grupo).insert(Visibility::hidden());
    assert!(
        forma_desligada(&w, dentro),
        "o grupo escondido não desligou a forma"
    );
    assert_eq!(main_shape(&w, corpo), None);
    w.entity_mut(grupo).insert(Visibility::default());
    w.entity_mut(dentro).insert(Visibility::hidden());
    let neta = w.spawn((caixa(0.2), ChildOf(dentro))).id();
    assert!(
        forma_desligada(&w, neta),
        "a filha da forma desligada colide"
    );
    assert_eq!(
        main_shape(&w, corpo),
        None,
        "a filha da forma desligada virou principal"
    );
    let campo = ph2d_ecs::canal_fisica::MEIO_X;
    assert!(ph2d_ecs::canal_fisica::e_da_forma(campo));
    let dono = ph2d_physics_ecs::dono_do_campo(&w, corpo.to_bits(), campo);
    assert_eq!(
        dono,
        dentro.to_bits(),
        "a chave da forma desligada foi para o corpo"
    );
}

/// **O «Collides with» de um corpo de formas todas desligadas mora na FORMA** (GEF02 09/11): a leitura
/// (`linha_efetiva`, o cartão) acha a mesma forma que a escrita alveja — nunca o corpo, que não tem
/// `Collider` e fazia o cartão mostrar «sem exceção».
#[test]
fn a_excecao_de_um_corpo_de_formas_desligadas_mora_na_forma() {
    use ph2d_physics_ecs::CollisionOverride;
    use ph2d_physics_ecs::excecao::{forma_da_excecao, linha_efetiva};
    let mut w = ph2d_ecs::World::new();
    let rb = RigidBody {
        kind: BodyKind::Dynamic,
    };
    let corpo = w.spawn((Name::new("Body"), rb)).id();
    let forma = w
        .spawn((caixa(0.3), Visibility::hidden(), ChildOf(corpo)))
        .id();
    let o = CollisionOverride::with_bit(u32::MAX, None, 0, false).expect("exceção");
    w.entity_mut(forma).insert(o);
    assert_eq!(main_shape(&w, corpo), None, "a forma devia estar desligada");
    assert_eq!(
        forma_da_excecao(&w, corpo),
        forma,
        "a exceção foi lida no corpo"
    );
    let (linha, difere) = linha_efetiva(&w, corpo).expect("o corpo perdeu a linha da forma");
    assert!(difere, "a exceção da forma desligada não aparece");
    assert_eq!(linha & 1, 0, "a camada 0 devia estar fora da linha");
}

/// **Um corpo SEM forma nenhuma também fica no solver** (dono 09/11): o dinâmico cai com a massa de um
/// corpo sem forma (`MASS_WITHOUT_SHAPE_KG`, ou o `MassOverride`), o estático e o cinemático ficam
/// onde estão — o «No Shape Body» da cena 120 não se mexe.
#[test]
fn um_corpo_sem_forma_fica_no_solver_e_so_o_dinamico_cai() {
    for (kind, manual) in [
        (BodyKind::Dynamic, None),
        (BodyKind::Dynamic, Some(5.0)),
        (BodyKind::Static, None),
        (BodyKind::Kinematic, None),
    ] {
        let mut sim = SimWorld::new();
        let w = sim.world_mut();
        let nu = w
            .spawn((
                Name::new("No Shape Body"),
                RigidBody { kind },
                Transform::from_translation(Vec2::new(0.0, 3.0)),
            ))
            .id();
        if let Some(m) = manual {
            w.entity_mut(nu).insert(ph2d_physics_ecs::MassOverride(m));
        }
        let mut bridge = PhysicsBridge::new();
        corre(&mut sim, &mut bridge, 0, 60);
        assert!(
            bridge.body_pose(nu).is_some(),
            "{kind:?}: o corpo sem forma ficou fora do solver"
        );
        if kind == BodyKind::Dynamic {
            let massa = bridge.body_mass(nu).expect("o corpo está no solver");
            let esperada = manual.unwrap_or(ph2d_physics::MASS_WITHOUT_SHAPE_KG);
            assert!(
                (massa - esperada).abs() < 1e-5,
                "massa {massa} contra {esperada}"
            );
            assert!(y(&sim, nu) < 0.0, "um segundo de queda: {}", y(&sim, nu));
        } else {
            let parado = y(&sim, nu);
            assert!(
                (parado - 3.0).abs() < 1e-6,
                "{kind:?}: o corpo sem forma mexeu-se para {parado}"
            );
        }
    }
}

/// **Reabrir o olho a meio do play devolve a massa da FORMA** (o ramo `was_off` do `retune_body`): o
/// corpo de uma forma só pesa a forma; de olho fechado pesa `MASS_WITHOUT_SHAPE_KG`; reaberto, o
/// quilo emprestado sai e volta a pesar a forma — nunca forma + 1 kg.
#[test]
fn reabrir_o_olho_devolve_a_massa_da_forma() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let corpo = w
        .spawn((
            Name::new("Solo"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Transform::from_translation(Vec2::new(0.0, 3.0)),
        ))
        .id();
    let f = w
        .spawn((caixa(0.3), Transform::IDENTITY, ChildOf(corpo)))
        .id();
    let mut bridge = PhysicsBridge::new();
    let massa = |bridge: &PhysicsBridge| bridge.body_mass(corpo).expect("no solver");
    corre(&mut sim, &mut bridge, 0, 0);
    let aberta = massa(&bridge);
    olho(&mut sim, f);
    corre(&mut sim, &mut bridge, 1, 1);
    let fechada = massa(&bridge);
    olho(&mut sim, f);
    corre(&mut sim, &mut bridge, 2, 2);
    let reaberta = massa(&bridge);
    assert!(
        (fechada - ph2d_physics::MASS_WITHOUT_SHAPE_KG).abs() < 1e-5,
        "fechada pesa {fechada}"
    );
    assert!(
        (aberta - ph2d_physics::MASS_WITHOUT_SHAPE_KG).abs() > 0.1,
        "a fixtura não distingue a forma do quilo: {aberta}"
    );
    assert!(
        (reaberta - aberta).abs() < 1e-5,
        "reaberta pesa {reaberta}, a forma pesa {aberta}"
    );
}
