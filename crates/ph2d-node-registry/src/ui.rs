//! Node UI side-metadata (Motion Nodes M1.R1).
//!
//! Keyed by `NodeTypeId`, **additive and non-frozen** — it lives beside the ops
//! on [`super::NodeRegistry`], entirely outside the frozen `NodeManifest` (8
//! fields) and its contract gate. The motion-graph editor reads it to label,
//! categorize, and shape each node card; a node registers its UI manifest right
//! next to its op in `register`.
//!
//! Param hints (`ParamUiHint`) + attribute access (`AttrAccess`) land with the
//! params panel (M1.P1) as further additive fields here.

/// Visual category of a node — selects the card's header tint (the `node-cat-*`
/// ColorTokens) so the palette's colors teach the library map (plan §2.4).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeUiCategory {
    /// Generators (grid, random, …) — green.
    Source,
    /// Distribution / cloning — muted green.
    Distribute,
    /// Spatial transforms (move, scale, rotate) — blue.
    Transform,
    /// Falloffs / focus fields — amber.
    Focus,
    /// Color / stylistic effects — magenta.
    Fx,
    /// Terminal / output — red.
    Output,
    /// Values, adapters, utilities — grey.
    Utility,
}

/// Card silhouette — the 7 body shapes (plan §2.4). `Rect` is the neutral
/// modifier default; the others read the node's role at a glance.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeSilhouette {
    /// Rounded rectangle — a modifier (default).
    Rect,
    /// Cigar (fully rounded ends) — merge / group.
    Cigar,
    /// Circle — a terminal / sink.
    Circle,
    /// Diamond — a value / gate.
    Diamond,
    /// Trapezoid widening downward — a source / generator.
    TrapezoidDown,
    /// Trapezoid narrowing downward — a sink.
    TrapezoidUp,
    /// Tabbed rect — an event / signal node.
    Tabbed,
}

/// Per-node-type UI metadata (M1.R1). `display_name` is the English,
/// result-named label (HR-15); `category` + `silhouette` drive the card look.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NodeUiManifest {
    pub display_name: &'static str,
    pub category: NodeUiCategory,
    pub silhouette: NodeSilhouette,
}

/// One named read-channel for a [`ParamWidget::Channels`] picker (M1.P1 · plan §1.1).
/// `label` is the human word the artist picks; `column` is the stream column the text
/// param is set to; `mode` is the sibling f32 param's value (for `value.attribute`,
/// `0` = read the scalar column, `1` = the magnitude of a Vec2 column). `i32` so the
/// enclosing [`ParamWidget`] can stay `Eq`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ReadChannel {
    pub label: &'static str,
    pub column: &'static str,
    pub mode: i32,
}

