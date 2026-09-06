//! A seção **SKELETON** do painel Vector — módulo irmão do [`super`] (teto de 600 LOC).
//!
//! O que ela oferece é o que o gesto do modo Osso **não** pode dar: prender formas ao esqueleto,
//! soltá-las, e os dois números de um osso.
//!
//! # Por que Keep Pose E Release
//!
//! É o mesmo par do Envelope, e pela mesma razão: prender sem soltar é **porta de mão única**. Os
//! dois soltam a forma e diferem numa pergunta — *qual geometria fica?* **Keep Pose** materializa a
//! deformada (o que o artista está a ver); **Release** devolve a autorada (o que ele desenhou).
//! Adivinhar qual dos dois ele quer é que não.
//!
//! ⚠️ Os dois só são **pintados** com uma forma presa na seleção (`state::skinned`, publicado pela
//! shell) — *um botão que só sabe recusar é pior que um botão ausente*. O **Bind** é pintado sempre:
//! ele age sobre a seleção, e recusar em voz alta é mais honesto que esconder a porta de entrada.

use super::*;

impl BodyCtx<'_> {
    /// Seção **SKELETON** — prender ao esqueleto, as duas saídas, e o osso em foco.
    pub(crate) fn bone_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        // ⛔ **Ela não aparece num app que não tem esqueleto nenhum** — só na ferramenta que faz
        // ossos, que é onde o artista está prestes a ter um. Com um esqueleto na cena ela vale em
        // TODA ferramenta, e a razão é medida contra o próprio desenho: o osso posa-se com a seta
        // (o gizmo de sprite), então esconder os números dele fora do modo Osso tornaria
        // `Length`/`Strength` inalcançáveis exactamente quando se precisa deles.
        if !state::has_skeleton() && snap.mode != ph2d_tool_vector::params::DrawMode::Bone {
            return y;
        }
        let (mut y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_BONE, tr("panel.vector.section.bone"), y);
        if collapsed {
            return y;
        }
        // Tabela tipada como a do Blend e a do Envelope (HR-12): o `action_button` delega ao
        // `paint_button` canónico, que é quem costura o AccessKit — e nomear o [`ph2d_a11y::NodeId`]
        // aqui é o idioma que o gate `every_widget_file_wires_a11y` lê.
        let verbos: [(ph2d_a11y::NodeId, &str); 1] =
            [(ids::VECTOR_BONE_BIND, tr("panel.vector.bone.bind"))];
        for (id, label) in verbos {
            y = self.action_button(id, label, y);
        }
        if state::skinned() {
            let saidas: [(ph2d_a11y::NodeId, &str); 2] = [
                (ids::VECTOR_BONE_EXPAND, tr("panel.vector.bone.expand")),
                (ids::VECTOR_BONE_RELEASE, tr("panel.vector.bone.release")),
            ];
            for (id, label) in saidas {
                y = self.action_button(id, label, y);
            }
        }
        // Os dois números do OSSO em foco. Sem osso não há sujeito — e um campo sem sujeito é a
        // classe de controlo morto que o `CLAUDE.md` §5.0 nomeia.
        if state::current_bone().is_some() {
            let campos: [(ph2d_a11y::NodeId, &str, f64); 2] = [
                (
                    ids::VECTOR_BONE_LENGTH,
                    tr("panel.vector.bone.length"),
                    LENGTH_STEP,
                ),
                (
                    ids::VECTOR_BONE_STRENGTH,
                    tr("panel.vector.bone.strength"),
                    STRENGTH_STEP,
                ),
            ];
            for (id, label, step) in campos {
                y = self.labeled_number_field(label, id, step, y);
            }
        }
        y
    }
}

/// Passo do campo de comprimento, no domínio do DOCUMENTO (unidades de mundo).
const LENGTH_STEP: f64 = 1.0; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Passo do campo de força — ela é um **múltiplo do comprimento do osso**, então a escala útil é
/// a unidade, e o passo é o décimo dela.
const STRENGTH_STEP: f64 = 0.1; // LITERAL-PX-OK: passo no domínio do documento, não medida de design
