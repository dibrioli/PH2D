//! Params-panel state channels (Motion Nodes M1.P1) — the publish/return seam
//! between the shell bridge and this panel (mold of the graph panel's
//! `snapshot` + `intent` channels).
//!
//! The bridge builds a [`ParamsSnapshot`] each frame from the selected node's
//! manifest params + the registry's `ParamUiHint`s + the graph's per-instance
//! overrides, and hands it over [`set_current_params`]; `paint` reads it with
//! [`current_params`]. A row edit returns as a [`MotionParamIntent`] the bridge
//! drains + applies (`Graph::set_param`). Neither side downcasts the other.

use std::cell::RefCell;

/// One editable param row, resolved to primitives the panel paints without
/// touching the registry / graph:
/// - `Scalar` — a slider + numeric chip.
/// - `Color` — a swatch -> OKLCH picker (folds four RGBA channel params).
/// - `Toggle` — a real checkbox (never a 0/1 slider).
/// - `Enum` — a named segmented-button selector (never a number slider).
/// - `Angle` — a numeric box with a `deg` unit chip (never a raw-radians slider).
/// - `Seed` — a whole-number box + a re-roll button (never a slider to drag).
#[derive(Clone, Debug, PartialEq)]
pub enum ParamRow {
    Scalar(ScalarRow),
    Color(ColorRow),
    Toggle(ToggleRow),
    Enum(EnumRow),
    Angle(AngleRow),
    Seed(SeedRow),
    Text(TextRow),
    Curve(CurveRow),
    Gradient(GradientRow),
    Palette(PaletteRow),
    Steps(StepsRow),
    Channels(ChannelsRow),
    Source(SourceRow),
}

/// A **named-channel picker** row (plan §1.1) — segmented channel buttons plus a
/// trailing "Custom…", the artist-facing face of a stream-column TEXT param.
///
/// `selected` is the channel index, or `channels.len()` for **Custom** (the live
/// value matches no channel, so the raw text field is shown to edit `custom`).
/// Picking channel `i` writes `text_param = channels[i].column` and `mode_param =
/// channels[i].mode` (two intents); Custom writes only `text_param` (from the field).
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelsRow {
    pub label: String,
    /// The text param the chosen column lands in (`Graph::set_text_param`).
    pub text_param: &'static str,
    /// The sibling f32 param a channel also sets (`Graph::set_param`).
    pub mode_param: &'static str,
    /// One `(label, column, mode)` per channel — resolved primitives, so the panel
    /// stays registry-free. "Custom…" is appended by the paint, not stored here.
    pub channels: Vec<(&'static str, &'static str, i32)>,
    /// `channels.len()` = Custom (no channel matches the live column + mode).
    pub selected: usize,
    /// The live text-param value — shown in (and edited via) the Custom field.
    pub custom: String,
    /// The scalar columns the UPSTREAM stream actually carries right now (minus the
    /// ones the curated channels already cover) — the roadmap's *dropdown populated
    /// at runtime*. Shown as clickable chips under the Custom field so the artist
    /// picks a real column (`id`, `Index`, a custom attribute) by name instead of
    /// guessing it blind. Empty when nothing upstream cooked → just the text field.
    pub extra: Vec<String>,
}

/// A **source picker** row (doc 65) — clickable chips of the names the app has
/// published into the graph's external channel (drawn shapes), plus a text field for a
/// name not yet drawn. The artist-facing face of a TEXT param that references an
/// external by name; picking a chip writes `param = options[j]`, the field writes it raw.
///
/// `current` highlights the matching chip (or fills the field when it matches none). The
/// substrate is untouched — the text param stays the source of truth, so this is pure UI
/// sugar over [`TextRow`], the same way [`ChannelsRow`] is for a stream column.
#[derive(Clone, Debug, PartialEq)]
pub struct SourceRow {
    /// English label (from the `ParamUiHint`), e.g. "Shape".
    pub label: String,
    /// The text param the chosen name lands in (`Graph::set_text_param`).
    pub param: &'static str,
    /// The live published names (`Cook::externals` keys) — clickable chips. Empty when
    /// the app has published nothing yet → just the text field (the honest escape).
    pub options: Vec<String>,
    /// The current text-param value — highlights the matching chip and fills the field.
    pub current: String,
}

