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
    Button, ButtonKind, ButtonState, ColorSwatch, SectionHeader, SwatchSize, paint_button,
    paint_color_swatch, paint_section_header,
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
}

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
        let header = SectionHeader::new(id, label).collapsible(!collapsed);
        let rect = Rect::new(self.inner_x, y, self.inner_w, header_h);
        paint_section_header(&header, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
        (y + header_h + SECTION_LABEL_TO_CONTROL_PX, collapsed)
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
        // O AUTO LAYOUT vem logo DEPOIS da moldura, e a ordem é o assunto: a seção Frame diz o
        // que o contêiner É, esta diz o que ele FAZ com o conteúdo. Some inteira quando não há
        // moldura na seleção nem um filho dentro de um fluxo.
        y = self.step(y, |b, y| b.layout_section(y));
        // AS ÂNCORAS vêm logo a seguir ao layout, e são a OUTRA metade da mesma pergunta
        // (*o que este filho faz quando a moldura muda de tamanho?*): o layout responde por quem
        // está num fluxo, esta por quem não está. Elas nunca aparecem juntas — a shell só publica
        // a regra para um filho cujo pai NÃO flui.
        y = self.step(y, |b, y| b.anchors_section(y));
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
        self.path_section(y)
    }

    /// Um passo do corpo: pinta a seção e, **se ela pintou** (o `y` andou), fecha com o
    /// separador. As seções condicionais (Vertex, Align, Text, Axes, Fill Type) somem
    /// sem deixar uma linha órfã boiando no painel — que é o motivo de o separador ser
    /// decidido AQUI, no orquestrador, e não dentro de cada seção.
    pub(crate) fn step(&mut self, y: f32, f: impl FnOnce(&mut Self, f32) -> f32) -> f32 {
        let after = f(self, y);
        if after > y {
            self.separator(after)
        } else {
            after
        }
    }

    /// Width + Stroke swatch + Stroke opacity + Cap / Join + Dash / Gap.
    pub(crate) fn stroke_style(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_STROKE,
            tr("panel.vector.section.stroke"),
            y,
        );
        if collapsed {
            return y;
        }
        // Width slider + px chip.
        let track = self
            .store
            .slider(ids::VECTOR_WIDTH)
            .map(|(_, v)| v)
            .unwrap_or_else(|| px_to_slider(snap.stroke_width_px));
        let px = self
            .store
            .number_value(ids::VECTOR_WIDTH_NUM)
            .unwrap_or(snap.stroke_width_px);
        y = self.slider_row(
            "Width",
            ids::VECTOR_WIDTH,
            ids::VECTOR_WIDTH_NUM,
            track,
            px,
            &format!("{}", px.round() as i64),
            y,
        );

        let swatch_w = SwatchSize::Md.px();
        // Stroke colour swatch.
        paint_text(
            self.text_system,
            self.scene,
            tr("panel.vector.section.stroke"),
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, self.theme),
        );
        let stroke_swatch_rect = Rect::new(
            self.inner_x + self.inner_w - swatch_w,
            y,
            swatch_w,
            self.row_h,
        );
        let stroke_swatch =
            ColorSwatch::new(ids::VECTOR_STROKE_SWATCH, "Stroke color", snap.stroke)
                .size(SwatchSize::Md);
        paint_color_swatch(&stroke_swatch, stroke_swatch_rect, self.scene, self.theme);
        self.hit_index
            .register(ids::VECTOR_STROKE_SWATCH, stroke_swatch_rect);
        // A rachura, pela mesma razão do preenchimento.
        let bindings = crate::state::token_bindings();
        if bindings.as_ref().is_some_and(|b| b.stroke.is_some()) {
            self.token_slash(stroke_swatch_rect);
        }
        y += self.row_h + self.row_gap;

        // **O TOKEN do traço** (plano UI/UX W4), logo abaixo da cor que ele cobre. Só é oferecido
        // quando a seleção TEM traço: o token colore o traço que existe e nunca inventa largura.
        if let Some(b) = bindings.filter(|b| b.stroke_exists) {
            y = self.token_row(ids::VECTOR_TOKEN_STROKE, 1, b.stroke.as_deref(), y);
        }

        // Stroke Opacity slider (single source of the stroke alpha).
        let track = self
            .store
            .slider(ids::VECTOR_STROKE_OPACITY)
            .map(|(_, v)| v)
            .unwrap_or_else(|| opacity_to_slider(snap.stroke[3]));
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent for the opacity chip
        y = self.slider_row(
            "Opacity",
            ids::VECTOR_STROKE_OPACITY,
            ids::VECTOR_STROKE_OPACITY_NUM,
            track,
            pct,
            &format!("{}", pct.round() as i64),
            y,
        );

        // Cap / Join segmented rows.
        y = self.segmented3(
            "Cap",
            [
                (ids::VECTOR_CAP_BUTT, "Butt", snap.cap == StrokeCap::Butt),
                (ids::VECTOR_CAP_ROUND, "Round", snap.cap == StrokeCap::Round),
                (
                    ids::VECTOR_CAP_SQUARE,
                    "Square",
                    snap.cap == StrokeCap::Square,
                ),
            ],
            y,
        );
        y = self.segmented3(
            "Join",
            [
                (
                    ids::VECTOR_JOIN_MITER,
                    "Miter",
                    snap.join == StrokeJoin::Miter,
                ),
                (
                    ids::VECTOR_JOIN_ROUND,
                    "Round",
                    snap.join == StrokeJoin::Round,
                ),
                (
                    ids::VECTOR_JOIN_BEVEL,
                    "Bevel",
                    snap.join == StrokeJoin::Bevel,
                ),
            ],
            y,
        );

        // **ALINHAMENTO** — logo abaixo de Cap/Join porque é a mesma família (*como a caneta
        // se comporta*), e ACIMA do tracejado, que é sobre o PADRÃO e não sobre o lado.
        y = self.segmented3(
            "Align",
            [
                (
                    ids::VECTOR_ALIGN_CENTRE,
                    "Centre",
                    snap.align == StrokeAlign::Centre,
                ),
                (
                    ids::VECTOR_ALIGN_INNER,
                    "Inner",
                    snap.align == StrokeAlign::Inner,
                ),
                (
                    ids::VECTOR_ALIGN_OUTER,
                    "Outer",
                    snap.align == StrokeAlign::Outer,
                ),
            ],
            y,
        );

        // Dash length (multiple of width; 0 = solid).
        let track = self
            .store
            .slider(ids::VECTOR_DASH)
            .map(|(_, v)| v)
            .unwrap_or_else(|| dash_to_slider(snap.dash));
        let px = self
            .store
            .number_value(ids::VECTOR_DASH_NUM)
            .unwrap_or(snap.dash);
        y = self.slider_row(
            "Dash",
            ids::VECTOR_DASH,
            ids::VECTOR_DASH_NUM,
            track,
            px,
            &format!("{}", px.round() as i64),
            y,
        );

        // Gap length between dashes.
        let track = self
            .store
            .slider(ids::VECTOR_GAP)
            .map(|(_, v)| v)
            .unwrap_or_else(|| gap_to_slider(snap.gap));
        let px = self
            .store
            .number_value(ids::VECTOR_GAP_NUM)
            .unwrap_or(snap.gap);
        y = self.slider_row(
            "Gap",
            ids::VECTOR_GAP,
            ids::VECTOR_GAP_NUM,
            track,
            px,
            &format!("{}", px.round() as i64),
            y,
        );

        // As PONTAS do traço (Start / End). Ficam aqui, e não no catálogo de formas,
        // porque uma ponta é propriedade do stroke: herda a cor e a largura dele e gira
        // com a tangente da curva — a quarta decisão sobre como a caneta termina, ao lado
        // de cap, join e dash. Desenho em `paint_markers` (teto de LOC por arquivo).
        self.marker_rows(snap, y)
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
            y = self.token_row(ids::VECTOR_TOKEN_FILL, 0, b.fill.as_deref(), y);
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

    /// Seção **ARRANGE** — Duplicate + z-order (2×2) + Flip + Rotate. Age sobre o path
    /// selecionado (comandos de documento pelo dreno da shell).
    pub(crate) fn arrange_section(&mut self, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_ARRANGE,
            tr("panel.vector.section.arrange"),
            y,
        );
        if collapsed {
            return y;
        }
        y = self.action_button(ids::VECTOR_ARRANGE_DUPLICATE, "Duplicate", y);
        // Z-order: 2×2 grid — To Back | To Front · Backward | Forward.
        let zorder = [
            (ids::VECTOR_ARRANGE_TO_BACK, "To Back"),
            (ids::VECTOR_ARRANGE_TO_FRONT, "To Front"),
            (ids::VECTOR_ARRANGE_BACKWARD, "Backward"),
            (ids::VECTOR_ARRANGE_FORWARD, "Forward"),
        ];
        let z_cols = 2usize;
        let z_gap = Spacing::Sm.px();
        let z_w = ((self.inner_w - z_gap * (z_cols as f32 - 1.0)) / z_cols as f32).max(1.0);
        let z_top = y;
        for (i, (id, label)) in zorder.iter().enumerate() {
            let rx = self.inner_x + (i % z_cols) as f32 * (z_w + z_gap);
            let ry = z_top + (i / z_cols) as f32 * (self.row_h + z_gap);
            let rect = Rect::new(rx, ry, z_w, self.row_h);
            let bstate = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
            let btn = Button::new(*id, *label)
                .kind(ButtonKind::Default)
                .state(bstate);
            paint_button(&btn, rect, self.scene, self.text_system, self.theme);
            self.hit_index.register(*id, rect);
        }
        let z_rows = zorder.len().div_ceil(z_cols) as f32;
        y = z_top + z_rows * self.row_h + (z_rows - 1.0) * z_gap + self.row_gap;

        // Flip (mirror) + Rotate (90°) — each a 2-col row of action buttons.
        y = self.row2(
            z_w,
            z_gap,
            [
                (ids::VECTOR_ARRANGE_FLIP_H, "Flip H"),
                (ids::VECTOR_ARRANGE_FLIP_V, "Flip V"),
            ],
            y,
        );
        self.row2(
            z_w,
            z_gap,
            [
                (ids::VECTOR_ARRANGE_ROTATE_CW, "Rotate CW"),
                (ids::VECTOR_ARRANGE_ROTATE_CCW, "Rotate CCW"),
            ],
            y,
        )
    }
}
