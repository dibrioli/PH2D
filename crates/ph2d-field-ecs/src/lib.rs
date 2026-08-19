//! `ph2d-field-ecs` — a ponte que faz um documento de campo ser um **objeto do projeto**
//! ([ADR-0161]).
//!
//! Com o componente registrado, um objeto de modelagem 3D é salvo, desfeito e restaurado **sem uma
//! linha de código do lado do snapshot** — é a mesma porta que a física atravessa
//! ([ADR-0131](../../../docs/architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)).
//!
//! # O que entra num componente, e o que **nunca** entra
//!
//! Entra o **documento** — dado autorado, pequeno, que só muda quando o artista mexe.
//! ⛔ Não entra nada **derivado**: a árvore compilada, a malha, o quadro traçado. A lei é da casa e
//! está paga: o `canonicalize` do undo ordena as linhas pelos **bytes** do componente, então algo
//! que mude a cada quadro faz **todo quadro virar um passo espúrio de undo**.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Component, SimComponent};
use ph2d_field::{Blend, FieldDoc, FieldError};
use serde::{Deserialize, Serialize};

/// Um objeto de modelagem 3D: a entidade carrega o **documento** que a define.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldObject {
    pub doc: FieldDoc,
}

impl SimComponent for FieldObject {}

/// Registra os componentes do módulo no registro compartilhado.
///
/// Sem esta chamada o `WorldSnapshot` **descarta o componente em silêncio** — e o sintoma não é um
/// erro: é o objeto sumir ao desfazer ou ao reabrir o arquivo.
///
/// ⚠️ **O identificador vem do NOME** (`stable_type_id`), não de um contador. É o que torna
/// registrar um componente seguro entre linhas paralelas: duas linhas só colidem se escolherem a
/// **mesma string**, e o registro entra em pânico ao vê-lo — em vez de trocar de id em silêncio.
pub fn register_field_components(reg: &mut ComponentRegistry) {
    reg.register::<FieldObject>("ph2d::field::FieldObject");
}

/// O campo de uma **cena**: a união de todos os objetos, na ordem da chave.
///
/// ⚠️ **A chave estável não é cerimônia — é o que impede um bug de undo.** A ordem de uma consulta
/// ECS não é garantida, e unir os documentos na ordem em que a consulta os devolve produziria uma
/// árvore com os mesmos objetos e **bytes diferentes** a cada quadro. O snapshot compara bytes;
/// logo, cada quadro viraria um passo de undo — que é literalmente o bug que o `canonicalize()`
/// do shell existe para matar, e que este repositório já pagou uma vez.
///
/// Por isso a assinatura **exige** a chave em vez de aceitar um iterador solto: uma API que
/// permitisse a ordem errada seria usada na ordem errada.
///
/// Devolve `None` quando não há objeto nenhum — uma cena vazia não tem campo.
///
/// # Errors
/// Propaga a validação de [`FieldDoc::union_all`].
pub fn scene_field<K: Ord>(
    objects: impl IntoIterator<Item = (K, FieldDoc)>,
    blend: Blend,
) -> Option<Result<FieldDoc, FieldError>> {
    let mut v: Vec<(K, FieldDoc)> = objects.into_iter().collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    let docs: Vec<FieldDoc> = v.into_iter().map(|(_, d)| d).collect();
    FieldDoc::union_all(&docs, blend)
}

#[cfg(test)]
mod tests;
