//! **A cena da BARRA RÍGIDA** (`PH2D_PHYSICS_SMOKE=56`, W-Rod).
//!
//! O vão que o Rod fecha é estreito o bastante para passar despercebido lendo a
//! lista de tipos, então a cena é feita de **comparações lado a lado** em vez de
//! uma demonstração isolada: um rod só se distingue *contra* os três vizinhos
//! que quase fazem a mesma coisa.
//!
//! - **Os três pêndulos** (esquerda): MESMO comprimento autorado, MESMA carga,
//!   soltos na horizontal. A **corda** afrouxa e o peso despenca antes de a
//!   linha esticar; a **mola** cede sob o peso e balança comprida; a **barra**
//!   gira com o comprimento intacto do primeiro ao último quadro.
//! - **A TRELIÇA** (direita): um ápice dinâmico preso por **duas barras** a dois
//!   pontos estáticos. É o que só um rod faz — duas cordas deixariam o ápice
//!   cair para dentro (elas só puxam), e é também a prova de que um corpo aceita
//!   mais de um joint.
//!
//! Os números da mensagem saem da sonda `probe_smoke_56`, rodada sobre ESTAS
//! peças antes de a mensagem ser escrita.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const ANCHOR: [f32; 4] = [0.75, 0.75, 0.78, 1.0];
const ROPE_TINT: [f32; 4] = [0.35, 0.8, 0.95, 1.0];
const SPRING_TINT: [f32; 4] = [0.95, 0.85, 0.3, 1.0];
const ROD_TINT: [f32; 4] = [0.45, 0.9, 0.45, 1.0];

/// A menor distância que a CORDA da cena atinge nos 3 s (autorada com `SPAN`).
///
/// ⚠️ Os quatro `MEASURED_*` saem da sonda `probe_smoke_56` e existem para a
/// mensagem afirmar NÚMEROS em vez de adjetivos — e há um gate
/// (`the_scene_message_states_the_numbers_the_sim_produces`) provando que eles
/// continuam sendo o que a simulação produz. Prosa que envelhece em silêncio é o
/// defeito que esta linha já cometeu duas vezes.
pub(crate) const MEASURED_ROPE_MIN: f32 = 0.347;
/// Idem para a MOLA.
pub(crate) const MEASURED_SPRING_MIN: f32 = 1.782;
/// Idem para a BARRA — o número que tem de continuar sendo o autorado.
pub(crate) const MEASURED_ROD_MIN: f32 = 2.0;
/// Quanto o ápice da treliça se afasta do lugar em que nasceu, em 3 s.
pub(crate) const MEASURED_TRUSS_DRIFT: f32 = 0.0;

/// O comprimento autorado de TODOS os vínculos da cena — um número só, para as
/// três colunas serem comparáveis por construção.
pub(crate) const SPAN: f32 = 2.0;

fn ball(world: &mut World, name: &str, x: f32, y: f32, r: f32, kind: BodyKind, tint: [f32; 4]) {
    world.spawn((
        Transform::from_translation(Vec2::new(x, y)),
        Sprite::atlas(WHITE_TILE_KEY, [r * 2.0, r * 2.0], tint),
        Name::new(name),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Ball { radius: r },
            density: 1.0,
            ..Collider::default()
        },
    ));
}

/// Um vínculo de DISTÂNCIA entre dois corpos, no tipo pedido.
///
/// Os três tipos que a cena compara guardam o comprimento em campos diferentes
/// (`rest_length` numa mola, `max_length` numa corda e numa barra), e essa é a
/// única diferença de autoria entre eles — por isso a função a resolve uma vez,
/// em vez de a cena repetir o `match` três vezes.
fn linked(world: &mut World, name: &str, a: &str, b: &str, at: [f32; 2], kind: JointKind) {
    let mut j = PhysicsJoint {
        body_a: stable_name_id(a),
        body_b: stable_name_id(b),
        kind,
        ..PhysicsJoint::of_kind(kind)
    };
    match kind {
        JointKind::Spring => j.rest_length = SPAN,
        _ => j.max_length = SPAN,
    }
    world.spawn((
        Transform::from_translation(Vec2::new(at[0], at[1])),
        Name::new(name),
        j,
    ));
}

