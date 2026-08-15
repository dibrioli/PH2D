//! **A cena 117 — O ESTOURO** (`W-Launch`), o empurrão de fora.
//!
//! Três personagens IDÊNTICOS, um por MODO, no mesmo chão — e a ferramenta de
//! interação já existente (a **explosão**) entre eles.
//!
//! # ⚠️ Ela é uma CORREÇÃO, e o mundo de antes está medido
//!
//! O `PhysicsWorld::explode` pula todo corpo que não é `Dynamic`, então o
//! estouro alcançava **1** corpo sob Spring e **ZERO** sob Snap e Pure: o botão
//! existia, o toast dizia *"0 corpos"*, e dois dos três personagens ficavam
//! parados ao lado de uma explosão.
//!
//! # ⚠️ E metade da wave é a JANELA, que só se vê no dinâmico
//!
//! Sob Spring o impulso **sempre** chegou — e a caminhada apagava-o em **9
//! tiques (0,15 s)**: `13,92 m/s` no primeiro, `0,000` no décimo, com o jogador
//! a não tocar em nada; quem come é o **freio**, não o direcional. Com a janela,
//! o MESMO estouro leva-o `5,808 m` em meio segundo em vez de `1,031`.
//!
//! # ⚠️ Os números do roteiro saem das sondas, não do olho
//!
//! `measure_launch.rs` (o mundo de antes) e `player_launch::measure_what_the_
//! explosion_now_does` (o de agora).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PlatformPlayer, PlayerMode, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::App;
use crate::physics_smoke_player::slab;

/// A altura de flutuação da cena — a das outras cenas de player.
pub(crate) const FLOAT: f32 = 0.9;
/// A distância entre personagens, m — larga o bastante para o artista poder
/// estourar **entre** dois deles e ver os dois responderem.
pub(crate) const LANE_SPAN: f32 = 4.0;

/// Os três modos da cena, da esquerda para a direita.
///
/// ⚠️ **O CONTROLE é o da esquerda**, e ele não é "sem componente": é o modo
/// que o estouro **já** alcançava. É contra ele que os outros dois se leem.
pub(crate) const LANES: [(&str, Option<PlayerMode>, [f32; 4]); 3] = [
    ("Spring", None, [0.25, 0.85, 1.0, 1.0]),
    ("Snap", Some(PlayerMode::Kinematic), [1.0, 0.72, 0.25, 1.0]),
    ("Pure", Some(PlayerMode::Pure), [0.72, 0.55, 1.0, 1.0]),
];

/// Onde o personagem `i` nasce.
#[must_use]
pub(crate) fn lane_x(i: usize) -> f32 {
    -LANE_SPAN + i as f32 * LANE_SPAN
}

