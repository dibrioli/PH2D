//! ⭐ **O que anexar um componente faz ALÉM de inserir o ponto neutro** (ADR-0166 / F3).
//!
//! # Porque isto existe
//!
//! A paleta anexa pelo `insert_default` do registo, que é **type-erased**: ele constrói o
//! `Default` do tipo e mais nada. Para quase todos os 100+ tipos autorados isso é exatamente
//! certo — *anexar tem de ser inerte*, e o neutro é o que não muda a cena.
//!
//! ⚠️ **Mas a F0 mediu uma exceção e a nomeou** (emenda ao ADR-0166): *nem todas as cinco portas
//! por-seção são redundantes com o `+` — as que **SEMEIAM do valor vivo** fazem algo que a paleta
//! genérica não pode fazer*. Um valor derivado do que a entidade **já tem** não está no `Default`
//! do tipo, e não pode estar: o `Default` não conhece a entidade.
//!
//! Duas foram medidas, e as duas eram um botão que a F3 apagou:
//!
//! | Componente | O que o `Default` sozinho produziria | O que o seed faz |
//! |---|---|---|
//! | `Collider` | uma caixa `0,5 × 0,5` **sem relação com o desenho** | as meias-extensões do `Sprite` |
//! | `PlatformPlayer` | uma cápsula **tangente** ao chão (`float_height = 0,5`) | a altura que de facto paira sobre este collider |
//!
//! # ⚠️ As três leis desta tabela
//!
//! 1. **O seed corre DEPOIS do `insert_default`**, sobre o componente já lá — nunca em vez dele.
//!    Assim o valor gravado continua a ser *o neutro do tipo, corrigido pelo contexto*, e não uma
//!    segunda construção que apodrece quando o tipo ganha campos.
//! 2. **É idempotente e conservador:** correr duas vezes não move nada, e nenhum seed rebaixa um
//!    valor que o artista já autorou.
//! 3. ⛔ **A lei mora no módulo DONO**, nunca aqui: este ficheiro é a tabela `nome → quem sabe`. Um
//!    segundo cálculo da meia-extensão aqui divergiria do `PhysicsFieldEdit::AddShape` no dia em
//!    que um dos dois fosse corrigido.

use ph2d_ecs::SimWorld;

/// Os nomes canónicos que semeiam. ⚠️ **É a lista que o gate percorre** — ver
/// `component_seed_tests`: tudo o que não está aqui tem de anexar **inerte**.
pub(crate) const SEEDED: &[&str] = &["ph2d::physics::Collider", "ph2d::physics::PlatformPlayer"];

