//! ⭐⭐⭐ **OS OSSOS INTELIGENTES, vivos** — girar um osso percorre uma animação inteira.
//!
//! O *Smart Bone* do Moho / *Action Constraint* do Blender (doc em [`ph2d_skeleton_ecs::SmartBone`]);
//! aqui mora o PASSE. O que ele escreve é PRÉ-VISUALIZAÇÃO, pelo ledger do timeline
//! ([`ph2d_timeline_preview`]), e corre ANTES da âncora de IK (a IK vê a pose já corrigida).

use ph2d_ecs::{Entity, Name, SimWorld, StableId, Transform, World};
use ph2d_skeleton_ecs::SmartBone;
use ph2d_timeline::TimelineDoc;

use ph2d_preview_drive::PreviewDrive;

/// ⭐⭐⭐ **O PICK DO ALVO RESOLVEU** — escreve no controlo de `osso` o NOME de `alvo`.
///
/// Uma porta para o clique no canvas e na Hierarquia. `false` (sem `Name`, ou o osso já não é um
/// controlo) ⇒ o chamador mantém o pick armado.
#[must_use]
pub fn set_target(sim: &mut SimWorld, osso: Entity, alvo: Entity) -> bool {
    let Some(nome) = sim
        .world()
        .get::<Name>(alvo)
        .map(|n| n.as_str().to_string())
    else {
        return false;
    };
    let Some(mut sb) = sim.world_mut().get_mut::<SmartBone>(osso) else {
        return false;
    };
    eprintln!("[ph2d-vec] osso inteligente: alvo = \"{nome}\"");
    sb.target = nome;
    true
}

/// ⭐⭐⭐ **AS ACÇÕES QUE ESTE CONTROLO PODE OFERECER** — todas, ou só as que animam o ALVO dele.
///
/// O que filtra é RELEVÂNCIA (a lista é de clips, ≤ [`ph2d_timeline::MAX_CLIPS`] por animação, a
/// do ancestral do osso ou a da Cena). Um filtro que esvaziaria a lista NÃO se aplica. O alvo
/// resolve-se pelo NOME.
#[must_use]
pub fn actions_for(world: &World, doc: &TimelineDoc, sb: &SmartBone, osso: Entity) -> Vec<String> {
    let animacao = doc.animation_of(world, osso);
    let nome = |i: usize| doc.clips()[i].name.clone();
    let todas = || doc.clips_of(animacao).map(nome).collect::<Vec<_>>();
    if sb.target.is_empty() {
        return todas();
    }
    let Some(bits) = world
        .iter_entities()
        .find(|er| er.get::<Name>().is_some_and(|n| n.as_str() == sb.target))
        .map(|er| er.id().to_bits())
    else {
        return todas(); // o objecto foi apagado ou renomeado — filtrar por ele esconderia tudo
    };
    let alvos: Vec<ph2d_anim::AnimTarget> = doc
        .bindings()
        .iter()
        .filter(|b| b.entity == bits)
        .map(|b| b.target)
        .collect();
    let filtradas: Vec<String> = doc
        .clips_of(animacao)
        .filter(|&i| {
            alvos
                .iter()
                .any(|t| doc.clips()[i].clip.track(*t).is_some())
        })
        .map(nome)
        .collect();
    if filtradas.is_empty() {
        todas()
    } else {
        filtradas
    }
}

/// ⭐⭐⭐ **A ACÇÃO NA POSIÇÃO `i` DA LISTA QUE O PAINEL PINTOU.**
///
/// Uma posição só significa algo ao lado da lista que a produziu: quem indexa é a MESMA porta que
/// constrói ([`actions_for`]).
#[must_use]
pub fn action_at(
    world: &World,
    doc: &TimelineDoc,
    sb: &SmartBone,
    osso: Entity,
    i: usize,
) -> Option<String> {
    actions_for(world, doc, sb, osso).into_iter().nth(i)
}

/// ⭐⭐⭐ **O CLIP QUE ESTE OSSO PERCORRE** — o nome dele DENTRO da animação onde o osso vive.
///
/// O nome é único por ANIMAÇÃO (a cópia de um Prefab tem o seu); sem ele o controlo fica inerte.
#[must_use]
pub fn clip_of(world: &World, doc: &TimelineDoc, sb: &SmartBone, osso: Entity) -> Option<usize> {
    doc.clip_named(doc.animation_of(world, osso), &sb.clip)
}