/// An interactive **transfer-curve editor** (A1) — a `ph2d-curve` serialized in a
/// text param (`Graph::set_text_param`), drawn as a graph with **draggable control
/// points** (the foundational `InteractiveState::CurvePoint` 2-D drag, reused from
/// the Painter's falloff editor) + add/remove buttons. Like [`TextRow`] the value is
/// a `String`, so an edit rides [`MotionParamIntent::SetTextParam`]; unlike it, the
/// artist never sees the string — only the curve and the handles.
#[derive(Clone, Debug, PartialEq)]
pub struct CurveRow {
    /// The text-param key (`Graph::set_text_param`) — echoed in the intent.
    pub name: &'static str,
    /// English label (from the `ParamUiHint`).
    pub label: String,
    /// The current serialized curve (the text-param override, else empty → the
    /// panel opens on the identity diagonal).
    pub value: String,
}

/// Uma LISTA DE NÚMEROS ordenada — uma faixa de barras arrastáveis com `+`/`−`, carregada
/// num text param (`ph2d_steps::format`). A gêmea NUMÉRICA de [`PaletteRow`]: o valor é
/// uma `String`, então tanto o arrasto quanto o `+`/`−` viajam por
/// [`MotionParamIntent::SetTextParam`].
///
/// ⚠️ **Sem campo de comprimento, e não falta nenhum:** a contagem É
/// `ph2d_steps::parse(value).len()`. Um número à parte seria uma segunda resposta a
/// *quantos passos existem*.
#[derive(Clone, Debug, PartialEq)]
pub struct StepsRow {
    /// The text-param key — echoed in the intent + the per-bar ids.
    pub name: &'static str,
    /// English label (from the `ParamUiHint`).
    pub label: String,
    /// A lista serializada (`ph2d_steps::format`); vazia = nada autorado.
    pub value: String,
    /// A faixa em que uma barra é lida e escrita — a MESMA que os sliders escalares do nó
    /// declaram no hint. ⚠️ Nunca auto-ajustada ao conteúdo: um strip que se re-escala
    /// durante o arrasto não acompanha o dedo.
    pub min: f32,
    pub max: f32,
}

/// An ordered COLOUR PALETTE — a wrapping strip of OKLCH swatches with `+`/`−`, carried
/// in a text param (`ph2d_color::palette_text`). Like [`GradientRow`] the value is a
/// `String`, so add/remove rides [`MotionParamIntent::SetTextParam`] and a colour edit
/// rides the shell's picker read-back.
///
/// ⚠️ **No length field, and none is missing:** the count IS `parse_palette(value).len()`.
/// A separate number would be a second answer to *how many colours are there*, and the
/// one this wave removed.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteRow {
    /// The text-param key — echoed in the intent + the per-swatch ids.
    pub name: &'static str,
    /// English label (from the `ParamUiHint`).
    pub label: String,
    /// The serialized palette (empty → the node's factory colours).
    pub value: String,
}

/// An interactive **gradient editor** (doc 85) — the colour sibling of [`CurveRow`]: a
/// `ph2d_color::ColorRamp` serialized in a text param (`Graph::set_text_param`,
/// `serialize_gradient`), drawn as a gradient BAR with **draggable position markers** (the
/// same `InteractiveState::CurvePoint` x-drag the curve editor uses, y ignored) + a
/// per-stop **OKLCH swatch** (`register_picker_swatch`, the shell reads the pick back into
/// the string) + add/remove and an interp cycle. Like [`CurveRow`] the value is a `String`,
/// so a position/add/remove edit rides [`MotionParamIntent::SetTextParam`]; a colour edit
/// rides the shell's picker read-back (mirror of [`ColorRow`]). The artist never sees the
/// string — only the bar and the stops.
#[derive(Clone, Debug, PartialEq)]
pub struct GradientRow {
    /// The text-param key (`Graph::set_text_param`) — echoed in the intent + the
    /// per-stop swatch ids ([`param_grad_swatch_id`]).
    pub name: &'static str,
    /// English label (from the `ParamUiHint`).
    pub label: String,
    /// The current serialized gradient (the text-param override, else empty → the
    /// panel opens on the default black→white ramp).
    pub value: String,
}

