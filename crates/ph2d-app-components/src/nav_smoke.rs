//! ⭐⭐⭐ **Smoke da NAVEGAÇÃO** (plano 30, W3). `PH2D_NAV_SMOKE=1` (e `=2`, o DESVIO da W5:
//! [`crate::nav_smoke_porta`]; e `=3`, O GUARDA da W6: [`crate::nav_smoke_guarda`]; e `=4`, A LAVA E
//! O PORTAL da W7: [`crate::nav_smoke_lava`]; e `=5`, A LAMA da W18: [`crate::nav_smoke_lama`]; e `=6`, a lama nos JOGOS: [`crate::nav_smoke_usos`]).
//!
//! # A cena: **o labirinto em S, e três perseguidores**
//!
//! O herói AMARELO anda com as **setas**. Três coisas querem apanhá-lo, e cada uma ensina uma metade:
//!
//! | quem | o quê | o que tem de acontecer |
//! |---|---|---|
//! | VERMELHO | agente de navegação, corpo pequeno | **dá a volta às paredes** pelo caminho mais curto e apanha o herói onde quer que ele vá |
//! | ROXO | o MESMO agente, corpo GRANDE | a porta de baixo (`1 m`) é mais estreita que ele: **fica do lado de cá da parede, o mais perto do herói que consegue**, e diz *«too big for the door»* |
//! | CINZENTO (o CONTROLO) | uma bala que persegue em linha recta | **bate nas paredes** e fica presa — é o que seria um inimigo sem navegação |
//!
//! ⭐ **A linha azul-aço** que sai de cada agente é o CAMINHO que ele planeou (o overlay de física,
//! tecla `B`): ela dobra nos cantos das paredes, e muda quando o herói foge.
//!
//! ⚠️ **O roxo e o vermelho são o MESMO componente** — só o raio do corpo muda. É isso que torna a
//! porta legível: a malha andável é recuada pelo raio de quem anda nela.
//!
//! ⚠️ **O cinzento é o controlo**: sem ele, o artista não tem como saber se o vermelho dá a volta
//! porque acha o caminho ou porque a cena está desenhada para isso.
//!
//! ⚠️ Se a linha `[nav-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavRegion, NavTarget, ProjectileMotion, RigidBody,
    TopDownPlayer,
};
use ph2d_projectile::ProjectileLaw;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

/// ⭐⭐ **Quantas cenas este roteador serve** — contado do `match` do [`montar`].
pub const CENAS: u32 = 6;

const PAREDE_RGBA: [f32; 4] = [0.38, 0.40, 0.46, 1.0];
const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const HEROI_RGBA: [f32; 4] = [0.95, 0.72, 0.25, 1.0];
const VERMELHO_RGBA: [f32; 4] = [0.90, 0.30, 0.28, 1.0];
const ROXO_RGBA: [f32; 4] = [0.62, 0.38, 0.85, 1.0];
const CONTROLO_RGBA: [f32; 4] = [0.55, 0.57, 0.60, 1.0];

/// O CENTRO do recinto. ⚠️ **Não é a origem, e a razão é a FOTO:** com a timeline aberta a banda
/// que sobra do ecrã enquadra `y ∈ [−2,6 ; 4,1]` a `100 px/m` (medido na 1.ª foto desta cena, que
/// escondia a porta debaixo do painel) — o recinto sobe para caber nela.
pub const CENTRO: [f32; 2] = [0.0, 0.8];
/// O interior do recinto (meio): `x ∈ [−5,8 ; 5,8]`, `y ∈ [−2,0 ; 3,6]`.
pub const MEIO_RECINTO: [f32; 2] = [5.8, 2.8];
/// Espessura das paredes do labirinto (meia).
///
/// ⚠️ (W8) **A cena `=1` estava APERTADA** (smoke do dono, 01/10: *«não sei se intencionalmente»*):
/// paredes de `0,5 m` e um roxo de `1,3 m` num recinto que já enche o ecrã. A cura escala os CORPOS,
/// a porta e as paredes — a lição (o pequeno passa, o grande não) é a mesma, com o dobro do ar.
const MEIA_PAREDE: f32 = 0.12;
/// A parede de baixo-a-meio deixa uma PORTA junto ao chão com esta largura.
pub const PORTA: f32 = 0.7;
/// O raio do corpo do VERMELHO do labirinto — passa a porta com folga (`2·r < PORTA`).
pub const LAB_RAIO_PEQUENO: f32 = 0.25;
/// O raio do corpo do ROXO — não passa a porta (`2·r > PORTA`).
pub const RAIO_GRANDE: f32 = 0.45;
/// O raio do herói do labirinto.
pub const LAB_RAIO_HEROI: f32 = 0.25;
/// O raio de um perseguidor das cenas `=3`/`=4` (a lava e o guarda) — o de antes da W8.
pub const RAIO_PEQUENO: f32 = 0.35;
/// O raio do herói das cenas `=3`/`=4`.
pub const RAIO_HEROI: f32 = 0.35;
/// Onde a parede da porta está.
pub const X_PORTA: f32 = 2.0;
/// O `y` do chão do recinto (a porta abre junto dele).
pub const Y_CHAO: f32 = CENTRO[1] - MEIO_RECINTO[1];

