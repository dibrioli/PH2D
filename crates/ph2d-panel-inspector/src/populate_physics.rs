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
    register_button_ids(store, &ids::INSP_JOINT_ANCHOR_B);
    // W-JointCustom: os três eixos e o eixo do motor. **Registrados aqui como
    // todo grupo irmão** — um chip que o pintor desenha e o `populate` pula é
    // pintado, hit-registrado e MORTO sob o mouse (a lição das 36 células do
    // W2c, que esta seção já cita duas linhas acima).
    for group in &ids::INSP_JOINT_AXIS_MODE {
        register_button_ids(store, group);
    }
    register_button_ids(store, &ids::INSP_JOINT_MOTOR_AXIS);
    // W-SoftWeld — mesma razão da linha acima.
    register_button_ids(store, &ids::INSP_JOINT_SOFT);
    register_button_ids(
        store,
        &[
            ids::INSP_JOINT_REMOVE,
            ids::INSP_JOINT_SWAP,
            ids::INSP_JOINT_ADD_WHEEL,
            ids::INSP_JOINT_COPY,
            ids::INSP_JOINT_PASTE,
            ids::INSP_JOINT_PICK_A,
            ids::INSP_JOINT_PICK_B,
            ids::INSP_PHYS_JOIN,
            ids::INSP_PHYS_JOIN_DRAW,
            ids::INSP_PHYS_RIG,
            ids::INSP_PHYS_ADD_SHAPE,
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
        // Os batentes por eixo de um Custom. A faixa tem de segurar as DUAS
        // unidades que a row pode mostrar (metros nos lineares, graus na
        // rotação), pelo mesmo motivo que a do motor acima: uma faixa por eixo
        // seria um segundo lugar decidindo a unidade.
        (ids::INSP_JOINT_AXIS_MIN[0], -1.0, -3600.0, 3600.0, 0.01), // LITERAL-PX-OK: m
        (ids::INSP_JOINT_AXIS_MAX[0], 1.0, -3600.0, 3600.0, 0.01),  // LITERAL-PX-OK: m
        (ids::INSP_JOINT_AXIS_MIN[1], -1.0, -3600.0, 3600.0, 0.01), // LITERAL-PX-OK: m
        (ids::INSP_JOINT_AXIS_MAX[1], 1.0, -3600.0, 3600.0, 0.01),  // LITERAL-PX-OK: m
        (ids::INSP_JOINT_AXIS_MIN[2], -45.0, -3600.0, 3600.0, 1.0), // LITERAL-PX-OK: degrees
        (ids::INSP_JOINT_AXIS_MAX[2], 45.0, -3600.0, 3600.0, 1.0),  // LITERAL-PX-OK: degrees
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

    // As rows dos **sinais** (W-Signal · W-SignalLeave). Sem este registro elas
    // nasceriam pintadas, hit-registradas e MORTAS sob o mouse — a falha que este
    // arquivo existe para impedir, e que o `architecture_panel_wiring_parity`
    // pega.
    for id in [ids::INSP_PHYS_SIGNAL, ids::INSP_PHYS_SIGNAL_LEAVE] {
        store.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
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

/// §14 Platform Player (W5) — os três botões e os oito números.
///
/// ⚠️ **Registrado sem condição**, como o Weston ao lado e pelo mesmo motivo: o
/// `populate` roda uma vez no boot e não sabe que entidade está selecionada, e
/// um id não-registrado é o widget PINTADO e morto sob o mouse que o
/// `architecture_panel_wiring_parity` existe para pegar.
///
/// ⚠️ **Os defaults são o `PlayerConfig::STARTING_POINT`** — o mesmo ponto de
/// partida que o componente usa. Um segundo conjunto aqui seria a segunda
/// resposta a *"com que números um player nasce?"*, e ela divergiria no dia em
/// que a varredura da wave movesse um deles.
pub(super) fn populate_player(store: &mut WidgetStore) {
    // ⚠️ **O grupo do chip, registrado como os quinze da §11** — sem isto os dois
    // botões nascem pintados, no hit-index, e MORTOS sob o mouse (a cicatriz das
    // 36 células do W2c).
    register_button_ids(store, &ids::INSP_PLAYER_MODE_IDS);
    store.set_tooltip(
        ids::INSP_PLAYER_MODE_IDS[0],
        "The floating capsule: impulses, a spring leg, the solver owns the pose.",
    );
    store.set_tooltip(
        ids::INSP_PLAYER_MODE_IDS[1],
        "The controller: the pose is written, the world only says how much fit.",
    );
    store.set_tooltip(
        ids::INSP_PLAYER_MODE_IDS[2],
        "Classic platformer: the same controller, but the physical world is \
         scenery. Everything stops him and he moves nothing.",
    );
    register_button_ids(
        store,
        &[
            ids::INSP_PLAYER_ADD,
            ids::INSP_PLAYER_REMOVE,
            ids::INSP_PLAYER_FIT,
            ids::INSP_PLAYER_CLEAR_RUN,
            ids::INSP_PLAYER_RESTORE_RUN,
            ids::INSP_PLAYER_FIT_CROUCH,
        ],
    );
    // ⚠️ **As DICAS saem da MESMA tabela que o pintor lê** (W9, Enio: *"precisamos
    // de dicas no mouse hover"*). O `set_tooltip` é a infra que o app já tem — o
    // hover publica o `hot_id`, e o passe de tooltip do `hero` procura o texto
    // dele —, então uma seção que não registra nada é uma seção sem dica, em
    // silêncio. Registrar aqui, num laço sobre a tabela, é o que faz um controle
    // novo nascer explicado.
    for (_, _, rows) in crate::sections::player::PLAYER_CARDS {
        for (_, id, tip) in rows {
            store.set_tooltip(*id, *tip);
        }
    }
    for (id, tip) in crate::sections::player::PLAYER_BUTTON_TIPS {
        store.set_tooltip(id, tip);
    }
    for (id, value, min, max, step) in [
        // Metros. O piso NÃO é zero por acaso: uma perna de comprimento zero é
        // um personagem que não paira, e o piso geométrico real é maior ainda
        // (o botão Fit to Collider o diz).
        (ids::INSP_PLAYER_FLOAT, 0.5, 0.01, 100.0, 0.01), // LITERAL-PX-OK: meters
        (ids::INSP_PLAYER_CLING, 0.25, 0.0, 100.0, 0.01), // LITERAL-PX-OK: meters
        // Aceleração-por-metro. Sem teto medido: o que aperta a rigidez é a
        // estabilidade do passo, e o amortecimento é quem a governa.
        (ids::INSP_PLAYER_STIFFNESS, 400.0, 0.0, 100_000.0, 1.0), // LITERAL-PX-OK: accel per metre
        // ⚠️ TETO MEDIDO em 1.0 (`RideConfig::MAX_DAMPING`): acima dele o boost
        // INVERTE a velocidade em vez de matá-la, e o personagem pipoca. É o
        // único teto desta seção que descreve um limite de estabilidade em vez
        // de conveniência de stepper.
        (ids::INSP_PLAYER_DAMPING, 0.5, 0.0, 1.0, 0.05), // LITERAL-PX-OK: fraction per tick
        // m/s, relativa ao chão.
        (ids::INSP_PLAYER_SPEED, 6.0, 0.0, 1000.0, 0.1), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_ACCEL, 60.0, 0.0, 10_000.0, 1.0), // LITERAL-PX-OK: m/s^2
        (ids::INSP_PLAYER_AIR_ACCEL, 20.0, 0.0, 10_000.0, 1.0), // LITERAL-PX-OK: m/s^2
        // Graus. O teto é 90 porque acima disso a superfície aponta para BAIXO
        // e a pergunta deixa de ter sentido — recurso, não conveniência.
        (ids::INSP_PLAYER_MAX_SLOPE, 45.0, 0.0, 90.0, 1.0), // LITERAL-PX-OK: degrees
        // O PULO (W4). A altura em METROS, que é o que o artista pensa; os seis
        // multiplicadores em fração de gravidade, onde `1.0` é a do mundo.
        (ids::INSP_PLAYER_JUMP_HEIGHT, 2.0, 0.0, 1000.0, 0.1), // LITERAL-PX-OK: metres
        // ⚠️ Os multiplicadores NÃO têm teto de recurso: o piso é 0 (gravidade
        // nenhuma naquela fase) e o topo é onde o desenho deixa de ser um pulo,
        // que é decisão de LOOK e não um limite físico. 20 é folga de sobra
        // sobre os defaults de 0,5..4 e é declarado como tal.
        (ids::INSP_PLAYER_TAKEOFF_G, 1.0, 0.0, 20.0, 0.1), // LITERAL-PX-OK: gravity multiple
        (ids::INSP_PLAYER_TAKEOFF_SPEED, 0.0, 0.0, 1000.0, 0.1), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_PEAK_G, 0.5, 0.0, 20.0, 0.1),    // LITERAL-PX-OK: gravity multiple
        (ids::INSP_PLAYER_PEAK_SPEED, 1.5, 0.0, 1000.0, 0.1), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_FALL_G, 2.0, 0.0, 20.0, 0.1),    // LITERAL-PX-OK: gravity multiple
        (ids::INSP_PLAYER_CUT_G, 4.0, 0.0, 20.0, 0.1),     // LITERAL-PX-OK: gravity multiple
        // ⚠️ **O TETO de 0,5 s é MEDIDO, e o recurso dele é a QUEDA:** a 0,5 s
        // o personagem já desceu `½·g·t² = 1,23 m` — mais de uma altura de
        // corpo (a cápsula tem 0,9 m) —, e a janela deixa de ler como *"eu
        // ainda estava na borda"* para ler como *"pulei do ar"*. A 0,1 s do
        // perfil de partida a queda é de 4,9 cm.
        (ids::INSP_PLAYER_COYOTE, 0.1, 0.0, 0.5, 0.01), // LITERAL-PX-OK: seconds
        (ids::INSP_PLAYER_BUFFER, 0.1, 0.0, 0.5, 0.01), // LITERAL-PX-OK: seconds
        // ⚠️ METROS, e o teto sai da MEDIÇÃO: acima de ~⅓ da largura do corpo a
        // assistência começa a salvar pulos que visivelmente bateram (a cápsula
        // das cenas tem 0,4 m). 0,3 é folga generosa para um corpo maior.
        (ids::INSP_PLAYER_CORNER, 0.12, 0.0, 0.3, 0.01), // LITERAL-PX-OK: metres
        // ⚠️ **O teto de 257 é MEDIDO e o recurso NÃO é tempo**
        // (`measure_player_probes::measure_what_a_sample_costs`): 18 ns por raio,
        // PLANO em N, então 257 custam 4,55 us = 0,027% de um quadro. O que se
        // esgota é a PRECISAO — o passo cai a 2,5 mm, e o solver assenta com
        // ~1,3 mm. O passo 2 é o clamp para ÍMPAR feito visível.
        (ids::INSP_PLAYER_CORNER_SAMPLES, 65.0, 1.0, 257.0, 2.0), // LITERAL-PX-OK: count
        // ⚠️ TIQUES, e 8 é folga larga: o default 2 é literalmente *"o boost no
        // tique ANTERIOR ao contato"*, e o leque se escala sozinho com a
        // velocidade (`rel_up · dt · N`), então um número grande aqui só antecipa
        // mais cedo — nunca alonga um sensor parado.
        (ids::INSP_PLAYER_CORNER_AHEAD, 2.0, 0.0, 8.0, 0.5), // LITERAL-PX-OK: ticks
        // ⚠️ Segundos, e o teto cobre o pulo mais longo da config de partida
        // (1,45 s no ar, medido) com folga para um `jump_height` maior.
        (ids::INSP_PLAYER_LIFT, 1.5, 0.0, 4.0, 0.05), // LITERAL-PX-OK: seconds
        // AS PAREDES (W13) — ⚠️ as duas primeiras nascem em ZERO: a capacidade
        // e' opt-in, e ligá-la por default mudaria todo player já autorado.
        (ids::INSP_PLAYER_WALL_SLIDE, 0.0, 0.0, 12.0, 0.25), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_WALL_JUMP, 0.0, 0.0, 6.0, 0.1),    // LITERAL-PX-OK: metres
        (ids::INSP_PLAYER_WALL_PUSH, 6.0, 0.0, 16.0, 0.25),  // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_WALL_LOCK, 0.2, 0.0, 0.6, 0.02),   // LITERAL-PX-OK: seconds
        (ids::INSP_PLAYER_WALL_REACH, 0.08, 0.0, 0.4, 0.01), // LITERAL-PX-OK: metres
        // ⚠️ O MESMO teto do irmão da quina, e pela MESMA medição — um número, um
        // argumento. O que N compra aqui é COBERTURA DE FRESTA (3 num corpo de
        // 1 m cegam-se com 0,5 m; 9, com 12,5 cm), não precisão de beirada.
        (ids::INSP_PLAYER_FOOT_SAMPLES, 3.0, 1.0, 257.0, 2.0), // LITERAL-PX-OK: count
        (ids::INSP_PLAYER_FOOT_SPREAD, 1.0, 0.0, 1.0, 0.05),   // LITERAL-PX-OK: fraction
        (ids::INSP_PLAYER_WALL_SAMPLES, 3.0, 1.0, 257.0, 2.0), // LITERAL-PX-OK: count
        // ⚠️ FRAÇÃO da meia-altura: 1 põe as amostras de fora na borda exata da
        // caixa (o mundo de sempre) e baixá-lo afasta-as das PONTAS, onde uma
        // cápsula é um ponto e um raio rasante vê parede onde o corpo mal encosta.
        (ids::INSP_PLAYER_WALL_SPREAD, 1.0, 0.0, 1.0, 0.05), // LITERAL-PX-OK: fraction
        // ⚠️ O teto de 10 s NAO e' um limite de recurso — nao ha' recurso, e' um
        // `f32`. E' a FAIXA da UI, e o numero vem da referencia: a reserva do
        // Celeste da' ~11 s de pendura pura. Acima disso o artista nao quer um
        // numero maior, quer INFINITO, que e' outra feature (uma habilidade que
        // se ganha, nao um recurso que se gasta).
        (ids::INSP_PLAYER_WALL_GRAB, 0.0, 0.0, 10.0, 0.25), // LITERAL-PX-OK: seconds
        (ids::INSP_PLAYER_DASH_SPEED, 0.0, 0.0, 40.0, 0.5), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_DASH_TIME, 0.15, 0.0, 1.0, 0.01), // LITERAL-PX-OK: seconds
        (ids::INSP_PLAYER_DASH_COOL, 0.2, 0.0, 2.0, 0.02),  // LITERAL-PX-OK: seconds
        (ids::INSP_PLAYER_CROUCH_HEIGHT, 0.0, 0.0, 3.0, 0.05), // LITERAL-PX-OK: metres
        (ids::INSP_PLAYER_CROUCH_SPEED, 2.0, 0.0, 20.0, 0.25), // LITERAL-PX-OK: m/s
        // O NADO (W-Swim). ⚠️ O teto do LIMIAR é `4`, e ele é MEDIDO: numa poça
        // quatro vezes mais densa que o corpo — a fixture destas waves — a razão
        // satura em `3,99` com o corpo todo submerso (`measure_the_swim_threshold`),
        // então acima disso o número deixaria de ser alcançável e a capacidade
        // ficaria desligada com o slider a dizer o contrário.
        (ids::INSP_PLAYER_SWIM_SPEED, 0.0, 0.0, 20.0, 0.25), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_SWIM_ACCEL, 12.0, 0.0, 60.0, 0.5), // LITERAL-PX-OK: m/s2
        (ids::INSP_PLAYER_SWIM_ENTER, 1.0, 0.0, 4.0, 0.05),  // LITERAL-PX-OK: weights
        // A REAÇÃO (W6), em FRAÇÃO da força que o personagem faz. ⚠️ O piso é 0
        // (nada volta) e o teto é 1 (volta inteira) porque **acima de 1 o
        // personagem devolveria mais do que recebeu** — inventar energia, e o
        // único teto desta seção que é de RECURSO e não de gosto.
        (ids::INSP_PLAYER_REACT_SUPPORT, 1.0, 0.0, 1.0, 0.05), // LITERAL-PX-OK: fraction
        (ids::INSP_PLAYER_REACT_MOVEMENT, 0.0, 0.0, 1.0, 0.05), // LITERAL-PX-OK: fraction
        (ids::INSP_PLAYER_REACT_PUSH, 1.0, 0.0, 1.0, 0.05),    // LITERAL-PX-OK: fraction
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
