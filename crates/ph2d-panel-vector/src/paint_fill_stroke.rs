//! ⭐⭐ **Fill e Stroke no layout do Figma** (F44, dono 01/11: *«o MESMO padrão de Effects/Filters»*,
//! referências `img_F44/`).
//!
//! ```text
//! Fill                                     +
//! [■ 808080              ] [100 %]  👁  −  ◇   ← camadas extra (o topo em cima)
//! [■ Linear              ] [ 80 %]         ◇   ← o preenchimento de BASE
//! Stroke                                   +
//! [■ 000000              ] [100 %]      −  ◇
//! Position          Weight
//! [▭|▭|▭]           [ 2 px ]           [⚙] ◇   ← a linha FIXA; ⚙ abre «Stroke settings»
//! ```
//!
//! A caixa de uma linha é um `Dropdown`: aberta, as propriedades dela pintam-se à ESQUERDA do
//! Inspector ([`crate::paint_side`]) — o tipo, o degradê e o token do preenchimento de base; a
//! largura, o deslocamento, a quina e a mistura de uma camada.
//!
//! ⚠️ **A base não tem olho**: o documento não guarda «base escondida» (`fill`/`stroke` são
//! `Option`), e um olho que não esconde nada seria um controlo morto. O «−» do traço de base é o
//! «tem traço» de sempre (`VECTOR_STROKE_PRESENT`), e o «+» do título dá-o de volta.

use super::*;
use crate::paint_sections::fx_list::is_open;
use crate::state::PaintRow;
use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, stroke_rounded_rect};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_icon_group;
use ph2d_editor_core::widget::{
    ButtonState, ColorSwatch, IconButtonStyle, IconGlyph, NumberInput, SwatchSize, Unit,
    form_row_columns, paint_color_swatch, paint_icon_button, paint_number_input_with_buffer,
    paint_swatch_or_mixed,
};
use ph2d_tokens::{Radius, StrokeToken, TypeToken};
use ph2d_tool_vector::StyleFields;
use std::cell::RefCell;

/// O que o corpo deixou ABERTO neste quadro, para o passe diferido ([`crate::paint_side`]).
/// ⚠️ Sobrescreve (não acumula): o corpo pode pintar-se mais de uma vez por quadro.
#[derive(Clone, Copy, Default)]
pub(crate) struct Abertos {
    /// A linha do preenchimento de base.
    pub fill: Option<Rect>,
    /// A camada extra aberta (índice do DOCUMENTO) e a caixa dela.
    pub layer: Option<(usize, Rect)>,
    /// O «Stroke settings»: a âncora é a linha fixa.
    pub stroke: Option<Rect>,
}

thread_local! {
    static ABERTOS: RefCell<Abertos> = RefCell::new(Abertos::default());
}

/// Lê e limpa o que o corpo deixou neste quadro.
pub(crate) fn take_abertos() -> Abertos {
    ABERTOS.with(|a| std::mem::take(&mut *a.borrow_mut()))
}