/// Which widget a param row renders in the params panel (M1.P1). The frozen
/// [`ph2d_nodegraph::node::ParamSpec`] only carries `{name, default: f32}`, so
/// the editable range + control + label live here as additive side-metadata.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ParamWidget {
    /// Continuous float slider.
    Slider,
    /// Whole-number slider — the chip rounds to an integer (count / index).
    IntSlider,
    /// An angle: a numeric box with a `deg` unit chip (type `90deg`, drag-scrub,
    /// stepper). **The param stores DEGREES** — the app's one authored-angle unit
    /// (the Painter's `*_angle_deg` fields, the Inspector's `deg` boxes). Radians
    /// and turns are implementation details of whatever consumes the value, never
    /// of the value itself, so there is nothing for the panel to convert.
    Angle,
    /// On/off toggle (0.0 / 1.0). Rendered as a slider clamped to `0..1` in v1.
    Toggle,
    /// Random seed — a whole-number box plus a **re-roll** button (a seed has no
    /// meaningful drag range; the artist wants "another one", not "a bigger one").
    Seed,
    /// An RGBA colour authored via a single swatch → OKLCH picker (the canonical
    /// colour UI, not four raw channel sliders). Declared on ONE hint whose
    /// `param` anchors the group; `channels` names the four **linear-straight**
    /// RGBA channel params it drives (each a declared [`ph2d_nodegraph::node::ParamSpec`]).
    /// The params panel paints one swatch for the group (suppressing the four
    /// scalar rows); the shell bridge reads the pick back into these params
    /// (sRGB→linear) and seeds the picker (linear→sRGB). Reusable by every colour
    /// node (tint, colour-array, gradient, …).
    Color { channels: [&'static str; 4] },
    /// A single-select from a fixed set of **named** options, rendered as a
    /// segmented-button row (the same selector the Vector panel uses for
    /// Cap / Join / Draw) — never a number slider the user must decode. The param
    /// stores the selected option **index** as its `f32` value (`0..labels.len()`).
    /// Reusable by every enum-valued param (channel, waveform, easing, …).
    Enum { labels: &'static [&'static str] },
    /// A **named-channel picker** for a TEXT param that names a stream column, with a
    /// sibling f32 `mode_param` folded in (plan §1.1). The panel paints the channel
    /// LABELS as a segmented selector plus a trailing "Custom…"; picking a channel sets
    /// the text param to its `column` and the mode param to its `mode`, and only Custom
    /// reveals the raw text field (the power-user escape). It is artist-facing sugar
    /// over [`Self::Text`] — the substrate is untouched, the text param stays the source
    /// of truth — so an artist reads "Speed" instead of typing `vel` and a magic mode.
    Channels {
        mode_param: &'static str,
        channels: &'static [ReadChannel],
    },
    /// A **source picker** for a TEXT param that names a value the APP published into
    /// the graph's external channel (doc 65) — e.g. a drawn vector shape, by the name it
    /// carries in the scene. The panel offers the LIVE list of published names as
    /// clickable chips (populated at runtime from `Cook::externals`) plus a text field
    /// for a name not yet drawn (a forward reference is legal — it is a reference, not a
    /// lookup). Artist-facing sugar over [`Self::Text`]: the artist picks "Star" from the
    /// shapes they drew instead of typing its exact internal name, and the substrate is
    /// untouched — the text param stays the source of truth. Reusable by every node that
    /// reads an external by name (`motion.path`, and later any `*.external` reader).
    Source,
    /// A free-text field editing a **text param** (a `motion.expression` formula) —
    /// NOT a `ParamSpec` (which is f32-only). The hint's `param` names the text-param
    /// key (`Graph::set_text_param`), read/written through the additive text channel
    /// (doc 32), and the panel renders the shared `TextInput` widget. `min/max/step`
    /// are inert for a text field.
    Text,
    /// An interactive **transfer-curve editor** — the sibling of [`Self::Text`],
    /// on the SAME text channel: the hint's `param` names a text-param key whose
    /// value is a `ph2d-curve` serialized curve (`Graph::set_text_param`, doc
    /// 32), never a `ParamSpec` (a curve is not one number). The panel draws the
    /// unit-square editor (draggable points + per-point interp) and writes the
    /// string back on edit. Reusable by every curve-valued transfer — the
    /// `field.remap` **Curve** contour, and (later) `value.curve` / `force.curve`.
    /// `min/max/step` are inert.
    Curve,
    /// An interactive **gradient editor** — the colour sibling of [`Self::Curve`],
    /// on the SAME text channel: the hint's `param` names a text-param key whose
    /// value is a `ph2d_color::ColorRamp` serialized with `serialize_gradient`
    /// (`Graph::set_text_param`, doc 32), never a `ParamSpec` (a multi-stop
    /// gradient is not a fixed set of `f32`s). The panel draws a gradient bar
    /// with **draggable stops** (position via the same `InteractiveState::CurvePoint`
    /// x-drag the curve editor uses) and a per-stop **OKLCH swatch** (the canonical
    /// colour UI, `register_picker_swatch`), plus add/remove and an interp cycle.
    /// A stop's colour is read back through the shell's sRGB↔linear boundary
    /// exactly as [`Self::Color`] is — the string stays the source of truth.
    /// Reusable by every multi-stop-gradient param (`motion.color_ramp` Custom,
    /// and later any colour-over-value node). `min/max/step` are inert.
    Gradient,
    /// An ordered COLOUR PALETTE, carried as a text param
    /// (`ph2d_color::palette_text`), painted as a wrapping strip of swatches with
    /// `+`/`−`. Each swatch opens the OKLCH picker, exactly like [`Self::Color`].
    ///
    /// ⚠️ **Distinct from [`Self::Gradient`] because a palette is a LIST, not samples of
    /// a curve** — it has no positions and no interpolation, and offering either would
    /// put controls on screen that change nothing. The strip WRAPS, so the row has no
    /// length limit of its own: the artist adds colours until they stop wanting more.
    Palette,
}

impl ParamWidget {
    /// Whether the chip displays / commits whole numbers (drives the
    /// integer-snapping link in the params panel).
    #[must_use]
    pub fn is_integer(self) -> bool {
        matches!(self, ParamWidget::IntSlider | ParamWidget::Seed)
    }
}

