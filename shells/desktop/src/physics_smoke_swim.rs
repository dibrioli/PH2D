//! **NADAR** (`PH2D_PHYSICS_SMOKE=105`) — a `W-Swim`.
//!
//! A água já existia como *lugar onde se cai*; esta cena existe para o veredito
//! de que ela virou *lugar onde se anda, nos dois eixos*.
//!
//! # ⚠️ Três sujeitos, e as diferenças são UM knob e UM modo
//!
//! Os três corpos são idênticos em forma, densidade e lei:
//!
//! - **âmbar** — `swim_speed = 0`: a boia. É o que a água sempre fez, e é o
//!   CONTROLE;
//! - **azul** — o nadador, corpo DINÂMICO;
//! - **verde** — o mesmo nadador, corpo **CINEMÁTICO**.
//!
//! ⚠️ **O verde existe porque a pergunta foi feita** (Enio: *"Kinematic não vai
//! nadar?"*): vai, e os gates de produto já a respondiam nos dois modos
//! (`player_swims.rs` varre `[false, true]` em todos). O que faltava era o
//! artista poder **ver** — e uma capacidade que só os testes conhecem é uma
//! capacidade que o próximo report vai perguntar de novo.
//!
//! ⚠️ **Os três recebem a MESMA entrada** (há um teclado, logo um dedo — a lei do
//! `hand_input_to_players`), então a cena é uma ablação com a mão do artista: a
//! mesma tecla, três resultados, e as únicas variáveis são a capacidade e a
//! espécie do corpo.
//!
//! # ⚠️ A POÇA RASA é metade da cena, não cenário
//!
//! Atravessá-la a pé é o que o LIMIAR existe para proteger: um nado que armasse
//! ao molhar os pés interromperia a caminhada — e o arco de um salto — cada vez
//! que alguém pisasse numa poça. Medido (`measure_the_swim_threshold`), de pé no
//! chão sob esta poça o corpo lê `buoyed = 0,68`, abaixo do limiar de `1,0`.
//!
//! Os números da mensagem saem da sonda `physics_smoke_swim_tests`.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, BodyKind, Collider, ColliderShape, LockRotation, PlatformPlayer,
    PlayerMode, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// O cais sólido, topo em `y = 0`, à esquerda da poça funda.
const DOCK_HALF: [f32; 2] = [7.0, 0.5];
const DOCK_X: f32 = -9.0;

/// A poça RASA, em cima do cais: superfície em `y = 0,6`.
///
/// ⚠️ **A altura não é arbitrária** — de pé no cais o centro do corpo fica na
/// [`FLOAT`], `0,3 m` acima desta superfície, que a tabela do
/// `measure_the_swim_threshold` lê como **20% submerso, `buoyed = 0,68`**:
/// abaixo do limiar, logo caminhada.
const PUDDLE_HALF: [f32; 2] = [3.0, 0.3];
const PUDDLE_X: f32 = -9.0;

/// A poça FUNDA: superfície em `y = 0`, e um FUNDO alcançável em `y = -5`.
///
/// ⚠️ **A primeira versão desta cena não tinha fundo, e a medição a reprovou:**
/// segurar BAIXO por seis segundos levava o nadador a **−26,5 m** — ele saía
/// pela base do sensor e o resto era queda livre, fora de quadro. Uma piscina
/// tem fundo, e mergulhar tem de ter destino.
const POOL_HALF: [f32; 2] = [12.0, 2.5];
const POOL_X: f32 = 4.0;
/// O fundo da piscina — sólido, topo em `y = -5`.
const BED_Y: f32 = -5.0;

/// ⚠️ **O arrasto não é enfeite** — empuxo sem resistência é uma mola sem
/// amortecimento; a fixture irmã (cena 104) tem a medição.
const POOL_DRAG: f32 = 0.6;
/// Quatro vezes a densidade dos corpos — o mesmo par das fixtures irmãs.
const FLUID_DENSITY: f32 = 4.0;