/// O texto de uma caixa de cor: o hexadecimal, como o Figma.
fn hex(c: [u8; 4]) -> String {
    format!("{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

/// Uma linha da lista: o que muda entre a base e uma camada.
struct Linha<'a> {
    /// O `Dropdown` da caixa (abre o painel à esquerda); `None` = caixa sem painel.
    open: Option<NodeId>,
    swatch: NodeId,
    color: [u8; 4],
    mixed_color: bool,
    name: &'a str,
    /// O campo `%` e o valor que ele mostra fora do foco.
    opacity: NodeId,
    pct: f64,
    mixed_pct: bool,
    eye: Option<(NodeId, IconId)>,
    minus: Option<NodeId>,
    /// A opacidade é um canal PRÓPRIO (uma camada): losango colado ao campo `%`. O losango da
    /// coluna (a cor) pinta-o quem chama, com os ids à vista do censo.
    opacity_key: bool,
    /// A camada está desligada (o nome a cinzento).
    off: bool,
}

impl BodyCtx<'_> {
    /// O cabeçalho com o «+» (botão simples: o verbo é o id).
    fn list_header(
        &mut self,
        section: NodeId,
        label: &str,
        add: NodeId,
        live: bool,
        y: f32,
    ) -> (f32, bool) {
        self.with_rows(|rc| {
            rc.section_header_with_action(section, label, y, Some((add, IconId::Plus, false, live)))
        })
    }

    /// Um campo `%` (ou `px`) solto, pintado do valor que o chamador traz — salvo com foco, quando
    /// o buffer do store manda (a lei do [`Self::live_number`]).
    fn loose_number(&mut self, id: NodeId, value: f64, mixed: bool, unit: Unit, rect: Rect) {
        let (state, _, buffer, caret, anchor) =
            ph2d_editor_core::widget::showcase::read_number_input(self.store, id);
        let focused = self.store.focus_id() == Some(id);
        let input = NumberInput::new(id, "", value)
            .visual((state, self.store.hover_live(id)))
            .suffix(Some(unit.suffix()));
        paint_number_input_with_buffer(
            &input,
            focused.then_some(buffer),
            mixed && !focused,
            caret,
            anchor,
            rect,
            self.scene,
            self.text_system,
            self.theme,
        );
        self.hit_index.register(id, rect);
    }

    /// A swatch de uma linha: um quadrado dentro da caixa, colado à esquerda dela.
    fn line_swatch(&self, y: f32) -> Rect {
        let pad = Spacing::Xs.px();
        let side = (self.row_h - pad * 2.0).max(1.0);
        Rect::new(self.inner_x + pad, y + pad, side, side)
    }

    /// ⭐ **Uma linha**: `[■ nome] [%] 👁 − ◇`. Devolve o `y` seguinte e, se o painel dela está
    /// aberto, a caixa (a âncora do painel). A swatch fica DENTRO da caixa e ganha o clique dela.
    fn paint_line(&mut self, l: &Linha<'_>, y: f32) -> (f32, Option<Rect>) {
        let h = self.row_h;
        let gap = Spacing::Xs.px();
        let (w, _) = form_row_columns(self.inner_x, self.inner_w, y, h);
        // ⭐ Cada canal tem o losango dele: o do olho e o da opacidade (de uma camada) colados ao
        // controlo, como o olho da pilha sempre teve.
        let mw = if ph2d_editor_core::paint::ui_is_redesign() {
            ph2d_editor_core::widget::DECORATOR_W
        } else {
            0.0
        };
        let mut x = self.inner_x + w;
        for slot in [l.minus.map(|id| (id, IconId::Minus)), l.eye] {
            x -= h;
            if let Some((id, icon)) = slot {
                if icon != IconId::Minus {
                    x -= mw;
                    self.decorator(Rect::new(x + h, y, mw, h), &[id]);
                }
                let r = Rect::new(x, y, h, h);
                paint_icon_button(
                    r,
                    IconGlyph::Builtin(icon),
                    IconButtonStyle::Plain,
                    self.store.button_visual(id),
                    self.scene,
                    self.theme,
                );
                self.hit_index.register(id, r);
            }
            x -= gap;
        }
        if l.opacity_key {
            x -= mw;
            self.decorator(Rect::new(x, y, mw, h), &[l.opacity]);
        }
        x -= self.chip_w;
        let pr = Rect::new(x, y, self.chip_w, h);
        self.loose_number(l.opacity, l.pct, l.mixed_pct, Unit::Percent, pr);
        x -= gap;
        let field = Rect::new(self.inner_x, y, (x - self.inner_x).max(1.0), h);
        let open = l.open.is_some_and(|id| is_open(self.store, id));
        let (bg, edge) = if open {
            (ColorToken::AccentSoft, ColorToken::Accent)
        } else {
            (ColorToken::Bg1, ColorToken::Border)
        };
        let radius = Radius::Sm.px();
        fill_rounded_rect(self.scene, field, radius, resolve(bg, self.theme));
        stroke_rounded_rect(
            self.scene,
            field,
            radius,
            StrokeToken::Thin.px(),
            resolve(edge, self.theme),
        );
        if let Some(id) = l.open {
            self.hit_index.register(id, field);
        }
        let pad = Spacing::Xs.px();
        let sr = self.line_swatch(y);
        let a11y = tr("panel.vector.appearance.layer_color");
        if l.mixed_color {
            paint_swatch_or_mixed(
                sr,
                l.swatch,
                a11y,
                None,
                self.store.picker_target() == Some(l.swatch),
                self.scene,
                self.theme,
            );
        } else {
            paint_color_swatch(
                &ColorSwatch::new(l.swatch, a11y, l.color).size(SwatchSize::Sm),
                sr,
                self.scene,
                self.theme,
            );
        }
        self.hit_index.register(l.swatch, sr);
        let ink = if l.off {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        let tx = sr.x + sr.w + ph2d_tokens::icon_label_gap_px();
        paint_text(
            self.text_system,
            self.scene,
            l.name,
            tx,
            y + (h - self.font) * 0.5,
            self.font,
            (field.x + field.w - pad - tx).max(1.0),
            resolve(ink, self.theme),
        );
        (y + h + self.row_gap, open.then_some(field))
    }

    /// As camadas EXTRA de uma espécie (`fills`), do topo para o chão — ou «Mixed layers» quando as
    /// pilhas da seleção têm estruturas diferentes (a linha `2` de uma não é a `2` da outra).
    fn layer_lines(&mut self, fills: bool, mut y: f32) -> f32 {
        let Some(a) = state::current_appearance() else {
            return y;
        };
        if a.layers_differ {
            return self.hint_line(tr("panel.vector.appearance.mixed_layers"), y);
        }
        for i in (0..a.layers.len()).rev() {
            let row = &a.layers[i];
            if row.is_fill == fills {
                y = self.layer_line(i, row, y);
            }
        }
        y
    }

    /// A linha da camada `i` (índice do DOCUMENTO).
    fn layer_line(&mut self, i: usize, row: &PaintRow, y: f32) -> f32 {
        let opacity = ids::vector_paint_row_opacity_id(i);
        let pct = self.live_number(opacity, f64::from(row.opacity) * 100.0); // LITERAL-PX-OK: fraction→percent
        let name = if row.mixed.color {
            tr("panel.vector.mixed").to_owned()
        } else {
            hex(row.color)
        };
        let (_, dot) = form_row_columns(self.inner_x, self.inner_w, y, self.row_h);
        self.decorator(dot, &[ids::vector_paint_swatch_id(i)]);
        let (y, open) = self.paint_line(
            &Linha {
                open: Some(ids::vector_paint_row_id(i)),
                swatch: ids::vector_paint_swatch_id(i),
                color: row.color,
                mixed_color: row.mixed.color,
                name: &name,
                opacity,
                pct,
                mixed_pct: row.mixed.opacity,
                eye: Some((
                    ids::vector_paint_eye_id(i),
                    crate::paint_stack_rows::eye_icon(row),
                )),
                minus: Some(ids::vector_paint_del_id(i)),
                opacity_key: true,
                off: !row.enabled,
            },
            y,
        );
        if let Some(r) = open {
            ABERTOS.with(|a| a.borrow_mut().layer = Some((i, r)));
        }
        y
    }

    /// Há onde pôr mais uma camada: há forma selecionada (a aparência publicada) e a pilha está
    /// abaixo do tecto do documento.
    fn room_for_layer() -> bool {
        state::current_appearance()
            .is_some_and(|a| a.layers.len() < ph2d_vec_scene::MAX_PAINT_LAYERS)
    }

    /// ⭐ **A secção FILL**: o «+» (uma camada nova no topo), as camadas e a linha de base.
    pub(crate) fn fill_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let (mut y, collapsed) = self.list_header(
            ph2d_tool_vector::ids::VECTOR_SECTION_FILL,
            tr("panel.vector.section.fill"),
            ids::VECTOR_PAINT_ADD_FILL,
            Self::room_for_layer(),
            y,
        );
        if collapsed {
            return y;
        }
        y = self.layer_lines(true, y);
        let kind = state::current_fill_kind();
        let mixed = !snap.agrees(StyleFields::FILL_RGB);
        let name = match kind {
            Some(state::FillKind::Linear) => tr("panel.vector.arrange.linear").to_owned(),
            Some(state::FillKind::Radial) => tr("panel.vector.arrange.radial").to_owned(),
            Some(state::FillKind::MultiPoint) => tr("panel.vector.arrange.multi").to_owned(),
            Some(state::FillKind::Pattern) => tr("panel.vector.arrange.pattern").to_owned(),
            _ if mixed => tr("panel.vector.mixed").to_owned(),
            _ => hex(snap.fill),
        };
        let track = self
            .store
            .slider(ph2d_tool_vector::ids::VECTOR_FILL_OPACITY)
            .map_or_else(|| opacity_to_slider(snap.fill[3]), |(_, v)| v);
        // Na base a cor e a opacidade são o MESMO canal (a tinta): um losango, na coluna.
        let (_, dot) = form_row_columns(self.inner_x, self.inner_w, y, self.row_h);
        self.decorator(
            dot,
            &[
                ph2d_tool_vector::ids::VECTOR_FILL_SWATCH,
                ph2d_tool_vector::ids::VECTOR_FILL_OPACITY,
            ],
        );
        let (next, open) = self.paint_line(
            &Linha {
                // Sem forma selecionada não há tipo, degradê nem token a mostrar: a caixa não abre.
                open: (kind.is_some() || state::token_bindings().is_some())
                    .then_some(ids::VECTOR_FILL_ROW),
                swatch: ph2d_tool_vector::ids::VECTOR_FILL_SWATCH,
                color: snap.fill,
                mixed_color: mixed,
                name: &name,
                opacity: ph2d_tool_vector::ids::VECTOR_FILL_OPACITY_NUM,
                pct: f64::from(track) * 100.0, // LITERAL-PX-OK: fraction→percent
                mixed_pct: !snap.agrees(StyleFields::FILL_A),
                eye: None,
                minus: None,
                opacity_key: false,
                off: false,
            },
            y,
        );
        // ⚠️ **A RACHURA**: com um token a cobrir, a cor da swatch NÃO é a que a arte desenha.
        let swatch = self.line_swatch(y);
        if state::token_bindings().is_some_and(|b| b.fill.is_some()) {
            self.token_slash(swatch);
        }
        if let Some(r) = open {
            ABERTOS.with(|a| a.borrow_mut().fill = Some(r));
        }
        next
    }

    /// ⭐ **A secção STROKE**: o «+» (dá o traço de base, ou uma camada nova), as linhas e a fixa.
    pub(crate) fn stroke_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let sem_traco = state::stroke_present() == Some(false);
        let (add, live) = if sem_traco {
            // `Some(false)` só é publicado com uma forma selecionada.
            (crate::ids::VECTOR_STROKE_PRESENT, true)
        } else {
            (ids::VECTOR_PAINT_ADD_STROKE, Self::room_for_layer())
        };
        let (y, collapsed) = self.list_header(
            ph2d_tool_vector::ids::VECTOR_SECTION_STROKE,
            tr("panel.vector.section.stroke"),
            add,
            live,
            y,
        );
        if collapsed {
            return y;
        }
        // F16: com o painel do traço aberto por baixo da barra ②, os controlos moram LÁ neste quadro
        // — um id pintado em dois sítios faria o arrasto seguir o rect do outro.
        if crate::bars::stroke_popover_open(self.store) {
            return self.hint_line(tr("panel.vector.stroke.in_bar"), y);
        }
        self.stroke_lines(snap, y, false)
    }

    /// As linhas do traço — a secção e o painel da barra ② pintam estas mesmas (`in_bar`: sem o
    /// botão de ajustes, porque o painel da barra já traz o «Stroke settings» por baixo; e, sem
    /// traço, o «Add stroke» que no Inspector é o «+» do título).
    pub(crate) fn stroke_lines(&mut self, snap: &VectorStyleSnapshot, y: f32, in_bar: bool) -> f32 {
        let mut y = self.layer_lines(false, y);
        let present = state::stroke_present();
        if present == Some(false) {
            if in_bar {
                y = self.action_button(
                    crate::ids::VECTOR_STROKE_PRESENT,
                    tr("panel.vector.stroke.add"),
                    y,
                );
            }
            return y;
        }
        let mixed = !snap.agrees(StyleFields::STROKE_RGB);
        let name = if mixed {
            tr("panel.vector.mixed").to_owned()
        } else {
            hex(snap.stroke)
        };
        let track = self
            .store
            .slider(ph2d_tool_vector::ids::VECTOR_STROKE_OPACITY)
            .map_or_else(|| opacity_to_slider(snap.stroke[3]), |(_, v)| v);
        let (_, dot) = form_row_columns(self.inner_x, self.inner_w, y, self.row_h);
        self.decorator(
            dot,
            &[
                ph2d_tool_vector::ids::VECTOR_STROKE_SWATCH,
                ph2d_tool_vector::ids::VECTOR_STROKE_OPACITY,
            ],
        );
        let (next, _) = self.paint_line(
            &Linha {
                open: None,
                swatch: ph2d_tool_vector::ids::VECTOR_STROKE_SWATCH,
                color: snap.stroke,
                mixed_color: mixed,
                name: &name,
                opacity: ph2d_tool_vector::ids::VECTOR_STROKE_OPACITY_NUM,
                pct: f64::from(track) * 100.0, // LITERAL-PX-OK: fraction→percent
                mixed_pct: !snap.agrees(StyleFields::STROKE_A),
                eye: None,
                minus: (present == Some(true)).then_some(crate::ids::VECTOR_STROKE_PRESENT),
                opacity_key: false,
                off: false,
            },
            y,
        );
        let swatch = self.line_swatch(y);
        if state::token_bindings().is_some_and(|b| b.stroke.is_some()) {
            self.token_slash(swatch);
        }
        self.stroke_position_row(snap, next, !in_bar)
    }

    /// ⭐ **A linha FIXA**: Position (Inside · Center · Outside, em glifos) · Weight · ⚙.
    fn stroke_position_row(&mut self, snap: &VectorStyleSnapshot, y: f32, settings: bool) -> f32 {
        let h = self.row_h;
        let gap = Spacing::Xs.px();
        let font = TypeToken::Sm.px();
        let (w, _) = form_row_columns(self.inner_x, self.inner_w, y, h);
        let mut right = self.inner_x + w;
        if settings {
            right -= h + gap;
        }
        let half = ((right - self.inner_x - gap) / 2.0).max(1.0);
        let wx = self.inner_x + half + gap;
        // As legendas por cima, como no Figma.
        let ink = resolve(ColorToken::Text2, self.theme);
        for (key, x) in [
            ("panel.vector.stroke.align", self.inner_x),
            ("panel.vector.stroke.weight", wx),
        ] {
            paint_text(self.text_system, self.scene, tr(key), x, y, font, half, ink);
        }
        let y = y + font + Spacing::Xs.px();
        // Dois canais, dois losangos: a ESPESSURA na coluna da linha, a POSIÇÃO colada aos glifos.
        let (_, dot) = form_row_columns(self.inner_x, self.inner_w, y, h);
        self.decorator(
            dot,
            &[
                ph2d_tool_vector::ids::VECTOR_WIDTH,
                ph2d_tool_vector::ids::VECTOR_WIDTH_NUM,
            ],
        );
        let mw = if ph2d_editor_core::paint::ui_is_redesign() {
            ph2d_editor_core::widget::DECORATOR_W
        } else {
            0.0
        };
        let seg_w = (half - mw).max(1.0);
        self.decorator(
            Rect::new(self.inner_x + seg_w, y, mw, h),
            &[
                ph2d_tool_vector::ids::VECTOR_ALIGN_INNER,
                ph2d_tool_vector::ids::VECTOR_ALIGN_CENTRE,
                ph2d_tool_vector::ids::VECTOR_ALIGN_OUTER,
            ],
        );
        let agrees = snap.agrees(StyleFields::ALIGN);
        paint_segmented_icon_group(
            Rect::new(self.inner_x, y, seg_w, h),
            &[
                (
                    IconId::StrokeInside,
                    agrees && snap.align == StrokeAlign::Inner,
                    ph2d_tool_vector::ids::VECTOR_ALIGN_INNER,
                ),
                (
                    IconId::StrokeCenter,
                    agrees && snap.align == StrokeAlign::Centre,
                    ph2d_tool_vector::ids::VECTOR_ALIGN_CENTRE,
                ),
                (
                    IconId::StrokeOutside,
                    agrees && snap.align == StrokeAlign::Outer,
                    ph2d_tool_vector::ids::VECTOR_ALIGN_OUTER,
                ),
            ],
            self.scene,
            self.text_system,
            self.theme,
            self.store,
            self.hit_index,
        );
        let wid = ph2d_tool_vector::ids::VECTOR_WIDTH_NUM;
        let px = self.store.number_value(wid).unwrap_or(snap.stroke_width_px);
        let wr = Rect::new(wx, y, half, h);
        self.loose_number(wid, px, !snap.agrees(StyleFields::WIDTH), Unit::Px, wr);
        // O TOKEN da espessura risca o número: ele mostra a largura autorada, não a que se desenha.
        if state::token_bindings().is_some_and(|b| b.width.is_some()) {
            self.token_slash(wr);
        }
        if settings {
            let id = ids::VECTOR_STROKE_SETTINGS;
            let open = is_open(self.store, id);
            let r = Rect::new(right + gap, y, h, h);
            let visual = if open {
                (ButtonState::Pressed, 1.0)
            } else {
                self.store.button_visual(id)
            };
            paint_icon_button(
                r,
                IconGlyph::Builtin(IconId::StrokeSettings),
                IconButtonStyle::Plain,
                visual,
                self.scene,
                self.theme,
            );
            self.hit_index.register(id, r);
            if open {
                let line = Rect::new(self.inner_x, y, self.inner_w, h);
                ABERTOS.with(|a| a.borrow_mut().stroke = Some(line));
            }
        }
        y + h + self.row_gap
    }
}
