//! **A CENA 119 — A BEIRADA** (`W-Brink`, o `bCanWalkOffLedges` do Unreal): ele
//! anda para a quina e **PARA**, em vez de cair dela.
//!
//! ⚠️ **As duas raias são a MESMA cena com a MESMA autoria**, e só a trava
//! difere — é isso que torna o contraste legível: o que muda entre elas não é o
//! patamar, não é o personagem e não é a velocidade, é uma escolha.
//!
//! # ⚠️ A terceira raia é a que responde à pergunta difícil
//!
//! O primeiro desenho desta wave lia a quina dos **pés que perderam o chão**, e
//! a sonda o refutou: sobre uma **fenda de 5 cm** — que o corpo de 40 cm
//! atravessa sem esforço — o veredito acendia na mesma. O leque só amostra
//! DENTRO da pegada, e ali *"o chão acaba"* e *"há um buraco à frente"* são
//! indistinguíveis. A raia da fenda é o que põe isso na tela: com a trava
//! armada ele **atravessa** as fendas que a perna vence e **pára** na que ela
//! não vence.
//!
//! # ⚠️ O número que decide o alcance está MEDIDO, e é uma soma de duas metades
//!
//! A lei dá a distância de PARAGEM (`v²/2a`) e a ponte soma a **meia-largura do
//! corpo**, porque a pergunta certa é *"quando eu parar, ainda haverá chão onde
//! a minha BORDA estiver?"*. Sem a segunda parcela o alcance é o caso de
//! fronteira: medido a 2 m/s, ele acabava equilibrado num pé só sobre o lábio
//! **e caía na mesma**.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PlatformPlayer, RigidBody,
};

/// A que distância uma raia fica da outra — larga o bastante para o personagem
/// de uma nunca alcançar o patamar da vizinha.
const LANE_SPAN: f32 = 16.0;

/// Meia-largura do patamar de cada raia.
const SLAB_HALF: f32 = 4.0;

/// A velocidade de cruzeiro das três raias — a MESMA nas três.
///
/// ⚠️ Ela é o que faz o alcance ser o que é (`v²/2a`), então mudá-la muda o
/// ponto em que ele para. É por isso que ela é uma const e não um literal
/// espalhado: o gate da cena lê-a, e não uma segunda cópia.
pub(crate) const WALK_SPEED: f32 = 6.0;

/// A largura da fenda que a raia 3 põe no caminho.
///
/// ⚠️ **MEDIDA, não escolhida:** a perna de três pés cobre `±0,18 m` do centro,
/// então ela vence um vão até ~0,36 m. `0,25` está confortavelmente dentro, e a
/// raia mostra o personagem a **atravessá-lo** com a trava armada — que é a
/// metade que o primeiro desenho da wave não conseguia.
pub(crate) const GAP: f32 = 0.25;

/// Onde o patamar de cada raia está.
const SLAB_Y: f32 = -0.5;
/// A altura de repouso do personagem.
const FLOAT: f32 = 0.9;

/// **A geometria da cena 119**, separada do `App` de propósito — é ela que os
/// gates montam, sem janela e sem GPU.
pub(crate) fn build_brink_scene(world: &mut World) -> [Entity; 3] {
    let mut lane = |i: usize, name: &str, walk_off: bool, gap: Option<f32>| {
        let x0 = i as f32 * LANE_SPAN;
        // O patamar: acaba em `x0 + SLAB_HALF`, que e' a QUINA desta raia.
        world.spawn((
            Name::new(format!("{name} Slab")),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: SLAB_HALF,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x0, SLAB_Y)),
        ));
        // A raia da FENDA ganha um segundo patamar do outro lado do vao.
        if let Some(g) = gap {
            world.spawn((
                Name::new(format!("{name} Far Slab")),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: SLAB_HALF,
                        half_y: 0.5,
                    },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(x0 + 2.0 * SLAB_HALF + g, SLAB_Y)),
            ));
        }
        world
            .spawn((
                Name::new(name.to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Capsule {
                        half_height: 0.3,
                        radius: 0.2,
                    },
                    ..Collider::default()
                },
                LockRotation,
                PlatformPlayer {
                    float_height: FLOAT,
                    speed: WALK_SPEED,
                    walk_off_ledges: walk_off,
                    ..PlatformPlayer::default()
                },
                // Comeca bem atras da quina, para o artista ver o passo inteiro.
                Transform::from_translation(Vec2::new(x0 - SLAB_HALF + 0.5, FLOAT)),
            ))
            .id()
    };
    [
        lane(0, "Walks Off", true, None),
        lane(1, "Stops At Edge", false, None),
        lane(2, "Crosses The Gap", false, Some(GAP)),
    ]
}

/// O roteiro. ⚠️ **Ele imprime o que montou** — se a linha não aparecer, o resto
/// do smoke não significa nada.
pub(crate) const BRINK_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 119] A BEIRADA (W-Brink) -- ele para' na quina.\n",
    "  Tres raias, a MESMA autoria, so' a trava difere.\n",
    "    esquerda  'Walks Off'       -- Walk Off Ledges: Yes  (o mundo de sempre)\n",
    "    meio      'Stops At Edge'   -- Walk Off Ledges: Stop At Edge\n",
    "    direita   'Crosses The Gap' -- a MESMA trava, com uma fenda de 0,25 m\n",
    "\n",
    "  1) Play. Ande com o direcional para a DIREITA nas tres raias.\n",
    "  2) A da ESQUERDA cai do patamar -- e' o CONTROLE, e tem de cair.\n",
    "  3) A do MEIO para' na quina, com a borda do corpo junto dela.\n",
    "     Ande para a ESQUERDA: ele sai de la' normalmente (a trava corta UM\n",
    "     sentido, nunca os dois).\n",
    "  4) A da DIREITA ATRAVESSA a fenda e so' para' na quina do fim -- uma\n",
    "     fenda que a perna vence nao e' um patamar.\n",
    "  5) PULE da quina na raia do meio: o pulo continua a funcionar. A trava\n",
    "     governa ANDAR, e so' isso.\n",
);

impl crate::App {
    /// **A cena 119** — o roteiro está em [`BRINK_SMOKE_MESSAGE`].
    pub(crate) fn physics_smoke_brink(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        build_brink_scene(gfx.sim.world_mut());
        eprint!("{BRINK_SMOKE_MESSAGE}");
    }
}
