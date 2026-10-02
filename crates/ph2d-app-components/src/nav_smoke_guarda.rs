//! ⭐⭐⭐ **Smoke da NAVEGAÇÃO, cena `=3`: O GUARDA** (plano 30, W6) — patrulha, vê, persegue, e a
//! porta fecha-se.
//!
//! # A cena
//!
//! Duas salas e uma parede com uma PORTA. Na sala da esquerda, dois guardas fazem a RONDA por uma
//! forma desenhada com a caneta (os rectângulos cinzentos). O herói AMARELO começa na sala da direita
//! e anda com as setas.
//!
//! | quem | o quê | o que tem de acontecer |
//! |---|---|---|
//! | VERMELHO | patrulha + cérebro (a `StateMachine`) | pisar a zona AMARELA (do lado dele da porta) faz-se ver: ele PERSEGUE o herói e a porta ARMA-SE; fugindo para a zona VERDE (do lado de cá), a porta fecha-se atrás do herói e na cara dele: ele DESISTE e volta à ronda |
//! | CINZENTO (o CONTROLO) | a MESMA patrulha, sem cérebro | só faz a ronda, veja o que vir — a perseguição é do cérebro, não do agente |
//!
//! # A corrente, toda autorada (nenhuma linha de código decide nada aqui)
//!
//! ```text
//! zona amarela (SignalOnHit «viu_heroi») ─► cérebro do guarda: Patrol → Chase ─► «perseguir»
//!   «perseguir» ─► tabela do guarda: Start Navigation «Hero»
//!               └► cérebro da porta: Open → Armed
//! zona verde (SignalOnHit «salvo»), com a porta armada ─► Armed → Closed ─► «fechar»
//!   «fechar» ─► tabela da porta: Start Timer «desliza» (o tween que a desce)
//!   a porta (cinemática) desce e PÁRA ─► a malha recorta-a ─► o caminho do guarda fica parcial
//!   ─► On No Path «perdeu_heroi» ─► cérebro: Chase → Patrol ─► «patrulhar» ─► Start Navigation «»
//! ```
//!
//! ⚠️ A porta FECHADA é uma parede para a malha porque é um cinemático PARADO (a W6); enquanto
//! desce, não é (o contorno claro da área andável — tecla `B` — fecha o vão quando ela pára).
//!
//! ⛔ **Recusado, medido nas contas da cena:** um ALARME de tempo fixo (a porta fecha `2,5 s` depois
//! de o guarda ver). O guarda (`2,2 m/s`) chega à porta, do ponto da ronda mais perto, em `~1,1 s` —
//! antes de ela fechar, e a cena ensinava o contrário (o guarda passava). A porta armada que fecha
//! quando o herói chega ao outro lado fecha SEMPRE entre os dois: o herói (`4 m/s`) vai à frente.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Entity, MachineState, Name, SignalAction, SignalActions, SignalVerb, SimWorld, StateMachine,
    StateTransition, Timer, Timers, Transform, Tweens, stable_name_id,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavRegion, NavTarget, RigidBody, SignalOnHit,
    TopDownPlayer,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};
use ph2d_tween::{AoAcabar, Canal, Tween};
use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, VertexKind};

use crate::nav_smoke::{CENTRO, MEIO_RECINTO, RAIO_HEROI};

/// Os NOMES da corrente — lidos nos dois lados pela mesma const.
pub const RONDA: &str = "Patrol Route";
/// A ronda do CONTROLO.
pub const RONDA_CONTROLO: &str = "Patrol Route 2";
const VIU: &str = "viu_heroi";
const PERDEU: &str = "perdeu_heroi";
const PERSEGUIR: &str = "perseguir";
const PATRULHAR: &str = "patrulhar";
const FECHAR: &str = "fechar";
const SALVO: &str = "salvo";

