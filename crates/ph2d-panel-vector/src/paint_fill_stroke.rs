//! ⭐⭐ **Fill e Stroke no layout do Figma** (F44, dono 01/11: *«o MESMO padrão de Effects/Filters»*,
//! referências `img_F44/`).
//!
//! ```text
//! Fill & Stroke                                 +
//! [■]◇ [   2 px    ]◇ [100 %]  [⚙] [👁]◇ [−]       ← um traço (a cor e o Weight)
//! [▤]◇               [ 80 %]  [⚙] [👁]◇ [−]       ← um preenchimento (rampa = degradê)
//! ```
//!
//! ⭐ **F45 (dono 01/11): UMA lista, na ordem da pilha** (o topo em cima) — a de cima pinta por cima,
//! a tinta de base incluída; arrastar pelo ⚙ muda a ordem ([`crate::paint_sections::fill_stroke_list`]).
//!
//! ⭐ **F44b (dono 01/11): TODAS as linhas iguais** — as mesmas colunas, os mesmos botões, e o ⚙ de
//! cada uma abre à ESQUERDA ([`crate::paint_side`]) os ajustes da tinta DELA. A caixa mostra só a cor
//! (o tipo mora no ⚙) e é a amostra inteira.
//!
//! ⭐ **F44c (dono 01/11): a caixa é SÓ a cor** (sem o hex), o Weight de cada traço vem na linha,
//! ao lado da cor, e a Position mora no ⚙ de todo traço. O olho da base guarda a tinta no documento
//! (`VecPath::fill_hidden`); o «−» tira-a, e o «+» do título dá-a de volta. O losango de um campo
//! fica colado ao controlo dele, e a coluna fica reservada mesmo sem canal.

use super::*;
use crate::paint_sections::fx_list::is_open;
use crate::state::PaintRow;
use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::stroke_rounded_rect;
use ph2d_editor_core::widget::{
    ButtonState, ColorSwatch, IconButtonStyle, IconGlyph, NumberInput, SwatchSize, Unit,
    paint_color_swatch, paint_icon_button, paint_number_input_with_buffer, paint_swatch_or_mixed,
};
use ph2d_tokens::{Radius, StrokeToken};
use ph2d_tool_vector::StyleFields;
use std::cell::RefCell;

#[path = "paint_fill_stroke_colunas.rs"]
mod colunas;
pub(super) use colunas::PesoValor;

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
    /// A linha que VOA sob o cursor pinta-se fora do lugar: o que ela deixa aberto não conta.
    pub(super) static VOANDO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Anota o que ficou aberto — salvo na linha que voa.
fn abre(f: impl FnOnce(&mut Abertos)) {
    if !VOANDO.with(std::cell::Cell::get) {
        ABERTOS.with(|a| f(&mut a.borrow_mut()));
    }
}

/// Lê e limpa o que o corpo deixou neste quadro.
pub(crate) fn take_abertos() -> Abertos {
    ABERTOS.with(|a| std::mem::take(&mut *a.borrow_mut()))
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
    /// O Weight, num traço: a coluna entre a cor e o `%`. `None` (um preenchimento) = a cor ocupa
    /// as duas colunas.
    weight: Option<Peso<'a>>,
    /// O olho e o «−». `None` = a seleção não tem UMA resposta (vazia ou múltipla): o botão pinta-se
    /// apagado e sem alvo — um gesto sem sujeito não se oferece.
    eye: Option<(NodeId, IconId)>,
    minus: Option<NodeId>,
    /// Os canais do Auto Key: o da cor (colado à caixa), o do `%` e o do olho (colados a eles).
    /// Vazio = o campo não tem canal próprio — a coluna fica reservada, para alinhar.
    color_keys: &'a [NodeId],
    opacity_keys: &'a [NodeId],
    eye_keys: &'a [NodeId],
}

