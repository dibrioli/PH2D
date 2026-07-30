//! **§11 Physics Body + §12 Physics Joint — o `populate` deles.**
//!
//! Irmão do [`super::populate`] pelo cap de 600 LOC do arquivo, e o corte é o
//! mesmo que a linha de física já desenhou duas vezes (`inspector_model_physics.rs`
//! no W8, `inspector_physics_area.rs` na W-AreaFalloff): a churn de física passa a
//! morar num arquivo que esta linha possui, em vez de empurrar o orquestrador
//! compartilhado do Inspector contra o teto a cada wave.
//!
//! ⚠️ **Registrar aqui não é opcional:** um id que o painel PINTA e o `populate`
//! não registra nasce hit-registrado e **morto sob o mouse** — o
//! `architecture_panel_wiring_parity` é quem cobra, e as células/chips que
//! registram em LAÇO são o ponto cego dele (por isso os arrays são `const`).

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::TextInputState;

use crate::populate::register_button_ids;

/// §11 Physics Body (ADR-0131 D8): the two segmented groups and the two
/// buttons register as `Button`s (is_focusable → clicks route), the five
/// dimensions as `NumberInput`s.
///
/// **Every one gets a range**, and that is not decoration: `set_number_range`
/// is what makes drag-scrub proportional to the field's own span, and its
/// absence is the known gotcha (a `0..1` field dragged at the unbounded rate
/// crosses its whole domain in a few pixels). Bounce is physically `0..=1`;
/// friction above 1 is legal (it means "grips harder than it weighs"), so it
/// gets headroom rather than a false ceiling; density and the extents have no
/// natural maximum, so their caps are generous rather than meaningful.
/// §12 Physics Joint (W3). Registering these is what makes the widgets
/// FOCUSABLE — a control that is painted and hit-registered but never
/// registered here is dead under the mouse, and looks perfectly fine.
pub(super) fn populate_joint(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_JOINT_KIND);
    register_button_ids(store, &ids::INSP_JOINT_LIMITS);
    register_button_ids(store, &ids::INSP_JOINT_MOTOR);
    register_button_ids(store, &ids::INSP_JOINT_MOTOR_MODE);
    register_button_ids(store, &ids::INSP_JOINT_BREAK);
    // W-J8. Registered in `populate` like every sibling group: a chip the
    // painter draws and `populate` skips is painted, hit-registered and DEAD
    // under the mouse (the 36-cell lesson of W2c).
    register_button_ids(store, &ids::INSP_JOINT_ACTIVE);
    register_button_ids(store, &ids::INSP_JOINT_COLLIDE);
    register_button_ids(
        store,
        &[
            ids::INSP_JOINT_REMOVE,
            ids::INSP_JOINT_SWAP,
            ids::INSP_JOINT_ADD_WHEEL,
            ids::INSP_JOINT_PICK_A,
            ids::INSP_JOINT_PICK_B,
            ids::INSP_PHYS_JOIN,
            ids::INSP_PHYS_JOIN_DRAW,
            // §13 (W3): o eyedropper que arma o pick do corpo de montagem, e a
            // lixeira que desmonta. Sem o registro eles nascem pintados,
            // hit-registrados e MORTOS sob o mouse.
            ids::INSP_WHEEL_MOUNT_PICK,
            ids::INSP_WHEEL_UNMOUNT,
            // §13 (W1): o eyedropper que arma o pick da CORDA.
            ids::INSP_WHEEL_ROPE_PICK,
        ],
    );
    // The join-kind selector's four chips (Pin/Spring/Rope/Weld). Registered in a
    // loop, which `architecture_panel_wiring_parity` cannot see — the const array
    // covers them for `node_id_collisions`, and the seam test clicks each.
    register_button_ids(store, &ids::INSP_PHYS_JOIN_KIND);
    // Physical quantities again — degrees, meters, N·m, spring constants —
    // so none of these literals has a design token to come from.
    for (id, value, min, max, step) in [
        // A hinge limit is an angle; a full turn each way is the widest thing
        // that still means something.
        (ids::INSP_JOINT_LIMIT_MIN, -45.0, -360.0, 360.0, 1.0), // LITERAL-PX-OK: degrees
        (ids::INSP_JOINT_LIMIT_MAX, 45.0, -360.0, 360.0, 1.0),  // LITERAL-PX-OK: degrees
        // Motor speed, either direction. The range has to hold BOTH units the
        // row can be labelled with (degrees/second on a hinge, metres/second on
        // a rail or a winch), so it is the wider of the two — a range per kind
        // would be a second place for the unit to be decided.
        (ids::INSP_JOINT_MOTOR_SPEED, 114.0, -3600.0, 3600.0, 1.0), // LITERAL-PX-OK: deg/s or m/s
        // The servo's target place: degrees on a hinge, metres on a rail/winch.
        (ids::INSP_JOINT_MOTOR_TARGET, 0.0, -3600.0, 3600.0, 1.0), // LITERAL-PX-OK: degrees or m
        (ids::INSP_JOINT_MOTOR_FORCE, 10.0, 0.0, 10000.0, 0.1),    // LITERAL-PX-OK: N·m ceiling
        (ids::INSP_JOINT_REST_LENGTH, 1.0, 0.0, 1000.0, 0.01),     // LITERAL-PX-OK: meters
        (ids::INSP_JOINT_STIFFNESS, 30.0, 0.0, 100000.0, 1.0),     // LITERAL-PX-OK: spring constant
        (ids::INSP_JOINT_DAMPING, 0.5, 0.0, 1000.0, 0.1), // LITERAL-PX-OK: damping constant
        // Break thresholds (W-J7). The seed and the span come off the MEASURED
        // scale (`ph2d-physics/tests/measure_joint_break.rs`): a hanging weight
        // reads its own weight exactly, so 100 N is "it holds about ten kilos"
        // and 10 kN is a joint nothing in a scene will reach by accident. Never
        // negative — a negative threshold is crossed by every load, so the joint
        // would part on its first frame.
        (ids::INSP_JOINT_BREAK_FORCE, 100.0, 0.0, 10000.0, 1.0), // LITERAL-PX-OK: newtons
        (ids::INSP_JOINT_BREAK_TORQUE, 50.0, 0.0, 10000.0, 1.0), // LITERAL-PX-OK: newton-metres
        (ids::INSP_JOINT_MAX_LENGTH, 1.0, 0.001, 1000.0, 0.01),  // LITERAL-PX-OK: meters
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, min, max, step);
    }
}

