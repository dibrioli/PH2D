//! ⭐ **O MESTRE de um componente de objeto, e o que o torna INERTE** (ADR-0164 / plano F4).
//!
//! Um *mestre* é a subárvore que a biblioteca guarda: a receita de que as instâncias nascem. Ele
//! vive no **mesmo `World`** que a cena (é a tese do ADR-0164 — instâncias são entidades reais,
//! não um documento à parte), e é isso que cria o problema que este módulo resolve.
//!
//! # ⚠️ Porque ele tem de ser EXCLUÍDO da ponte de física
//!
//! A [refutação 1] mediu o preço de não o excluir, e é ela que manda aqui. A `BodyQuery` da ponte
//! é `(Entity, &RigidBody, &Collider, &Transform)` **sem filtro nenhum** — logo um mestre com uma
//! peça `RigidBody` no mesmo mundo **é simulado**:
//!
//! - o `readback` carimba o `Transform` dele **por tique**, e um sync dirigido por change-tick
//!   leria isso como *«o mestre mudou»* e propagaria a **pose SIMULADA** a todas as instâncias;
//! - pausado, o `settle` compara o `Transform` com a pose do corpo e, se diferirem,
//!   **teleporta e zera a velocidade** — todas as instâncias saltavam para a pose do mestre a cada
//!   quadro parado;
//! - e o que se desenha passaria a depender de o sync correr antes ou depois do `readback`.
//!
//! ⇒ *«a receita não cai»*. Um mestre é autoria guardada, não um objeto na cena.
//!
//! # ⚠️ `MasterRoot` é AUTORADO; `MasterPiece` é DERIVADO — e a diferença decide o formato
//!
//! O artista cria um mestre com um gesto, e isso é um facto do documento: o [`MasterRoot`] é
//! **registado** e viaja no arquivo. Já *«que entidades pertencem a este mestre»* é uma pergunta
//! sobre a HIERARQUIA, e a resposta re-deriva-se de graça — o [`MasterPiece`] é **stampado por um
//! passe** e **não é registado**, exatamente como o `StableId`.
//!
//! *A lei da casa: o invariante impõe-se na DERIVAÇÃO, nunca em cada gesto* (ADR-0161 W-34). Gravar
//! um valor derivado seria estado a envenenar o undo, e um mestre editado depois do gesto ficaria
//! com peças por marcar — **simuladas em silêncio**, que é precisamente o defeito.
//!
//! [refutação 1]: ../../../docs/Components/pesquisa/instancias_2026-08-21/refutacao_1_sync_determinismo.md

use crate::{ChildOf, Children};
use bevy_ecs::prelude::Component;
use bevy_ecs::query::{With, Without};
use bevy_ecs::world::World;
use serde::{Deserialize, Serialize};

/// **A raiz de um MESTRE** — a subárvore que a biblioteca guarda como receita.
///
/// ⚠️ **Registado e persistido**, porque é autoria: o artista disse *«isto é um componente»*.
/// Quem o marca é o gesto, e não uma derivação.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MasterRoot;

/// **Uma entidade que pertence a um mestre** — a raiz e toda a descendência dela.
///
/// ⚠️ **DERIVADO, e por isso NÃO registado** (ver o cabeçalho do módulo). É por ele que as
/// consultas da ponte filtram: `Without<MasterPiece>`.
///
/// ⛔ Não o insira à mão. Quem o mantém é [`assign_master_pieces`], e uma marca escrita fora dela
/// sobrevive só até ao passe seguinte — o que é pior que não existir, porque funciona uma vez.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct MasterPiece;

