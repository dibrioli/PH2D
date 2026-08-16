//! Gates do `motion.strobe` — o envelope (attack · hold · decay · forma), a
//! probabilidade e a byte-identidade do neutro.
//!
//! ⚠️ FILHO por `#[path]`, não irmão: `use super::*` tem de alcançar o `step`, o
//! `glow_of` e o `fires`, que são privados do nó.

use super::*;

fn dot() -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]]))
        .with("tint", Column::Vec4(vec![[0.2, 0.2, 0.2, 1.0]]))
}
fn fire(v: f32) -> Stream {
    Stream::new(1).with(PULSE_COL, Column::Scalar(vec![v]))
}
/// ⚠️ **A fixture DECLARA os neutros em vez de os herdar.** Os seis gates
/// abaixo foram escritos contra o mundo pré-envelope, e é o facto de eles
/// passarem **sem uma edição de asserção** que prova que `attack = hold = 0`
/// e curva ausente reduzem à lei antiga. Uma fixture que chegasse ao neutro
/// por omissão inverteria de sentido no dia em que um default se movesse
/// [[reference_topic_fixture_discipline]].
fn params() -> Params {
    Params {
        decay: 0.5,
        attack: 0.0,
        hold: 0.0,
        probability: 1.0,
        curve: None,
        size_boost: 1.0,
        flash: [1.0, 1.0, 1.0],
        flash_amount: 1.0,
    }
}

/// Quantos ticks o brilho leva a chegar ao piso, dirigido pela MESMA porta
/// que o produto usa (nunca por um `glow * rate` à mão — um espelho da
/// recorrência ficaria verde sobre uma escada que já não a percorre).
fn life_of(decay_ticks: f32) -> u32 {
    let mut p = params();
    p.decay = decay_per_tick(decay_ticks);
    let (mut glow, mut age, mut n) = (1.0f32, 0.0f32, 0u32);
    while glow > GLOW_FLOOR && n < 10_000 {
        let (g, a) = glow_of(false, glow, age, &p);
        glow = g;
        age = a;
        n += 1;
    }
    n
}
fn glow(s: &Stream) -> f32 {
    match s.get(GLOW_COL).unwrap() {
        Column::Scalar(v) => v[0],
        _ => panic!(),
    }
}
fn size_x(s: &Stream) -> f32 {
    match s.get("size").unwrap() {
        Column::Vec2(v) => v[0][0],
        _ => panic!(),
    }
}

/// A pulse lights the element to full glow, then the envelope decays
/// geometrically (×decay per tick) — size and flash follow it down. The
/// upstream geometry is fresh each tick, so the boost never compounds.
#[test]
fn a_pulse_lights_then_the_envelope_decays_geometrically() {
    let p = params();
    // Tick 0: fire → glow 1.0, size ×(1+1·1)=2.0.
    let s = step(&dot(), &fire(1.0), &Stream::new(1), &p);
    assert_eq!(glow(&s), 1.0);
    assert_eq!(size_x(&s), 2.0);
    // Tick 1: no fire → glow ×0.5 = 0.5, size ×1.5 (from the FRESH unit size).
    let s = step(&dot(), &fire(0.0), &s, &p);
    assert_eq!(glow(&s), 0.5);
    assert_eq!(size_x(&s), 1.5);
    // Tick 2: glow 0.25, size ×1.25.
    let s = step(&dot(), &fire(0.0), &s, &p);
    assert_eq!(glow(&s), 0.25);
    assert_eq!(size_x(&s), 1.25);
}

/// FALSIFICATION of the "apply to fresh upstream, not to state" rule: the
/// size boost must not COMPOUND. After a pulse and one decay tick, size is
/// 1.5 (unit × 1.5), NOT 2.0 × 1.5 = 3.0 (which is what re-boosting the
/// already-boosted state would give).
#[test]
fn the_size_boost_does_not_compound_across_ticks() {
    let p = params();
    let s = step(&dot(), &fire(1.0), &Stream::new(1), &p); // size 2.0
    let s = step(&dot(), &fire(0.0), &s, &p);
    assert_eq!(
        size_x(&s),
        1.5,
        "boost applies to fresh geometry, not to 2.0"
    );
}

