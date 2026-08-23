//! Os gates do `motion.wave` — a lei do passo, o pino de Dirichlet e os
//! produtores injetados pela cadeia de estado.
//!
//! ⚠️ Irmão por `#[path]` e **FILHO** do `lib.rs`, não um módulo irmão de verdade:
//! o `use super::*` tem de alcançar os privados (`step`, `drive_value`, `Params`),
//! que é o precedente do `value.noise` e do `motion.strobe`.

use super::*;

fn params(rows: usize, cols: usize, speed: f32, damping: f32) -> Params {
    Params {
        rows,
        cols,
        spacing: 0.5,
        speed,
        damping,
        center: [0.0, 0.0],
        channel: Channel::Size,
    }
}

/// Run `drive(t)` for `ticks` fixed steps, returning each frame's height field.
fn run(drive: impl Fn(f32) -> f32, p: &Params, ticks: usize, dt: f32) -> Vec<Vec<f32>> {
    let mut state = Stream::new(0);
    let mut frames = Vec::new();
    for k in 0..ticks {
        let t = k as f32 * dt;
        let out = simulate(Some(drive(t)), &state, t, p);
        state = out.clone();
        frames.push(scalar_col(&out, "wave_h"));
    }
    frames
}

/// O mesmo laço, com a fonte podendo ESTAR AUSENTE num tique.
fn run_opt(drive: impl Fn(f32) -> Option<f32>, p: &Params, ticks: usize, dt: f32) -> Vec<Vec<f32>> {
    let mut state = Stream::new(0);
    let mut frames = Vec::new();
    for k in 0..ticks {
        let t = k as f32 * dt;
        let out = simulate(drive(t), &state, t, p);
        state = out.clone();
        frames.push(scalar_col(&out, "wave_h"));
    }
    frames
}

/// Um estado com o campo **INJETADO** — exactamente o que a cadeia
/// `field.box → value.attribute → motion.drive(Custom "wave_h")` entrega à porta
/// `state`. O `sim_t` fica no PASSADO, senão o ramo de *hold* segura o campo e o
/// passo nunca corre (a armadilha que o `measure_wave_producers` mediu na rota 2).
fn injected_state(p: &Params, cell: usize, amount: f32, t_prev: f32) -> Stream {
    let n = p.count();
    let mut h = vec![0.0f32; n];
    h[cell] = amount;
    Stream::new(n)
        .with("wave_h", Column::Scalar(h))
        .with("wave_prev", Column::Scalar(vec![0.0; n]))
        .with("sim_t", Column::Scalar(vec![t_prev; n]))
}

/// **A ENTREGA — sem fonte ligada não há pino de Dirichlet.** O oráculo mais
/// afiado é um produtor injetado NO PRÓPRIO centro: sob o pino ele é apagado
/// (o número redondo que só uma atribuição produz), sem o pino ele sobrevive.
/// FALSIFICADO por `next[source] = drive.unwrap_or(0.0)` — o campo livre voltaria
/// a ler `0` exacto.
#[test]
fn an_absent_drive_leaves_no_dirichlet_pin() {
    let p = params(11, 11, 0.35, 0.0);
    let src = p.source();
    let state = injected_state(&p, src, 0.75, 0.0);
    let dt = 1.0 / 60.0;

    let free = scalar_col(&simulate(None, &state, dt, &p), "wave_h")[src];
    let pinned = scalar_col(&simulate(Some(0.0), &state, dt, &p), "wave_h")[src];
    assert_eq!(
        pinned, 0.0,
        "o CONTROLE: com a fonte LIGADA em zero o centro e' cravado"
    );
    assert!(
        free.abs() > 0.1,
        "sem fonte nenhuma o centro nao e' cravado: {free}"
    );
}

/// **O CONTROLE da lei antiga:** uma fonte LIGADA continua a ser Dirichlet — ela
/// crava o centro no valor pedido, por cima do que a física acabou de computar.
/// FALSIFICADO por deixar de aplicar o pino.
#[test]
fn a_connected_drive_still_pins_the_centre() {
    let p = params(11, 11, 0.35, 0.0);
    let src = p.source();
    let state = injected_state(&p, src, 0.75, 0.0);
    let h = scalar_col(&simulate(Some(0.7), &state, 1.0 / 60.0, &p), "wave_h");
    assert_eq!(h[src], 0.7, "a fonte crava o centro, nao soma nele");
}

