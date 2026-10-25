//! As RECEITAS e as peças de montagem da arena ([`super`], `PH2D_VIDA_SMOKE=4`): os moldes da
//! Salamandra, do morcego, do coração e das balas, a fábrica e o relógio.

use super::*;

/// **A receita da Salamandra** — a da cena `=3`, com mais vida e o grito da vitória.
///
/// ⚠️ (W8) **O corpo é CINEMÁTICO, não estático**: parada é a mesma parede para os morcegos (a malha
/// conta o cinemático parado), e o tutorial 03 fá-la perseguir com UM gesto — *Add Component → Nav
/// Agent* — porque a semente do mover nunca rebaixa um `Static` que o artista pôs.
pub(super) fn receita_da_salamandra(world: &mut World) -> Entity {
    world
        .spawn((
            Name::new("Salamandra"),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(
                WHITE_TILE_KEY,
                [LADO_DA_SALAMANDRA, LADO_DA_SALAMANDRA],
                SALAMANDRA_RGBA,
            ),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: LADO_DA_SALAMANDRA / 2.0,
                    half_y: LADO_DA_SALAMANDRA / 2.0,
                },
                ..Collider::default()
            },
            Health {
                max: VIDA_DA_SALAMANDRA,
                start: VIDA_DA_SALAMANDRA,
                team: MONSTROS.to_owned(),
                on_death: VENCEU.to_owned(),
                numbers: true,
                resistances: vec![
                    Resistance {
                        item: Default::default(),
                        kind: FOGO.to_owned(),
                        rate: 0.0,
                        absorbs: false,
                    },
                    Resistance {
                        item: Default::default(),
                        kind: GELO.to_owned(),
                        rate: 2.0,
                        absorbs: false,
                    },
                ],
                ..Health::default()
            },
            HealthBar {
                offset_y: BARRA_Y,
                height: BARRA_H,
                ..HealthBar::default()
            },
        ))
        .id()
}

/// **A receita de um MORCEGO** — um agente de navegação que PERSEGUE o herói pelo nome à volta do
/// muro, com uma vida (um tiro mata-o) e um dano que o gasta ao morder.
///
/// ⚠️ (W8) **Era um projéctil teleguiado, e batia no muro** (o CONTROLO do gate
/// `um_morcego_da_a_volta_ao_muro`). ⚠️ A chegada é a de fábrica (`0,1` m entre CENTROS): os corpos
/// encostam muito antes, logo ele empurra até morder — com a chegada da cena `=1` (encostar com
/// folga) ele parava ao lado do herói e nunca mordia.
pub(super) fn receita_do_morcego(world: &mut World) -> Entity {
    world
        .spawn((
            Name::new("Morcego"),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(
                WHITE_TILE_KEY,
                [LADO_DO_MORCEGO, LADO_DO_MORCEGO],
                MORCEGO_RGBA,
            ),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            // ⭐ (report do dono, 05/10) O corpo é o QUADRADO que se desenha. Era uma bola inscrita
            // nele: os cantos desenhados saíam `41 %` para fora do corpo, e um morcego que passava a
            // `2 mm` da lava (a bola) aparecia a entrar `6 cm` nela — *«encostam na lava»*. Com o corpo
            // do desenho, o raio da navegação é o que envolve o quadrado, e o que se vê é o que toca.
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: LADO_DO_MORCEGO / 2.0,
                    half_y: LADO_DO_MORCEGO / 2.0,
                },
                ..Collider::default()
            },
            TopDownPlayer::from_law(TopDownLaw {
                speed: MORCEGO_RAPIDEZ,
                direction: DirectionMode::Free,
                default_controls: false,
                ..TopDownLaw::default()
            }),
            NavAgent {
                target: NavTarget::Named(stable_name_id(HEROI)),
                ..NavAgent::default()
            },
            Health {
                max: VIDA_DO_MORCEGO,
                start: VIDA_DO_MORCEGO,
                team: MONSTROS.to_owned(),
                numbers: true,
                ..Health::default()
            },
            Damage {
                amount: MORDIDA,
                team: MONSTROS.to_owned(),
                on_hit: OnHit::Vanish,
                ..Damage::default()
            },
        ))
        .id()
}

/// **A receita do CORAÇÃO** — um sensor que «fere» com o tipo `cura`, que o herói absorve.
pub(super) fn receita_do_coracao(world: &mut World) -> Entity {
    world
        .spawn((
            Name::new("Coracao"),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(
                WHITE_TILE_KEY,
                [LADO_DO_CORACAO, LADO_DO_CORACAO],
                CORACAO_RGBA,
            ),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: LADO_DO_CORACAO / 2.0,
                    half_y: LADO_DO_CORACAO / 2.0,
                },
                is_sensor: true,
                ..Collider::default()
            },
            Damage {
                amount: CURA,
                team: MONSTROS.to_owned(),
                on_hit: OnHit::Vanish,
                kind: CURA_TIPO.to_owned(),
                ..Damage::default()
            },
        ))
        .id()
}

/// **A receita de uma bala** do tipo `kind` — a da cena `=3` (a de fogo leva a queimadura).
pub(super) fn receita_da_bala(
    world: &mut World,
    kind: &str,
    rgba: [f32; 4],
    queima: bool,
) -> Entity {
    world
        .spawn((
            Name::new(format!("Bala de {kind}")),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            crate::smoke_desenho::disco(0.1, rgba),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.1 },
                ..Collider::default()
            },
            ProjectileMotion::from_law(
                ProjectileLaw {
                    initial_speed: 9.0,
                    range: 14.0,
                    face_velocity: true,
                    ..ProjectileLaw::default()
                },
                0,
            ),
            ph2d_ecs::Lifetime {
                duration_us: 3_000_000,
                ..ph2d_ecs::Lifetime::default()
            },
            Damage {
                amount: DANO,
                team: HEROIS.to_owned(),
                on_hit: OnHit::Vanish,
                kind: kind.to_owned(),
                over_time_per_s: if queima { QUEIMA_POR_S } else { 0.0 },
                over_time_s: QUEIMA_S,
                over_time_every_s: QUEIMA_CADA_S,
                ..Damage::default()
            },
        ))
        .id()
}

/// **Uma fábrica** que nasce `receita` em `xy` quando ouve `sinal`, com um tecto de vivos.
pub(super) fn fabrica(
    world: &mut World,
    nome: &str,
    xy: [f32; 2],
    sinal: &str,
    vivos: u32,
    receita: Entity,
) {
    world.spawn((
        Name::new(format!("Fabrica: {nome}")),
        Transform::from_translation(Vec2::new(xy[0], xy[1])),
        Factory {
            master: 0,
            on_signal: sinal.to_owned(),
            burst: 1,
            alive_max: vivos,
            ..Factory::default()
        },
        Pendente(receita),
    ));
}

/// **Um relógio** que publica `sinal` de `cada_us` em `cada_us` (ou uma vez, com `repete = false`).
pub(super) fn relogio(world: &mut World, nome: &str, sinal: &str, cada_us: u64, repete: bool) {
    world.spawn((
        Name::new(nome.to_owned()),
        Transform::from_translation(Vec2::new(0.0, 5.0)),
        Timers(vec![Timer {
            duration_us: cada_us,
            signal: sinal.to_owned(),
            autostart: true,
            repeat: repete,
            ..Timer::default()
        }]),
    ));
}
