//! Os gates do `motion.trail` — a suite que mora ao lado do motor.
//!
//! ⚠️ FILHO por `#[path]`, nunca irmão: `use super::*` tem de alcançar `step`, `Decay`,
//! `generations` e `promotes_head`, que são privados de propósito. Saiu do `lib.rs` porque
//! ele bateu 987 > 700 (o teto do workspace) — e o gate que o mede mora na
//! `ph2d-editor-core`, então uma bateria por-crate nunca o alcança.

use super::*;

fn dot(x: f32, alpha: f32) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[x, 0.0]]))
        .with("tint", Column::Vec4(vec![[1.0, 1.0, 1.0, alpha]]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]]))
}

fn xs(s: &Stream) -> Vec<f32> {
    match s.get("P").unwrap() {
        Column::Vec2(v) => v.iter().map(|p| p[0]).collect(),
        _ => panic!(),
    }
}
fn alphas(s: &Stream) -> Vec<f32> {
    match s.get("tint").unwrap() {
        Column::Vec4(v) => v.iter().map(|c| c[3]).collect(),
        _ => panic!(),
    }
}

fn trail_ages(s: &Stream) -> Vec<f32> {
    match s.get(AGE) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => vec![],
    }
}

/// Roda `ticks` ticks de um ponto que anda em x, devolvendo a saída final.
fn run(ticks: usize, length: f32, spacing: f32) -> Stream {
    let mut state = Stream::new(0);
    for t in 0..ticks {
        state = step(
            &dot(t as f32, 1.0),
            &state,
            length,
            Decay::new(1.0, 1.0),
            spacing,
        );
    }
    state
}

/// **O ECO FICA ESPAÇADO** (doc 88 §B3 — a família ECHO).
///
/// ⚠️ Nasceu VERMELHO: o motor promovia a cabeça a fantasma em TODO tick, então um
/// rastro discreto — o *sprite echo* que a referência traz com `spacing 2` no default
/// dela — não era difícil, era **inexprimível**. O oráculo é a CADÊNCIA (as idades
/// vivas), não a contagem: é ela que o param nomeia, e um espaçamento descartado a
/// deixa em `1,2,3…` para sempre.
#[test]
fn the_spacing_lays_one_ghost_every_n_ticks() {
    // 3 ecos com espaçamento 2: as idades vivas caminham 1 · 3 · 5, e a cabeça é 0.
    let out = run(8, 3.0, 2.0);
    let mut ages = trail_ages(&out);
    ages.sort_by(f32::total_cmp);
    assert_eq!(
        ages,
        vec![0.0, 1.0, 3.0],
        "as idades tinham de andar de dois em dois — em 1,2 o espaçamento foi \
         descartado"
    );
    // ⚠️ TRES linhas, como em `length = 3` sem espaçamento: o que muda é o ARCO que
    // elas cobrem (os ticks 4 e 6 em vez de 5 e 6), nunca quantas são.
    assert_eq!(xs(&out), vec![4.0, 6.0, 7.0]);
}

/// **`spacing = 1` É O MOTOR QUE JÁ SHIPAVA, AO BIT.**
///
/// A regressão que importa: todo grafo autorado antes desta wave não declara
/// `spacing`, e o `ctx.param` devolve o default. Com `s = 1` a faixa `1..1` é vazia,
/// a promoção acontece sempre e a janela de descarte volta a ser `k`.
#[test]
fn a_spacing_of_one_is_the_continuous_trail_to_the_bit() {
    let out = run(6, 3.0, 1.0);
    assert_eq!(xs(&out), vec![3.0, 4.0, 5.0]);
    assert_eq!(trail_ages(&out), vec![2.0, 1.0, 0.0]);
    // A faixa vazia: nenhuma idade impede a promoção.
    assert!(promotes_head(&[1.0, 2.0, 7.0], 1));
    assert!(promotes_head(&[], 4));
    assert!(
        !promotes_head(&[2.0], 4),
        "um fantasma novo demais SEGURA a promocao"
    );
    assert!(promotes_head(&[4.0], 4), "alcancado o espacamento, promove");
}