const CAP_HALF_H: f32 = 0.3;
const CAP_RADIUS: f32 = 0.2;
const FLOAT: f32 = 0.9;

/// A velocidade de nado do azul. ⚠️ O âmbar fica em **zero**, que é como a
/// capacidade nasce — e é isso que faz do controle o produto de ontem.
const SWIM_SPEED: f32 = 4.0;
/// A autoridade do servo. ⚠️ Ela é **maior que o ponto de partida da lei** (12),
/// e o motivo é MEDIDO: nesta poça o empuxo líquido sobre um corpo submerso vale
/// `|g|·(4 − 1) ≈ 29,4 m/s²`, então com `12` o azul **não conseguiria
/// mergulhar** — só subiria mais devagar. Ver
/// `diving_needs_more_authority_than_the_water_has`.
const SWIM_ACCEL: f32 = 44.0;

/// **MEDIDO** (`physics_smoke_swim_tests`): a altura média do azul na segunda
/// metade de seis segundos, largado submerso, com o dedo em BAIXO · parado ·
/// CIMA — relativa ao ponto de largada.
///
/// ⚠️ **É uma média e não um instante**: o corpo na água OSCILA, e uma amostra
/// única de um sistema que oscila não é um repouso (a lição que a cena 104 já
/// carrega, e que o harness desta wave repetiu antes de ser corrigido).
///
/// ⚠️ **O do meio moveu de `+1,0409` para `+1,2284`, e é a wave da LINHA:** o
/// repouso do nado deixou de ser *velocidade zero* (congelar onde estava) e
/// passou a ser *procurar a linha de flutuação*. O número novo é a linha da
/// física desta poça — `y = 0,228`, **25% submerso** —, o MESMO ponto em que a
/// boia oscila.
pub(crate) const DIVE_IDLE_RISE: [f32; 3] = [-3.0873, 1.2284, 1.7589];

/// **MEDIDO**: a mesma tabela para a BOIA — `+1,50` nas TRÊS entradas.
///
/// ⚠️ **Ela é o controle da cena, e é o número que faz da diferença um KNOB em
/// vez de uma coincidência:** com a capacidade desligada os botões são mudos, e
/// os três valores coincidem até a quarta decimal.
///
/// ⚠️ **E ela é MAIOR que o repouso do nadador sem os dois discordarem:** a boia
/// SALTA fora da poça e volta (a média de uma parábola que passa mais tempo no
/// alto), enquanto o nadador ASSENTA na linha. Medido numa poça funda o
/// bastante para ninguém sair dela, os dois caem no mesmo `y` a menos de `0,01`
/// (`turning_the_swimming_on_does_not_move_the_float_line`).
pub(crate) const FLOATER_RISE: f32 = 1.5009;

const DOCK_RGBA: [f32; 4] = [0.55, 0.50, 0.44, 1.0];
const WATER_RGBA: [f32; 4] = [0.20, 0.34, 0.46, 1.0];
const PUDDLE_RGBA: [f32; 4] = [0.30, 0.48, 0.60, 1.0];
const FLOATER_RGBA: [f32; 4] = [0.90, 0.72, 0.32, 1.0];
const SWIMMER_RGBA: [f32; 4] = [0.42, 0.66, 0.94, 1.0];
/// O nadador CINEMÁTICO — a pergunta do Enio (*"Kinematic não vai nadar?"*)
/// respondida na tela, e não só nos gates.
const KIN_RGBA: [f32; 4] = [0.46, 0.82, 0.55, 1.0];

const CAMERA_CENTRE: [f32; 2] = [0.0, -1.5];
const CAMERA_HEIGHT: f32 = 14.0;

