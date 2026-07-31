//! **A cena do RIG A PARTIR DA HIERARQUIA** (`PH2D_PHYSICS_SMOKE=67`, W-Rig).
//!
//! ⚠️ **A cena nasce com ZERO corpos e zero joints**, e isso é o desenho: um
//! personagem começa como sprites parenteados, e o gerador existe para não pedir
//! ao artista que redescreva à mão a árvore que ele já desenhou.
//!
//! A forma foi escolhida para ser exatamente o que a **corrente não expressa**:
//! o tronco tem QUATRO filhos. Uma corrente por seleção ligaria
//! `Torso→Head→ArmL→ArmR→…`, uma fila que descreve um boneco que não existe.
//!
//! E há um **grupo** no meio dos braços (um nó sem desenho, só organização) para
//! o smoke mostrar que ele fica transparente: não vira osso, e os braços se ligam
//! ao TRONCO, não a ele.
//!
//! Os números da mensagem saem da sonda `probe_smoke_67`, rodada sobre ESTAS
//! constantes.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, Transform, World};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// O tronco, e o centro do mundo do boneco.
const TORSO_Y: f32 = 3.0;
const TORSO: [f32; 2] = [0.5, 1.0];
const HEAD: [f32; 2] = [0.4, 0.4];
const ARM: [f32; 2] = [0.6, 0.2];
const LEG: [f32; 2] = [0.2, 0.7];

const SKIN: [f32; 4] = [0.85, 0.70, 0.55, 1.0];
const CLOTH: [f32; 4] = [0.35, 0.55, 0.85, 1.0];

/// Quantas PARTES o boneco tem — o número que o botão mostra.
pub(crate) const DOLL_PARTS: usize = 6;
/// E quantos joints o rig faz: uma aresta por parte que não é a raiz.
pub(crate) const DOLL_JOINTS: usize = DOLL_PARTS - 1;

const CAMERA_CENTRE: [f32; 2] = [0.0, 1.4];
const CAMERA_HEIGHT: f32 = 8.0;

/// **MEDIDO** pela sonda `probe_smoke_67`, 3 s de simulação depois do rig.
///
/// A queda do TRONCO, em metros — o boneco larga de `y = 3,0` e desaba sobre o
/// piso. É o número que separa *"o rig funcionou"* de *"nada ficou ligado"*.
pub(crate) const MEASURED_TORSO_DROP: f32 = 3.15;
/// E o quanto o par tronco↔cabeça se AFASTA nesses 3 s, em metros. Um Pin segura;
/// se este número crescesse, o rig teria feito joints que não prendem.
///
/// ⚠️ **Ele mediu 1,09 m até a W-Rig consertar a porta de criação** — o ponto
/// médio saía de `Transform` CRU, que é LOCAL num corpo parenteado, e a âncora do
/// pescoço nascia 1,65 m abaixo da emenda. O boneco esparramava, e este número é
/// a prova executável de que ele não esparrama mais.
pub(crate) const MEASURED_NECK_DRIFT: f32 = 0.003;

fn limb(
    world: &mut World,
    name: &str,
    size: [f32; 2],
    tint: [f32; 4],
    local: Vec2,
    parent: Entity,
) -> Entity {
    world
        .spawn((
            Name::new(name),
            Sprite::atlas(WHITE_TILE_KEY, size, tint),
            Transform::from_translation(local),
            ChildOf(parent),
        ))
        .id()
}

/// O boneco, **sem um único corpo** — só desenho e parentesco.
///
/// As partes se TOCAM de propósito: o joint da rota por seleção nasce no ponto
/// MÉDIO dos dois corpos, e entre dois retângulos encostados o meio É a emenda.
pub(crate) fn build_rig_doll(world: &mut World) -> Entity {
    let torso = world
        .spawn((
            Name::new("Torso"),
            Sprite::atlas(WHITE_TILE_KEY, TORSO, CLOTH),
            Transform::from_translation(Vec2::new(0.0, TORSO_Y)),
        ))
        .id();

    // A cabeça encostada no topo do tronco.
    limb(
        world,
        "Head",
        HEAD,
        SKIN,
        Vec2::new(0.0, (TORSO[1] + HEAD[1]) * 0.5),
        torso,
    );

    // ⚠️ **Um GRUPO** — nome, `Transform`, e NENHUM sprite. Ele não vira osso; os
    // braços se ligam ao tronco por cima dele.
    let arms = world
        .spawn((
            Name::new("Arms"),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            ChildOf(torso),
        ))
        .id();
    let arm_y = (TORSO[1] - ARM[1]) * 0.5;
    let arm_x = (TORSO[0] + ARM[0]) * 0.5;
    limb(world, "ArmL", ARM, SKIN, Vec2::new(-arm_x, arm_y), arms);
    limb(world, "ArmR", ARM, SKIN, Vec2::new(arm_x, arm_y), arms);

    let leg_y = -(TORSO[1] + LEG[1]) * 0.5;
    let leg_x = (TORSO[0] - LEG[0]) * 0.5;
    limb(world, "LegL", LEG, CLOTH, Vec2::new(-leg_x, leg_y), torso);
    limb(world, "LegR", LEG, CLOTH, Vec2::new(leg_x, leg_y), torso);

    torso
}

