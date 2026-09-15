//! **Fase do quadro: AS EDIÇÕES DA FÁBRICA E DO CICLO DE VIDA** (TOP-20 #11 e #12, W3) —
//! fase-filha do [`super`], num ficheiro irmão.
//!
//! ⚠️ **O corte foi imposto pelo tecto de LOC e está certo por outra razão:** estas edições pedem
//! a **árvore de tags** (o campo do ponto de nascimento guarda uma IDENTIDADE e o painel mostra um
//! CAMINHO), e são as únicas do Inspector, com as das tags, a precisarem dela. *Partir por
//! responsabilidade, nunca subir o número.*

use ph2d_ecs::SimWorld;
use ph2d_editor_core::FactoryFieldEdit;

/// Aplica as edições que o painel emitiu neste quadro. `true` = o documento mudou.
///
/// ⚠️ **A conversão nome ⟷ identidade é do [`super::inspector_factory`]**, e não daqui: esta fase
/// é composição — ela sabe onde estão a árvore e o mundo, não o que é um mestre.
pub(super) fn aplicar(
    sim: &mut SimWorld,
    tags: &ph2d_tags::TagTree,
    edits: &[(u64, FactoryFieldEdit)],
) -> bool {
    let mut mexeu = false;
    for (bits, edit) in edits {
        if super::inspector_factory::apply_factory_edit(sim.world_mut(), tags, *bits, edit) {
            mexeu = true;
        }
    }
    mexeu
}
