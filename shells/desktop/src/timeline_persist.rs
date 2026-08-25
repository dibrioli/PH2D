//! **Identidade** dos objetos animados — como uma track reencontra o objeto dela (W4.T6/B5).
//!
//! Os bits de entidade não sobrevivem a um respawn: o restore do undo e o load de projeto
//! recriam tudo com bits NOVOS. Então uma binding não pode ser reconectada por bits — nem
//! através de um Ctrl+Z, nem através de um load. O objeto bound é identificado pelo
//! [`ph2d_ecs::StableId`], que o snapshot v2 **reinsere ao restaurar** (ADR-0164 F1).
//!
//! # A troca de substrato (ADR-0164 F1 passo 5b), e o que ela de facto curou
//!
//! Até esta wave a identidade era o **hash do nome** (`stable_name_id`). ⚠️ **E o plano
//! afirmava que renomear desligava a binding — MEDIDO, era FALSO** (as sondas estão no handoff):
//! a metade viva do [`refresh_and_heal_bindings`] re-carimbava o `wire_id` a partir do nome
//! CORRENTE a cada frame, então um rename era seguido em silêncio, inclusive através do
//! arquivo. *A família da física e a da timeline partilhavam a função e **não** partilhavam o
//! defeito* — a junta guardava o hash e nunca o refrescava; a binding refrescava-o sempre.
//!
//! O que a troca cura é outra coisa, e essa foi medida a vermelho:
//!
//! - ⚠️ **Dois objetos com o MESMO nome faziam a animação DESAPARECER.** O nome deixa de ser
//!   uma resposta (o gate antigo chamava-se `an_ambiguous_name_refuses_to_heal_and_refuses_to`
//!   `_purge`; hoje o par é `two_homonyms_no_longer_hide_the_animation` + a metade legada), a
//!   binding fica
//!   dormente e as rows saem da vista — sem badge, sem erro, sem caminho de volta enquanto o
//!   empate durar. Dois `StableId` nunca empatam: a pergunta *"qual deles?"* deixa de existir.
//! - **Uma pergunta, uma resposta.** A física (F1 passo 5a) e o cache do undo incremental (F2)
//!   respondem *"que objeto é este?"* com o `StableId`. Um terceiro mecanismo para a mesma
//!   pergunta é o que esta fase existe para colapsar.
//!
//! ⚠️ **O LEGADO cura-se sozinho, e tem de o fazer.** Um documento gravado antes desta wave
//! guarda hashes de nome; se o heal só olhasse ids, a binding não resolveria e a **purga** — que
//! corre no mesmo `upkeep` — apagaria a animação do artista. Por isso o mapa do [`upkeep`] leva
//! as DUAS chaves (ver lá), e o re-carimbo escreve o `StableId` por cima: o documento sobe de
//! substrato no primeiro frame depois do load, sem degrau de schema e sem migração escrita.
//!
//! É o mesmo mecanismo em três situações, e é por isso que ele é UM só:
//!
//! - **delete + undo**: o undo global respawna o objeto (bits novos, mesmo `Name`) e a binding
//!   órfã se recola — as rows voltam ([`upkeep`], por-frame).
//! - **load de projeto**: o `TimelineDoc` viaja DENTRO do arquivo de projeto ([`crate::project`],
//!   campo `timeline`) e volta com as bindings **destacadas** ([`install_from_project`]); o
//!   `upkeep` do frame as recola nos objetos que o load acabou de spawnar.
//! - **save**: [`serialize`] carimba a identidade em cada binding antes de gravar.
//!
//! **O sidecar MORREU** (era `ph2d_timeline.postcard` + Ctrl+S/Ctrl+O no contexto do painel).
//! Ele existia porque "não havia save de projeto" — e havia: o bloco GLOBAL de Ctrl+S/Ctrl+O em
//! `keyboard.rs` já retornava antes, então o sidecar era **código morto** que ainda dizia o
//! contrário no comentário. Um 2º formato para o mesmo dado é a coisa que o `project.rs` existe
//! para não ser.

use ph2d_ecs::{Entity, Name, StableId, World};
use ph2d_timeline::{
    TimelineState, WireId, refresh_and_heal_bindings, resolve_entities, stamp_wire_ids,
};

