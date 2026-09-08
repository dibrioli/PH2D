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
        // ⭐⭐⭐ **CRIAR × TRANSFORMAR, no TOPO da seção** (Enio, 2026-09-07: *«do modo como está
        // fica confuso para o usuário»*). Ele vem antes dos verbos porque **decide o que os outros
        // controlos significam**: com *Criar* o arrasto faz osso, com *Transformar* ele posa — e
        // ler isso depois de já ter carregado é tarde.
        //
        // ⚠️ **Ele só aparece no MODO Osso**, e é a mesma lei da seção: os números de um osso valem
        // em toda ferramenta (o osso posa-se com a seta), mas *o que o arrasto faz* só tem sujeito
        // onde há arrasto de osso.
        if snap.mode == ph2d_tool_vector::params::DrawMode::Bone {
            let acoes: [(ph2d_a11y::NodeId, &str, bool); 2] = [
                (
                    ids::VECTOR_BONE_ACT_CREATE,
                    tr("panel.vector.bone.create"),
                    snap.bone_action == ph2d_tool_vector::BoneAction::Create,
                ),
                (
                    ids::VECTOR_BONE_ACT_TRANSFORM,
                    tr("panel.vector.bone.transform"),
                    snap.bone_action == ph2d_tool_vector::BoneAction::Transform,
                ),
            ];
            y = self.segmented(tr("panel.vector.bone.action"), &acoes, y);
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
            y = self.limit_rows(y);
            y = self.ik_rows(y);
        }
        y
    }

    /// ⭐⭐⭐ **O LIMITE DE ÂNGULO** da junta em foco — a porta de entrada, ou os dois extremos.
    ///
    /// ⚠️ **Vem ANTES da âncora**, e a ordem diz o que ele é: o limite é uma propriedade da JUNTA
    /// (vale com IK e sem ela), a âncora é uma restrição que se põe e se tira. Pô-lo dentro do
    /// bloco da IK ensinaria que ele é parte dela — que é exactamente o desenho do Godot, e o que
    /// esta casa decidiu não fazer.
    fn limit_rows(&mut self, y: f32) -> f32 {
        let Some(_) = state::current_bone_limit() else {
            return self.action_button(
                ids::VECTOR_BONE_LIMIT_ADD,
                tr("panel.vector.bone.limit.add"),
                y,
            );
        };
        let mut y = self.action_button(
            ids::VECTOR_BONE_LIMIT_REMOVE,
            tr("panel.vector.bone.limit.remove"),
            y,
        );
        let campos: [(ph2d_a11y::NodeId, &str); 2] = [
            (ids::VECTOR_BONE_LIMIT_MIN, tr("panel.vector.bone.limit.min")),
            (ids::VECTOR_BONE_LIMIT_MAX, tr("panel.vector.bone.limit.max")),
        ];
        for (id, label) in campos {
            y = self.labeled_number_field(label, id, ANGLE_STEP, y);
        }
        y
    }

    /// ⭐⭐⭐ **A ÂNCORA DE IK** do osso em foco — a porta de entrada, ou os três números dela.
    ///
    /// ⚠️ **É um OU exclusivo, e é a lei do controlo morto:** *Add IK* só aparece em quem não tem
    /// âncora, e *Remove IK* mais os três números só em quem tem. Oferecer as duas portas ao mesmo
    /// tempo daria um botão que só sabe recusar — e o gesto recusa-o também, então o painel estaria
    /// a prometer o que o app não faz.
    fn ik_rows(&mut self, y: f32) -> f32 {
        let Some((_, _, _, lado)) = state::current_bone_ik() else {
            return self.action_button(ids::VECTOR_BONE_IK_ADD, tr("panel.vector.bone.ik.add"), y);
        };
        let mut y = self.action_button(
            ids::VECTOR_BONE_IK_REMOVE,
            tr("panel.vector.bone.ik.remove"),
            y,
        );
        let campos: [(ph2d_a11y::NodeId, &str, f64); 3] = [
            (
                ids::VECTOR_BONE_IK_MIX,
                tr("panel.vector.bone.ik.mix"),
                MIX_STEP,
            ),
            (
                ids::VECTOR_BONE_IK_SOFTNESS,
                tr("panel.vector.bone.ik.softness"),
                MIX_STEP,
            ),
            (
                ids::VECTOR_BONE_IK_CHAIN,
                tr("panel.vector.bone.ik.chain"),
                CHAIN_STEP,
            ),
        ];
        for (id, label, step) in campos {
            y = self.labeled_number_field(label, id, step, y);
        }
        // ⭐⭐⭐ **PARA QUE LADO O JOELHO DOBRA** — e ele vem DEPOIS de `Chain` porque é o `Chain`
        // que decide quantos ossos têm lado: ler *«de que lado»* antes de saber *«de que corrente»*
        // é ler a resposta antes da pergunta.
        //
        // ⚠️ A fileira é construída a partir de [`ph2d_skeleton::BendSide::ALL`] e da tabela de ids
        // **ao mesmo tempo**, por índice: uma variante nova na lei sem um id ao lado é erro de
        // compilação (os dois arrays têm de ter o mesmo comprimento), em vez de um segmento que
        // desaparece em silêncio.
        let rotulos = [
            tr("panel.vector.bone.ik.bend.auto"),
            tr("panel.vector.bone.ik.bend.ccw"),
            tr("panel.vector.bone.ik.bend.cw"),
        ];
        let mut lados: [(ph2d_a11y::NodeId, &str, bool); ph2d_skeleton::BendSide::ALL.len()] =
            [(ids::VECTOR_BONE_BEND_IDS[0], "", false); ph2d_skeleton::BendSide::ALL.len()];
        for (i, slot) in lados.iter_mut().enumerate() {
            *slot = (ids::VECTOR_BONE_BEND_IDS[i], rotulos[i], i == lado);
        }
        self.segmented(tr("panel.vector.bone.ik.bend"), &lados, y)
    }
}

/// Passo do campo de comprimento, no domínio do DOCUMENTO (unidades de mundo).
const LENGTH_STEP: f64 = 1.0; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Passo do campo de força — ela é um **múltiplo do comprimento do osso**, então a escala útil é
/// a unidade, e o passo é o décimo dela.
const STRENGTH_STEP: f64 = 0.1; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Passo dos dois números adimensionais da âncora (`Mix` e `Softness`), que vivem em `0..1`: o
/// décimo da unidade, como o da força do osso.
const MIX_STEP: f64 = 0.1; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Passo dos dois extremos do limite, em **graus**: cinco de cada vez.
///
/// ⚠️ O campo fala GRAUS e o documento guarda radianos — a conversão vive na shell. Um passo de
/// `0,0873` (um grau em radianos) neste campo seria o número certo na unidade errada.
const ANGLE_STEP: f64 = 5.0; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Passo da CORRENTE — ela conta ossos, então o passo é **um osso**.
const CHAIN_STEP: f64 = 1.0; // LITERAL-PX-OK: passo no domínio do documento, não medida de design