/// A parede que divide as salas, e o VÃO da porta nela (`y ∈ [Y_VAO.0, Y_VAO.1]`).
pub const X_DIVISAO: f32 = 1.0;
/// O vão: `1,2 m` — um guarda (`2r = 0,7 m`) passa.
pub const Y_VAO: (f32, f32) = (0.3, 1.5);
/// A porta: meia-largura e meia-altura (cobre o vão com folga quando fechada).
pub const MEIA_PORTA: [f32; 2] = [0.2, 0.7];
/// O centro da porta FECHADA e ABERTA (aberta, ela esconde-se dentro da parede de cima).
pub const Y_PORTA_FECHADA: f32 = 0.9;
pub const Y_PORTA_ABERTA: f32 = 2.5;
/// Quanto a porta demora a descer.
pub const DESLIZE_US: u64 = 250_000;
/// O raio dos guardas.
pub const RAIO_GUARDA: f32 = 0.35;
/// A velocidade dos guardas (o herói anda a `4 m/s`: fugir é possível).
pub const VELOCIDADE_GUARDA: f32 = 2.2;
/// Os cantos das duas rondas (anti-horário).
///
/// ⚠️ **A ronda passa a `3,4 m` do vão, e é a conta da cena:** o guarda (`2,2 m/s`) leva `≥ 1,5 s` a
/// chegar à porta, e o herói (`4 m/s`, a travar e a voltar) chega à zona verde em `~0,6 s` — mais os
/// `0,25 s` da porta a descer. Com a ronda a `2,2 m` o guarda entrava no vão antes de ela fechar
/// (medido no gate da cena).
pub const CANTOS_RONDA: [[f64; 2]; 4] = [[-5.0, 1.8], [-2.4, 1.8], [-2.4, 3.0], [-5.0, 3.0]];
pub const CANTOS_CONTROLO: [[f64; 2]; 4] = [[-5.0, -1.3], [-2.4, -1.3], [-2.4, -0.3], [-5.0, -0.3]];

const MEIA_PAREDE: f32 = 0.25;
const PAREDE_RGBA: [f32; 4] = [0.38, 0.40, 0.46, 1.0];
const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const PORTA_RGBA: [f32; 4] = [0.78, 0.52, 0.30, 1.0];
const ZONA_RGBA: [f32; 4] = [0.95, 0.72, 0.25, 0.18];
const SALVO_RGBA: [f32; 4] = [0.45, 0.85, 0.50, 0.18];
const HEROI_RGBA: [f32; 4] = [0.95, 0.72, 0.25, 1.0];
const VERMELHO_RGBA: [f32; 4] = [0.90, 0.30, 0.28, 1.0];
const CONTROLO_RGBA: [f32; 4] = [0.55, 0.57, 0.60, 1.0];

/// As peças da cena `=3`.
pub struct Guarda {
    pub guarda: Entity,
    pub controlo: Entity,
    pub heroi: Entity,
    pub porta: Entity,
}

fn parede(world: &mut ph2d_ecs::World, nome: &str, centro: Vec2, meio: Vec2) {
    world.spawn((
        Name::new(nome),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: meio.x,
                half_y: meio.y,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [meio.x * 2.0, meio.y * 2.0], PAREDE_RGBA),
        Transform::from_translation(centro),
    ));
}

/// Uma forma DESENHADA (um rectângulo fechado, só traço), com o nome dado — como a caneta a deixa.
fn desenha(
    sim: &mut SimWorld,
    cena: &mut ph2d_vec_scene::VecScene,
    mapa: &mut ph2d_vec_entities::entities::VecEntityMap,
    nome: &str,
    cantos: [[f64; 2]; 4],
) {
    let id = cena.push_path(VecPath {
        verts: cantos
            .iter()
            .map(|&p| VecVertex {
                anchor: p,
                in_handle: p,
                out_handle: p,
                kind: VertexKind::Corner,
                corner_radius: 0.0,
            })
            .collect(),
        closed: true,
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(90, 100, 120, 255), 0.06)),
        ..VecPath::default()
    });
    // ⚠️ A `sync` adopta a forma numa entidade AGORA: sem ela o nome só existiria no quadro seguinte
    // e o guarda abria a cena com a queixa da forma perdida (a lição do seguidor de caminho).
    ph2d_vec_entities::entities::sync(sim, cena, mapa);
    if let Some(&bits) = mapa.get(&id)
        && let Some(mut n) = sim.world_mut().get_mut::<Name>(Entity::from_bits(bits))
    {
        n.0 = nome.to_owned();
    }
}

/// Uma zona SENSORA junto à porta, que grita `sinal` a quem a pisa.
fn zona(world: &mut ph2d_ecs::World, nome: &str, x: f32, sinal: &str, cor: [f32; 4]) {
    world.spawn((
        Name::new(nome),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.35,
                half_y: 0.6,
            },
            is_sensor: true,
            ..Collider::default()
        },
        SignalOnHit(sinal.to_owned()),
        Sprite::atlas(WHITE_TILE_KEY, [0.7, 1.2], cor),
        Transform::from_translation(Vec2::new(x, Y_PORTA_FECHADA)),
    ));
}

fn estado(nome: &str, ao_entrar: &str) -> MachineState {
    MachineState {
        name: nome.into(),
        on_enter: ao_entrar.into(),
        on_exit: String::new(),
    }
}

fn passo(from: u8, on: &str, to: u8) -> StateTransition {
    StateTransition {
        from,
        on: on.into(),
        to,
    }
}