/// A free-text row editing a **text param** (a `motion.expression` formula) — the
/// shared single-line `TextInput` widget. The value is a `String` (not a number), so it
/// rides a dedicated [`MotionParamIntent::SetTextParam`] rather than the `f64` `SetParam`.
#[derive(Clone, Debug, PartialEq)]
pub struct TextRow {
    /// The text-param key (`Graph::set_text_param`) — echoed in the intent.
    pub name: &'static str,
    /// English label (from the `ParamUiHint`).
    pub label: String,
    /// The current formula (the text-param override, else empty).
    pub value: String,
}

/// An angle row: a number box with a `deg` unit chip. **Degrees end to end** —
/// the app's one authored-angle unit, so the param already stores what the box
/// shows and there is nothing to convert in either direction. Whatever consumes
/// the value (a cycle-based trig, a rotation basis) converts at its own edge.
#[derive(Clone, Debug, PartialEq)]
pub struct AngleRow {
    /// Canonical `ParamSpec::name` — echoed back in [`MotionParamIntent::SetParam`].
    pub name: &'static str,
    /// English label (from the `ParamUiHint`).
    pub label: String,
    /// Current value, in degrees.
    pub deg: f64,
    pub min_deg: f64,
    pub max_deg: f64,
    pub step_deg: f64,
}

/// A random-seed row: a whole-number box plus a re-roll button. A seed has no
/// meaningful magnitude, so it is never a slider — the artist wants *another*
/// seed, not a *bigger* one.
#[derive(Clone, Debug, PartialEq)]
pub struct SeedRow {
    pub name: &'static str,
    pub label: String,
    /// Current seed (a whole number).
    pub value: f64,
    pub min: f64,
    pub max: f64,
}

/// A checkbox row for a boolean param (`>= 0.5` = on).
#[derive(Clone, Debug, PartialEq)]
pub struct ToggleRow {
    pub name: &'static str,
    pub label: String,
    pub on: bool,
}

/// A named single-select row. `selected` is the current option index (the param
/// value rounded); `labels` are the option captions painted as segmented buttons.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumRow {
    pub name: &'static str,
    pub label: String,
    pub selected: usize,
    pub labels: &'static [&'static str],
}

/// **The display FACE of a scalar row's numbers** (doc 88, Wave A) — the one
/// place a stored quantity becomes the number the artist reads, and the receipt
/// for getting back.
///
/// ⚠️ **Every numeric field of [`ScalarRow`] is ALREADY in this face.** The
/// bridge converts the WHOLE tuple in one call ([`ScalarRow::in_display`]),
/// because converting some fields and not others is exactly how a range stops
/// containing its value — the *lying widget* the params bridge's `contain()`
/// exists to prevent, arriving through the other door: `normalized_track` clamps
/// to the track end, the panel paints the clamped number, and the first touch
/// writes it back.
///
/// The store stays in the quantity's own unit (metres for a length) and only this
/// boundary converts, because the cook must not depend on a project setting — see
/// `ph2d_node_registry::unit`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RowDisplay {
    /// `display = stored × scale`. Exactly `1.0` unless the param is a world
    /// LENGTH and the project shows pixels; every other quantity is stored in the
    /// unit it is shown in.
    pub scale: f64,
    /// Shown after the number (`""` when the quantity has no suffix). `'static`
    /// because every face is one of a fixed few — no per-frame allocation.
    pub suffix: &'static str,
}

impl Default for RowDisplay {
    /// Unitless and unscaled — what every param means until a node declares one,
    /// so nothing moves by default.
    fn default() -> Self {
        Self {
            scale: 1.0,
            suffix: "",
        }
    }
}