/// At full glow the tint reaches the flash colour (amount 1.0); with no glow
/// it is the untouched upstream tint. Alpha is never touched.
#[test]
fn the_flash_lerps_rgb_toward_the_flash_colour_leaving_alpha_alone() {
    let p = params();
    let lit = step(&dot(), &fire(1.0), &Stream::new(1), &p);
    match lit.get("tint").unwrap() {
        Column::Vec4(v) => {
            assert_eq!(v[0], [1.0, 1.0, 1.0, 1.0], "full flash = white, alpha kept")
        }
        _ => panic!(),
    }
    // Idle (no pulse, glow 0) → the upstream tint, verbatim.
    let dark = step(&dot(), &fire(0.0), &Stream::new(1), &p);
    match dark.get("tint").unwrap() {
        Column::Vec4(v) => assert_eq!(v[0], [0.2, 0.2, 0.2, 1.0]),
        _ => panic!(),
    }
}

/// A re-pulse mid-decay re-arms to full glow (the envelope retriggers, like a
/// bang restarting a `line~`). Not additive — it resets, it does not stack.
#[test]
fn a_re_pulse_retriggers_the_envelope_to_full() {
    let p = params();
    let s = step(&dot(), &fire(1.0), &Stream::new(1), &p);
    let s = step(&dot(), &fire(0.0), &s, &p); // glow 0.5
    let s = step(&dot(), &fire(1.0), &s, &p); // re-fire
    assert_eq!(glow(&s), 1.0, "retrigger resets to full, not 0.5+something");
}

/// The focus field gates the flash: two dots pulse together, but the one at
/// falloff 0 keeps its plain look while its neighbour lights up. The glow
/// MEMORY is per-instance and unmasked (the envelope keeps decaying), only
/// the applied look is masked — so animating the field over a live glow
/// fades the flash in and out without restarting it.
#[test]
fn falloff_zero_gates_the_flash_without_touching_the_envelope() {
    let p = params();
    let two = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]; 2]))
        .with("tint", Column::Vec4(vec![[0.2, 0.2, 0.2, 1.0]; 2]))
        .with("falloff", Column::Scalar(vec![1.0, 0.0]));
    let fire2 = Stream::new(2).with(PULSE_COL, Column::Scalar(vec![1.0, 1.0]));
    let s = step(&two, &fire2, &Stream::new(2), &p);
    match s.get("size").unwrap() {
        Column::Vec2(v) => {
            assert_eq!(v[0], [2.0, 2.0], "focused dot flashes");
            assert_eq!(v[1], [1.0, 1.0], "masked dot stays plain");
        }
        _ => panic!(),
    }
    match s.get("tint").unwrap() {
        Column::Vec4(v) => {
            assert_eq!(v[0], [1.0, 1.0, 1.0, 1.0], "focused dot lights up");
            assert_eq!(v[1], [0.2, 0.2, 0.2, 1.0], "masked dot untouched");
        }
        _ => panic!(),
    }
    // The envelope itself is NOT masked: both instances carry full glow.
    match s.get(GLOW_COL).unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![1.0, 1.0], "memory unmasked"),
        _ => panic!(),
    }
}

/// A bare positional stream (no size/tint) still gets those columns created
/// at their identities and modulated — the strobe is not a no-op on a
/// generator that only emits P.
#[test]
fn a_bare_stream_gains_size_and_tint_at_their_identities() {
    let p = params();
    let bare = Stream::new(1).with("P", Column::Vec2(vec![[5.0, 6.0]]));
    let lit = step(&bare, &fire(1.0), &Stream::new(1), &p);
    assert_eq!(size_x(&lit), 2.0, "unit identity ×2");
    match lit.get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v[0], [5.0, 6.0], "P passes through"),
        _ => panic!(),
    }
}