/// **BYTE-IDENTIDADE — a mudança não alcança nenhum documento de hoje.** Sem
/// fonte e sem injeção o campo é plano, e cravar zero num campo já plano é a
/// identidade: os dois caminhos dão o MESMO campo, tique a tique.
///
/// ⚠️ É esta a razão de a guarda ser segura: um documento que hoje deixa o
/// `drive` solto **não tem como excitar o campo**, então nada nele muda.
#[test]
fn an_absent_drive_over_an_unexcited_field_is_byte_identical() {
    let p = params(9, 9, 0.35, 0.02);
    let free = run_opt(|_| None, &p, 60, 1.0 / 60.0);
    let zeroed = run_opt(|_| Some(0.0), &p, 60, 1.0 / 60.0);
    assert_eq!(free, zeroed, "campo nao-excitado: as duas leis coincidem");
    assert!(
        free.last().unwrap().iter().all(|&z| z == 0.0),
        "e o campo e' de facto plano -- senao a igualdade acima seria vacua"
    );
}

/// **O PRODUTOR INJETADO PROPAGA.** Um bump que não se espalha é tinta no campo
/// de altura, não uma fonte: com o campo semeado fora do centro, uma célula que
/// a injeção nunca tocou tem de se mover. FALSIFICADO a `speed = 0`.
#[test]
fn an_injected_producer_radiates() {
    let p = params(11, 11, 0.35, 0.0);
    let cell = 5 * 11 + 2; // linha do meio, bem à esquerda do centro
    let far = 5 * 11 + 6; // quatro células à direita dela
    let mut state = injected_state(&p, cell, 1.0, -1.0 / 60.0);
    let mut last = Vec::new();
    for k in 0..40 {
        let t = k as f32 / 60.0;
        let out = simulate(None, &state, t, &p);
        state = out.clone();
        last = scalar_col(&out, "wave_h");
    }
    assert!(
        last[far].abs() > 1e-3,
        "a ondulacao injetada alcancou a celula distante: {}",
        last[far]
    );
}

/// Tick 0 seeds a flat field: `rows·cols` cells, every height 0 and every dot at
/// the baseline size.
#[test]
fn seeds_a_flat_field() {
    let p = params(5, 5, 0.3, 0.0);
    let out = simulate(Some(0.0), &Stream::new(0), 0.0, &p);
    assert_eq!(out.count(), 25);
    assert!(scalar_col(&out, "wave_h").iter().all(|&z| z == 0.0), "flat");
    match out.get("size").unwrap() {
        Column::Vec2(v) => assert!(v.iter().all(|s| (s[0] - SIZE_BASE).abs() < 1e-6)),
        _ => panic!("size"),
    }
}

/// The disturbance PROPAGATES outward: a driven centre eventually lifts a cell far
/// from the source. FALSIFIED at speed 0 — only the driven centre ever moves, the
/// rim stays flat (no propagation).
#[test]
fn a_driven_source_propagates_outward() {
    let p = params(11, 11, 0.35, 0.0);
    let corner = 0usize; // top-left, far from the centre
    let live = run(|t| (t * 30.0).clamp(-1.0, 1.0), &p, 120, 1.0 / 60.0);
    let dead = run(
        |t| (t * 30.0).clamp(-1.0, 1.0),
        &params(11, 11, 0.0, 0.0),
        120,
        1.0 / 60.0,
    );
    let live_corner = live.last().unwrap()[corner].abs();
    let dead_corner = dead.last().unwrap()[corner].abs();
    assert!(
        live_corner > 0.02,
        "the ripple reached the corner: {live_corner}"
    );
    assert!(
        dead_corner < 1e-6,
        "speed 0 never propagates: {dead_corner}"
    );
}

