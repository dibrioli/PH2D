//! **O mover de VISTA DE CIMA contra a `rapier` de verdade** (TOP-20 #13).
//!
//! O gate do corpus (`ph2d-topdown/tests/it/o_corpus_do_oraculo.rs`) mede a LEI
//! contra uma parede de mentira; estes medem o **produto**, com o controlador da
//! biblioteca no meio. ⚠️ *Uma sonda que mede um sucedâneo para sempre mede outro
//! programa* — foi a lição que a wave da fábrica pagou nesta mesma linha.
//!
//! A barra de cada gate está ao lado dela, e a grandeza é sempre a mesma:
//! **quanto o corpo andou num tique, em fracções do orçamento `|v|·dt`**.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, RigidBody, TopDownPlayer,
};
use ph2d_platformer::PlayerInput;

const VELOCIDADE: f32 = 4.0;
const DT: f32 = 1.0 / 60.0;
/// O orçamento de um tique: `|v| · dt`.
const ORCAMENTO: f32 = VELOCIDADE * DT;

fn parede(sim: &mut SimWorld, nome: &str, em: Vec2, meio: (f32, f32)) {
    sim.world_mut().spawn((
        Name::new(nome),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: meio.0,
                half_y: meio.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(em),
    ));
}

/// Um corpo de vista de cima em `em`, com a config que o teste pedir.
fn cena(em: Vec2, cfg: TopDownPlayer) -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    // ⚠️ **Uma parede VERTICAL com a face em `x = −1`**, e nada mais: sem chão,
    // porque numa vista de cima não há chão — é essa a metade do desenho que o
    // `max_slope_deg = 0` da ponte escreve.
    parede(&mut sim, "Parede", Vec2::new(0.0, 0.0), (1.0, 20.0));
    let quem = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                ..Collider::default()
            },
            LockRotation,
            cfg,
            Transform::from_translation(em),
        ))
        .id();
    (sim, PhysicsBridge::new(), quem)
}

fn pos(sim: &SimWorld, nome: &str) -> Vec2 {
    let mut achado = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == nome {
            achado = Some(t.translation);
        }
    }
    achado.expect("a cena tem de conter o corpo")
}

/// Empurra `dir` por `tiques` e devolve **o deslocamento do ÚLTIMO tique**.
fn passo_final(cfg: TopDownPlayer, em: Vec2, dir: [f32; 2], tiques: u64) -> Vec2 {
    let (mut sim, mut bridge, quem) = cena(em, cfg);
    bridge.set_player_input(
        quem,
        PlayerInput {
            drive: dir[0],
            drive_y: dir[1],
            ..PlayerInput::default()
        },
    );
    let mut antes = pos(&sim, "Player");
    for t in 1..=tiques {
        antes = pos(&sim, "Player");
        bridge.dispatch(&mut sim, true, t);
    }
    let depois = pos(&sim, "Player");
    Vec2::new(depois.x - antes.x, depois.y - antes.y)
}

fn comprimento(v: Vec2) -> f32 {
    (v.x * v.x + v.y * v.y).sqrt()
}

/// Uma fita que devolve sempre a mesma entrada.
///
/// ⚠️ O `HeldInput` da casa devolve `None` (*«fica como está»*), logo não serve
/// para provar que a fita ALCANÇA ou NÃO alcança uma entidade — era preciso uma
/// que de facto escrevesse.
struct FitaFixa(PlayerInput);

impl ph2d_physics_ecs::PlayerInputAtTick for FitaFixa {
    fn input(&mut self, _tick: u64) -> Option<PlayerInput> {
        Some(self.0)
    }
}

/// ⭐⭐⭐ **A lei da wave, medida no produto.**
///
/// Encostado a uma parede e empurrado a 45°, o corpo tem de andar o **orçamento
/// inteiro** ao longo dela. ⛔ A projecção (o que o `move_character` sozinho faz,
/// e o que o mover de plataforma quer) daria `sin 45° = 0,707` disso.
///
/// A barra é `0,9` do orçamento: o oráculo entrega `1,000` e a projecção entrega
/// `0,707`, então qualquer barra no meio separa as duas leis — `0,9` deixa folga
/// para a margem de des-penetração do controlador sem chegar perto do defeito.
#[test]
fn um_mover_de_vista_de_cima_desliza_a_velocidade_cheia() {
    let cfg = TopDownPlayer {
        speed: VELOCIDADE,
        ..TopDownPlayer::default()
    };
    // Nasce longe e chega à parede: `x = −3` com a face em `−1` e raio `0,2`.
    let d = passo_final(cfg, Vec2::new(-3.0, 0.0), [0.707_106_8, 0.707_106_8], 90);
    let fraccao = comprimento(d) / ORCAMENTO;
    assert!(
        fraccao > 0.9,
        "encostado e a 45° ele andou {fraccao:.4} do orcamento.\n\
         ⚠️ `0,707` e' a PROJECCAO — a lei do platformer a correr numa vista de cima, \
         que e' o corpo a rastejar na parede. O oraculo (Godot FLOATING) entrega `1,000`."
    );
    // E o deslocamento é tangente: a parede não deixa passar em `x`.
    assert!(
        d.x.abs() < ORCAMENTO * 0.1,
        "ele andou {:.5} em x contra a parede",
        d.x
    );
}