/// The bound entity's stable identity, or `NULL` when it has none.
///
/// `WireId` stays as the timeline's **namespace** for the id — the document's own word for
/// *"the object this binding names"* — while the number inside it is now the scene's
/// [`StableId`]. Collapsing the two types would push `ph2d-ecs` into `ph2d-timeline`, which
/// is a crate that deliberately does not know what a scene is (`persist.rs` takes closures
/// for exactly that reason).
fn wire_of(world: &World, entity_bits: u64) -> WireId {
    // `try_from_bits`: uma binding DESTACADA carrega `entity = 0`, e `0` não é nulo no bevy —
    // o índice é `NonZero<u32>`, então `from_bits(0)` **entra em pânico**. Salvar um projeto
    // logo depois de abrir outro (bindings ainda pendentes de recolagem) chegaria aqui.
    //
    // Sem `StableId` a resposta é NULL, e NULL nunca apaga um id guardado (contrato do
    // `stamp_wire_ids`): quem chama garante os ids ANTES de perguntar — ver [`upkeep`] e
    // [`serialize`], que assinam `&mut World` por isso.
    match Entity::try_from_bits(entity_bits).and_then(|e| world.get::<StableId>(e)) {
        Some(id) if !id.is_none() => WireId(id.0),
        _ => WireId::NULL,
    }
}

/// Per-frame identity upkeep (called by `timeline_bridge::run`, after the apply
/// refreshed the `missing` flags), in two passes over the SAME name map:
///
/// 1. **Heal** — live bindings keep their identity stamped, and missing ones
///    reconnect to the live entity carrying that identity. This is what recolors a
///    project load (`install_from_project` detaches everything) and what makes
///    a timeline-undo of the purge below recoverable.
/// 2. **Purge** — a binding still missing after the heal, whose object is
///    genuinely GONE (no live entity carries its identity), is removed from the
///    document together with its tracks in every clip (Enio, 2026-07-22: *"a
///    timeline precisa ser resetada ao deletar o objeto"*). When the purge
///    removes the LAST animated object, the whole document resets — clips,
///    containers, lanes, loop — because composition authored around an object
///    that no longer exists is exactly the stale state that arrived "totalmente
///    bugada" for the next object created. One timeline-undo step covers it.
///
/// ⚠️ **A ordem heal→purge é load-bearing:** o load de projeto destaca TODA
/// binding (`entity = 0`, `missing`) e conta com o heal DESTE chamado para
/// recolá-las — uma purga que rodasse antes (ou que ignorasse o resultado do
/// heal) apagaria o documento inteiro um frame depois de todo Ctrl+O.
///
/// ⭐ **Duas chaves no mapa, e a ordem entre elas é lei.** O mapa leva a identidade
/// ([`StableId`]) de todo objeto vivo **e** — só para o LEGADO — o hash do nome dele. Um
/// documento gravado antes do ADR-0164 F1 guarda hashes; sem esta segunda chave a binding
/// não resolveria e a purga abaixo apagaria a animação no primeiro frame depois do load.
/// Os hashes entram primeiro e os ids **por cima**, porque só a chave nova é uma resposta:
/// ids são pequenos e sequenciais, hashes FNV-1a são grandes, mas a precedência não é
/// deixada à aritmética — é escrita.
///
/// **Um nome AMBÍGUO não cura NEM purga — recusa** (⚠️ e só a chave LEGADA pode ficar
/// ambígua: dois objetos nunca partilham um `StableId`, que é precisamente o defeito de
/// produto que a troca de substrato curou — ver o cabeçalho do módulo). Se DOIS objetos
/// vivos dividem o nome, "qual deles é o dono desta track?" não tem resposta — curar num
/// deles seria dirigir a pose do objeto errado, e purgar seria destruir trabalho por causa
/// de um empate transitório. A binding fica dormente até o empate acabar.
///
/// Steady state (nothing missing) touches no allocation (HR-3: the bridge's
/// paused path is gated zero-alloc): the map is only built when a missing
/// binding exists to resolve.
///
/// Returns whether the document was **RESET** this frame — the purge removed
/// the last animated object. The bridge reacts (rewind + pause + drop the
/// panel's container trail); nothing here touches a playhead, because this
/// function does not own one. Healing is not counted back: its oracle is the
/// bindings themselves (`entity` bits + `!missing`).
pub(crate) fn upkeep(timeline: &mut TimelineState, world: &mut World) -> bool {
    // ⚠️ **Incondicional, e é o que torna esta função auto-suficiente.** A metade VIVA do heal
    // carimba o id em cada binding, e ela corre mesmo com nada `missing`: um objeto acabado de
    // criar que ainda não tivesse id seria carimbado NULL, e ao morrer a binding dele seria
    // purgada sem identidade nenhuma. É a mesma razão pela qual o `resolve_body_names` da
    // física chama isto em vez de contar com outro sistema. Idempotente e sem alocação quando
    // não há nada a atribuir (`collect` de um iterador vazio não toca no heap).
    ph2d_ecs::assign_missing_stable_ids(world);
    let any_missing = timeline.doc.bindings().iter().any(|b| b.missing);
    // `None` no valor = chave AMBÍGUA — só possível na chave legada (o nome).
    let by_wire: std::collections::BTreeMap<u64, Option<u64>> = if any_missing {
        let mut m: std::collections::BTreeMap<u64, Option<u64>> = std::collections::BTreeMap::new();
        // 1. LEGADO: o hash do nome, para documentos gravados antes do ADR-0164 F1.
        {
            let mut q = world.query::<(Entity, &Name)>();
            for (e, name) in q.iter(world) {
                m.entry(ph2d_ecs::stable_name_id(name.as_str()))
                    .and_modify(|slot| *slot = None) // já havia um: empate
                    .or_insert(Some(e.to_bits()));
            }
        }
        // 2. A identidade, POR CIMA — ver o doc desta função.
        {
            let mut q = world.query::<(Entity, &StableId)>();
            for (e, id) in q.iter(world) {
                if !id.is_none() {
                    m.insert(id.0, Some(e.to_bits()));
                }
            }
        }
        m
    } else {
        std::collections::BTreeMap::new() // alloc-free
    };
    {
        let world = &*world;
        refresh_and_heal_bindings(
            &mut timeline.doc,
            |bits| wire_of(world, bits),
            |w| by_wire.get(&w.0).copied().flatten(),
        );
    }
    purge_the_dead(timeline, &by_wire)
}

