//! Os gates do `motion.velocity` — a suíte que mora ao lado do motor.
//!
//! ⚠️ FILHO por `#[path]`, nunca irmão: `use super::*` tem de alcançar `step`, `velocity_of`,
//! `pairing` e `VEL`, que são privados de propósito.

use super::*;

/// Um stream de pontos, opcionalmente com identidade.
fn dots(p: &[[f32; 2]], ids: Option<&[f32]>) -> Stream {
    let mut s = Stream::new(p.len()).with("P", Column::Vec2(p.to_vec()));
    if let Some(ids) = ids {
        s.set("id", Column::Scalar(ids.to_vec()));
    }
    s
}

fn vels(s: &Stream) -> Vec<[f32; 2]> {
    match s.get(VEL) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("o no tem de emitir a coluna `vel`"),
    }
}

/// Um sexagésimo de segundo — o tick que o app de facto corre.
const DT: f32 = 1.0 / 60.0;

/// **A LEI, E A UNIDADE.**
///
/// ⚠️ O oráculo não é *"mexeu-se"*: é o NÚMERO em unidades por segundo. Andar 2 unidades num
/// tick de 1/60 s são **120 u/s**, e é essa a régua que o `motion.integrate` (`d += v·dt`) e o
/// `force.drag` do repo já falam — reportar deslocamento por TICK daria 2, e as duas leituras
/// só coincidem se `dt` for 1.
#[test]
fn a_moving_element_reports_units_per_second() {
    let prev = dots(&[[0.0, 0.0]], None);
    let live = dots(&[[2.0, -1.0]], None);
    let out = step(&live, &prev, DT, 0.0);
    let v = vels(&out)[0];
    assert!(
        (v[0] - 120.0).abs() < 1e-3 && (v[1] + 60.0).abs() < 1e-3,
        "andar (2,-1) num tick de 1/60 s sao (120,-60) u/s; veio {v:?}"
    );
}

/// **QUEM NAO TEM PASSADO NAO TEM VELOCIDADE.**
///
/// As três portas de ausência (estado vazio · id que o estado não conhece · `dt` zero) caem
/// todas em zero — e é a decisão que impede um motion-stretch de esticar um elemento que ainda
/// não se moveu.
#[test]
fn without_a_yesterday_the_velocity_is_zero_never_invented() {
    // Estado vazio: o primeiro tick de um cook.
    let out = step(&dots(&[[5.0, 5.0]], None), &Stream::new(0), DT, 0.0);
    assert_eq!(vels(&out), vec![[0.0, 0.0]], "estado vazio");

    // `dt` zero: o contrato do `EvalCtx` no 1º tick e depois de um reset.
    let out = step(
        &dots(&[[5.0, 5.0]], None),
        &dots(&[[0.0, 0.0]], None),
        0.0,
        0.0,
    );
    assert_eq!(vels(&out), vec![[0.0, 0.0]], "dt zero");

    // Um id que o estado não conhece — uma partícula que nasceu neste tick.
    let prev = dots(&[[0.0, 0.0]], Some(&[7.0]));
    let live = dots(&[[9.0, 0.0], [1.0, 0.0]], Some(&[7.0, 8.0]));
    let v = vels(&step(&live, &prev, DT, 0.0));
    assert!(v[0][0] > 1.0, "o id 7 tem passado e tem velocidade: {v:?}");
    assert_eq!(v[1], [0.0, 0.0], "o id 8 nasceu agora");
}

/// **A IDENTIDADE E POR `id`, NAO POR SLOT.**
///
/// ⚠️ Sem isto, um emitter — onde as linhas nascem e morrem a cada tick — reportaria a
/// velocidade de OUTRA partícula em toda linha depois da primeira morte, e o número seria
/// plausível: a fixture embaralha os slots de propósito, deixando cada elemento PARADO, então
/// um pareamento posicional produz velocidade onde a resposta certa é zero.
#[test]
fn the_pairing_follows_the_id_not_the_slot() {
    let prev = dots(&[[0.0, 0.0], [10.0, 0.0]], Some(&[1.0, 2.0]));
    // Os MESMOS dois elementos, nas MESMAS posições, com os slots trocados.
    let live = dots(&[[10.0, 0.0], [0.0, 0.0]], Some(&[2.0, 1.0]));
    let v = vels(&step(&live, &prev, DT, 0.0));
    assert_eq!(
        v,
        vec![[0.0, 0.0], [0.0, 0.0]],
        "ninguem se mexeu; um pareamento posicional teria reportado +-600 u/s: {v:?}"
    );
}

/// **`smooth = 0` E A DIFERENCA CRUA, BYTE A BYTE.**
///
/// O oráculo é a expressão escrita à mão, não a função sob teste — chamar `velocity_of` para
/// computar o que se espera é o gate sempre-verde.
#[test]
fn a_smooth_of_zero_is_the_raw_difference_to_the_bit() {
    let prev = dots(&[[0.3, -0.7]], None);
    let live = dots(&[[1.9, 2.5]], None);
    let raw = [(1.9f32 - 0.3) / DT, (2.5f32 + 0.7) / DT];
    assert_eq!(vels(&step(&live, &prev, DT, 0.0))[0], raw);
}

