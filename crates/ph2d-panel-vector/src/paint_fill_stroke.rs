//! ⭐⭐ **Fill e Stroke no layout do Figma** (F44, dono 01/11: *«o MESMO padrão de Effects/Filters»*,
//! referências `img_F44/`).
//!
//! ```text
//! Fill                                          +
//! [■ 808080       ]◇ [100 %]◇ [⚙] [👁]◇ [−]       ← camadas extra (o topo em cima)
//! [▤ 4682E6       ]◇ [ 80 %]  [⚙] [👁]◇ [−]       ← o preenchimento de BASE (rampa = degradê)
//! Stroke                                        +
//! [■ 000000       ]◇ [100 %]  [⚙] [👁]◇ [−]
//! Position           Weight
//! [◎|◎|◎]          ◇ [ 2 px ]◇                    ← a linha FIXA do traço de base
//! ```
//!
//! ⭐ **F44b (dono 01/11): TODAS as linhas iguais** — as mesmas colunas, os mesmos botões, e o ⚙ de
//! cada uma abre à ESQUERDA ([`crate::paint_side`]) os ajustes da tinta DELA. A caixa mostra só a cor
//! e o hex (o tipo mora no ⚙) e é a amostra inteira. O olho da base guarda a tinta no documento
//! (`VecPath::fill_hidden`); o «−» tira-a, e o «+» do título dá-a de volta. O losango de um campo
//! fica colado ao controlo dele, e a coluna fica reservada mesmo sem canal.

use super::*;
use crate::paint_sections::fx_list::is_open;
use crate::state::PaintRow;
use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, stroke_rounded_rect};
use ph2d_editor_core::widget::{
    ButtonState, ColorSwatch, IconButtonStyle, IconGlyph, NumberInput, SwatchSize, Unit,
    paint_color_swatch, paint_icon_button, paint_number_input_with_buffer, paint_swatch_or_mixed,
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

/// Uma linha da lista. ⭐ **A base e as camadas são a MESMA linha** (F44b, dono 01/11): só os ids e
/// os canais do Auto Key mudam.
struct Linha<'a> {
    /// O ⚙ (um `Dropdown`): aberto, os ajustes da tinta pintam-se à ESQUERDA. `None` = sem ajustes
    /// a mostrar (a forma não está selecionada) — o botão pinta-se apagado e não se regista.
    settings: Option<NodeId>,
    swatch: NodeId,
    color: [u8; 4],
    /// As pontas do degradê: a amostra pinta a rampa em vez da cor.
    ramp: Option<[[u8; 4]; 2]>,
    mixed_color: bool,
    /// O campo `%` e o valor que ele mostra fora do foco.
    opacity: NodeId,
    pct: f64,
    mixed_pct: bool,
    /// O olho e o «−». `None` = a seleção não tem UMA resposta (vazia ou múltipla): o botão pinta-se
    /// apagado e sem alvo — um gesto sem sujeito não se oferece.
    eye: Option<(NodeId, IconId)>,
    minus: Option<NodeId>,
    /// Os canais do Auto Key: o da cor (colado à caixa), o do `%` e o do olho (colados a eles).
    /// Vazio = o campo não tem canal próprio — a coluna fica reservada, para alinhar.
    color_keys: &'a [NodeId],
    opacity_keys: &'a [NodeId],
    eye_keys: &'a [NodeId],
    /// A tinta está escondida (o hex a cinzento).
    off: bool,
}

/// ⭐ **As colunas de uma linha**, da direita para a esquerda — as MESMAS em toda linha de Fill e
/// de Stroke (e a linha fixa Position | Weight usa as duas primeiras): `[caixa]◇ [%]◇ [⚙] [👁]◇ [−]`.
#[derive(Clone, Copy)]
pub(super) struct Colunas {
    pub(super) caixa: Rect,
    pub(super) caixa_key: Rect,
    pub(super) pct: Rect,
    pub(super) pct_key: Rect,
    pub(super) settings: Rect,
    pub(super) eye: Rect,
    pub(super) eye_key: Rect,
    pub(super) minus: Rect,
}

