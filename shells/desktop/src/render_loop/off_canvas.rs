//! ⭐⭐ **O que NÃO está na cena** — a pergunta que o extract faz a cada entidade.
//!
//! ⚠️⚠️ **E ela tem DOIS leitores, de propósito** (F4.6): o extract de sprites e a cadeia de
//! visibilidade do vetor ([`crate::vec_entities`]). Enquanto era só o primeiro, a regra tinha duas
//! respostas — a arte vetorial de um mestre continuava a desenhar **por baixo da cópia** que o
//! *Criar componente* deixa no lugar, e o artista não distinguia uma da outra. Foi isso que fez a
//! propagação parecer morta estando viva (o §14 do handoff): a cena 2 do smoke, com a receita
//! LONGE das cópias, propaga.
//!
//! ⚠️ **Irmão de [`super::sim_extract`] por ASSUNTO** (e porque aquele ficheiro já vive sob uma
//! excepção de LOC): lá mora *como* uma sprite vira `RenderInstance`; aqui mora *se* ela vira.
//! A decisão precisa de nome próprio — ela vive dentro de um closure que pede um renderer, e uma
//! mutação que a desligasse compilaria e passaria a suíte inteira.

use ph2d_ecs::{Entity, World};

/// ⭐⭐ **«Esta entidade está FORA da tela?»** — a pergunta que decide se ela emite instância.
///
/// Duas razões, e as duas são *«não está na cena»*:
///
/// 1. o **olho** da Hierarquia (`Visibility`), que é autoria do artista;
/// 2. ser peça de uma **RECEITA** (ADR-0164). *Um mestre é autoria guardada, não um objeto na
///    cena* — é a frase do `ph2d_ecs::master`, e até 2026-08-26 ela não valia para os pixels.
///
/// ⛔⛔ **A F4.5 escondia só a RAIZ do mestre com `Visibility`, e a premissa era falsa.** O
/// `sim_extract` di-lo pelo nome no doc do `resolve_clip_grouping`: *«Visibility is per-entity, it
/// does not propagate to descendants»*. Com uma receita que fosse um GRUPO, as peças dela continuavam a
/// desenhar — o artista fazia *Criar componente* e via **dois objetos empilhados**, um que cai e
/// outro que não, que é exatamente o defeito que a nota daquela fatia dizia ter evitado.
///
/// ⚠️ **A marca é o `MasterPiece`, que é DERIVADO** por `assign_master_pieces` (a raiz e toda a
/// descendência, re-carimbado por quadro) — e por isso não pode discordar da árvore. ⛔ Escrever
/// `Visibility` nas peças seria o contrário: a `Visibility` de uma peça é **autoria** e propaga
/// para as instâncias, logo toda cópia nasceria invisível.
///
/// ⚠️ **Função com NOME e não uma linha no fio**: a decisão vive dentro de um closure que pede um
/// renderer, e a mutação que a desligasse compilaria e passaria a suíte inteira.
pub(crate) fn is_off_canvas(sim: &World, entity: Entity) -> bool {
    // ⭐⭐ **A receita volta enquanto está a ser EDITADA** (ver `super::master_editing`): esconder
    // sempre tornaria a forma do mestre impossível de mudar, e desenhar sempre põe dois objetos
    // empilhados. A marca é derivada da selecção, e por isso as duas famílias de arte leem a
    // MESMA resposta sem ninguém lhes passar a selecção.
    (sim.get::<ph2d_ecs::MasterPiece>(entity).is_some()
        && sim.get::<ph2d_ecs::MasterEditing>(entity).is_none())
        || sim
            .get::<ph2d_ecs::Visibility>(entity)
            .is_some_and(|v| v.hidden)
}

#[cfg(test)]
mod off_canvas_tests {
    use super::is_off_canvas;
    use ph2d_ecs::{ChildOf, MasterRoot, Name, SimWorld, Transform, Visibility};

    /// ⭐⭐⭐ **A RECEITA INTEIRA sai da tela — a raiz e as peças.**
    ///
    /// ⛔ Era este o defeito: a F4.5 escondia só a raiz com `Visibility`, que não desce aos
    /// descendentes; uma receita que fosse um GRUPO continuava a desenhar as peças, e o artista
    /// via dois objetos empilhados.
    ///
    /// (Mutação: tirar o ramo do `MasterPiece` ⇒ RED na peça e na raiz.)
    #[test]
    fn a_recipe_draws_nothing_root_or_piece() {
        let mut sim = SimWorld::new();
        let root = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Recipe"), MasterRoot))
            .id();
        let arm = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Arm"), ChildOf(root)))
            .id();
        // O controlo NEGATIVO vem primeiro: antes do passe derivado, nada está marcado.
        assert!(
            !is_off_canvas(sim.world(), arm),
            "a peca ja' estava fora da tela antes de a receita existir — o gate nao mede nada"
        );
        ph2d_ecs::assign_master_pieces(sim.world_mut());
        for (what, e) in [("a raiz", root), ("a peca", arm)] {
            assert!(
                is_off_canvas(sim.world(), e),
                "{what} da receita continua a desenhar"
            );
        }
    }

    /// ⚠️ **E o olho da Hierarquia continua a valer, per-entidade.**
    ///
    /// (Mutação: tirar o ramo da `Visibility` ⇒ RED.)
    #[test]
    fn the_eye_still_hides_the_entity_it_is_on() {
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Thing")))
            .id();
        assert!(!is_off_canvas(sim.world(), e));
        sim.world_mut().entity_mut(e).insert(Visibility::hidden());
        assert!(is_off_canvas(sim.world(), e), "o olho fechado nao escondeu");
    }
}