/// **O `smooth` E O ONE-POLE, E ELE APROXIMA SEM ULTRAPASSAR.**
///
/// A lei é a do `Blend` do `motion.delay`; o que a torna a escolha certa para uma diferença
/// finita é não ter overshoot — uma mola tocaria num sinal ruidoso.
#[test]
fn the_smooth_is_a_one_pole_that_approaches_and_never_overshoots() {
    let mut state = dots(&[[0.0, 0.0]], None);
    let target = 2.0 / DT;
    let mut seen = Vec::new();
    // O elemento anda exatamente 2 unidades por tick: a velocidade CRUA é constante.
    for k in 1..=8 {
        let live = dots(&[[2.0 * k as f32, 0.0]], None);
        state = step(&live, &state, DT, 4.0);
        seen.push(vels(&state)[0][0]);
    }
    for (a, b) in seen.iter().zip(seen.iter().skip(1)) {
        assert!(b > a, "o one-pole tem de subir monotonicamente: {seen:?}");
        assert!(*b <= target, "e nunca passar de {target}: {seen:?}");
    }
    assert!(
        seen[7] > target * 0.85,
        "e chegar perto em 8 ticks: {seen:?}"
    );
}

/// **A ARTE NAO SE MEXE UM PIXEL.**
///
/// ⚠️ Este nó MEDE. Filtrar `P` seria trabalho do `motion.delay`, e o gate existe porque a
/// diferença entre os dois é invisível numa foto: os dois desenham um traço mais suave.
#[test]
fn the_node_never_moves_the_art() {
    let live = dots(&[[1.0, 2.0], [3.0, 4.0]], None);
    let prev = dots(&[[0.0, 0.0], [0.0, 0.0]], None);
    for smooth in [0.0, 4.0, 32.0] {
        let out = step(&live, &prev, DT, smooth);
        assert_eq!(
            match out.get("P") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => panic!(),
            },
            vec![[1.0, 2.0], [3.0, 4.0]],
            "smooth {smooth} moveu a posicao"
        );
    }
}

/// **O STREAM DE ENTRADA ATRAVESSA INTEIRO.**
///
/// Um nó que reconstruísse o stream perderia `tint`, `size`, `id`, `texture_id` — o defeito que
/// o `motion.morph` shipou por um ano (doc 89, folha 07).
#[test]
fn every_other_column_survives() {
    let mut live = dots(&[[1.0, 0.0]], Some(&[3.0]));
    live.set("size", Column::Vec2(vec![[2.0, 2.0]]));
    live.set("tint", Column::Vec4(vec![[0.9, 0.1, 0.1, 1.0]]));
    let out = step(&live, &Stream::new(0), DT, 0.0);
    assert!(out.get("size").is_some(), "size");
    assert!(out.get("tint").is_some(), "tint");
    assert!(out.get("id").is_some(), "id");
}

/// **O `smooth` NAO TEM `ParamHardMax`, E A AUSENCIA E DELIBERADA.**
///
/// Medido: um one-pole com constante enorme é *lento*, nunca quebrado — ele continua a
/// convergir, monotonicamente, e não há valor em que o controle deixe de controlar. Sem este
/// gate a próxima varredura de tetos "completa" a tabela com um número que nada mede.
#[test]
fn a_huge_smooth_is_slow_and_never_broken() {
    let mut state = dots(&[[0.0, 0.0]], None);
    let mut last = 0.0f32;
    for k in 1..=6 {
        let live = dots(&[[2.0 * k as f32, 0.0]], None);
        state = step(&live, &state, DT, 1.0e6);
        let v = vels(&state)[0][0];
        assert!(
            v > last && v.is_finite(),
            "a constante enorme tem de continuar a convergir, so devagar: {v}"
        );
        last = v;
    }
}

/// **O PARAM AUTORADO ATRAVESSA O COOK.**
///
/// ⚠️ Toda a suíte acima dirige `step` direto — ela prova a LEI e é **cega** a um `ctx.param`
/// que ninguém chamou. Este dirige o grafo REAL, com o `pre` self-loop que o editor plumba.
#[test]
fn the_authored_smooth_reaches_the_node_through_the_cook() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.velocity.test.src"),
        name: "motion.velocity.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Temporal,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            // Um ponto que anda 2 unidades por tick.
            let x = ctx.playhead() as f32 * 2.0;
            ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[x, 0.0]])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionVelocity),
                _ => None,
            }
        }
    }

    let run = |smooth: f32| {
        let mut g = Graph::new();
        let src = g.add_node("motion.velocity.test.src");
        let vn = g.add_node("motion.velocity");
        g.set_param(vn, "smooth", smooth);
        g.connect(Edge {
            from: (src, 0),
            to: (vn, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (vn, 0),
            to: (vn, 1),
            delayed: true,
        })
        .unwrap();
        let mut cook = Cook::new();
        let mut last = Stream::new(0);
        // O playhead anda de 1 em 1 segundo, então `dt` é 1 e a velocidade crua é 2 u/s.
        for t in 0..4 {
            let t = f64::from(t);
            last = cook.cook(&g, &Ops, vn, t).unwrap()[0].as_stream().clone();
            cook.advance_tick(&g, &Ops, t).unwrap();
        }
        vels(&last)[0][0]
    };

    let raw = run(0.0);
    let smoothed = run(8.0);
    assert!(
        (raw - 2.0).abs() < 1e-4,
        "a diferenca crua sao 2 u/s; veio {raw}"
    );
    assert!(
        smoothed < raw * 0.75,
        "o `smooth` autorado nao chegou ao no: cru {raw}, suavizado {smoothed}"
    );
}
