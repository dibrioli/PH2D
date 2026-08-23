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

impl BodyCtx<'_> {
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
        y = self.step(y, |b, y| b.tool_section(snap, y));
        y = self.step(y, |b, y| b.shape_catalog_section(snap, y));
        // O LÁPIS entra aqui, entre a fileira TOOL e os parâmetros da forma: os três respondem
        // "o que estou desenhando, e como". Some inteira fora do modo Pencil.
        y = self.step(y, |b, y| b.pencil_section(snap, y));
        // A SIMETRIA vem logo depois: ela é uma OPÇÃO das ferramentas de desenho (o outro lado
        // do traço nasce derivado enquanto está ligada), então mora ao lado do que decide o que se
        // desenha — e não entre os deformadores, que é onde ela estava quando era um efeito.
        y = self.step(y, |b, y| b.symmetry_section(snap, y));
        // A MOLDURA vem logo depois da simetria: as duas são propriedades do OBJETO que se
        // acabou de desenhar, e a seção some inteira quando a seleção não é moldura.
        y = self.step(y, |b, y| b.frame_section(y));
        // O RECORTE vem logo depois, e é irmã da Frame por assunto — mas com alcance MAIOR: ela
        // aparece sobre qualquer forma fechada, e uma moldura recebe as duas (2026-08-21).
        y = self.step(y, |b, y| b.clip_section(y));
        // O AUTO LAYOUT vem logo DEPOIS da moldura, e a ordem é o assunto: a seção Frame diz o
        // que o contêiner É, esta diz o que ele FAZ com o conteúdo. Some inteira quando não há
        // moldura na seleção nem um filho dentro de um fluxo.
        y = self.step(y, |b, y| b.layout_section(y));
        // AS ÂNCORAS vêm logo a seguir ao layout, e são a OUTRA metade da mesma pergunta
        // (*o que este filho faz quando a moldura muda de tamanho?*): o layout responde por quem
        // está num fluxo, esta por quem não está. Elas nunca aparecem juntas — a shell só publica
        // a regra para um filho cujo pai NÃO flui.
        y = self.step(y, |b, y| b.anchors_section(y));
        // OS COMPONENTES (plano UI/UX W5): o prefab. Depois das ÂNCORAS porque uma instância é
        // primeiro uma forma (pose, moldura, regra) e só depois uma cópia de outra coisa.
        y = self.step(y, |b, y| b.component_section(y));
        y = self.step(y, |b, y| b.widget_skin_section(y));
        y = self.step(y, |b, y| b.ui_states_section(y));
        y = self.step(y, |b, y| b.shape_params_section(snap, y));
        // Os parâmetros do CONECTOR selecionado — irmã da seção acima (mesma estética,
        // mesmos widgets), logo abaixo dela: as duas respondem "o que é este objeto que
        // eu selecionei, e como o afino?". Some inteira sem conector na seleção.
        y = self.step(y, Self::connector_section);
        y = self.step(y, Self::text_section);
        y = self.step(y, Self::font_section);
        y = self.step(y, Self::paragraph_section);
        // Text on Path fica com as outras seções de TEXTO (Font / Paragraph / Axes) e não com
        // os deformadores: o artista a procura onde afina o texto, e ela não deforma glyph
        // nenhum — os glyphs continuam rígidos (plano 22 §1, caso A).
        y = self.step(y, Self::textpath_section);
        y = self.step(y, Self::axes_section);
        y = self.step(y, |b, y| b.stroke_style(snap, y));
        y = self.step(y, |b, y| b.fill_style(snap, y));
        y = self.step(y, Self::fill_type_section);
        y = self.step(y, Self::snap_section);
        y = self.step(y, Self::transform_section);
        y = self.step(y, Self::vertex_section);
        y = self.step(y, Self::boolean_section);
        // Expand é irmã da Boolean: comandos destrutivos sobre a seleção, mesmo motor.
        y = self.step(y, Self::expand_section);
        y = self.step(y, |b, y| b.blend_section(snap, y));
        y = self.step(y, |b, y| b.morph_section(y));
        // O Envelope fica junto dos outros deformadores não-destrutivos (Blend/Morph): os três
        // produzem geometria DERIVADA de uma relação viva, e o artista os procura no mesmo lugar.
        y = self.step(y, Self::envelope_section);
        // Pattern on Path é da MESMA família (geometria derivada de uma relação: motivo + guia).
        // Só sobe quando há vínculo ou a seleção o permite (plano 23), então não vira ruído.
        y = self.step(y, Self::patternpath_section);
        // Contour fecha a família da geometria derivada, e fica ao lado do Pattern on Path pelo
        // mesmo critério: os anéis são desenho, a forma autorada segue intocada. Some inteira
        // sem contour vivo nem seleção que o permita.
        y = self.step(y, Self::contour_section);
        // Effects fica logo depois dos deformadores: os três são não-destrutivos, e a
        // pilha é a generalização deles (ADR-0132).
        y = self.step(y, Self::effects_section);
        // Filters (FX raster, plano 24) — logo após Effects, mas de OUTRA natureza: pixels, não
        // geometria. Some inteira sem forma selecionada e sem filtro vivo.
        y = self.step(y, Self::filters_section);
        y = self.step(y, Self::align_section);
        y = self.step(y, Self::arrange_section);
        // ⚠️ **A última NÃO passa pelo `step`** (ela fecha sem separador de rodapé), então a dobra
        //    dela fecha aqui — pela MESMA porta, nunca por uma cópia. Sem esta linha o escopo
        //    ficaria pendurado e o `Drop` do `SectionFold` gritaria em debug.
        self.open_fold = None;
        let after = self.path_section(y);
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

    /// Seção **BOOLEAN** — ops N-árias sobre as regiões fechadas SELECIONADAS (a de trás
    /// é a base, a da frente doa o estilo) + a linha Compound / Release.
    pub(crate) fn boolean_section(&mut self, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_BOOLEAN,
            tr("panel.vector.section.boolean"),
            y,
        );
        if collapsed {
            return y;
        }
        // **O MODO dos oito botões abaixo** (plano UI/UX W1). Acima deles e não abaixo, porque
        // ele decide o que eles FAZEM: `Off` consome os operandos (o mundo de sempre), `On` cria
        // um grupo cujos filhos se combinam e continuam editáveis.
        let live = crate::state::bool_live_on();
        y = self.segmented(
            tr("panel.vector.bool.live"),
            &[
                (
                    ids::VECTOR_BOOL_LIVE_OFF,
                    tr("panel.vector.bool.live.off"),
                    !live,
                ),
                (
                    ids::VECTOR_BOOL_LIVE_ON,
                    tr("panel.vector.bool.live.on"),
                    live,
                ),
            ],
            y,
        );
        // **O VERBO DESTA FORMA** (Enio, 2026-08-22): *"o modo do boolean é escolhido por shape e
        // na ordem em que aparece na hierarquia atua sobre o resultante das operações pregressas"*.
        // É o compound shape vivo do Illustrator, em que cada componente guarda o seu Shape Mode.
        //
        // ⚠️ **A fileira vem ANTES dos oito botões, e não junto deles.** Estes quatro são uma
        // PROPRIEDADE da forma em mãos; os oito são AÇÕES sobre a seleção (criam ou re-miram o
        // grupo). Vizinhos, leriam-se como uma família de doze, e o artista descobriria pelo
        // efeito que quatro mudam uma forma e oito mexem no grupo inteiro.
        //
        // ⚠️ `None` faz a fileira **não existir**, e a regra inteira mora do lado da shell
        // (`vec_bool_shape`) — inclusive as duas recusas que ela carrega: a BASE não tem verbo, e
        // um grupo numa RECEITA não deixa forma nenhuma escolher.
        if let Some((code, name)) = crate::state::bool_shape_row() {
            // ⚠️ **O rótulo NOMEIA a forma de que fala.** Com o grupo inteiro aceso no canvas
            // (tocar um filho seleciona o grupo — lei do editor), um rótulo genérico não diria de
            // QUAL das formas ele fala, e o artista escolheria o verbo no escuro. Sem `Name` no
            // documento cai-se no genérico, que é o melhor que há a dizer.
            let label = if name.is_empty() {
                tr("panel.vector.bool.shape").to_string()
            } else {
                name
            };
            y = self.segmented(
                &label,
                &[
                    (
                        ids::VECTOR_BOOL_SHAPE_UNION,
                        tr("panel.vector.bool.shape.union"),
                        code == 0,
                    ),
                    (
                        ids::VECTOR_BOOL_SHAPE_SUBTRACT,
                        tr("panel.vector.bool.shape.subtract"),
                        code == 1,
                    ),
                    (
                        ids::VECTOR_BOOL_SHAPE_INTERSECT,
                        tr("panel.vector.bool.shape.intersect"),
                        code == 2,
                    ),
                    (
                        ids::VECTOR_BOOL_SHAPE_EXCLUDE,
                        tr("panel.vector.bool.shape.exclude"),
                        code == 3,
                    ),
                ],
                y,
            );
        }
        // **O Apply só existe com uma booleana viva selecionada.** Sem ela não há o que
        // consolidar, e um botão que não aplica nada é pior que botão nenhum — a mesma lei do
        // Apply da simetria e dos dois botões do corte.
        if crate::state::bool_group_selected() {
            y = self.action_button_kind(
                ids::VECTOR_BOOL_APPLY,
                tr("panel.vector.bool.apply"),
                ButtonKind::Accent,
                y,
            );
        }
        // **As OITO** (plano 25 §8): as quatro de conjunto e as quatro receitas. Duas colunas —
        // a fileira de oito botoes de largura cheia empurrava a linha Compound para fora da vista.
        let ops = [
            (ids::VECTOR_BOOL_UNION, tr("panel.vector.bool.union")),
            (ids::VECTOR_BOOL_SUBTRACT, tr("panel.vector.bool.subtract")),
            (
                ids::VECTOR_BOOL_INTERSECT,
                tr("panel.vector.bool.intersect"),
            ),
            (ids::VECTOR_BOOL_EXCLUDE, tr("panel.vector.bool.exclude")),
            (
                ids::VECTOR_BOOL_MINUS_BACK,
                tr("panel.vector.bool.minus_back"),
            ),
            (ids::VECTOR_BOOL_TRIM, tr("panel.vector.bool.trim")),
            (ids::VECTOR_BOOL_CROP, tr("panel.vector.bool.crop")),
            (ids::VECTOR_BOOL_MERGE, tr("panel.vector.bool.merge")),
        ];
        let gap = Spacing::Sm.px();
        let w = ((self.inner_w - gap) / 2.0).max(1.0);
        for pair in ops.chunks(2) {
            let [a, b] = pair else { continue };
            y = self.row2(w, gap, [(a.0, a.1), (b.0, b.1)], y);
        }
        self.compound_row(y)
    }
}
