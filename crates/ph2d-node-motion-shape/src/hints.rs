//! Os **hints de UI** desta forma — que widget cada param veste, e em que ordem
//! o painel os pinta.
//!
//! Saíram do `lib.rs` no teto de LOC, por assunto: o pai fica com **o que a forma
//! É** (o manifesto, o descritor, a chave de conteúdo) e este irmão com **como
//! ela se AUTORA**. `pub(crate)` porque só o `register` os consome.
//!
//! ⚠️ **A tabela de VISIBILIDADE mudou-se para cá na wave dos knobs de forma** (doc 89
//! folha 14), e pelo mesmo corte: *que widget cada param veste* e *quando ele aparece* são
//! a mesma pergunta — a de como a forma se autora —, e mantê-las em arquivos diferentes
//! fazia com que acrescentar um param exigisse duas viagens ao `lib.rs`.

use super::{KIND_LABELS, ShapeKind, param};
use ph2d_node_registry::{ParamGate, ParamUiHint, ParamWidget};

/// The param rows: a real dropdown for the shape family (the segmented `Enum`
/// widget the Vector panel uses for Cap/Join), then the geometry sliders. Every
/// row past `size` is gated by [`PARAM_GATES`], so the panel shows ONLY the
/// controls the current `kind` uses.
pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    // **O TRAÇO** (doc 89 folha 14, P0) — o controle que separa *forma* de
    // *silhueta*. `0` = sem traço ⇒ a forma que sempre shipou.
    ParamUiHint {
        param: param::STROKE_WIDTH,
        label: "Stroke Width",
        min: 0.0,
        max: 1.0,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **Um SWATCH, nunca quatro sliders lineares** — a lei que o `motion.tint`
    // escreve ao lado do dele: *"nunca sliders lineares crus, um `0.5` linear lê
    // como cinza claro"*. A hint ancora no primeiro canal e nomeia os quatro; o
    // bridge lê o pick de volta (sRGB→linear).
    ParamUiHint {
        param: param::STROKE_R,
        label: "Stroke",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: [
                param::STROKE_R,
                param::STROKE_G,
                param::STROKE_B,
                param::STROKE_A,
            ],
        },
    },
    ParamUiHint {
        param: param::KIND,
        label: "Shape",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Enum {
            labels: KIND_LABELS,
        },
    },
    ParamUiHint {
        param: param::SIZE,
        label: "Size",
        min: 0.05,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::ASPECT,
        label: "Aspect (H/W)",
        min: 0.1,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::SIDES,
        label: "Sides / Points / Teeth",
        min: 3.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: param::CORNER,
        label: "Corner Radius",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::STAR_DEPTH,
        label: "Point Depth",
        min: 0.05,
        max: 0.95,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::CLEFT,
        label: "Cleft",
        min: 0.02,
        max: 0.45,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::TOOTH_DEPTH,
        label: "Tooth Depth",
        min: 0.05,
        max: 0.6,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::HOLE,
        label: "Hole",
        min: 0.0,
        max: 0.9,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **A família do círculo** (doc 89 folha 14). O `Sweep` para em 360 porque uma
    // volta é o máximo que a forma tem; o `Start` **dá a volta INTEIRA** porque girar o
    // começo é o gesto, e parar em 359 faria o slider bater numa parede invisível.
    ParamUiHint {
        param: param::SWEEP,
        label: "Sweep",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::START,
        label: "Start",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    // O miolo para em `0,95`: a `1` a rosquinha tem espessura zero e desaparece — o
    // mesmo teto do `Hole` da engrenagem, pela mesma razão.
    ParamUiHint {
        param: param::INNER,
        label: "Inner",
        min: 0.0,
        max: 0.95,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **Os desvios por canto vão de −1 a 1, e o NEGATIVO é metade do gesto:** eles
    // somam-se ao `Corner`, então um desvio negativo é como se AFIA aquele canto sozinho
    // (o `round_rect_radii` clampa a soma em zero). Um slider de 0 a 1 daria só metade da
    // faixa que a biblioteca aceita.
    ParamUiHint {
        param: param::CORNER_TR,
        label: "Corner TR",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::CORNER_BR,
        label: "Corner BR",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::CORNER_BL,
        label: "Corner BL",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::SMOOTHING,
        label: "Smoothing",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

/// **Per-kind visibility** — a param appears only when `kind` is one of the listed
/// values (the enum indices from [`ShapeKind`]). `kind` and `size` have no gate
/// (always shown). This is the SAME per-kind truth the builder keys off, so a
/// shown control is a control that does something.
pub(crate) static PARAM_GATES: &[ParamGate] = &[
    // ⚠️ `aspect` vale para TUDO que é cortado de uma caixa, e as trinta e cinco
    // formas do catálogo são: a receita as corta de `[-s,-ry]..[s,ry]`. Deixá-las
    // de fora esconderia um controlo VIVO — o inverso exato do botão morto, e o
    // gate `no_kind_hides_a_live_knob_or_shows_a_dead_one` recusa os dois sentidos.
    ParamGate {
        param: param::ASPECT,
        when: param::KIND,
        values: &[
            ShapeKind::Ellipse as i32,
            ShapeKind::Rectangle as i32,
            ShapeKind::Pie as i32,
            ShapeKind::Segment as i32,
            ShapeKind::ArrowRight as i32,
            ShapeKind::ArrowDouble as i32,
            ShapeKind::ArrowBent as i32,
            ShapeKind::Chevron as i32,
            ShapeKind::Diamond as i32,
            ShapeKind::Pill as i32,
            ShapeKind::Parallelogram as i32,
            ShapeKind::Trapezoid as i32,
            ShapeKind::TrapezoidFlip as i32,
            ShapeKind::HexagonFlat as i32,
            ShapeKind::Cylinder as i32,
            ShapeKind::Document as i32,
            ShapeKind::Delay as i32,
            ShapeKind::Display as i32,
            ShapeKind::PredefinedProcess as i32,
            ShapeKind::OffPage as i32,
            ShapeKind::Junction as i32,
            ShapeKind::SpeechRect as i32,
            ShapeKind::SpeechOval as i32,
            ShapeKind::Thought as i32,
            ShapeKind::Burst as i32,
            ShapeKind::Cloud as i32,
            ShapeKind::Bolt as i32,
            ShapeKind::Moon as i32,
            ShapeKind::Drop as i32,
            ShapeKind::Shield as i32,
            ShapeKind::Tag as i32,
            ShapeKind::Cross as i32,
            ShapeKind::Check as i32,
            ShapeKind::Banner as i32,
            ShapeKind::IsoCube as i32,
            ShapeKind::IsoCone as i32,
            ShapeKind::IsoPyramid as i32,
        ],
    },
    ParamGate {
        param: param::SIDES,
        when: param::KIND,
        values: &[
            ShapeKind::Polygon as i32,
            ShapeKind::Star as i32,
            ShapeKind::Gear as i32,
        ],
    },
    // ⚠️ **O `corner` deixou de ser da caixa e passou a ser do CATÁLOGO** (feedback do Enio,
    // 2026-08-19: *"senti falta de controle das quinas de uma rosca cortada e formas
    // similares"*). Quatro espécies o recebem DENTRO da receita (round-rect, polígono,
    // estrela); nas outras o shell aplica as Live Corners depois do `cook`, carimbando o raio
    // em todos os vértices — o motor recusa uma quina colinear, então um vértice de arco fica
    // intacto por construção.
    //
    // ⚠️ **A lista é DERIVADA, não escrita de cabeça:** a sonda
    // `which_kinds_the_corner_moves` empurra o número em cada espécie e imprime quem responde.
    // São **36** — as 7 de fora (Circle · Ellipse · Heart · Pill · Cylinder · Junction ·
    // Moon) não têm quina nenhuma, e mostrar-lhes o slider seria o botão morto que o gate
    // `no_kind_hides_a_live_knob_or_shows_a_dead_one` recusa nos dois sentidos.
    ParamGate {
        param: param::CORNER,
        when: param::KIND,
        values: &[
            ShapeKind::Square as i32,
            ShapeKind::Rectangle as i32,
            ShapeKind::Polygon as i32,
            ShapeKind::Star as i32,
            ShapeKind::Gear as i32,
            ShapeKind::Pie as i32,
            ShapeKind::Segment as i32,
            ShapeKind::ArrowRight as i32,
            ShapeKind::ArrowDouble as i32,
            ShapeKind::ArrowBent as i32,
            ShapeKind::Chevron as i32,
            ShapeKind::Diamond as i32,
            ShapeKind::Parallelogram as i32,
            ShapeKind::Trapezoid as i32,
            ShapeKind::TrapezoidFlip as i32,
            ShapeKind::HexagonFlat as i32,
            ShapeKind::Document as i32,
            ShapeKind::Delay as i32,
            ShapeKind::Display as i32,
            ShapeKind::PredefinedProcess as i32,
            ShapeKind::OffPage as i32,
            ShapeKind::SpeechRect as i32,
            ShapeKind::SpeechOval as i32,
            ShapeKind::Thought as i32,
            ShapeKind::Burst as i32,
            ShapeKind::Cloud as i32,
            ShapeKind::Bolt as i32,
            ShapeKind::Drop as i32,
            ShapeKind::Shield as i32,
            ShapeKind::Tag as i32,
            ShapeKind::Cross as i32,
            ShapeKind::Check as i32,
            ShapeKind::Banner as i32,
            ShapeKind::IsoCube as i32,
            ShapeKind::IsoCone as i32,
            ShapeKind::IsoPyramid as i32,
        ],
    },
    ParamGate {
        param: param::STAR_DEPTH,
        when: param::KIND,
        values: &[ShapeKind::Star as i32],
    },
    ParamGate {
        param: param::CLEFT,
        when: param::KIND,
        values: &[ShapeKind::Heart as i32],
    },
    ParamGate {
        param: param::TOOTH_DEPTH,
        when: param::KIND,
        values: &[ShapeKind::Gear as i32],
    },
    ParamGate {
        param: param::HOLE,
        when: param::KIND,
        values: &[ShapeKind::Gear as i32],
    },
    // ⚠️ **A FAMÍLIA DO CÍRCULO é UMA forma na biblioteca**, e estas quatro espécies são
    // os atalhos dela: `ellipse_sweep` recebe `sweep`/`start`/`inner`, e a `Segment` é a
    // corda (`ellipse_chord`), que tem os dois primeiros e **não tem miolo**. A tabela não
    // é opinião — o gate `no_kind_hides_a_live_knob_or_shows_a_dead_one` mexe em cada
    // número e recusa tanto esconder um vivo quanto pintar um morto.
    ParamGate {
        param: param::SWEEP,
        when: param::KIND,
        values: &[
            ShapeKind::Circle as i32,
            ShapeKind::Ellipse as i32,
            ShapeKind::Pie as i32,
            ShapeKind::Segment as i32,
        ],
    },
    ParamGate {
        param: param::START,
        when: param::KIND,
        values: &[
            ShapeKind::Circle as i32,
            ShapeKind::Ellipse as i32,
            ShapeKind::Pie as i32,
            ShapeKind::Segment as i32,
        ],
    },
    ParamGate {
        param: param::INNER,
        when: param::KIND,
        values: &[
            ShapeKind::Circle as i32,
            ShapeKind::Ellipse as i32,
            ShapeKind::Pie as i32,
        ],
    },
    // Os desvios por canto e a suavização são do ROUND-RECT, e as duas espécies que o
    // cozinham são a caixa quadrada e a retangular.
    ParamGate {
        param: param::CORNER_TR,
        when: param::KIND,
        values: &[ShapeKind::Square as i32, ShapeKind::Rectangle as i32],
    },
    ParamGate {
        param: param::CORNER_BR,
        when: param::KIND,
        values: &[ShapeKind::Square as i32, ShapeKind::Rectangle as i32],
    },
    ParamGate {
        param: param::CORNER_BL,
        when: param::KIND,
        values: &[ShapeKind::Square as i32, ShapeKind::Rectangle as i32],
    },
    ParamGate {
        param: param::SMOOTHING,
        when: param::KIND,
        values: &[ShapeKind::Square as i32, ShapeKind::Rectangle as i32],
    },
];
