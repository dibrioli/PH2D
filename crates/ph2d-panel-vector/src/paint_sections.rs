//! Body-section painters for the Vector Style panel, extracted from
//! [`crate::paint`] so the orchestrator fn + the file stay under the panel
//! LOC caps (600/file, 200/fn — `architecture_panel_loc_cap`).
//!
//! [`BodyCtx`] bundles the per-frame mutables (Vello scene, text shaper, widget
//! store, hit index) + the shared layout metrics; each `paint_*` method takes
//! the running `y` and returns the advanced `y`. Pure relocation of the drawing
//! calls — no behavioral change.

use crate::ids;
use crate::state;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::widget::{ButtonKind, SectionFold};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_tool_vector::params::{
    BLEND_STEPS_DEFAULT, MORPH_T_DEFAULT, blend_steps_from_track, blend_steps_to_track,
    dash_to_slider, gap_to_slider, opacity_to_slider,
};
use ph2d_tool_vector::{
    StrokeCap, StrokeJoin, VectorStyleSnapshot, VertexSel, VertexType, px_to_slider,
};
use ph2d_vec_scene::StrokeAlign;
use ph2d_vector::VectorScene;

/// **A coluna de RÓTULO** deste painel — a linha do slider de largura, os rótulos de Stroke / Fill,
/// os chips de token, a fileira de marcadores. Ela **pergunta a porta** desde 2026-09-14.
///
/// ⛔⛔ **Era uma constante de `64.0`, e o preço da conversão estava escrito na moeda
/// errada.** A dívida dizia *«**33 sítios** a usá-la como constante livre, **muitos deles sem o
/// `y`/`row_h` em alcance**, logo a conversão é uma wave própria e não um `sed`»* — e a largura da
/// coluna **nunca dependeu do vertical**: a [`ph2d_editor_core::widget::property_label_col_w`]
/// lê `x` e `w` e mais nada. Os 33 sítios tinham todos o `inner_x`/`inner_w` à mão, porque todos
/// vivem num método do [`BodyCtx`]. *Um bloqueio afirmado sobre um argumento que o resultado não lê
/// é um palpite com cara de medição.*
///
/// ⚠️ **É uma função e não uma constante de propósito:** a coluna docada é arrastável
/// (`WidgetStore::DOCK_W_MIN`..`720`), e um número afinado à largura de omissão come o controlo
/// numa coluna estreita e deixa-o absurdo numa larga.
pub(crate) fn label_col_w(inner_x: f32, inner_w: f32) -> f32 {
    ph2d_editor_core::panel::label_col_w(inner_x, inner_w)
}

/// Per-frame paint context for the Vector Style panel body — the mutable render
/// targets + the shared layout metrics. Constructed once per frame in
/// [`crate::paint`]; each section method borrows disjoint fields.
pub(crate) struct BodyCtx<'a> {
    pub scene: &'a mut VectorScene,
    pub text_system: &'a mut TextSystem,
    pub store: &'a WidgetStore,
    pub hit_index: &'a mut HitIndex,
    pub theme: Theme,
    pub inner_x: f32,
    pub inner_w: f32,
    pub row_h: f32,
    pub row_gap: f32,
    pub chip_w: f32,
    pub font: f32,
    /// **A dobra da secção que está a ser pintada AGORA** (F4b), guardada entre o
    /// [`BodyCtx::section_header`] que a abre e o [`BodyCtx::step`] que a fecha.
    ///
    /// ⚠️ **Fica aqui e não é devolvida ao pintor** porque este painel tem **35** secções e um
    /// orquestrador só: pedir a cada uma que carregasse o escopo custaria 35 sítios onde a única
    /// coisa que pode acontecer é esquecer o `finish`. O `step` já é a porta por onde toda secção
    /// passa — é lá que ela fecha.
    pub open_fold: Option<SectionFold>,
}