/// O Weight de uma linha de traço: o campo, o que ele mostra fora do foco e os canais dele.
struct Peso<'a> {
    id: NodeId,
    px: f64,
    mixed: bool,
    keys: &'a [NodeId],
    /// Um token cobre a espessura: o número mostra a autorada, não a que se desenha (rachura).
    token: Option<String>,
}

impl Peso<'_> {
    fn valor(&self) -> PesoValor {
        PesoValor {
            id: self.id,
            px: self.px,
            mixed: self.mixed,
        }
    }
}

impl BodyCtx<'_> {
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

    /// ⭐ **Uma linha**: `[cor]◇ [Weight]◇ [%]◇ [⚙] [👁]◇ [−]`. A caixa é SÓ a cor (abre o seletor,
    /// como no Figma) — o tipo mora nos ajustes (⚙). Devolve o `y` seguinte, a caixa da cor (para a
    /// rachura de um token) e, se os ajustes estão abertos, a linha (a âncora do painel à esquerda).
    fn paint_line(&mut self, l: &Linha<'_>, y: f32) -> (f32, Rect, Option<Rect>) {
        let peso = l.weight.as_ref().map(Peso::valor);
        let c = self.colunas(y, peso);
        // A cor é um QUADRADO em toda linha (dono 01/11); num preenchimento a coluna do Weight
        // fica vazia.
        let caixa = c.caixa;
        self.decorator(c.caixa_key, l.color_keys);
        if let Some(w) = &l.weight {
            self.decorator(c.weight_key, w.keys);
            self.weight_field(w, c.weight);
        }
        self.decorator(c.pct_key, l.opacity_keys);
        self.decorator(c.eye_key, l.eye_keys);
        self.loose_number(l.opacity, l.pct, l.mixed_pct, Unit::Percent, c.pct);
        let open = l.settings.is_some_and(|id| is_open(self.store, id));
        self.column_button(l.settings, IconId::StrokeSettings, c.settings, open);
        let (eye, olho) = l.eye.map_or((None, IconId::Eye), |(id, i)| (Some(id), i));
        self.column_button(eye, olho, c.eye, false);
        self.column_button(l.minus, IconId::Minus, c.minus, false);
        let a11y = tr("panel.vector.appearance.layer_color");
        if l.mixed_color {
            paint_swatch_or_mixed(
                caixa,
                l.swatch,
                a11y,
                None,
                self.store.picker_target() == Some(l.swatch),
                self.scene,
                self.theme,
            );
        } else if let Some(ramp) = l.ramp {
            self.ramp_swatch(caixa, ramp);
        } else {
            paint_color_swatch(
                &ColorSwatch::new(l.swatch, a11y, l.color).size(SwatchSize::Sm),
                caixa,
                self.scene,
                self.theme,
            );
            // FRAME-RAW-OK: o halo da amostra de cor — uma cor escura some no fundo escuro sem ele.
            stroke_rounded_rect(
                self.scene,
                caixa,
                ph2d_editor_core::paint::frame_radius(self.theme, Radius::Sm.px()),
                StrokeToken::Thin.px(),
                resolve(ColorToken::Border, self.theme),
            );
        }
        self.hit_index.register(l.swatch, caixa);
        let lh = self.linha_h(peso);
        let line = Rect::new(self.inner_x, y, self.inner_w, lh);
        (y + lh + self.row_gap, caixa, open.then_some(line))
    }

    /// O Weight de um traço: o número em `px` (a unidade diz o que ele é). ⛔ Um glifo à frente
    /// tirava ao número a largura dele no Inspector estreito (foto F44c: só sobrava o stepper).
    fn weight_field(&mut self, w: &Peso<'_>, r: Rect) {
        self.loose_number(w.id, w.px, w.mixed, Unit::Px, r);
        if w.token.is_some() {
            self.token_slash(r);
        }
    }

    /// A amostra de um degradê: as duas pontas em rampa, na moldura da amostra.
    fn ramp_swatch(&mut self, r: Rect, [a, b]: [[u8; 4]; 2]) {
        use ph2d_vector::{Affine, Brush, Fill, Gradient, Point, RoundedRect};
        let radius = ph2d_editor_core::paint::frame_radius(self.theme, Radius::Sm.px());
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
        // FRAME-RAW-OK: o halo da amostra do degradê, o mesmo da amostra de cor lisa.
        stroke_rounded_rect(
            self.scene,
            r,
            radius,
            StrokeToken::Thin.px(),
            resolve(ColorToken::Border, self.theme),
        );
    }

    /// A linha da camada `i` (índice do DOCUMENTO).
    pub(super) fn layer_line(&mut self, i: usize, row: &PaintRow, y: f32) -> f32 {
        let opacity = ids::vector_paint_row_opacity_id(i);
        let pct = self.live_number(opacity, f64::from(row.opacity) * 100.0); // LITERAL-PX-OK: fraction→percent
        let eye = ids::vector_paint_eye_id(i);
        let PesoValor { id: wid, px, .. } = self.peso_camada(i, row);
        let (y, _, open) = self.paint_line(
            &Linha {
                settings: Some(ids::vector_paint_row_id(i)),
                swatch: ids::vector_paint_swatch_id(i),
                color: row.color,
                ramp: row.ramp,
                mixed_color: row.mixed.color,
                opacity,
                pct,
                mixed_pct: row.mixed.opacity,
                weight: (!row.is_fill).then_some(Peso {
                    id: wid,
                    px,
                    mixed: row.mixed.width,
                    keys: &[wid],
                    token: None,
                }),
                eye: Some((eye, crate::paint_stack_rows::eye_icon(row))),
                minus: Some(ids::vector_paint_del_id(i)),
                color_keys: &[ids::vector_paint_swatch_id(i)],
                opacity_keys: &[opacity],
                eye_keys: &[eye],
            },
            y,
        );
        if let Some(r) = open {
            abre(|a| a.layer = Some((i, r)));
        }
        y
    }

    /// Há onde pôr mais uma camada: há forma selecionada (a aparência publicada) e a pilha está
    /// abaixo do tecto do documento.
    pub(super) fn room_for_layer() -> bool {
        state::current_appearance()
            .is_some_and(|a| a.layers.len() < ph2d_vec_scene::MAX_PAINT_LAYERS)
    }

    /// ⭐ **A linha do preenchimento de BASE** (nada, se a forma não o tem).
    pub(super) fn base_fill_line(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let base = state::base_fill();
        if base == Some(state::BasePaint::Absent) {
            return y;
        }
        let track = self
            .store
            .slider(ph2d_tool_vector::ids::VECTOR_FILL_OPACITY)
            .map_or_else(|| opacity_to_slider(snap.fill[3]), |(_, v)| v);
        let hidden = base == Some(state::BasePaint::Hidden);
        // Na base a cor e a opacidade são o MESMO canal (a tinta): um losango, colado à caixa.
        let (next, swatch, open) = self.paint_line(
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
                weight: None,
                eye: base.map(|_| (crate::ids::VECTOR_FILL_EYE, eye_of(hidden))),
                minus: base.map(|_| crate::ids::VECTOR_FILL_PRESENT),
                color_keys: &[
                    ph2d_tool_vector::ids::VECTOR_FILL_SWATCH,
                    ph2d_tool_vector::ids::VECTOR_FILL_OPACITY,
                ],
                opacity_keys: &[],
                eye_keys: &[crate::ids::VECTOR_FILL_EYE],
            },
            y,
        );
        // ⚠️ **A RACHURA**: com um token a cobrir, a cor da swatch NÃO é a que a arte desenha.
        if state::token_bindings().is_some_and(|b| b.fill.is_some()) {
            self.token_slash(swatch);
        }
        if let Some(r) = open {
            abre(|a| a.fill = Some(r));
        }
        next
    }

    /// As linhas do TRAÇO, na ordem da pilha — o painel da barra ② pinta-as (sem o ⚙ da base, porque
    /// o painel da barra já traz o «Stroke settings» por baixo; e, sem traço, o «Add stroke» que no
    /// Inspector é uma linha do «+» da secção).
    pub(crate) fn stroke_lines(&mut self, snap: &VectorStyleSnapshot, y: f32, in_bar: bool) -> f32 {
        let mut y = y;
        let a = state::current_appearance();
        let pilha = a.as_ref().filter(|a| !a.layers_differ).map_or_else(
            || vec![ph2d_vec_scene::StackItem::BaseStroke],
            state::Appearance::stack,
        );
        for it in pilha.into_iter().rev() {
            match it {
                ph2d_vec_scene::StackItem::BaseStroke => {
                    y = self.base_stroke_line(snap, y, in_bar);
                }
                ph2d_vec_scene::StackItem::Entry(i) => {
                    if let Some(row) = a.as_ref().and_then(|a| a.layers.get(i))
                        && !row.is_fill
                    {
                        y = self.layer_line(i, row, y);
                    }
                }
                ph2d_vec_scene::StackItem::BaseFill => {}
            }
        }
        if in_bar && state::stroke_present() == Some(false) {
            y = self.action_button(
                crate::ids::VECTOR_STROKE_PRESENT,
                tr("panel.vector.stroke.add"),
                y,
            );
        }
        y
    }

    /// ⭐ **A linha do traço de BASE** (nada, se a forma não o tem).
    pub(super) fn base_stroke_line(
        &mut self,
        snap: &VectorStyleSnapshot,
        y: f32,
        in_bar: bool,
    ) -> f32 {
        let base = state::base_stroke();
        if state::stroke_present() == Some(false) {
            return y;
        }
        let track = self
            .store
            .slider(ph2d_tool_vector::ids::VECTOR_STROKE_OPACITY)
            .map_or_else(|| opacity_to_slider(snap.stroke[3]), |(_, v)| v);
        let hidden = base == Some(state::BasePaint::Hidden);
        let PesoValor { id: wid, px, .. } = self.peso_base(snap);
        let (next, swatch, open) = self.paint_line(
            &Linha {
                settings: (!in_bar).then_some(ids::VECTOR_STROKE_SETTINGS),
                swatch: ph2d_tool_vector::ids::VECTOR_STROKE_SWATCH,
                color: snap.stroke,
                ramp: None,
                mixed_color: !snap.agrees(StyleFields::STROKE_RGB),
                opacity: ph2d_tool_vector::ids::VECTOR_STROKE_OPACITY_NUM,
                pct: f64::from(track) * 100.0, // LITERAL-PX-OK: fraction→percent
                mixed_pct: !snap.agrees(StyleFields::STROKE_A),
                weight: Some(Peso {
                    id: wid,
                    px,
                    mixed: !snap.agrees(StyleFields::WIDTH),
                    keys: &[
                        ph2d_tool_vector::ids::VECTOR_WIDTH,
                        ph2d_tool_vector::ids::VECTOR_WIDTH_NUM,
                    ],
                    token: state::token_bindings().and_then(|b| b.width),
                }),
                eye: base.map(|_| (crate::ids::VECTOR_STROKE_EYE, eye_of(hidden))),
                minus: state::stroke_present().map(|_| crate::ids::VECTOR_STROKE_PRESENT),
                color_keys: &[
                    ph2d_tool_vector::ids::VECTOR_STROKE_SWATCH,
                    ph2d_tool_vector::ids::VECTOR_STROKE_OPACITY,
                ],
                opacity_keys: &[],
                eye_keys: &[crate::ids::VECTOR_STROKE_EYE],
            },
            y,
        );
        if state::token_bindings().is_some_and(|b| b.stroke.is_some()) {
            self.token_slash(swatch);
        }
        if let Some(r) = open {
            abre(|a| a.stroke = Some(r));
        }
        next
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
