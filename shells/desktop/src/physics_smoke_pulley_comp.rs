//! **A cena da COMPOSIÇÃO** (`PH2D_PHYSICS_SMOKE=63`, W-Pulley W5).
//!
//! Irmã de `physics_smoke_pulley{,_tackle,_break,_diff}.rs` pelo cap de 600 LOC,
//! e o corte é o assunto: lá moram o elevador, o guincho, a ruptura, a talha e o
//! tambor **sozinhos**; aqui os dois ÚLTIMOS na mesma corda — a cadernal móvel do
//! W3 pendurada num tambor de dois raios do W4, que é a costura que nenhuma
//! fixture exercitava.
//!
//! Os números da mensagem saem da sonda `probe_smoke_63`, rodada sobre ESTAS
//! constantes.

use super::physics_smoke_pulley::{GROUND, POST, R, ball};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, PulleyWheel, RigidBody, WrapSide,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const GEARED: [f32; 4] = [0.45, 0.90, 0.55, 1.0];
const PLAIN: [f32; 4] = [0.95, 0.45, 0.45, 1.0];
const COUNTER: [f32; 4] = [0.30, 0.32, 0.40, 1.0];

/// **A carga**, kg. Os DOIS rigs a carregam.
///
/// Escolhida ENTRE as duas vantagens: acima dos 2 kg que a talha sozinha segura
/// e abaixo dos 8 kg que a composição segura. É esse intervalo que faz os dois
/// rigs andarem para lados opostos — qualquer massa fora dele deixaria os dois
/// subindo ou os dois caindo, e a wave inteira sumiria da tela.
const LOAD_MASS: f32 = 7.0;
/// **O contrapeso**, kg — e os dois rigs têm o mesmo.
const COUNTER_MASS: f32 = 1.0;
/// O raio por onde a corda ENTRA no tambor.
const R_IN: f32 = 0.5;
/// O raio por onde ela SAI — só o rig da esquerda o tem.
///
/// `R_IN / R_OUT` = **4**, e a cadernal móvel dobra de novo: a vantagem é **8**.
const R_OUT: f32 = 0.125;
/// O raio da cadernal móvel.
const SHEAVE_R: f32 = 0.3;
/// A meia-distância entre a ponta morta e o tambor.
const SPAN: f32 = 0.9;
/// Onde a carga nasce.
///
/// ⚠️ **Baixa de propósito, e a medição decidiu o número contra a minha primeira
/// escolha.** No rig comum a carga CAI, e enquanto ela cai o contrapeso SOBE a
/// **2×** a velocidade dela (ela pende de dois ramos). Nascendo a 3,0 ela caía
/// 2,65 m, o contrapeso subia 5,3 m **e depois saltava mais 7,8 m** — chegando a
/// 13,2 num tambor a 11,0.
///
/// ⚠️ **O salto é uma FOLGA DE CORDA, não uma degeneração**, e a medição separou
/// as duas: subir o tambor 1 m inteiro moveu o pico de 8,086 para 8,100, ou seja
/// nada. Quando a carga pousa, o contrapeso leve segue por inércia — e subir
/// ENCURTA a perna dele, o que uma corda permite (ela só puxa). O salto é
/// comparável à subida presa, então *queda longa* e *contrapeso em quadro* são
/// requisitos que brigam: perto do chão a queda termina cedo e o salto cabe.
const LOAD_Y: f32 = 1.2;
/// Onde o contrapeso nasce.
///
/// ⚠️ **Alto, e a razão é a vantagem:** o lado do contrapeso anda `2·(R/r)` = **8×**
/// o que a carga anda, então no rig engrenado ele DESPENCA enquanto a carga sobe um
/// palmo — medido, 2,22 m contra 0,28 m em 2 s (razão 8,05). Nascer no alto é o que
/// lhe dá pista sem que ele atravesse o chão.
const COUNTER_Y: f32 = 5.0;
/// A altura dos tambores.
///
/// ⚠️ **Acima do `BOOM_Y` das cenas irmãs (7,0), e não por gosto:** o teto real
/// desta cena é *até onde o contrapeso do rig COMUM é arremessado*, medido em
/// **8,10** — e o que ele não pode alcançar é o CÍRCULO do tambor, que começa em
/// `DRUM_Y − R_IN`. O gate `the_counterweight_never_reaches_its_drum` é quem mede
/// isso, e ele nasceu VERMELHO com o tambor a 7,0 (o contrapeso entrava no
/// círculo aos 0,67 s e o rig inteiro passava a oscilar).
const DRUM_Y: f32 = 10.0;

