//! **QUANDO cada linha do cartão APARECE** — as duas tabelas de portão do `source.shape`.
//!
//! Irmão do [`super::hints`] pelo tecto de LOC (HR-18) e por PERGUNTA: ali responde-se *como é que
//! esta linha é desenhada* (rótulo, faixa, widget), aqui *se ela chega a existir* — o `ParamGate`
//! por VALOR de outro param (a forma que a lê) e o `ParamGateAbove` por LIMIAR.
//!
//! ⚠️ **Duas tabelas e não uma**: um portão por valor e um por limiar são perguntas diferentes, e
//! o registo tem um canal para cada. Fundi-las obrigaria a inventar um valor-sentinela.

use super::{ShapeKind, param};
use ph2d_node_registry::{ParamGate, ParamGateAbove};

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
    // São **38** — as 5 de fora (Heart · Pill · Cylinder · Junction · Moon) não têm quina
    // nenhuma, e mostrar-lhes o slider seria o botão morto que o gate
    // `no_kind_hides_a_live_knob_or_shows_a_dead_one` recusa nos dois sentidos.
    //
    // ⚠️ **O `Circle` e a `Ellipse` estão aqui, e foram a CORREÇÃO** (Enio, 2026-08-19: *"não
    // há opções de corner na UI"*). Um círculo INTEIRO não tem quina; o MESMO círculo com
    // `sweep < 360` é uma rosca cortada e tem QUATRO. A primeira versão desta lista foi
    // derivada de um único ponto do espaço de params — o neutro — e escondeu o slider
    // exactamente na forma que o pedia. Um param é do KIND se ele move a forma em ALGUMA
    // configuração alcançável daquele kind, e é isso que a `CONTEXTS` do gate encena.
    ParamGate {
        param: param::CORNER,
        when: param::KIND,
        values: &[
            ShapeKind::Circle as i32,
            ShapeKind::Ellipse as i32,
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
    // O TAMANHO do colisor mostra só os números que a forma escolhida lê (doc 109 §5): largura e
    // altura na caixa, raio no círculo. O `Collide` desligado esconde os três (`PARAM_GATES_ABOVE`).
    ParamGate {
        param: param::COLLIDER_WIDTH,
        when: param::COLLIDER_SHAPE,
        values: &[super::collider::SHAPE_BOX],
    },
    ParamGate {
        param: param::COLLIDER_HEIGHT,
        when: param::COLLIDER_SHAPE,
        values: &[super::collider::SHAPE_BOX],
    },
    ParamGate {
        param: param::COLLIDER_RADIUS,
        when: param::COLLIDER_SHAPE,
        values: &[super::collider::SHAPE_CIRCLE],
    },
];

/// **A FAMÍLIA DO TRAÇO aparece com o traço** — a visibilidade que o `ParamGate` por-espécie
/// não sabe fazer, porque a condição não é uma escolha de enum e sim uma GRANDEZA.
///
/// ⚠️ **O Trim está nesta lista, e é o motivo de a lista existir.** Um contorno aparado é
/// ABERTO, e um contorno aberto **não tem interior** — a silhueta não é preenchida (é a mesma
/// lei que impede a tampa do cilindro de recortar a forma). Sem traço, então, mexer no Trim não
/// dá um controle morto: dá a **forma a desaparecer**, com o nó certo selecionado e nada na
/// tela. *Um controle que apaga a arte é pior que um que não faz nada.*
///
/// ⚠️ E a cor entra pelo `stroke_r`, que é a ÂNCORA do swatch (os outros três canais são
/// dobrados nele e nem chegam a ter linha própria) — gatear um canal dobrado não esconderia
/// nada.
pub(crate) static PARAM_GATES_ABOVE: &[ParamGateAbove] = &[
    // ⚠️ O swatch do preenchimento só aparece com o modo ligado — e ancora no `fill_r` pela
    // mesma razão que o do traço: os outros três canais estão dobrados nele.
    ParamGateAbove {
        param: param::FILL_R,
        when: param::FILL,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::STROKE_R,
        when: param::STROKE_WIDTH,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::DASH,
        when: param::STROKE_WIDTH,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::DASH_GAP,
        when: param::STROKE_WIDTH,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::TRIM_START,
        when: param::STROKE_WIDTH,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::TRIM_END,
        when: param::STROKE_WIDTH,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::TRIM_OFFSET,
        when: param::STROKE_WIDTH,
        above: 0.0,
    },
    // A forma e o tamanho do colisor só existem com ele ligado — desligado, nenhuma coluna é escrita.
    ParamGateAbove {
        param: param::COLLIDER_SHAPE,
        when: param::COLLIDE,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::COLLIDER_WIDTH,
        when: param::COLLIDE,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::COLLIDER_HEIGHT,
        when: param::COLLIDE,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::COLLIDER_RADIUS,
        when: param::COLLIDE,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::SHOW_COLLIDER,
        when: param::COLLIDE,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::LOCK_ROTATION,
        when: param::COLLIDE,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::FRICTION,
        when: param::COLLIDE,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::BOUNCE,
        when: param::COLLIDE,
        above: 0.0,
    },
    ParamGateAbove {
        param: param::ROLLING,
        when: param::COLLIDE,
        above: 0.0,
    },
];
