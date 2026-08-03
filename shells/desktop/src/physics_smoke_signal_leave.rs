//! **A PORTA QUE FECHA** (`PH2D_PHYSICS_SMOKE=76`, W-SignalLeave).
//!
//! A cena 73 deu à física o poder de GRITAR quando algo chega. O W-Signal deferiu
//! o outro extremo com o motivo escrito: *emitir os dois sob o MESMO nome
//! tornaria o sinal ambíguo — quem escuta não saberia se a porta abriu ou
//! fechou*. Esta cena é o extremo que faltava, construído como o motivo prescreve:
//! um **segundo nome**.
//!
//! ⚠️ **O CONTROLE é a terceira pista, e sem ele a cena não prova nada:** uma
//! porta marcada só na chegada tem de ficar em SILÊNCIO ao ser deixada. Se ela
//! gritasse o nome de chegada outra vez, toda cena já autorada passaria a
//! disparar o dobro.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, GravityScale, InitialVelocity, RigidBody, SignalOnHit,
    SignalOnLeave,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const DOOR_RGBA: [f32; 4] = [0.90, 0.55, 0.88, 1.0];
const BELL_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 1.0];
const HALF_RGBA: [f32; 4] = [0.55, 0.58, 0.62, 1.0];
const WALKER_RGBA: [f32; 4] = [0.45, 0.80, 0.95, 1.0];
const GROUND_RGBA: [f32; 4] = [0.35, 0.45, 0.40, 1.0];

const CAMERA_CENTRE: [f32; 2] = [0.0, 2.2];
const CAMERA_HEIGHT: f32 = 11.0;

/// A velocidade com que os andarilhos cruzam as portas. **Medida, não escolhida:**
/// devagar o bastante para a chegada e a saída ficarem visivelmente separadas no
/// toast, e rápida o bastante para a travessia caber nos primeiros segundos.
const WALK_SPEED: f32 = 2.5;

/// Onde cada andarilho começa — à ESQUERDA da porta dele.
const START_X: f32 = -5.0;

/// A altura das três pistas.
pub(crate) const LANE_Y: [f32; 3] = [4.2, 2.6, 1.0];
pub(crate) const LANE_NAMES: [&str; 3] = ["Door", "Bell", "Half"];

fn cuboid(hx: f32, hy: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: hx,
            half_y: hy,
        },
        density: 1.0,
        ..Collider::default()
    }
}

/// Um andarilho: cruza a pista da esquerda para a direita, sem peso (a gravidade
/// o faria cair da pista antes de sair da porta, e o que esta cena mede é o
/// EXTREMO da travessia, não a queda).
fn walker(world: &mut World, i: usize, y: f32) {
    world.spawn((
        Name::new(format!("Walker {}", i + 1)),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        GravityScale(0.0),
        InitialVelocity {
            linvel: [WALK_SPEED, 0.0],
            angvel: 0.0,
        },
        cuboid(0.25, 0.25),
        Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], WALKER_RGBA),
        Transform::from_translation(Vec2::new(START_X, y)),
    ));
}

pub(crate) fn build_signal_leave_scene(world: &mut World) {
    world.spawn((
        Name::new("Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        cuboid(8.0, 0.3),
        Sprite::atlas(WHITE_TILE_KEY, [16.0, 0.6], GROUND_RGBA),
        Transform::from_translation(Vec2::new(0.0, -0.3)),
    ));

    // 1. A PORTA — um SENSOR com os DOIS nomes. O andarilho atravessa: `open`
    //    quando entra, `close` quando sai.
    world.spawn((
        Name::new(LANE_NAMES[0]),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            ..cuboid(1.2, 0.6)
        },
        SignalOnHit("door_open".to_string()),
        SignalOnLeave("door_close".to_string()),
        Sprite::atlas(WHITE_TILE_KEY, [2.4, 1.2], DOOR_RGBA),
        Transform::from_translation(Vec2::new(0.0, LANE_Y[0])),
    ));

    // 2. O SINO — um corpo SÓLIDO com os dois nomes. O andarilho BATE nele
    //    (`hit`) e QUICA para longe (`part`): o extremo sólido do mesmo par.
    world.spawn((
        Name::new(LANE_NAMES[1]),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            restitution: 0.9,
            ..cuboid(0.3, 0.6)
        },
        SignalOnHit("bell_hit".to_string()),
        SignalOnLeave("bell_part".to_string()),
        Sprite::atlas(WHITE_TILE_KEY, [0.6, 1.2], BELL_RGBA),
        Transform::from_translation(Vec2::new(0.0, LANE_Y[1])),
    ));

    // 3. O CONTROLE — a MESMA porta, marcada SÓ na chegada. Ela abre e **nunca
    //    fecha**: um extremo sem o componente dele é SILÊNCIO, não o outro nome.
    world.spawn((
        Name::new(LANE_NAMES[2]),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            ..cuboid(1.2, 0.6)
        },
        SignalOnHit("half_open".to_string()),
        Sprite::atlas(WHITE_TILE_KEY, [2.4, 1.2], HALF_RGBA),
        Transform::from_translation(Vec2::new(0.0, LANE_Y[2])),
    ));

    for (i, y) in LANE_Y.iter().enumerate() {
        walker(world, i, *y);
    }
}