/// The wave travels at a FINITE speed: one tick after the first impulse only the
/// source's immediate neighbours have moved — a cell two rings out is still flat.
/// FALSIFIED by instantaneous coupling (the far cell would already be non-zero).
#[test]
fn propagation_is_finite_speed_not_instant() {
    let p = params(11, 11, 0.4, 0.0);
    // Tick 0 seeds flat; tick 1 drives the centre; tick 2 spreads to the 4
    // immediate neighbours only — a cell three away is still untouched.
    let frames = run(|_| 1.0, &p, 3, 1.0 / 60.0);
    let h2 = &frames[2];
    let src = p.source();
    let neighbour = src - 1; // same row, one cell left (adjacent to the source)
    let two_out = src - 3; // three cells left — beyond the first ring's reach
    assert!(h2[neighbour].abs() > 1e-4, "the near neighbour moved");
    assert!(
        h2[two_out].abs() < 1e-9,
        "a far cell has not been reached yet"
    );
}

/// Damping keeps a continuously-driven field BOUNDED and shrinks the far-field:
/// heavier damping leaves the rim quieter. FALSIFIED if damping did nothing (the
/// two rim amplitudes would match).
#[test]
fn damping_bounds_and_quiets_the_field() {
    // A bounded sawtooth oscillation in [-1, 1] (no trig, no TAU).
    let drive = |t: f32| 2.0 * (t * 5.0 - (t * 5.0).floor()) - 1.0;
    let soft = run(drive, &params(11, 11, 0.35, 0.12), 300, 1.0 / 60.0);
    let hard = run(drive, &params(11, 11, 0.35, 0.28), 300, 1.0 / 60.0);
    let rim = 0usize;
    let soft_rim = soft.iter().map(|f| f[rim].abs()).fold(0.0, f32::max);
    let hard_rim = hard.iter().map(|f| f[rim].abs()).fold(0.0, f32::max);
    // Everything stays finite (bounded), and heavier damping is quieter.
    assert!(
        soft.last().unwrap().iter().all(|z| z.is_finite()),
        "bounded"
    );
    assert!(
        hard_rim < soft_rim,
        "heavier damping quiets the rim (hard {hard_rim} < soft {soft_rim})"
    );
}

/// Deterministic replay (HR-5: pure arithmetic): two runs match bit-for-bit.
#[test]
fn replay_is_deterministic() {
    let p = params(9, 9, 0.35, 0.02);
    let a = run(|t| (t * 5.0).sin_cos_free(), &p, 90, 1.0 / 60.0);
    let b = run(|t| (t * 5.0).sin_cos_free(), &p, 90, 1.0 / 60.0);
    assert_eq!(a, b);
}

/// Without the state loop it re-seeds every tick → a flat field forever.
#[test]
fn without_the_state_loop_it_stays_flat() {
    let p = params(7, 7, 0.35, 0.0);
    for k in 0..20 {
        let out = simulate(Some(1.0), &Stream::new(0), k as f32 / 60.0, &p);
        // Only the driven centre is non-zero; everything else is flat (no history).
        let h = scalar_col(&out, "wave_h");
        for (i, &z) in h.iter().enumerate() {
            if i != p.source() {
                assert_eq!(z, 0.0, "no propagation without feedback at {i}");
            }
        }
    }
}

/// Cooks through the registry with the `pre` self-loop, emitting `P` + `size`.
#[test]
fn registers_and_ripples_through_the_cook() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionWave as &dyn NodeOp)
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();

    let mut g = Graph::new();
    let wave = g.add_node("motion.wave");
    g.set_param(wave, "rows", 9.0);
    g.set_param(wave, "cols", 9.0);
    g.connect(Edge {
        from: (wave, 0),
        to: (wave, 1),
        delayed: true,
    })
    .unwrap();

    // Drive stays unconnected (→ 0); the field still steps through the pre loop.
    let mut cook = Cook::new();
    let out0 = cook.cook(&g, &Ops, wave, 0.0).unwrap();
    assert!(matches!(out0[0].as_stream().get("P"), Some(Column::Vec2(v)) if v.len() == 81));
    assert!(
        out0[0].as_stream().get("size").is_some(),
        "emits a size column"
    );
    for k in 0..30 {
        let t = k as f64 / 60.0;
        cook.cook(&g, &Ops, wave, t).unwrap();
        cook.advance_tick(&g, &Ops, t).unwrap();
    }
    let out = cook.cook(&g, &Ops, wave, 0.5).unwrap();
    assert!(matches!(out[0].as_stream().get("P"), Some(Column::Vec2(v)) if v.len() == 81));
}

