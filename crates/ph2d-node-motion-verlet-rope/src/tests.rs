//! Testes unitários de `motion.verlet_rope` — a relaxação, o pino, a âncora
//! animada, o replay e a RIGIDEZ À FLEXÃO. Irmão de `lib.rs` pelo teto de 700
//! LOC (HR-18), no mesmo corte que a crate irmã `motion.boids` já usa: o pai
//! fica com o que a corda É, o filho com o que se afirma sobre ela.

use super::*;

fn params(count: usize, length: f32, gravity: f32, pin_tail: bool) -> Params {
    Params {
        count,
        seg_rest: length / (count as f32 - 1.0),
        rest: None,
        gravity,
        iterations: 32,
        damping: 0.0,
        pin_tail,
        // A fixture DECLARA o neutro em vez de o herdar: `0` é *sem rigidez
        // à flexão* e `1` é *um passo de integração por tique*, e escrevê-los
        // aqui é o que impede estes testes de mudarem de sentido em silêncio no
        // dia em que um default se mover.
        bend: 0.0,
        substeps: 1,
    }
}

/// The longest segment length in the chain (the stretch we constrain).
fn max_seg(pos: &[[f32; 2]]) -> f32 {
    pos.windows(2)
        .map(|w| {
            let d = [w[1][0] - w[0][0], w[1][1] - w[0][1]];
            (d[0] * d[0] + d[1] * d[1]).sqrt()
        })
        .fold(0.0, f32::max)
}

/// Drive `ticks` fixed steps through the pure `simulate`, feeding each tick's
/// output back as the next tick's state (what the `pre` loop does live).
fn run(anchor: impl Fn(f32) -> [f32; 2], p: &Params, ticks: usize, dt: f32) -> Vec<Stream> {
    let mut state = Stream::new(0); // Empty → tick 0 seeds
    let mut frames = Vec::new();
    for k in 0..ticks {
        let t = k as f32 * dt;
        let out = simulate(anchor(t), &state, t, p);
        state = out.clone();
        frames.push(out);
    }
    frames
}

fn pos_of(s: &Stream) -> Vec<[f32; 2]> {
    vec2_col(s, "P")
}

/// A retidão de cada dobradiça: `|p[i]−p[i+2]| / (2·seg_rest)`.
/// `1` = reta · `0` = dobrada ao meio.
///
/// ⚠️ **Ela SATURA em 1 e passa dele quando a corda ESTICA** — um vão de dois
/// segmentos maior que dois repousos não é rigidez, é a relaxação de
/// distância não dando conta sob gesto violento. Quem lê a tabela da sonda
/// tem de saber disso: `> 1` é estiramento, não retidão.
fn hinge_straightness(pos: &[[f32; 2]], rest: f32) -> Vec<f32> {
    pos.windows(3)
        .map(|w| {
            let d = [w[2][0] - w[0][0], w[2][1] - w[0][1]];
            (d[0] * d[0] + d[1] * d[1]).sqrt() / (2.0 * rest)
        })
        .collect()
}

/// A SACUDIDA 2D — o gesto que de fato dobra a corda sobre si (o único dos
/// quatro varridos que produz o fenômeno; ver `probe_what_bend_changes`).
fn shake(t: f32) -> [f32; 2] {
    [(t * 20.0).sin() * 7.0, (t * 17.0).cos() * 7.0]
}

