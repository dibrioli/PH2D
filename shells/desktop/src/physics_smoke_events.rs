//! **Cena de TRANSIÇÃO** (`PH2D_PHYSICS_SMOKE=29`) — o instante em que dois corpos
//! se tocam, e o instante em que se largam.
//!
//! Arquivo próprio, e não mais um braço em [`crate::physics_smoke_contacts`]: aquele
//! mostra o ESTADO permanente (quem está encostado, sob que carga) e este mostra a
//! TRANSIÇÃO, que é a pergunta oposta — e o arquivo de lá estava em 492 das suas 600.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{
    BodyKind, Ccd, Collider, ColliderShape, InitialVelocity, MassOverride, RigidBody,
};
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
        // Largada de **1,2** — um quique calmo para a primeira leitura. (Até a wave
        // W-TickContacts a altura era FORÇADA aqui: acima de ~2 m o primeiro quique
        // era invisível, resolvido e separado dentro do mesmo `step`. Isso FOI
        // corrigido — a cena 31 larga de bem mais alto e o impacto rápido acende.)
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
                                    Abre, some em ~0,1 s, e nao volta. O TAMANHO dela\n\
                                    e a FORCA do impacto (cena 30 mostra isso).\n\
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
             ---- O QUE ERA LIMITE E DEIXOU DE SER (W-TickContacts) ---------------\n\
             \n\
             Antes desta wave, uma queda ALTA de um corpo QUE QUICA nao gerava evento\n\
             nenhum: o motor resolvia o pouso e ja separava dentro do MESMO passo, entao\n\
             o canal, que so olhava o fim do passo, nunca via o toque. Agora o diff roda\n\
             por TICK sobre a uniao dos sub-passos, entao o toque rapido acende. A CENA\n\
             31 mostra isso: uma bola largada de 8 m, cujos pousos eram invisiveis.\n\
             \n\
             (B liga/desliga o contorno dos colliders.)\n\
             ======================================================================\n"
        );
    }

    /// **Cena 30 (W-ImpactForce) — a DEMOLIÇÃO.** Duas raias IGUAIS, cada uma uma
    /// TORRE de caixas leves e uma bola pesada lançada contra ela — só a VELOCIDADE
    /// muda. Em cima, um empurrão lento: a torre balança, o `×` é pequeno. Embaixo, um
    /// tiro rápido: a torre EXPLODE, e o `×` no impacto é enorme.
    ///
    /// É o que faltava na cena da bola no chão: bater num chão imóvel só mostra o `×`
    /// abstrato. Aqui a força do impacto tem uma CONSEQUÊNCIA visível — quanto mais
    /// forte o tiro, mais longe as caixas voam, e o `×` cresce junto. Medido (probe
    /// `probe_scene_30`, bola pesada num alvo leve): impacto **1,4** N·s a 6 m/s,
    /// **4,5** N·s a 16 m/s.
    ///
    /// ⚠️ A CARGA (o `+`) não veria isso: depois que a poeira assenta, cada caixa parada
    /// pesa o mesmo peso, tenha sido empurrada de leve ou pulverizada. O que distingue os
    /// dois tiros é o PICO — o `×` — que vive dentro dos sub-passos e some quando o passo
    /// termina, e que esta wave captura.
    pub(crate) fn physics_smoke_impact_demolition(&mut self) {
        // Uma raia: um chão, uma torre de caixas leves em `tower_x`, e uma bola pesada em
        // `-tower_x` disparada para a direita a `vx`. `fast` liga o CCD (a bola rápida não
        // atravessa a caixa fina entre dois passos).
        let hue_ball = [0.95, 0.55, 0.35, 1.0];
        let lane = |world: &mut ph2d_ecs::World, floor_y: f32, vx: f32, fast: bool| {
            let top = floor_y + 0.2;
            world.spawn((
                Transform::from_translation(Vec2::new(0.0, floor_y)),
                Sprite::atlas(WHITE_TILE_KEY, [16.0, 0.4], [0.30, 0.32, 0.38, 1.0]),
                Name::new("Floor"),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 8.0,
                        half_y: 0.2,
                    },
                    ..Collider::default()
                },
            ));
            // A torre: 5 caixas leves empilhadas.
            for row in 0..5 {
                let hue = [0.50, 0.62 + 0.05 * row as f32, 0.85, 1.0];
                world.spawn((
                    Transform::from_translation(Vec2::new(4.0, top + 0.25 + row as f32 * 0.5)),
                    Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], hue),
                    Name::new(format!("Brick {row}")),
                    RigidBody {
                        kind: BodyKind::Dynamic,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: 0.25,
                            half_y: 0.25,
                        },
                        friction: 0.4,
                        ..Collider::default()
                    },
                ));
            }
            // A bola pesada, lançada contra a torre.
            let ball = (
                Transform::from_translation(Vec2::new(-5.0, top + 0.35)),
                Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.7], hue_ball),
                Name::new("Wrecker"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.35 },
                    ..Collider::default()
                },
                InitialVelocity {
                    linvel: [vx, 0.0],
                    angvel: 0.0,
                },
                MassOverride(6.0),
            );
            if fast {
                world.spawn((ball, Ccd));
            } else {
                world.spawn(ball);
            }
        };

        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        lane(world, 2.0, 5.0, false); // EM CIMA: empurrao lento
        lane(world, -3.0, 16.0, true); // EMBAIXO: tiro rapido

        eprintln!(
            "\n\
             ============== [physics-smoke 30] A DEMOLICAO =====================\n\
             \n\
             Duas raias IGUAIS -- uma torre de caixas leves e uma bola pesada. So\n\
             a VELOCIDADE da bola muda.\n\
             \n\
             EM CIMA  (lento, 5 m/s)   ... a torre BALANCA, o  x  do impacto e' pequeno.\n\
             EMBAIXO  (rapido, 16 m/s) ... a torre EXPLODE, e o  x  e' ENORME.\n\
             \n\
             >>> O TAMANHO do  x  e a FORCA do impacto -- e ela tem CONSEQUENCIA:\n\
                 quanto mais forte o tiro, mais longe as caixas voam. <<<\n\
             \n\
             Medido (impact da bola no 1o tijolo, N.s):  ~1,4 a 6 m/s   ~4,5 a 16 m/s\n\
             \n\
             ---- POR QUE A CARGA NAO BASTA -------------------------------------\n\
             \n\
             Depois que a poeira assenta, cada caixa PARADA pesa o mesmo -- tenha sido\n\
             empurrada de leve ou pulverizada. A CARGA (a cruz  +  em pe) nao distingue\n\
             os dois tiros. So o PICO (o  x ) distingue, e o pico vive DENTRO dos\n\
             sub-passos e some quando o passo termina. Capturar esse pico e' a wave.\n\
             E' o que um som de impacto ou um dano querem: quao forte foi o toque.\n\
             \n\
             (Tudo em UM frame -- tecla L abre a timeline para dar scrub e rever os\n\
             impactos. B liga/desliga o contorno dos colliders.)\n\
             ===================================================================\n"
        );
    }

    /// **Cena 31 (W-TickContacts) — o impacto RÁPIDO acende.** Duas bolas que quicam,
    /// lado a lado, na mesma restituição — só a ALTURA muda.
    ///
    /// **ESQUERDA — a bola BAIXA (larga de 1,2 m).** Cada pouso é lento o bastante para
    /// o par ainda estar encostado no fim do tick, então este pouso SEMPRE acendeu um
    /// `×` — é o controle.
    ///
    /// **DIREITA — a bola ALTA (larga de 8 m).** Cada pouso é tão rápido que o motor
    /// resolve e já separa dentro de um único tick: no fim do passo a bola já subiu.
    /// Até esta wave, o canal só olhava o fim do passo, então esses pousos eram
    /// **INVISÍVEIS** — a bola quicava alto e não acendia `×` nenhum, e quanto mais
    /// forte o pouso, mais invisível. Agora o diff roda por TICK sobre a união dos
    /// sub-passos, então **todo pouso acende** — e o `×` é maior, porque o impacto é
    /// maior (medido `probe_fast_bounces`: pico ~1,6 N·s na baixa, ~4,8 N·s na alta).
    ///
    /// É a mesma máquina do pico de impacto (cena 30): o pico vive entre os sub-passos.
    /// A cena 30 mostrou a FORÇA de um toque que já era reportado; esta mostra um toque
    /// que **não era reportado de jeito nenhum** passando a existir.
    pub(crate) fn physics_smoke_fast_impact(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // Uma bola que quica em `x`, largada de `y`. A física é IDÊNTICA nas duas — só a
        // altura difere, e é a altura que torna o pouso rápido ou lento.
        let mut bouncy = |x: f32, y: f32, hue: [f32; 4], name: &str| {
            world.spawn((
                Transform::from_translation(Vec2::new(x, y)),
                Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.6], hue),
                Name::new(name.to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.3 },
                    restitution: 0.75,
                    ..Collider::default()
                },
            ));
        };
        bouncy(-2.5, 1.2, [0.35, 0.85, 0.55, 1.0], "Low bounce"); // controle
        bouncy(2.5, 8.0, [0.95, 0.55, 0.35, 1.0], "High bounce"); // a wave

        eprintln!(
            "\n\
             ============ [physics-smoke 31] O IMPACTO RAPIDO ACENDE ============\n\
             \n\
             Duas bolas que quicam, mesma restituicao. So a ALTURA muda.\n\
             \n\
             ESQUERDA  bola BAIXA (larga de 1,2 m) ... pouso LENTO. O par ainda esta\n\
                       encostado no fim do tick, entao este pouso SEMPRE acendeu um  x .\n\
                       E' o controle.\n\
             DIREITA   bola ALTA  (larga de 8 m)   ... pouso RAPIDO. O motor resolve e\n\
                       ja separa a bola dentro de UM tick -- no fim do passo ela ja subiu.\n\
             \n\
             >>> Ate esta wave, os pousos da bola ALTA eram INVISIVEIS: ela quicava\n\
                 alto e nao acendia  x  nenhum, e quanto mais forte o pouso, mais\n\
                 invisivel. Agora TODO pouso acende -- e o  x  da alta e' MAIOR, porque\n\
                 o impacto e' maior. <<<\n\
             \n\
             Medido (pico do 1o pouso):  ~1,6 N.s na baixa   ~4,8 N.s na alta\n\
             \n\
             ---- POR QUE ERA INVISIVEL ----------------------------------------\n\
             \n\
             O diff antigo olhava so o FIM do passo. Um pouso rapido comeca e termina\n\
             DENTRO do passo, entao nos dois instantes em que o canal olhava a bola nao\n\
             estava encostada. Agora o diff roda por TICK sobre a UNIAO dos sub-passos\n\
             -- a mesma leitura que captura o pico de impacto (cena 30). O unico toque\n\
             que ainda escapa e' o que comeca e termina no MESMO sub-passo, que o solver\n\
             discreto nem produz (isso seria um tunel, e e' trabalho do CCD).\n\
             \n\
             (Tudo em UM frame. L abre a timeline; B liga/desliga o contorno.)\n\
             ===================================================================\n"
        );
    }
}