// A tiny transcendental-free stand-in for a bounded oscillator in the determinism
// test (keeps the test itself HR-5-clean).
trait BoundedOsc {
    fn sin_cos_free(self) -> f32;
}
impl BoundedOsc for f32 {
    fn sin_cos_free(self) -> f32 {
        // Triangle wave in [-1, 1] from the fractional part — no trig.
        let f = self - self.floor();
        (2.0 * (2.0 * f - 1.0).abs() - 1.0).clamp(-1.0, 1.0)
    }
}

/// **A ALTURA VAI PARA O CANAL ESCOLHIDO, E O SINAL SOBREVIVE FORA DO TAMANHO.**
///
/// ⚠️ **É o defeito que o doc 90 §5 nomeou e ninguém tinha curado.** O nó publicava `wave_h`
/// ASSINADO e escrevia `size = BASE + GAIN·|z|` como ÚNICO destino: uma crista e um vale
/// desenhavam a mesma bolha, e nada no painel dizia porquê. *O `abs()` nunca foi o erro — um
/// tamanho negativo não quer dizer nada; o erro era não haver para onde mais mandar a altura.*
///
/// O oráculo é a ASSIMETRIA: com o campo a ter alturas de sinais opostos, o canal `Y` tem de
/// separá-las e o `Size` tem de as confundir. Um `abs()` deixado no `Y` mata a primeira metade.
#[test]
fn the_height_reaches_the_chosen_channel_and_keeps_its_sign() {
    // Um campo com uma crista e um vale, montado à mão — a fixture CONTÉM o fenómeno.
    let p = params(1, 3, 0.35, 0.0);
    let h = vec![0.5f32, 0.0, -0.5];
    let state = Stream::new(3)
        .with("wave_h", Column::Scalar(h.clone()))
        .with("wave_prev", Column::Scalar(h.clone()))
        .with("sim_t", Column::Scalar(vec![0.0; 3]));

    let y_of = |s: &Stream| match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|q| q[1]).collect::<Vec<_>>(),
        _ => panic!("P"),
    };
    let size_of = |s: &Stream| match s.get("size") {
        Some(Column::Vec2(v)) => v.iter().map(|q| q[0]).collect::<Vec<_>>(),
        _ => panic!("size"),
    };

    // CANAL `Size` — o de sempre: a crista e o vale ENGORDAM igual (o `abs()` é correcto aqui).
    let mut ps = params(1, 3, 0.35, 0.0);
    ps.channel = Channel::Size;
    let out_s = simulate(None, &state, 0.0, &ps);
    let sz = size_of(&out_s);
    assert!(
        (sz[0] - sz[2]).abs() < 1e-6,
        "em Size a crista e o vale engordam igual: {sz:?}"
    );
    let ys = y_of(&out_s);
    assert!((ys[0] - ys[2]).abs() < 1e-6, "em Size o Y nao se mexe");

    // CANAL `Y` — a crista SOBE e o vale DESCE: o sinal sobrevive.
    let mut py = params(1, 3, 0.35, 0.0);
    py.channel = Channel::Y;
    let out_y = simulate(None, &state, 0.0, &py);
    let yy = y_of(&out_y);
    assert!(
        yy[0] - yy[2] > 0.5,
        "em Y a crista tem de ficar ACIMA do vale: {yy:?}"
    );
    // …e o tamanho fica no neutro, senão a altura responderia duas vezes.
    for v in size_of(&out_y) {
        assert!((v - SIZE_BASE).abs() < 1e-6, "em Y o tamanho fica neutro");
    }

    // CONTROLE: o campo tem de ter alturas de sinais opostos, senão o gate passa por vácuo.
    assert!(
        h[0] > 0.0 && h[2] < 0.0,
        "controle: a fixture tem crista E vale"
    );
    let _ = p;
}

/// **O DEFAULT É BYTE-IDÊNTICO AO QUE SHIPOU** — o canal novo não move uma cena que existe.
#[test]
fn the_default_channel_is_the_size_that_always_shipped() {
    assert_eq!(Channel::from_param(0.0), Channel::Size);
    // Um valor fora da escada cai no de sempre, nunca num canal por acidente.
    assert_eq!(Channel::from_param(9.0), Channel::Size);
    assert_eq!(Channel::from_param(1.0), Channel::Y);
    assert_eq!(Channel::from_param(2.0), Channel::Rotation);
}
