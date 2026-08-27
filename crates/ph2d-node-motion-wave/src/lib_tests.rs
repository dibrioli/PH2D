//! Os gates do `motion.wave` — a lei do passo, o pino de Dirichlet e os
//! produtores injetados pela cadeia de estado.
//!
//! ⚠️ Irmão por `#[path]` e **FILHO** do `lib.rs`, não um módulo irmão de verdade:
//! o `use super::*` tem de alcançar os privados (`step`, `drive_value`, `Params`),
//! que é o precedente do `value.noise` e do `motion.strobe`.

use super::*;

pub(super) fn params(rows: usize, cols: usize, speed: f32, damping: f32) -> Params {
    Params {
        rows,
        cols,
        spacing: 0.5,
        speed,
        damping,
        center: [0.0, 0.0],
        channel: Channel::Size,
        edges: 0,
        sponge: Sponge::SHIPPED,
        inject_gain: 0.0,
    }
}

/// Run `drive(t)` for `ticks` fixed steps, returning each frame's height field.
fn run(drive: impl Fn(f32) -> f32, p: &Params, ticks: usize, dt: f32) -> Vec<Vec<f32>> {
    let mut state = Stream::new(0);
    let mut frames = Vec::new();
    for k in 0..ticks {
        let t = k as f32 * dt;
        let out = simulate(Some(drive(t)), &state, &[], t, p);
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
        let out = simulate(drive(t), &state, &[], t, p);
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

    let free = scalar_col(&simulate(None, &state, &[], dt, &p), "wave_h")[src];
    let pinned = scalar_col(&simulate(Some(0.0), &state, &[], dt, &p), "wave_h")[src];
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
    let h = scalar_col(&simulate(Some(0.7), &state, &[], 1.0 / 60.0, &p), "wave_h");
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
        let out = simulate(None, &state, &[], t, &p);
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
    let out = simulate(Some(0.0), &Stream::new(0), &[], 0.0, &p);
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
        let out = simulate(Some(1.0), &Stream::new(0), &[], k as f32 / 60.0, &p);
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
    let out_s = simulate(None, &state, &[], 0.0, &ps);
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
    let out_y = simulate(None, &state, &[], 0.0, &py);
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

// ---------------------------------------------------------------------------
// A BORDA ABSORVENTE (doc 89, folha 06 · célula 36 — o *Reflect Edges* do AE)
// ---------------------------------------------------------------------------

/// A energia do campo — a soma dos quadrados. É a grandeza que RESSOA numa caixa
/// fechada, e a única que distingue *"a onda parou"* de *"a onda está do outro lado"*.
fn energy(h: &[f32]) -> f32 {
    h.iter().map(|x| x * x).sum()
}

/// Um campo excitado por **um pulso só** e depois deixado em paz, sem amortecimento
/// global: é a fixtura em que uma parede reflectora se denuncia, porque nada mais no
/// modelo tira energia.
fn pulse_only(p: &Params, ticks: usize) -> Vec<Vec<f32>> {
    let mut state = Stream::new(0);
    let mut frames = Vec::new();
    for k in 0..ticks {
        // ⚠️ **O pulso vai no tique 1, nunca no 0.** O tique 0 SEMEIA o campo plano e
        // **descarta a fonte** (é o que o `seeds_a_flat_field` afirma), então uma
        // fixtura que dispara ali mede um campo que nunca foi excitado — as duas
        // metades saíram `0e0` e o gate reprovou por não conter o fenómeno, que é
        // exactamente o que a metade de controle existe para apanhar.
        let out = simulate((k == 1).then_some(1.0), &state, &[], k as f32 / 60.0, p);
        state = out.clone();
        frames.push(scalar_col(&out, "wave_h"));
    }
    frames
}

fn absorbing(rows: usize, cols: usize) -> Params {
    let mut p = params(rows, cols, 0.4, 0.0);
    p.edges = EDGES_ABSORB;
    p
}

/// ⭐⭐ **A ENTREGA: a caixa deixa de ressoar.** Com `damping = 0` uma parede
/// reflectora guarda a energia para sempre (ela só se redistribui); a esponja
/// deixa-a sair. O CONTROLE é o mesmo campo com a parede de sempre, e é ele que
/// mede o fenómeno — sem ele este gate só diria *"um número desceu"*.
#[test]
fn the_sponge_kills_the_ringing_a_reflecting_box_keeps() {
    const TICKS: usize = 400;
    let refl = pulse_only(&params(31, 31, 0.4, 0.0), TICKS);
    let absb = pulse_only(&absorbing(31, 31), TICKS);
    let (e_refl, e_absb) = (energy(&refl[TICKS - 1]), energy(&absb[TICKS - 1]));
    // O controle tem de RESSOAR — se ele próprio já tivesse morrido, a fixtura não
    // conteria o fenómeno e o gate estaria a medir nada.
    assert!(
        e_refl > 1e-3,
        "a fixtura nao contem o fenomeno: a caixa reflectora ja' esta' quieta ({e_refl:e})"
    );
    assert!(
        e_absb < e_refl * 0.05,
        "a esponja tem de deixar menos de 5% da energia que a parede guarda: \
         reflect {e_refl:e} · absorb {e_absb:e}"
    );
}

/// ⚠️ **A METADE JUSTA.** Sem ela, *"absorve"* podia ser implementado como
/// *"zera o campo"* e o gate acima passaria — a onda tem de continuar a atravessar
/// o miolo e a chegar lá com amplitude de verdade.
#[test]
fn the_absorbing_wall_does_not_kill_the_interior() {
    let p = absorbing(31, 31);
    let frames = pulse_only(&p, 60);
    // Uma célula a meio caminho entre o centro e a parede, longe da esponja.
    let probe = (p.rows / 2) * p.cols + p.cols / 2 + 6;
    let peak = frames.iter().map(|h| h[probe].abs()).fold(0.0f32, f32::max);
    let refl_peak = pulse_only(&params(31, 31, 0.4, 0.0), 60)
        .iter()
        .map(|h| h[probe].abs())
        .fold(0.0f32, f32::max);
    assert!(
        peak > refl_peak * 0.9,
        "a onda tem de chegar ao miolo praticamente inteira: absorb {peak:e} contra \
         reflect {refl_peak:e}"
    );
}

/// **A esponja só morde perto da parede.** No miolo ela é a identidade EXACTA — não
/// «quase», porque um factor de `0,999` no centro seria um amortecimento global
/// disfarçado de condição de fronteira.
#[test]
fn the_sponge_only_bites_near_the_wall() {
    let s = Sponge::SHIPPED;
    let (rows, cols) = (31, 31);
    for r in 0..rows {
        for c in 0..cols {
            let dr = r.min(rows - 1 - r);
            let dc = c.min(cols - 1 - c);
            let bite = s.at(r, c, rows, cols);
            #[expect(clippy::cast_precision_loss)]
            let far = (dr.min(dc) as f32) >= s.cells;
            if far {
                assert_eq!(bite, 1.0, "({r},{c}) fica no miolo e foi mordida: {bite}");
            } else {
                assert!(
                    bite < 1.0 && bite > 0.0,
                    "({r},{c}) mordida invalida: {bite}"
                );
            }
        }
    }
    // E a mordida é MONÓTONA para dentro da parede: a parede é o pior sítio.
    let wall = s.at(0, cols / 2, rows, cols);
    let inner = s.at(1, cols / 2, rows, cols);
    assert!(
        wall < inner,
        "a parede tem de morder mais: {wall} vs {inner}"
    );
    assert!(
        (wall - (1.0 - s.strength)).abs() < 1e-6,
        "no limite a mordida e' a `strength` declarada: {wall}"
    );
}

/// ⚠️ **`Reflect` é a caixa de sempre, e o perfil da esponja NÃO a alcança.** Um
/// perfil absurdo tem de deixar o campo reflector byte-idêntico — é isso que prova
/// que o modo novo não vazou para o caminho que shipava.
#[test]
fn a_reflecting_box_is_byte_identical_whatever_the_sponge_profile_is() {
    let base = pulse_only(&params(21, 21, 0.4, 0.0), 120);
    let mut wild = params(21, 21, 0.4, 0.0);
    wild.sponge = Sponge {
        cells: 40.0,
        strength: 0.99,
    };
    assert_eq!(
        base,
        pulse_only(&wild, 120),
        "o perfil vazou para o `Reflect`"
    );
}

/// A largura do miolo em que se procura o ECO, em células a contar da parede. É FIXA
/// e maior que a esponja mais larga que se varre — comparar profiles num miolo que
/// mudasse de tamanho com o profile mediria o recorte, não o eco.
const ECHO_INTERIOR: usize = 10;

/// **O ECO** — o maior `|h|` que aparece no miolo DEPOIS de a frente de saída já o ter
/// deixado. É isso que o artista vê como uma onda que volta da parede.
///
/// ⚠️ A janela começa em `40` porque a frente demora ~24 tiques a chegar à parede
/// (`c = √0,4 ≈ 0,63 células/tique` sobre 15 células) e a saída já limpou o miolo aos
/// ~11 — qualquer coisa depois disso veio de volta.
fn echo(frames: &[Vec<f32>], rows: usize, cols: usize) -> f32 {
    let mut m = 0.0f32;
    for h in frames.iter().skip(40) {
        for r in 0..rows {
            for c in 0..cols {
                let d = r.min(rows - 1 - r).min(c).min(cols - 1 - c);
                if d >= ECHO_INTERIOR {
                    m = m.max(h[r * cols + c].abs());
                }
            }
        }
    }
    m
}

/// **SONDA** — a grade dos dois números do perfil. ⚠️ *Uma esponja estreita e forte e
/// uma larga e fraca tiram energia parecida e reflectem quantidades muito diferentes*,
/// então varrer um com o outro fixo mediria a combinação e não a lei.
///
/// ⚠️⚠️ **A RÉGUA CORRIGIU-SE ANTES DE ESCOLHER NÚMERO NENHUM.** A 1.ª versão media a
/// *energia que sobra ao tique 400* e imprimiu **`0,003 %`–`0,24 %` nas trinta
/// combinações, sem monotonia nenhuma** — toda a grade estava no chão, e o que variava
/// entre células era ruído. Uma régua que dá o mesmo número para toda a grade não
/// escolheu nada; ela só diz que **qualquer** esponja mata a ressonância a longo prazo,
/// que é a pergunta do gate e não a deste varrimento. A grandeza que DISTINGUE é o
/// **eco** — o que volta da parede —, e é essa que o artista vê.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn sweeps_the_sponge_profile() {
    const TICKS: usize = 240;
    let base = pulse_only(&params(31, 31, 0.4, 0.0), TICKS);
    let refl = echo(&base, 31, 31);
    eprintln!("\n[esponja] o ECO que volta ao miolo, em % do que a parede reflectora devolve\n");
    eprintln!("  a caixa reflectora (o CONTROLE) = {refl:.6}\n");
    eprint!("  {:>9}", "cells\\str");
    for st in [0.02f32, 0.05, 0.10, 0.15, 0.25, 0.50, 1.0] {
        eprint!("  {st:>7.2}");
    }
    eprintln!();
    for cells in [2.0f32, 4.0, 6.0, 8.0, 12.0] {
        eprint!("  {cells:>9.0}");
        for strength in [0.02f32, 0.05, 0.10, 0.15, 0.25, 0.50, 1.0] {
            let mut p = absorbing(31, 31);
            p.sponge = Sponge { cells, strength };
            let e = echo(&pulse_only(&p, TICKS), 31, 31);
            eprint!("  {:>6.2}%", 100.0 * e / refl);
        }
        eprintln!();
    }
    eprintln!(
        "\n  LEITURA: procura-se o menor par que ja' esteja no joelho -- cada celula de
  esponja e' miolo que o artista deixa de poder usar, entao gastar largura depois do
  joelho e' preco sem produto. Um `strength = 1,0` e' a mascara que cai a ZERO na
  parede: se a coluna dele NAO for a melhor, a razao esta' escrita no
  `Sponge::strength` -- uma mudanca abrupta de impedancia reflecte."
    );
}

/// ⭐ **O PERFIL QUE SHIPA É O JOELHO MEDIDO, e o gate pina as DUAS leis** que a
/// varredura achou — não o número, que seria uma tautologia sobre uma constante.
///
/// ⚠️ *A 1.ª régua desta varredura media a energia que sobra ao tique 400 e imprimiu o
/// mesmo valor nas trinta combinações* (tudo no chão, sem monotonia): uma régua que dá
/// a mesma resposta para toda a grade não escolheu nada. Ver `sweeps_the_sponge_profile`.
#[test]
fn the_shipped_sponge_profile_sits_at_the_measured_knee() {
    let echo_of = |cells: f32, strength: f32| {
        let mut p = absorbing(31, 31);
        p.sponge = Sponge { cells, strength };
        echo(&pulse_only(&p, 240), 31, 31)
    };
    let s = Sponge::SHIPPED;
    let shipped = echo_of(s.cells, s.strength);
    // **LEI 1 — o U da mordida.** As duas pontas são piores que o que shipa, e por
    // razões diferentes: fraca demais mal absorve, forte demais reflecte na escada.
    for extreme in [0.02, 1.0] {
        let e = echo_of(s.cells, extreme);
        assert!(
            e > shipped * 1.15,
            "a mordida {extreme} tinha de ser pior que a que shipa: {e:.4} vs {shipped:.4}"
        );
    }
    // **LEI 2 — a largura paga até ao joelho e depois não.** Metade da largura é
    // NOTAVELMENTE pior; o dobro dela não compra praticamente nada, e é isso que
    // proíbe gastar mais miolo do artista.
    let half = echo_of(s.cells * 0.5, s.strength);
    let double = echo_of(s.cells * 2.0, s.strength);
    assert!(
        half > shipped * 1.05,
        "metade da largura tinha de ser pior: {half:.4} vs {shipped:.4}"
    );
    assert!(
        double > shipped * 0.9,
        "o dobro da largura compraria demais para o miolo que custa: \
         {double:.4} vs {shipped:.4}"
    );
}

// ---------------------------------------------------------------------------
// O `MAX_DT` QUE NÃO EXISTE (doc 91 §5.4 — a dívida que este bloco pagou)
// ---------------------------------------------------------------------------

/// ⭐⭐ **O PASSO DESTE NÓ NÃO DEPENDE DO RELÓGIO, e é por isso que aqui não há `MAX_DT`.**
///
/// Este nó carregava um `MAX_DT = 0,1` copiado dos integradores, com o doc a afirmar
/// estabilidade. Ele **não participava dela**: o leapfrog é um passo FIXO — a [`step`] nem
/// recebe `dt` — e o `dt` aparecia numa linha só, a do *"passou tempo?"*.
///
/// Este gate prende a propriedade: **N tiques dão o MESMO campo, seja qual for o espaçamento
/// deles**. FALSIFICADO por qualquer lei que faça o passo crescer com o relógio.
#[test]
fn the_field_after_n_ticks_does_not_depend_on_how_far_apart_they_were() {
    let p = params(15, 15, 0.4, 0.0);
    // `1/60` está muito abaixo do grampo antigo; `30 s` é 300× ACIMA dele.
    let close = pulse_only(&p, 40);
    let far = {
        let mut state = Stream::new(0);
        let mut frames = Vec::new();
        for k in 0..40 {
            let out = simulate((k == 1).then_some(1.0), &state, &[], k as f32 * 30.0, &p);
            state = out.clone();
            frames.push(scalar_col(&out, "wave_h"));
        }
        frames
    };
    assert_eq!(
        close.last(),
        far.last(),
        "o campo tem de ser o mesmo com tiques a 1/60 s e a 30 s de distancia"
    );
    // ⚠️ E o CONTROLE: o campo de facto EVOLUIU — senão a igualdade acima seria a de dois
    // campos planos, que qualquer lei satisfaz.
    let peak = close
        .last()
        .expect("tiques")
        .iter()
        .fold(0.0f32, |a, x| a.max(x.abs()));
    assert!(
        peak > 0.1,
        "a fixtura tem de conter o fenomeno: pico {peak}"
    );
}

/// Os gates do RELÓGIO — cortados pelo teto de LOC; ver o cabeçalho deles.
#[path = "clock_tests.rs"]
mod clock;