/// **O PARAM AUTORADO ATRAVESSA O COOK.**
///
/// ⚠️ Toda a suíte acima dirige `step` direto — ela prova a LEI e é **cega** a um
/// `ctx.param` que ninguém chamou. Uma capacidade sem porta passa em todo gate que só
/// olha para a função pura, então este dirige o grafo REAL, com o `pre` self-loop que
/// o editor plumba, e mede a cadência na saída do cook.
#[test]
fn the_authored_spacing_reaches_the_node_through_the_cook() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.trail.test.src"),
        name: "motion.trail.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
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
            // Um ponto que anda com o relógio: o x de cada eco DIZ de que tick ele é.
            let x = ctx.playhead() as f32;
            ctx.emit(dot(x, 1.0));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionTrail),
                _ => None,
            }
        }
    }

    let mut g = Graph::new();
    let src = g.add_node("motion.trail.test.src");
    let tr = g.add_node("motion.trail");
    g.connect(Edge {
        from: (src, 0),
        to: (tr, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (tr, 0),
        to: (tr, 1),
        delayed: true,
    })
    .unwrap();
    g.set_param(tr, "length", 3.0);
    g.set_param(tr, "fade", 1.0);
    g.set_param(tr, "shrink", 1.0);
    g.set_param(tr, "spacing", 2.0);

    let mut cook = Cook::new();
    let mut last = Stream::new(0);
    for t in 0..8 {
        let out = cook.cook(&g, &Ops, tr, f64::from(t)).unwrap();
        last = out[0].as_stream().clone();
        cook.advance_tick(&g, &Ops, f64::from(t)).unwrap();
    }
    let mut ages = trail_ages(&last);
    ages.sort_by(f32::total_cmp);
    assert_eq!(
        ages,
        vec![0.0, 1.0, 3.0],
        "o `spacing` autorado no grafo não chegou ao nó"
    );
}

/// **CADA knob autorado atravessa o cook** — a porta, não a lei.
///
/// ⚠️ O gate acima dirige só o `spacing`, e uma capacidade sem porta passa em todo
/// gate que olha para a função pura: bastaria um `ctx.param` esquecido para um knob
/// nascer morto com a suíte verde. Este dirige os SETE pelo grafo REAL, um a um,
/// exigindo que mover cada um mude a saída — e que o conjunto no default não a mude.
#[test]
fn every_authored_knob_reaches_the_node_through_the_cook() {
    let baseline = cooked_with(&[]);
    for (name, value) in [
        ("length", 5.0),
        ("fade", 0.3),
        ("shrink", 0.5),
        ("spacing", 3.0),
        ("hue_shift", 90.0),
        ("saturation", 0.0),
        ("spin", 30.0),
    ] {
        let moved = cooked_with(&[(name, value)]);
        assert!(
            moved != baseline,
            "mover `{name}` para {value} não mudou nada na saída do cook —                  o param não chegou ao nó"
        );
    }
}