/// **Um sarilho com talha**, ou o controle sem engrenagem.
///
/// ```text
///   morta (x−0.9, DRUM)          tambor (x+0.9, DRUM)   R = 0.5 · r = 0.125
///          \                          / \
///           \                        /   contrapeso (x+0.9)
///            \                      /
///             cadernal MÓVEL (x, LOAD_Y)  [montada na CARGA]
/// ```
///
/// A corda anda **do contrapeso** (ponta A) para a **ponta morta** (B): entra no
/// tambor pelo raio grande, sai pelo pequeno, desce até a cadernal, abraça e
/// volta ao teto. Os dois ramos que seguram a carga estão **depois** do tambor,
/// então os dois já valem `R/r` — e é por isso que as duas vantagens
/// MULTIPLICAM em vez de escolher uma.
///
/// `geared = false` é o **CONTROLE**: o mesmo rig com um tambor de um raio só, e
/// aí sobra a talha sozinha (dois ramos ⇒ 2).
fn windlass_tackle(world: &mut World, tag: &str, x: f32, geared: bool, rgba: [f32; 4]) {
    let dead = format!("{tag} Post");
    let load = format!("{tag} Load");
    let counter = format!("{tag} Counterweight");
    let rope = format!("{tag} Rope");
    // A ponta morta: um poste estático. Massa infinita para a corda, que é o que
    // uma amarração no teto é.
    world.spawn((
        Name::new(dead.clone()),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.15,
                half_y: 0.15,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [0.3, 0.3], POST),
        Transform::from_translation(Vec2::new(x - SPAN, DRUM_Y)),
    ));
    ball(world, &load, x, LOAD_MASS, rgba, R * 1.4);
    ball(world, &counter, x + SPAN, COUNTER_MASS, COUNTER, R);
    for (name, y) in [(&load, LOAD_Y), (&counter, COUNTER_Y)] {
        let mut q = world.query::<(&Name, &mut Transform)>();
        for (n, mut t) in q.iter_mut(world) {
            if n.as_str() == *name {
                t.translation.y = y;
            }
        }
    }
    world.spawn((
        Name::new(rope.clone()),
        PhysicsJoint {
            // ⚠️ **O CONTRAPESO é a ponta A**, e a ordem não é decorativa: a
            // engrenagem conta a partir de A, então é ela que decide se a carga
            // está do lado MULTIPLICADO da corda. Trocar as pontas dividiria em
            // vez de multiplicar.
            body_a: stable_name_id(&counter),
            body_b: stable_name_id(&dead),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        // O `Transform` de uma corda é a ÂNCORA EM A — o ponto do contrapeso onde
        // ela se amarra, não o lugar do tambor.
        Transform::from_translation(Vec2::new(x + SPAN, COUNTER_Y)),
    ));
    // **O TAMBOR** (ordem 0): a corda o atravessa ANTES de chegar à carga.
    world.spawn((
        Name::new(format!("{rope} Drum")),
        PulleyWheel {
            rope: stable_name_id(&rope),
            order: 0,
            radius: R_IN,
            // A única diferença entre os dois rigs.
            radius_out: if geared { R_OUT } else { 0.0 },
            wrap: WrapSide::Auto,
            ..PulleyWheel::default()
        },
        Transform::from_translation(Vec2::new(x + SPAN, DRUM_Y)),
    ));
    // **A CADERNAL MÓVEL** (ordem 1): montada na carga. O `local` é semeado pela
    // ponte contra a pose de REPOUSO dela, uma vez — daí o `mounted: false`
    // implícito no default, que é sentinela e não omissão.
    world.spawn((
        Name::new(format!("{rope} Sheave")),
        PulleyWheel {
            rope: stable_name_id(&rope),
            order: 1,
            radius: SHEAVE_R,
            wrap: WrapSide::Auto,
            body: stable_name_id(&load),
            ..PulleyWheel::default()
        },
        Transform::from_translation(Vec2::new(x, LOAD_Y)),
    ));
}

/// **Quanto a carga do rig ENGRENADO sobe em 2 s**, metros.
pub(crate) const MEASURED_GEARED_RISE: f32 = 0.28;
/// **Quanto a carga do CONTROLE cai no mesmo tempo**, metros — ela para aí porque
/// encontra o CHÃO, não porque a corda a pegou.
pub(crate) const MEASURED_PLAIN_DROP: f32 = 0.75;
/// **Quanto o contrapeso do rig engrenado DESCE**, metros — `2·(R/r)` vezes o que
/// a carga sobe, e é o número que se vê de longe.
pub(crate) const MEASURED_GEARED_COUNTER_DROP: f32 = 2.22;
/// **O comprimento que a corda engrenada desta cena de fato mede**, metros — o
/// número que o piso devolve quando o artista digita um absurdo na row
/// `Rope Length (m)`. Medido por `probe_smoke_63_floor`.
pub(crate) const MEASURED_GEARED_ROPE_LENGTH: f32 = 80.8986;
/// **E o mesmo depois de um `Add Wheel`** — a roldana nova cai quase sobre a linha
/// da corda nesta cena, então o piso a estica só 0,02 m. É por isso que a mensagem
/// pede que se confira o TRANCO, não o movimento.
pub(crate) const MEASURED_GEARED_ROPE_LENGTH_ADDED: f32 = 80.9190;
/// **O maior passo que a carga engrenada dá num tique** — o mesmo na cena
/// intocada, com a roldana acrescentada e com o comprimento absurdo corrigido.
pub(crate) const MEASURED_GEARED_WORST_JUMP: f32 = 0.0046;