fn player(
    world: &mut bevy_ecs::world::World,
    name: &str,
    at: Vec2,
    tint: [f32; 4],
    mode: Option<PlayerMode>,
) -> Entity {
    let mut e = world.spawn((
        Name::new(name.to_string()),
        Transform::from_translation(at),
        Sprite::atlas(WHITE_TILE_KEY, [0.4, 1.0], tint),
        RigidBody {
            kind: if mode.is_some() {
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
    // decide a lei e quem escreve a pose, o `RigidBody.kind` decide o que o corpo
    // é no rapier. Escrever só um monta um estado que o gesto não produz.
    if let Some(m) = mode {
        e.insert(m);
    }
    e.id()
}

impl App {
    /// **O estouro** — o empurrão de fora, nos três modos.
    pub(crate) fn physics_smoke_blast(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = build_blast_scene(gfx.sim.world_mut());
        eprintln!("{BLAST_SMOKE_MESSAGE}");
    }
}

/// **A geometria da cena 117**, separada do `App` de propósito — é ela que os
/// gates dirigem, e não uma reconstrução deles.
pub(crate) fn build_blast_scene(world: &mut bevy_ecs::world::World) -> Vec<Entity> {
    slab(
        world,
        "Floor",
        Vec2::new(0.0, -0.5),
        [14.0, 0.5],
        0.0,
        [0.35, 0.35, 0.4, 1.0],
    );
    // ⚠️ **SEM caixas, e a ausência é medida.** Duas versões desta cena puseram
    // uma ao lado de cada personagem como *régua viva* — e nas duas ela entrou no
    // caminho: à FRENTE, o do meio andava `0,740 m` contra `4,979` do vizinho (a
    // caixa bloqueava-o a 0,9 m); ATRÁS, a caixa de um é a da FRENTE do outro, e
    // o da esquerda parava a meio contra ela. As raias estão a 4 m e o empurrão
    // leva-os 5 a 8 — **qualquer** objecto entre eles é um obstáculo, não uma
    // régua. A comparação são os três personagens, e o mundo de antes está na
    // mensagem.
    LANES
        .iter()
        .enumerate()
        .map(|(i, (tag, mode, tint))| player(world, tag, Vec2::new(lane_x(i), FLOAT), *tint, *mode))
        .collect()
}

/// O roteiro da cena 117 — ⚠️ **os números saem das sondas**, e não do olho.
pub(crate) const BLAST_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 117] O ESTOURO (W-Launch). Tres personagens IGUAIS, um por\n",
    "MODO, no mesmo chao: ESQUERDA Spring - MEIO Snap - DIREITA Pure.\n",
    "Sem mais nada no caminho: as raias estao a 4 m e o empurrao leva-os 5 a 8,\n",
    "entao qualquer objecto entre eles seria um obstaculo, nao uma regua.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam -- e o teclado dirige os TRES ao\n",
    "mesmo tempo. A tecla W abre o painel de fisica; a tecla B liga/desliga o\n",
    "desenho dos colliders.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de' Play. Na secao INTERACTION do\n",
    "    Inspector escolha a ferramenta EXPLODE e clique no chao logo a'\n",
    "    ESQUERDA de cada personagem, um de cada vez. (O raio da ferramenta e'\n",
    "    3 m: um clique so' nao alcanca os tres.)\n",
    " 2. OS TRES SAEM DO LUGAR -- 5.57 m o da esquerda, 8.09 os outros dois em\n",
    "    meio segundo -- e o toast conta UM corpo em cada clique. ⚠️ Antes desta\n",
    "    wave o estouro alcancava ZERO corpos sob Snap e Pure: os dois da\n",
    "    direita ficavam parados ao lado da explosao e o toast dizia '0'.\n",
    "    (Se algum ficar parado agora, PARE.)\n",
    " 2b. ⚠️ E eles NAO andam o mesmo, de proposito: o da esquerda e' travado\n",
    "    pelo SOLVER quando a janela acaba, e os outros dois pela caminhada, que\n",
    "    rampeia. Saem quase juntos (15.3 contra 17.9 m/s) e param diferente.\n",
    " 3. Olhe o da ESQUERDA em particular: ele SEMPRE foi alcancado, e agora vai\n",
    "    MUITO mais longe -- 5.57 m em meio segundo contra 1.03 antes. A\n",
    "    diferenca nao e' forca, e' a JANELA: sem ela a caminhada apagava o\n",
    "    empurrao em 9 tiques (0.15 s).\n",
    " 4. Repita SEGURANDO a direcao CONTRARIA ao estouro. Os tres ainda sao\n",
    "    empurrados: um empurrao que o dedo do jogador apaga nao e' um empurrao.\n",
    "    (Se algum ficar preso no lugar, PARE.)\n",
    " 5. Espere ele parar e ande normalmente. O controle volta INTEIRO assim que\n",
    "    a janela acaba -- o personagem nao pode ficar 'pesado' depois.\n",
    " 6. OS AJUSTES: no painel de fisica (tecla W), secao Interaction, suba o\n",
    "    Blast Impulse e repita. Os tres vao mais longe, na mesma proporcao.\n",
    " 7. E a MASSA manda: selecione um deles, ponha Mass: Manual = 8 kg, de'\n",
    "    Reset (a massa so' e' re-lida com o relogio no inicio) e estoure de\n",
    "    novo. Ele quase nao sai do lugar -- 0.22 m contra 7.11.\n",
    "\n",
    "O QUE ISTO ACRESCENTA: um empurrao de fora e' a unica coisa que o mundo faz\n",
    "a um personagem que o controlador dele nao sabe fazer. Sem esta porta, uma\n",
    "explosao, uma almofada de salto ou um cano de vento so' existiam para quem\n",
    "estivesse no modo dinamico -- e mesmo la' duravam 0.15 s.\n",
);

#[cfg(test)]
#[path = "physics_smoke_blast_tests.rs"]
mod tests;