const _: () = assert!(2.0 * LAB_RAIO_PEQUENO < PORTA && 2.0 * RAIO_GRANDE > PORTA);

/// O que o roteador montou — o nível, quem fica ESCOLHIDO (o Inspector mostra-o) e as peças da cena.
pub struct Montada {
    pub nivel: u32,
    pub escolhido: Entity,
    pub labirinto: Option<Labirinto>,
    pub porta: Option<crate::nav_smoke_porta::Porta>,
    /// As peças da cena `=3` (W6).
    pub guarda: Option<crate::nav_smoke_guarda::Guarda>,
    /// As peças da cena `=4` (W7).
    pub lava: Option<crate::nav_smoke_lava::Lava>,
    /// As peças da cena `=5` (W18).
    pub lama: Option<crate::nav_smoke_lama::Lama>,
    /// As peças da cena `=6` (W18, a lama nos jogos).
    pub usos: Option<crate::nav_smoke_usos::Usos>,
}

impl Montada {
    /// ⭐ A secção do Inspector que o roteiro manda ler — a shell abre-a (a política fecha toda secção
    /// viva menos o Transform). Na `=5` e na `=6` o escolhido é uma área, e o passo é o `Cost` dela.
    #[must_use]
    pub fn secao_do_roteiro(&self) -> ph2d_editor_core::ids::NodeId {
        if self.lama.is_some() || self.usos.is_some() {
            ph2d_editor_core::ids::INSP_LIVE_NAV_COST_AREA_SECTION
        } else {
            ph2d_editor_core::ids::INSP_LIVE_NAV_AGENT_SECTION
        }
    }
}

/// As peças da cena `=1`.
pub struct Labirinto {
    pub vermelho: Entity,
    pub roxo: Entity,
    pub controlo: Entity,
    pub heroi: Entity,
}

pub(crate) fn parede(world: &mut World, nome: &str, centro: Vec2, meio: Vec2) {
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

/// Uma parede vertical em `x`, de `y0` a `y1`.
fn parede_v(world: &mut World, nome: &str, x: f32, y0: f32, y1: f32) {
    parede(
        world,
        nome,
        Vec2::new(x, (y0 + y1) * 0.5),
        Vec2::new(MEIA_PAREDE, (y1 - y0) * 0.5),
    );
}

/// Um perseguidor: o mover de vista de cima com os controlos DESLIGADOS (quem escreve a intenção é
/// a navegação) e o agente. `(raio, raio_do_alvo)`: o corpo dele e o de quem persegue (a chegada é
/// encostar, com folga).
pub(crate) fn perseguidor(
    world: &mut World,
    nome: &str,
    em: Vec2,
    (raio, raio_do_alvo): (f32, f32),
    velocidade: f32,
    cor: [f32; 4],
    sinais: (&str, &str),
) -> Entity {
    world
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: raio },
                ..Collider::default()
            },
            crate::smoke_desenho::disco(raio, cor),
            TopDownPlayer::from_law(TopDownLaw {
                speed: velocidade,
                direction: DirectionMode::Free,
                default_controls: false,
                ..TopDownLaw::default()
            }),
            NavAgent {
                target: NavTarget::Named(stable_name_id("Hero")),
                // ⚠️ **Os corpos COLIDEM**: o centro do perseguidor nunca chega a menos de
                // `r + r_herói` do centro do herói, logo «chegar» é encostar — com uma folga.
                arrive_distance: raio + raio_do_alvo + 0.15,
                on_arrived: sinais.0.to_owned(),
                on_no_path: sinais.1.to_owned(),
                ..NavAgent::default()
            },
            Transform::from_translation(em),
        ))
        .id()
}