/// ⭐⭐⭐ **ESCOLHER A ACÇÃO NA POSIÇÃO `i` DA LISTA PINTADA** — a porta que o dreno do painel chama.
///
/// Devolve `Some(nome)` quando escreveu, e se essa acção está ABERTA na timeline. Quem indexa é
/// o [`action_at`], nunca `doc.clips().get(i)`.
pub fn choose_action(
    sim: &mut SimWorld,
    doc: &TimelineDoc,
    osso: Entity,
    i: usize,
) -> Option<(String, bool)> {
    let sb = sim.world().get::<SmartBone>(osso)?.clone();
    let nome = action_at(sim.world(), doc, &sb, osso, i)?;
    let escolhido = SmartBone {
        clip: nome.clone(),
        ..sb
    };
    let aberta = clip_of(sim.world(), doc, &escolhido, osso) == Some(doc.active_index());
    sim.world_mut().get_mut::<SmartBone>(osso)?.clip = nome.clone();
    Some((nome, aberta))
}

/// ⭐⭐⭐ **OS CONTROLOS QUE NASCERAM MUDOS** — os ossos inteligentes cujo ângulo é **DERIVADO**.
///
/// Um osso governado por âncora tem a rotação reescrita pelo solver DEPOIS do controlo ⇒ a acção
/// congela. O aviso pende do FACTO, não do verbo, para responder por todas as ordens.
#[must_use]
pub fn governed_controls(sim: &SimWorld) -> Vec<Entity> {
    sim.world()
        .iter_entities()
        .filter(|er| er.get::<SmartBone>().is_some())
        .map(|er| er.id())
        .filter(|e| crate::goal::is_governed(sim, *e))
        .collect()
}

/// ⭐⭐⭐ **ANEXA O CONTROLO VAZIO a este osso.** `true` quando ele ainda não era um.
///
/// Não cria nada (dono 2026-09-08): quem dá sujeito são *Pick Object* e o selector *Action*. O
/// `false` não é recusa a mostrar — existe para o censo afirmar que a 2.ª chamada não mexe.
pub fn add(sim: &mut SimWorld, osso: Entity) -> bool {
    if sim.world().get::<SmartBone>(osso).is_some() {
        return false;
    }
    sim.world_mut()
        .entity_mut(osso)
        .insert(SmartBone::default());
    true
}

/// ⭐⭐⭐ **APAGA o controlo deste osso E DEVOLVE A POSE QUE O ARTISTA AUTOROU.** `true` se havia um.
///
/// O motor foi DESLIGADO, não largou (o `settle` não serve, como em [`crate::goal::remove`]); o
/// preço é a LISTA do clip deste controlo, lida ANTES de o componente sair.
pub fn remove(
    sim: &mut SimWorld,
    doc: &TimelineDoc,
    osso: Entity,
    preview: &mut PreviewDrive,
) -> bool {
    let Some(sb) = sim.world().get::<SmartBone>(osso).cloned() else {
        return false;
    };
    let conduzidas = driven_by(sim.world(), doc, &sb, osso);
    sim.world_mut().entity_mut(osso).remove::<SmartBone>();
    for e in conduzidas {
        for d in ph2d_timeline_preview::DRIVERS {
            preview.release_to_authored(sim, e, d);
        }
    }
    true
}

/// **As entidades que a acção deste controlo anima.**
///
/// `try_from_bits`, nunca `from_bits` (bits gravados); binding pendurada some do censo.
#[must_use]
fn driven_by(world: &World, doc: &TimelineDoc, sb: &SmartBone, osso: Entity) -> Vec<Entity> {
    let Some(c) = clip_of(world, doc, sb, osso).map(|i| &doc.clips()[i]) else {
        return Vec::new();
    };
    let mut bits: Vec<u64> = doc
        .bindings()
        .iter()
        .filter(|b| c.clip.track(b.target).is_some())
        .map(|b| b.entity)
        .collect();
    bits.sort_unstable();
    bits.dedup();
    bits.into_iter()
        .filter_map(|b| {
            let e = Entity::try_from_bits(b)?;
            // ⚠️ **E ela tem de estar VIVA** — repor uma pose numa entidade apagada não é um
            // no-op, é escrever num id que já não existe.
            world.get_entity(e).ok()?;
            Some(e)
        })
        .collect()
}

/// **Os ossos inteligentes da cena**, em ordem determinística.
///
/// A ordem é o [`StableId`], nunca o `to_bits` (a composição tem de ser estável entre sessões).
fn controls(sim: &SimWorld) -> Vec<(StableId, Entity, SmartBone)> {
    let mut out: Vec<(StableId, Entity, SmartBone)> = sim
        .world()
        .iter_entities()
        .filter_map(|er| {
            let sb = er.get::<SmartBone>()?.clone();
            (!sb.clip.is_empty()).then(|| {
                (
                    er.get::<StableId>().copied().unwrap_or(StableId::NONE),
                    er.id(),
                    sb,
                )
            })
        })
        .collect();
    out.sort_by_key(|(id, _, _)| *id);
    out
}

