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

use super::{KIND_LABELS, param};
use ph2d_node_registry::{ParamGroup, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

/// The param rows: a real dropdown for the shape family (the segmented `Enum`
/// widget the Vector panel uses for Cap/Join), then the geometry sliders. Every
/// row past `size` is gated by [`super::param_gates::PARAM_GATES`], so the panel shows ONLY the
/// controls the current `kind` uses.
pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    // ⚠️⚠️ **A ordem ABRE pela FORMA** (ciclo 8, W2 — doc 113 §4). Até 2026-09-16 a lista
    // começava em `Own Fill`, e o comentário ao lado dela dizia a lei que ela violava: *«de que
    // cor é e para que lado aponta são o que se pergunta de uma forma DEPOIS de escolher qual ela
    // é»*. O cartão de uma FONTE abre com o que ela produz.
    ParamUiHint {
        param: param::KIND,
        label: "node.source.shape.param.kind",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Enum {
            labels: KIND_LABELS,
        },
    },
    ParamUiHint {
        param: param::SIZE,
        label: "node.source.shape.param.size",
        min: 0.05,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::ROTATION,
        label: "node.source.shape.param.rotation",
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    // **A COR e a ROTAÇÃO próprias** (doc 89 folha 14, as duas últimas células) — no topo, com
    // o traço, porque *de que cor é* e *para que lado aponta* são o que se pergunta de uma
    // forma depois de escolher qual ela é.
    ParamUiHint {
        param: param::FILL,
        label: "node.source.shape.param.fill",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // O MESMO swatch do traço, pela mesma lei (nunca quatro sliders lineares crus).
    ParamUiHint {
        param: param::FILL_R,
        label: "node.source.shape.param.fill_r",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: [param::FILL_R, param::FILL_G, param::FILL_B, param::FILL_A],
        },
    },
    // **O TRAÇO** (doc 89 folha 14, P0) — o controle que separa *forma* de
    // *silhueta*. `0` = sem traço ⇒ a forma que sempre shipou.
    ParamUiHint {
        param: param::STROKE_WIDTH,
        label: "node.source.shape.param.stroke_width",
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
        label: "node.source.shape.param.stroke_r",
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
        param: param::ASPECT,
        label: "node.source.shape.param.aspect",
        min: 0.1,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::SIDES,
        label: "node.source.shape.param.sides",
        min: 3.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: param::CORNER,
        label: "node.source.shape.param.corner",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::STAR_DEPTH,
        label: "node.source.shape.param.star_depth",
        min: 0.05,
        max: 0.95,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::CLEFT,
        label: "node.source.shape.param.cleft",
        min: 0.02,
        max: 0.45,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::TOOTH_DEPTH,
        label: "node.source.shape.param.tooth_depth",
        min: 0.05,
        max: 0.6,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::HOLE,
        label: "node.source.shape.param.hole",
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
        label: "node.source.shape.param.sweep",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::START,
        label: "node.source.shape.param.start",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    // O miolo para em `0,95`: a `1` a rosquinha tem espessura zero e desaparece — o
    // mesmo teto do `Hole` da engrenagem, pela mesma razão.
    ParamUiHint {
        param: param::INNER,
        label: "node.source.shape.param.inner",
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
        label: "node.source.shape.param.corner_tr",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::CORNER_BR,
        label: "node.source.shape.param.corner_br",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::CORNER_BL,
        label: "node.source.shape.param.corner_bl",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::SMOOTHING,
        label: "node.source.shape.param.smoothing",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **O TRIM** (doc 89 folha 14). Faixa `0..1` em fração do comprimento, como o
    // `TrimSpec` a fala — nunca em unidades de mundo: o mesmo `End = 0.5` revela metade
    // de um círculo e metade de uma estrela, que é a promessa do *Trim Paths*.
    ParamUiHint {
        param: param::TRIM_START,
        label: "node.source.shape.param.trim_start",
        min: 0.0,
        max: 1.0,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::TRIM_END,
        label: "node.source.shape.param.trim_end",
        min: 0.0,
        max: 1.0,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::TRIM_OFFSET,
        label: "node.source.shape.param.trim_offset",
        min: 0.0,
        max: 1.0,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **O TRACEJADO em múltiplos da LARGURA**, então o teto de 20 é *vinte larguras
    // de traço*, não vinte unidades de mundo — a faixa que cobre do pontilhado denso à
    // linha de corte larga sem que engrossar o traço mude o ritmo.
    ParamUiHint {
        param: param::DASH,
        label: "node.source.shape.param.dash",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::DASH_GAP,
        label: "node.source.shape.param.dash_gap",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    // ⭐⭐ **O COLISOR** (doc 109 — ordem do dono: *«colidem sozinhas»*). Um toggle, e não uma
    // sentinela no tamanho: um `Collider Width` de zero é uma caixa sem largura, não a ausência dela.
    ParamUiHint {
        param: param::COLLIDE,
        label: "node.source.shape.param.collide",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // ⭐⭐ **A FORMA do colisor** (doc 109 §5 — report do dono: *«no mínimo colliders circulares e
    // retangulares que tentam se adaptar às dimensões da shape»*).
    ParamUiHint {
        param: param::COLLIDER_SHAPE,
        label: "node.source.shape.param.collider_shape",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: super::collider::SHAPE_LABELS,
        },
    },
    // ⚠️ **`0..2` é a faixa CONFORTÁVEL do arrasto, não um recurso**: `1` é a caixa envolvente da
    // forma, o dobro já afasta as peças à vista, e nada no solver deixa de honrar um número maior —
    // a alça do canvas escreve além dele.
    ParamUiHint {
        param: param::COLLIDER_WIDTH,
        label: "node.source.shape.param.collider_width",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::COLLIDER_HEIGHT,
        label: "node.source.shape.param.collider_height",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::COLLIDER_RADIUS,
        label: "node.source.shape.param.collider_radius",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⭐⭐ **VER o colisor** (report do dono, 2026-09-13: *«coloque um botão no nó shape: Ver
    // collider»*) — o contorno de cada peça desta forma, por cima da arte.
    ParamUiHint {
        param: param::SHOW_COLLIDER,
        label: "node.source.shape.param.show_collider",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // ⭐⭐ **TRAVAR a rotação** (doc 109 §6 — *«precisa destravar a rot. e colocar outro botão para
    // travar rotação»*): desligado elas tombam, ligado só deslizam.
    ParamUiHint {
        param: param::LOCK_ROTATION,
        label: "node.source.shape.param.lock_rotation",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // ⭐⭐⭐ **O MATERIAL** (doc 109 §7 — *«precisamos de parâmetros do material»*). O atrito é o
    // que faz um círculo RODAR em vez de derrapar, e não apenas o que o trava.
    ParamUiHint {
        param: param::FRICTION,
        label: "node.source.shape.param.friction",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⭐⭐⭐ **A faixa vai a `2`, o DOBRO do de todo motor** — ordem do dono (2026-09-13: *«quero
    // mais capacidade de Bounciness — de zero até o dobro do máximo atual»*), e o número é LIDO do
    // tecto da coluna (`BOUNCE_MAX`), nunca escrito aqui: um literal neste hint seria o segundo
    // sítio onde a faixa vive, e a caixa de texto de um param sem `ParamHardMax` é capada
    // exactamente por ele. A tabela MEDIDA que abriu a faixa está no doc de `BOUNCE_MAX`.
    ParamUiHint {
        param: param::BOUNCE,
        label: "node.source.shape.param.bounce",
        min: 0.0,
        max: ph2d_nodegraph::attr::BOUNCE_MAX,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⭐⭐⭐ **O que faz uma bola PARAR sozinha** (doc 109 §7.10). O tecto é `1,5` e sai da MEDIÇÃO
    // (o doc de `ROLLING_MAX` tem a tabela): é onde a coluna satura, porque a partir dali quem
    // trava a peça é o atrito de Coulomb e não esta lei.
    ParamUiHint {
        param: param::ROLLING,
        label: "node.source.shape.param.rolling",
        min: 0.0,
        max: ph2d_nodegraph::attr::ROLLING_MAX,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
///
/// ⚠️ Irmã das hints pelo tecto de LOC do `lib.rs` e por ASSUNTO: as duas dizem como um número
/// se LÊ, e nenhuma delas é o `NodeOp`.
pub(crate) static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: param::SIZE,
    unit: ParamUnit::Length,
}];

/// **A secção «Collision»** — os do colisor juntos, depois de tudo o que desenha a forma (os params
/// sem grupo vêm antes de toda secção, que é onde os essenciais moram).
pub(crate) static PARAM_GROUPS: &[ParamGroup] = &[
    // ⭐ **A APARÊNCIA junta, depois da forma** (ciclo 8, W2): o que a forma É vem primeiro (sem
    // secção, que é onde os essenciais moram), e *de que cor* é outra pergunta.
    ParamGroup::new(param::FILL, "node.group.look"),
    ParamGroup::new(param::FILL_R, "node.group.look"),
    ParamGroup::new(param::STROKE_WIDTH, "node.group.look"),
    ParamGroup::new(param::STROKE_R, "node.group.look"),
    ParamGroup::new(param::COLLIDE, "node.group.collision"),
    ParamGroup::new(param::COLLIDER_SHAPE, "node.group.collision"),
    ParamGroup::new(param::COLLIDER_WIDTH, "node.group.collision"),
    ParamGroup::new(param::COLLIDER_HEIGHT, "node.group.collision"),
    ParamGroup::new(param::COLLIDER_RADIUS, "node.group.collision"),
    ParamGroup::new(param::SHOW_COLLIDER, "node.group.collision"),
    ParamGroup::new(param::LOCK_ROTATION, "node.group.collision"),
    ParamGroup::new(param::FRICTION, "node.group.collision"),
    ParamGroup::new(param::BOUNCE, "node.group.collision"),
    ParamGroup::new(param::ROLLING, "node.group.collision"),
];
