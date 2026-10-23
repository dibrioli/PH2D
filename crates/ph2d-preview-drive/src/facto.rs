//! **O facto de cada motor, lido e escrito no mundo** — ver o doc do `mod facto` no `lib.rs`.

#[allow(clippy::wildcard_imports)]
use super::*;

impl Driven {
    /// Qual motor é dono deste facto — a chave do ledger sai daqui, não de um argumento a mais.
    pub fn driver(self) -> Driver {
        match self {
            Self::SpriteAnim { .. } => Driver::SpriteAnim,
            Self::SolverPose(_) => Driver::SolverPose,
            Self::SpriteAlpha(_) => Driver::SpriteAlpha,
            Self::MorphT(_) => Driver::MorphT,
            Self::MorphPair(_) => Driver::MorphPair,
            Self::JointParams(_) => Driver::JointParams,
            Self::StagePose(_) => Driver::PrefabStage,
            Self::Visible(_) => Driver::SignalVisibility,
            Self::ScriptPose(_) => Driver::ScriptPose,
            Self::CanvasPose(_) => Driver::CanvasPose,
            Self::ParallaxPose(_) => Driver::ParallaxPose,
            Self::TweenPose(_) => Driver::TweenPose,
            Self::TweenTint { .. } => Driver::TweenTint,
            Self::SpriteCor { .. } => Driver::SpriteCor,
            Self::PontaNoMundo(_) => Driver::PontaNoMundo,
            Self::Roldana { .. } => Driver::Roldana,
        }
    }

    /// Lê do mundo o facto que `driver` conduz em `entity`. `None` = a entidade não o tem (foi
    /// despawnada por um restore, ou nunca teve o componente).
    #[must_use]
    pub fn read(driver: Driver, sim: &SimWorld, entity: Entity) -> Option<Self> {
        match driver {
            Driver::SpriteAnim => {
                let a = sim.world().get::<ph2d_ecs::SpriteAnimator>(entity)?;
                // ⚠️ **A célula viva mudou de casa** (ADR-0164 F1 passo 6): era `Sprite::frame`,
                // hoje é `SpriteGrid::frame`. Uma sprite SEM grelha não tem célula que o motor
                // possa mexer, então não há facto de pré-visualização a guardar — e o `?` aqui é
                // a leitura certa disso, não uma falha.
                let g = sim.world().get::<ph2d_ecs::SpriteGrid>(entity)?;
                Some(Self::SpriteAnim {
                    elapsed_ticks: a.elapsed_ticks,
                    pingpong_reverse: a.pingpong_reverse,
                    repeat_count: a.repeat_count,
                    frame: g.frame,
                })
            }
            Driver::SolverPose => Some(Self::SolverPose(*sim.world().get::<Transform>(entity)?)),
            Driver::SpriteAlpha => Some(Self::SpriteAlpha(
                sim.world().get::<Sprite>(entity)?.tint[3],
            )),
            Driver::MorphT => Some(Self::MorphT(
                sim.world().get::<ph2d_ecs::VecMorph>(entity)?.t,
            )),
            Driver::MorphPair => Some(Self::MorphPair(
                sim.world().get::<ph2d_ecs::VecMorph>(entity)?.sources,
            )),
            Driver::JointParams => Some(Self::JointParams(
                *sim.world().get::<ph2d_physics_ecs::PhysicsJoint>(entity)?,
            )),
            Driver::PrefabStage => Some(Self::StagePose(*sim.world().get::<Transform>(entity)?)),
            Driver::SignalVisibility => Some(Self::Visible(
                sim.world().get::<ph2d_ecs::Visibility>(entity)?.hidden,
            )),
            Driver::ScriptPose => Some(Self::ScriptPose(*sim.world().get::<Transform>(entity)?)),
            Driver::CanvasPose => Some(Self::CanvasPose(*sim.world().get::<Transform>(entity)?)),
            Driver::ParallaxPose => {
                Some(Self::ParallaxPose(*sim.world().get::<Transform>(entity)?))
            }
            Driver::TweenPose => Some(Self::TweenPose(*sim.world().get::<Transform>(entity)?)),
            Driver::TweenTint => {
                let s = sim.world().get::<Sprite>(entity)?;
                Some(Self::TweenTint {
                    self_tint: s.self_tint,
                    fill: s.tint_fill,
                })
            }
            Driver::SpriteCor => {
                let w = sim.world();
                let s = w.get::<Sprite>(entity)?;
                Some(Self::SpriteCor {
                    tint: [s.tint[0], s.tint[1], s.tint[2]],
                    opacidade: s.opacity,
                    cantos: w.get::<ph2d_ecs::SpriteCornerTint>(entity).map(|c| c.0),
                    emissao: w.get::<ph2d_ecs::SpriteEmissive>(entity).map(|e| e.0),
                })
            }
            Driver::PontaNoMundo => {
                let w = sim.world();
                w.get::<ph2d_physics_ecs::PhysicsJoint>(entity)?;
                Some(Self::PontaNoMundo(
                    w.get::<ph2d_physics_ecs::JointWorldAnchor>(entity)
                        .is_some(),
                ))
            }
            Driver::Roldana => {
                let w = sim.world();
                Some(Self::Roldana {
                    roda: *w.get::<ph2d_physics_ecs::PulleyWheel>(entity)?,
                    weston: w.get::<ph2d_physics_ecs::WestonAxle>(entity).is_some(),
                })
            }
        }
    }

