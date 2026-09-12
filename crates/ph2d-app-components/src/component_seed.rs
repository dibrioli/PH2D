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
//! 3. ⛔ **A lei — e a TABELA — moram no módulo DONO**, nunca aqui: este ficheiro é a porta. Um
//!    segundo cálculo da meia-extensão aqui divergiria do `PhysicsFieldEdit::AddShape` no dia em
//!    que um dos dois fosse corrigido.

use ph2d_ecs::SimWorld;

/// O que uma semente faz: escreve o valor inicial daquele componente na entidade dada.
pub type Seed = fn(&mut SimWorld, u64);

/// ⭐ **A TABELA** `nome canónico → quem sabe semear` — e ela é INJECTADA por quem compõe.
///
/// ⚠️ **É uma tabela e não um `match`, e a razão é medida:** o gate precisa de percorrer *quem
/// semeia* para afirmar que **todo o resto anexa inerte**, e uma lista de nomes escrita ao lado de
/// um `match` seriam **duas respostas à mesma pergunta** — a que o gate lê envelheceria em silêncio
/// no dia em que alguém acrescentasse um braço sem a lista. Aqui os dois leem a mesma linha.
///
/// ⭐⭐ **E ela já não mora aqui** (auditoria de arquitectura 2026-09-12, A1): esta família
/// importava as duas sementes da `ph2d-app-physics` pelo nome, e uma família não chama outra
/// (ADR-0075). Hoje a família DONA do componente publica a sua tabela
/// (`ph2d_app_physics::physics_seed::COMPONENT_SEEDS`) e a COMPOSIÇÃO — a shell — entrega-a à porta
/// de anexar. Os gates que provam as sementes reais moram com a composição
/// (`shells/desktop/src/component_seed_seam_tests.rs`).
pub type SeedTable<'a> = &'a [(&'a str, Seed)];

/// Corre o seed de `canonical_name`, se a tabela tiver um. Silenciosamente no-op para todo o resto —
/// que é a resposta certa: *anexar é inerte*.
pub fn seed_after_attach(
    seeds: SeedTable<'_>,
    sim: &mut SimWorld,
    entity_bits: u64,
    canonical_name: &str,
) {
    if let Some((_, seed)) = seeds.iter().find(|(n, _)| *n == canonical_name) {
        seed(sim, entity_bits);
    }
}

#[cfg(test)]
mod tests {
    use super::SeedTable;
    use ph2d_ecs::{SimWorld, Transform};

    fn registry() -> ph2d_ecs::scene::ComponentRegistry {
        crate::component_registry_for_tests::registo()
    }

    /// ⭐ **A porta só semeia o que a TABELA INJECTADA nomeia** — e sem tabela anexar é inerte.
    ///
    /// ⚠️ É o gate do MECANISMO, e mora aqui porque não precisa da física: as sementes reais e o
    /// censo de inércia com elas moram na composição (`component_seed_seam_tests` da shell).
    /// (Mutação: ignorar a tabela no `seed_after_attach` ⇒ o 2.º `assert` reprova.)
    #[test]
    fn the_door_seeds_only_what_the_injected_table_names() {
        fn marca(sim: &mut SimWorld, bits: u64) {
            let e = ph2d_ecs::Entity::from_bits(bits);
            sim.world_mut()
                .entity_mut(e)
                .insert(ph2d_ecs::Name::new("semeado"));
        }
        let reg = registry();
        let tabela: SeedTable<'_> = &[("ph2d::physics::RigidBody", marca)];
        let semeado = |sim: &SimWorld, e| {
            sim.world().get::<ph2d_ecs::Name>(e) == Some(&ph2d_ecs::Name::new("semeado"))
        };

        let mut sim = SimWorld::new();
        let e = sim.world_mut().spawn(Transform::IDENTITY).id();
        crate::component_attach::attach_by_name(
            &mut sim,
            &reg,
            tabela,
            e.to_bits(),
            "ph2d::physics::Collider",
        )
        .expect("anexa");
        assert!(
            !semeado(&sim, e),
            "semeou um componente que a tabela não nomeia"
        );
        crate::component_attach::attach_by_name(
            &mut sim,
            &reg,
            tabela,
            e.to_bits(),
            "ph2d::physics::RigidBody",
        )
        .expect("anexa");
        assert!(
            semeado(&sim, e),
            "a tabela nomeia o RigidBody e a porta não o semeou"
        );

        let f = sim.world_mut().spawn(Transform::IDENTITY).id();
        crate::component_attach::attach_by_name(
            &mut sim,
            &reg,
            &[],
            f.to_bits(),
            "ph2d::physics::RigidBody",
        )
        .expect("anexa");
        assert!(!semeado(&sim, f), "sem tabela, anexar tem de ser inerte");
    }

    /// ⚠️ **A cascata NÃO rebaixa o que já existe.** O `insert` do bevy substitui, não funde: sem o
    /// no-op sobre o presente, anexar *Platform Player* num corpo `Static` já autorado devolvia-o
    /// ao `Default` — apagando trabalho em silêncio, na porta que existe para não o fazer.
    #[test]
    fn the_cascade_never_downgrades_what_is_already_there() {
        let reg = registry();
        let mut sim = SimWorld::new();
        let authored = ph2d_physics_ecs::RigidBody {
            kind: ph2d_physics_ecs::BodyKind::Static,
        };
        let e = sim.world_mut().spawn((Transform::IDENTITY, authored)).id();
        crate::component_attach::attach_by_name(
            &mut sim,
            &reg,
            &[],
            e.to_bits(),
            "ph2d::physics::PlatformPlayer",
        )
        .expect("anexa");
        assert_eq!(
            sim.world().get::<ph2d_physics_ecs::RigidBody>(e).copied(),
            Some(authored),
            "a cascata reescreveu o corpo que o artista tinha autorado"
        );
    }
}