impl BodyCtx<'_> {
    /// ⭐⭐⭐ **A PORTA para o vocabulário de linhas partilhado**
    /// ([`ph2d_editor_core::panel::RowCtx`]).
    ///
    /// ⛔⛔ **Ela existe para que os 147 chamadores deste painel NÃO mudem.** O vocabulário saiu
    /// daqui em 2026-09-09, quando o esqueleto ganhou painel próprio e passaram a ser dois
    /// hospedeiros — e *uma extracção que obriga 147 sítios a mudar de nome no mesmo commit é uma
    /// extracção que colide com toda linha viva*. Os métodos ficaram; os corpos passaram a delegar.
    ///
    /// ⚠️ **A dobra ATRAVESSA** (entra e volta): ela é aberta pelo `section_header` e fechada pelo
    /// `close_fold`, que são duas chamadas distintas — deixá-la para trás faria toda secção deste
    /// painel nascer sem dobra.
    pub(crate) fn with_rows<R>(
        &mut self,
        f: impl FnOnce(&mut ph2d_editor_core::panel::RowCtx) -> R,
    ) -> R {
        let mut rc = ph2d_editor_core::panel::RowCtx {
            scene: &mut *self.scene,
            text_system: &mut *self.text_system,
            store: self.store,
            hit_index: &mut *self.hit_index,
            theme: self.theme,
            inner_x: self.inner_x,
            inner_w: self.inner_w,
            row_h: self.row_h,
            row_gap: self.row_gap,
            font: self.font,
            open_fold: self.open_fold.take(),
        };
        let out = f(&mut rc);
        self.open_fold = rc.open_fold.take();
        out
    }
}

/// A seção **States** (plano UI/UX W7) — módulo irmão (teto de 600 LOC).
#[path = "paint_states.rs"]
mod ui_states;

/// ⭐ **A TABELA SINAL → PAPEL** — irmã da de States pelo mesmo corte de assunto.
#[path = "paint_signals.rs"]
mod ui_signals;

/// ⭐ **A seção MORPH STATES** (plano 32 W4/W7) — irmã das duas acima por assunto e **seção
/// própria** por decisão de produto: ela era uma sub-lista da `ui_states` e fazia o cabeçalho de
/// uma feature já entregue aparecer por causa de outra.
#[path = "paint_morph_states.rs"]
pub(crate) mod morph_arrows;

pub(crate) use ui_signals::mirror as mirror_signal_fields;

/// A seção **Blend** — módulo irmão (teto de 600 LOC).
#[path = "paint_blend.rs"]
mod blend;

/// A seção **Envelope** — módulo irmão (teto de 600 LOC).
#[path = "paint_envelope.rs"]
mod envelope;

/// A seção **Text on Path** (plano 22) — módulo irmão (teto de 600 LOC).
#[path = "paint_textpath.rs"]
mod textpath;

/// A seção **Pattern on Path** (plano 23) — módulo irmão (teto de 600 LOC).
#[path = "paint_patternpath.rs"]
mod patternpath;
/// A secção **PATTERN** — a TINTA de uma forma (plano 33). ⚠️ Não confundir com a `patternpath`
/// acima, que é o motivo-sobre-guia.
#[path = "paint_texture_pattern.rs"]
pub mod texture_pattern;

/// ⭐⭐⭐ **A secção BRUSH** (plano 36, W4) — irmã da do padrão pelo teto de LOC, e o corte é por
/// MODELO: um pincel tem avanço e escala relativa, um padrão tem reticulado, fase e repetição.
#[path = "paint_brush.rs"]
pub mod brush;

/// A seção **Contour** (pesquisa `20_*` #9) — módulo irmão (teto de 600 LOC).
#[path = "paint_contour.rs"]
mod contour;

/// A seção **Filters** (FX raster, plano 24) — módulo irmão (teto de 600 LOC).
#[path = "paint_filters.rs"]
pub(crate) mod filters;

/// O **trilho da rampa** do card de Gradient Map — módulo irmão (teto de 600 LOC).
#[path = "paint_filters_ramp.rs"]
mod filters_ramp;

/// A **lei de mistura** de um degrau de filtro — módulo irmão (teto de 600 LOC).
#[path = "paint_filters_blend.rs"]
pub(crate) mod filters_blend;

