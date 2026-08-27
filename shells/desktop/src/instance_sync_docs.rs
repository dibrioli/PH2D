//! ⭐⭐ **A propagação dos DOCUMENTOS possuídos** — a geometria vetorial de uma peça (F4.6b).
//!
//! ⚠️ **Irmão de [`super::instance_sync`] por ASSUNTO** (e porque aquele ficheiro está no tecto de
//! 600 LOC): lá mora a propagação por **bytes de componente**; aqui a que tem de ser por
//! **CONTEÚDO**.
//!
//! # Por que os bytes do componente não servem, e isso não é uma excepção — é a definição
//!
//! O `VecPathRef` de uma instância aponta para o path **dela** (F4.6a: cada peça tem geometria
//! própria, senão duas entidades escreviam no mesmo documento). Logo os bytes do componente
//! **diferem para sempre**, de propósito — exatamente como os da junta, que nomeia os corpos da
//! instância. Comparar bytes aqui daria *«diferente»* em todo quadro e o passe reescreveria a peça
//! sempre, matando o ponto fixo.
//!
//! ⇒ o que se compara é o **conteúdo do documento**, com o id normalizado. E o que se escreve é o
//! conteúdo do mestre **dentro do path da instância** — o id dela nunca se mexe.
//!
//! # ⚠️ As três respostas são as MESMAS
//!
//! (1) a instância possui (override) ⇒ não se toca; (2) o mestre mexeu-se ⇒ propaga; (3) só a
//! instância mexeu ⇒ nasce override. É o mesmo eco e a mesma chave de override
//! (`OverrideKey { piece, type_id: stable_type_id("ph2d::ecs::VecPathRef") }`), e por isso o
//! *Revert* e o *Apply* alcançam a forma de uma peça sem uma segunda tabela.

use ph2d_ecs::{Entity, ObjectInstance, OverrideKey, SimWorld, VecPathRef};
use ph2d_vec_scene::{VecPath, VecPathId};
use std::collections::BTreeMap;

use crate::instance_docs::OwnedDocs;

/// O nome canónico do único documento que esta porta propaga hoje.
pub(crate) const VEC_PATH: &str = "ph2d::ecs::VecPathRef";

/// O eco, na mesma forma do de [`super::instance_sync`].
type Echo = BTreeMap<(u64, u64), Option<Vec<u8>>>;

/// **Os bytes do CONTEÚDO de um path** — o id zerado, que é o que o torna comparável entre o
/// mestre e a cópia.
///
/// ⚠️ Sem esta normalização, dois paths com a mesma forma comparariam **diferente** para sempre, e
/// o passe deixaria de ser um ponto fixo.
fn content_bytes(path: &VecPath) -> Vec<u8> {
    let mut c = path.clone();
    c.id = VecPathId::default();
    postcard::to_allocvec(&c).unwrap_or_default()
}

/// ⭐ **Propaga o documento de UMA peça.** Devolve `1` se escreveu.
///
/// `master_id` é a `piece` da [`OverrideKey`] — a identidade da peça do MESTRE, como no resto do
/// passe.
pub(super) fn sync_one(
    sim: &mut SimWorld,
    docs: &mut OwnedDocs<'_>,
    (inst, master, master_id): (Entity, Entity, u64),
    overrides: &mut ObjectInstance,
    echo: &Echo,
    next_master: &mut Echo,
) -> usize {
    let type_id = ph2d_ecs::scene::stable_type_id(VEC_PATH);
    let (Some(mp), Some(ip)) = (
        sim.world().get::<VecPathRef>(master).copied(),
        sim.world().get::<VecPathRef>(inst).copied(),
    ) else {
        return 0;
    };
    let Some(want) = docs.vec_scene.path(mp.0).cloned() else {
        return 0;
    };
    let want_bytes = Some(content_bytes(&want));
    let echo_key = (master_id, type_id);
    let master_moved = echo.get(&echo_key).is_some_and(|p| *p != want_bytes);
    next_master
        .entry(echo_key)
        .or_insert_with(|| want_bytes.clone());

    let key = OverrideKey {
        piece: master_id,
        type_id,
    };
    if overrides.overrides.contains(&key) {
        return 0; // (1) a forma desta peça é dela
    }
    let Some(have) = docs.vec_scene.path(ip.0) else {
        return 0;
    };
    if content_bytes(have) == want_bytes.clone().unwrap_or_default() {
        return 0;
    }
    // ⚠️ **Sem eco não há atribuição** — o 1.º passe, ou o 1.º depois de um load. Aí o mestre
    // ganha, como no resto do passe: inventar um override a partir de um estado que ninguém viu
    // mudar seria congelar contra a receita algo que o artista nunca pediu.
    if !master_moved && echo.contains_key(&echo_key) {
        overrides.overrides.insert(key); // (3)
        return 0;
    }
    write_content(docs, ip.0, &want);
    1
}

/// **Escreve o conteúdo de `src` no path `dst`, preservando o id de `dst`.**
///
/// ⚠️ **O id da instância NUNCA se mexe:** ele é a chave do `vec_entities` (uma entidade por path)
/// e do `VecPathRef` dela. Escrever o id do mestre poria as duas a apontar para o mesmo documento —
/// o defeito que a F4.6a existe para não cometer.
fn write_content(docs: &mut OwnedDocs<'_>, dst: VecPathId, src: &VecPath) {
    if let Some(p) = docs.vec_scene.path_mut(dst) {
        let keep = p.id;
        *p = src.clone();
        p.id = keep;
    }
}

/// ⭐ **APLICAR ao mestre**, do lado do documento: o conteúdo da peça da instância entra no path da
/// receita.
///
/// ⚠️ O `insert_from_bytes` do caminho geral escreveria o **id** do `VecPathRef` da instância no
/// mestre — as duas passariam a apontar para o mesmo path, e editar uma mexeria na outra. *Um
/// documento aplica-se por conteúdo, como se propaga por conteúdo.*
pub(super) fn apply_one(
    sim: &SimWorld,
    docs: &mut OwnedDocs<'_>,
    inst_piece: Entity,
    master_piece: Entity,
) -> bool {
    let (Some(ip), Some(mp)) = (
        sim.world().get::<VecPathRef>(inst_piece).copied(),
        sim.world().get::<VecPathRef>(master_piece).copied(),
    ) else {
        return false;
    };
    let Some(src) = docs.vec_scene.path(ip.0).cloned() else {
        return false;
    };
    write_content(docs, mp.0, &src);
    true
}

#[cfg(test)]
#[path = "instance_sync_docs_tests.rs"]
mod tests;
