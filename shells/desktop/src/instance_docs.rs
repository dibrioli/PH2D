//! ⭐⭐ **Os DOCUMENTOS possuídos, clonados para a cópia** (ADR-0164 / plano F4.6).
//!
//! A cópia profunda ([`ph2d_ecs::deep_copy_subtree`]) copia os bytes de todo componente registado
//! — **menos** os quatro que apontam para um documento fora do ECS (`ComponentDesc::owned_document`:
//! `VecPathRef` · `PaintedDoc` · `BakedForm` · `FlipObjectRef`). O id deles é opaco, e copiá-lo
//! verbatim poria **duas entidades a escrever no mesmo documento** — duplicar uma sprite pintada
//! devolvia um sósia que apaga a tinta do original (F4.2).
//!
//! ⚠️ **Saltar era a resposta certa e METADE do trabalho.** Uma peça vetorial saltada não fica
//! *«sem o vínculo»*: ela fica **sem geometria nenhuma** — uma linha na Hierarquia que não desenha
//! um pixel. É exatamente o que uma instância de arte vetorial era antes desta fatia, e é o que a
//! §2.9 do [doc 04] manda curar: *«as peças de uma instância vetorial viram entidades, com
//! `VecPathRef` próprio e `VecPathId` novo»*.
//!
//! ⇒ o que este módulo faz é a outra metade: **clonar o documento** e apontar a cópia para o
//! clone. Ela passa a ter arte própria, que é o que permite ao sync propagar edições da receita
//! como para qualquer outro componente.
//!
//! # ⚠️ O mapa `path ⟺ entidade` entra JUNTO, e não é asseio
//!
//! [`crate::vec_entities::sync`] mantém **uma** entidade por path, nas duas direções, e um path
//! que não esteja no mapa ganha uma entidade **nova** no quadro seguinte. Clonar o path sem
//! registar o par daria à cópia uma entidade fantasma ao lado — a arte apareceria duas vezes na
//! Hierarquia e uma delas seria inalcançável.
//!
//! # ⛔ Os outros TRÊS continuam a ser DROPADOS, e isso é uma decisão
//!
//! Ver [`DROPPED`]. Cada um deles precisa do *store* do módulo dele (o Painter, o 3D, o Flip), e
//! nenhum desses está aqui — clonar às cegas seria pior que dropar. O gate
//! [`tests::every_owned_document_is_cloned_or_declared_dropped`] é um censo de DOIS lados: um
//! bridge novo que não venha a esta lista **não compila o gate**, em vez de nascer mudo.
//!
//! [doc 04]: https://github.com/dibrioli/PH2D/blob/main/docs/Components/04_decisao_arquitetura.md

use ph2d_ecs::{DeepCopy, SimWorld, VecPathRef};
use ph2d_vec_scene::VecScene;

use crate::vec_entities::VecEntityMap;

/// **O documento vetorial e o mapa dele** — o que uma cópia profunda precisa para clonar os paths.
///
/// ⚠️ **Entra na ASSINATURA das duas portas de cópia**, e não num passo que o chamador se lembre de
/// dar a seguir: uma cópia profunda sem os documentos está **incompleta**, e uma invariante que
/// dois sítios têm de lembrar é uma invariante que um deles vai esquecer.
pub(crate) struct OwnedDocs<'a> {
    pub(crate) vec_scene: &'a mut VecScene,
    pub(crate) vec_entities: &'a mut VecEntityMap,
}

/// ⛔ **Os documentos possuídos que a cópia NÃO clona hoje, e porquê.**
///
/// A entidade copiada nasce **sem** eles: uma sprite pintada perde as camadas, uma peça 3D perde
/// os canais assados, um objeto Flip perde os desenhos. É o comportamento de sempre, agora
/// **declarado** — e cada um espera o store do módulo dele.
pub(crate) const DROPPED: &[(&str, &str)] = &[
    (
        "ph2d::ecs::PaintedDoc",
        "o documento em camadas do Painter vive no store dele, fora desta porta",
    ),
    (
        "ph2d::ecs::BakedForm",
        "os canais assados do 3D vivem no módulo de escultura",
    ),
    (
        "ph2d::ecs::FlipObjectRef",
        "o objeto do Flip vive no `FlipDoc`",
    ),
];

