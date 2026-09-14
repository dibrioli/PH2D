//! **Fase do quadro: OS GESTOS SOBRE A ÁRVORE DE TAGS** (TOP-20 #9, W4) — fase-filha do
//! [`super`], num ficheiro irmão.
//!
//! ⚠️ **Ela é filha do `fase_inspector_commits` e NÃO é uma edição do Inspector**, e as duas coisas
//! são verdade ao mesmo tempo: o sujeito destes cinco gestos é a **taxonomia do projecto**, não um
//! objecto — mas a fase-mãe é a única do quadro que tem, ao mesmo tempo, as três coisas que eles
//! pedem: a **árvore** (que não está no mundo), o **mundo** (apagar leva a pertença junto) e o
//! **ecrã** (o *Select* muda a selecção, que é do editor).
//!
//! ⚠️ **O corte foi imposto pelo tecto de LOC e está certo por outra razão:** ele segue a fronteira
//! que o comentário do bloco já escrevia. *Partir por responsabilidade, nunca subir o número.*

use super::tags_panel::{self, TagEditOutcome};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::TagTreeEdit;
use ph2d_editor_core::screens::hero::HeroScreen;

/// Aplica os gestos que o painel emitiu neste quadro.
///
/// ⚠️ **A recusa é ESCRITA, não devolvida**: ela tem de sobreviver até ao gesto seguinte (o painel
/// repinta dezenas de vezes entre dois cliques), e um gesto **aceite apaga a frase do anterior** —
/// senão o painel acusaria para sempre um problema que o artista já resolveu.
pub(super) fn aplicar(
    sim: &mut SimWorld,
    tags: &mut ph2d_tags::TagTree,
    tags_problem: &mut Option<(u64, String)>,
    hero: &mut HeroScreen,
    edits: &[TagTreeEdit],
) {
    for edit in edits {
        match tags_panel::apply_tag_tree_edit(sim.world_mut(), tags, edit) {
            TagEditOutcome::Changed { born } => {
                *tags_problem = None;
                #[cfg(feature = "panel-tags")]
                ph2d_panel_tags::set_born_tag(born);
                #[cfg(not(feature = "panel-tags"))]
                let _ = born;
            }
            TagEditOutcome::Refused { tag, why } => {
                // ⚠️ A frase vem da LEI (`TagError::message`), ao lado de quem a produziu — a shell
                // não a escreve, e o painel não a interpreta.
                *tags_problem = Some((tag, why.message().to_string()));
            }
            TagEditOutcome::Select(alvos) => {
                *tags_problem = None;
                // ⚠️ **Substitui e depois acrescenta** — a mesma porta do clique de canvas, e é ela
                // que garante que o primeiro alvo vira o PRIMÁRIO (é dele que o Inspector fala).
                let mut it = alvos.iter();
                if let Some(primeiro) = it.next() {
                    hero.gizmo.replace_selection(Some(primeiro.to_bits()));
                    for e in it {
                        hero.gizmo.add_to_selection(e.to_bits());
                    }
                }
            }
            TagEditOutcome::Nothing => {}
        }
    }
}
