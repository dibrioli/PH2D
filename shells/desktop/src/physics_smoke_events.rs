//! **Cena de TRANSIÇÃO** (`PH2D_PHYSICS_SMOKE=29`) — o instante em que dois corpos
//! se tocam, e o instante em que se largam.
//!
//! Arquivo próprio, e não mais um braço em [`crate::physics_smoke_contacts`]: aquele
//! mostra o ESTADO permanente (quem está encostado, sob que carga) e este mostra a
//! TRANSIÇÃO, que é a pergunta oposta — e o arquivo de lá estava em 492 das suas 600.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::physics_smoke::spawn_floor;

impl crate::App {
    /// **Cena 29 (W-ContactEvents).** Três leituras da mesma coisa, lado a lado.
    ///
    /// **ESQUERDA — a bola quicando (`restitution` 0,75).** Cada pouso acende um `×`
    /// que se abre e some; entre um pouso e outro a cruz `+` desaparece por completo,
    /// porque o par de fato deixou de existir. É o par *começou / terminou / começou*
    /// que um consumidor de gameplay viraria em dois sons de impacto.
    ///
    /// **MEIO — a caixa morta (`restitution` 0).** Cai, toca UMA vez e fica. O `×`
    /// vive uns poucos ticks e morre; a cruz `+` fica para sempre. É o contraste que
    /// mostra que o flash é o EVENTO e a cruz é o ESTADO — duas coisas diferentes
    /// desenhadas no mesmo lugar.
    ///
    /// **DIREITA — a pilha autorada já encostada.** Ela pisca **uma vez**, no primeiro
    /// tick simulado, e depois nunca mais. Isso é deliberado e é a leitura da Unity
    /// (`OnCollisionEnter` dispara no primeiro `FixedUpdate` mesmo para corpos que
    /// já nasceram se tocando): a narrow phase nunca tinha rodado, então não existe
    /// verdade anterior contra a qual chamar aquele contato de "velho".
    ///
    /// ⚠️ **E as duas coisas que a cena pede que você FAÇA, porque são o desenho da
    /// wave e não se veem sozinhas:**
    ///
    /// 1. **Arraste a régua para TRÁS.** As cruzes mudam para o que toca naquele tick
    ///    — e **nada pisca**. Um scrub não atravessou transição nenhuma; anunciá-las
    ///    seria chamar de colisão um movimento do relógio.
    /// 2. **Desmarque o `Physics` na barra de transporte.** As cruzes **somem** — sem
    ///    solver não há toque vivo, e uma cruz parada descreveria um mundo que você
    ///    pode agora desmontar com a mão. Marque de novo: elas voltam **sem piscar**.
    ///
    /// Toca de imediato. **B** liga/desliga o overlay inteiro.
    pub(crate) fn physics_smoke_events(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // ── ESQUERDA: a bola que quica ──────────────────────────────────────
        // ⚠️ Largada de **1,2**, e a altura foi MEDIDA, não escolhida por estética:
        // acima de ~2,0 o primeiro quique é INVISÍVEL para este canal. O impacto
        // rápido é resolvido e separado dentro do mesmo `step` (medido: a bola desce
        // até y = -0,478 e volta, e o toque acontece em -0,5), então nos dois
        // instantes em que o canal amostra o par não está encostado. Uma cena que
        // largasse de 3,4 ensinaria que a feature falha. Ver `ContactEvent`.
        world.spawn((
            Transform::from_translation(Vec2::new(-2.6, 1.2)),
            Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.6], [0.35, 0.85, 0.55, 1.0]),
            Name::new("Bouncy"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                restitution: 0.75,
                ..Collider::default()
            },
        ));

        // ── MEIO: a caixa que toca uma vez e fica ───────────────────────────
        world.spawn((
            Transform::from_translation(Vec2::new(0.0, 3.4)),
            Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], [0.95, 0.70, 0.30, 1.0]),
            Name::new("Dead drop"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.25,
                    half_y: 0.25,
                },
                restitution: 0.0,
                friction: 0.6,
                ..Collider::default()
            },
        ));

        // ── DIREITA: a pilha autorada JÁ encostada ──────────────────────────
        // Empilhada exatamente (0,5 de altura, sem vão): o ponto da coluna é que
        // esses contatos existem antes do primeiro passo.
        for i in 0..3 {
            world.spawn((
                Transform::from_translation(Vec2::new(2.6, -0.55 + i as f32 * 0.5)),
                Sprite::atlas(
                    WHITE_TILE_KEY,
                    [0.5, 0.5],
                    [0.55, 0.60, 0.95 - 0.12 * i as f32, 1.0],
                ),
                Name::new(format!("Resting {i}")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.25,
                        half_y: 0.25,
                    },
                    friction: 0.6,
                    ..Collider::default()
                },
            ));
        }

        eprintln!(
            "[physics-smoke 29] Tocando. A cruz BRANCA e o ESTADO (quem toca, sob que carga); o \
             X que abre e some e o EVENTO (comecou agora). Numeros MEDIDOS pela sonda headless \
             (probe_scene_29), 4 s a 60 Hz. ESQUERDA, a bola quicando: 7 transicoes, a primeira \
             no tick 36 -- comecou/terminou alternando (36/37, 63/64, 73/74) ate assentar no 76, \
             e entre um quique e outro a cruz SOME porque o par deixou de existir. MEIO, a caixa \
             morta: UMA transicao, no tick 55, e mais nenhuma -- o X morre em 6 ticks e a cruz \
             fica para sempre (o flash e o evento, a cruz e o estado). DIREITA, a pilha autorada \
             JA encostada: 3 transicoes, TODAS no tick 1, e depois silencio absoluto -- a leitura \
             da Unity, porque antes do primeiro passo a narrow phase nunca tinha rodado e nao \
             existia verdade anterior. AGORA FACA AS DUAS COISAS QUE NAO SE VEEM SOZINHAS: (1) \
             arraste a regua para TRAS -- as cruzes mudam para o que toca naquele tick e NADA \
             pisca, porque um scrub nao atravessou transicao nenhuma; (2) desmarque Physics na \
             barra -- as cruzes SOMEM (sem solver nao ha toque vivo), e ao remarcar elas voltam \
             SEM piscar. B liga o contorno. LIMITE HONESTO, medido: um impacto rapido (acima de \
             ~2 m de queda, ~7 m/s) nao produz evento nenhum -- o solver resolve e separa DENTRO \
             do mesmo passo, entao nos dois instantes em que o canal amostra o par nao esta \
             encostado. E o mesmo mecanismo do pico de impulso, e teria a mesma cura."
        );
    }
}