/// **O DEFAULT REPRODUZ O FLASH QUE JÁ SHIPAVA**, e não foi escolhido.
///
/// `34` ticks derivam exatamente a taxa `0.85` que o manifesto trazia — o strobe no
/// default não muda um pixel, e o que muda é que o número passou a ser linear no que
/// se vê.
#[test]
fn the_default_reproduces_the_shipped_rate() {
    let d = MANIFEST
        .params
        .iter()
        .find(|p| p.name == "decay")
        .expect("param")
        .default;
    assert_eq!(d, 34.0);
    let rate = decay_per_tick(d);
    assert!((rate - 0.85).abs() < 1e-3, "taxa derivada: {rate}");
}

/// **O FLASH DURA O QUE O SLIDER DIZ** — o gate do report de 2026-08-08.
///
/// ⚠️ Nasceu VERMELHO: o knob ERA a taxa, então `0.5` durava 8 ticks e `0.99` durava
/// 551 — a mesma unidade de curso do slider comprando dois tempos que diferem por
/// setenta vezes. O oráculo é o RELÓGIO: quantos ticks até o brilho chegar ao piso.
#[test]
fn the_flash_lasts_as_long_as_the_slider_says() {
    for want in [5u32, 15, 34, 60, 120] {
        let ticks = life_of(want as f32);
        assert!(
            ticks.abs_diff(want) <= 1,
            "{want} ticks pedidos, {ticks} medidos"
        );
    }
}

/// **O SLIDER É LINEAR NO QUE SE VÊ** — dobrar o número dobra o flash.
///
/// ⚠️ É a propriedade que a lei antiga não tinha: lá `0.5` durava 8 ticks e `0.99`
/// durava 551, então uma mesma fração do curso comprava tempos que diferiam por
/// setenta vezes.
#[test]
fn doubling_the_number_doubles_the_flash() {
    let life = |ticks: f32| life_of(ticks) as f32;
    for base in [10.0f32, 20.0, 40.0] {
        let r = life(base * 2.0) / life(base);
        assert!((r - 2.0).abs() < 0.1, "base {base}: a razao deu {r}");
    }
}

/// **A LEI ANTIGA, CONGELADA** — o `glow_of` que shipava, verbatim.
///
/// ⚠️ `pub` sem chamador seria uma **segunda resposta** esperando alguém a
/// chamar; sob `cfg(test)` ela é o que é: o oráculo contra o qual a escada
/// nova prova reduzir. (A lição do `warp_axis`/`serial_side` do Painter.)
fn legacy_glow(pulse: f32, prev_glow: f32, decay: f32) -> f32 {
    if pulse > 0.5 { 1.0 } else { prev_glow * decay }
}

/// **A ESCADA NOVA REDUZ À ANTIGA, BYTE A BYTE** — com `attack = hold = 0` e
/// nenhuma curva.
///
/// ⚠️ Os onze gates acima já passam sem uma edição, o que é a prova pelo
/// COMPORTAMENTO; este afirma a redução pelos BITS, sobre uma sequência de
/// pulsos irregular (o caso em que um retrigger cai no meio de uma queda).
#[test]
fn the_neutral_envelope_is_the_old_law_to_the_bit() {
    let p = params();
    let fire = [
        1.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0,
    ];
    let (mut now, mut age) = (0.0f32, 0.0f32);
    let mut old = 0.0f32;
    for (t, &f) in fire.iter().enumerate() {
        let (g, a) = glow_of(f > 0.5, now, age, &p);
        now = g;
        age = a;
        old = legacy_glow(f, old, p.decay);
        assert_eq!(
            now.to_bits(),
            old.to_bits(),
            "tick {t}: escada {now} contra a lei antiga {old}"
        );
    }
}