/// ⭐⭐ **Esta entidade é peça da receita que o artista está a EDITAR agora.**
///
/// ⚠️ **DERIVADA da SELEÇÃO, e por isso NÃO registada** — como o [`MasterPiece`], e pela mesma
/// razão: a selecção é vista, não documento, e um valor derivado no arquivo envenena o undo.
///
/// Quem a mantém é o passe da shell (`render_loop::master_editing`), e ela existe para uma
/// pergunta só: *uma receita não está na cena, **excepto** enquanto se mexe nela* — sem isso, ou o
/// artista vê dois objetos empilhados, ou não consegue mudar a forma do mestre.
///
/// ⛔ Não a insira à mão: uma marca escrita fora do passe sobrevive só até ao passe seguinte, o que
/// é pior que não existir, porque funciona uma vez.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct MasterEditing;

/// ⭐ **Marca toda a descendência de cada [`MasterRoot`], e DESMARCA o que já não pertence a um.**
/// Devolve `true` quando mexeu em alguma coisa.
///
/// Idempotente: correr de novo é no-op — é isso que permite chamá-la todo o quadro, ao lado do
/// `assign_missing_root_order` e do `assign_missing_stable_ids`, que existem pela mesma razão.
///
/// ⚠️ **As DUAS metades são obrigatórias.** Só marcar deixa uma peça arrastada para fora do mestre
/// permanentemente invisível ao solver — *um objeto que o artista tirou da biblioteca e que não cai*
/// —, e esse defeito é mudo: nada na tela o explica.
pub fn assign_master_pieces(world: &mut World) -> bool {
    // Passagem 1: quem DEVERIA estar marcado — a descendência de cada raiz, em largura.
    let roots: Vec<crate::Entity> = {
        let mut q = world.query_filtered::<crate::Entity, With<MasterRoot>>();
        q.iter(world).collect()
    };
    let mut want: std::collections::BTreeSet<crate::Entity> = std::collections::BTreeSet::new();
    let mut stack = roots;
    while let Some(e) = stack.pop() {
        if !want.insert(e) {
            continue;
        }
        if let Some(kids) = world.get::<Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }

    // Passagem 2: quem ESTÁ marcado.
    let have: std::collections::BTreeSet<crate::Entity> = {
        let mut q = world.query_filtered::<crate::Entity, With<MasterPiece>>();
        q.iter(world).collect()
    };

    let mut touched = false;
    for &e in want.difference(&have) {
        if let Ok(mut em) = world.get_entity_mut(e) {
            em.insert(MasterPiece);
            touched = true;
        }
    }
    for &e in have.difference(&want) {
        if let Ok(mut em) = world.get_entity_mut(e) {
            em.remove::<MasterPiece>();
            touched = true;
        }
    }
    touched
}

/// **Esta entidade pertence a um mestre?** — a pergunta que o resto do app faz.
///
/// ⚠️ Lê a MARCA, e não sobe a árvore: subir por entidade e por quadro seria `O(profundidade)` em
/// cada consumidor, e cada um teria a sua resposta. A marca é a resposta, num sítio só.
#[must_use]
pub fn is_master_piece(world: &World, entity: crate::Entity) -> bool {
    world.get::<MasterPiece>(entity).is_some()
}

/// **A raiz do mestre a que esta entidade pertence**, se alguma — subindo por `ChildOf`.
///
/// ⚠️ Usada por quem precisa do DONO (o sync, a biblioteca), nunca por quem só precisa de saber
/// *se* pertence: para isso está o [`is_master_piece`], que é `O(1)`.
#[must_use]
pub fn master_root_of(world: &World, entity: crate::Entity) -> Option<crate::Entity> {
    let mut e = entity;
    loop {
        if world.get::<MasterRoot>(e).is_some() {
            return Some(e);
        }
        e = world.get::<ChildOf>(e)?.0;
    }
}

/// ⚠️ Só para gates: quantas entidades a ponte **veria** hoje — a conta que o filtro muda.
#[must_use]
#[doc(hidden)]
pub fn count_simulatable(world: &mut World) -> usize {
    let mut q = world.query_filtered::<crate::Entity, Without<MasterPiece>>();
    q.iter(world).count()
}

#[cfg(test)]
#[path = "master_tests.rs"]
mod tests;