/// **O CONTROLO da lei acima: longe da parede, o orçamento é o mesmo.**
///
/// ⚠️ Sem ele, um gate que lesse `1,0` não distinguiria *«deslizou inteiro»* de
/// *«não tocou em nada»* — que é a família de fixturas que a sonda do oráculo
/// desta wave apanhou **quatro** vezes.
#[test]
fn e_o_controlo_e_o_mesmo_corpo_sem_parede_nenhuma() {
    let cfg = TopDownPlayer {
        speed: VELOCIDADE,
        ..TopDownPlayer::default()
    };
    // Longe, e a andar PARA LONGE da parede.
    let d = passo_final(cfg, Vec2::new(-8.0, 0.0), [-0.707_106_8, 0.707_106_8], 10);
    let fraccao = comprimento(d) / ORCAMENTO;
    assert!(
        (fraccao - 1.0).abs() < 0.05,
        "em campo aberto ele tem de andar o orcamento exacto e andou {fraccao:.4}"
    );
    assert!(d.x < 0.0, "e para o lado que lhe pediram: {:?}", d);
}

/// **De cabeça contra a parede ele PÁRA** — a cláusula 2 da lei, no produto.
#[test]
fn de_cabeca_contra_a_parede_ele_para() {
    let cfg = TopDownPlayer {
        speed: VELOCIDADE,
        ..TopDownPlayer::default()
    };
    let d = passo_final(cfg, Vec2::new(-3.0, 0.0), [1.0, 0.0], 90);
    assert!(
        comprimento(d) / ORCAMENTO < 0.05,
        "de cabeca ele andou {:.4} do orcamento",
        comprimento(d) / ORCAMENTO
    );
}

/// **Sem gravidade**: um mover de vista de cima parado fica onde está.
///
/// ⚠️ Não é óbvio: o corpo é `Kinematic` e a cena não tem chão. Se alguma coisa
/// na ponte lhe aplicasse gravidade, ele cairia para sempre — e o sintoma num
/// jogo seria *«o personagem foge para baixo»*.
#[test]
fn sem_intencao_ele_fica_exactamente_onde_esta() {
    let cfg = TopDownPlayer {
        speed: VELOCIDADE,
        ..TopDownPlayer::default()
    };
    let (mut sim, mut bridge, _quem) = cena(Vec2::new(-5.0, 0.0), cfg);
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let p = pos(&sim, "Player");
    assert!(
        (p.x + 5.0).abs() < 1.0e-3 && p.y.abs() < 1.0e-3,
        "parado, ele foi parar a {p:?}"
    );
}

/// **`default_controls` desligado ignora o teclado** — a lei transversal 3.
///
/// ⚠️ O canal continua a existir: quem chama `set_player_input` dirige-o. O que o
/// interruptor tira é a FITA, e é por isso que o teste escreve a entrada pela
/// fita (o `dispatch` com `HeldInput`) em vez de a escrever à mão.
#[test]
fn com_os_controlos_de_fabrica_desligados_a_fita_nao_o_alcanca() {
    let cfg = TopDownPlayer {
        speed: VELOCIDADE,
        default_controls: false,
        ..TopDownPlayer::default()
    };
    let (mut sim, mut bridge, quem) = cena(Vec2::new(-5.0, 0.0), cfg);
    // A FITA empurra para a direita — e não pode alcançá-lo.
    let mut fita = FitaFixa(PlayerInput {
        drive: 1.0,
        ..PlayerInput::default()
    });
    for t in 1..=60u64 {
        bridge.dispatch_with_tape(&mut sim, true, t, &mut fita);
    }
    let p = pos(&sim, "Player");
    assert!(
        (p.x + 5.0).abs() < 1.0e-3,
        "com os controlos desligados a fita moveu-o para {p:?}"
    );
    // ⭐ **E o CONTROLO**: o mesmo, com eles ligados, anda.
    let ligado = TopDownPlayer {
        speed: VELOCIDADE,
        ..TopDownPlayer::default()
    };
    let (mut sim, mut bridge, _) = cena(Vec2::new(-5.0, 0.0), ligado);
    for t in 1..=60u64 {
        bridge.dispatch_with_tape(&mut sim, true, t, &mut fita);
    }
    let p = pos(&sim, "Player");
    assert!(
        p.x > -5.0 + 0.5,
        "com eles ligados a MESMA fita tinha de o mover, e ele esta' em {p:?}"
    );
    let _ = quem;
}

