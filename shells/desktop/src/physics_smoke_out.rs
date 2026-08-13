//! **Cena 113 — A SAÍDA** (`PH2D_PHYSICS_SMOKE=113`, `W-PlayerOut` A5).
//!
//! O personagem corre, salta, aterra, arranca e agarra uma beirada — e a cena
//! existe para julgar **duas superfícies que até esta wave não existiam**: o
//! readout da §14 e os sinais que ele publica.
//!
//! ⚠️ **`Emit Signals` nasce LIGADO aqui, e em lugar nenhum mais.** O default do
//! app é desligado (senão toda cena de smoke com um personagem cuspiria toasts,
//! e a conta cairia sobre waves que nada têm com esta); esta é a única cena cuja
//! razão de existir é ouvi-lo, então ela o arma — e o passo 5 manda desligá-lo,
//! porque um opt-in que não se desfaz não é um opt-in.
//!
//! ⚠️ **O número 113 e não o 105 que a §5 do `CLAUDE.md` anuncia como livre:**
//! medido, o roteador vai até ao **112**, e o 105 é do `physics_smoke_swim`. A
//! nota sobreviveu ao facto — e o que a pegou foi o **compilador**: um braço
//! duplicado num `match` de strings é `unreachable_patterns`, que este repo
//! trata como erro no fecho. (O gate `no_two_smoke_scenes_claim_the_same_level`
//! cobre a família do `build_smoke_router`, que é uma cadeia de `if` e não tem
//! quem a avise.)
//!
//! ⚠️ **A cena imprime o readout a cada meio segundo** (`[player] …`). *Se essa
//! linha não aparecer, pare*: sem ela o resto do smoke não diz nada — não há
//! como distinguir *"a lei publicou Ground"* de *"ninguém publicou nada"*.
//!
//! # O percurso, e o que cada peça pergunta
//!
//! ```text
//!    x=0        x=6         x=16..20      x=20.8
//!    início     vão         degrau        parede colada à ponta
//! ```
//!
//! ⚠️ **A ordem das peças é MEDIDA, não escolhida** — ver [`WALL_X`]: dois
//! arranjos anteriores não produziam pulo de parede nenhum, e a sonda foi quem
//! disse.
//!
//! * o **VÃO** força um salto e um pouso — os dois eventos que um ciclo de
//!   animação consome (`player.jumped.ground`, `player.landed`);
//! * a **PAREDE** dá o pulo de parede, que é o caso em que um palpite de fora
//!   erra (*"ele estava no chão?"* responde **não** para o do ar e para o de
//!   parede, e não os distingue) ⇒ `player.jumped.wall`;
//! * o **DEGRAU ALTO** tem uma beirada ao alcance do braço ⇒
//!   `player.ledge_grabbed`;
//! * e o **ARRANQUE** está armado, então `Shift` dá `player.dashed`.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PlatformPlayer, PlayerSignals, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::physics_smoke_player::slab;

/// A altura de flutuação — a mesma das outras cenas de player.
const FLOAT: f32 = 0.9;
/// O topo do chão desta cena.
const GROUND_TOP: f32 = 0.0;
/// O VÃO — de onde a doca acaba até onde a seguinte começa.
const GAP_FROM: f32 = 5.0;
const GAP_TO: f32 = 7.0;
/// Onde a parede sobe — **logo depois do degrau**, e a posição é MEDIDA.
///
/// ⚠️ **Dois cortes anteriores não produziam pulo de parede, e a sonda os
/// derrubou.** Com ela no meio do caminho (`x = 11`) o percurso MORRIA nela
/// (cada tique depois do 156 imprimia `x = 10.50`): uma parede de 5 m não se
/// escala com um pulo de parede de 1,2 m. Movida para trás do início, ele
/// chegava a ela pelo CHÃO — e a lei recusa agarrar-se a quem está apoiado
/// (*"brushing a wall is not clinging"*), então quatro toques deliberados no
/// botão deram quatro pulos de CHÃO e nenhum de parede.
///
/// ⚠️ **A geometria que funciona é a do gate que já prova a capacidade:** o
/// personagem precisa de estar A CAIR encostado nela. Colada à ponta ela
/// simplesmente o BLOQUEIA em cima do degrau (medido: `x = 20.30` para sempre)
/// — o que ele precisa é de **um metro de vão** para sair do degrau e cair
/// rente à face, que é exactamente o rig do
/// `the_jump_kind_distinguishes_the_three`.
const WALL_X: f32 = STEP_X + 5.3;
/// Onde o degrau alto começa, e quão alto ele é.
const STEP_X: f32 = 16.5;
const STEP_TOP: f32 = 2.6;