/// A seção **Effects** (ADR-0132) — módulo irmão (teto de 600 LOC).
#[path = "paint_effects.rs"]
mod effects;

/// A seção **Vertex** (tipo do vértice + Delete) — módulo irmão (teto de 600 LOC).
#[path = "paint_vertex.rs"]
mod vertex;

/// A seção **Expand** (Outline Stroke · Offset Path · Power Stroke) — módulo irmão (teto de
/// 600 LOC).
#[path = "paint_expand.rs"]
mod expand;

#[path = "paint_sections_stroke.rs"]
mod stroke;

/// A metade PURA do [`BodyCtx::live_track`] — sem cena, sem texto, sem painel. É ela que os gates
/// dirigem: um `BodyCtx` exige uma `VectorScene` e um `TextSystem`, e a LEI não precisa de nenhum.
pub(crate) fn live_track(store: &WidgetStore, id: ph2d_a11y::NodeId, from_doc: f32) -> f32 {
    match store.slider(id) {
        Some((ph2d_editor_core::widget::SliderState::Dragging, v)) => v,
        _ => from_doc,
    }
}

/// A metade PURA do [`BodyCtx::live_number`], pela mesma razão.
pub(crate) fn live_number(store: &WidgetStore, id: ph2d_a11y::NodeId, from_doc: f64) -> f64 {
    if store.focus_id() == Some(id) {
        store.number_value(id).unwrap_or(from_doc)
    } else {
        from_doc
    }
}