#[cfg(test)]
#[path = "physics_smoke_rig_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 67 (W-Rig).** Um boneco de sprites, sem física nenhuma — e um
    /// clique que o transforma num ragdoll.
    pub(crate) fn physics_smoke_rig(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        crate::physics_smoke::spawn_floor(gfx.sim.world_mut());
        let torso = build_rig_doll(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        // Já com o tronco marcado: a §11 abre na FACE VAZIA, que é onde o botão
        // do rig mora — e é a face que o artista de verdade encontra.
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.gizmo.selection = Some(torso.to_bits());
            hero.gizmo.extra_selection.clear();
        }

        eprintln!(
            "[physics-smoke 67] O RIG SAI DA HIERARQUIA -- um clique transforma um\n  \
               desenho parenteado num ragdoll.\n  \
               A cena nasce PARADA, o contorno JA ESTA LIGADO (B alterna) e o TRONCO\n  \
               ja esta selecionado.\n\n  \
               O boneco tem {parts} partes desenhadas e ZERO corpos -- olhe a secao\n  \
               Physics Body: ela diz 'Not simulated'. Repare na Hierarquia que o\n  \
               TRONCO tem QUATRO filhos (cabeca, o grupo 'Arms', e as duas pernas):\n  \
               isso e' uma ARVORE, e uma corrente por selecao nao consegue expressa-la\n  \
               (ela ligaria tudo em FILA).\n\n  \
               O GESTO (o ragdoll wizard do Fyrox):\n  \
               1. com o 'Torso' selecionado, a secao Physics Body mostra DOIS botoes:\n     \
                  'Add Physics Body' (so este objeto) e **'Rig {parts} Parts from\n     \
                  Hierarchy'**. A contagem esta NO rotulo, porque o clique alcanca a\n     \
                  subarvore inteira e voce tem de ver isso antes.\n  \
               2. clique em Rig. Toast: 'Rigged {parts} new bodies with {joints} joints'.\n  \
               3. De PLAY: o boneco DESABA sobre o piso e os membros se dobram --\n     \
                  o tronco cai ~{drop:.1} m e as juntas SEGURAM (o pescoco separa\n     \
                  {drift_mm:.0} mm em 3 s; ele ja separou 1090 mm, quando a ancora\n     \
                  nascia no lugar errado).\n\n  \
               O QUE CONFERIR, e vale mais que a queda:\n  \
               - a Hierarquia ganhou {joints} objetos-joint nomeados pelo par que eles\n    \
                 ligam ('Torso : Head', 'Torso : ArmL', ...). Nenhum deles se chama\n    \
                 'Arms : ArmL': o GRUPO e' transparente e os bracos penduram do\n    \
                 TRONCO. Selecione 'Arms' -- ele NAO ganhou corpo.\n  \
               - **clique em Rig de novo.** Nada acontece, e e' de proposito: uma\n    \
                 aresta que ja tem joint e' pulada. E' o que deixa voce acrescentar\n    \
                 um membro depois e re-rigar sem duplicar o que ja existe.\n  \
               - UM Ctrl+Z desfaz o rig inteiro -- os {parts} corpos e os {joints}\n    \
                 joints num passo so.\n\n  \
               (!) Depois do rig a §12 abre no ULTIMO joint: afine UM (limites, mola)\n  \
               e use Copy/Paste Properties (cena 66) para carimbar os outros.\n",
            parts = DOLL_PARTS,
            joints = DOLL_JOINTS,
            drop = MEASURED_TORSO_DROP,
            drift_mm = MEASURED_NECK_DRIFT * 1000.0,
        );
    }
}