/// **O que a clonagem fez, e o que ela DEIXOU CAIR.**
///
/// ⚠️ A segunda metade existe porque *um importador que ignora em silêncio é pior que um que
/// recusa* (a lei do `.ase`): uma sprite pintada que perde as camadas ao ser copiada não tem nada
/// na tela a dizer porquê. O chamador **nomeia a camada** no log.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct DocReport {
    /// Quantos documentos foram de facto clonados.
    pub(crate) cloned: usize,
    /// Os nomes canónicos que a cópia deixou cair, sem repetição e em ordem de [`DROPPED`].
    pub(crate) dropped: Vec<&'static str>,
}

/// ⭐ **Clona os documentos possuídos de uma cópia profunda.**
///
/// ⚠️ **A geometria de um `VecPath` é LOCAL** (ADR-0111: *«a geometria em `VecScene` passa a ser
/// LOCAL, e o afim que a leva ao mundo é `parent_world_transform ∘ Transform`»*), então o clone
/// entra **sem deslocamento**: o `Transform` que a cópia profunda já levou verbatim é que põe a
/// arte no sítio. ⛔ Um `paste_clip` com offset — a porta do *Duplicate* de canvas — moveria a
/// arte DENTRO da cópia, e a peça sairia deslocada da irmã dela.
pub(crate) fn clone_owned_documents(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut OwnedDocs<'_>,
    copy: &DeepCopy,
) -> DocReport {
    let mut out = DocReport::default();
    for (&src, &dst) in &copy.entities {
        if let Some(vp) = sim.world().get::<VecPathRef>(src).copied()
            && let Some(path) = docs.vec_scene.path(vp.0).cloned()
        {
            let new_id = docs.vec_scene.push_path(path);
            sim.world_mut().entity_mut(dst).insert(VecPathRef(new_id));
            docs.vec_entities.insert(new_id, dst.to_bits());
            out.cloned += 1;
        }
        // ⚠️ **O que se perde é NOMEADO** — ver [`DocReport`]. A pergunta é *«a fonte tinha-o?»*,
        // e a resposta sai da vtable do registo: esta shell não conhece os tipos dos outros três.
        for (name, _) in DROPPED {
            if out.dropped.contains(name) {
                continue;
            }
            if registry
                .get_by_name(name)
                .and_then(|e| (e.serialize)(sim.world(), src).ok().flatten())
                .is_some()
            {
                out.dropped.push(name);
            }
        }
    }
    out
}

/// **Um par VAZIO de documentos**, para os gates que não têm arte vetorial.
///
/// ⚠️ Os que têm vivem em [`tests`] e usam um `VecScene` de verdade — este atalho existe para os
/// outros vinte não ganharem duas linhas de cerimónia cada.
#[cfg(test)]
pub(crate) fn empty_docs() -> (VecScene, VecEntityMap) {
    (VecScene::new(), VecEntityMap::new())
}

impl DocReport {
    /// **Diz o que caiu**, uma linha por espécie. No-op quando não caiu nada.
    ///
    /// ⚠️ No `stderr` e não num toast: é diagnóstico de quem lê o log, e um aviso por cada peça de
    /// uma receita grande seria uma parede de toasts sobre um comportamento declarado.
    pub(crate) fn warn(&self, verb: &str) {
        for name in &self.dropped {
            let why = DROPPED
                .iter()
                .find(|(n, _)| n == name)
                .map_or("", |(_, w)| *w);
            eprintln!("[ph2d-instancia] {verb}: `{name}` nao foi copiado — {why}");
        }
    }
}

#[cfg(test)]
#[path = "instance_docs_tests.rs"]
mod tests;