/// **O ENQUADRAMENTO que esta cena precisa** — o centro e a altura de mundo da
/// câmera.
///
/// ⚠️ **A câmera padrão mostra `y ∈ [−5, +5]`** (`center = (0,0)`,
/// `height_world = 10`; o `main.rs` diz isso na definição do `WORLD_HALF`), e o
/// tambor desta cena está em **y = 10** — *cinco metros acima do topo da tela*.
/// Sem enquadrar, o rig inteiro acima da carga é invisível: os postes, os
/// contrapesos e os dois tambores.
///
/// ⚠️ **E o preço não é cosmético.** As alças de roldana são desenhadas ONDE a
/// roldana está: o artista seleciona `Geared Rope Drum` na Hierarquia, o
/// publicador entrega as três alças corretamente, e nada aparece — porque elas
/// estão fora do quadro. Foi exatamente esse o report que abriu este fix, e a
/// medição separou as duas hipóteses: o publicador devolve `["Centre", "Rim",
/// "RimOut"]` para este tambor, então o defeito nunca esteve nele.
///
/// **A lei:** *uma cena de smoke enquadra o que ela pede que se olhe.* Um número
/// de geometria escolhido por motivo físico (a pista do contrapeso, W5) não sabe
/// nada sobre o que cabe na tela, e as duas coisas têm de ser reconciliadas por
/// alguém — aqui, e com gate.
pub(crate) const CAMERA_CENTRE: [f32; 2] = [0.0, 5.0];
/// A altura de mundo que cobre do chão ao topo dos tambores com folga.
pub(crate) const CAMERA_HEIGHT: f32 = 13.0;

/// ⚠️ **Enquadrar não é AFASTAR a câmera até tudo caber** — o modo de "consertar"
/// um gate de fit que o esvazia. Compile-time de propósito: mexer na const acima
/// para além disto quebra o BUILD, não um teste que alguém pode filtrar.
const _: () = assert!(CAMERA_HEIGHT < 20.0);

/// Monta a cena 63.
pub(crate) fn build_composition(world: &mut World) {
    world.spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 16.0,
                half_y: 0.3,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [32.0, 0.6], GROUND),
        Transform::from_translation(Vec2::new(0.0, -0.3)),
    ));
    windlass_tackle(world, "Geared", -3.0, true, GEARED);
    windlass_tackle(world, "Plain", 3.0, false, PLAIN);
}

