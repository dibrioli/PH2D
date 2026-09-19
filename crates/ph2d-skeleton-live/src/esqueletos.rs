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
///
/// ⚠️⚠️ **LIMITE MEDIDO, necessário-mas-não-suficiente:** o envelope também é **inerte** numa forma
/// presa a **UM** osso — os pesos renormalizam e o único osso leva sempre a fatia inteira (medido:
/// `[1.0]` a `strength = 0,3` **e** a `4,0`; com **três** ossos vai de `[1, 0, 0]` a
/// `[0,42, 0,58, 0]`). *O alcance só decide quando DOIS ossos disputam o mesmo ponto.* ⛔ Contar
/// essa disputa por ponto seria caro e frágil ⇒ esta porta esconde o caso **claro** e mostra o
/// resto, que é o lado conservador.
///
/// ⚠️ **E a pergunta por CENA foi APAGADA, não guardada:** ela ficou sem chamador no instante em que
/// esta nasceu, e *uma lei viva que nenhum gesto consulta é uma lei órfã*.
///
/// ⛔⛔⛔ **E ELA É POR OSSO, não por cena — a 1.ª redacção errou por uma premissa MINHA que caiu.**
/// Eu escrevi que *«o `SkinBind` guarda a malha e os pesos, **não** a que ossos ficou preso»*, e ele
/// guarda: cada [`ph2d_skeleton_ecs::Tendon`] carrega o `StableId` do osso. ⇒ a pergunta larga
/// («a CENA tem alguma forma vectorial?») acendia a mancha em **todos** os ossos de uma cena mista,
/// e foi a cena que o dono pediu — a que mostra as duas mídias lado a lado — que a expôs.
#[must_use]
pub fn o_envelope_deste_osso_manda(sim: &SimWorld, osso: ph2d_ecs::Entity) -> bool {
    let mundo = sim.world();
    let Some(id) = mundo.get::<ph2d_ecs::StableId>(osso).copied() else {
        // Sem identidade durável nenhum tendão o pode nomear — e a resposta conservadora é SIM,
        // para nunca esconder um controlo vivo num osso acabado de nascer.
        return true;
    };
    let Some(mut q) = mundo.try_query::<(ph2d_ecs::Entity, &ph2d_skeleton_ecs::SkinBind)>() else {
        return false;
    };
    q.iter(mundo).any(|(e, b)| {
        !crate::skin_image::is_skinned_image(mundo, e) && b.tendons.iter().any(|t| t.bone == id)
    })
}

#[must_use]
pub fn influence_radius(sim: &SimWorld, bits: u64) -> Option<f64> {
    let e = Entity::from_bits(bits);
    let forca = sim.world().get::<Bone>(e)?.strength;
    let (_, a, b) = crate::skin_live::bone_segments(sim)
        .into_iter()
        .find(|(x, _, _)| *x == bits)?;
    Some((b[0] - a[0]).hypot(b[1] - a[1]) * forca.max(0.0))
}

/// **A região de influência de um osso** — `(raio, origem, ponta)` em MUNDO, para o overlay.
pub fn influence_region(sim: &SimWorld, bits: u64) -> Option<(f64, [f64; 2], [f64; 2])> {
    // ⭐⭐⭐ **A MANCHA SÓ EXISTE ONDE O ENVELOPE MANDA** (report do dono, 2026-09-18: *«o gizmo do
    // envelope fica sempre visível mesmo quando não é usado?»* — sim, ficava).
    //
    // ⛔⛔ Com os pesos do **padrão-ouro** uma imagem deforma igual a `1` e a `2` (medido, coluna a
    // coluna), logo num rig só de imagens esta região desenhava *«até onde este osso alcança»* sobre
    // uma lei que **não usa alcance nenhum**. ⚠️ E o envelope não morreu — **mudou de dono**: uma
    // forma vectorial presa continua na lei euclidiana, e ali a mancha diz a verdade.
    //
    // ⭐⭐ **A lei entra AQUI e não em quem desenha, porque esta porta tem DOIS consumidores** — o
    // desenho da mancha e o **hit-test da alça**. Curar só o pintor deixaria o artista a arrastar
    // uma alça invisível, que é pior do que a mancha a mais.
    let e = Entity::try_from_bits(bits)?;
    if !crate::esqueletos::o_envelope_deste_osso_manda(sim, e) {
        return None;
    }
    let r = influence_radius(sim, bits)?;
    let (_, a, b) = crate::skin_live::bone_segments(sim)
        .into_iter()
        .find(|(x, _, _)| *x == bits)?;
    Some((r, a, b))
}