/// **O ATTACK SOBE, e o pico chega EXATAMENTE em `attack`.**
///
/// ⚠️ O oráculo não é *"o brilho difere"* — é a RAMPA inteira: `0` no tick do
/// pulso (um envelope começa em zero, o textbook do ADSR), estritamente
/// crescente, e `1.0` no tick `attack`. Um `attack` que só mudasse o número
/// sem subir passaria num gate de diferença.
#[test]
fn the_attack_ramps_from_zero_and_peaks_exactly_at_its_length() {
    let mut p = params();
    p.attack = 4.0;
    let (mut glow, mut age) = (0.0f32, 0.0f32);
    let mut seen = Vec::new();
    for t in 0..6 {
        let (g, a) = glow_of(t == 0, glow, age, &p);
        glow = g;
        age = a;
        seen.push(glow);
    }
    assert_eq!(seen[0], 0.0, "o envelope começa em ZERO: {seen:?}");
    for w in seen[..5].windows(2) {
        assert!(w[1] > w[0], "a rampa tem de subir: {seen:?}");
    }
    assert_eq!(seen[4], 1.0, "o pico cai no tick `attack`: {seen:?}");
    assert!(seen[5] < 1.0, "e depois dele começa a cair: {seen:?}");
}

/// **O HOLD é um PLATÔ, não uma queda mais lenta** — `hold` ticks em `1.0`
/// cravado, e só então a recorrência.
///
/// ⚠️ Sem a metade *"e só então CAI"* um `hold` que nunca soltasse o pico
/// passaria: o gate mede o platô E a saída dele.
#[test]
fn the_hold_is_a_plateau_and_then_it_falls() {
    let mut p = params();
    p.hold = 3.0;
    let (mut glow, mut age) = (0.0f32, 0.0f32);
    let mut seen = Vec::new();
    for t in 0..6 {
        let (g, a) = glow_of(t == 0, glow, age, &p);
        glow = g;
        age = a;
        seen.push(glow);
    }
    assert_eq!(&seen[..4], &[1.0, 1.0, 1.0, 1.0], "o platô: {seen:?}");
    assert!(seen[4] < 1.0 && seen[5] < seen[4], "e cai: {seen:?}");
}

/// **A CURVA molda o LOOK e NÃO entra no estado** — a metade que impede o
/// produto sobre a lista.
///
/// ⚠️ É este gate que a mutação *"guarde o valor moldado no `glow`"* sangra:
/// com a curva no estado, `c` seria aplicada uma vez por tick e a queda
/// viraria `c(c(c(…)))`. O oráculo é a COLUNA `glow` — ela tem de continuar a
/// ser a progressão geométrica crua, com a curva visível só no tamanho.
#[test]
fn the_curve_shapes_the_look_and_never_the_memory() {
    // Um degrau: tudo abaixo de 0,5 vira 0, tudo acima vira 1.
    const STEP: &str = "c1 0:0:H 0.5:1:H 1:1:H";
    let shaped = ph2d_curve::parse(STEP).expect("a curva parseia");

    let mut plain = Stream::new(1);
    let mut curved = Stream::new(1);
    let p_plain = params();
    let mut p_curved = params();
    p_curved.curve = Some(shaped);

    let mut raw_plain = Vec::new();
    let mut raw_curved = Vec::new();
    let mut size_curved = Vec::new();
    for t in 0..4 {
        plain = step(&dot(), &fire(f32::from(t == 0)), &plain, &p_plain);
        curved = step(&dot(), &fire(f32::from(t == 0)), &curved, &p_curved);
        raw_plain.push(glow(&plain));
        raw_curved.push(glow(&curved));
        size_curved.push(size_x(&curved));
    }

    assert_eq!(
        raw_plain, raw_curved,
        "a MEMÓRIA é a mesma nos dois — a curva não a compõe"
    );
    // E o LOOK difere: com `decay = 0.5` o brilho cru cai 1 → 0,5 → 0,25, e o
    // degrau em 0,5 leva o tamanho de 2,0 (cheio) direto a 1,0 (apagado) no
    // tick em que ele cruza — o que a rampa exponencial nunca faz.
    assert_eq!(size_x(&plain), 1.125, "controle: o cru é gradual");
    assert_eq!(size_curved[3], 1.0, "o degrau apaga: {size_curved:?}");
    assert_eq!(size_curved[0], 2.0, "e acende cheio no pico");
}