/// The purge half of [`upkeep`] — see its docs for the policy. Returns whether
/// the document was fully reset.
fn purge_the_dead(
    timeline: &mut TimelineState,
    by_wire: &std::collections::BTreeMap<u64, Option<u64>>,
) -> bool {
    // Still missing after the heal ⇒ dead, EXCEPT the ambiguous tie (a live
    // duplicate exists — `Some(None)` in the map), which stays dormant. A NULL
    // wire id (a transient that never had a name) can never heal, so it purges.
    let dead: Vec<ph2d_anim::AnimTarget> = timeline
        .doc
        .bindings()
        .iter()
        .filter(|b| b.missing && by_wire.get(&b.wire_id.0) != Some(&None))
        .map(|b| b.target)
        .collect();
    if dead.is_empty() {
        return false;
    }
    // ONE timeline-undo step for the whole purge (reset included): the recovery
    // path after an accidental delete is global Ctrl+Z (the object respawns,
    // same name) + timeline Ctrl+Z (the doc returns, bindings dormant) — and
    // the heal above recolors them on the next frame.
    timeline.history.push(timeline.doc.clone());
    for target in dead {
        timeline.doc.purge_binding(target);
    }
    // Purged targets may be selected; a selection into removed tracks is stale.
    timeline.selection.clear();
    if !timeline.doc.bindings().is_empty() {
        return false; // other objects still animated: their work is untouchable
    }
    // Last animated object gone ⇒ the timeline resets to a fresh **4 s composition** — the
    // boot default (clip 0 + scene authored), NOT a derived-0 doc (Enio, 2026-07-28: *"deletei
    // os objetos da cena, criei outro, mas a timeline ficou com dur infinita — deveria estar com
    // dur 4 e véu visível"*). Without this the next object the artist creates opens the timeline
    // at Dur ∞ / no veil: the reported bug. The display fps is a project-ish setting, not the
    // dead object's animation — it survives.
    let fps = timeline.doc.fps_display;
    timeline.doc = ph2d_timeline::TimelineDoc::new();
    timeline.doc.fps_display = fps;
    timeline
        .doc
        .set_clip_length_override(0, Some(ph2d_timeline::DEFAULT_DURATION_SECONDS));
    timeline
        .doc
        .set_scene_length(Some(ph2d_timeline::DEFAULT_DURATION_SECONDS));
    timeline.edit_path.clear();
    true
}