/// **A rigidez à flexão impede a corda de dobrar sobre si — e o CONTROLE de
/// que ela dobra sem ela mora no mesmo teste.**
///
/// A relaxação de hoje só tem a restrição `i↔i+1`, que fixa o COMPRIMENTO e
/// não diz nada sobre o ÂNGULO: sacudida, a corda fecha uma dobradiça a
/// **27% da configuração reta** — o "dobra 180° sobre si" que a conferência
/// (doc 89, família 3) nomeia. Com `bend = 1` a mesma sacudida deixa a pior
/// dobradiça em **0,9982**.
///
/// ⚠️ **As duas metades num gate só, de propósito:** afirmar apenas que a
/// corda rígida fica reta seria verde numa fixture que nunca dobra, que é
/// exatamente o erro que as três primeiras tentativas desta wave cometeram
/// (balanço lento e chicote não dobram: mín 1,0007 e 1,0454).
#[test]
fn bend_stiffness_stops_the_rope_folding_on_itself() {
    let worst = |bend: f32| {
        let mut p = params(16, 6.0, 9.0, false);
        p.bend = bend;
        let frames = run(shake, &p, 240, 1.0 / 60.0);
        let last = pos_of(frames.last().unwrap());
        hinge_straightness(&last, p.seg_rest)
            .into_iter()
            .fold(f32::INFINITY, f32::min)
    };
    let (limp, stiff) = (worst(0.0), worst(1.0));
    assert!(
        limp < 0.5,
        "o CONTROLE: sem rigidez a corda TEM de dobrar, senao o gate abaixo e verde \
         por vacuo — pior dobradica {limp}"
    );
    assert!(
        stiff > 0.9,
        "com rigidez maxima a corda nao pode fechar uma dobradica: pior {stiff} \
         (sem rigidez, {limp})"
    );
}

/// **E a rigidez é MONOTÔNICA no knob** — a propriedade que um repouso errado
/// quebraria em silêncio. Puxar `i↔i+2` para UM segmento em vez de dois
/// dobraria a corda MAIS quanto mais rígida ela fosse, e o gate acima sozinho
/// nomearia o sintoma sem apontar a causa.
#[test]
fn more_bend_is_straighter_never_the_reverse() {
    let worst = |bend: f32| {
        let mut p = params(16, 6.0, 9.0, false);
        p.bend = bend;
        let frames = run(shake, &p, 240, 1.0 / 60.0);
        let last = pos_of(frames.last().unwrap());
        hinge_straightness(&last, p.seg_rest)
            .into_iter()
            .fold(f32::INFINITY, f32::min)
    };
    let steps = [0.0f32, 0.25, 1.0].map(worst);
    for w in steps.windows(2) {
        assert!(
            w[1] >= w[0] - 1e-3,
            "mais rigidez nao pode dobrar MAIS: {:?}",
            steps
        );
    }
}

/// Um gesto de âncora: o playhead entra, a posição da cabeça sai.
type Gesture = fn(f32) -> [f32; 2];

/// **SONDA — o que a rigidez muda, e sob que gesto.** É dela que sai a
/// fixture dos gates acima: dos quatro gestos varridos, só a sacudida 2D
/// dobra a corda.
///
/// ```text
/// cargo test -p ph2d-node-motion-verlet-rope probe_what_bend_changes -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: imprime a tabela que escolheu a fixture dos gates acima"]
fn probe_what_bend_changes() {
    let gestures: [(&str, Gesture); 4] = [
        ("balanco lento", |t| [(t * 3.0).sin() * 3.0, 0.0]),
        ("chicote rapido", |t| [(t * 18.0).sin() * 6.0, 0.0]),
        ("queda vertical", |t| [0.0, (t * 14.0).sin() * 8.0]),
        ("sacudida 2D", shake),
    ];
    for (name, gesture) in gestures {
        for b in [0.0f32, 0.25, 1.0] {
            let mut p = params(16, 6.0, 9.0, false);
            p.bend = b;
            let frames = run(gesture, &p, 240, 1.0 / 60.0);
            let last = pos_of(frames.last().unwrap());
            let h = hinge_straightness(&last, p.seg_rest);
            let mean = h.iter().sum::<f32>() / h.len() as f32;
            eprintln!(
                "{name:>16}  bend {b:>4}  retidao min {:>7.4}  media {mean:>7.4}",
                h.iter().copied().fold(f32::INFINITY, f32::min)
            );
        }
    }
}

/// Tick 0 seeds a straight horizontal strand from the anchor — the pin at the
/// anchor, each point one rest-length along +x, all collinear.
#[test]
fn seeds_a_straight_strand_at_tick_zero() {
    let p = params(5, 4.0, 9.0, false);
    let out = simulate([1.0, 2.0], &Stream::new(0), 0.0, &p);
    let pos = pos_of(&out);
    assert_eq!(pos.len(), 5);
    assert_eq!(pos[0], [1.0, 2.0], "head pinned at the anchor");
    for (i, q) in pos.iter().enumerate() {
        assert!((q[0] - (1.0 + i as f32)).abs() < 1e-5, "point {i} along +x");
        assert!((q[1] - 2.0).abs() < 1e-5, "flat (no gravity yet)");
    }
}