/// A cena `=1`.
fn cena_um(world: &mut World) -> Montada {
    let [mx, my] = MEIO_RECINTO;
    let [cx, cy] = CENTRO;
    let c = Vec2::new(cx, cy);
    world.spawn((
        Name::new("Floor"),
        Sprite::atlas(WHITE_TILE_KEY, [mx * 2.0, my * 2.0], CHAO_RGBA),
        Transform::from_translation(c),
    ));
    // O recinto.
    let e = MEIA_PAREDE;
    let topo = cy + my;
    parede(
        world,
        "Wall N",
        Vec2::new(cx, topo + e),
        Vec2::new(mx + 2.0 * e, e),
    );
    parede(
        world,
        "Wall S",
        Vec2::new(cx, Y_CHAO - e),
        Vec2::new(mx + 2.0 * e, e),
    );
    parede(
        world,
        "Wall W",
        Vec2::new(cx - mx - e, cy),
        Vec2::new(e, my),
    );
    parede(
        world,
        "Wall E",
        Vec2::new(cx + mx + e, cy),
        Vec2::new(e, my),
    );
    // ⭐ O S: a 1.ª parede deixa a passagem LARGA em cima (`2 m`), a 2.ª deixa a PORTA estreita em
    // baixo.
    parede_v(world, "Wall Left", -X_PORTA, Y_CHAO, topo - 2.0);
    parede_v(world, "Wall Door", X_PORTA, Y_CHAO + PORTA, topo);
    // A região andável: o interior inteiro.
    world.spawn((
        Name::new("Nav Region"),
        NavRegion {
            half_extents: MEIO_RECINTO,
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(c),
    ));

    let heroi = world
        .spawn((
            Name::new("Hero"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball {
                    radius: LAB_RAIO_HEROI,
                },
                ..Collider::default()
            },
            crate::smoke_desenho::disco(LAB_RAIO_HEROI, HEROI_RGBA),
            TopDownPlayer::from_law(TopDownLaw {
                speed: 4.0,
                direction: DirectionMode::EightWay,
                ..TopDownLaw::default()
            }),
            Transform::from_translation(Vec2::new(4.2, 2.2)),
        ))
        .id();

    let vermelho = perseguidor(
        world,
        "Chaser",
        Vec2::new(-4.5, -1.2),
        (LAB_RAIO_PEQUENO, LAB_RAIO_HEROI),
        2.5,
        VERMELHO_RGBA,
        ("caught you", ""),
    );
    let roxo = perseguidor(
        world,
        "Big Chaser",
        Vec2::new(-4.5, 1.0),
        (RAIO_GRANDE, LAB_RAIO_HEROI),
        2.0,
        ROXO_RGBA,
        ("", "too big for the door"),
    );
    // ⭐⭐ **O CONTROLO**: uma bala que persegue o herói em linha recta, SEM navegação. Ela bate nas
    // paredes e ricocheteia — é o inimigo que a casa sabia fazer antes desta wave.
    let controlo = world
        .spawn((
            Name::new("Control (homing)"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball {
                    radius: LAB_RAIO_PEQUENO,
                },
                ..Collider::default()
            },
            crate::smoke_desenho::disco(LAB_RAIO_PEQUENO, CONTROLO_RGBA),
            ProjectileMotion::from_law(
                ProjectileLaw {
                    initial_speed: 2.5,
                    max_speed: 2.5,
                    gravity: 0.0,
                    homing_accel: 6.0,
                    max_bounces: u8::MAX,
                    bounciness: 0.4,
                    range: 0.0,
                    ..ProjectileLaw::default()
                },
                stable_name_id("Hero"),
            ),
            Transform::from_translation(Vec2::new(-4.5, 3.0)),
        ))
        .id();
    Montada {
        nivel: 1,
        // ⭐ O ROXO escolhido (W4): a leitura viva dele — *«can't reach it»* — é o que explica a
        // espera atrás da parede, e a secção é o passo do roteiro.
        escolhido: roxo,
        labirinto: Some(Labirinto {
            vermelho,
            roxo,
            controlo,
            heroi,
        }),
        porta: None,
        guarda: None,
        lava: None,
        lama: None,
        usos: None,
    }
}

/// **Monta a cena `nivel`** — o roteador.
pub fn montar(world: &mut World, nivel: u32) -> Montada {
    if nivel == 6 {
        let u = crate::nav_smoke_usos::montar(world);
        crate::nav_smoke_usos::anuncia();
        return Montada {
            nivel: 6,
            // ⭐ O TERRENO escolhido: o `Cost` dele a `1` e o corredor 1 corta a direito.
            escolhido: u.terreno,
            labirinto: None,
            porta: None,
            guarda: None,
            lava: None,
            lama: None,
            usos: Some(u),
        };
    }
    if nivel == 5 {
        let l = crate::nav_smoke_lama::montar(world);
        crate::nav_smoke_lama::anuncia();
        return Montada {
            nivel: 5,
            // ⭐ A lama PESADA escolhida: o `Cost` dela no Inspector é o passo do roteiro.
            escolhido: l.pesada,
            labirinto: None,
            porta: None,
            guarda: None,
            lava: None,
            lama: Some(l),
            usos: None,
        };
    }
    if nivel == 4 {
        let l = crate::nav_smoke_lava::montar(world);
        crate::nav_smoke_lava::anuncia();
        return Montada {
            nivel: 4,
            // ⭐ O VERMELHO escolhido: a caixa Avoid Harm dele é o passo do roteiro.
            escolhido: l.vermelho,
            labirinto: None,
            porta: None,
            guarda: None,
            lava: Some(l),
            lama: None,
            usos: None,
        };
    }
    if nivel == 2 {
        let porta = crate::nav_smoke_porta::montar(world);
        println!(
            "[nav-smoke] =2 a PORTA de dois sentidos. Em cima, os VERMELHOS (com Avoid Others): quatro \
             vao para a direita e quatro para a esquerda pela MESMA porta, e cruzam-se nela - cada um \
             passa pela sua direita. Em baixo, os CINZENTOS (o controlo): os mesmos, SEM o desvio - \
             entalam-se na porta de frente uns para os outros. O Red 1 esta' escolhido: tire-lhe o \
             visto de Avoid Others no Inspector e veja os outros desviarem-se dele"
        );
        return Montada {
            nivel: 2,
            escolhido: porta.vermelhos[0],
            labirinto: None,
            porta: Some(porta),
            guarda: None,
            lava: None,
            lama: None,
            usos: None,
        };
    }
    let m = cena_um(world);
    println!(
        "[nav-smoke] =1 setas para andar com o AMARELO. O VERMELHO da' a volta as paredes para te \
         apanhar; o ROXO e' grande demais para a porta de baixo e fica do lado de ca' da parede; o CINZENTO (o \
         controlo) persegue em linha recta e bate nas paredes. A linha azul de cada um e' o caminho \
         que ele planeou (tecla B), e o contorno CLARO e' por onde o centro de cada um pode andar: ha' \
         um por tamanho, e o do ROXO fecha a porta de baixo. O ROXO esta' escolhido: a seccao Nav Agent \
         do Inspector diz o que ele esta' a fazer agora e porque espera"
    );
    m
}

/// ⭐ **Monta a cena `nivel` com o CONTEXTO inteiro** — a porta da shell. A `=3` desenha as formas da
/// ronda e precisa da cena vectorial; as outras só do mundo.
pub fn montar_cena(cx: &mut crate::scene_ctx::SceneCtx, nivel: u32) -> Montada {
    if nivel == 3 {
        return monta_tres(cx.sim, cx.vec_scene, cx.vec_entities);
    }
    montar(cx.sim.world_mut(), nivel)
}

/// A cena `=3` nos três empréstimos que ela usa (a porta dos gates).
pub fn monta_tres(
    sim: &mut ph2d_ecs::SimWorld,
    cena: &mut ph2d_vec_scene::VecScene,
    mapa: &mut ph2d_vec_entities::entities::VecEntityMap,
) -> Montada {
    let g = crate::nav_smoke_guarda::monta_em(sim, cena, mapa);
    crate::nav_smoke_guarda::anuncia();
    Montada {
        nivel: 3,
        // ⭐ O GUARDA escolhido: a secção Nav Agent diz «Patrol» e a forma; a leitura viva muda
        // quando ele persegue.
        escolhido: g.guarda,
        labirinto: None,
        porta: None,
        guarda: Some(g),
        lava: None,
        lama: None,
        usos: None,
    }
}

#[cfg(test)]
#[path = "nav_smoke_tests.rs"]
mod tests;
