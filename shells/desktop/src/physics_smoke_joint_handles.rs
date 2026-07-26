//! **A cena das DUAS ALÇAS de âncora** (`PH2D_PHYSICS_SMOKE=44`, W-J2).
//!
//! A cena 43 mostrou o que um joint DIZ de si; esta mostra o que se pode FAZER
//! com ele no canvas. Até esta wave só a ponta A tinha alça: a âncora no corpo B
//! era o que a política de semeadura produzisse (o mesmo ponto num Pin/Weld, o
//! centro do corpo numa Spring/Rope) e nenhum gesto do editor a movia.
//!
//! Duas pistas, porque as duas alças se leem de formas diferentes:
//!
//! - **esquerda (Rope):** as pontas nascem SEPARADAS, então as duas marcas ficam
//!   visíveis lado a lado — é a pista onde se aprende que existe uma segunda.
//! - **direita (Pin):** as pontas nascem no MESMO ponto (dois corpos num lugar só
//!   *é* o que um pino é), então as marcas ficam concêntricas: o disco cheio é o
//!   A, o anel em volta é o B. É a pista que ensina o vocabulário, e a única em
//!   que separar as duas produz o traço VERMELHO da W-J1 (a restrição que o
//!   solver ainda não impôs).
//!
//! ⚠️ **Duas joints, e a cena NÃO seleciona nenhuma** (W-J2b). É essa a metade
//! nova: as quatro alças aparecem sozinhas, porque uma joint não tem sprite e a
//! seleção era a única coisa que as trazia à tela — ou seja, elas só eram
//! alcançáveis depois de caçar a joint na Hierarquia.
//!
//! Os números abaixo saíram de uma sonda headless sobre esta mesma armação,
//! rodada ANTES desta mensagem ser escrita.

use crate::physics_smoke::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

impl crate::App {
    /// **Cena 44 (W-J2).** Duas alças por joint, PAUSADA.
    ///
    /// Pausada porque autorar âncora é gesto de repouso — e porque é isso que o
    /// produto faz: as alças nem são oferecidas com o relógio andando (durante o
    /// play o overlay desenha as âncoras do SOLVER, e uma alça que aceitasse
    /// arrasto contra um corpo balançando autoraria uma pose que ninguém
    /// escolheu).
    pub(crate) fn physics_smoke_joint_handles(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        let mut post = |name: &str, x: f32, y: f32| {
            world.spawn((
                Transform::from_translation(Vec2::new(x, y)),
                Sprite::atlas(WHITE_TILE_KEY, [0.3, 0.3], [0.75, 0.75, 0.8, 1.0]),
                Name::new(name.to_string()),
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
            ));
        };
        post("RopePost", -3.0, 6.0);
        post("PinPost", 3.0, 6.0);

        // ── ESQUERDA: a Rope, cujas pontas nascem separadas ──────────────────
        // A barra é LONGA (1,6 m) de propósito: a âncora semeada cai no CENTRO
        // dela, e arrastar o anel até a ponta é uma diferença de 0,8 m — visível
        // sem zoom, e que muda como a barra pendura.
        world.spawn((
            Transform::from_translation(Vec2::new(-2.0, 4.6)),
            Sprite::atlas(WHITE_TILE_KEY, [1.6, 0.24], [0.95, 0.6, 0.2, 1.0]),
            Name::new("RopeBar".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.8,
                    half_y: 0.12,
                },
                ..Collider::default()
            },
        ));
        world.spawn((
            Transform::from_translation(Vec2::new(-3.0, 6.0)),
            Name::new("Tie".to_string()),
            PhysicsJoint {
                body_a: stable_name_id("RopePost"),
                body_b: stable_name_id("RopeBar"),
                kind: JointKind::Rope,
                max_length: 1.6,
                ..PhysicsJoint::default()
            },
        ));