/// The defining property: the segments stay at their rest length even after
/// the rope has swung for a while under gravity. FALSIFIED against no
/// relaxation — an unconstrained Verlet chain stretches without bound as
/// gravity accelerates it, so the longest segment blows past the rest length.
#[test]
fn the_constraint_holds_the_segment_lengths() {
    let p = params(16, 6.0, 9.0, false);
    let frames = run(|_| [0.0, 0.0], &p, 200, 1.0 / 60.0);
    let last = pos_of(frames.last().unwrap());
    let longest = max_seg(&last);
    assert!(
        longest < p.seg_rest * 1.05,
        "segments held near rest {} (longest {longest}); an unconstrained chain stretches",
        p.seg_rest
    );
    // And it really did move (gravity did work) — the tail fell well below.
    assert!(
        last[15][1] < -1.0,
        "the free tail hangs down: {}",
        last[15][1]
    );
}

/// Gravity pulls the free tail DOWN below the pinned head over time. A dead
/// (zero-gravity) rope stays flat — the falsification.
#[test]
fn gravity_hangs_the_free_tail_below_the_anchor() {
    let live = params(12, 5.0, 12.0, false);
    let dead = params(12, 5.0, 0.0, false);
    let tail = |p: &Params| pos_of(run(|_| [0.0, 0.0], p, 180, 1.0 / 60.0).last().unwrap())[11];
    assert!(tail(&live)[1] < -1.0, "live gravity: tail sinks");
    assert!(
        (tail(&dead)[1]).abs() < 1e-3,
        "zero gravity: the tail stays flat"
    );
}

/// The pinned head tracks the (animated) anchor EXACTLY, every tick — the
/// whip's handle. A moving anchor also drags the rest of the chain along
/// (the tail is pulled well away from where a still anchor would leave it).
#[test]
fn a_moving_anchor_pins_the_head_and_whips_the_chain() {
    let p = params(14, 5.0, 6.0, false);
    // Anchor slides in +x with time.
    let frames = run(|t| [3.0 * t, 0.0], &p, 120, 1.0 / 60.0);
    for (k, f) in frames.iter().enumerate() {
        let head = pos_of(f)[0];
        let t = k as f32 / 60.0;
        assert!(
            (head[0] - 3.0 * t).abs() < 1e-4 && head[1].abs() < 1e-4,
            "head glued to the anchor at tick {k}: {head:?}"
        );
    }
    // The dragged rope ends up displaced in +x vs a still-anchored one.
    let still = run(|_| [0.0, 0.0], &p, 120, 1.0 / 60.0);
    let moved_tail = pos_of(frames.last().unwrap())[13][0];
    let still_tail = pos_of(still.last().unwrap())[13][0];
    assert!(
        moved_tail > still_tail + 0.5,
        "the moving anchor dragged the chain (+x {moved_tail} vs {still_tail})"
    );
}

/// `pin_tail` fixes BOTH ends: the tail holds its far point instead of falling,
/// and the middle sags between them (a suspension line).
#[test]
fn pin_tail_fixes_both_ends_into_a_hanging_line() {
    let p = params(11, 6.0, 12.0, true);
    let last = pos_of(run(|_| [0.0, 0.0], &p, 200, 1.0 / 60.0).last().unwrap());
    let far = PINNED_SPAN * (p.count as f32 - 1.0) * p.seg_rest;
    assert_eq!(last[0], [0.0, 0.0], "head pinned");
    assert!(
        (last[10][0] - far).abs() < 1e-3,
        "tail pinned at its far point"
    );
    assert!((last[10][1]).abs() < 1e-3, "tail holds its height");
    // The 25% slack sags the middle well below the endpoints.
    assert!(
        last[5][1] < -0.4,
        "the span sags in the middle: {}",
        last[5][1]
    );
}

