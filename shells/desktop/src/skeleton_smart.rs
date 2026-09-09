//! ⭐⭐⭐ **OS OSSOS INTELIGENTES, vivos** — girar um osso percorre uma animação inteira.
//!
//! É o *Smart Bone* do Moho e o *Action Constraint* do Blender. O mecanismo, o porquê da forma e as
//! referências estão no doc do [`ph2d_skeleton_ecs::SmartBone`]; aqui mora o **passe**, que é o que
//! só existe com um mundo ECS e um documento de timeline.
//!
//! # ⚠️ O que ele escreve é PRÉ-VISUALIZAÇÃO
//!
//! A mesma lei da âncora de IK: *o documento é o valor **AUTORADO** — o ângulo do osso de controlo e
//! as chaves do clip; o que a acção escreve agora **vê-se, não se guarda nem se desfaz**.* Sem isto,
//! cada clique com um controlo fora do repouso empilharia um passo de undo cujo conteúdo é *«a acção
//! correu»*.
//!
//! ⭐ E o ledger é o **do timeline** ([`crate::timeline_preview`]), não um novo: o que a acção
//! escreve é literalmente o que a timeline escreveria, e aquela porta já cobre os **quatro** factos
//! (pose · alfa da sprite · `t` do morph · params de junta). ⛔ Um ledger próprio poria dois memos
//! sobre o mesmo componente, e quem ganharia seria a ordem do `BTreeMap` — o defeito que o
//! `skeleton_goal` já nomeia no cabeçalho dele.
//!
//! # ⚠️ A ORDEM no quadro, e por que ela é esta
//!
//! O passe corre **antes** da âncora de IK. Um osso inteligente move a pose de base (ele é a
//! correcção autorada); a IK é a restrição que persegue um alvo e tem de ver a pose já corrigida.
//! ⛔ Ao contrário, a IK resolveria sobre uma pose que a acção ainda vai mudar, e o alvo deixaria de
//! ser alcançado no mesmo quadro.

use ph2d_ecs::{Entity, Name, SimWorld, StableId, Transform, World};
use ph2d_skeleton_ecs::SmartBone;
use ph2d_timeline::TimelineDoc;

use crate::preview_drive::PreviewDrive;