/// **Publica o NOME de cada objeto que tem track**, para o painel poder rotular uma row
/// (e o card de Expressão titular-se) com *quem* em vez de `#7294` (FASE C.3 do plano 12).
///
/// Mora aqui porque este módulo já é o dono da pergunta *"qual objeto é este?"* — é ele que
/// resolve binding↔objeto pelo `Name` no heal, no save e no load. Uma segunda travessia do
/// mundo noutro arquivo seria a 2ª resposta que o `wire_of` acima existe para não ter.
///
/// ⚠️ **Escopo: as tracks que o snapshot tem, nunca a cena.** A pergunta é sobre as rows que
/// vão ser pintadas; varrer o mundo publicaria centenas de nomes que ninguém lê.
///
/// ⚠️ **Reaproveita as `String`s** (`clear` + `push_str` quando o nome não mudou; nada é
/// realocado em regime) e **poda por varredura das tracks** em vez de montar um conjunto:
/// são poucas rows, e um `Vec` de scratch por frame custaria mais que o laço que ele evitaria.
pub(crate) fn publish_object_names(view: &mut ph2d_timeline::TimelineViewSnapshot, world: &World) {
    let ph2d_timeline::TimelineViewSnapshot {
        tracks,
        object_names,
        ..
    } = view;
    // Poda primeiro: um objeto que saiu das tracks (deletado, ou a track foi removida) não
    // pode continuar nomeando bits que o bevy já reciclou para OUTRO objeto.
    object_names.retain(|bits, _| tracks.iter().any(|t| t.entity == *bits));
    for t in tracks.iter() {
        let Some(name) = Entity::try_from_bits(t.entity).and_then(|e| world.get::<Name>(e)) else {
            // Sem `Name` não há o que publicar — e uma entrada VELHA teria de sair, senão o
            // rótulo mostraria o nome que o objeto tinha antes de perdê-lo.
            object_names.remove(&t.entity);
            continue;
        };
        let slot = object_names.entry(t.entity).or_default();
        if slot != name.as_str() {
            slot.clear();
            slot.push_str(name.as_str());
        }
    }
}

/// Carimba o `wire_id` (a identidade estável do objeto) em cada binding e serializa o
/// documento — os bytes que o arquivo de projeto carrega no campo `timeline`
/// ([`crate::project`]).
///
/// ⚠️ **`&mut World` porque a invariante se impõe na DERIVAÇÃO** — a mesma lei do
/// `world_to_snapshot` (ADR-0164 F1 passo 3). Esta função corre **antes** da captura no
/// `project_save_to` (precisa do mundo vivo), então não pode contar com os ids que a captura
/// atribui: um objeto criado no mesmo frame do Ctrl+S seria gravado com `wire_id` NULL, e a
/// track dele nunca mais reencontraria ninguém. Pedir `&mut` é o que torna isso impossível
/// em vez de improvável.
pub(crate) fn serialize(
    timeline: &mut TimelineState,
    world: &mut World,
) -> Result<Vec<u8>, String> {
    ph2d_ecs::assign_missing_stable_ids(world);
    let world = &*world;
    stamp_wire_ids(&mut timeline.doc, |bits| wire_of(world, bits));
    timeline.doc.to_bytes()
}