#[cfg(test)]
#[path = "physics_smoke_signal_leave_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 76 (W-SignalLeave).** A porta que abre E fecha.
    pub(crate) fn physics_smoke_signal_leave(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_signal_leave_scene(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 76] A PORTA QUE FECHA -- o extremo que faltava.\n\n  \
               A cena 73 deu a' fisica o poder de gritar quando algo CHEGA. O outro\n  \
               extremo ficou diferido com o motivo escrito: emitir os dois sob o MESMO\n  \
               nome tornaria o sinal ambiguo -- quem escuta nao saberia se a porta abriu\n  \
               ou fechou. A resposta e' um SEGUNDO nome, e e' o que esta cena mostra.\n\n  \
               1. Toque Play e olhe o canto superior. Tres andarilhos iguais cruzam da\n     \
                  esquerda para a direita, um por pista:\n     \
                  - EM CIMA '{n0}' (rosa) -- um SENSOR com os DOIS nomes. Sobe\n       \
                    'Signal: door_open' quando ele ENTRA e 'Signal: door_close' quando\n       \
                    ele SAI. Dois toasts, nessa ordem.\n     \
                  - MEIO '{n1}' (ambar) -- um corpo SOLIDO com os dois nomes. O\n       \
                    andarilho BATE ('bell_hit') e QUICA para longe ('bell_part'): o\n       \
                    mesmo par, no extremo solido.\n     \
                  - EMBAIXO '{n2}' (cinza) -- a MESMA porta, marcada SO' na chegada.\n       \
                    Sobe 'half_open' e **nada mais**. Ela e' o CONTROLE: um extremo sem\n       \
                    o componente dele e' SILENCIO, nao o outro nome -- senao toda cena\n       \
                    ja' autorada passaria a disparar o dobro.\n\n  \
               2. AUTORE VOCE MESMO: selecione '{n2}' na Hierarquia. Na secao Physics\n     \
                  Body as DUAS ultimas rows sao campos de texto -- 'Signal on hit...' e\n     \
                  'Signal on leave...'. Escreva 'half_close' na segunda e de Play de\n     \
                  novo: agora ela fecha tambem.\n\n  \
               3. E O CONSERTO QUE VEIO JUNTO: selecione '{n0}' e olhe as duas rows.\n     \
                  Elas MOSTRAM 'door_open' e 'door_close'. Ate esta wave a row de\n     \
                  chegada era WRITE-ONLY: digitar funcionava, e re-selecionar a\n     \
                  entidade mostrava um campo em BRANCO sobre um componente que dizia\n     \
                  'door' -- indistinguivel de *o nome nao foi guardado*.\n\n  \
               (!) Arraste a regua para TRAS depois de um andarilho atravessar: nada\n      \
                   dispara. Uma descontinuidade do relogio nao e' uma saida -- o\n      \
                   consumidor fecharia uma porta que, no tempo para onde voce foi,\n      \
                   esta' aberta.\n\n  \
               (!) Os dois nomes sao dois CONTRATOS, nao um nome com uma fase. Quem\n      \
                   escuta casa numa string, e este outbox e' o MESMO em que a timeline\n      \
                   emite os sinais dos markers, que nao tem fase nenhuma.\n",
            n0 = LANE_NAMES[0],
            n1 = LANE_NAMES[1],
            n2 = LANE_NAMES[2],
        );
    }
}