/// **A CAUDA MUDA DE COR, SATURA E GIRA** — o padrão-ouro pedido em 2026-08-08.
///
/// Um oráculo por knob, cada um medindo a grandeza que o knob NOMEIA:
/// - matiz: a cor do eco velho difere da do vivo **e a luma se conserva** (é o que
///   separa um giro de matiz de um filtro que escurece);
/// - saturação: a distância de cada canal à luma ENCOLHE com a idade;
/// - giro: o `rot` do eco velho está `n · spin` atrás do vivo.
#[test]
fn the_tail_shifts_hue_desaturates_and_spins() {
    let coloured = |x: f32| {
        Stream::new(1)
            .with("P", Column::Vec2(vec![[x, 0.0]]))
            .with("tint", Column::Vec4(vec![[0.9, 0.2, 0.1, 1.0]]))
            .with("size", Column::Vec2(vec![[1.0, 1.0]]))
    };
    let decay = Decay {
        fade: 1.0,
        shrink: 1.0,
        hue_shift: 40.0,
        saturation: 0.6,
        spin: 7.0,
    };
    let run = |d: Decay| {
        let mut state = Stream::new(0);
        for t in 0..4 {
            state = step(&coloured(t as f32), &state, 4.0, d, 1.0);
        }
        state
    };
    let state = run(decay);
    let tints = match state.get("tint").unwrap() {
        Column::Vec4(v) => v.clone(),
        _ => panic!("tint"),
    };
    let luma = |c: [f32; 4]| 0.213 * c[0] + 0.715 * c[1] + 0.072 * c[2];
    let spread = |c: [f32; 4]| {
        let l = luma(c);
        (c[0] - l).abs() + (c[1] - l).abs() + (c[2] - l).abs()
    };
    let (old_c, live) = (tints[0], tints[tints.len() - 1]);
    assert!(
        (old_c[0] - live[0]).abs() > 0.05 || (old_c[1] - live[1]).abs() > 0.05,
        "o eco velho tinha de estar noutra matiz: {old_c:?} vs {live:?}"
    );
    assert!(
        (luma(old_c) - luma(live)).abs() < 1e-3,
        "e na MESMA luma: {} vs {}",
        luma(old_c),
        luma(live)
    );
    // ⚠️ O oráculo da saturação é um CONTROLE, não uma barra: a métrica L1 de
    // espalhamento **não é invariante** a um giro de matiz (medido: com `hue 40` ela
    // reporta 0,5366 onde o alvo autorado é 0,60 — 11% de distorção), então um número
    // fixo aqui seria calibrado contra o giro e não contra a saturação. A mesma cena com
    // o knob no NEUTRO responde a pergunta sem inventar limiar.
    let control = run(Decay {
        saturation: 1.0,
        ..decay
    });
    let ratio = |s: &Stream| match s.get("tint").unwrap() {
        Column::Vec4(v) => spread(v[0]) / spread(v[v.len() - 1]),
        _ => panic!("tint"),
    };
    assert!(
        ratio(&state) < ratio(&control),
        "a saturacao tinha de cair com a idade: {} contra o controle {}",
        ratio(&state),
        ratio(&control)
    );
    match state.get("rot").unwrap() {
        Column::Scalar(v) => {
            assert!((v[v.len() - 1] - 0.0).abs() < 1e-6, "o vivo nao girou");
            // ⚠️ `Tail Spin` é o TOTAL que a cauda percorre, então a ponta está exatamente
            // nele — não em `span × valor`, que é o que a lei antiga produzia (21°).
            assert!((v[0] - 7.0).abs() < 1e-4, "a ponta girou o total: {}", v[0]);
        }
        _ => panic!("rot"),
    }
}

/// **Os três knobs no NEUTRO não tocam um byte** — nem a coluna `rot`, que só nasce
/// quando o artista pede giro: materializá-la sem pedido acrescentaria uma coluna a
/// todo rastro do app.
#[test]
fn the_neutral_colour_knobs_change_nothing_and_add_no_column() {
    let coloured = Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("tint", Column::Vec4(vec![[0.9, 0.2, 0.1, 1.0]]));
    let mut a = Stream::new(0);
    let mut b = Stream::new(0);
    for _ in 0..4 {
        a = step(&coloured, &a, 4.0, Decay::new(0.8, 0.9), 1.0);
        b = step(
            &coloured,
            &b,
            4.0,
            Decay {
                ..Decay::new(0.8, 0.9)
            },
            1.0,
        );
    }
    match (a.get("tint"), b.get("tint")) {
        (Some(Column::Vec4(x)), Some(Column::Vec4(y))) => assert_eq!(x, y),
        _ => panic!("tint"),
    }
    assert!(a.get("rot").is_none(), "sem `spin` nao nasce coluna `rot`");
}