#[cfg(test)]
#[path = "physics_smoke_pulley_comp_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 63 (W-Pulley W5).** Duas máquinas com a MESMA carga e o MESMO
    /// contrapeso; a única diferença é o segundo diâmetro do tambor.
    pub(crate) fn physics_smoke_composition(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_composition(gfx.sim.world_mut());
        // O rig sobe a 10 m e a câmera padrão mostra até 5: sem isto, tudo acima
        // das cargas — inclusive os dois tambores e as alças deles — nasce fora
        // do quadro. Ver o doc de `CAMERA_CENTRE`.
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 63] A COMPOSICAO -- as duas vantagens MULTIPLICAM.\n  \
               A cena nasce PARADA e o contorno JA ESTA LIGADO -- B o ALTERNA, entao\n  \
               aperta-lo aqui o DESLIGA e as alcas somem junto. Faca o passo das ALCAS\n  \
               primeiro (o ultimo bloco); o PLAY vem por ultimo (o toggle Physics ja\n  \
               esta armado).\n\n  \
               Os DOIS sarilhos tem a MESMA carga ({load:.0} kg) e o MESMO contrapeso\n  \
               ({counter:.0} kg), e os DOIS tem uma cadernal MOVEL pendurada na carga.\n  \
               A unica diferenca e o SEGUNDO diametro do tambor -- e eles andam para\n  \
               lados OPOSTOS.\n\n  \
               1. VERDE (esquerda) -- o tambor tem DOIS raios ({r_in:.2} e {r_out:.3}), entao\n     \
                  ele sozinho vale {gear:.0}x; a cadernal movel dobra de novo. Vantagem {mech:.0}x:\n     \
                  1 kg segura ate {hold:.0} kg, e os {load:.0} kg SOBEM (+{geared:.2} m em 2 s).\n     \
                  Olhe o CONTRAPESO: ele desce {drop:.2} m no mesmo tempo -- {mech:.0}x o que a\n     \
                  carga sobe. E essa razao que se ve de longe.\n  \
               2. VERMELHO (direita) -- o mesmo rig com um tambor de UM raio. Sobra a\n     \
                  talha: vantagem 2, e 1 kg so segura 2 kg. Os {load:.0} kg CAEM ({plain:.2} m)\n     \
                  ate o chao.\n\n  \
               (!) A carga de {load:.0} kg esta ENTRE as duas vantagens de proposito: pesada\n     \
               demais para a talha sozinha, leve demais para a composicao. E por isso\n     \
               que a MESMA massa sobe de um lado e cai do outro.\n  \
               (!) O contrapeso VERMELHO e arremessado para cima quando a carga pousa, e\n     \
               isso e a corda sendo corda: ela so PUXA. Um contrapeso de 1 kg puxado\n     \
               por 7 kg sai voando, e subir afrouxa a corda em vez de a esticar.\n\n  \
               AUTORE VOCE MESMO: selecione 'Plain Rope Drum' na Hierarquia e digite\n  \
               {r_out:.3} na row 'Out Radius (m)' da secao Pulley Wheel. O segundo anel\n  \
               aparece e, no Play, a carga da direita para de cair.\n\n  \
               E AGORA SEM DIGITAR NADA (W6) -- a cena nasce PARADA, e e por isso:\n     \
               as alcas de roldana so existem em REPOUSO (tocando, o overlay desenha a\n     \
               geometria do SOLVER, e elas autoram a AUTORADA). Com B ligado:\n  \
               - selecione 'Geared Rope Drum': ele tem TRES alcas ambar. O centro o\n    \
                 move; o aro da DIREITA e o raio de entrada; o da ESQUERDA e o de\n    \
                 SAIDA -- e so ele existe porque este tambor tem um segundo raio.\n    \
                 Arraste o da esquerda para FORA: a vantagem cai (2R/r encolhe) e no\n    \
                 Play a carga sobe menos. Puxe-o para DENTRO e ela sobe mais.\n  \
               - selecione 'Geared Rope Sheave' (a cadernal montada na carga) e arraste\n    \
                 o CENTRO dela: o eixo se re-coloca NO BLOCO e FICA. Antes desta wave\n    \
                 ele voltava sozinho ao soltar -- o centro de uma roldana montada e\n    \
                 derivado, e quem nao desarma o sentinela escreve num campo que o\n    \
                 frame seguinte reescreve.\n\n  \
               E O PISO -- dois gestos que NAO podem dar tranco:\n  \
               - com a corda 'Geared Rope' selecionada, digite 2 na row\n    \
                 'Rope Length (m)' da secao Physics Joint. A corda desta cena mede\n    \
                 {l0:.1} m, entao 2 e impossivel: o numero volta para {l0:.1} sozinho\n    \
                 (a corda nao pode ser mais curta que o caminho que ela enfia) e o\n    \
                 Play continua liso. Sem o piso isso era {viol:.0} m de violacao, e o\n    \
                 solver a comia num tique so.\n  \
               - clique 'Add Wheel': nasce uma 3a roldana e o comprimento sobe\n    \
                 sozinho ({l0:.4} -> {l0_added:.4} m). Nesta cena a roldana nova cai\n    \
                 quase sobre a linha da corda, entao o Play muda pouco -- o que se\n    \
                 confere aqui e que ele nao TRANCA (maior salto {jump:.4} m, o mesmo\n    \
                 da cena intocada).",
            load = LOAD_MASS,
            counter = COUNTER_MASS,
            r_in = R_IN,
            r_out = R_OUT,
            gear = R_IN / R_OUT,
            mech = 2.0 * R_IN / R_OUT,
            hold = COUNTER_MASS * 2.0 * R_IN / R_OUT,
            geared = MEASURED_GEARED_RISE,
            drop = MEASURED_GEARED_COUNTER_DROP,
            plain = MEASURED_PLAIN_DROP,
            l0 = MEASURED_GEARED_ROPE_LENGTH,
            l0_added = MEASURED_GEARED_ROPE_LENGTH_ADDED,
            jump = MEASURED_GEARED_WORST_JUMP,
            viol = MEASURED_GEARED_ROPE_LENGTH - 2.0,
        );
    }
}