/// **A CURVA vem ANTES da máscara** — `curva(envelope) × falloff`.
///
/// ⚠️ Sem este gate a ordem trocada passa despercebida: os gates de `falloff`
/// usam `1.0` e `0.0`, e ali as duas ordens COINCIDEM (`c(g·1) = c(g)` e, com
/// uma curva que passa pela origem, `c(g·0) = c(0) = 0 = c(g)·0`). O
/// discriminante é uma influência PARCIAL — é lá que *"a máscara move o
/// artista ao longo da curva"* deixa de ser uma frase e vira um número.
#[test]
fn the_curve_is_read_before_the_mask_not_after() {
    // Degrau em 0,5: `c(1.0) = 1`, `c(0.5) = 1`, `c(0.4) = 0`.
    const STEP: &str = "c1 0:0:H 0.5:1:H 1:1:H";
    let mut p = params();
    p.curve = Some(ph2d_curve::parse(STEP).expect("a curva parseia"));

    let half = Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]]))
        .with("falloff", Column::Scalar(vec![0.5]));
    let lit = step(&half, &fire(1.0), &Stream::new(1), &p);

    // Envelope cheio (1,0) ⇒ curva ⇒ 1,0 ⇒ máscara 0,5 ⇒ tamanho 1,5.
    // A ordem trocada daria `c(1,0 × 0,5) = c(0,5) = 1` ⇒ tamanho 2,0.
    assert_eq!(
        size_x(&lit),
        1.5,
        "curva PRIMEIRO, máscara depois — 2.0 seria a ordem invertida"
    );
}

/// **`probability = 1` acende TODO pulso e não sorteia** (o default).
/// **`probability = 0` não acende NENHUM.**
#[test]
fn the_extremes_of_the_probability_are_all_and_none() {
    let mut all = params();
    all.probability = 1.0;
    let mut none = params();
    none.probability = 0.0;
    for row in 0..64usize {
        assert!(fires(1.0, &all, row, 0.0), "1.0 acende a linha {row}");
        assert!(!fires(1.0, &none, row, 0.0), "0.0 recusa a linha {row}");
        assert!(!fires(0.0, &all, row, 0.0), "sem pulso não há flash");
    }
}

/// **A probabilidade é a FRAÇÃO que acende** — medida sobre uma fileira, que
/// é onde ela vive (o AE sorteia por strobe, nós por instância).
#[test]
fn the_probability_is_the_fraction_that_lights() {
    for want in [0.25f32, 0.5, 0.75] {
        let mut p = params();
        p.probability = want;
        let lit = (0..2000usize).filter(|&r| fires(1.0, &p, r, 0.0)).count();
        #[expect(clippy::cast_precision_loss, reason = "contagem pequena")]
        let got = lit as f32 / 2000.0;
        assert!(
            (got - want).abs() < 0.05,
            "probabilidade {want} acendeu {got:.3}"
        );
    }
}