impl BodyCtx<'_> {
    /// ⭐⭐⭐ **A PISTA que um slider desenha: a do DOCUMENTO, salvo enquanto o dedo a arrasta.**
    ///
    /// # A lei, e por que ela é uma inversão e não um espelho
    ///
    /// A alternativa óbvia — **re-semear** o store a partir do documento a cada quadro — é a que as
    /// secções de Transform/Vertex/Connector usam para os campos NUMÉRICOS, e é a errada aqui: ela
    /// escreve todo quadro, exige uma guarda **diferente por tipo de controlo** (foco na caixa,
    /// arrasto no slider) e pode escrever no store um valor **fora da faixa registada** do widget.
    /// Esta lei não escreve nada: *o store só vence enquanto a mão está no controlo.*
    ///
    /// ⚠️ **Ela já vivia nesta crate, privada, na secção de FILTROS** — com a razão escrita:
    /// *"é o que faz o slider mostrar o filtro da forma no instante em que ela é selecionada — sem
    /// um «mirror» que re-semeie o store — e ainda seguir o dedo durante o arrasto"*. Estava numa
    /// função privada de um módulo, e as secções *Pattern* e *Brush* nasceram sem ela: elas liam o
    /// store **primeiro e sempre**, e o `unwrap_or(documento)` era código morto, porque
    /// `WidgetStore::slider` devolve `Some` para todo widget registado.
    /// *Uma lei escrita num módulo só ainda não é uma lei — só uma PORTA é.*
    ///
    /// O sintoma que isso produzia: escolher uma forma com *Offset 1/4* depois de outra com *1/2*
    /// mostrava **1/2**, e o primeiro toque escrevia esse número alheio na forma nova.
    pub(crate) fn live_track(&self, id: ph2d_a11y::NodeId, from_doc: f32) -> f32 {
        live_track(self.store, id, from_doc)
    }

    /// ⭐⭐ **O NÚMERO que um chip mostra: o do DOCUMENTO, salvo enquanto o artista o digita.**
    ///
    /// A metade irmã do [`Self::live_track`], e a guarda **não é a mesma**: o arrasto de um slider
    /// **não dá foco** ao widget, e digitar não põe nada em `Dragging`. Trocá-las é um defeito
    /// silencioso — uma caixa guardada por arrasto é reescrita a cada tecla, e um slider guardado
    /// por foco é reescrito debaixo do dedo.
    ///
    /// ⚠️ **O `NumberInput` é dono do próprio buffer enquanto tem foco**; fora dele, quem manda é o
    /// documento. O atraso de um quadro entre largar e ver o valor novo é o mesmo que os campos do
    /// Transform já declaram aceitável (*"1-frame post-commit lag, ok"*).
    pub(crate) fn live_number(&self, id: ph2d_a11y::NodeId, from_doc: f64) -> f64 {
        live_number(self.store, id, from_doc)
    }

    /// O cabeçalho CANÔNICO de seção (`docs/UI_Padrao/components/section_header.md`:
    /// "TODA seção usa `paint_section_header`" · "TODA seção é colapsável"). Chevron +
    /// rótulo em MAIÚSCULAS; o clique DOBRA a seção. Retorna `(y, collapsed)` — o caller
    /// sai cedo quando dobrada.
    ///
    /// O collapse é dispatch GENÉRICO e exige **dois** sites: o `populate` marca o id
    /// (`mark_collapsible_section`) e aqui registramos o hit-rect. Sem os dois o header
    /// vira um título morto que pinta um chevron e não dobra.
    ///
    /// Substituiu os 14 `section_label` caseiros do painel (`paint_text` em Sm/Text2):
    /// eram tipograficamente idênticos aos botões abaixo deles, e é por isso que
    /// categoria e tipo de forma se confundiam.
    pub(crate) fn section_header(
        &mut self,
        id: ph2d_a11y::NodeId,
        label: &str,
        y: f32,
    ) -> (f32, bool) {
        self.with_rows(|r| r.section_header(id, label, y))
    }

    /// **O corpo**: a lista NATURAL das secções ([`crate::paint_body_plan::corpo`]) pintada pela
    /// ordem do artista e cada uma no tema dela ([`BodyCtx::corre`]).
    pub(crate) fn paint_body(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let p = crate::paint_body_plan::corpo(snap);
        self.corre(p, y)
    }

    /// **Fecha a dobra que o [`BodyCtx::section_header`] abriu**, se houver — a porta única do
    /// fecho, com DOIS consumidores (o [`BodyCtx::step`] e a última secção, que não passa por ele).
    /// Sem dobra em voo é a identidade.
    pub(crate) fn close_fold(&mut self, after: f32) -> f32 {
        self.with_rows(|r| r.close_fold(after))
    }

    /// Fill swatch + Fill opacity (0 % = none).
    pub(crate) fn fill_style(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ph2d_tool_vector::ids::VECTOR_SECTION_FILL,
            tr("panel.vector.section.fill"),
            y,
        );
        if collapsed {
            return y;
        }
        // ⭐ Pela PORTA — ver [`super::paint_rows`] / `colour_swatch_row_rect`.
        let (proximo_y, fill_swatch_rect) = self.colour_swatch_row_rect(
            ph2d_tool_vector::ids::VECTOR_FILL_SWATCH,
            snap.fill,
            tr("panel.vector.section.fill"),
            y,
        );
        // ⚠️ **A RACHURA**: com um token a cobrir, a cor que a swatch mostra NÃO é a que a arte
        // desenha — e uma swatch que afirma um valor que ninguém usa é a pior UI possível.
        let bindings = crate::state::token_bindings();
        if bindings.as_ref().is_some_and(|b| b.fill.is_some()) {
            self.token_slash(fill_swatch_rect);
        }
        y = proximo_y;

        // **O TOKEN do preenchimento** (plano UI/UX W4), logo abaixo da swatch que ele cobre.
        if let Some(b) = bindings {
            y = self.token_row(ids::VECTOR_TOKEN_FILL, b.fill.as_deref(), y);
        }

        let track = self
            .store
            .slider(ph2d_tool_vector::ids::VECTOR_FILL_OPACITY)
            .map(|(_, v)| v)
            .unwrap_or_else(|| opacity_to_slider(snap.fill[3]));
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent for the opacity chip
        self.slider_row(
            tr("panel.vector.paint.opacity"),
            ph2d_tool_vector::ids::VECTOR_FILL_OPACITY,
            ph2d_tool_vector::ids::VECTOR_FILL_OPACITY_NUM,
            track,
            pct,
            &format!("{}", pct.round() as i64),
            y,
        )
    }
}

#[cfg(test)]
#[path = "paint_sections_live_tests.rs"]
mod live_tests;