fn linha(on: &str, verb: SignalVerb, arg: &str) -> SignalAction {
    SignalAction {
        on: on.to_owned(),
        target: String::new(),
        verb,
        arg: arg.to_owned(),
        ..SignalAction::default()
    }
}

/// Um guarda em patrulha pela forma `ronda`.
fn guarda(
    world: &mut ph2d_ecs::World,
    nome: &str,
    em: [f64; 2],
    ronda: &str,
    cor: [f32; 4],
) -> Entity {
    world
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball {
                    radius: RAIO_GUARDA,
                },
                ..Collider::default()
            },
            Sprite::atlas(WHITE_TILE_KEY, [RAIO_GUARDA * 2.0, RAIO_GUARDA * 2.0], cor),
            TopDownPlayer::from_law(TopDownLaw {
                speed: VELOCIDADE_GUARDA,
                direction: DirectionMode::Free,
                default_controls: false,
                ..TopDownLaw::default()
            }),
            NavAgent {
                target: NavTarget::Patrol(stable_name_id(ronda)),
                // ⚠️ **Encostar é chegar** (os corpos colidem) — e a mesma régua serve à ronda.
                arrive_distance: RAIO_GUARDA + RAIO_HEROI + 0.1,
                ..NavAgent::default()
            },
            #[allow(clippy::cast_possible_truncation)]
            Transform::from_translation(Vec2::new(em[0] as f32, em[1] as f32)),
        ))
        .id()
}