/// As três colunas + a treliça. `pub(crate)` porque a sonda monta as MESMAS
/// peças que o artista abre — uma sonda com cena própria mede outra coisa.
pub(crate) fn spawn_props(world: &mut World) {
    // --- Os três pêndulos --------------------------------------------------
    // ⚠️ **A carga nasce quase EM CIMA do gancho, e a atitude é o experimento.**
    // A primeira versão desta cena os soltava na HORIZONTAL e a sonda a refutou:
    // um peso solto de lado descreve um ARCO em torno do gancho, e num arco a
    // corda fica **tesa** o tempo todo — as três colunas mediam 2,0000 e a cena
    // não distinguia nada. O que uma corda não sabe fazer é EMPURRAR, então o
    // peso tem de estar acima, com um desvio de 10° para tombar para um lado.
    for (i, (label, kind, tint)) in [
        ("Rope", JointKind::Rope, ROPE_TINT),
        ("Spring", JointKind::Spring, SPRING_TINT),
        ("Rod", JointKind::Rod, ROD_TINT),
    ]
    .into_iter()
    .enumerate()
    {
        let hx = -7.0 + i as f32 * 3.0;
        let hy = 6.0;
        ball(
            world,
            &format!("{label} Hook"),
            hx,
            hy,
            0.12,
            BodyKind::Static,
            ANCHOR,
        );
        // 80° acima da horizontal: exatamente `SPAN` de distância, e fora do
        // equilíbrio instável o bastante para tombar sem empurrão.
        ball(
            world,
            &format!("{label} Load"),
            hx + SPAN * 0.173_648_2,
            hy + SPAN * 0.984_807_8,
            0.35,
            BodyKind::Dynamic,
            tint,
        );
        linked(
            world,
            label,
            &format!("{label} Hook"),
            &format!("{label} Load"),
            [hx, hy],
            kind,
        );
    }

    // --- A treliça ----------------------------------------------------------
    // Dois pontos estáticos no alto e um ápice pendurado entre eles por DUAS
    // barras. Duas cordas deixariam o ápice cair (uma corda só puxa, e ele está
    // ABAIXO da linha entre elas — a componente que o segura de lado não existe
    // numa corda frouxa); duas molas o deixariam pulando.
    // ⚠️ **As âncoras ficam a 2,4 m e não a 4 m**, e a razão é física, não
    // estética: com elas a 4 m o ápice sentaria EXATAMENTE na linha entre as
    // duas, que é a configuração degenerada — segurar peso vertical ali exige
    // tensão infinita, e a sonda mediu o ápice cedendo 4,7 cm. Com 2,4 m ele
    // desce 1,6 m e forma um triângulo de verdade, onde as barras têm braço.
    ball(
        world,
        "Truss Left",
        4.8,
        7.0,
        0.12,
        BodyKind::Static,
        ANCHOR,
    );
    ball(
        world,
        "Truss Right",
        7.2,
        7.0,
        0.12,
        BodyKind::Static,
        ANCHOR,
    );
    // 1,2² + 1,6² = 2,0² ⇒ cada barra mede exatamente `SPAN`.
    ball(
        world,
        "Truss Apex",
        6.0,
        5.4,
        0.3,
        BodyKind::Dynamic,
        ROD_TINT,
    );
    linked(
        world,
        "Truss A",
        "Truss Left",
        "Truss Apex",
        [4.8, 7.0],
        JointKind::Rod,
    );
    linked(
        world,
        "Truss B",
        "Truss Right",
        "Truss Apex",
        [7.2, 7.0],
        JointKind::Rod,
    );
}

impl crate::App {
    /// **Cena 56 (W-Rod).** Três pêndulos comparáveis + uma treliça, TOCANDO —
    /// esta cena é sobre o que a simulação faz, não sobre um gesto de autoria.
    pub(crate) fn physics_smoke_rod(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_props(gfx.sim.world_mut());
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 56] A BARRA RIGIDA -- o vinculo que faltava.\n  \
               Aperte B para ver os joints, e depois PLAY (o toggle Physics ja esta armado).\n\n  \
               1. OS TRES PENDULOS (esquerda). Mesmo comprimento autorado (2 m), mesma\n     \
                  carga, os tres soltos na horizontal. Olhe a DISTANCIA ate o gancho:\n     \
                  - CIANO (corda): afrouxa e o peso DESPENCA antes de a linha esticar.\n     \
                  - AMARELO (mola): cede sob a carga e balanca comprida.\n     \
                  - VERDE (barra): gira com o comprimento INTACTO, do 1o ao ultimo quadro.\n     \
                  (medido, menor distancia ao longo de 3 s: corda {rope:.2} m, mola {spring:.2} m,\n      \
                  barra {rod:.2} m -- as tres autoradas com 2,00)\n  \
               2. Selecione a barra verde na Hierarquia. A secao do joint mostra UM\n     \
                  numero -- 'Length (m)'. Sem rigidez, sem limites, sem motor: um rod\n     \
                  e um numero so. Mude para 3 e a barra se estica ate 3 m.\n  \
               3. O DESENHO diz o tipo: uma barra sao DUAS linhas paralelas entre os\n     \
                  olhais -- nem o zigue-zague da mola nem a curva da corda.\n  \
               4. A TRELICA (direita): o apice verde esta preso por DUAS barras a dois\n     \
                  pontos estaticos. Arraste-o e solte: ele volta. Duas cordas o\n     \
                  deixariam cair, porque corda so PUXA.\n     \
                  (medido: o apice desvia {truss:.3} m do lugar em 3 s de simulacao)\n  \
               5. E o gesto de CRIAR: selecione dois corpos quaisquer, va na secao\n     \
                  Physics Body, escolha 'Rod' no seletor 'Join As' e clique em Join.",
            rope = MEASURED_ROPE_MIN,
            spring = MEASURED_SPRING_MIN,
            rod = MEASURED_ROD_MIN,
            truss = MEASURED_TRUSS_DRIFT,
        );
    }
}

#[cfg(test)]
#[path = "physics_smoke_rod_tests.rs"]
mod tests;
