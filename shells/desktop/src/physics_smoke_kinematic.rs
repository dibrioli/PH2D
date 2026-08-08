//! **Cena 101 — OS DOIS MODOS, LADO A LADO** (W-KinMove).
//!
//! ⚠️ **Uma cena, dois personagens, uma entrada.** O `hand_input_to_players`
//! entrega UM dedo a todos os players, então as duas cápsulas andam e pulam
//! juntas — e é isso que torna a comparação honesta: tudo o que difere entre
//! elas é o modo, e nada mais.
//!
//! # ⚠️ A cena roda no DEFAULT, e a primeira versão dela não rodava
//!
//! Ela nascia com `spring_damping` a um quarto do teto — o valor que os GATES
//! usam, porque no default os dois defeitos que o modo promete zerar já são zero
//! nos dois modos (o §0 do plano 07 escreve isso). Levar esse valor para a mão
//! do artista foi o erro: o smoke de 2026-08-08 voltou com *"mola extremamente
//! exagerada, um pula-pula"*, e estava certo.
//!
//! Medido (`probe_scene_101::probe_what_makes_the_bounce_29_metres`), o quique
//! precisa de **TRÊS** condições ao mesmo tempo — e cada uma sozinha dá zero:
//!
//! | rampa | berço | `damping` | quique |
//! |---|---|---|---|
//! | plano | fora do repouso | 0,25 | 0,0 mm |
//! | rampa | **no repouso** | 0,25 | 0,0 mm |
//! | rampa | fora do repouso | 0,50 | 0,0 mm |
//! | **rampa** | **fora do repouso** | **0,25** | **5913 mm** |
//!
//! O berço era o gatilho: a cena punha o ciano a `0,334 m` da rampa quando a
//! perna dele repousa a `0,900`, ou seja **comprimida meio metro** — e meio
//! metro de compressão vezes a rigidez de 2000 é uma catapulta de `1132 m/s²`
//! que só o amortecimento no teto consegue engolir.
//!
//! ⇒ Agora os dois nascem **na própria altura de repouso** e no default. Quem
//! quiser ver o preço do knob baixa-o pelo painel, que é o passo 4.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PlatformPlayer, PlayerMode, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::physics_smoke_player::slab;

/// A altura de flutuação da cena — a mesma das outras cenas de player.
const FLOAT: f32 = 0.9;
/// **A altura de repouso do modo Snap, MEDIDA** — a cápsula pousa (`0,5` de
/// meia-altura) mais a PELE do controlador (`predict_ground = offset + 0,05`).
/// Não é escolha: é o número que
/// `probe_scene_101::probe_what_differs_at_the_shipping_default` imprime.
const SNAP_REST: f32 = 0.5566;

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

        // ⚠️ **Cada um na PRÓPRIA altura de repouso, sobre o chão plano** — ver o
        // aviso do módulo. O topo do chão é `y = 0`, então a altura de repouso é
        // o número de cada modo, sem aritmética de rampa a errar.
        player(
            world,
            "Spring",
            Vec2::new(-2.0, FLOAT),
            [0.25, 0.85, 1.0, 1.0],
            false,
        );
        player(
            world,
            "Snap",
            Vec2::new(-1.0, SNAP_REST),
            [1.0, 0.65, 0.25, 1.0],
            true,
        );

        eprintln!(
            "[physics-smoke 101] OS DOIS MODOS (W-KinMove). CIANO = Spring (a capsula\n\
             que PAIRA numa perna elastica) e LARANJA = Snap (o controlador\n\
             cinematico, que POUSA). Os dois no default que shipa, cada um nascido\n\
             na propria altura de repouso.\n\
             \n\
             ⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n\
             \n\
             ⚠️ LEIA ISTO ANTES: no DEFAULT os dois modos sao quietos. A deriva de\n\
             rampa e' 0,0000 m nos DOIS, e quem afunda menos no pouso e' o CIANO\n\
             (0,0 mm contra 44 mm do laranja -- a PELE do controlador). O modo novo\n\
             NAO conserta o que ja e' zero; o que ele compra e' o passo 4.\n\
             \n\
             1) OLHE A ALTURA. O ciano paira 0,900 m acima do chao e o laranja\n\
                pousa a 0,557 -- 34 cm de diferenca, e e' o desenho de cada um:\n\
                um tem perna, o outro tem cápsula.\n\
             \n\
             2) ANDE (setas <- ->). Os dois andam JUNTOS -- e' um dedo so' para\n\
                todos os players, entao a caminhada tem de ser a MESMA lei. Se um\n\
                ficar para tras, a lei divergiu. Suba a rampa (a esquerda) e o\n\
                degrau (a direita, x = 6): os dois tem de vencer os dois.\n\
             \n\
             3) PARE NA RAMPA por ~10 s. NENHUM dos dois pode escorregar, e\n\
                nenhum pode subir sozinho. (Era aqui que a versao anterior desta\n\
                cena mentia: ela detunava a mola para fabricar uma diferenca.)\n\
             \n\
             4) O QUE O MODO COMPRA -- faca este passo, e' a wave inteira:\n\
                selecione o CIANO, abra 'Platform Player' no Inspector e baixe o\n\
                'Spring Damping' de 1,00 para ~0,25. Agora leve os dois ao\n\
                plateau (em cima da rampa) e deixe-os cair.\n\
                O CIANO passa a afundar ~156 mm no pouso e a escorregar na rampa;\n\
                o LARANJA continua exatamente igual (44 mm, e nao cresce com a\n\
                altura da queda). O numero do laranja NAO depende de knob nenhum\n\
                -- e' isso que 'estrutural' quer dizer aqui.\n\
             \n\
             5) O GESTO. Com um deles selecionado, use o chip 'Body:\n\
                Dynamic | Kinematic' na mesma secao. Ele tem de continuar la'\n\
                NOS DOIS estados -- e' o caminho de volta. Trocar move o\n\
                personagem ~34 cm na vertical, porque a altura de repouso dos\n\
                dois modos e' diferente (passo 1)."
        );
    }
}
