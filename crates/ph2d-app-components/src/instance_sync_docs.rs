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
    diag: &mut crate::instance_diag::PassDiag,
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
    // ⚠️ Contado AQUI, e não no chamador: é este o ponto em que o par «tem documento dos dois
    // lados», e é a guarda que morre em silêncio quando a cópia nasce sem geometria.
    diag.doc_pairs += 1;
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
    diag.doc_diff += 1;
    // ⚠️ **Sem eco não há atribuição** — o 1.º passe, ou o 1.º depois de um load. Aí o mestre
    // ganha, como no resto do passe: inventar um override a partir de um estado que ninguém viu
    // mudar seria congelar contra a receita algo que o artista nunca pediu.
    if !master_moved && echo.contains_key(&echo_key) {
        // ⭐⭐⭐ **A cópia LIGADA responde ao contrário, e é o `Alt+D`** (Enio, 2026-08-27): a
        // edição dela não é uma excepção **dela**, é uma edição da receita feita a partir dela.
        // Sobe, e o passe seguinte leva-a às irmãs.
        //
        // ⚠️ **A peça já existia** — é o mesmo `apply_one` do verbo *Apply to Master*. O modo
        // ligado não é uma segunda lei de propagação: é a lei que já havia, escolhida no gesto em
        // vez de num clique posterior.
        //
        // ⛔⛔ **E o ECO FICA COMO ESTAVA — a 1.ª versão escrevia aqui o valor NOVO e isso era o
        // defeito** (report do Enio, 2026-08-27: *«nem todas as instâncias aceitam a edição dos
        // pontos»*). Medido por sonda, com três cópias:
        //
        // ```
        // editei a LIGADA -> mestre=-2.0  copias=[-1.0, -1.0, -2.0]  overrides=[0, 0, 0]
        //    +1 quadro:      mestre=-2.0  copias=[-1.0, -1.0, -2.0]   ← as irmãs NUNCA recebem
        //    e a seguir:                                overrides=[1, 1, 0] ← e ficam SURDAS
        // ```
        //
        // O eco é *«o que o mestre era da última vez»*, e é ele que responde **quem se mexeu**.
        // Ensiná-lo o valor novo no mesmo passe faz o quadro seguinte concluir *«o mestre não se
        // mexeu»* — e então cada irmã, que ainda tem a forma velha, lê-se como *«só a instância
        // mexeu»* e captura uma excepção que ninguém pediu. Uma subida que atualiza o eco é uma
        // subida que **ninguém vê**.
        //
        // ⇒ deixá-lo velho é o que faz o passe seguinte dizer *«o mestre mexeu-se»* e levar a
        // forma às irmãs. A cópia que subiu já a tem (`want == have`), logo não é reescrita, e o
        // ponto fixo chega no 3.º passe. *O eco não é um cache do mestre: é a memória de quem
        // mexeu.*
        if sim.world().get::<ph2d_ecs::LinkedArt>(inst).is_some() {
            if apply_one(sim, docs, inst, master) {
                diag.doc_wrote += 1;
                return 1;
            }
            return 0;
        }
        overrides.overrides.insert(key); // (3)
        return 0;
    }
    write_content(docs, ip.0, &want);
    diag.doc_wrote += 1;
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