/// Monta a cena e devolve os bits do sujeito.
pub(crate) fn build(world: &mut World) -> u64 {
    // Duas docas com um VÃO entre elas — é ele que obriga a saltar.
    slab(
        world,
        "Dock A",
        Vec2::new((GAP_FROM - 8.0) * 0.5, GROUND_TOP - 0.5),
        [(GAP_FROM + 8.0) * 0.5, 0.5],
        0.0,
        [0.30, 0.32, 0.36, 1.0],
    );
    slab(
        world,
        "Dock B",
        Vec2::new((GAP_TO + STEP_X) * 0.5, GROUND_TOP - 0.5),
        [(STEP_X - GAP_TO) * 0.5, 0.5],
        0.0,
        [0.30, 0.32, 0.36, 1.0],
    );
    // A PAREDE — colada à ponta do degrau, para quem sai dele descer rente a
    // ela. Alta o bastante para ele não a passar por cima.
    slab(
        world,
        "Wall",
        Vec2::new(WALL_X, 3.0),
        [0.3, 3.0],
        0.0,
        [0.45, 0.30, 0.30, 1.0],
    );
    // O DEGRAU ALTO — o topo dele é a beirada.
    slab(
        world,
        "Step",
        Vec2::new(STEP_X + 2.0, STEP_TOP * 0.5),
        [2.0, STEP_TOP * 0.5],
        0.0,
        [0.30, 0.36, 0.32, 1.0],
    );

    let player = world
        .spawn((
            Name::new("Runner"),
            Transform::from_translation(Vec2::new(0.0, FLOAT)),
            Sprite::atlas(WHITE_TILE_KEY, [0.4, 1.0], [0.35, 0.9, 0.4, 1.0]),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                density: 1.0,
                ..Collider::default()
            },
            // Sem ele a cápsula tomba na primeira aterragem e o readout descreve
            // um personagem a rolar.
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                // ⚠️ **As quatro capacidades ARMADAS**, porque cada uma é um
                // evento que a cena existe para ouvir. Todas nascem desligadas no
                // app, e uma cena que as deixasse assim julgaria o silêncio.
                wall_slide_speed: 2.0,
                wall_jump_height: 1.2,
                wall_jump_push: 4.0,
                dash_speed: 14.0,
                dash_time: 0.15,
                dash_cooldown: 0.4,
                ledge_grab: 0.5,
                ledge_speed: 3.0,
                ..PlatformPlayer::default()
            },
            // ⚠️ **O opt-in, e só aqui.**
            PlayerSignals,
        ))
        .id();
    player.to_bits()
}

impl crate::App {
    pub(crate) fn physics_smoke_out(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let bits = build(gfx.sim.world_mut());
        // ⚠️ **Quem sabe QUAL entidade é o sujeito é a cena** — ver o campo.
        self.player_readout_log = Some(bits);

        eprintln!(
            "[physics-smoke 113] A SAIDA DO PLAYER (W-PlayerOut). O corredor\n\
             VERDE publica o que faz: a §14 mostra, e cada transicao vira um\n\
             sinal.\n\
             \n\
             ⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n\
             ⚠️ E rode com PH2D_SIGNAL_LOG=1 -- e' o consumidor de\n\
             DIAGNOSTICO, com cursor proprio, e e' ele que mostra a ORDEM.\n\
             \n\
             0) ARME o Physics na barra de transporte (ele nasce desmarcado) e\n\
                de' Play. A cada meio segundo o log imprime uma linha\n\
                '[player] Ground facing +1 vel (...)'. ⚠️ SE ELA NAO APARECER,\n\
                PARE: sem ela nada abaixo diz nada. Com o Physics DESARMADO ela\n\
                diz '(a fisica esta' desarmada)' -- a ausencia e' o outro\n\
                readout, e e' ela que ensina o toggle.\n\
             \n\
             1) SELECIONE o Runner e olhe 'Platform Player': as tres primeiras\n\
                linhas sao POSTURE / FACING / SPEED, e elas MEXEM enquanto ele\n\
                anda. Ande para a esquerda: Facing vira 'left'.\n\
             \n\
             2) ANDE PARA A DIREITA (seta ->) e PULE (espaco) sobre o vao\n\
                (x = 5..7). No log: 'player.jumped.ground' e, ao aterrar,\n\
                'player.landed'. Sao DOIS nomes, nao um com um campo.\n\
             \n\
             3) A PAREDE esta' um metro depois do degrau: continue a andar para\n\
                a direita e SAIA do degrau -- ele cai rente a' face dela e se\n\
                agarra. Pule ali: sai\n\
                'player.jumped.wall' -- um nome PROPRIO. E' o caso que um\n\
                palpite de fora erra: 'ele estava no chao?' responde NAO para o\n\
                pulo do ar E para o de parede, e nao os distingue.\n\
             \n\
             4) O DEGRAU (x = 16): corra ate' ele e encoste na altura do topo.\n\
                Sai 'player.ledge_grabbed'. E com Shift, em qualquer lugar:\n\
                'player.dashed'.\n\
             \n\
             5) DESLIGUE 'Emit Signals' na §14 e repita o passo 2: ele salta e\n\
                aterra em SILENCIO, e o readout continua a mexer. Sao dois\n\
                canais -- um diz o que ELE E', o outro o que ACONTECEU."
        );
    }
}

#[cfg(test)]
#[path = "physics_smoke_out_tests.rs"]
mod tests;