/// UI hint for one declared param (M1.P1) — the editable range, step, control,
/// and English label for a `ParamSpec` of some node type. Registered as an
/// additive side-table on [`super::NodeRegistry`] (keyed by `NodeTypeId`),
/// entirely outside the frozen `NodeManifest`. A param with no hint falls back
/// to a plain [`ParamWidget::Slider`] over a default range in the panel.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ParamUiHint {
    /// The `ParamSpec::name` this hint annotates.
    pub param: &'static str,
    /// English, result-named label (HR-15).
    pub label: &'static str,
    pub min: f32,
    /// The **soft** maximum: how far the SLIDER drags. Also the typed ceiling
    /// unless a [`ParamHardMax`] widens it.
    pub max: f32,
    pub step: f32,
    pub widget: ParamWidget,
}

/// A param whose typed entry reaches further than its slider drags — Blender's
/// **soft vs hard limits**, and the same reason: the useful drag range and the
/// legal range are different questions. `rate` on `motion.emitter` is the case
/// that forced it — a fountain is authored between 0 and ~12k particles/sec, but
/// a one-frame burst is a legitimate `rate` in the millions paired with a
/// millisecond `life`, and no slider should have to span that to allow it.
///
/// Registered as its own additive side-table rather than a field on
/// [`ParamUiHint`] **for a mechanical reason worth stating**: the hint is a
/// struct literal at 275 sites across 78 crates, and a new field is 275 edits to
/// express one exception. A param with no entry here types to its soft `max`,
/// which is what every existing hint already means — so nothing moves by
/// default.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ParamHardMax {
    /// The `ParamSpec::name` this widens.
    pub param: &'static str,
    /// The typed-entry ceiling. Must be ≥ the hint's `max` to mean anything.
    pub max: f32,
}

/// **A que SEÇÃO do painel um param pertence** — a metade visual do doc 88 B3.
///
/// Um nó com treze params é uma parede de sliders: eles cabem na tela (o teto de linhas é
/// medido contra a altura do dock), e ainda assim não se LEEM. Agrupá-los é o que todo editor
/// profissional faz — e o grupo é uma propriedade do param, não do widget dele.
///
/// ⚠️ Tabela paralela, nunca campo do [`ParamUiHint`], **pela mesma razão mecânica do
/// [`ParamHardMax`]**: o hint é literal `&'static` em ~275 sítios, e um campo novo são 275
/// edições para exprimir o que uma tabela exprime nas linhas que interessam.
///
/// ⚠️ E param sem entrada **não fica num grupo "Outros"** — ele fica ANTES de toda seção, solto,
/// que é onde os essenciais devem estar (o padrão do Blender: o principal na cara, o resto em
/// sub-painéis). Um nó sem tabela nenhuma pinta exatamente como pintava.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ParamGroup {
    /// The `ParamSpec::name` (ou o nome do text param) que esta linha agrupa.
    pub param: &'static str,
    /// O título da seção, em inglês (HR-15: a face do artista sai por i18n no painel).
    pub group: &'static str,
}

impl ParamGroup {
    /// `ParamGroup::new("min", "Range")`.
    ///
    /// A tabela de um nó grande são dezenas de entradas, e a forma literal escreve cada uma em
    /// quatro linhas — uma tabela que o autor do nó não consegue LER de relance é uma tabela
    /// que ele não confere. Um construtor no tipo, e não um helper por-crate, porque seis
    /// cópias de `const fn g(..)` são seis respostas à mesma pergunta.
    #[must_use]
    pub const fn new(param: &'static str, group: &'static str) -> Self {
        Self { param, group }
    }
}

/// **Show a param row only when another param has one of a set of values** — the
/// side-channel that makes a param CONDITIONAL, so a node shows only the controls
/// that apply to its current mode (a `source.shape` gear shows `hole`/`tooth_depth`;
/// a circle shows neither). Side-metadata like [`ParamHardMax`], **never** a field
/// of the frozen `NodeManifest` — a param with no gate here is always shown, which
/// is what every existing param already means, so nothing moves by default.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ParamGate {
    /// The `ParamSpec::name` this gate conditionally SHOWS.
    pub param: &'static str,
    /// The `ParamSpec::name` whose current value decides.
    pub when: &'static str,
    /// The allowed `when` values (rounded to int — an `Enum` stores its index as
    /// f32) for `param` to appear. Empty ⇒ the param is never shown.
    pub values: &'static [i32],
}