/// §13 Pulley Wheel (W-Pulley W1) — o mesmo dever da irmã acima.
///
/// ⚠️ **Alcance E taxa de arrasto, nos dois números.** Os dois têm PISO e não
/// têm TETO — um raio não tem máximo natural (`MIN_RADIUS` é 0, e raio zero é a
/// roldana-ponto que a rota reproduz exata) e a ordem é `u16`. A combinação é a
/// receita documentada em `set_number_drag_rate` para exatamente esse caso: o
/// alcance dá `step` e piso ao stepper, a taxa dá ao arrasto uma escala
/// calibrada em vez de uma proporção sobre um intervalo que não termina. Os
/// máximos abaixo são conveniência do stepper, **não recurso medido** — e é por
/// isso que eles não podem ser o que impede uma corda de ter cem roldanas.
pub(super) fn populate_wheel(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_WHEEL_WRAP);
    register_button_ids(store, &ids::INSP_WHEEL_BREAK);
    // W-Weston. Registrado SEM condição, ao contrário de a row ser pintada: o
    // `populate` roda uma vez no boot e não sabe que roldana está selecionada, e um
    // id não-registrado é o chip morto sob o mouse que o `wiring_parity` pega.
    register_button_ids(store, &ids::INSP_WHEEL_DIFF);
    for (id, value, min, max, step, rate) in [
        // Metros. O seed é o `PulleyWheel::DEFAULT_RADIUS`.
        (ids::INSP_WHEEL_RADIUS, 0.25, 0.0, 100.0, 0.01, 0.01), // LITERAL-PX-OK: meters
        (ids::INSP_WHEEL_RADIUS_OUT, 0.0, 0.0, 100.0, 0.01, 0.01), // LITERAL-PX-OK: meters
        // 1-based, como a row mostra.
        (ids::INSP_WHEEL_ORDER, 1.0, 1.0, 99.0, 1.0, 0.1), // LITERAL-PX-OK: ordinal
        // Graus por segundo, COM SINAL (negativo paga corda). A faixa é a mesma
        // do motor do Pin (`INSP_JOINT_MOTOR_SPEED`): dez voltas por segundo em
        // cada sentido, que é o que aquele knob já oferece.
        (ids::INSP_WHEEL_MOTOR, 0.0, -3600.0, 3600.0, 1.0, 1.0), // LITERAL-PX-OK: deg/s
        // Newtons, com o mesmo default do joint: um ponto de partida que a
        // primeira coisa pendurada já tem chance de cruzar, que é como o artista
        // descobre que o controle funciona.
        (ids::INSP_WHEEL_BREAK_FORCE, 500.0, 0.0, 1.0e9, 1.0, 1.0), // LITERAL-PX-OK: newtons
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, min, max, step);
        store.set_number_drag_rate(id, rate);
    }
}