/// Um dos três sujeitos — idênticos em tudo menos na capacidade e no MODO.
fn subject(world: &mut World, name: &str, y: f32, tint: [f32; 4], swim: f32, kinematic: bool) {
    let mut e = world.spawn((
        Name::new(name.to_string()),
        RigidBody {
            kind: if kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: CAP_HALF_H,
                radius: CAP_RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: FLOAT,
            swim_speed: swim,
            swim_acceleration: SWIM_ACCEL,
            ..PlatformPlayer::default()
        },
        Sprite::atlas(
            WHITE_TILE_KEY,
            [CAP_RADIUS * 2.0, (CAP_HALF_H + CAP_RADIUS) * 2.0],
            tint,
        ),
        // ⚠️ Os três em `x` IGUAL e alturas diferentes: com o mesmo dedo eles
        // andam juntos, e empilhá-los em `x` os faria disputar o mesmo espaço.
        Transform::from_translation(Vec2::new(DOCK_X, y)),
    ));
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
}

pub(crate) fn build_swim_scene(world: &mut World) {
    // O CAIS — sólido, topo em y = 0.
    world.spawn((
        Name::new("Dock"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: DOCK_HALF[0],
                half_y: DOCK_HALF[1],
            },
            density: 1.0,
            ..Collider::default()
        },
        Sprite::atlas(
            WHITE_TILE_KEY,
            [DOCK_HALF[0] * 2.0, DOCK_HALF[1] * 2.0],
            DOCK_RGBA,
        ),
        Transform::from_translation(Vec2::new(DOCK_X, -DOCK_HALF[1])),
    ));

    // A POÇA RASA, em cima do cais — a que se atravessa A PÉ.
    world.spawn((
        Name::new("Puddle"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: PUDDLE_HALF[0],
                half_y: PUDDLE_HALF[1],
            },
            density: 1.0,
            ..Collider::default()
        },
        AreaBuoyancy(FLUID_DENSITY),
        AreaDrag(POOL_DRAG),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [PUDDLE_HALF[0] * 2.0, PUDDLE_HALF[1] * 2.0],
            PUDDLE_RGBA,
        ),
        Transform::from_translation(Vec2::new(PUDDLE_X, PUDDLE_HALF[1])),
    ));

    // A POÇA FUNDA — superfície em y = 0, à direita do cais.
    world.spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: POOL_HALF[0],
                half_y: POOL_HALF[1],
            },
            density: 1.0,
            ..Collider::default()
        },
        AreaBuoyancy(FLUID_DENSITY),
        AreaDrag(POOL_DRAG),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [POOL_HALF[0] * 2.0, POOL_HALF[1] * 2.0],
            WATER_RGBA,
        ),
        Transform::from_translation(Vec2::new(POOL_X, -POOL_HALF[1])),
    ));

    // O FUNDO da piscina — sólido, para o mergulho ter destino.
    world.spawn((
        Name::new("Bed"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: POOL_HALF[0],
                half_y: 0.5,
            },
            density: 1.0,
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [POOL_HALF[0] * 2.0, 1.0], DOCK_RGBA),
        Transform::from_translation(Vec2::new(POOL_X, BED_Y - 0.5)),
    ));

    subject(world, "Floater", FLOAT, FLOATER_RGBA, 0.0, false);
    subject(
        world,
        "Swimmer",
        FLOAT + 1.6,
        SWIMMER_RGBA,
        SWIM_SPEED,
        false,
    );
    subject(world, "KinSwimmer", FLOAT + 3.2, KIN_RGBA, SWIM_SPEED, true);
}

#[cfg(test)]
#[path = "physics_smoke_swim_tests.rs"]
mod tests;