/// **UM SORTEIO RECUSADO NÃO TRAVA A INSTÂNCIA** — a pista avança em todo
/// pulso que CHEGA.
///
/// ⚠️ Este é o gate que a implementação óbvia (avançar só nos aceitos) falha:
/// ali um recusado re-sortearia o MESMO número e a linha nunca mais acenderia.
/// O oráculo é o produto, não o `fires`: a coluna da pista tem de subir num
/// tick em que o brilho não acendeu.
#[test]
fn a_refused_draw_does_not_lock_the_instance_out() {
    let mut p = params();
    p.probability = 0.5;
    // Uma fileira larga: com meia chance, alguma linha recusa o 1º pulso e
    // acende num pulso posterior — o que um sorteio travado torna impossível.
    let n = 200;
    let wide = Stream::new(n)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; n]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]; n]));
    let all_fire = Stream::new(n).with(PULSE_COL, Column::Scalar(vec![1.0; n]));
    let quiet = Stream::new(n).with(PULSE_COL, Column::Scalar(vec![0.0; n]));

    let mut st = Stream::new(n);
    let mut ever_refused_then_lit = 0;
    let mut refused_first = vec![false; n];
    for pulse_no in 0..6 {
        st = step(&wide, &all_fire, &st, &p);
        let Some(Column::Scalar(g)) = st.get(GLOW_COL) else {
            panic!("glow")
        };
        for (i, &gi) in g.iter().enumerate() {
            if pulse_no == 0 && gi < 0.5 {
                refused_first[i] = true;
            } else if pulse_no > 0 && refused_first[i] && gi >= 0.99 {
                ever_refused_then_lit += 1;
                refused_first[i] = false;
            }
        }
        st = step(&wide, &quiet, &st, &p); // um tick de silêncio entre pulsos
    }
    assert!(
        ever_refused_then_lit > 20,
        "linhas que recusaram e depois acenderam: {ever_refused_then_lit}"
    );
    // E a pista de facto ANDOU: 6 pulsos vistos por toda linha.
    let Some(Column::Scalar(seq)) = st.get(SEQ_COL) else {
        panic!("seq")
    };
    assert!(
        seq.iter().all(|&v| v == 6.0),
        "a pista conta os pulsos que CHEGARAM: {:?}",
        &seq[..4]
    );
}

/// O degenerado: duração zero (ou lixo de documento) é um flash de UM tick.
#[test]
fn the_degenerate_lengths_mean_what_the_artist_means() {
    assert_eq!(decay_per_tick(0.0), 0.0, "zero = um tick");
    assert_eq!(decay_per_tick(-3.0), 0.0);
    assert_eq!(decay_per_tick(f32::NAN), 0.0);
    assert_eq!(decay_per_tick(f32::INFINITY), 0.0);
}

