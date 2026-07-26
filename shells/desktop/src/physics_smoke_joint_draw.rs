//! **A cena de CRIAR APONTANDO** (`PH2D_PHYSICS_SMOKE=46`, W-J4).
//!
//! As cenas 43-45 mostraram o que um joint DIZ, o que se AGARRA nele e o que se
//! POSA. Esta é sobre o começo: como ele nasce, e **onde as âncoras nascem**.
//!
//! Duas rotas, e as duas ficam:
//!
//! - **DESENHAR** (nova): aponte um corpo, arraste, solte noutro. As âncoras
//!   nascem NOS dois pontos, e uma mola/corda ganha de brinde o comprimento que
//!   o arrasto mediu.
//! - **A SELEÇÃO** (a do W3): marque 2 corpos e aperte o botão — não há pontos a
//!   oferecer, então a política de semeadura decide (o centro do corpo B numa
//!   corda). Com 3 ou mais ela vira uma **CORRENTE** de N−1 joints, que é a razão
//!   de ela sobreviver: sete elos à mão são sete gestos; marcá-los é um.
//!
//! Nenhuma escola da pesquisa (Unity/Unreal/Godot/Fyrox/RUBE/Algodoo/Newton) tem
//! as DUAS; nós temos.
//!
//! Os números abaixo saíram de uma sonda headless sobre esta mesma armação,
//! rodada ANTES desta mensagem ser escrita.

use crate::physics_smoke::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

impl crate::App {
    /// **Cena 46 (W-J4).** Criar joints apontando, e a corrente. PAUSADA.
    pub(crate) fn physics_smoke_joint_draw(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        let mut body = |name: &str,
                        kind: BodyKind,
                        shape: ColliderShape,
                        size: [f32; 2],
                        rgba: [f32; 4],
                        at: [f32; 2]| {
            world.spawn((
                Transform::from_translation(Vec2::new(at[0], at[1])),
                Sprite::atlas(WHITE_TILE_KEY, size, rgba),
                Name::new(name.to_string()),
                RigidBody { kind },
                Collider {
                    shape,
                    ..Collider::default()
                },
            ));
        };
        let post = ColliderShape::Cuboid {
            half_x: 0.15,
            half_y: 0.15,
        };
        let grey = [0.75, 0.75, 0.8, 1.0];

        // ── ESQUERDA: o gesto. Um poste e uma prancha LONGA, sem joint nenhum —
        // a prancha é longa de propósito: a diferença entre amarrá-la pela PONTA
        // e pelo centro é 0,8 m, visível sem zoom.
        body(
            "Post",
            BodyKind::Static,
            post,
            [0.3, 0.3],
            grey,
            [-4.0, 6.0],
        );
        body(
            "Plank",
            BodyKind::Dynamic,
            ColliderShape::Cuboid {
                half_x: 0.8,
                half_y: 0.12,
            },
            [1.6, 0.24],
            [0.95, 0.6, 0.2, 1.0],
            [-2.6, 6.0],
        );

        // ── DIREITA: a corrente. Um gancho estático + 3 elos, nenhum joint.
        body("Hook", BodyKind::Static, post, [0.3, 0.3], grey, [3.0, 7.0]);
        for i in 1..=3 {
            body(
                &format!("Link{i}"),
                BodyKind::Dynamic,
                ColliderShape::Ball { radius: 0.2 },
                [0.4, 0.4],
                [0.4, 0.8, 0.95, 1.0],
                [3.0 + i as f32 * 0.7, 7.0],
            );
        }

        eprintln!(
            "[physics-smoke 46] CRIAR APONTANDO, e a CORRENTE (W-J4).\n\
             PAUSADA. NENHUM joint na cena -- os dois voce cria.\n  \
               1. Clique a prancha laranja. Na secao Physics Body ha AGORA dois\n     \
                  botoes: 'Draw Joint on Canvas' (sempre) e 'Join Selected Bodies'\n     \
                  (so com 2+ corpos marcados). Escolha o tipo em 'Join As' -- ele\n     \
                  vale para as DUAS rotas. Ponha em Rope.\n  \
               2. Aperte 'Draw Joint on Canvas'. Agora PRESSIONE sobre o poste\n     \
                  cinza da esquerda e ARRASTE ate a PONTA DIREITA da prancha, e\n     \
                  solte. Durante o arrasto uma banda ambar TRACEJADA acompanha o\n     \
                  cursor (tracejada porque o joint ainda nao existe) e um anel\n     \
                  marca de onde ela saiu.\n  \
               3. De Play -- medido: amarrada pela PONTA a prancha assenta em\n     \
                  (-3,748, 4,226) e roda 104,2 graus, pendurada pela ponta.\n     \
                  Rebobine, apague o joint (selecione 'Joint' na Hierarquia +\n     \
                  Delete) e refaca pelo BOTAO (marque poste + prancha, 'Join\n     \
                  Selected Bodies'): medido, ela assenta NIVELADA em (-3,034,\n     \
                  5,036), rot 0,0 graus -- a semeadura poe a ponta B no CENTRO do\n     \
                  corpo, porque a seleçao nao tem ponto nenhum a oferecer.\n     \
                  ESSA e a diferenca entre as duas rotas, num numero.\n  \
               4. A corda tambem ficou do TAMANHO do gesto: o Max Length no \u{a7}12 e\n     \
                  a distancia que voce arrastou -- um numero que ninguem digitou.\n  \
               5. Solte no VAZIO de proposito: um toast explica, nada e criado, e o\n     \
                  gesto SEGUE ARMADO para a proxima tentativa (soltar no mundo nao\n     \
                  cria um pino-no-mundo: isso e outra coisa). Solte no MESMO corpo\n     \
                  em que apertou: outro toast, mesma coisa.\n  \
               6. A CORRENTE: marque o gancho e os 3 elos ciano na ordem (clique o\n     \
                  gancho, Ctrl+clique os elos). O botao passa a dizer 'Chain 4\n     \
                  Selected Bodies' -- ele CONTA. Aperte: medido, 4 corpos viram 3\n     \
                  joints, e um Play deixa os elos em (2,661, 6,276) / (2,192,\n     \
                  5,111) / (1,224, 4,322), pendurados em cadeia.\n  \
               7. Ctrl+Z desfaz a corrente INTEIRA num passo (os 3 spawns caem no\n     \
                  mesmo frame, e o undo global e por diff de fim de frame).\n\
             ONDE AS ANCORAS NASCEM e a frase inteira desta wave: pela selecao a\n\
             politica decide; pelo gesto sao os seus dois pontos."
        );
    }
}