/// Instala o documento vindo do arquivo de projeto, com **toda binding DESTACADA**
/// (`entity = 0`, `missing`). Devolve quantas bindings ficaram pendentes de recolagem.
///
/// **Por que destacar, e não resolver aqui.** As bindings chegam com os bits de entidade da
/// sessão em que foram SALVAS — e bits são reciclados: nada impede que os bits gravados de um
/// objeto morto sejam, nesta sessão, os bits de um objeto **diferente e vivo**. Aí a track
/// dirigiria em silêncio a pose do objeto errado, sem nunca ser marcada `missing`. Destacar
/// torna isso impossível: o único caminho de volta é o `wire_id` (o nome).
///
/// Quem recola é o [`upkeep`] do frame — a MESMA função que cura o delete+undo. É de propósito:
/// a resolução por nome existe UMA vez, então o load não pode divergir do undo (e o load não
/// precisa do mundo, o que o torna dirigível sem janela — ver os gates em `project_tests`).
///
/// Selection / history / clipboard não são persistidos: nascem limpos em volta do documento
/// carregado, então um undo depois do load não alcança a sessão anterior.
pub(crate) fn install_from_project(bytes: &[u8]) -> Result<TimelineState, String> {
    if bytes.is_empty() {
        // **Projeto SEM animação abre com o padrão do PRODUTO** (4 s autorados), não com o
        // documento derivado-0 (Enio: *"a duração padrão do clip deve ser 4seg, mas ao abrir a
        // timeline está 0"* / *"o véu deve ficar visível sempre"*). Um Ctrl+O de um arquivo
        // velho (sem timeline) ou de um projeto salvo antes de tocar a animação instalava um
        // `TimelineState::new()` DERIVADO — `scene_length: None`, clips sem override —, e a
        // sessão abria com Dur 0, sem véu, e com as expressões PURAS extrapolando (nada as
        // cortava: `cut_scene`/`clip_cut` só clampam quando há duração autorada). É a MESMA
        // composição de 4 s que o boot instala (`main.rs`), agora também na porta de load.
        return Ok(TimelineState::with_default_duration());
    }
    let mut timeline = TimelineState::new();
    timeline.doc = ph2d_timeline::TimelineDoc::from_bytes(bytes)?;
    // `entity_of` sempre `None` ⇒ `entity = 0` + `missing` em TODA binding (contrato do
    // `resolve_entities`). O `wire_id` — que é o que importa — vem do arquivo, intacto.
    //
    // `0` NÃO é um entity válido no bevy (o índice é `NonZero<u32>`), o que é exatamente o que
    // faz dele um sentinel seguro — e exatamente o que obriga TODO leitor de bits a usar
    // `Entity::try_from_bits`: o `from_bits(0)` **entra em pânico**. O apply do frame seguinte
    // faria isso com cada binding recém-carregada, e o Ctrl+O virava um crash.
    resolve_entities(&mut timeline.doc, |_| None);
    // **Um projeto LEGADO (salvo antes do padrão de 4 s) abre com 4 s + véu, não com Dur 0**
    // (Enio, 2026-07-28: *"deixe a duração em 4 seg e o véu visível mesmo sem nenhum clip na
    // timeline"*). Se o documento carregado não tem NENHUMA duração autorada — nem no clip 0,
    // nem na cena — ele é da era derivada-0, e Autokey num clip desses cai num clip SEM véu (o
    // que o artista reportou). Estampa o mesmo 4 s que o boot e o caso vazio acima instalam.
    //
    // ⚠️ Só dispara na assinatura LEGADA (clip 0 E cena derivados). Todo clip/cena criado a
    // partir de 2026-07-23 já carrega uma duração autorada, então isto nunca clobbera uma
    // composição autorada — nem uma que o artista deixou INFINITA de propósito, porque uma
    // timeline infinita autorada limpa o clip OU a cena de forma DELIBERADA, e o outro escopo
    // segue autorado. Uma timeline com AMBOS derivados nunca foi autorada: é legado.
    if timeline.doc.clip_length_override(0).is_none() && timeline.doc.scene_length.is_none() {
        timeline
            .doc
            .set_clip_length_override(0, Some(ph2d_timeline::DEFAULT_DURATION_SECONDS));
        timeline
            .doc
            .set_scene_length(Some(ph2d_timeline::DEFAULT_DURATION_SECONDS));
    }
    Ok(timeline)
}

#[cfg(test)]
#[path = "timeline_persist_tests.rs"]
mod tests;
