//! **Cena de ZONA rotacional** (`PH2D_PHYSICS_SMOKE=32`) — a mesa giratória: uma área
//! que GIRA o que está dentro dela (W-AreaTorque).
//!
//! Arquivo próprio para a família das ZONAS (a metade rotacional do campo de força), com
//! espaço para as próximas — o falloff e o frame da zona. As cenas de força linear vivem
//! em [`crate::physics_smoke_collision`] (=24) e [`crate::physics_smoke_contacts`]
//! (=26/27/28); esta é a irmã do torque.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{AreaTorque, BodyKind, Collider, ColliderShape, GravityScale, RigidBody};
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
}
