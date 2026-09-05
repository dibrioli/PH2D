//! Body-section painters for the Vector Style panel, extracted from
//! [`crate::paint`] so the orchestrator fn + the file stay under the panel
//! LOC caps (600/file, 200/fn — `architecture_panel_loc_cap`).
//!
//! [`BodyCtx`] bundles the per-frame mutables (Vello scene, text shaper, widget
//! store, hit index) + the shared layout metrics; each `paint_*` method takes
//! the running `y` and returns the advanced `y`. Pure relocation of the drawing
//! calls — no behavioral change.

use crate::ids;
use crate::section_scope as scope;
use crate::state;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::panel_chrome::{SECTION_LABEL_TO_CONTROL_PX, paint_segmented_button};
use ph2d_editor_core::widget::{
    ButtonKind, ColorSwatch, SectionFold, SectionHeader, SwatchSize, paint_color_swatch,
    paint_section_header,
};
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

/// Label column width for the Width slider row + the Stroke / Fill labels.
pub(crate) const LABEL_COL_W: f32 = 64.0; // LITERAL-PX-OK: panel grid metric (per-panel label gutter width)

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
        let header_h = TypeToken::Md.px() + Spacing::Md.px();
        let collapsed = self.store.is_collapsed(id);
        let header = SectionHeader::new(id, label)
            .collapsible(!collapsed)
            .open_t(self.store.section_open_live(id));
        let rect = Rect::new(self.inner_x, y, self.inner_w, header_h);
        paint_section_header(&header, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
        let body_top = y + header_h + SECTION_LABEL_TO_CONTROL_PX;
        // ⚠️ **A DOBRA (F4b)** — o `bool` devolvido deixou de ser o `is_collapsed` e passou a ser
        //    *fechada **E PARADA***. A distinção é o que a wave entrega: ao clicar para fechar, o
        //    flag semântico vira neste quadro e o `t` ainda desce, então um corpo gateado nele
        //    sumiria de repente por baixo de um chevron a rodar. Quem fecha o escopo é o `step`.
        self.open_fold = SectionFold::begin(
            self.store,
            id,
            self.inner_x,
            self.inner_w,
            body_top,
            self.scene,
            self.hit_index,
        );
        (body_top, self.open_fold.is_none())
    }

    /// **A ordem do corpo.** "O que estou fazendo" (modo · forma · parâmetros da forma)
    /// vem ANTES de "com que cor" (stroke · fill) — o corpo abria em Stroke/Fill e o
    /// seletor de ferramenta aparecia depois, invertido.
    ///
    /// Cada `step` pinta a seção e, só se ela pintou, fecha com o separador canônico: as
    /// condicionais (Text, Axes, Fill Type, Transform, Vertex, Align) somem sem deixar uma
    /// linha órfã. A última (Path) fecha sem separador de rodapé.
    pub(crate) fn paint_body(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let mut y = y;
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| b.tool_section(snap, y));
        y = self.step_in(y, snap.mode, scope::CATALOG, |b, y| {
            b.shape_catalog_section(snap, y)
        });
        // O LÁPIS entra aqui, entre a fileira TOOL e os parâmetros da forma: os três respondem
        // "o que estou desenhando, e como". Some inteira fora do modo Pencil.
        y = self.step_in(y, snap.mode, scope::PENCIL, |b, y| {
            b.pencil_section(snap, y)
        });
        // A SIMETRIA vem logo depois: ela é uma OPÇÃO das ferramentas de desenho (o outro lado
        // do traço nasce derivado enquanto está ligada), então mora ao lado do que decide o que se
        // desenha — e não entre os deformadores, que é onde ela estava quando era um efeito.
        y = self.step_in(y, snap.mode, scope::symmetry(snap), |b, y| {
            b.symmetry_section(snap, y)
        });
        // A MOLDURA vem logo depois da simetria: as duas são propriedades do OBJETO que se
        // acabou de desenhar, e a seção some inteira quando a seleção não é moldura.
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| b.frame_section(y));
        // O RECORTE vem logo depois, e é irmã da Frame por assunto — mas com alcance MAIOR: ela
        // aparece sobre qualquer forma fechada, e uma moldura recebe as duas (2026-08-21).
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| b.clip_section(y));
        // O AUTO LAYOUT vem logo DEPOIS da moldura, e a ordem é o assunto: a seção Frame diz o
        // que o contêiner É, esta diz o que ele FAZ com o conteúdo. Some inteira quando não há
        // moldura na seleção nem um filho dentro de um fluxo.
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| b.layout_section(y));
        // AS ÂNCORAS vêm logo a seguir ao layout, e são a OUTRA metade da mesma pergunta
        // (*o que este filho faz quando a moldura muda de tamanho?*): o layout responde por quem
        // está num fluxo, esta por quem não está. Elas nunca aparecem juntas — a shell só publica
        // a regra para um filho cujo pai NÃO flui.
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| b.anchors_section(y));
        // OS COMPONENTES (plano UI/UX W5): o prefab. Depois das ÂNCORAS porque uma instância é
        // primeiro uma forma (pose, moldura, regra) e só depois uma cópia de outra coisa.
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| b.component_section(y));
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| b.widget_skin_section(y));
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| b.ui_states_section(y));
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| {
            b.shape_params_section(snap, y)
        });
        // Os parâmetros do CONECTOR selecionado — irmã da seção acima (mesma estética,
        // mesmos widgets), logo abaixo dela: as duas respondem "o que é este objeto que
        // eu selecionei, e como o afino?". Some inteira sem conector na seleção.
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::connector_section);
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::text_section);
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::font_section);
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::paragraph_section);
        // Text on Path fica com as outras seções de TEXTO (Font / Paragraph / Axes) e não com
        // os deformadores: o artista a procura onde afina o texto, e ela não deforma glyph
        // nenhum — os glyphs continuam rígidos (plano 22 §1, caso A).
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::textpath_section);
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::axes_section);
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| b.stroke_style(snap, y));
        // ⭐⭐ **A secção PATTERN do TRAÇO** (tinta 1) — Enio, 2026-08-28: *"cada seção deve ter seus
        // ajustes próprios"*. Logo abaixo da secção que descreve a mesma linha, e só sobe quando o
        // traço TEM padrão.
        //
        // ⚠️⚠️ **IRMÃ, e não aninhada dentro da secção *Stroke***: a 1.ª redacção pintou-a lá dentro
        // e o `SectionFold` estourou (*"aberto e nunca fechado: o recorte da cena ficou pendurado"*)
        // — uma dobra não aninha, e o `return` de uma secção vazia deixaria a de fora por fechar.
        // *A UI que se quer ali é uma secção a seguir, não uma secção dentro.*
        y = self.step_in(y, snap.mode, scope::ALWAYS, |s, y| {
            s.texture_pattern_section(1, y)
        });
        // ⭐⭐⭐ E a do PINCEL (plano 36, W4), ao lado da do padrão do traço: as duas descrevem a
        // mesma linha, e só sobe a que o traço de facto tem.
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::brush_section);
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| b.fill_style(snap, y));
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::fill_type_section);
        // A lei do PADRÃO fica colada ao selector que a escolhe: o artista carrega no chip
        // `Pattern` e os controles dele aparecem logo abaixo. Ela só sobe quando a forma TEM um
        // padrão, então numa forma sólida não custa uma linha.
        // ⭐ A secção do PREENCHIMENTO (tinta 0); a do TRAÇO é irmã da *Stroke*, logo acima.
        y = self.step_in(y, snap.mode, scope::ALWAYS, |s, y| {
            s.texture_pattern_section(0, y)
        });
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::snap_section);
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::transform_section);
        // ⭐ A APARÊNCIA do objecto (opacidade + mistura) logo abaixo do Transform: as duas
        // descrevem a forma SELECIONADA como um todo, e o artista procura-as juntas.
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::appearance_section);
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::vertex_section);
        y = self.step_in(y, snap.mode, scope::WHEN_SELECTED, Self::boolean_section);
        // Expand é irmã da Boolean: comandos destrutivos sobre a seleção, mesmo motor.
        y = self.step_in(y, snap.mode, scope::WHEN_SELECTED, Self::expand_section);
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| b.blend_section(snap, y));
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| b.morph_section(y));
        // ⭐ A MÁQUINA do Morph (plano 32 W7) — logo abaixo da seção que cria o objecto, porque a
        // ordem é o assunto: ali *o que ele é*, aqui *como ele decide em que forma está*.
        // ⛔ Seção PRÓPRIA e não uma sub-lista da `ui_states_section`: ver `paint_morph_states`.
        y = self.step_in(y, snap.mode, scope::ALWAYS, |b, y| {
            b.morph_states_section(y)
        });
        // O Envelope fica junto dos outros deformadores não-destrutivos (Blend/Morph): os três
        // produzem geometria DERIVADA de uma relação viva, e o artista os procura no mesmo lugar.
        y = self.step_in(y, snap.mode, scope::WHEN_SELECTED, Self::envelope_section);
        // Pattern on Path é da MESMA família (geometria derivada de uma relação: motivo + guia).
        // Só sobe quando há vínculo ou a seleção o permite (plano 23), então não vira ruído.
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::patternpath_section);
        // Contour fecha a família da geometria derivada, e fica ao lado do Pattern on Path pelo
        // mesmo critério: os anéis são desenho, a forma autorada segue intocada. Some inteira
        // sem contour vivo nem seleção que o permita.
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::contour_section);
        // Effects fica logo depois dos deformadores: os três são não-destrutivos, e a
        // pilha é a generalização deles (ADR-0132).
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::effects_section);
        // Filters (FX raster, plano 24) — logo após Effects, mas de OUTRA natureza: pixels, não
        // geometria. Some inteira sem forma selecionada e sem filtro vivo.
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::filters_section);
        y = self.step_in(y, snap.mode, scope::ALWAYS, Self::align_section);
        y = self.step_in(y, snap.mode, scope::WHEN_SELECTED, Self::arrange_section);
        // ⚠️ **A última NÃO passa pelo `step`** (ela fecha sem separador de rodapé), então a dobra
        //    dela fecha aqui — pela MESMA porta, nunca por uma cópia. Sem esta linha o escopo
        //    ficaria pendurado e o `Drop` do `SectionFold` gritaria em debug.
        self.open_fold = None;
        // ⚠️ **A ÚNICA fora do `step_in`, e o escopo dela entra à mão** — ela fecha a dobra sozinha
        //    (é a última e não leva separador de rodapé). Os seis botões dela morrem todos numa
        //    guarda de seleção, então é `WHEN_SELECTED` como as outras quatro; o gate nomeia esta
        //    excepção pelo nome, e um segundo nome ali significa que alguém copiou o contrato.
        let after = if scope::WHEN_SELECTED.covers(snap.mode, state::current_selection_count()) {
            self.path_section(y)
        } else {
            y
        };
        self.close_fold(after)
    }

    /// **Fecha a dobra que o [`BodyCtx::section_header`] abriu**, se houver — a porta única do
    /// fecho, com DOIS consumidores (o [`BodyCtx::step`] e a última secção, que não passa por ele).
    /// Sem dobra em voo é a identidade.
    fn close_fold(&mut self, after: f32) -> f32 {
        match self.open_fold.take() {
            Some(fold) => fold.finish(self.store, self.scene, self.hit_index, after),
            None => after,
        }
    }

    /// ⭐⭐⭐ **Um passo do corpo QUE PERGUNTA DE QUEM É A SEÇÃO** — o irmão do [`Self::step`] com a
    /// tabela ([`crate::section_scope`]) no caminho.
    ///
    /// Report do Enio (2026-08-31): *"deixar no painel apenas o que é útil para a ferramenta em
    /// uso"*. ⚠️ **Medido: das 39 seções, UMA consultava o modo** — e a lei já estava escrita e
    /// honrada uma fileira acima, dentro da seção TOOL (o Marquee só no modo Node, o Cut só no modo
    /// Cut). *Ela não atravessava a fronteira da seção porque vivia escrita à mão dentro de cada
    /// uma, onde 38 podem esquecê-la.*
    ///
    /// ⚠️ Uma seção fora do escopo devolve o `y` INTACTO, então o `step` não pinta separador e ela
    /// some inteira — cabeçalho, corpo e linha. É a mesma porta das seções condicionais, e é por
    /// isso que a decisão mora aqui e não dentro de cada uma.
    pub(crate) fn step_in(
        &mut self,
        y: f32,
        mode: ph2d_tool_vector::params::DrawMode,
        scope: crate::section_scope::Scope,
        f: impl FnOnce(&mut Self, f32) -> f32,
    ) -> f32 {
        if !scope.covers(mode, state::current_selection_count()) {
            return y;
        }
        self.step(y, f)
    }

    /// Um passo do corpo: pinta a seção e, **se ela pintou** (o `y` andou), fecha com o
    /// separador. As seções condicionais (Vertex, Align, Text, Axes, Fill Type) somem
    /// sem deixar uma linha órfã boiando no painel — que é o motivo de o separador ser
    /// decidido AQUI, no orquestrador, e não dentro de cada seção.
    pub(crate) fn step(&mut self, y: f32, f: impl FnOnce(&mut Self, f32) -> f32) -> f32 {
        // ⚠️ **LIMPA ANTES, e não é higiene:** uma secção condicional sai sem chamar o
        //    `section_header`, e sem isto ela herdaria a dobra da anterior — que já foi fechada.
        self.open_fold = None;
        let after = f(self, y);
        // ⚠️ **Fecha a dobra que a secção abriu** (F4b). Aqui, e não em cada uma das 35: este é o
        //    único ponto por onde todas passam, então uma secção nova nasce com a dobra fechada.
        let after = self.close_fold(after);
        if after > y {
            self.separator(after)
        } else {
            after
        }
    }

    /// Fill swatch + Fill opacity (0 % = none).
    pub(crate) fn fill_style(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let (mut y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_FILL, tr("panel.vector.section.fill"), y);
        if collapsed {
            return y;
        }
        let swatch_w = SwatchSize::Md.px();
        paint_text(
            self.text_system,
            self.scene,
            tr("panel.vector.section.fill"),
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, self.theme),
        );
        let fill_swatch_rect = Rect::new(
            self.inner_x + self.inner_w - swatch_w,
            y,
            swatch_w,
            self.row_h,
        );
        let fill_swatch =
            ColorSwatch::new(ids::VECTOR_FILL_SWATCH, "Fill color", snap.fill).size(SwatchSize::Md);
        paint_color_swatch(&fill_swatch, fill_swatch_rect, self.scene, self.theme);
        self.hit_index
            .register(ids::VECTOR_FILL_SWATCH, fill_swatch_rect);
        // ⚠️ **A RACHURA**: com um token a cobrir, a cor que a swatch mostra NÃO é a que a arte
        // desenha — e uma swatch que afirma um valor que ninguém usa é a pior UI possível.
        let bindings = crate::state::token_bindings();
        if bindings.as_ref().is_some_and(|b| b.fill.is_some()) {
            self.token_slash(fill_swatch_rect);
        }
        y += self.row_h + self.row_gap;

        // **O TOKEN do preenchimento** (plano UI/UX W4), logo abaixo da swatch que ele cobre.
        if let Some(b) = bindings {
            y = self.token_row(ids::VECTOR_TOKEN_FILL, b.fill.as_deref(), y);
        }

        let track = self
            .store
            .slider(ids::VECTOR_FILL_OPACITY)
            .map(|(_, v)| v)
            .unwrap_or_else(|| opacity_to_slider(snap.fill[3]));
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent for the opacity chip
        self.slider_row(
            "Opacity",
            ids::VECTOR_FILL_OPACITY,
            ids::VECTOR_FILL_OPACITY_NUM,
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