/// ⭐ **Monta a cena `=3`** nos três empréstimos que ela usa (a lei do `path_follow_smoke::monta_em`:
/// um gate não constrói um `SceneCtx`).
pub fn monta_em(
    sim: &mut SimWorld,
    cena: &mut ph2d_vec_scene::VecScene,
    mapa: &mut ph2d_vec_entities::entities::VecEntityMap,
) -> Guarda {
    let [mx, my] = MEIO_RECINTO;
    let [cx, cy] = CENTRO;
    let (topo, chao) = (cy + my, cy - my);
    let e = MEIA_PAREDE;
    // ⚠️ O CHÃO PRIMEIRO — a ordem de criação é a de desenho.
    let w = sim.world_mut();
    w.spawn((
        Name::new("Floor"),
        Sprite::atlas(WHITE_TILE_KEY, [mx * 2.0, my * 2.0], CHAO_RGBA),
        Transform::from_translation(Vec2::new(cx, cy)),
    ));
    for (nome, c, m) in [
        (
            "Wall N",
            Vec2::new(cx, topo + e),
            Vec2::new(mx + 2.0 * e, e),
        ),
        (
            "Wall S",
            Vec2::new(cx, chao - e),
            Vec2::new(mx + 2.0 * e, e),
        ),
        ("Wall W", Vec2::new(cx - mx - e, cy), Vec2::new(e, my)),
        ("Wall E", Vec2::new(cx + mx + e, cy), Vec2::new(e, my)),
        (
            "Wall Low",
            Vec2::new(X_DIVISAO, (chao + Y_VAO.0) * 0.5),
            Vec2::new(e, (Y_VAO.0 - chao) * 0.5),
        ),
        (
            "Wall High",
            Vec2::new(X_DIVISAO, (Y_VAO.1 + topo) * 0.5),
            Vec2::new(e, (topo - Y_VAO.1) * 0.5),
        ),
    ] {
        parede(w, nome, c, m);
    }
    w.spawn((
        Name::new("Nav Region"),
        NavRegion {
            half_extents: MEIO_RECINTO,
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(Vec2::new(cx, cy)),
    ));
    // ⭐ A ZONA DA PORTA, do lado do guarda: quem a pisa é visto (só o herói lá chega — a ronda
    // passa longe dela).
    zona(w, "Guard Sees You Here", X_DIVISAO - 0.75, VIU, ZONA_RGBA);
    // ⭐ A ZONA DO OUTRO LADO: com a porta armada, chegar aqui fecha-a.
    // ⚠️ LIVRE da porta: o herói só lhe toca com o corpo inteiro fora do vão (a porta não lhe desce
    // em cima).
    zona(w, "Safe Side", X_DIVISAO + 1.35, SALVO, SALVO_RGBA);
    // ⭐ AS RONDAS, por cima do chão e das paredes. ⚠️ A `sync` dá à forma o 1.º lugar LIVRE da
    // pilha de desenho — sem os números do chão e das paredes congelados ANTES, as formas ficavam
    // por baixo do chão (medido na 1.ª foto desta cena: as rondas não se viam).
    ph2d_ecs::assign_missing_root_order(w);
    desenha(sim, cena, mapa, RONDA, CANTOS_RONDA);
    desenha(sim, cena, mapa, RONDA_CONTROLO, CANTOS_CONTROLO);
    let w = sim.world_mut();
    // ⭐⭐ A PORTA: um cinemático que um tween desce quando o relógio «desliza» corre, e um cérebro
    // de três estados que só a deixa fechar DEPOIS de o guarda ver o herói.
    let porta = w
        .spawn((
            Name::new("Door"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: MEIA_PORTA[0],
                    half_y: MEIA_PORTA[1],
                },
                ..Collider::default()
            },
            Sprite::atlas(
                WHITE_TILE_KEY,
                [MEIA_PORTA[0] * 2.0, MEIA_PORTA[1] * 2.0],
                PORTA_RGBA,
            ),
            // ⚠️ O tween `i` corre no relógio `i`.
            Timers(vec![Timer {
                name: "desliza".into(),
                duration_us: DESLIZE_US,
                // ⚠️ O de fábrica ARRANCA sozinho — e a porta abria a cena fechada.
                autostart: false,
                ..Timer::default()
            }]),
            Tweens(vec![Tween {
                ao_acabar: AoAcabar::Hold,
                ..Tween::linear(Canal::PositionY, Y_PORTA_ABERTA, Y_PORTA_FECHADA)
            }]),
            StateMachine {
                states: vec![
                    estado("Open", ""),
                    estado("Armed", ""),
                    estado("Closed", FECHAR),
                ],
                transitions: vec![passo(0, PERSEGUIR, 1), passo(1, SALVO, 2)],
                initial: 0,
            },
            SignalActions(vec![linha(FECHAR, SignalVerb::StartTimer, "desliza")]),
            Transform::from_translation(Vec2::new(X_DIVISAO, Y_PORTA_ABERTA)),
        ))
        .id();

    let heroi = w
        .spawn((
            Name::new("Hero"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: RAIO_HEROI },
                ..Collider::default()
            },
            Sprite::atlas(
                WHITE_TILE_KEY,
                [RAIO_HEROI * 2.0, RAIO_HEROI * 2.0],
                HEROI_RGBA,
            ),
            TopDownPlayer::from_law(TopDownLaw {
                speed: 4.0,
                direction: DirectionMode::EightWay,
                ..TopDownLaw::default()
            }),
            Transform::from_translation(Vec2::new(4.0, Y_PORTA_FECHADA)),
        ))
        .id();

    let vermelho = guarda(w, "Guard", CANTOS_RONDA[0], RONDA, VERMELHO_RGBA);
    // ⭐⭐ O CÉREBRO: ronda ↔ perseguição, e a tabela que o liga à navegação.
    w.entity_mut(vermelho).insert((
        StateMachine {
            states: vec![estado("Patrol", PATRULHAR), estado("Chase", PERSEGUIR)],
            transitions: vec![passo(0, VIU, 1), passo(1, PERDEU, 0)],
            initial: 0,
        },
        SignalActions(vec![
            linha(PERSEGUIR, SignalVerb::StartNavigation, "Hero"),
            linha(PATRULHAR, SignalVerb::StartNavigation, ""),
        ]),
    ));
    if let Some(mut a) = w.get_mut::<NavAgent>(vermelho) {
        // A porta fechada deixa o herói noutra ilha: é aí que ele desiste.
        a.on_no_path = PERDEU.to_owned();
    }
    // ⭐⭐ O CONTROLO: a mesma ronda, nenhum cérebro.
    let controlo = guarda(
        w,
        "Control (no brain)",
        CANTOS_CONTROLO[0],
        RONDA_CONTROLO,
        CONTROLO_RGBA,
    );
    Guarda {
        guarda: vermelho,
        controlo,
        heroi,
        porta,
    }
}

/// O roteiro, impresso.
pub fn anuncia() {
    println!(
        "[nav-smoke] =3 O GUARDA. Os dois quadrados da esquerda fazem a RONDA pelas formas \
         desenhadas (os rectangulos). Ande com as setas: entre pela PORTA na sala deles e pise a \
         zona AMARELA - o VERMELHO ve-o e persegue-o. Volte logo para tras ate' a zona VERDE: a \
         porta fecha-se atras de si e na cara dele, e ele desiste e volta a ronda. O CINZENTO (o \
         controlo) nao tem cerebro: so' faz a ronda. Tecla B: o contorno claro fecha o vao quando \
         a porta para"
    );
}

#[cfg(test)]
#[path = "nav_smoke_guarda_tests.rs"]
mod tests;
