//! **A face do colisor** — os hints, as unidades e os gates de visibilidade que o painel lê.
//!
//! Separado do nó por RESPONSABILIDADE (*o que o colisor FAZ* contra *o que o artista VÊ*), que
//! é também o que mantém o `lib.rs` sob o teto de LOC. Tudo aqui é `pub(crate)` e tem exatamente
//! um consumidor, [`crate::register`].

use super::*;
use ph2d_node_registry::ParamGroup;

/// Param UI hints. `shape` is a NAMED enum — a float slider would make the artist decode
/// "2" into Bowl, which is exactly the decode the segmented selector exists to abolish.
pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "shape",
        label: "Shape",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Plane", "Disc", "Bowl"],
        },
    },
    ParamUiHint {
        // ⚠️ "Offset", not "Height": the number is the distance from the origin ALONG the
        // plane's normal, so at 90° it places a wall in x. A label has to promise what the
        // model delivers, and "Height" stopped being able to
        // ([[feedback_a_label_must_promise_what_the_model_delivers]]).
        param: "height",
        label: "Offset",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "angle",
        label: "Angle",
        // The full turn, because every quarter of it is a different idiom: 0 floor,
        // 90 wall, 180 ceiling, 270 the other wall.
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_x",
        label: "Center X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_y",
        label: "Center Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "radius_from",
        label: "Radius From",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            // Reads as a sentence, and "Point" names the default HONESTLY: what the
            // collider did before it could know how big anything was.
            labels: &["Point", "Fixed", "Sprite Size"],
        },
    },
    ParamUiHint {
        param: "particle_radius",
        label: "Particle Radius",
        min: 0.0,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "size_scale",
        label: "Size Scale",
        // 1 = the circle inscribed in the sprite; 1.414… = the one around a square
        // one. Above ~2 the collider is visibly bigger than the art it catches.
        min: 0.0,
        max: 3.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "restitution",
        label: "Bounce",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "friction",
        label: "Friction",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // **A ALEATORIEDADE DA RESTITUIÇÃO** (doc 89 folha 13). ⚠️ O teto é `1` porque a lei só
    // TIRA: em `1` a partícula mais azarada não devolve nada, e não há nada abaixo de «não
    // devolve nada». Um teto maior seria um número que o produto não consegue honrar.
    ParamUiHint {
        param: "restitution_randomness",
        label: "Restitution Randomness",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
];

/// A semente pertence à grandeza que a lê: com `restitution_randomness = 0` o cook nunca a
/// abre, e um knob que o cook não abre é um controlo morto (a mesma lei do `seed` do
/// `motion.duplicator`, aqui sobre um LIMIAR em vez de um enum).
pub(crate) static PARAM_GATES_ABOVE: &[ph2d_node_registry::ParamGateAbove] =
    &[ph2d_node_registry::ParamGateAbove {
        param: "seed",
        when: "restitution_randomness",
        above: 0.0,
    }];

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
pub(crate) static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "height",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "center_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "center_y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "radius",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "particle_radius",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "angle",
        unit: ParamUnit::Angle,
    },
];

/// **Only the controls this shape and this radius mode actually READ.**
///
/// Before this, every param was painted always — so a Disc showed a `Height` the kernel never
/// looks at, and a Plane showed a `Center X`/`Center Y`/`Radius` it never looks at either: four
/// dead knobs, which is the thing this codebase refuses (`sim.collide` was one of the nodes the
/// doc-89 folha flagged for it). The gate is side-metadata — a param with no entry here is
/// always shown, which is what every one of them meant yesterday.
///
/// ⚠️ `radius` appears in the **shape** group and `particle_radius` in the **radius** one on
/// purpose: they are two different lengths (*how big is the WORLD* × *how big is the THING*),
/// and the labels say so ("Radius" × "Particle Radius").
pub(crate) static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    ph2d_node_registry::ParamGate {
        param: "height",
        when: "shape",
        values: &[SHAPE_PLANE],
    },
    // ⚠️ The tilt is gated by GEOMETRY, not by taste: a disc and a bowl are rotationally
    // symmetric, so an angle on them is provably a knob that changes nothing.
    ph2d_node_registry::ParamGate {
        param: "angle",
        when: "shape",
        values: &[SHAPE_PLANE],
    },
    ph2d_node_registry::ParamGate {
        param: "center_x",
        when: "shape",
        values: &[SHAPE_DISC, SHAPE_BOWL],
    },
    ph2d_node_registry::ParamGate {
        param: "center_y",
        when: "shape",
        values: &[SHAPE_DISC, SHAPE_BOWL],
    },
    ph2d_node_registry::ParamGate {
        param: "radius",
        when: "shape",
        values: &[SHAPE_DISC, SHAPE_BOWL],
    },
    ph2d_node_registry::ParamGate {
        param: "particle_radius",
        when: "radius_from",
        values: &[RADIUS_FIXED],
    },
    ph2d_node_registry::ParamGate {
        param: "size_scale",
        when: "radius_from",
        values: &[RADIUS_SIZE],
    },
];

/// **AS SEÇÕES DESTE NÓ** (doc 88 B3, doc 89 folha 13 — a célula *"nenhuma `ParamSection`
/// num nó de onze params"*).
///
/// ⚠️ **O corte é por PERGUNTA, não por tipo de dado.** Uma parede de onze sliders obriga o
/// artista a ler todos para achar um; três títulos transformam-na em três perguntas que ele
/// já tem na cabeça: *onde está o obstáculo* · *de que tamanho é a coisa que bate nele* ·
/// *o que acontece na batida*.
///
/// ⚠️ **A célula propunha DUAS** (Forma × Resposta) e a leitura dos params pediu três: o trio
/// `radius_from`/`particle_radius`/`size_scale` não responde nem a *onde* nem a *o que
/// acontece* — ele responde *quem bate*, e enfiá-lo em qualquer uma das outras duas faria a
/// secção mentir sobre o que contém.
///
/// ⚠️ **Nenhuma nasce fechada.** Uma secção dobrada por omissão esconde um param que já
/// existia, e o `folded()` é para o que o artista raramente toca — aqui ele toca em todos.
///
/// A aleatoriedade e a semente vivem em **Response**: elas dizem *o que acontece na batida*,
/// tal como a restituição que modulam.
pub(super) static PARAM_GROUPS: &[ParamGroup] = &[
    // ONDE está o obstáculo.
    ParamGroup::new("shape", "Shape"),
    ParamGroup::new("center_x", "Shape"),
    ParamGroup::new("center_y", "Shape"),
    ParamGroup::new("radius", "Shape"),
    ParamGroup::new("height", "Shape"),
    ParamGroup::new("angle", "Shape"),
    // QUEM bate nele — o raio da partícula, e de onde ele sai.
    ParamGroup::new("radius_from", "Particle Size"),
    ParamGroup::new("particle_radius", "Particle Size"),
    ParamGroup::new("size_scale", "Particle Size"),
    // O QUE acontece na batida.
    ParamGroup::new("restitution", "Response"),
    ParamGroup::new("friction", "Response"),
    ParamGroup::new("restitution_randomness", "Response"),
    ParamGroup::new("seed", "Response"),
];