/// **A DURAÇÃO AUTORADA ATRAVESSA O COOK** — a porta, não a lei.
///
/// ⚠️ Os seis gates que este arquivo já tinha constroem `Params` com a TAXA à mão, e
/// são portanto cegos a um `decay_per_tick` que ninguém tivesse ligado no `eval`: a
/// capacidade passaria em todos eles nascendo morta. Este dirige o grafo REAL, com o
/// `pre` self-loop, e mede o brilho depois de um pulso.
#[test]
fn the_authored_length_reaches_the_node_through_the_cook() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // ⚠️ DUAS fontes, como no produto: a geometria fala `Clock::Frame` e o pulso
    // fala `Clock::Event`. A 1ª versao deste gate cravou os dois na MESMA fonte de
    // Frame — o cook serviu o pulso do cache e ele ficou aceso para sempre, com o
    // brilho pregado em 1 nos dois bracos e o gate reprovando produto correto.
    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.strobe.test.src"),
        name: "motion.strobe.test.src",
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
    static PSRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.strobe.test.pulse"),
        name: "motion.strobe.test.pulse",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: PULSE,
        }],
        // ⚠️ `Temporal`, nao `Pure`: o fingerprint do cook so inclui o playhead para
        // um no TEMPORAL (`cook.rs`, `playhead: (effect == Temporal).then_some(..)`).
        // Com `Pure` esta fonte era servida do CACHE do tick 0 para sempre, o pulso
        // ficava aceso em todo tick, o brilho pregava em 1 nos dois bracos e o gate
        // reprovava produto correto. A fixture tem de dizer a verdade sobre si mesma.
        effect: Effect::Temporal,
        clock: Clock::Event,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(1)
                    .with("P", Column::Vec2(vec![[0.0, 0.0]]))
                    .with("size", Column::Vec2(vec![[1.0, 1.0]])),
            );
        }
    }
    struct PulseSrc;
    impl NodeOp for PulseSrc {
        fn manifest(&self) -> &'static NodeManifest {
            &PSRC
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            // Um pulso no tick 0 e mais nada: o brilho decai a partir dali.
            let fire = f32::from(u8::from(ctx.playhead() < 0.5));
            ctx.emit(Stream::new(1).with(PULSE_COL, Column::Scalar(vec![fire])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == PSRC.id => Some(&PulseSrc),
                t if t == MANIFEST.id => Some(&MotionStrobe),
                _ => None,
            }
        }
    }

    // Duas cenas identicas, so a DURACAO difere: a longa tem de brilhar mais no
    // mesmo tick. Um `eval` que ignorasse a derivacao daria o mesmo nos dois.
    let glow_at = |seconds: f32, at: u32| {
        let mut g = Graph::new();
        let src = g.add_node("motion.strobe.test.src");
        let psrc = g.add_node("motion.strobe.test.pulse");
        let st = g.add_node("motion.strobe");
        g.connect(Edge {
            from: (src, 0),
            to: (st, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (psrc, 0),
            to: (st, 1),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (st, 0),
            to: (st, 2),
            delayed: true,
        })
        .unwrap();
        g.set_param(st, "decay", seconds);
        g.set_param(st, "size_boost", 1.0);
        let mut cook = Cook::new();
        let mut last = Stream::new(0);
        for t in 0..=at {
            last = cook.cook(&g, &Ops, st, f64::from(t)).unwrap()[0]
                .as_stream()
                .clone();
            cook.advance_tick(&g, &Ops, f64::from(t)).unwrap();
        }
        // `size = 1 + size_boost*glow`, entao o tamanho LE o brilho.
        match last.get("size").unwrap() {
            Column::Vec2(v) => v[0][0] - 1.0,
            _ => panic!("size"),
        }
    };
    let (short, long) = (glow_at(3.0, 8), glow_at(120.0, 8));
    assert!(
        long > short * 2.0,
        "a duracao autorada nao chegou ao no: curta {short}, longa {long}"
    );
}

/// **QUEM NUNCA ACENDEU NÃO ACENDE SOZINHO** — o defeito que a cena `=46`
/// mostrou antes do smoke.
///
/// ⚠️ Um estado recém-nascido é zero em toda coluna, e `age = 0` significa *"um
/// pulso acabou de chegar"*. Com `attack = 0` os dois casos colapsam e o defeito
/// é invisível — é por isso que ele só nasceu com esta wave, e é por isso que a
/// fixture TEM de ter attack: sem ele o gate é verde por vácuo.
#[test]
fn an_instance_that_never_fired_does_not_swell_on_its_own() {
    let mut p = params();
    p.attack = 8.0;
    let mut st = Stream::new(1);
    for t in 0..12 {
        st = step(&dot(), &fire(0.0), &st, &p);
        assert_eq!(
            glow(&st),
            0.0,
            "tick {t}: sem pulso nenhum, o brilho tem de ser ZERO"
        );
        assert_eq!(size_x(&st), 1.0, "e o tamanho fica no de repouso");
    }
    // E o CONTROLE: um pulso ainda ACENDE — a sentinela não matou a feature.
    let lit = step(&dot(), &fire(1.0), &st, &p);
    assert_eq!(glow(&lit), 0.0, "a rampa começa em zero");
    let up = step(&dot(), &fire(0.0), &lit, &p);
    assert!(glow(&up) > 0.0, "e no tick seguinte ela SOBE: {}", glow(&up));
}