/// Corre o seed de `canonical_name`, se ele tiver um. Silenciosamente no-op para todo o resto —
/// que é a resposta certa: *anexar é inerte*.
pub(crate) fn seed_after_attach(sim: &mut SimWorld, entity_bits: u64, canonical_name: &str) {
    match canonical_name {
        "ph2d::physics::Collider" => {
            crate::render_loop::seed_attached_collider(sim, entity_bits);
        }
        "ph2d::physics::PlatformPlayer" => {
            crate::render_loop::seed_attached_player(sim, entity_bits);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{SEEDED, seed_after_attach};
    use ph2d_ecs::{SimWorld, Transform};

    fn registry() -> ph2d_ecs::scene::ComponentRegistry {
        crate::init::build_component_registry()
    }

    /// ⭐ **Anexar é INERTE — para tudo o que não semeia.**
    ///
    /// A metade que o plano exige (*"anexar é inerte: bytes do componente == default"*), medida
    /// pela porta de produção e não por um `insert` à mão.
    ///
    /// ⚠️ **A lista `SEEDED` é a única excepção, e ela é curta de propósito:** um seed é um valor
    /// que o `Default` do tipo **não pode** conhecer porque depende da entidade. Tudo o resto tem
    /// de sair do registo exactamente como o tipo o define — senão o `+` passa a ser um gesto que
    /// muda a cena, e a promessa *"acrescentar uma seção não mexe no teu trabalho"* cai.
    #[test]
    fn attaching_is_inert_for_everything_that_does_not_seed() {
        let reg = registry();
        let mut checked = 0usize;
        for d in ph2d_component_desc::all() {
            if !matches!(d.attach, ph2d_component_desc::Attach::Authored { .. })
                || SEEDED.contains(&d.canonical_name)
            {
                continue;
            }
            let Some(entry) = reg.get_by_id(ph2d_ecs::scene::stable_type_id(d.canonical_name))
            else {
                continue;
            };
            let Some(insert) = entry.insert_default else {
                continue;
            };
            let mut sim = SimWorld::new();
            let e = sim
                .world_mut()
                .spawn((
                    Transform::IDENTITY,
                    ph2d_ecs::Name::new("Object"),
                    ph2d_render::Sprite::atlas(0, [2.0, 3.0], [1.0; 4]),
                ))
                .id();
            insert(sim.world_mut(), e).expect("o ponto neutro constroi");
            let before = (entry.serialize)(sim.world(), e).expect("serializa");
            seed_after_attach(&mut sim, e.to_bits(), d.canonical_name);
            let after = (entry.serialize)(sim.world(), e).expect("serializa");
            assert_eq!(
                before, after,
                "anexar {} deixou de ser inerte — se isso e' intencional, o nome tem de entrar no SEEDED com o porque",
                d.canonical_name
            );
            checked += 1;
        }
        assert!(
            checked > 40,
            "o censo varreu so' {checked} componentes — ele nao pode ficar verde por nao medir nada"
        );
    }

    /// ⭐ **O `Collider` nasce com a CAIXA DO DESENHO**, e não com a bola de meio metro do `Default`.
    ///
    /// (Mutação: tirar o braço do `Collider` do [`seed_after_attach`] ⇒ RED, e o valor que sai é a
    /// `Ball { radius: 0.5 }` — exactamente o desencontro de 2026-07-18.)
    #[test]
    fn the_collider_seed_takes_the_sprites_box() {
        let reg = registry();
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                ph2d_render::Sprite::atlas(0, [2.0, 3.0], [1.0; 4]),
            ))
            .id();
        crate::component_attach::attach_by_name(
            &mut sim,
            &reg,
            e.to_bits(),
            "ph2d::physics::Collider",
        )
        .expect("anexa");
        let shape = sim
            .world()
            .get::<ph2d_physics_ecs::Collider>(e)
            .expect("o collider")
            .shape;
        assert_eq!(
            shape,
            ph2d_physics_ecs::ColliderShape::Cuboid {
                half_x: 1.0,
                half_y: 1.5
            },
            "o collider tem de casar com o sprite 2x3"
        );
    }

    /// ⚠️ **E o seed do `Collider` NÃO reescreve uma forma autorada** — a lei que o `AddShape` já
    /// honrava, medida numa peça que voltava `0,10 x 0,50` com tudo zerado.
    #[test]
    fn the_collider_seed_never_overwrites_authored_work() {
        let mut sim = SimWorld::new();
        let authored = ph2d_physics_ecs::Collider {
            shape: ph2d_physics_ecs::ColliderShape::Cuboid {
                half_x: 0.17,
                half_y: 0.91,
            },
            density: 3.5,
            ..ph2d_physics_ecs::Collider::default()
        };
        let e = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                ph2d_render::Sprite::atlas(0, [2.0, 3.0], [1.0; 4]),
                authored,
            ))
            .id();
        seed_after_attach(&mut sim, e.to_bits(), "ph2d::physics::Collider");
        assert_eq!(
            sim.world().get::<ph2d_physics_ecs::Collider>(e).copied(),
            Some(authored),
            "o seed reescreveu trabalho do artista"
        );
    }

    /// ⭐ **O `PlatformPlayer` nasce PAIRANDO sobre o próprio collider**, e não tangente.
    ///
    /// (Mutação: tirar o braço do player ⇒ o `float_height` fica no `0,5` do `Default`, que é
    /// exactamente o piso desta cápsula: ele encosta.)
    #[test]
    fn the_player_seed_lifts_him_off_his_own_collider() {
        let reg = registry();
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                ph2d_physics_ecs::RigidBody::default(),
                ph2d_physics_ecs::Collider {
                    shape: ph2d_physics_ecs::ColliderShape::Capsule {
                        half_height: 0.3,
                        radius: 0.2,
                    },
                    ..ph2d_physics_ecs::Collider::default()
                },
            ))
            .id();
        crate::component_attach::attach_by_name(
            &mut sim,
            &reg,
            e.to_bits(),
            "ph2d::physics::PlatformPlayer",
        )
        .expect("anexa");
        let p = sim
            .world()
            .get::<ph2d_physics_ecs::PlatformPlayer>(e)
            .copied()
            .expect("o player");
        let neutral = ph2d_physics_ecs::PlatformPlayer::default();
        assert!(
            p.float_height > neutral.float_height,
            "ele nasceu tangente ({} vs o neutro {})",
            p.float_height,
            neutral.float_height
        );
    }

    /// ⚠️ **Correr o seed duas vezes não move nada** — ele é idempotente por construção (`max` /
    /// «só na forma ainda neutra»), e é isso que o torna seguro numa porta que alguém pode repetir.
    #[test]
    fn seeding_twice_changes_nothing() {
        let reg = registry();
        for name in SEEDED {
            let mut sim = SimWorld::new();
            let e = sim
                .world_mut()
                .spawn((
                    Transform::IDENTITY,
                    ph2d_render::Sprite::atlas(0, [2.0, 3.0], [1.0; 4]),
                    ph2d_physics_ecs::RigidBody::default(),
                    ph2d_physics_ecs::Collider {
                        shape: ph2d_physics_ecs::ColliderShape::Capsule {
                            half_height: 0.3,
                            radius: 0.2,
                        },
                        ..ph2d_physics_ecs::Collider::default()
                    },
                ))
                .id();
            crate::component_attach::attach_by_name(&mut sim, &reg, e.to_bits(), name)
                .expect("anexa");
            let entry = reg
                .get_by_id(ph2d_ecs::scene::stable_type_id(name))
                .expect("registado");
            let once = (entry.serialize)(sim.world(), e).expect("serializa");
            seed_after_attach(&mut sim, e.to_bits(), name);
            let twice = (entry.serialize)(sim.world(), e).expect("serializa");
            assert_eq!(once, twice, "o seed de {name} nao e' idempotente");
        }
    }
}
