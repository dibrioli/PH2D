//! ⭐ **QUANTOS ESQUELETOS HÁ, E QUAL É A RAIZ DE UM OSSO** — a topologia da árvore de ossos, numa
//! porta só.
//!
//! ⚠️ **Ela existe porque a subida à raiz passou a ter DOIS leitores:** o [`skeleton_of`], que já a
//! fazia à mão, e o [`crate::recusa_do_bind`], que precisa de contar esqueletos independentes.
//! *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é*, e este repo já a pagou
//! três vezes (o traço uniforme, a chave da ordem das raízes, a média do anel).
//!
//! ⚠️ **Ela vive aqui e não no `skin_live.rs`**, que estava a `697` linhas de um tecto de `700`:
//! mover a subida para cá **tira** linhas de lá em vez de as somar.

use ph2d_ecs::{ChildOf, Entity, SimWorld};
use ph2d_skeleton_ecs::Bone;

/// **A raiz do esqueleto a que este osso pertence.**
///
/// ⚠️ Sobe enquanto o **pai também for osso** — parar no primeiro pai não-osso é o que permite
/// pendurar um esqueleto inteiro dentro de um grupo sem ele deixar de ser um esqueleto.
#[must_use]
pub fn raiz_do_osso(sim: &SimWorld, osso: Entity) -> Entity {
    let mut raiz = osso;
    while let Some(p) = sim.world().get::<ChildOf>(raiz).map(ChildOf::parent) {
        if sim.world().get::<Bone>(p).is_some() {
            raiz = p;
        } else {
            break;
        }
    }
    raiz
}

/// **As raízes dos esqueletos da cena** — uma por esqueleto independente, ordenadas.
///
/// ⚠️ **Contar RAÍZES e não ossos** é o que distingue *«uma cadeia de três»* de *«três esqueletos»*.
#[must_use]
pub fn bone_roots(sim: &SimWorld) -> Vec<Entity> {
    let mut out: Vec<Entity> = crate::skin_live::ossos_da_cena(sim)
        .into_iter()
        .map(|(e, _)| raiz_do_osso(sim, e))
        .collect();
    out.sort_by_key(|e| e.to_bits());
    out.dedup();
    out
}

#[cfg(test)]
#[path = "esqueletos_tests.rs"]
mod tests;

/// ⭐⭐⭐ **O ENVELOPE AINDA MANDA EM ALGUMA COISA NESTA CENA?** (report do dono, 2026-09-18:
/// *«Por que o envelope já não influencia na deformação?»*).
///
/// ⛔⛔⛔ **Ele deixou de mandar numa IMAGEM, e isso está MEDIDO há waves** (a tabela vive no doc do
/// [`crate::skin_image`] via a cena do pincel): os pesos do **padrão-ouro** são resolvidos *sobre a
/// arte* e não sobre um raio, logo `strength = 1` e `strength = 2` dão a MESMA deformação, coluna a
/// coluna. ⚠️ **O envelope não morreu — MUDOU DE DONO:** uma forma **vectorial** presa ao mesmo
/// esqueleto continua na lei euclidiana (o padrão-ouro precisa de uma malha do domínio, e uma
/// Bézier não tem uma), e ali ele manda como sempre.
///
/// ⇒ o painel pintava um número que, num rig só de imagens, **não muda um pixel** — a espécie de
/// controlo morto que o `§5.0` nomeia, e que o dono apanhou perguntando.
///
/// ⚠️ **A pergunta é da CENA e não do osso, de propósito:** o `SkinBind` guarda a malha e os pesos,
/// **não a que ossos ficou preso** — logo *«este esqueleto tem forma vectorial?»* não é derivável
/// daqui. A pergunta mais larga erra sempre para o lado **conservador**: com uma forma vectorial
/// presa em qualquer sítio, o campo fica à vista. *Esconder um controlo vivo é pior do que mostrar
/// um inerte.*
#[must_use]
pub fn ha_forma_vectorial_presa(sim: &SimWorld) -> bool {
    let mundo = sim.world();
    let Some(mut q) = mundo.try_query::<(ph2d_ecs::Entity, &ph2d_skeleton_ecs::SkinBind)>() else {
        return false;
    };
    q.iter(mundo)
        .any(|(e, _)| !crate::skin_image::is_skinned_image(mundo, e))
}