impl RowDisplay {
    /// A face with a checked scale.
    ///
    /// A non-finite or non-positive scale falls back to the neutral `1.0`: it
    /// would otherwise reach `to_stored` as a division that sends the artist's
    /// number to `inf`/`NaN` **and writes it into the document**. Refusing here
    /// means the boundary can never be the thing that poisons a param.
    #[must_use]
    pub fn new(scale: f64, suffix: &'static str) -> Self {
        Self {
            scale: if scale.is_finite() && scale > 0.0 {
                scale
            } else {
                1.0
            },
            suffix,
        }
    }

    /// **The one door back** — the number the DOCUMENT stores, for a number the
    /// artist typed or dragged.
    ///
    /// Every emit site in `events.rs` goes through it, so a row's value cannot
    /// reach `Graph::set_param` still wearing the artist's face.
    #[must_use]
    pub fn to_stored(self, displayed: f64) -> f64 {
        displayed / self.scale
    }
}

/// A slider + numeric-chip row for one scalar `ParamSpec`.
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarRow {
    /// Canonical `ParamSpec::name` — echoed back in [`MotionParamIntent::SetParam`].
    pub name: &'static str,
    /// English label (from the `ParamUiHint`, else the param name).
    pub label: String,
    /// Current value: the per-instance override if set, else the manifest default.
    pub value: f64,
    pub min: f64,
    /// The **hard** minimum: how far DOWN a typed value may go. Equals
    /// [`Self::min`] unless the node registered a `ParamHardMin` — the floor twin
    /// of [`Self::hard_max`], and the half that did not exist until doc 88: the
    /// ceiling shipped alone, so the box could never reach *below* the slider's
    /// start. That asymmetry is what stopped an artist typing `0.001` into a param
    /// whose useful drag begins at `0.01`.
    pub hard_min: f64,
    /// The **soft** maximum: how far the slider drags.
    pub max: f64,
    /// The **hard** maximum: how far a TYPED value may go. Equals [`Self::max`]
    /// unless the node registered a `ParamHardMax` (Blender's soft/hard limits).
    /// The slider cannot represent anything above `max`, so a value up here is
    /// authored by the box and the box alone — see the params panel's paint.
    pub hard_max: f64,
    pub step: f64,
    /// The chip snaps to whole numbers (count / index / seed).
    pub integer: bool,
    /// **A wire is driving this param** (doc 58) — `value` is the live number coming down it.
    ///
    /// The row is then READ-ONLY: it paints the number and registers no widget at all. A
    /// knob that still turns under the finger while a wire decides the value is a control
    /// that lies once per drag — and dimming it would not help, because a dimmed widget
    /// still dispatches ([[feedback_disabled_button_still_dispatches]]).
    ///
    /// ⚠️ **É o NOME do card que dirige, não um booleano** (doc 88 B3). A row lia
    /// *"este número não é seu"* e parava aí: o artista via um valor apagado sem widget e
    /// tinha de ir caçar o fio no grafo para saber de onde ele vem. O nome estava na mão o
    /// tempo todo — `param_sources` resolve o nó fonte e o bridge o descartava.
    ///
    /// E é `Option<String>` e não um par `bool` + `Option<String>` porque os dois campos
    /// seriam **duas cópias do mesmo fato**, livres para discordar; aqui *dirigido* é
    /// exatamente *tem um nome*.
    pub driven_by: Option<String>,
    /// The face every number above is already wearing, and the way back to the
    /// document. [`RowDisplay::default`] (unitless, unscaled) for a param that
    /// declared no unit — which is every param until a node opts in.
    pub display: RowDisplay,
}

impl ScalarRow {
    /// **Re-express the WHOLE numeric tuple in a display face** — the one door
    /// from stored to shown.
    ///
    /// Taking `self` and returning it is what makes the bridge write
    /// `ScalarRow { … }.in_display(face)` as a single expression covering both
    /// arms of its `match`, so there is one conversion site and not one per arm.
    ///
    /// ⚠️ `step` converts too. A step of `0.01 m` is a step of `1 px`; leaving it
    /// behind would make the chip's stepper walk a hundredth of a pixel.
    #[must_use]
    pub fn in_display(mut self, display: RowDisplay) -> Self {
        let s = display.scale;
        self.value *= s;
        self.min *= s;
        self.hard_min *= s;
        self.max *= s;
        self.hard_max *= s;
        self.step *= s;
        self.display = display;
        self
    }
}

