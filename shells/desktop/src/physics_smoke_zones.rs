//! **Cena de ZONA rotacional** (`PH2D_PHYSICS_SMOKE=32`) — a mesa giratória: uma área
//! que GIRA o que está dentro dela (W-AreaTorque).
//!
//! Arquivo próprio para a família das ZONAS, hoje com quatro cenas: a mesa giratória (=32),
//! a autoria dela pela UI (=33), o frame da força (=34) e o falloff (=35). As de força linear vivem
//! em [`crate::physics_smoke_collision`] (=24) e [`crate::physics_smoke_contacts`]
//! (=26/27/28); esta é a irmã do torque.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{
    AreaEffector, AreaFalloff, AreaForceWorldAxes, AreaTorque, BodyKind, Collider, ColliderShape,
    GravityScale, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

impl crate::App {
    /// **Cena 32 (W-AreaTorque).** Quatro leituras da mesma coisa, lado a lado.
    ///
    /// Os corpos que giram flutuam (`GravityScale(0)`) — assim a rampa de ângulo vem
    /// INTEIRA do torque da zona, sem gravidade a atrapalhar, e cada um fica onde o
    /// artista o pôs. O `AreaEffector` empurra pelo centro de massa e não gira nada; ESTA
    /// é a metade que faz girar.
    ///
    /// **ESQUERDA — a caixa COMPACTA (1x1) num torque +1.** Gira depressa: ~171 graus no
    /// primeiro segundo, no sentido anti-horário (o glifo violeta mostra a mão).
    ///
    /// **MEIO — a BARRA COMPRIDA (4x0.25) no MESMO torque +1.** Mesma área, mesma massa,
    /// mas ~21 graus no mesmo segundo — **8x mais devagar**. É o MOMENTO DE INÉRCIA: a
    /// forma resiste ao giro como a massa resiste à translação. Um torque não é uma
    /// aceleração angular, e este par é a prova visual (medido: 171 contra 21, razão
    /// 8,03 — exatamente `I_barra / I_compacta`).
    ///
    /// **DIREITA — a caixa compacta num torque -1.** Gira no sentido OPOSTO (horário): o
    /// SINAL é a direção, e o glifo violeta aponta para o outro lado. Sem esse glifo uma
    /// zona de giro seria uma caixa magenta indistinguível de um sensor comum.
    ///
    /// **PONTA — a caixa de controle, sem zona.** Fica parada: o torque é local à área.
    ///
    /// ⚠️ **B** liga o contorno (e os glifos de giro). Deixe **Physics** MARCADO — esta
    /// cena É a simulação girando; é o oposto da cena 7, que pede para desmarcá-lo.
    pub(crate) fn physics_smoke_spin_zone(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // Uma zona de giro: um sensor estático com `AreaTorque`, mais um corpo que flutua
        // dentro dela. `half` é meio-lado; a barra de 4 m cabe folgado num sensor de 6 m.
        let mut spin_lane =
            |cx: f32, torque: f32, spinner: (f32, f32), tint: [f32; 4], tag: &str| {
                world.spawn((
                    Transform::from_translation(Vec2::new(cx, 0.0)),
                    Sprite::atlas(WHITE_TILE_KEY, [6.0, 6.0], [0.55, 0.4, 0.85, 0.16]),
                    Name::new(format!("{tag} zone")),
                    RigidBody {
                        kind: BodyKind::Static,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: 3.0,
                            half_y: 3.0,
                        },
                        is_sensor: true,
                        ..Collider::default()
                    },
                    AreaTorque(torque),
                ));
                world.spawn((
                    Transform::from_translation(Vec2::new(cx, 0.0)),
                    Sprite::atlas(WHITE_TILE_KEY, [spinner.0 * 2.0, spinner.1 * 2.0], tint),
                    Name::new(format!("{tag} spinner")),
                    RigidBody {
                        kind: BodyKind::Dynamic,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: spinner.0,
                            half_y: spinner.1,
                        },
                        ..Collider::default()
                    },
                    // Flutua: o giro vem só do torque, e o corpo fica onde foi posto.
                    GravityScale(0.0),
                ));
            };

        // ESQUERDA: compacta, +1 (rápida). MEIO: barra, +1 (lenta, 8x a inércia).
        spin_lane(-7.0, 1.0, (0.5, 0.5), [0.95, 0.80, 0.30, 1.0], "compact");
        spin_lane(0.0, 1.0, (2.0, 0.125), [0.55, 0.95, 0.60, 1.0], "bar");
        // DIREITA: compacta, -1 (gira ao contrário).
        spin_lane(7.0, -1.0, (0.5, 0.5), [0.95, 0.45, 0.30, 1.0], "reverse");

        // PONTA: a caixa de controle, sem zona nenhuma. Flutua e fica parada.
        world.spawn((
            Transform::from_translation(Vec2::new(13.0, 0.0)),
            Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [0.6, 0.6, 0.66, 1.0]),
            Name::new("control (no zone)"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            GravityScale(0.0),
        ));

        eprintln!(
            "[physics-smoke 32] Tocando. Quatro caixas flutuantes. ESQUERDA (amarela) e uma \
             1x1 num torque +1: gira depressa, ~171 graus no 1o segundo, anti-horario. MEIO \
             (verde) e uma BARRA 4x0.25 no MESMO torque: mesma area e mesma massa, mas so \
             ~21 graus -- 8x mais devagar, porque tem 8x o MOMENTO DE INERCIA (medido: 171 \
             contra 21, razao 8,03). Um torque e resistido pela FORMA como uma forca e \
             resistida pela massa. DIREITA (laranja) e a mesma compacta num torque -1: gira \
             para o OUTRO lado -- o sinal e a direcao, e o glifo violeta mostra a mao. A PONTA \
             (cinza) nao tem zona e fica parada. B liga o contorno e os glifos de giro; deixe \
             Physics MARCADO."
        );
    }

    /// **Cena 33 (W-AreaTorque, autoria pela UI).** O palco para AUTORAR a mesa giratória
    /// só com cliques — a resposta ao *"não vi nada disso agindo na UI"*.
    ///
    /// **ESQUERDA — a mesa a AUTORAR.** Um sprite pelado (sem física) com uma caixa
    /// flutuante dentro. Selecione o sprite e, no Inspector, seção **Physics Body**:
    /// 1. **Add Physics Body**
    /// 2. **Kind = Static** (a mesa não cai)
    /// 3. **Trigger = Sensor** ← *é este passo que faz a linha **Torque** aparecer* (junto
    ///    de Force/Drag/Fluid Density/Shape Drag — são todas sensor-only)
    /// 4. Digite um valor em **Torque (N·m)** (ex.: 1) — a caixa começa a girar. O SINAL é
    ///    o sentido (negativo = horário).
    ///
    /// **DIREITA — a mesa PRONTA.** Já autorada (Static + Sensor + Torque 1). No Play ela
    /// gira sozinha. Selecione o sprite dela: o Inspector mostra Kind=Static, Trigger=Sensor
    /// e a linha **Torque com o valor 1** — a prova de que a row **reflete** o que está no
    /// collider (antes desta wave as rows de área eram write-only e liam 0 ao re-selecionar).
    ///
    /// ⚠️ **B** liga o contorno e o glifo de giro violeta. Deixe **Physics** MARCADO.
    pub(crate) fn physics_smoke_author_spin(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // Um chão bem abaixo, só para dar referência visual (as caixas flutuam).
        world.spawn((
            Transform::from_translation(Vec2::new(0.0, -8.0)),
            Sprite::atlas(WHITE_TILE_KEY, [16.0, 0.5], [0.40, 0.42, 0.48, 1.0]),
            Name::new("Floor"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 8.0,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
        ));

        // Uma caixa 1x1 flutuante (o "disco" que a mesa gira). O giro vem só do torque.
        let disc = |world: &mut bevy_ecs::world::World, cx: f32, tag: &str| {
            world.spawn((
                Transform::from_translation(Vec2::new(cx, 0.0)),
                Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [0.95, 0.80, 0.30, 1.0]),
                Name::new(format!("{tag} disc")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.5,
                        half_y: 0.5,
                    },
                    ..Collider::default()
                },
                GravityScale(0.0),
            ));
        };

        // ESQUERDA: o sprite PELADO (sem RigidBody/Collider) que o artista vai autorar.
        world.spawn((
            Transform::from_translation(Vec2::new(-4.0, 0.0)),
            Sprite::atlas(WHITE_TILE_KEY, [4.0, 4.0], [0.55, 0.4, 0.85, 0.16]),
            Name::new("Turntable (author me)"),
        ));
        disc(world, -4.0, "left");

        // DIREITA: a mesa PRONTA -- Static Sensor + AreaTorque(1). Ao selecionar, o
        // Inspector mostra a linha Torque com o valor (o fix de sync desta wave).
        world.spawn((
            Transform::from_translation(Vec2::new(4.0, 0.0)),
            Sprite::atlas(WHITE_TILE_KEY, [4.0, 4.0], [0.4, 0.7, 0.55, 0.16]),
            Name::new("Turntable (working)"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 2.0,
                    half_y: 2.0,
                },
                is_sensor: true,
                ..Collider::default()
            },
            AreaTorque(1.0),
        ));
        disc(world, 4.0, "right");

        eprintln!(
            "[physics-smoke 33] AUTORIA PELA UI. ESQUERDA: um sprite PELADO com uma caixa \
             flutuante. Selecione o sprite -> Inspector > Physics Body: (1) Add Physics Body \
             (2) Kind=Static (3) Trigger=Sensor <- e AQUI a linha 'Torque (N*m)' aparece, com \
             Force/Drag/Fluid Density/Shape Drag (todas sensor-only) (4) digite Torque=1 e a \
             caixa comeca a girar. DIREITA: a mesma mesa JA autorada (Static+Sensor+Torque 1) \
             -- no Play ela gira sozinha, e ao selecionar o sprite dela o Inspector mostra a \
             linha Torque com o valor 1 (a prova de que a row reflete o collider). B liga o \
             contorno e o glifo de giro violeta; deixe Physics MARCADO. De Play."
        );
    }
    /// **Cena 34 (W-AreaFrame).** Duas faixas idênticas, um bit de diferença.
    ///
    /// A zona de força era autorada em eixos de MUNDO, então **girar o sensor não girava
    /// o vento** — uma esteira diagonal era inexprimível. Agora a força é autorada no
    /// frame da ZONA, e o toggle `Force Axes: Zone | World` prende a direção de volta ao
    /// mundo (o `useGlobalAngle` da Unity).
    ///
    /// As duas zonas têm a MESMA rotação (40°) e a MESMA força (0,9 N ao longo do próprio
    /// +X). A única diferença é o marcador `AreaForceWorldAxes` na da direita. Medido
    /// headless, 120 ticks (2 s), caixas de 0,7 m:
    ///
    /// | faixa | deslocamento | ângulo |
    /// |---|---|---|
    /// | **Zone** (esquerda) | `(2,81, 2,36)` | **40,0°** — o da zona, ao décimo |
    /// | **World** (direita) | `(3,67, 0,00)` | **0,0°** — o vento velho |
    ///
    /// ⚠️ As caixas flutuam (`GravityScale(0)`) pelo motivo da cena 32: assim a trajetória
    /// vem INTEIRA da zona e o ângulo é legível, sem gravidade a torcê-lo.
    ///
    /// **O gesto que fecha a quarta condição de UI:** selecione a zona da ESQUERDA e
    /// clique `World` na linha *Force Axes* — as caixas dela passam a andar na horizontal,
    /// como as da direita, e a SETA laranja do overlay gira junto (ela lê a mesma porta que
    /// o solver). Clique `Zone` e o diagonal volta.
    pub(crate) fn physics_smoke_force_frame(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        // 40 graus: nem eixo (onde seno e cosseno são triviais e um frame errado passaria
        // despercebido), nem 45 (onde as duas componentes são iguais e trocá-las não se vê).
        let rot = 40.0f32.to_radians();

        let mut lane = |cx: f32, world_axes: bool, tag: &str, tint: [f32; 4]| {
            let mut zone = world.spawn((
                Transform {
                    translation: Vec2::new(cx, 0.0),
                    rotation: rot,
                    scale: Vec2::new(1.0, 1.0),
                    skew_x: 0.0,
                    skew_y: 0.0,
                },
                Sprite::atlas(WHITE_TILE_KEY, [6.0, 6.0], tint),
                Name::new(format!("{tag} zone")),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 3.0,
                        half_y: 3.0,
                    },
                    is_sensor: true,
                    ..Collider::default()
                },
                AreaEffector { force: [0.9, 0.0] },
            ));
            if world_axes {
                zone.insert(AreaForceWorldAxes);
            }
            // Três caixas empilhadas na vertical: uma só diria "andou", três dizem "andaram
            // PARALELAS", que é o que uma direção de vento parece.
            for (i, dy) in [-1.6f32, 0.0, 1.6].iter().enumerate() {
                world.spawn((
                    Transform::from_translation(Vec2::new(cx - 1.5, *dy)),
                    Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.7], [0.95, 0.80, 0.30, 1.0]),
                    Name::new(format!("{tag} box {i}")),
                    RigidBody {
                        kind: BodyKind::Dynamic,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: 0.35,
                            half_y: 0.35,
                        },
                        ..Collider::default()
                    },
                    GravityScale(0.0),
                ));
            }
        };

        lane(-7.0, false, "Zone-frame", [0.4, 0.7, 0.55, 0.16]);
        lane(7.0, true, "World-axes", [0.55, 0.4, 0.85, 0.16]);

        eprintln!(
            "[physics-smoke 34] O FRAME DA ZONA. Duas zonas com a MESMA rotacao (40 graus) \
             e a MESMA forca (0,9 N no proprio +X); a da DIREITA carrega o marcador \
             World-axes. ESQUERDA (Zone): as caixas sobem na DIAGONAL, no angulo da zona -- \
             medido headless em 2s: deslocamento (2,81, 2,36) = 40,0 graus. DIREITA (World): \
             as mesmas caixas andam na HORIZONTAL -- (3,67, 0,00) = 0,0 graus, o vento velho. \
             AGORA O GESTO: selecione a zona da ESQUERDA, Inspector > Physics Body > linha \
             'Force Axes' -> clique World: as caixas dela passam a andar na horizontal e a \
             SETA laranja gira junto (ela le a mesma porta que o solver). Clique Zone e o \
             diagonal volta. B liga o contorno e a seta; deixe Physics MARCADO. De Play."
        );
    }

    /// **Cena 35 (W-AreaFalloff).** A fila que voa junta, e a fila que se abre em leque.
    ///
    /// Até aqui uma zona empurrava IGUAL em toda a sua extensão: encostado na parede ou no
    /// olho da rajada, o mesmo empurrão. O `Falloff` faz a força e o torque desvanecerem do
    /// centro para a borda, **chegando a zero exatamente na fronteira**, e a régua é a
    /// silhueta da própria zona — não há raio à parte para o artista manter em dia.
    ///
    /// As duas rajadas são redondas (raio 5), com a MESMA força (1,2 N em +X), e cada uma
    /// tem quatro caixas idênticas empilhadas na coluna do centro, a 0 / 1,4 / 2,8 / 4,2 m
    /// do olho — ou seja a `t` = 0 / 0,28 / 0,56 / 0,84 do caminho até a borda. Medido
    /// headless, 3 s, deslocamento em metros:
    ///
    /// | faixa | olho | 0,28 | 0,56 | 0,84 |
    /// |---|---|---|---|---|
    /// | **uniforme** (esquerda) | 10,01 | 9,95 | 9,70 | 8,96 |
    /// | **Falloff 1** (direita) | 7,64 | 6,43 | 4,35 | **1,71** |
    ///
    /// À esquerda a fila **voa junta** (a última fica um pouco atrás só porque sai da bola
    /// por uma corda mais curta); à direita ela se **abre em leque**, e a caixa mais externa
    /// anda 5× menos que a do olho.
    ///
    /// ⚠️ As caixas flutuam (`GravityScale(0)`) pelo motivo das cenas 32 e 34: assim a
    /// trajetória vem INTEIRA da zona.
    ///
    /// **O gesto:** selecione a zona da ESQUERDA e digite `1` na linha *Falloff* — o anel
    /// laranja apagado do meio caminho aparece no overlay (é a silhueta encolhida à metade,
    /// a curva de nível exata) e a fila dela passa a se abrir também.
    pub(crate) fn physics_smoke_falloff(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        // Redonda de propósito: numa rajada circular "longe do centro" se lê de bater o
        // olho, e o anel de meio caminho do overlay é um círculo concêntrico.
        const R: f32 = 5.0;

        let mut lane = |cx: f32, falloff: f32, tag: &str, tint: [f32; 4]| {
            let mut zone = world.spawn((
                Transform::from_translation(Vec2::new(cx, 0.0)),
                Sprite::atlas(WHITE_TILE_KEY, [R * 2.0, R * 2.0], tint),
                Name::new(format!("{tag} gust")),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: R },
                    is_sensor: true,
                    ..Collider::default()
                },
                AreaEffector { force: [1.2, 0.0] },
            ));
            if falloff > 0.0 {
                zone.insert(AreaFalloff(falloff));
            }
            // Quatro caixas na COLUNA DO CENTRO: começam todas a `x` do olho, então a
            // distância que as separa é puramente a vertical — e o leque é a única coisa
            // que pode abri-las.
            for (i, dy) in [0.0f32, 1.4, 2.8, 4.2].iter().enumerate() {
                world.spawn((
                    Transform::from_translation(Vec2::new(cx, *dy)),
                    Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.7], [0.95, 0.80, 0.30, 1.0]),
                    Name::new(format!("{tag} box {i}")),
                    RigidBody {
                        kind: BodyKind::Dynamic,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: 0.35,
                            half_y: 0.35,
                        },
                        ..Collider::default()
                    },
                    GravityScale(0.0),
                ));
            }
        };

        lane(-9.0, 0.0, "Uniform", [0.4, 0.7, 0.55, 0.16]);
        lane(9.0, 1.0, "Falloff", [0.55, 0.4, 0.85, 0.16]);

        eprintln!(
            "[physics-smoke 35] O FALLOFF DA ZONA. Duas rajadas redondas (raio 5) com a \
             MESMA forca (1,2 N em +X) e quatro caixas identicas cada, na coluna do centro, \
             a 0 / 1,4 / 2,8 / 4,2 m do olho. ESQUERDA (uniforme, como era ate hoje): a fila \
             VOA JUNTA -- medido em 3s, deslocamentos 10,01 / 9,95 / 9,70 / 8,96 m. DIREITA \
             (Falloff 1): a fila se ABRE EM LEQUE -- 7,64 / 6,43 / 4,35 / 1,71 m, a mais \
             externa andando 5x menos que a do olho, porque o empurrao chega a ZERO na borda. \
             AGORA O GESTO: selecione a rajada da ESQUERDA, Inspector > Physics Body > linha \
             'Falloff' -> digite 1: o anel laranja apagado do meio caminho aparece (a \
             silhueta encolhida a metade, que e a curva de nivel exata) e a fila dela passa a \
             se abrir tambem. B liga o contorno, a seta e o anel; deixe Physics MARCADO."
        );
    }
}