        // ── DIREITA: o Pin, cujas pontas nascem no mesmo ponto ───────────────
        world.spawn((
            Transform::from_translation(Vec2::new(3.8, 6.0)),
            Sprite::atlas(WHITE_TILE_KEY, [1.6, 0.24], [0.4, 0.8, 0.95, 1.0]),
            Name::new("PinBar".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.8,
                    half_y: 0.12,
                },
                ..Collider::default()
            },
        ));
        world.spawn((
            Transform::from_translation(Vec2::new(3.0, 6.0)),
            Name::new("Hinge".to_string()),
            PhysicsJoint {
                body_a: stable_name_id("PinPost"),
                body_b: stable_name_id("PinBar"),
                kind: JointKind::Pin,
                ..PhysicsJoint::default()
            },
        ));

        eprintln!(
            "[physics-smoke 44] AS DUAS ALCAS de ancora, em TODA joint (W-J2 + W-J2b).\n\
             PAUSADA, e NADA selecionado. Aperte B se o contorno nao estiver ligado.\n  \
               1. Sem clicar em nada, olhe o canvas: as QUATRO marcas ja estao la\n     \
                  (medido: 2 joints x 2 pontas = 4 alcas com selecao vazia; com o\n     \
                  relogio andando sao 0). Ambar, e a forma diz a ponta: DISCO CHEIO\n     \
                  = A, ANEL VAZADO = B -- a mesma gramatica das linhas de posse da\n     \
                  cena 43 (solida = A, tracejada = B). Estao MAIORES que no smoke\n     \
                  anterior (disco 6 -> 9 px de raio, anel 10 -> 15).\n  \
               2. Arraste o ANEL da esquerda (a Rope 'Tie', em -2,000/4,600 -- o\n     \
                  centro da barra) ate a PONTA DIREITA dela. Voce NAO precisou\n     \
                  selecionar nada na Hierarquia; e repare que o Inspector passou a\n     \
                  mostrar a secao Physics Joint: pegar a alca SELECIONA a joint.\n     \
                  De Play: a barra passa a pendurar PELA PONTA e nao pelo meio --\n     \
                  medido: amarrada no centro ela assenta NIVELADA em (-3,862, 4,652),\n     \
                  rot 0,0 graus; amarrada na ponta assenta em (-3,378, 4,320), rot\n     \
                  145,0 graus. Rebobine e arraste o anel de volta ao centro.\n  \
               3. Segure CTRL enquanto arrasta: o anel IMA nos pontos do collider --\n     \
                  centro, 4 quinas e 4 meios de aresta (9 no total para uma caixa,\n     \
                  os MESMOS nove que a alca de pivo ja oferece). Uma CRUZ marca o\n     \
                  ponto capturado; sem ela, um ima e indistinguivel de um arrasto\n     \
                  que parou de seguir o cursor. Solte o CTRL e ele se solta.\n  \
               4. Olhe o Pin da direita ('Hinge'): as duas marcas estao no MESMO\n     \
                  lugar -- medido, as duas em (3,000, 6,000), e e o que um pino e --\n     \
                  entao se desenham CONCENTRICAS: o disco no meio, o anel em volta.\n     \
                  O centro pega o A; a faixa fora dele pega o B (senao a ponta B de\n     \
                  todo Pin seria inalcancavel).\n  \
               5. Arraste o ANEL do Pin 0,5 m para a direita: aparece o traco\n     \
                  VERMELHO da cena 43 -- a restricao existe e ainda nao foi imposta\n     \
                  (medido: vao 0,00000 -> 0,50000 m). De Play: o solver MONTA os dois\n     \
                  corpos em DOIS ticks (a barra salta de x=3,800 para 3,300) e a marca\n     \
                  some. Rebobine: ela volta -- as ancoras AUTORADAS nao se moveram, e\n     \
                  o vermelho le as VIVAS. E o `connectedAnchor` do Unity.\n  \
               6. Selecione uma SPRITE qualquer e arraste o gizmo dela por cima de\n     \
                  uma alca de ancora: a alca continua pegavel (ela e registrada por\n     \
                  ULTIMO, entao ganha o pixel de quem estiver embaixo). Um painel\n     \
                  por cima, nao -- painel e desenhado depois de todo o canvas.\n  \
               7. Ctrl+Z desfaz um arrasto inteiro num passo so (o gesto todo, nao\n     \
                  um passo por frame de mouse).\n\
             O QUE MUDOU POR DENTRO: arrastar a ponta A NAO reseta mais a ponta B.\n\
             O sentinela `anchored` e do JOINT INTEIRO -- limpa-lo re-deriva as DUAS\n\
             ancoras da politica de semeadura -- entao, com a 2a alca no mundo, o\n\
             gesto antigo teria jogado fora a ancora que o artista acabara de por no\n\
             outro corpo, em silencio. Confira: faca o passo 2, depois arraste o DISCO\n\
             do 'Tie' pelo poste. O anel tem de ficar onde voce o deixou."
        );
    }
}