/// Deterministic replay (HR-5: arithmetic + IEEE sqrt only): two runs of the
/// same rope produce bit-identical trajectories.
#[test]
fn replay_is_deterministic() {
    let p = params(20, 6.0, 9.0, false);
    let a = run(|t| [2.0 * t, 0.0], &p, 90, 1.0 / 60.0);
    let b = run(|t| [2.0 * t, 0.0], &p, 90, 1.0 / 60.0);
    for (fa, fb) in a.iter().zip(&b) {
        assert_eq!(pos_of(fa), pos_of(fb));
    }
}

/// Without the `pre` state loop the node re-seeds every tick → a straight,
/// still strand (the reference's "only simulates with feedback" footnote).
#[test]
fn without_the_state_loop_it_stays_a_straight_strand() {
    let p = params(8, 5.0, 12.0, false);
    for k in 0..30 {
        // Empty state every tick.
        let out = simulate([0.0, 0.0], &Stream::new(0), k as f32 / 60.0, &p);
        let pos = pos_of(&out);
        assert!(pos.iter().all(|q| q[1].abs() < 1e-6), "flat, no sim at {k}");
    }
}

/// A poisoned (non-finite) state point recovers at the anchor instead of
/// freezing the whole chain (reference NaN guard).
#[test]
fn non_finite_state_recovers() {
    let p = params(4, 3.0, 9.0, false);
    let state = Stream::new(4)
        .with(
            "P",
            Column::Vec2(vec![[0.0, 0.0], [f32::NAN, 1.0], [2.0, 0.0], [3.0, 0.0]]),
        )
        .with("rope_prev", Column::Vec2(vec![[0.0, 0.0]; 4]))
        .with("sim_t", Column::Scalar(vec![0.0; 4]));
    let out = simulate([0.0, 0.0], &state, 1.0 / 60.0, &p);
    assert!(
        pos_of(&out)
            .iter()
            .all(|q| q[0].is_finite() && q[1].is_finite()),
        "the diverged point was reset, not propagated"
    );
}

/// Cooks through the registry with the `pre` self-loop, exactly as the editor
/// wires it — proving the node is registered and steps live.
#[test]
fn registers_and_steps_through_the_cook() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionVerletRope as &dyn NodeOp)
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());

    let mut g = Graph::new();
    let rope = g.add_node("motion.verlet_rope");
    g.set_param(rope, "count", 10.0);
    g.set_param(rope, "gravity", 12.0);
    // The pre self-loop (out --pre--> state), as the plumbing auto-wires it.
    g.connect(Edge {
        from: (rope, 0),
        to: (rope, 2),
        delayed: true,
    })
    .unwrap();

    let mut cook = Cook::new();
    // Tick 0: seeded flat.
    let out0 = cook.cook(&g, &Ops, rope, 0.0).unwrap();
    assert!(matches!(out0[0].as_stream().get("P"), Some(Column::Vec2(v)) if v.len() == 10));
    // Advance a second of frames; the tail must have fallen.
    for k in 0..60 {
        let t = k as f64 / 60.0;
        cook.cook(&g, &Ops, rope, t).unwrap();
        cook.advance_tick(&g, &Ops, t).unwrap();
    }
    let out = cook.cook(&g, &Ops, rope, 1.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => {
            assert!(v[9][1] < -0.5, "the tail hangs after a second: {}", v[9][1])
        }
        _ => panic!("P"),
    }
}

/// **A FÓRMULA PBD REDUZ LITERALMENTE À TABELA DE QUATRO BRAÇOS QUE SHIPAVA.**
///
/// A relaxação carregava um `match (pinado_a, pinado_b)` com quatro braços; ela é a
/// forma booleana de `w_i/(w_i+w_j)` com pesos em `{0, 1}`, e a substituição é
/// **byte-idêntica por ARITMÉTICA e não por promessa**: `0/1`, `1/1` e `1/2` são
/// todos exactamente representáveis em IEEE-754, e o par degenerado `(0,0)` — que
/// a fórmula levaria a `0/0 = NaN` — cai no guard.
///
/// É este gate que torna seguro apagar o caso especial: a tabela morreu, e o que
/// ficou no lugar dela responde o mesmo em todo ponto onde ela respondia.
#[test]
fn the_pbd_share_reduces_to_the_four_arm_table() {
    // A tabela EXACTA que shipava, escrita à mão como oráculo — chamar a função sob
    // teste para computar o que se espera dela é o gate sempre-verde.
    let shipped = |pa: bool, pb: bool| match (pa, pb) {
        (true, true) => (0.0f32, 0.0f32),
        (true, false) => (0.0, 1.0),
        (false, true) => (1.0, 0.0),
        (false, false) => (0.5, 0.5),
    };
    for pa in [false, true] {
        for pb in [false, true] {
            let w = |pinned: bool| if pinned { 0.0f32 } else { 1.0f32 };
            let (ga, gb) = super::share(w(pa), w(pb));
            let (wa, wb) = shipped(pa, pb);
            assert_eq!(
                (ga.to_bits(), gb.to_bits()),
                (wa.to_bits(), wb.to_bits()),
                "({pa}, {pb}): a fórmula tem de dar os MESMOS bits da tabela",
            );
        }
    }
}