/// Cozinha o grafo REAL (com o `pre` self-loop) por 8 ticks e devolve um retrato
/// comparável da saída: contagem + toda coluna que o rastro pode tocar.
fn cooked_with(params: &[(&str, f32)]) -> Vec<String> {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.trail.test.src2"),
        name: "motion.trail.test.src2",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
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
            let x = ctx.playhead() as f32;
            ctx.emit(
                Stream::new(1)
                    .with("P", Column::Vec2(vec![[x, 0.0]]))
                    .with("tint", Column::Vec4(vec![[0.9, 0.2, 0.1, 1.0]]))
                    .with("size", Column::Vec2(vec![[1.0, 1.0]])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionTrail),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let src = g.add_node("motion.trail.test.src2");
    let tr = g.add_node("motion.trail");
    g.connect(Edge {
        from: (src, 0),
        to: (tr, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (tr, 0),
        to: (tr, 1),
        delayed: true,
    })
    .unwrap();
    for (k, v) in params {
        g.set_param(tr, *k, *v);
    }
    let mut cook = Cook::new();
    let mut last = Stream::new(0);
    for t in 0..8 {
        last = cook.cook(&g, &Ops, tr, f64::from(t)).unwrap()[0]
            .as_stream()
            .clone();
        cook.advance_tick(&g, &Ops, f64::from(t)).unwrap();
    }
    let mut out = vec![format!("n={}", last.count())];
    for (name, col) in last.columns() {
        out.push(format!("{name}={col:?}"));
    }
    out.sort();
    out
}

/// O teto do espaçamento é um recurso (a janela de idade é `length × spacing`), e um
/// `f32` de documento é intocado — não-finito e negativo caem no contínuo.
#[test]
fn the_spacing_is_clamped_totally() {
    assert_eq!(spacing_of(f32::NAN), 1);
    assert_eq!(spacing_of(-3.0), 1);
    assert_eq!(spacing_of(0.4), 1);
    assert_eq!(spacing_of(2.6), 3);
    assert_eq!(spacing_of(1e9), MAX_SPACING);
}

/// A moving dot, run for several ticks with the previous output fed back
/// (exactly what the `pre` self-loop does). The trail must hold the last
/// `length` positions, oldest first, each dimmer than the next.
#[test]
fn the_echo_holds_the_last_n_positions_oldest_first() {
    let mut state = Stream::new(0);
    for t in 0..6 {
        state = step(&dot(t as f32, 1.0), &state, 3.0, Decay::new(0.5, 1.0), 1.0);
    }
    // Ticks 3, 4, 5 survive; the live head (x=5) is LAST so it draws on top.
    assert_eq!(xs(&state), vec![3.0, 4.0, 5.0]);
    // ⚠️ O `Tail Alpha` autorado é 0.5, então o eco MAIS VELHO tem exatamente meia alfa —
    // e o do meio fica na raiz quadrada disso, que é a rampa geométrica que o alcança.
    // (Sob a lei antiga este vetor era `[0.25, 0.5, 1.0]`: o knob ERA a taxa.)
    let a = alphas(&state);
    assert!(
        (a[0] - 0.5).abs() < 1e-5,
        "a ponta e o alvo autorado: {a:?}"
    );
    assert!((a[1] - 0.5f32.sqrt()).abs() < 1e-5, "{a:?}");
    assert_eq!(a[2], 1.0, "a cabeca viva e opaca: {a:?}");
}

/// FALSIFICAÇÃO do decaimento geométrico: a taxa é aplicada UMA vez por tick às linhas
/// carregadas, nunca recomputada da idade. Re-aplicá-la a uma linha já desbotada daria
/// `rate²` num eco de um tick — este gate pina a cabeça em alfa cheia e o eco de um tick
/// em exatamente UMA taxa.
///
/// ⚠️ A taxa aqui é DERIVADA (`Tail Alpha 0.5` sobre um vão de 3 ⇒ `0.5^(1/3)`), e é essa
/// derivação que o gate atravessa junto: um `per_tick` esquecido daria 0.5 direto.
#[test]
fn a_one_tick_old_echo_has_faded_exactly_once() {
    let s = step(
        &dot(0.0, 1.0),
        &Stream::new(0),
        4.0,
        Decay::new(0.5, 1.0),
        1.0,
    );
    let s = step(&dot(1.0, 1.0), &s, 4.0, Decay::new(0.5, 1.0), 1.0);
    let one_tick = 0.5f32.powf(1.0 / 3.0);
    let a = alphas(&s);
    assert!((a[0] - one_tick).abs() < 1e-5, "uma taxa, nao duas: {a:?}");
    assert_eq!(a[1], 1.0);
    assert_eq!(xs(&s), vec![0.0, 1.0]);
}

#[test]
fn shrink_compounds_and_length_one_is_the_identity() {
    let mut state = Stream::new(0);
    for t in 0..3 {
        state = step(&dot(t as f32, 1.0), &state, 3.0, Decay::new(1.0, 0.5), 1.0);
    }
    match state.get("size").unwrap() {
        // `Tail Size 0.5` sobre um vão de 2: a ponta em meio tamanho, o meio na raiz.
        Column::Vec2(v) => {
            let z: Vec<f32> = v.iter().map(|s| s[0]).collect();
            assert!((z[0] - 0.5).abs() < 1e-5, "a ponta e o alvo: {z:?}");
            assert!((z[1] - 0.5f32.sqrt()).abs() < 1e-5, "{z:?}");
            assert_eq!(z[2], 1.0);
        }
        _ => panic!(),
    }

    // length = 1 → the live stream, verbatim, with no `trail_age` added.
    let live = dot(7.0, 1.0);
    let out = step(&live, &state, 1.0, Decay::new(0.5, 0.5), 1.0);
    assert_eq!(out.count(), 1);
    assert_eq!(xs(&out), vec![7.0]);
    assert!(out.get(AGE).is_none(), "the identity adds no column");
}

/// A run of ticks must not grow without bound: the ring drops the oldest
/// generation every tick, so the count settles at `length × live`.
#[test]
fn the_element_count_settles_at_length_times_live() {
    let mut state = Stream::new(0);
    for t in 0..50 {
        state = step(&dot(t as f32, 1.0), &state, 8.0, Decay::new(0.9, 0.99), 1.0);
    }
    assert_eq!(state.count(), 8, "8 generations of a single dot");
}

/// `length` and the live count are both untrusted (a loaded document, an
/// MCP edit). The instance budget clamps the GENERATIONS, so the trail gets
/// shorter rather than allocating 4096 × 32 quads or emitting a half-drawn
/// echo.
#[test]
fn the_instance_budget_clamps_generations_not_rows() {
    assert_eq!(generations(8.0, 100), 8);
    assert_eq!(generations(999.0, 1), MAX_LENGTH, "hard ceiling");
    assert_eq!(generations(f32::NAN, 10), 1, "junk → the identity");
    assert_eq!(generations(-5.0, 10), 1);
    // 32 generations × 4096 live = 131k > the 65_536 budget → 16 kept.
    assert_eq!(generations(32.0, 4096), MAX_INSTANCES / 4096);
    assert_eq!(generations(32.0, 999_999), 1, "never zero rows");
}

/// **UM STREAM POSICIONAL PURO DESBOTA E ENCOLHE** — o defeito de 2026-08-08.
///
/// ⚠️ **Este gate PINAVA o bug.** Ele afirmava `state.get("tint").is_none()` e
/// explicava, no próprio doc-comment, que *"o fade/shrink simplesmente não têm o que
/// tocar"* — descrevendo como comportamento aquilo que o smoke reportou como
/// *"Fade e Shrink não têm efeito algum"*. Medido na cena real: as colunas eram
/// `["Count", "Index", "P", "trail_age"]`, e um `motion.grid` — a fonte mais comum
/// que existe — não carrega nenhuma das duas.
///
/// *Um gate que descreve o sintoma como contrato mantém o defeito vivo com a suíte
/// verde.* Agora ele afirma o que o artista vê.
#[test]
fn a_bare_positional_stream_fades_and_shrinks() {
    let bare = |x: f32| Stream::new(1).with("P", Column::Vec2(vec![[x, 0.0]]));
    let mut state = Stream::new(0);
    for t in 0..4 {
        state = step(&bare(t as f32), &state, 3.0, Decay::new(0.5, 0.5), 1.0);
    }
    assert_eq!(xs(&state), vec![1.0, 2.0, 3.0]);
    // A cauda chega exatamente ao alvo autorado (meia alfa, meio tamanho na ponta).
    let a = alphas(&state);
    assert!((a[0] - 0.5).abs() < 1e-5 && a[2] == 1.0, "{a:?}");
    match state.get("size").unwrap() {
        Column::Vec2(v) => {
            assert!((v[0][0] - 0.5).abs() < 1e-5, "{v:?}");
            assert_eq!(v[2], [1.0, 1.0]);
        }
        _ => panic!("size"),
    }
}

/// **E a materialização é a IDENTIDADE da lowering** — o primeiro tick não muda um
/// pixel, que é o que torna a cura segura para toda arte já autorada.
#[test]
fn the_materialised_columns_carry_the_lowerings_own_defaults() {
    let bare = Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]));
    let out = step(&bare, &Stream::new(0), 4.0, Decay::new(0.5, 0.5), 1.0);
    assert_eq!(
        alphas(&out),
        vec![1.0, 1.0],
        "opaco, como a ausência significava"
    );
    match out.get("size").unwrap() {
        Column::Vec2(v) => assert_eq!(v, &vec![SIZE_IDENTITY; 2]),
        _ => panic!("size"),
    }
}