impl crate::App {
    pub(crate) fn physics_smoke_swim(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_swim_scene(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 105] NADAR (W-Swim).\n  \
               Tres capsulas IDENTICAS; o que muda e' UM knob e UM modo:\n    \
                 AMBAR = swim_speed 0 (a capacidade nasce assim) -- a BOIA, o CONTROLE\n    \
                 AZUL  = swim_speed {s:.1} m/s -- o NADADOR (corpo dinamico)\n    \
                 VERDE = o mesmo nadador, corpo CINEMATICO\n  \
               Os tres recebem a MESMA entrada: ha' um teclado, logo um dedo.\n\n  \
               1. ATRAVESSE A POCA RASA (D). Os tres tem de CAMINHAR por ela, sem\n     \
                  nadar. E' o LIMIAR: de pe' no cais o corpo le' buoyed = 0,68,\n     \
                  abaixo do 1,0 que arma o regime. Se o azul comecar a nadar\n     \
                  numa poca que da' pela canela, PARE -- e' o arco de todo salto\n     \
                  sobre agua rasa que se perde junto.\n\n  \
               2. CAIA NA POCA FUNDA (continue D ate' o fim do cais). Os tres\n     \
                  entram; a partir dali eles deixam de fazer a mesma coisa.\n\n  \
               3. SEGURE W (pulo) DENTRO D'AGUA. O azul E O VERDE SOBEM -- o botao\n     \
                  virou BRACADA, e a especie do corpo nao e' uma pergunta que a\n     \
                  agua faca. O ambar nao faz nada com ele.\n     \
                  (!) E o azul NAO PULA: quem nada nao pula, e o coyote nao e'\n     \
                  gasto por um pulo que nao houve.\n\n  \
               4. SEGURE S (baixo). O azul MERGULHA ate' o fundo; o ambar continua\n     \
                  a boiar. Medido, a altura media da segunda metade de seis\n     \
                  segundos, largado submerso:\n       \
                    AZUL   baixo {d:+.2} m · parado {i:+.2} · cima {u:+.2}\n       \
                    AMBAR  {f:+.2} nas TRES -- os botoes sao mudos sem a capacidade\n     \
                  (!) SOLTE TUDO: o azul volta a' SUPERFICIE e FICA la'. O repouso\n     \
                  do nado e' a LINHA (o Swim Line da §14, em pesos), e o default\n     \
                  1,0 e' a mesma altura em que o ambar boia -- 25% submerso nesta\n     \
                  poca. Quanto fica submerso e' a razao entre as DUAS densidades\n     \
                  (a do fluido na zona, a do corpo no collider): 2x da' 50%,\n     \
                  1,25x da' 80%. Se o azul afundar e ficar no fundo parado, PARE.\n     \
                  (!) MERGULHAR PEDE AUTORIDADE: nesta poca o empuxo liquido vale\n     \
                  ~29,4 m/s^2, e o ponto de partida da lei e' 12 -- esta cena usa\n     \
                  {a:.0}. Com 12 o azul so' subiria mais devagar, e isso e' a\n     \
                  fisica da cena, nao um defeito.\n\n  \
               5. A/D DENTRO D'AGUA. O azul nada de lado com o orcamento DELE; o\n     \
                  ambar so' e' arrastado. A caminhada CALA na agua -- se o azul\n     \
                  acelerasse como no chao, seriam dois servos no mesmo eixo.\n\n  \
               6. NADE PARA FORA (W ate' passar do cais e depois D). Sair da agua\n     \
                  larga a trava: fora dela ele volta a ser um personagem no ar.\n     \
                  (!) Sair com o W apertado ENCHE o buffer do pulo -- se houver\n     \
                  chao dentro de 0,1 s ele salta. E' o `hop out`, e e' o preco\n     \
                  honesto de um botao com dois significados.\n\n  \
               (!) O card SWIM esta' na §14 do Inspector (selecione o azul).\n      \
                   Baixe o Swim Line para 0,3 e atravesse a poca rasa outra vez:\n      \
                   agora ele NADA nela -- e la' dentro ele fica mais ALTO, porque\n      \
                   o mesmo numero e' a porta E o repouso.\n\n  \
               (!) Toque B para o contorno: as duas pocas ficam magenta (sensor).\n",
            s = SWIM_SPEED,
            a = SWIM_ACCEL,
            d = DIVE_IDLE_RISE[0],
            i = DIVE_IDLE_RISE[1],
            u = DIVE_IDLE_RISE[2],
            f = FLOATER_RISE,
        );
    }
}