impl BodyCtx<'_> {
    /// As colunas da linha em `y`. O losango reserva a largura dele mesmo sem canal (alinhamento).
    pub(super) fn colunas(&mut self, y: f32) -> Colunas {
        let h = self.row_h;
        let mw = if ph2d_editor_core::paint::ui_is_redesign() {
            ph2d_editor_core::widget::DECORATOR_W
        } else {
            0.0
        };
        let right = self.inner_x + self.inner_w;
        // Os três botões encostados (o glifo já tem a folga dele): a caixa do hex fica com a largura.
        let minus = Rect::new(right - h, y, h, h);
        let eye_key = Rect::new(minus.x - mw, y, mw, h);
        let eye = Rect::new(eye_key.x - h, y, h, h);
        let settings = Rect::new(eye.x - h, y, h, h);
        let pct_key = Rect::new(settings.x - mw, y, mw, h);
        let pct_w = self.pct_w();
        let pct = Rect::new(pct_key.x - pct_w, y, pct_w, h);
        let caixa_key = Rect::new(pct.x - mw, y, mw, h);
        let caixa = Rect::new(self.inner_x, y, (caixa_key.x - self.inner_x).max(1.0), h);
        Colunas {
            caixa,
            caixa_key,
            pct,
            pct_key,
            settings,
            eye,
            eye_key,
            minus,
        }
    }

    /// ⭐ **A largura do `%`, MEDIDA** — o maior valor («100 %») com a folga e o stepper do campo, nunca
    /// acima do piso genérico dos números. É o que dá à caixa do hex a largura dos seis dígitos no
    /// Inspector estreito (o piso de `72 px` é do maior número da casa, e aqui o maior é `100`).
    fn pct_w(&mut self) -> f32 {
        let texto = format!("100{}", Unit::Percent.suffix());
        let medido = self.text_system.prefix_width_weighted(
            &texto,
            self.font,
            ph2d_text::FontWeight::MEDIUM,
        );
        let host = Rect::new(0.0, 0.0, self.chip_w, self.row_h);
        let w = medido
            + ph2d_editor_core::widget::field_pad_x() * 2.0
            + ph2d_editor_core::widget::stepper_width(host);
        w.min(self.chip_w)
    }

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
    pub(super) fn loose_number(
        &mut self,
        id: NodeId,
        value: f64,
        mixed: bool,
        unit: Unit,
        rect: Rect,
    ) {
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

    /// A amostra de uma linha: um quadrado dentro da caixa, colado à esquerda dela.
    fn line_swatch(&self, y: f32) -> Rect {
        let pad = Spacing::Xs.px();
        let side = (self.row_h - pad * 2.0).max(1.0);
        Rect::new(self.inner_x + pad, y + pad, side, side)
    }

    /// Um botão de ícone de uma coluna; `id = None` = apagado e sem alvo.
    fn column_button(&mut self, id: Option<NodeId>, icon: IconId, r: Rect, pressed: bool) {
        let visual = match id {
            Some(_) if pressed => (ButtonState::Pressed, 1.0),
            Some(id) => self.store.button_visual(id),
            None => (ButtonState::Disabled, 0.0),
        };
        paint_icon_button(
            r,
            IconGlyph::Builtin(icon),
            IconButtonStyle::Plain,
            visual,
            self.scene,
            self.theme,
        );
        if let Some(id) = id {
            self.hit_index.register(id, r);
        }
    }

    /// ⭐ **Uma linha**: `[■ HEX]◇ [%]◇ [⚙] [👁]◇ [−]`. A caixa inteira é a amostra (abre o seletor
    /// de cor, como no Figma) e mostra SÓ a cor e o hex — o tipo mora nos ajustes (⚙). Devolve o `y`
    /// seguinte e, se os ajustes estão abertos, a linha (a âncora do painel à esquerda).
    fn paint_line(&mut self, l: &Linha<'_>, y: f32) -> (f32, Option<Rect>) {
        let c = self.colunas(y);
        self.decorator(c.caixa_key, l.color_keys);
        self.decorator(c.pct_key, l.opacity_keys);
        self.decorator(c.eye_key, l.eye_keys);
        self.loose_number(l.opacity, l.pct, l.mixed_pct, Unit::Percent, c.pct);
        let open = l.settings.is_some_and(|id| is_open(self.store, id));
        self.column_button(l.settings, IconId::StrokeSettings, c.settings, open);
        let (eye, olho) = l.eye.map_or((None, IconId::Eye), |(id, i)| (Some(id), i));
        self.column_button(eye, olho, c.eye, false);
        self.column_button(l.minus, IconId::Minus, c.minus, false);
        let radius = Radius::Sm.px();
        fill_rounded_rect(
            self.scene,
            c.caixa,
            radius,
            resolve(ColorToken::Bg1, self.theme),
        );
        stroke_rounded_rect(
            self.scene,
            c.caixa,
            radius,
            StrokeToken::Thin.px(),
            resolve(ColorToken::Border, self.theme),
        );
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
        } else if let Some(ramp) = l.ramp {
            self.ramp_swatch(sr, ramp);
        } else {
            paint_color_swatch(
                &ColorSwatch::new(l.swatch, a11y, l.color).size(SwatchSize::Sm),
                sr,
                self.scene,
                self.theme,
            );
        }
        // A caixa inteira é o alvo da amostra: clicar no hex abre o seletor, como no Figma.
        self.hit_index.register(l.swatch, c.caixa);
        let name = if l.mixed_color {
            tr("panel.vector.mixed").to_owned()
        } else {
            hex(l.color)
        };
        let ink = if l.off {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        // O hex no corpo pequeno, como o Figma: seis dígitos cabem na caixa do Inspector estreito.
        let font = TypeToken::Sm.px();
        let tx = sr.x + sr.w + pad;
        paint_text(
            self.text_system,
            self.scene,
            &name,
            tx,
            y + (self.row_h - font) * 0.5,
            font,
            (c.caixa.x + c.caixa.w - pad * 0.5 - tx).max(1.0),
            resolve(ink, self.theme),
        );
        let line = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
        (y + self.row_h + self.row_gap, open.then_some(line))
    }

    /// A amostra de um degradê: as duas pontas em rampa, na moldura da amostra.
    fn ramp_swatch(&mut self, r: Rect, [a, b]: [[u8; 4]; 2]) {
        use ph2d_vector::{Affine, Brush, Fill, Gradient, Point, RoundedRect};
        let radius = Radius::Sm.px();
        let cor = |c: [u8; 4]| {
            ph2d_vector::Color::from_rgba8(c[0], c[1], c[2], c[3]) // LITERAL-COLOR-OK: user-color — the artist's gradient stops
        };
        let g = Gradient::new_linear(
            Point::new(f64::from(r.x), f64::from(r.y)),
            Point::new(f64::from(r.x + r.w), f64::from(r.y)),
        )
        .with_stops([cor(a), cor(b)].as_slice());
        let shape = RoundedRect::new(
            f64::from(r.x),
            f64::from(r.y),
            f64::from(r.x + r.w),
            f64::from(r.y + r.h),
            f64::from(radius),
        );
        self.scene.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Gradient(g),
            None,
            &shape,
        );
        stroke_rounded_rect(
            self.scene,
            r,
            radius,
            StrokeToken::Thin.px(),
            resolve(ColorToken::Border, self.theme),
        );
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
        let eye = ids::vector_paint_eye_id(i);
        let (y, open) = self.paint_line(
            &Linha {
                settings: Some(ids::vector_paint_row_id(i)),
                swatch: ids::vector_paint_swatch_id(i),
                color: row.color,
                ramp: row.ramp,
                mixed_color: row.mixed.color,
                opacity,
                pct,
                mixed_pct: row.mixed.opacity,
                eye: Some((eye, crate::paint_stack_rows::eye_icon(row))),
                minus: Some(ids::vector_paint_del_id(i)),
                color_keys: &[ids::vector_paint_swatch_id(i)],
                opacity_keys: &[opacity],
                eye_keys: &[eye],
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

    /// ⭐ **A secção FILL**: o «+» (uma camada nova no topo, ou a base de volta), as camadas e a
    /// linha de base.
    pub(crate) fn fill_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let base = state::base_fill();
        let (add, live) = if base == Some(state::BasePaint::Absent) {
            (crate::ids::VECTOR_FILL_PRESENT, true)
        } else {
            (ids::VECTOR_PAINT_ADD_FILL, Self::room_for_layer())
        };
        let (mut y, collapsed) = self.list_header(
            ph2d_tool_vector::ids::VECTOR_SECTION_FILL,
            tr("panel.vector.section.fill"),
            add,
            live,
            y,
        );
        if collapsed {
            return y;
        }
        y = self.layer_lines(true, y);
        if base == Some(state::BasePaint::Absent) {
            return y;
        }
        let track = self
            .store
            .slider(ph2d_tool_vector::ids::VECTOR_FILL_OPACITY)
            .map_or_else(|| opacity_to_slider(snap.fill[3]), |(_, v)| v);
        let hidden = base == Some(state::BasePaint::Hidden);
        // Na base a cor e a opacidade são o MESMO canal (a tinta): um losango, colado à caixa.
        let (next, open) = self.paint_line(
            &Linha {
                // Sem forma selecionada não há tipo, degradê nem token a mostrar: o ⚙ apaga.
                settings: (state::current_fill_kind().is_some()
                    || state::token_bindings().is_some())
                .then_some(ids::VECTOR_FILL_ROW),
                swatch: ph2d_tool_vector::ids::VECTOR_FILL_SWATCH,
                color: snap.fill,
                ramp: state::base_fill_ramp(),
                mixed_color: !snap.agrees(StyleFields::FILL_RGB),
                opacity: ph2d_tool_vector::ids::VECTOR_FILL_OPACITY_NUM,
                pct: f64::from(track) * 100.0, // LITERAL-PX-OK: fraction→percent
                mixed_pct: !snap.agrees(StyleFields::FILL_A),
                eye: base.map(|_| (crate::ids::VECTOR_FILL_EYE, eye_of(hidden))),
                minus: base.map(|_| crate::ids::VECTOR_FILL_PRESENT),
                color_keys: &[
                    ph2d_tool_vector::ids::VECTOR_FILL_SWATCH,
                    ph2d_tool_vector::ids::VECTOR_FILL_OPACITY,
                ],
                opacity_keys: &[],
                eye_keys: &[crate::ids::VECTOR_FILL_EYE],
                off: hidden,
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

    /// As linhas do traço — a secção e o painel da barra ② pintam estas mesmas (`in_bar`: sem o ⚙
    /// da base, porque o painel da barra já traz o «Stroke settings» por baixo; e, sem traço, o
    /// «Add stroke» que no Inspector é o «+» do título).
    pub(crate) fn stroke_lines(&mut self, snap: &VectorStyleSnapshot, y: f32, in_bar: bool) -> f32 {
        let mut y = self.layer_lines(false, y);
        let base = state::base_stroke();
        if state::stroke_present() == Some(false) {
            if in_bar {
                y = self.action_button(
                    crate::ids::VECTOR_STROKE_PRESENT,
                    tr("panel.vector.stroke.add"),
                    y,
                );
            }
            return y;
        }
        let track = self
            .store
            .slider(ph2d_tool_vector::ids::VECTOR_STROKE_OPACITY)
            .map_or_else(|| opacity_to_slider(snap.stroke[3]), |(_, v)| v);
        let hidden = base == Some(state::BasePaint::Hidden);
        let (next, open) = self.paint_line(
            &Linha {
                settings: (!in_bar).then_some(ids::VECTOR_STROKE_SETTINGS),
                swatch: ph2d_tool_vector::ids::VECTOR_STROKE_SWATCH,
                color: snap.stroke,
                ramp: None,
                mixed_color: !snap.agrees(StyleFields::STROKE_RGB),
                opacity: ph2d_tool_vector::ids::VECTOR_STROKE_OPACITY_NUM,
                pct: f64::from(track) * 100.0, // LITERAL-PX-OK: fraction→percent
                mixed_pct: !snap.agrees(StyleFields::STROKE_A),
                eye: base.map(|_| (crate::ids::VECTOR_STROKE_EYE, eye_of(hidden))),
                minus: state::stroke_present().map(|_| crate::ids::VECTOR_STROKE_PRESENT),
                color_keys: &[
                    ph2d_tool_vector::ids::VECTOR_STROKE_SWATCH,
                    ph2d_tool_vector::ids::VECTOR_STROKE_OPACITY,
                ],
                opacity_keys: &[],
                eye_keys: &[crate::ids::VECTOR_STROKE_EYE],
                off: hidden,
            },
            y,
        );
        let swatch = self.line_swatch(y);
        if state::token_bindings().is_some_and(|b| b.stroke.is_some()) {
            self.token_slash(swatch);
        }
        if let Some(r) = open {
            ABERTOS.with(|a| a.borrow_mut().stroke = Some(r));
        }
        self.stroke_position_row(snap, next)
    }
}

/// O olho de uma tinta de base.
fn eye_of(hidden: bool) -> IconId {
    if hidden {
        IconId::EyeClosed
    } else {
        IconId::Eye
    }
}