/// **E um peso FRACIONÁRIO parte a correção na proporção certa** — a metade que a
/// tabela booleana não sabia exprimir.
#[test]
fn the_share_splits_a_fractional_weight_in_proportion() {
    let (a, b) = super::share(1.0, 0.5);
    assert!(
        (a - 2.0 / 3.0).abs() < 1e-6 && (b - 1.0 / 3.0).abs() < 1e-6,
        "{a} {b}"
    );
    // E a soma é a unidade: a correção é INTEIRA, só muda quem a paga.
    assert!((a + b - 1.0).abs() < 1e-6);
}

/// **UM PONTO PREGADO EM VOO PARA — não coasta.**
///
/// ⚠️ Este gate existe porque uma MUTAÇÃO sobreviveu: apagar o early-out de massa
/// infinita deixava os seis gates de cadeia VERDES, e a razão é **defesa em
/// camadas** — com a aceleração já escalada por `w`, um ponto que chega ao pino
/// PARADO não se move de qualquer maneira, e na fixture de cadeia o pino chega no
/// tique 1, logo a seguir à semeadura, onde `pos == prev`.
///
/// A camada só é observável quando o ponto **já tem momento** — o caso de produto
/// em que o artista arma o pino a meio do voo —, e ali a inércia de Verlet
/// (`(c − pv)·keep`, que NÃO é escalada, e correctamente) o levaria a deslizar para
/// sempre. *Pinado* significa que ele para.
#[test]
fn a_point_pinned_in_flight_stops_instead_of_coasting() {
    let p = super::Params {
        count: 3,
        seg_rest: 1.0,
        rest: None,
        gravity: 0.0,
        iterations: 1,
        damping: 0.0,
        pin_tail: false,
        bend: 0.0,
        substeps: 1,
    };
    // ⚠️ **O movimento é PERPENDICULAR à corda, e a primeira fixture não o era:**
    // ao longo do eixo a restrição de distância desfaz a inércia no mesmo passe (o
    // controle mediu `x = 1` EXACTO, com o ponto livre a acabar onde começou), e um
    // gate assim não pode falhar pelo motivo que alega. Através do eixo a corda mal
    // resiste, e o ponto livre de facto viaja.
    let pos = vec![[0.0f32, 0.0], [1.0, 0.0], [2.0, 0.0]];
    let prev = vec![[0.0f32, 0.0], [1.0, -1.0], [2.0, 0.0]];
    let dt = 1.0 / 60.0;
    // CONTROLE: livre, a inércia leva-o adiante.
    let (free, _) = super::step(
        pos.clone(),
        &prev,
        &[],
        &[1.0, 1.0, 1.0],
        [0.0, 0.0],
        [0.0, 0.0],
        dt,
        &p,
    );
    assert!(
        free[1][1] > 0.3,
        "controle: livre a inércia leva o ponto adiante, e ele mediu y = {}",
        free[1][1]
    );
    // PREGADO EM VOO: ele para exactamente onde estava.
    let (held, _) = super::step(
        pos,
        &prev,
        &[],
        &[1.0, 0.0, 1.0],
        [0.0, 0.0],
        [0.0, 0.0],
        dt,
        &p,
    );
    assert_eq!(
        held[1][1].to_bits(),
        0.0f32.to_bits(),
        "pregado em voo, o ponto tem de PARAR; ele mediu y = {}",
        held[1][1]
    );
}