/// ⛔⛔ **A DIVERGÊNCIA DECLARADA do oráculo, com o número dos dois lados.**
///
/// Abaixo do `min_slide_angle` o oráculo **pára seco** (`0,09 %` do orçamento) e
/// nós andamos **a projecção** (`sin θ`, medido `8,72 %` a 5° e `17,36 %` a 10°).
///
/// ⚠️ **É deliberado, e a alternativa foi construída e medida:** com o deslize da
/// biblioteca desligado o controlador deixa de reportar contacto de forma fiável
/// quando o corpo já toca — a mesma varredura devolveu **zeros errático**s a 25°,
/// 30°, 40° e 65°, que é um defeito pior do que a divergência que ele curava.
///
/// ⭐ E o que fica é defensável por si: o knob existe para que um toque quase
/// frontal **não** atire o corpo para o lado à velocidade cheia, e a projecção faz
/// isso de forma **contínua**, onde o alvo tem um penhasco de `235×` entre 15° e
/// 16°. *A nossa posição é mais suave; o gate defende-a e nomeia-a.*
#[test]
fn abaixo_do_limiar_nos_andamos_a_projeccao_e_o_oraculo_para() {
    let cfg = TopDownPlayer {
        speed: VELOCIDADE,
        // ⚠️ `Free`: com o encaixe de 8 direcções a 5° vira 0° e o teste mediria
        // o QUANTIZADOR — foi assim que a 1.ª tabela desta wave leu um limiar a
        // 22,5° que era a fronteira do encaixe.
        direction_mode: 0,
        ..TopDownPlayer::default()
    };
    for (graus, esperado) in [(5.0_f32, 0.0872_f32), (10.0, 0.1736)] {
        let a = graus.to_radians();
        let d = passo_final(cfg, Vec2::new(-3.0, 0.0), [a.cos(), a.sin()], 90);
        let fraccao = comprimento(d) / ORCAMENTO;
        assert!(
            (fraccao - esperado).abs() < 0.02,
            "a {graus}° nos andamos {fraccao:.4} e a projeccao e' {esperado:.4}.\n\
             ⚠️ `0,00` seria o oraculo (o penhasco); `1,00` seria nao haver limiar nenhum."
        );
    }
}

/// ⭐⭐ **A CAMADA que o cast consulta é a do CORPO, e não um literal.**
///
/// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE:** trocar o `layer` da ponte por um `9`
/// escrito à mão passou por todos os outros — porque a matriz de fábrica é permissiva e **toda
/// camada vê toda camada**. *Uma fixtura sem filtro de camada não pode medir o filtro de camada.*
///
/// A cena separa a camada do corpo (`1`) da da parede (`0`): o mover **tem de atravessar**. Com a
/// camada errada ele consultaria a máscara de outra e pararia.
#[test]
fn a_camada_do_cast_e_a_do_corpo() {
    use ph2d_physics_ecs::{LayerMatrix, PhysicsSettings};
    let mut sim = SimWorld::new();
    parede(&mut sim, "Parede", Vec2::new(0.0, 0.0), (1.0, 20.0));
    let cfg = TopDownPlayer {
        speed: VELOCIDADE,
        direction_mode: 0, // Free — ver a irmã: o encaixe mediria o quantizador
        ..TopDownPlayer::default()
    };
    let quem = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                // ⚠️ A camada do CORPO é `1`.
                layer: 1,
                ..Collider::default()
            },
            LockRotation,
            cfg,
            Transform::from_translation(Vec2::new(-3.0, 0.0)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    // `1` e `0` separadas ⇒ a parede não existe para este corpo.
    let mut m = LayerMatrix::all();
    m.set(1, 0, false);
    bridge.set_settings(PhysicsSettings {
        layer_matrix: m.rows(),
        ..PhysicsSettings::default()
    });
    bridge.set_player_input(
        quem,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let p = pos(&sim, "Player");
    assert!(
        p.x > 1.0,
        "com as camadas separadas ele tinha de atravessar a parede, e parou em {p:?}"
    );
}

/// ⭐⭐ **Dois movers na mesma entidade: o de PLATAFORMA ganha.**
///
/// ⚠️ **Este gate nasceu de outra mutação sobrevivente:** apagar o guarda do conflito passou,
/// porque o filtro que eu corri media o COMPONENTE e nenhuma fixtura punha os dois juntos.
///
/// O discriminador é a GRAVIDADE: o mover de vista de cima não a tem, o de plataforma cai. Com os
/// dois a escrever a pose no mesmo tique, o resultado deixa de ser o de nenhum dos dois.
#[test]
fn com_dois_movers_o_de_plataforma_ganha() {
    use ph2d_physics_ecs::{PlatformPlayer, PlayerMode};
    let mut sim = SimWorld::new();
    let quem = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer::default(),
            PlayerMode::Kinematic,
            TopDownPlayer {
                speed: VELOCIDADE,
                ..TopDownPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    bridge.set_player_input(
        quem,
        PlayerInput {
            // ⚠️ Nenhuma intenção horizontal: o que se mede é a QUEDA.
            drive: 0.0,
            drive_y: 0.0,
            ..PlayerInput::default()
        },
    );
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let p = pos(&sim, "Player");
    assert!(
        p.y < -0.5,
        "sem chão, o mover de PLATAFORMA cai — e ele parou em {p:?}, que é o de vista de cima a \
         escrever a pose por cima dele"
    );
}
