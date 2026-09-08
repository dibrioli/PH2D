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

use ph2d_ecs::{Entity, SimWorld, StableId, Transform};
use ph2d_skeleton_ecs::SmartBone;
use ph2d_timeline::TimelineDoc;

use crate::preview_drive::PreviewDrive;

/// ⭐⭐⭐ **O NOME DA ACÇÃO DE UM OSSO** — *«Bone 7 Action»*, e *«Bone 7 Action 2»* se aquele já
/// existir.
///
/// ⚠️⚠️ **Ela existe porque *Add Smart Bone* CRIA a acção, e não adopta a aberta** (report do dono,
/// 2026-09-08: *«não há meios de selecionar nem o objeto alvo nem a animação»*). Um documento novo
/// nasce com **uma** acção chamada `"Main"` ⇒ adoptar a aberta casava todo osso inteligente com a
/// animação principal da cena, em silêncio. É a lei do Moho (*Create Smart Bone Action* nasce com o
/// nome do osso) — e o nome é a referência durável desta casa, logo ele tem de ser **único**: dois
/// clips homónimos seriam o mesmo sujeito para o `drive`, que os procura por nome.
///
/// ⚠️ O sufixo começa em `2` porque o primeiro **não** o leva: *«Bone 7 Action»* e *«Bone 7 Action
/// 1»* lado a lado leem-se como uma lista que perdeu o zero.
#[must_use]
pub(crate) fn fresh_action_name(doc: &TimelineDoc, bone: &str) -> String {
    let base = format!("{bone} Action");
    let tomado = |n: &str| doc.clips().iter().any(|c| c.name == n);
    if !tomado(&base) {
        return base;
    }
    // ⚠️ O tecto é o `MAX_CLIPS + 2` e não um número solto: acima dele o documento já recusa o
    // clip, então procurar mais longe seria escolher um nome que ninguém vai poder usar.
    (2..=ph2d_timeline::MAX_CLIPS + 2)
        .map(|i| format!("{base} {i}"))
        .find(|n| !tomado(n))
        .unwrap_or(base)
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
        feitas += ph2d_timeline::apply_one_clip(sim.world_mut(), doc, i, quando);
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