/// **How a scalar row's number READS** — the one formatter for a row's value.
///
/// Both painters call it (the chip's `display_override` and the driven row's
/// accent text), because the same quantity must not wear two faces: a Gap that
/// says `100 px` unwired and `100` with a wire plugged in reads as the value
/// having changed when only its author did.
///
/// Wraps the app-wide [`format_number`](ph2d_editor_core::widget::format_number)
/// rather than formatting itself — a second number formatter is a second way to
/// round.
#[must_use]
pub fn scalar_text(row: &ScalarRow, value: f64) -> String {
    let n = ph2d_editor_core::widget::format_number(value);
    if row.display.suffix.is_empty() {
        n
    } else {
        format!("{n} {}", row.display.suffix)
    }
}

/// A colour-swatch row driving four **linear-straight** RGBA channel params
/// (the canonical colour UI). `srgb` is the bridge-converted display colour
/// (linear→sRGB) the panel paints + the shell seeds the OKLCH picker with;
/// `channels` are the params a pick writes back to (sRGB→linear). The swatch's
/// widget id is [`param_swatch_id`]`(channels[0])`.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorRow {
    /// English label (from the `ParamUiHint`).
    pub label: String,
    /// The four RGBA channel param names, in order — echoed to the bridge.
    pub channels: [&'static str; 4],
    /// sRGB8 (straight) for painting the swatch + seeding the picker.
    pub srgb: [u8; 4],
}

/// The selected node's params, resolved for the panel (M1.P1).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParamsSnapshot {
    /// `NodeId.0` of the selected node (echoed in the intent so a stale edit
    /// never lands on a different node).
    pub node: u32,
    /// The node's display name (panel header).
    pub title: String,
    pub rows: Vec<ParamRow>,
    /// Os params deste nó que carregam um **override** — isto é, que o artista mexeu.
    ///
    /// ⚠️ **Um conjunto por NÓ, não um campo por ROW**, e a razão é que a pergunta é do nó:
    /// *quais params deste nó saíram do default?* Um `modified: bool` em cada uma das doze
    /// structs de row seriam doze cópias da mesma resposta, e doze sítios de construção a
    /// manter de acordo — o mesmo motivo por que a unidade e o teto duro moram em tabelas
    /// paralelas em vez de campos do `ParamUiHint`.
    ///
    /// ⚠️ E é PRESENÇA DE CHAVE, nunca `valor != default`: o grafo guarda overrides esparsos,
    /// então um param digitado de volta ao número do default continua sendo uma escolha do
    /// artista — e uma comparação de `f32` responderia outra coisa.
    pub modified: std::collections::BTreeSet<String>,
    /// **Onde cada SEÇÃO começa** — `(título, índice da primeira row do grupo)`, em ordem.
    ///
    /// ⚠️ Não é um campo `group` em cada row (seriam doze structs e doze sítios de construção),
    /// e não é um mapa param→grupo (o painter teria de re-derivar as fronteiras): é exatamente
    /// o que o pintor precisa saber — *antes da row `i`, desenhe o cabeçalho `t`*. As rows já
    /// chegam ORDENADAS por grupo, com as soltas primeiro; quem ordena é a ponte, uma vez.
    ///
    /// Vazio = lista plana, que é como todo nó sem tabela de grupos pinta — e como TODOS
    /// pintavam antes do doc 88 B3.
    pub sections: Vec<(String, usize)>,
}

/// A param edit the panel asks the shell to apply (M1.P1). Tagged with the node
/// id + canonical param name so it is unambiguous even if the selection changed
/// between the edit and the drain.
#[derive(Clone, Debug, PartialEq)]
pub enum MotionParamIntent {
    SetParam {
        node: u32,
        param: &'static str,
        value: f64,
    },
    /// A **text** param edit (a formula) — carries a `String` (the `f64` `SetParam` cannot).
    /// The bridge applies it via `Graph::set_text_param`.
    SetTextParam {
        node: u32,
        param: &'static str,
        value: String,
    },
    /// **Devolve o param ao default do nó** — a ponte chama `Graph::clear_param` E
    /// `clear_text_param` para o mesmo nome.
    ///
    /// ⚠️ Os dois, de propósito: um nome viaja por UM dos canais, nunca pelos dois, e o painel
    /// não precisa saber por qual. Enumerar "este é de texto, aquele é de f32" no lado da UI é a
    /// lista que apodrece no dia em que um param muda de canal — e a §5 do CLAUDE.md registra
    /// exatamente essa migração acontecendo (o gradiente do `color_ramp`, a paleta do
    /// `color_array`). Limpar o que não existe é um no-op barato.
    ResetParam { node: u32, param: String },
}