    /// Escreve este facto no mundo. Entidade sem o componente = no-op silencioso (o restore do
    /// undo respawna tudo com bits novos, e um memo pode sobreviver-lhe um quadro).
    ///
    /// ⚠️ **Escreve só quando MUDA**, pela razão de sempre nesta casa: o `bevy` marca a alteração
    /// no `deref_mut`, e um componente tocado todo o quadro é ruído para quem lê `Changed<…>`.
    ///
    /// ⚠️ **`pub(crate)` desde 2026-09-07, e com uma obrigação colada:** quem escreve um facto de
    /// pré-visualização por aqui tem de o **declarar** ([`PreviewDrive::driven`]) no mesmo quadro,
    /// senão a `settle` esquece-o e o valor de pré-visualização vira documento. O primeiro
    /// consumidor de fora é o palco do *Edit Prefab* (o `prefab_stage` da shell), que escreve a pose
    /// de palco e a repõe ao fechar — e usar esta porta é o que o impede de ter a sua própria
    /// versão da regra *«só quando muda»*.
    pub fn write(self, sim: &mut SimWorld, entity: Entity) {
        match self {
            Self::SpriteAnim {
                elapsed_ticks,
                pingpong_reverse,
                repeat_count,
                frame,
            } => {
                if let Some(mut a) = sim.world_mut().get_mut::<ph2d_ecs::SpriteAnimator>(entity) {
                    if a.elapsed_ticks != elapsed_ticks {
                        a.elapsed_ticks = elapsed_ticks;
                    }
                    if a.pingpong_reverse != pingpong_reverse {
                        a.pingpong_reverse = pingpong_reverse;
                    }
                    if a.repeat_count != repeat_count {
                        a.repeat_count = repeat_count;
                    }
                }
                if let Some(mut g) = sim.world_mut().get_mut::<ph2d_ecs::SpriteGrid>(entity)
                    && g.frame != frame
                {
                    g.frame = frame;
                }
            }
            Self::SolverPose(pose) => {
                if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity)
                    && *t != pose
                {
                    *t = pose;
                }
            }
            Self::SpriteAlpha(a) => {
                if let Some(mut s) = sim.world_mut().get_mut::<Sprite>(entity)
                    && s.tint[3] != a
                {
                    s.tint[3] = a;
                }
            }
            Self::TweenPose(pose) => {
                if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity)
                    && *t != pose
                {
                    *t = pose;
                }
            }
            Self::TweenTint { self_tint, fill } => {
                if let Some(mut s) = sim.world_mut().get_mut::<Sprite>(entity)
                    && (s.self_tint != self_tint || s.tint_fill != fill)
                {
                    s.self_tint = self_tint;
                    s.tint_fill = fill;
                }
            }
            Self::SpriteCor {
                tint,
                opacidade,
                cantos,
                emissao,
            } => {
                if let Some(mut s) = sim.world_mut().get_mut::<Sprite>(entity)
                    && (s.tint[..3] != tint || s.opacity != opacidade)
                {
                    s.tint[..3].copy_from_slice(&tint);
                    s.opacity = opacidade;
                }
                let w = sim.world_mut();
                if w.get_entity(entity).is_err() {
                    return;
                }
                let ja = w.get::<ph2d_ecs::SpriteCornerTint>(entity).map(|c| c.0);
                if ja != cantos {
                    match cantos {
                        Some(c) => {
                            w.entity_mut(entity).insert(ph2d_ecs::SpriteCornerTint(c));
                        }
                        None => {
                            w.entity_mut(entity).remove::<ph2d_ecs::SpriteCornerTint>();
                        }
                    }
                }
                let ja = w.get::<ph2d_ecs::SpriteEmissive>(entity).map(|e| e.0);
                if ja != emissao {
                    match emissao {
                        Some(v) => {
                            w.entity_mut(entity).insert(ph2d_ecs::SpriteEmissive(v));
                        }
                        None => {
                            w.entity_mut(entity).remove::<ph2d_ecs::SpriteEmissive>();
                        }
                    }
                }
            }
            Self::MorphT(v) => {
                if let Some(mut m) = sim.world_mut().get_mut::<ph2d_ecs::VecMorph>(entity)
                    && m.t != v
                {
                    m.t = v;
                }
            }
            Self::MorphPair(v) => {
                if let Some(mut m) = sim.world_mut().get_mut::<ph2d_ecs::VecMorph>(entity)
                    && m.sources != v
                {
                    m.sources = v;
                }
            }
            Self::JointParams(j) => {
                if let Some(mut cur) = sim
                    .world_mut()
                    .get_mut::<ph2d_physics_ecs::PhysicsJoint>(entity)
                    && *cur != j
                {
                    *cur = j;
                }
            }
            Self::StagePose(pose)
            | Self::ScriptPose(pose)
            | Self::CanvasPose(pose)
            | Self::ParallaxPose(pose) => {
                if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity)
                    && *t != pose
                {
                    *t = pose;
                }
            }
            Self::Visible(hidden) => {
                if let Some(mut v) = sim.world_mut().get_mut::<ph2d_ecs::Visibility>(entity)
                    && v.hidden != hidden
                {
                    v.hidden = hidden;
                }
            }
            Self::PontaNoMundo(no_mundo) => {
                let w = sim.world_mut();
                if w.get::<ph2d_physics_ecs::PhysicsJoint>(entity).is_none()
                    || w.get::<ph2d_physics_ecs::JointWorldAnchor>(entity)
                        .is_some()
                        == no_mundo
                {
                    return;
                }
                if no_mundo {
                    w.entity_mut(entity)
                        .insert(ph2d_physics_ecs::JointWorldAnchor);
                } else {
                    w.entity_mut(entity)
                        .remove::<ph2d_physics_ecs::JointWorldAnchor>();
                }
            }
            Self::Roldana { roda, weston } => {
                let w = sim.world_mut();
                let Some(mut cur) = w.get_mut::<ph2d_physics_ecs::PulleyWheel>(entity) else {
                    return;
                };
                if *cur != roda {
                    *cur = roda;
                }
                if w.get::<ph2d_physics_ecs::WestonAxle>(entity).is_some() != weston {
                    if weston {
                        w.entity_mut(entity).insert(ph2d_physics_ecs::WestonAxle);
                    } else {
                        w.entity_mut(entity)
                            .remove::<ph2d_physics_ecs::WestonAxle>();
                    }
                }
            }
        }
    }
}