/// **What a node PRODUCES, CONSUMES, REQUIRES, or GENERATES on its instance
/// stream** — the semantic side-channel (ADR-0155) that lets the editor detect a
/// node placed where its output is INERT. The canonical case: a `force.*` node is
/// `Pure` and accumulates into the transient column `accel`; only `motion.integrate`
/// (and `sim.step`) consume it, so a force with no integrator downstream writes
/// `accel`, nothing reads it, and the scene stays static — with no error. The
/// substrate has no reachability analysis; this declaration is what makes one
/// possible.
///
/// Side-metadata like [`ParamGate`], **never** a field of the frozen
/// `NodeManifest` — a node with no entry here is semantically neutral (produces,
/// consumes, requires, and generates nothing), which is what every un-annotated
/// node already means, so nothing moves by default.
/// ⚠️ **Sem `PartialEq`/`Eq`, de propósito.** Desde o [`Coupling::ProducesWhen`]
/// um acoplamento pode carregar um ponteiro de função, cujo endereço o compilador
/// não garante único — uma igualdade derivada compararia endereços e daria uma
/// resposta sem significado. Medido: nenhum leitor do repo compara acoplamentos;
/// todos casam por padrão (`matches!`), que é a pergunta que de facto se faz.
#[derive(Copy, Clone, Debug)]
pub enum Coupling {
    /// Writes a transient column that is INERT unless a downstream node
    /// [`Consumes`](Coupling::Consumes) it. `force.*` → `Produces("accel")`;
    /// `field.*` → `Produces("falloff")`; `motion.pin_constraint` →
    /// `Produces("inv_mass")`.
    Produces(&'static str),
    /// Consumes a transient column — the plumbing that makes a producer live.
    /// `motion.integrate`/`sim.step` → `Consumes("accel")`.
    Consumes(&'static str),
    /// Requires a column present on its input to do its job. `motion.bend` →
    /// `Requires("P")` (a deformer with no points is a no-op).
    Requires(&'static str),
    /// Generates a column from nothing (a source). `motion.grid` →
    /// `Generates("P")`.
    Generates(&'static str),
    /// **`Produces`, mas só em parte do espaço de params** — o irmão exato do
    /// [`ph2d_nodegraph::gpu::ApplicableFn`] do `KernelSpec`, e pela mesma razão: um nó cujo modo
    /// decide QUAL coluna ele escreve não pode declarar a produção
    /// incondicionalmente.
    ///
    /// ⚠️ Declarar `Produces("accel")` num nó que só o escreve num modo faria o
    /// diagnóstico marcar TODA instância dele — o falso positivo que o ADR-0155
    /// combateu no Boids, agora vindo da declaração em vez da inferência. E não
    /// declarar nada devolve a classe de erro que o ADR existe para pegar: uma
    /// coluna transiente escrita, nada a consome, e a cena fica parada **sem
    /// erro**.
    ProducesWhen(&'static str, ph2d_nodegraph::gpu::ApplicableFn),
}

impl Coupling {
    /// The column this coupling names, whatever its kind.
    #[must_use]
    pub fn column(self) -> &'static str {
        match self {
            Coupling::Produces(c)
            | Coupling::Consumes(c)
            | Coupling::Requires(c)
            | Coupling::Generates(c)
            | Coupling::ProducesWhen(c, _) => c,
        }
    }
}

/// **O que o card deste nó DIZ** — a única resposta, para os dois lados que a fazem.
///
/// A regra é uma escada de três degraus (doc 61): o nome que o **artista** deu vence; sem ele o
/// card diz o que o nó É (o `display_name` do tipo); sem UI registrada, o `type_name` cru — que
/// é também o que a caixa de rename semeia, então a string que você edita é a que estava lendo.
///
/// ⚠️ Ela mora aqui, e não no painel do grafo onde nasceu, porque ganhou um **segundo
/// consumidor**: o painel de params nomeia o nó que DIRIGE um param, e esse nome tem de ser o
/// mesmo que está escrito no card — senão o artista lê um nome no inspector e procura outro no
/// grafo. Duas cópias da escada divergiriam no dia do rename, que é justamente o caso em que
/// ela existe ([[feedback_two_doors_to_the_same_question_diverge]]).
#[must_use]
pub fn card_title(label: Option<&str>, ui: Option<&NodeUiManifest>, type_name: &str) -> String {
    label
        .map(str::to_string)
        .or_else(|| ui.map(|u| u.display_name.to_string()))
        .unwrap_or_else(|| type_name.to_string())
}
