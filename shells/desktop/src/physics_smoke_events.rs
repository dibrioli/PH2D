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
            "\n\
             ================ [physics-smoke 29] EVENTOS DE CONTATO ================\n\
             \n\
             DUAS MARCAS BRANCAS, E ELAS DIZEM COISAS DIFERENTES:\n\
             \n\
             cruz  +  (em pe)     = ESTADO. Estes dois estao se tocando AGORA.\n\
                                    O TAMANHO dela e a CARGA (peso que o par aguenta).\n\
             cruz  x  (deitada)   = EVENTO. Estes dois COMECARAM a se tocar agora.\n\
                                    Abre, some em ~0,1 s, e nao volta.\n\
             \n\
             E ISSO QUE A WAVE ADICIONOU: antes o motor so sabia dizer 'estao se\n\
             tocando'. Agora ele sabe dizer 'comecaram AGORA' e 'pararam AGORA' --\n\
             que e o que um jogo precisa para tocar um som de impacto ou tirar vida.\n\
             \n\
             ---- O QUE OLHAR NA TELA (3 colunas) ---------------------------------\n\
             \n\
             ESQUERDA  bola quicando ... a cada pouso: um x pisca. Entre um quique\n\
                                         e outro a cruz + SOME (nao estao mais se\n\
                                         tocando). Medido: 7 transicoes.\n\
             MEIO      caixa que nao quica ... pisca UMA vez e para. A cruz + fica\n\
                                         para sempre. Este e o contraste que importa:\n\
                                         o x e o EVENTO, a cruz e o ESTADO.\n\
             DIREITA   pilha ja empilhada ... pisca UMA vez, no primeiro instante,\n\
                                         e nunca mais. E de proposito (o motor nunca\n\
                                         tinha olhado essa cena antes).\n\
             \n\
             ---- AS 2 COISAS QUE VOCE PRECISA FAZER (o resto e so olhar) ---------\n\
             \n\
             >>> Aperte L para abrir a TIMELINE embaixo. As duas ficam la. <<<\n\
             \n\
             (1) ARRASTE A REGUA DA TIMELINE PARA TRAS.\n\
                 ESPERADO: as cruzes + mudam (para o que toca naquele instante),\n\
                 e NENHUM x pisca.\n\
                 POR QUE IMPORTA: voltar no tempo nao e uma colisao. Sem isso, todo\n\
                 arrasto de regua dispararia dezenas de sons de impacto falsos.\n\
             \n\
             (2) DESMARQUE O BOTAO 'Physics' NA BARRA DA TIMELINE (perto de Loop).\n\
                 ESPERADO: as cruzes SOMEM. Remarcando, elas voltam SEM piscar.\n\
                 POR QUE IMPORTA: isto era um BUG ate esta wave -- com a fisica\n\
                 desligada as cruzes ficavam na tela, descrevendo toques num mundo\n\
                 que voce ja podia desmontar com a mao.\n\
             \n\
             ---- LIMITE CONHECIDO (medido, nao e defeito a reportar) -------------\n\
             \n\
             Queda ALTA (acima de ~2 m) nao gera evento: o motor resolve o pouso e\n\
             ja separa os corpos dentro do mesmo passo, entao nos dois instantes em\n\
             que o canal olha, eles nao estao encostados. Por isso a bola aqui cai\n\
             de perto. Consertar isso e a mesma tarefa de medir a forca de impacto.\n\
             \n\
             (B liga/desliga o contorno dos colliders.)\n\
             ======================================================================\n"
        );
    }
}
