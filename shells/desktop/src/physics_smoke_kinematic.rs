//! **Cena 101 — OS DOIS MODOS, LADO A LADO** (W-KinMove).
//!
//! ⚠️ **Uma cena, dois personagens, uma entrada.** O `hand_input_to_players`
//! entrega UM dedo a todos os players, então as duas cápsulas andam e pulam
//! juntas — e é isso que torna a comparação honesta: tudo o que difere entre
//! elas é o modo, e nada mais.
//!
//! ⚠️ **A rampa corre com o amortecimento ABAIXO do teto, de propósito.** No
//! default que shipa a deriva do modo dinâmico já é zero, e a cena mostraria dois
//! personagens idênticos — *uma cena que passa no controle está a demonstrar a
//! coisa errada*. Com o knob a um quarto, o dinâmico escorrega e o cinemático
//! não, que é a diferença que a wave entrega.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PlatformPlayer, PlayerMode, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::physics_smoke_player::slab;

/// A altura de flutuação da cena — a mesma das outras cenas de player.
const FLOAT: f32 = 0.9;
/// ⚠️ Um quarto do teto: é o que faz a rampa CONTER o fenômeno.
const DAMPING: f32 = 0.25;

fn player(
    world: &mut bevy_ecs::world::World,
    name: &str,
    at: Vec2,
    tint: [f32; 4],
    kinematic: bool,
) {
    let mut e = world.spawn((
        Name::new(name.to_string()),
        Transform::from_translation(at),
        Sprite::atlas(WHITE_TILE_KEY, [0.4, 1.0], tint),
        RigidBody {
            kind: if kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
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
            spring_damping: DAMPING,
            ..PlatformPlayer::default()
        },
    ));
    // ⚠️ **Os DOIS campos, como o chip da §14 os escreve** — o `PlayerMode`
    // decide a lei e quem escreve a pose; o `RigidBody.kind` decide o que o corpo
    // é no rapier. Escrever só um aqui seria a cena a montar um estado que o
    // gesto do artista não produz.
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
}

impl crate::App {
    pub(crate) fn physics_smoke_kinematic(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // Um chão longo, uma rampa de 30° que se sobe pela esquerda, e um degrau
        // baixo à direita — o degrau é o que mostra o `autostep` do controlador.
        slab(
            world,
            "Floor",
            Vec2::new(0.0, -0.5),
            [16.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        slab(
            world,
            "Ramp30",
            Vec2::new(-7.0, 1.3),
            [4.0, 0.5],
            -30.0_f32.to_radians(),
            [0.3, 0.5, 0.35, 1.0],
        );
        slab(
            world,
            "Plateau",
            Vec2::new(-13.0, 3.3),
            [3.5, 0.41],
            0.0,
            [0.32, 0.44, 0.36, 1.0],
        );
        slab(
            world,
            "Step",
            Vec2::new(6.0, 0.15),
            [1.5, 0.15],
            0.0,
            [0.45, 0.40, 0.30, 1.0],
        );

        // Os dois, na MESMA rampa e à mesma altura: o que se compara é a deriva.
        // ⚠️ A rampa desce para a direita, então os dois nascem sobre ela.
        player(
            world,
            "Spring",
            Vec2::new(-7.5, 2.5),
            [0.25, 0.85, 1.0, 1.0],
            false,
        );
        player(
            world,
            "Snap",
            Vec2::new(-6.0, 2.0),
            [1.0, 0.65, 0.25, 1.0],
            true,
        );

        eprintln!(
            "[physics-smoke 101] OS DOIS MODOS (W-KinMove). Uma rampa de 30deg, dois\n\
             personagens: CIANO = Spring (a capsula flutuante) e LARANJA = Snap (o\n\
             controlador cinematico). Amortecimento a 1/4 do teto de proposito -- no\n\
             default que shipa a deriva do dinamico ja e' zero, e a cena mostraria\n\
             dois personagens iguais.\n\
             \n\
             ⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n\
             \n\
             1) DEIXE-OS PARADOS na rampa por ~10 s, sem tocar em nada.\n\
                O CIANO ESCORREGA morro abaixo; o LARANJA fica onde esta'.\n\
                Medido headless: dinamico 0,0498 m, cinematico 0,0000 m.\n\
             \n\
             2) ANDE (setas <- ->). Os dois andam JUNTOS -- e' um dedo so' para\n\
                todos os players, entao a caminhada tem de ser a MESMA lei.\n\
                Medido em 2 s: 11,830 m nos DOIS, a tres decimais. Se um deles\n\
                ficar para tras, a lei divergiu.\n\
             \n\
             3) OLHE A ALTURA. O ciano PAIRA 0,4 m acima do chao (a capsula\n\
                flutuante, que e' o desenho dele) e o laranja POUSA. Medido em\n\
                repouso: 1,400 contra 1,057 -- os 5,7 cm sao a PELE do\n\
                controlador, que todo controlador tem.\n\
             \n\
             4) PULE (espaco) de uma altura grande e olhe o POUSO. O ciano\n\
                afunda e volta; o laranja quase nao mergulha, e o mergulho dele\n\
                NAO cresce com a queda -- 0,044 / 0,012 / 0,047 / 0,047 m para\n\
                quedas de 0,5 / 2 / 5 / 10 m, contra 0,052 / 0,149 / 0,261 /\n\
                0,296 do ciano.\n\
             \n\
             5) O GESTO. Selecione um deles na Hierarquia, abra a secao\n\
                'Platform Player' no Inspector e use o chip 'Body:\n\
                Dynamic | Kinematic'. Ele tem de continuar la' NOS DOIS estados\n\
                -- e' o caminho de volta.\n\
             \n\
             6) O DEGRAU a direita (x = 6): os dois tem de subi-lo andando.\n\
                O ciano sobe pela perna; o laranja pelo autostep."
        );
    }
}
