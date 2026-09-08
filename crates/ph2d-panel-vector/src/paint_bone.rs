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
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, paint_dropdown_chip,
    paint_dropdown_popover_scrolled, scrollbar_is_needed, scrollbar_track_rect,
};

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
            y = self.smart_rows(y);
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
            (
                ids::VECTOR_BONE_LIMIT_MIN,
                tr("panel.vector.bone.limit.min"),
            ),
            (
                ids::VECTOR_BONE_LIMIT_MAX,
                tr("panel.vector.bone.limit.max"),
            ),
        ];
        for (id, label) in campos {
            y = self.labeled_number_field(label, id, ANGLE_STEP, y);
        }
        y
    }

    /// ⭐⭐⭐ **O OSSO INTELIGENTE** — girar este osso percorre uma acção inteira.
    ///
    /// ⚠️⚠️ **A linha *Action* vem PRIMEIRO, e ela é a wave de 2026-09-08.** Até esse dia esta
    /// função pintava *Remove* mais dois números e mais nada — o dono carregava em *Add Smart Bone*
    /// e ficava com dois campos de graus **sem sujeito** (report: *«não há meios de selecionar nem o
    /// objeto alvo nem a animação»*). *Um controlo cujo sujeito é invisível lê-se exactamente como
    /// um controlo morto*, e a única resposta na casa era um `eprintln!` que o artista nunca vê.
    ///
    /// ⇒ o chip é o **readout e o gesto**: ele diz o nome da acção ligada e abre a lista das que o
    /// documento tem.
    fn smart_rows(&mut self, y: f32) -> f32 {
        let Some(_) = state::current_bone_smart() else {
            return self.action_button(
                ids::VECTOR_BONE_SMART_ADD,
                tr("panel.vector.bone.smart.add"),
                y,
            );
        };
        let mut y = self.smart_action_row(y);
        y = self.action_button(
            ids::VECTOR_BONE_SMART_REMOVE,
            tr("panel.vector.bone.smart.remove"),
            y,
        );
        let campos: [(ph2d_a11y::NodeId, &str); 2] = [
            (
                ids::VECTOR_BONE_SMART_FROM,
                tr("panel.vector.bone.smart.from"),
            ),
            (ids::VECTOR_BONE_SMART_TO, tr("panel.vector.bone.smart.to")),
        ];
        for (id, label) in campos {
            y = self.labeled_number_field(label, id, ANGLE_STEP, y);
        }
        y
    }

    /// ⭐⭐⭐ **QUAL ACÇÃO** — o chip que a nomeia e abre a lista. Espelho exacto da linha de mistura
    /// de um degrau de filtro (`paint_filters::filter_blend_row`).
    ///
    /// ⚠️ **Vazio mostra o traço**, e não uma cadeia vazia: uma célula em branco lê-se como um
    /// controlo por carregar, e o traço diz *«nenhuma»* em voz alta — a mesma lei da tecla de uma
    /// forma do Morph.
    fn smart_action_row(&mut self, y: f32) -> f32 {
        let gap = Spacing::Xs.px();
        let id = ids::VECTOR_BONE_SMART_CLIP;
        paint_text(
            self.text_system,
            self.scene,
            tr("panel.vector.bone.smart.action"),
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, self.theme),
        );
        let ligada = state::current_bone_smart_clip();
        let rotulo = if ligada.is_empty() {
            tr("panel.vector.bone.smart.none")
        } else {
            ligada.as_str()
        };
        let chip = Rect::new(
            self.inner_x + LABEL_COL_W + gap,
            y,
            (self.inner_w - LABEL_COL_W - gap).max(1.0),
            self.row_h,
        );
        let open = matches!(
            self.store.get(id),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        let dd = Dropdown::new(id, "", vec![DropdownOption::new(id, (), rotulo)])
            .selected(())
            .open(open)
            .visual(self.store.dropdown_visual(id));
        paint_dropdown_chip(&dd, chip, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, chip);
        if open {
            state::set_pending_bone_action_dd(Some(chip));
        }
        y + self.row_h + self.row_gap
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

/// ⭐⭐⭐ **A LISTA DAS ACÇÕES** do osso inteligente — pintada no passe DIFERIDO de `paint.rs`, POR
/// CIMA de todas as seções. Espelho exacto do `paint_filters_blend::paint_blend_popover`.
///
/// ⚠️ **A seção ROLA**, então sem o passe diferido a lista seria cortada na borda dela — foi o que
/// obrigou o Morph e a mistura de filtro a fazerem o mesmo.
///
/// ⚠️ **As acções saem da lista PUBLICADA pela shell**, nunca de uma leitura do painel: elas são
/// conteúdo do documento, e uma segunda leitura aqui envelheceria na primeira que ele criasse.
///
/// ⚠️ **O corte é o POOL de ids** ([`ids::VECTOR_BONE_SMART_CLIP_IDS`]) e não um número escrito
/// aqui: o chrome não cunha um id em tempo de execução, e uma opção sem id nasceria **morta sob o
/// dedo**. O gate da shell mantém o pool do tamanho do tecto do documento.
pub(crate) fn paint_action_popover(ctx: &mut PaintCtx, chip: Rect, theme: Theme) {
    let id = ids::VECTOR_BONE_SMART_CLIP;
    let nomes = state::bone_actions();
    let n = nomes.len().min(ids::VECTOR_BONE_SMART_CLIP_IDS.len());
    if n == 0 {
        return;
    }
    let ligada = state::current_bone_smart_clip();
    let sel = nomes.iter().take(n).position(|c| *c == ligada).unwrap_or(0);
    let options: Vec<DropdownOption<usize>> = nomes
        .iter()
        .take(n)
        .enumerate()
        .map(|(i, nome)| DropdownOption::new(ids::VECTOR_BONE_SMART_CLIP_IDS[i], i, nome.as_str()))
        .collect();
    let dd = Dropdown::new(id, "", options).selected(sel).open(true);

    let panel = dd.popover_rect_clamped(chip, ctx.layout.popover_region());
    let content_h = dd.content_height(chip.h);
    let visible_h = panel.h;
    let max_scroll = (content_h - visible_h).max(0.0);
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(id, panel);
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
        if store.panel_scroll(id) > max_scroll {
            store.set_panel_scroll(id, max_scroll);
        }
    }
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    paint_dropdown_popover_scrolled(
        &dd,
        chip,
        panel,
        scroll,
        ctx.host
            .store()
            .scrollbar_visual_for(DROPDOWN_SCROLLBAR_ID, Some(id)),
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Hit-register só a parte VISÍVEL de cada linha (a barra de rolagem é o alvo do arrasto).
    let hit_index = ctx.host.hit_index_mut();
    for i in 0..n {
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::VECTOR_BONE_SMART_CLIP_IDS[i],
                Rect::new(r.x, top, r.w, bot - top),
            );
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}
