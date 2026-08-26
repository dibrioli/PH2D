//! ⭐ **O objeto VAZIO na raiz** — o botão `Add` do cabeçalho da Hierarquia (ADR-0166 / F3).
//!
//! Irmão POR ASSUNTO da [`super::hierarchy`], e não uma função lá dentro: o `dispatch` dela não é
//! alcançável de um teste (pede janela, câmera, documento vetorial e o `HeroLive`), e a lei *"o que
//! nasce quando o artista cria um objeto"* é precisamente a que a F3 tem de defender com um gate.
//! O corte também devolveu o ficheiro-mãe ao teto de 600 LOC, que ele tinha acabado de passar.

use ph2d_ecs::{Name, SimWorld, Transform};

/// Cria um objeto vazio na raiz e devolve os bits dele.
///
/// ⚠️ **`Transform` + `Name`, e mais NADA.** É esta a base de que a F3 fala: o Inspector passa a
/// mostrar o que o objeto TEM, então um objeto acabado de nascer mostra **duas** seções, não doze.
/// A tentação de lhe dar um `Sprite` *"para se ver alguma coisa"* é exatamente o que esta fase
/// apaga — e o gate abaixo existe para a apanhar.
///
/// ⚠️ **Os dois assigners correm aqui, não «depois»:** uma raiz sem `RootOrder` colate em
/// `u32::MAX` e o desempate cai no `Entity::to_bits()`, que muda a cada respawn do undo — foi esse
/// o defeito que fez a captura deixar de ser ponto fixo (BUGS #15). O `StableId` responde à mesma
/// classe de pergunta (identidade que sobrevive ao respawn), e por isso os dois andam em par —
/// precedente: [`super::inspector_joint_create`].
pub(super) fn spawn_empty_root(sim: &mut SimWorld) -> u64 {
    let name = crate::name_unique::unique_name(sim, "Object");
    let bits = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(name)))
        .id()
        .to_bits();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    bits
}

#[cfg(test)]
mod tests {
    use super::spawn_empty_root;

    /// ⭐ **O que nasce é a BASE, e nada além dela** (ADR-0166 / F3).
    ///
    /// ⚠️ A metade que interessa é a NEGATIVA: um objeto vazio que já chegasse com `Sprite` faria o
    /// Inspector mostrar as seções de imagem, e o smoke da F3 — *"o Inspector mostra Name +
    /// Transform, e mais nada"* — passaria a ser sobre outra coisa.
    ///
    /// (Mutação: acrescentar `ph2d_render::Sprite::default()` ao spawn ⇒ RED.)
    #[test]
    fn an_empty_object_is_born_with_the_base_and_nothing_else() {
        let mut sim = ph2d_ecs::SimWorld::new();
        let bits = spawn_empty_root(&mut sim);
        let e = ph2d_ecs::Entity::from_bits(bits);
        let w = sim.world();

        assert!(
            w.get::<ph2d_ecs::Transform>(e).is_some(),
            "falta o Transform"
        );
        assert!(w.get::<ph2d_ecs::Name>(e).is_some(), "falta o Name");
        assert!(
            w.get::<ph2d_render::Sprite>(e).is_none(),
            "um objeto VAZIO nasceu com Sprite — o Inspector mostraria as seccoes de imagem"
        );
        assert!(
            w.get::<ph2d_ecs::SliceNine>(e).is_none()
                && w.get::<ph2d_ecs::NamedAnchorList>(e).is_none()
                && w.get::<ph2d_physics_ecs::RigidBody>(e).is_none(),
            "um objeto VAZIO nasceu com um componente autorado"
        );
    }

    /// ⚠️ **Ele nasce na RAIZ, com ordem e identidade explícitas.**
    ///
    /// Sem `RootOrder` a ordem das raízes desempata pelo `Entity::to_bits()`, que o respawn do undo
    /// muda — o defeito que fez a captura deixar de ser ponto fixo. Sem `StableId` a identidade do
    /// objeto não sobrevive a esse mesmo respawn.
    ///
    /// (Mutação: tirar qualquer um dos dois assigners ⇒ RED.)
    #[test]
    fn it_is_a_root_with_an_explicit_order_and_identity() {
        let mut sim = ph2d_ecs::SimWorld::new();
        let e = ph2d_ecs::Entity::from_bits(spawn_empty_root(&mut sim));
        let w = sim.world();
        assert!(
            w.get::<ph2d_ecs::ChildOf>(e).is_none(),
            "o botao do CABECALHO nao pertence a linha nenhuma — o objeto nasce sem pai"
        );
        assert!(
            w.get::<ph2d_ecs::RootOrder>(e).is_some(),
            "raiz sem RootOrder: a ordem passa a desempatar por bits de alocacao"
        );
        assert!(
            w.get::<ph2d_ecs::StableId>(e)
                .is_some_and(|id| !id.is_none()),
            "raiz sem StableId: a identidade nao sobrevive ao respawn do undo"
        );
    }

    /// **Dois cliques dão dois objetos, com nomes diferentes.**
    ///
    /// ⚠️ O nome vem do `unique_name`, e não de um literal: dois `Object` na lista são dois objetos
    /// que o artista não consegue distinguir — e o `stable_name_id` legado da timeline resolve
    /// homónimos para `None`, então um nome repetido não é só feio.
    #[test]
    fn two_clicks_make_two_objects_with_different_names() {
        let mut sim = ph2d_ecs::SimWorld::new();
        let a = ph2d_ecs::Entity::from_bits(spawn_empty_root(&mut sim));
        let b = ph2d_ecs::Entity::from_bits(spawn_empty_root(&mut sim));
        assert_ne!(a, b);
        let w = sim.world();
        let na = w.get::<ph2d_ecs::Name>(a).map(|n| n.0.clone());
        let nb = w.get::<ph2d_ecs::Name>(b).map(|n| n.0.clone());
        assert!(na.is_some() && nb.is_some());
        assert_ne!(na, nb, "dois objetos novos com o MESMO nome");
        assert_ne!(
            w.get::<ph2d_ecs::StableId>(a),
            w.get::<ph2d_ecs::StableId>(b),
            "dois objetos novos com a mesma identidade"
        );
    }
}