pub(super) fn populate_physics(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_PHYS_KIND);
    register_button_ids(store, &ids::INSP_PHYS_SHAPE);
    register_button_ids(store, &ids::INSP_PHYS_LAYER);
    register_button_ids(store, &ids::INSP_PHYS_SENSOR);
    register_button_ids(store, &ids::INSP_PHYS_CCD);
    register_button_ids(store, &ids::INSP_PHYS_LOCKROT);
    register_button_ids(store, &ids::INSP_PHYS_LOCKX);
    register_button_ids(store, &ids::INSP_PHYS_LOCKY);
    register_button_ids(store, &ids::INSP_PHYS_MASSMODE);
    register_button_ids(store, &ids::INSP_PHYS_REST_COMBINE);
    register_button_ids(store, &ids::INSP_PHYS_FRIC_COMBINE);
    register_button_ids(store, &ids::INSP_PHYS_DAMPMODE);
    register_button_ids(store, &ids::INSP_PHYS_ONEWAY);
    register_button_ids(store, &ids::INSP_PHYS_FORCE_AXES);
    register_button_ids(store, &ids::INSP_PHYS_BAKE_CH);
    register_button_ids(
        store,
        &[
            ids::INSP_PHYS_ADD,
            ids::INSP_PHYS_BAKE,
            ids::INSP_PHYS_REMOVE,
        ],
    );
    // Every literal below is a PHYSICAL quantity — meters, kg/m², or a
    // dimensionless coefficient — not a design measurement, so none of them
    // has a token to come from.
    for (id, value, min, max, step) in [
        (ids::INSP_PHYS_RADIUS, 0.5, 0.001, 1000.0, 0.01), // LITERAL-PX-OK: meters
        (ids::INSP_PHYS_HALF_X, 0.5, 0.001, 1000.0, 0.01), // LITERAL-PX-OK: meters
        (ids::INSP_PHYS_HALF_Y, 0.5, 0.001, 1000.0, 0.01), // LITERAL-PX-OK: meters
        // The capsule's STRAIGHT segment: min 0.0, not 0.001 — a zero-segment
        // capsule is exactly a ball, which is the honest bottom of the range
        // (and what Ball -> Capsule converts to).
        (ids::INSP_PHYS_CAP_HALF_H, 0.25, 0.0, 1000.0, 0.01), // LITERAL-PX-OK: meters
        // Collider offset (signed — the offset is a POSITION, can go either way).
        // Bounds the drag only; the component/BodyDesc take any f32.
        (ids::INSP_PHYS_OFFSET_X, 0.0, -1000.0, 1000.0, 0.01), // LITERAL-PX-OK: meters
        (ids::INSP_PHYS_OFFSET_Y, 0.0, -1000.0, 1000.0, 0.01), // LITERAL-PX-OK: meters
        // Initial velocity (W9): signed. The range bounds the DRAG only —
        // the component/BodyDesc/rapier take any f32 — so it spans a sane
        // authoring range around zero, like gravity scale does.
        (ids::INSP_PHYS_LINVEL_X, 0.0, -100.0, 100.0, 0.1), // LITERAL-PX-OK: m/s
        (ids::INSP_PHYS_LINVEL_Y, 0.0, -100.0, 100.0, 0.1), // LITERAL-PX-OK: m/s
        (ids::INSP_PHYS_ANGVEL, 0.0, -3600.0, 3600.0, 1.0), // LITERAL-PX-OK: deg/s
        (ids::INSP_PHYS_DENSITY, 1.0, 0.0, 1000.0, 0.01),   // LITERAL-PX-OK: kg/m^2
        // Explicit mass override (W-Mass), Manual mode. min 0.001 (mass must be
        // positive — a zero-mass dynamic body is degenerate); the range bounds the
        // DRAG only, the component/BodyDesc take any positive f32.
        (ids::INSP_PHYS_MASS, 1.0, 0.001, 100000.0, 0.1), // LITERAL-PX-OK: kg
        (ids::INSP_PHYS_RESTITUTION, 0.0, 0.0, 1.0, 0.01), // LITERAL-PX-OK: bounciness is 0..=1 by physics
        (ids::INSP_PHYS_FRICTION, 0.5, 0.0, 10.0, 0.01), // LITERAL-PX-OK: Coulomb coefficient, >1 is legal
        // Per-body gravity multiplier (W8). The range bounds the DRAG only — the
        // component/`BodyDesc`/rapier take any f32 (a loaded project may carry
        // more); -10..10 covers the authoring span (balloon → 10× heavy), the
        // same soft-UI bound `RADIUS` uses.
        (ids::INSP_PHYS_GRAVITY_SCALE, 1.0, -10.0, 10.0, 0.1), // LITERAL-PX-OK: dimensionless gravity multiplier
        // Dominance (W-Dominance): a signed integer collision priority, step 1. The
        // range bounds the DRAG only — the component/BodyDesc take any i8, and the
        // event arm rounds+clamps; -10..10 covers the authoring span (rapier's i8
        // allows ±127, but a legible priority is a small number).
        (ids::INSP_PHYS_DOMINANCE, 0.0, -10.0, 10.0, 1.0), // LITERAL-PX-OK: integer collision priority
        // Per-body damping (drag), Dynamic-only (W-Damping). Default 0.0 (the world
        // default drag). The drag range bounds only the scrub; the component/BodyDesc
        // take any f32 >= 0. 0..10 mirrors the world drag's own MAX (`MAX_DAMPING`) —
        // a coefficient past 10 is essentially "instant stop".
        (ids::INSP_PHYS_LINEAR_DAMPING, 0.0, 0.0, 10.0, 0.05), // LITERAL-PX-OK: linear drag coefficient
        (ids::INSP_PHYS_ANGULAR_DAMPING, 0.0, 0.0, 10.0, 0.05), // LITERAL-PX-OK: angular drag coefficient
        // Force zone (W-Area): newtons, signed (a wind blows either way). The range
        // is generous because it is weighed against a body's MASS — a 20 kg crate
        // needs ~200 N just to hold it against gravity.
        (ids::INSP_PHYS_FORCE_X, 0.0, -1000.0, 1000.0, 0.5), // LITERAL-PX-OK: newtons
        (ids::INSP_PHYS_FORCE_Y, 0.0, -1000.0, 1000.0, 0.5), // LITERAL-PX-OK: newtons
        // Torque zone (W-AreaTorque): N·m, signed (the sign is the spin direction). The
        // linear/rotational pair with Force, so it wears the same generous signed range —
        // weighed against the body's MOMENT OF INERTIA, which is smaller than its mass, so
        // a modest number already spins a compact body briskly.
        (ids::INSP_PHYS_AREA_TORQUE, 0.0, -1000.0, 1000.0, 0.5), // LITERAL-PX-OK: N*m
        // Falloff (W-AreaFalloff): uma FRAÇÃO, e por isso a faixa é fechada em `0..=1` —
        // não é um comprimento a calibrar contra o tamanho da zona (a régua é a silhueta
        // dela), e nada acima de 1 tem sentido: já se perde tudo na borda. Passo de 0.05
        // porque o efeito é contínuo e o artista o julga olhando, não digitando.
        (ids::INSP_PHYS_AREA_FALLOFF, 0.0, 0.0, 1.0, 0.05), // LITERAL-PX-OK: fracao 0..1
        // The medium's resistance. Same range as the other drag knobs in this app —
        // it is the same law, so it must be the same numbers.
        (ids::INSP_PHYS_AREA_DRAG, 0.0, 0.0, 10.0, 0.05), // LITERAL-PX-OK: drag coefficient
        // Densidade do fluido: MESMA faixa do `Density` do collider, porque a
        // comparação entre os dois é justamente a leitura (menor boia, maior afunda).
        (ids::INSP_PHYS_AREA_DENSITY, 0.0, 0.0, 1000.0, 0.05), // LITERAL-PX-OK: kg/m^2
        // Resistência de forma: mesma faixa dos outros arrastos, porque a leitura do
        // artista é comparar os dois knobs de resistência lado a lado.
        (ids::INSP_PHYS_AREA_FORM_DRAG, 0.0, 0.0, 10.0, 0.05), // LITERAL-PX-OK: coef. de forma
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, min, max, step);
    }
}