/// ⭐⭐⭐ **O PICK DO ALVO RESOLVEU** — escreve no controlo de `osso` o NOME de `alvo`.
///
/// ⚠️⚠️ **UMA porta para DUAS maneiras de achar o objecto**, e é o que impede as duas de divergirem:
/// o clique no **canvas** resolve por hit-test (modal, ele consome o press) e o clique na
/// **Hierarquia** resolve porque a selecção mudou. *Como se acha* é genuinamente diferente; *o que
/// se faz com o que se achou* tem de ser uma coisa só.
///
/// Devolve `false` quando o alvo não tem `Name` ou o osso já não é um controlo — nos dois casos o
/// chamador mantém o pick armado, porque desarmar sobre uma recusa lê-se como *«funcionou»*.
#[must_use]
pub(crate) fn set_target(sim: &mut SimWorld, osso: Entity, alvo: Entity) -> bool {
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
/// ⚠️⚠️ **Report do dono (2026-09-08):** *«se ouverem milhares de objetos animado a lista action
/// fica impossível. Só deve aparecer as animações relacionadas ao objeto selecionado»*. ⚠️ **A
/// medição corrige metade da premissa e confirma a outra:** a lista não cresce com os OBJECTOS — ela
/// lista **clips**, e o documento recusa mais que [`ph2d_timeline::MAX_CLIPS`] (`16`). O que ela
/// ganha aqui não é tamanho, é **relevância**: *quais destas animações tocam este objecto* é a única
/// pergunta que separa as `16`.
///
/// ⚠️⚠️ **UM filtro que esvaziaria a lista NÃO se aplica**, e é lei, não conforto. Um alvo vazio, um
/// alvo cujo objecto foi apagado, ou um objecto que nunca foi animado dariam um selector com zero
/// opções — *um controlo que só sabe recusar é pior que um controlo ausente*, e ali o artista não
/// teria gesto nenhum que o curasse (o alvo escolhe-se no canvas, não na lista).
///
/// ⚠️ O alvo resolve-se pelo **NOME** (a referência durável desta casa), o que custa uma varredura
/// do mundo — paga só quando um controlo está em foco, que é um gesto interactivo.
#[must_use]
pub(crate) fn actions_for(world: &World, doc: &TimelineDoc, sb: &SmartBone) -> Vec<String> {
    let todas = || {
        doc.clips()
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<_>>()
    };
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
        .clips()
        .iter()
        .filter(|c| alvos.iter().any(|t| c.clip.track(*t).is_some()))
        .map(|c| c.name.clone())
        .collect();
    if filtradas.is_empty() {
        todas()
    } else {
        filtradas
    }
}

/// ⭐⭐⭐ **A ACÇÃO NA POSIÇÃO `i` DA LISTA QUE O PAINEL PINTOU.**
///
/// ⛔⛔ **Ela existe por um report do dono** (2026-09-08: *«ao selecionar na lista de actions … não
/// consegue selecionar o clip desejado»*), e o defeito foi meu, na mesma jornada: o clique devolve
/// uma **POSIÇÃO** no pool de ids, e a shell resolvia-a contra `doc.clips()` — a lista **INTEIRA**.
/// Isso estava certo enquanto o painel mostrava todas; deixou de estar no instante em que o alvo
/// passou a **FILTRAR**. Com um filtro activo, escolher a 1.ª linha escrevia o 1.º clip do
/// documento, que é outra coisa.
///
/// ⇒ *uma posição só significa alguma coisa ao lado da lista que a produziu*, e por isso quem
/// indexa é a MESMA porta que constrói ([`actions_for`]) — não uma segunda leitura que concorde por
/// acidente enquanto ninguém filtrar.
#[must_use]
pub(crate) fn action_at(
    world: &World,
    doc: &TimelineDoc,
    sb: &SmartBone,
    i: usize,
) -> Option<String> {
    actions_for(world, doc, sb).into_iter().nth(i)
}

/// ⭐⭐⭐ **ESCOLHER A ACÇÃO NA POSIÇÃO `i` DA LISTA PINTADA** — a porta que o dreno do painel chama.
///
/// Devolve `Some(nome)` quando escreveu, e diz se essa acção está **ABERTA** na timeline (o
/// chamador avisa; ver o report de 2026-09-08).
///
/// ⛔⛔ **Ela existe porque o gate estava do lado errado da porta** (auditoria de 2026-09-08): a lei
/// *«uma posição só significa alguma coisa ao lado da lista que a produziu»* estava gateada sobre o
/// [`action_at`], e o defeito do report vivia no **chamador** — trocar a chamada por
/// `doc.clips().get(i)` deixava a suíte inteira verde e o defeito voltava ao bit. *Um gate sobre a
/// porta não cobre quem a ignora* — que é, letra por letra, a lição que esta linha escreveu no gate
/// do gizmo e não aplicou aqui.
pub(crate) fn choose_action(
    sim: &mut SimWorld,
    doc: &TimelineDoc,
    osso: Entity,
    i: usize,
) -> Option<(String, bool)> {
    let sb = sim.world().get::<SmartBone>(osso)?.clone();
    let nome = action_at(sim.world(), doc, &sb, i)?;
    let aberta = doc
        .clips()
        .get(doc.active_index())
        .is_some_and(|c| c.name == nome);
    sim.world_mut().get_mut::<SmartBone>(osso)?.clip = nome.clone();
    Some((nome, aberta))
}

/// ⭐⭐⭐ **APAGA o controlo deste osso E DEVOLVE A POSE QUE O ARTISTA AUTOROU.** `true` se havia um.
///
/// ⛔⛔ **Achado da auditoria de 2026-09-08 (o irmão já o fazia e este não):** o *Remove Smart Bone*
/// só tirava o componente, e o objecto ficava **assado no instante da acção** em que o controlo o
/// tinha deixado. É letra por letra o report de 2026-09-07 sobre o *Remove IK* — *«não funciona
/// plenamente»* — noutro verbo, e contradiz a lei que este módulo escreveu: *o que um motor escreve
/// vê-se, não se guarda*.
///
/// ⚠️ **A [`crate::preview_drive::PreviewDrive::settle`] NÃO serve** (a mesma nota do
/// [`crate::skeleton_goal::remove`]): ela é para um motor que **largou**, e aí o vivo *é* o
/// documento; aqui o motor foi **desligado**, e o vivo é dele.
///
/// # ⚠️ Por que o preço é uma LISTA e não uma corrente
///
/// A âncora larga a **corrente** que ela governava — uma cadeia de ossos, que ela sabe nomear. Um
/// controlo conduz o que a **ACÇÃO** dele anima, que é `N` objectos × os quatro factos que uma
/// curva escreve ([`crate::timeline_preview::DRIVERS`]). ⛔ Largar «tudo o que o ledger tem» apagaria
/// a reprodução da própria timeline, que partilha aqueles motores — por isso a lista sai do **clip
/// deste controlo**, e de mais nada.
///
/// ⚠️ **As entidades lêem-se ANTES de o componente sair**, pela razão do irmão: depois dele o nome
/// da acção já não está em lado nenhum.
pub(crate) fn remove(
    sim: &mut SimWorld,
    doc: &TimelineDoc,
    osso: Entity,
    preview: &mut PreviewDrive,
) -> bool {
    let Some(sb) = sim.world().get::<SmartBone>(osso).cloned() else {
        return false;
    };
    let conduzidas = driven_by(sim.world(), doc, &sb);
    sim.world_mut().entity_mut(osso).remove::<SmartBone>();
    for e in conduzidas {
        for d in crate::timeline_preview::DRIVERS {
            preview.release_to_authored(sim, e, d);
        }
    }
    true
}

/// **As entidades que a acção deste controlo anima.**
///
/// ⚠️ **`try_from_bits`, NUNCA `from_bits`** — os bits vêm de uma binding gravada, e o `from_bits`
/// **aborta o processo** com bits de outra sessão. É a lei que a `ph2d-timeline` escreve duas vezes.
///
/// ⚠️ Uma binding pendurada (o objecto foi apagado) some do censo, como no `state_of_bindings`: não
/// há pose para lhe devolver.
#[must_use]
fn driven_by(world: &World, doc: &TimelineDoc, sb: &SmartBone) -> Vec<Entity> {
    let Some(c) = doc.clips().iter().find(|c| c.name == sb.clip) else {
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
/// ⚠️ **A ordem é o [`StableId`], nunca o `to_bits`** — dois controlos que percorram acções que
/// escrevem o MESMO objecto compõem-se por ordem, e um desempate por id de alocação mudaria a pose
/// entre sessões. É a mesma lei da agenda das âncoras.
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
/// ⚠️ **Sai cedo quando não há nenhum**, e a guarda é a primeira coisa: sem ela toda cena pagaria
/// uma varredura do mundo por quadro para descobrir que não tem controlo nenhum. É a mesma lei que o
/// passe da âncora já segue, e o preço dela lá está medido em `0,098 %` de um quadro.
pub(crate) fn drive(sim: &mut SimWorld, doc: &TimelineDoc, preview: &mut PreviewDrive) -> usize {
    let ossos = controls(sim);
    if ossos.is_empty() {
        return 0;
    }
    let log = std::env::var_os("PH2D_BONE_LOG").is_some();
    // ⚠️ **O estado ANTES é lido UMA vez, para todos** — as acções compõem-se, e fotografar entre
    // cada uma faria o memo do segundo controlo guardar o que o primeiro escreveu (que é
    // pré-visualização, não o autorado).
    let antes = crate::timeline_preview::state_of_bindings(sim.world(), doc);
    let mut feitas = 0;
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
        let Some(i) = doc.clips().iter().position(|c| c.name == sb.clip) else {
            if log {
                eprintln!(
                    "[bone] osso inteligente de {e:?}: nao ha clip chamado \"{}\" -- inerte",
                    sb.clip
                );
            }
            continue;
        };
        // ⭐⭐⭐ **UM CONTROLO NÃO PERCORRE A ACÇÃO QUE ESTÁ ABERTA** — ali o artista está a
        // GRAVÁ-LA, e não a vê-la correr. É a lei do Moho: dentro de uma *smart bone action* o
        // relógio é o do editor, não o ângulo do osso.
        //
        // ⛔⛔ **Sem isto a feature é inutilizável, e não é uma nicety:** os dois escrevem o mesmo
        // objecto no mesmo quadro e o controlo corre DEPOIS do passe da timeline ⇒ arrastar o
        // playhead não move nada (o ângulo repõe sempre o mesmo instante) e a pose que o artista
        // acabou de pôr é reposta antes de ele a ver. ⚠️ E o autokey corre ainda mais tarde: com o
        // objecto seleccionado, ele leria a saída do próprio controlo como *«o artista mexeu»* e
        // cunharia chaves a partir dela — um laço fechado.
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
        // ⚠️⚠️ **`clip_end_seconds`, e NUNCA o `duration()` cru.** Um clip acabado de criar nasce
        // com `duration = 0` e o artista grava as chaves sem lhe tocar: multiplicar por esse zero
        // deixa a acção presa no instante `0` para todo ângulo — o controlo pinta, o gate de
        // escrita conta `1` e nada se move. **Medido** ao construir este passe (`dur = 0.0`,
        // `feitas = 1`, `x = 0`).
        //
        // ⭐ A porta certa já existia e responde à pergunta inteira: ela lê o override de duração,
        // senão a extensão das CHAVES, e trata o clip só-de-expressão. *A grandeza crua e a
        // pergunta têm nomes parecidos e respostas diferentes.*
        let dur = doc.clip_end_seconds(i);
        let quando = ph2d_skeleton::action_time(t, sb.from, sb.to, dur);
        let escritas = ph2d_timeline::apply_one_clip(sim.world_mut(), doc, i, quando);
        if log {
            // ⭐⭐⭐ **A LINHA QUE RESPONDE A PERGUNTA INTEIRA** — report do dono (2026-09-08:
            // *«tudo configurado e a animação não rodou»*). ⚠️ Um controlo mudo tem **seis** causas
            // que se leem exactamente igual de fora, e a linha nomeia todas de uma vez: o ângulo,
            // a faixa, a duração (um clip sem chaves dá `0` e prende a acção no instante zero), o
            // instante derivado, e **quantas** propriedades a acção conseguiu escrever contra
            // quantas ela TEM. *Sem a segunda metade dessa razão, `0 escritas` não distingue «o
            // clip está vazio» de «o objecto que ele anima já não existe».*
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
    crate::timeline_preview::declare_timeline_writes(sim.world(), &antes, preview);
    if log && feitas > 0 {
        eprintln!("[bone] ossos inteligentes: {feitas} propriedade(s) escritas");
    }
    feitas
}

#[cfg(test)]
#[path = "skeleton_smart_tests.rs"]
mod tests;

/// ⭐ **A LISTA de acções que o painel pinta** — irmão pelo teto de 600 LOC, e o corte é por
/// RESPONSABILIDADE: o `tests` mede *o controlo PERCORRE a acção*; este mede **quais** acções ele
/// oferece e o que uma POSIÇÃO nessa lista significa.
#[cfg(test)]
#[path = "skeleton_smart_list_tests.rs"]
mod list_tests;