/// ⭐⭐⭐ **UM QUADRO DE OSSOS INTELIGENTES** — devolve quantas propriedades foram escritas.
///
/// Sai cedo quando não há nenhum (sem varredura do mundo por quadro).
pub fn drive(sim: &mut SimWorld, doc: &TimelineDoc, preview: &mut PreviewDrive) -> usize {
    let ossos = controls(sim);
    if ossos.is_empty() {
        return 0;
    }
    let log = std::env::var_os("PH2D_BONE_LOG").is_some();
    // O antes lê-se UMA vez: as acções compõem-se.
    let antes = ph2d_timeline_preview::state_of_bindings(sim.world(), doc);
    let mut feitas = 0;
    // A união dos conduzidos é a mesma porta do [`remove`].
    let mut conduzidas: Vec<Entity> = Vec::new();
    for (_, e, sb) in ossos {
        let Some(t) = sim
            .world()
            .get::<Transform>(e)
            .map(|t| f64::from(t.rotation))
        else {
            continue;
        };
        // ⚠️ **O clip é achado pelo NOME** — a lei da referência durável desta casa. Um clip
        // apagado ou renomeado deixa o controlo inerte, e isso é a resposta certa: inventar um
        // índice poria o osso a percorrer a animação do vizinho.
        let Some(i) = clip_of(sim.world(), doc, &sb, e) else {
            if log {
                eprintln!(
                    "[bone] osso inteligente de {e:?}: nao ha clip chamado \"{}\" -- inerte",
                    sb.clip
                );
            }
            continue;
        };
        // ⭐⭐⭐ **UM CONTROLO NÃO PERCORRE A ACÇÃO QUE ESTÁ ABERTA** — ali o artista GRAVA-A.
        if i == doc.active_index() {
            if log {
                eprintln!(
                    "[bone] osso inteligente de {e:?}: a accao \"{}\" esta' ABERTA na timeline -- \
                     em edicao, logo inerte",
                    sb.clip
                );
            }
            continue;
        }
        // ⚠️⚠️ **`clip_end_seconds`, e NUNCA o `duration()` cru.** Um clip novo tem `duration = 0`.
        conduzidas.extend(driven_by(sim.world(), doc, &sb, e));
        let dur = doc.clip_end_seconds(i);
        let quando = ph2d_skeleton::action_time(t, sb.from, sb.to, dur);
        let escritas = ph2d_timeline::apply_one_clip(sim.world_mut(), doc, i, quando);
        if log {
            // ⭐⭐⭐ **A LINHA QUE RESPONDE A PERGUNTA INTEIRA** — as seis causas do controlo mudo.
            let tracks = doc.clips()[i].clip.tracks().len();
            eprintln!(
                "[bone] osso inteligente {e:?}: accao \"{}\" (clip {i}, activo {}) | rot {:+.2}° \
                 faixa {:+.1}..{:+.1}° | dur {dur:.3}s -> t {quando:.3}s | escreveu {escritas} de \
                 {tracks} propriedade(s)",
                sb.clip,
                doc.active_index(),
                t.to_degrees(),
                sb.from.to_degrees(),
                sb.to.to_degrees(),
            );
            if escritas == 0 && tracks > 0 {
                eprintln!(
                    "[bone]   ⛔ a accao TEM {tracks} propriedade(s) e nenhuma foi escrita -- o \
                     objecto que ela anima nao resolve nesta sessao (apagado, ou re-criado com \
                     bits novos sem identidade estavel)"
                );
            }
            if tracks == 0 {
                eprintln!(
                    "[bone]   ⛔ a accao esta' VAZIA -- grave chaves nela na timeline (mova o \
                     cursor de tempo e mexa no objecto)"
                );
            }
        }
        feitas += escritas;
    }
    // ⭐ E a declaração é UMA, no fim: o ledger compara o antes de todos com o depois de todos.
    ph2d_timeline_preview::declare_timeline_writes(sim.world(), &antes, preview);
    // ⭐⭐⭐ **E OS CONTROLOS DIZEM QUE AINDA ESTÃO A CONDUZIR, mesmo com o valor parado.**
    for e in conduzidas {
        for d in ph2d_timeline_preview::DRIVERS {
            preview.still_driving(e, d);
        }
    }
    if log && feitas > 0 {
        eprintln!("[bone] ossos inteligentes: {feitas} propriedade(s) escritas");
    }
    feitas
}

#[cfg(test)]
#[path = "smart_tests.rs"]
mod tests;

/// ⭐ **A LISTA de acções que o painel pinta** — irmão pelo teto de 600 LOC, e o corte é por
/// RESPONSABILIDADE: o `tests` mede *o controlo PERCORRE a acção*; este mede **quais** acções ele
/// oferece e o que uma POSIÇÃO nessa lista significa.
#[cfg(test)]
#[path = "smart_list_tests.rs"]
mod list_tests;
