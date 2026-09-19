//! ⭐⭐⭐ **O QUE *Reset Transform* QUER DIZER NESTA LINHA** — o dreno do verbo, e as três respostas.
//!
//! ⚠️ **Corte por RESPONSABILIDADE, imposto pelo tecto de 200 LOC** (`209`), e o irmão certo é este:
//! o `hierarchy_delete` e o `hierarchy_duplicate` já vivem assim. ⛔ **Nunca uma entrada nova no
//! `FN_OVERAGE_OK`** — ele só desce.
//!
//! ⛔⛔⛔ **O defeito que este ficheiro cura está MEDIDO** (`ph2d_skeleton_live::
//! sonda_do_reset_na_hierarquia_tests`): a tabela do menu de contexto da Hierarquia é **plana** —
//! ela não sabe o que a linha é —, e `*t = Transform::IDENTITY` sobre um osso movia a arte presa
//! **26,48 unidades num desenho de 60**, com uma mensagem **verde** a dizer que correu bem. Num
//! osso a **direcção** mora na `rotation` e a **posição** na `translation`.

use super::*;
use ph2d_i18n::tr;

/// Aplica o verbo à linha, se houver linha. Devolve `true` quando o título tem de ser remarcado.
///
/// ⚠️ **A decisão vive na CRATE e não neste `match`** (`pose_de_repouso::repor_transformacao`): ela
/// é a mesma pergunta em três superfícies — este menu, o botão do painel do esqueleto e amanhã um
/// atalho —, e a porta faz o trabalho e devolve o **veredito**. *Quem lê o veredito lê-o para
/// FALAR, nunca para decidir.*
pub(super) fn drain(
    row: Option<NodeId>,
    hero_live: Option<&super::HeroLive>,
    sim: &mut SimWorld,
    toasts: &mut ph2d_editor_core::ToastQueue,
) -> bool {
    let Some(row) = row else {
        return false;
    };
    let Some(entity_bits) = hero_live.and_then(|live| live.bridge.entity_for(row)) else {
        return false;
    };
    let entity = ph2d_ecs::Entity::from_bits(entity_bits);
    match ph2d_skeleton_live::pose_de_repouso::repor_transformacao(sim, entity) {
        ph2d_skeleton_live::pose_de_repouso::Reposicao::Reposta { ossos } => {
            toasts.push(Toast::success(ph2d_i18n::tr_with(
                "shell.hierarchy.bone_back_to_rest",
                &[("n", &ossos)],
            )));
            true
        }
        // ⚠️ **Recusa INERTE e em voz alta:** um osso sem repouso guardado **não** cai de volta na
        // identidade — era por aí que o defeito voltaria para todo rig anterior a esta wave.
        ph2d_skeleton_live::pose_de_repouso::Reposicao::SemRepouso => {
            toasts.push(Toast::warning(tr("shell.hierarchy.bone_has_no_rest")));
            false
        }
        // ⭐ E o resto do app fica **ao bit** como sempre: uma sprite, um grupo, um corpo de física
        // continuam a ir para a identidade.
        ph2d_skeleton_live::pose_de_repouso::Reposicao::NaoEOsso => {
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity) {
                *t = Transform::IDENTITY;
                toasts.push(Toast::info(tr("shell.hierarchy.transform_reset")));
                return true;
            }
            false
        }
    }
}