thread_local! {
    static CURRENT: RefCell<Option<ParamsSnapshot>> = const { RefCell::new(None) };
    static INTENTS: RefCell<Vec<MotionParamIntent>> = const { RefCell::new(Vec::new()) };
}

/// Publish the selected node's params (shell bridge → panel). `None` when no
/// single node is selected or the Motion tool is inactive.
pub fn set_current_params(snapshot: Option<ParamsSnapshot>) {
    CURRENT.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the published params (panel `paint` / `apply_event`).
pub(crate) fn current_params() -> Option<ParamsSnapshot> {
    CURRENT.with(|c| c.borrow().clone())
}

/// Queue a param edit for the bridge to apply (panel → shell).
pub(crate) fn push_param_intent(intent: MotionParamIntent) {
    INTENTS.with(|c| c.borrow_mut().push(intent));
}

/// Drain the queued param edits (shell bridge, each frame). Capacity-retaining.
pub fn drain_param_intents() -> Vec<MotionParamIntent> {
    INTENTS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

#[path = "snapshot_ids.rs"]
mod ids;
pub(crate) use ids::{
    CHANNELS_EXTRA_BASE, MAX_CURVE_POINTS, MAX_GRADIENT_STOPS, param_checkbox_id, param_chip_id,
    param_curve_add_id, param_curve_editor_id, param_curve_interp_id, param_curve_point_id,
    param_curve_remove_id, param_enum_id, param_grad_add_id, param_grad_editor_id,
    param_grad_hue_id, param_grad_interp_id, param_grad_preset_id, param_grad_remove_id,
    param_grad_space_id, param_grad_stop_id, param_number_id, param_pal_add_id,
    param_pal_remove_id, param_reroll_id, param_reset_id, param_slider_id, param_steps_add_id,
    param_steps_bar_id, param_steps_editor_id, param_steps_remove_id, param_text_id,
};
pub use ids::{
    MAX_ENUM_OPTIONS, MAX_PARAM_ROWS, param_grad_swatch_id, param_pal_swatch_id, param_swatch_id,
};

impl ParamRow {
    /// Os params que ESTA row edita — um só na maioria, quatro numa cor, dois num picker de canal.
    ///
    /// ⚠️ Existe porque *"esta row está modificada?"* e *"o que resetar quando clicarem a seta?"*
    /// são a MESMA pergunta, e uma segunda lista de nomes ao lado do pintor divergiria no dia em
    /// que uma row passar a dobrar mais um param. O `match` é exaustivo de propósito: uma variante
    /// nova não compila até dizer que params ela edita.
    #[must_use]
    pub fn params(&self) -> Vec<&str> {
        match self {
            Self::Scalar(r) => vec![r.name],
            Self::Toggle(r) => vec![r.name],
            Self::Enum(r) => vec![r.name],
            Self::Angle(r) => vec![r.name],
            Self::Seed(r) => vec![r.name],
            Self::Text(r) => vec![r.name],
            Self::Curve(r) => vec![r.name],
            Self::Gradient(r) => vec![r.name],
            Self::Palette(r) => vec![r.name],
            Self::Steps(r) => vec![r.name],
            Self::Source(r) => vec![r.param],
            // Uma cor é QUATRO params (o swatch dobra RGBA), então resetá-la é resetar os quatro.
            Self::Color(r) => r.channels.to_vec(),
            // E um picker de canal é dois: a coluna (texto) e o modo (f32) que a acompanha.
            Self::Channels(r) => vec![r.text_param, r.mode_param],
        }
    }
}
