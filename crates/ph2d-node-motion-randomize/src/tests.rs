//! Os gates do `motion.randomize`.

use super::*;
use ph2d_nodegraph::cook::OpResolver;

const N: usize = 40;
const SEED: u32 = 7;

/// Um stream com identidade e com as colunas que os canais tocam.
fn stream_with_ids(ids: &[u32]) -> Stream {
    let n = ids.len();
    Stream::new(n)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; n]))
        .with(
            "id",
            Column::Scalar(ids.iter().map(|i| *i as f32).collect()),
        )
        .with("rot", Column::Scalar(vec![0.0; n]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]; n]))
        .with("tint", Column::Vec4(vec![[0.4, 0.2, 0.8, 1.0]; n]))
}

/// ⭐ **`amount = 0` DEVOLVE A ENTRADA VERBATIM** — o nó recém-largado não mexe em nada.
#[test]
fn a_zero_amount_returns_the_input_untouched() {
    let ids: Vec<u32> = (0..N as u32).collect();
    let input = stream_with_ids(&ids);
    for ch in 0..CHANNEL_LABELS.len() as i32 {
        let out = cook_via_registry(&input, ch, 0.0);
        let (Some(Column::Scalar(a)), Some(Column::Scalar(b))) = (input.get("rot"), out.get("rot"))
        else {
            panic!("rot")
        };
        assert_eq!(a, b, "canal {ch} mexeu em `rot` com amount 0");
        let (Some(Column::Vec4(a)), Some(Column::Vec4(b))) = (input.get("tint"), out.get("tint"))
        else {
            panic!("tint")
        };
        for (i, (x, y)) in a.iter().zip(b).enumerate() {
            assert_eq!(
                x.map(f32::to_bits),
                y.map(f32::to_bits),
                "canal {ch} tint[{i}]"
            );
        }
    }
}

/// ⭐⭐ **A DISPERSÃO É POR IDENTIDADE, e a identidade é o `id`** — reordenar a lista não
/// troca o valor de dono.
///
/// ⚠️ É a armadilha 2 do doc do módulo: o `value.instance_field` nasce a chavear pelo
/// ÍNDICE, e num emissor a janela viva desliza. Aqui não há knob que possa estar errado.
#[test]
fn the_draw_follows_the_id_and_not_the_position_in_the_list() {
    let ids: Vec<u32> = (0..N as u32).collect();
    let a = cook_via_registry(&stream_with_ids(&ids), CH_ROTATION, 0.5);
    // A MESMA gente, noutra ordem.
    let mut rev = ids.clone();
    rev.reverse();
    let b = cook_via_registry(&stream_with_ids(&rev), CH_ROTATION, 0.5);
    let (Some(Column::Scalar(ra)), Some(Column::Scalar(rb))) = (a.get("rot"), b.get("rot")) else {
        panic!("rot")
    };
    for (i, id) in ids.iter().enumerate() {
        let j = rev.iter().position(|x| x == id).expect("o mesmo conjunto");
        assert_eq!(
            ra[i].to_bits(),
            rb[j].to_bits(),
            "a particula {id} trocou de valor ao mudar de posicao"
        );
    }
    // CONTROLE: a lista de facto mudou de ordem (senão o gate seria trivial).
    assert_ne!(ra[0].to_bits(), rb[0].to_bits(), "a ordem nao mudou");
}

/// ⚠️ **Sem coluna `id` a chave CAI no índice** — um conjunto sem identidade tem a
/// identidade que tem, e não uma em que todos partilham o mesmo valor.
#[test]
fn a_stream_without_ids_still_spreads() {
    let plain = Stream::new(N)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; N]))
        .with("rot", Column::Scalar(vec![0.0; N]));
    let out = cook_via_registry(&plain, CH_ROTATION, 0.5);
    let Some(Column::Scalar(r)) = out.get("rot") else {
        panic!("rot")
    };
    let mut seen: Vec<u32> = r.iter().map(|v| v.to_bits()).collect();
    seen.sort_unstable();
    seen.dedup();
    assert!(seen.len() > N / 2, "so' {} valores distintos", seen.len());
}

/// ⭐⭐⭐ **A LEI DO CANAL: ângulo SOMA, magnitude MULTIPLICA** — e as duas metades são
/// medidas contra a armadilha que a escolha errada produz.
///
/// ⚠️ Medido na composição que este nó substitui: `Add` numa opacidade saturada dá **um**
/// valor distinto em 21 partículas, e `Multiply` numa rotação nula dá **um**. Este gate
/// afirma que nenhuma das duas acontece aqui, seja qual for o canal.
#[test]
fn every_channel_actually_spreads_from_its_own_neutral() {
    let ids: Vec<u32> = (0..N as u32).collect();
    // O neutro de cada canal: rotação `0`, alfa `1` (saturada), tamanho `1`, cor viva.
    let input = stream_with_ids(&ids);
    for ch in 0..CHANNEL_LABELS.len() as i32 {
        let out = cook_via_registry(&input, ch, 0.6);
        let distinct = distinct_of(&out, ch);
        assert!(
            distinct > N / 2,
            "canal {} ({}) dispersou em {distinct} valores de {N} -- a escolha de operacao \
             esta' errada para ele",
            ch,
            CHANNEL_LABELS[ch as usize],
            distinct = distinct
        );
    }
}

/// ⚠️ **Um matiz DÁ A VOLTA e não é travado**, ao contrário das magnitudes — somar `+200°`
/// a um matiz de `300°` tem de aterrar em `140°`, e não colar em `360°`.
#[test]
fn the_hue_wraps_where_the_magnitudes_clamp() {
    let ids: Vec<u32> = (0..N as u32).collect();
    let input = stream_with_ids(&ids);
    let out = cook_via_registry(&input, CH_HUE, 1.0);
    let Some(Column::Vec4(t)) = out.get("tint") else {
        panic!("tint")
    };
    let hues: Vec<f32> = t.iter().map(|c| ph2d_color::rgb_to_hsv(*c).0).collect();
    let (lo, hi) = hues
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), h| (a.min(*h), b.max(*h)));
    // ⚠️ **A roda vale `1`, não `360`** — o `rgb_to_hsv` devolve o matiz em `[0,1)`, e uma
    // régua em graus aqui aceitaria um nó que roda três voltas e meia por centésimo de knob.
    assert!(
        hi - lo > 0.55,
        "com a volta inteira o matiz tinha de varrer a roda: {lo:.3}..{hi:.3}"
    );
    // E a alfa não se mexeu — o matiz é o matiz.
    for c in t {
        assert!((c[3] - 1.0).abs() < 1e-6, "o matiz mexeu na alfa: {}", c[3]);
    }
    // O CONTROLE do outro lado: a opacidade NUNCA passa de 1.
    let op = cook_via_registry(&input, CH_OPACITY, 1.0);
    let Some(Column::Vec4(t)) = op.get("tint") else {
        panic!("tint")
    };
    for c in t {
        assert!((0.0..=1.0).contains(&c[3]), "alfa fora da faixa: {}", c[3]);
    }
}

/// ⚠️ **Os dois eixos têm pistas PRÓPRIAS** — senão a posição dispersa na diagonal e o
/// tamanho nunca deixa de ser quadrado.
///
/// ⚠️ **Ele media só o TAMANHO, e uma mutação que juntou as pistas da POSIÇÃO sobreviveu.**
/// Os dois canais de duas lanes têm de ser perguntados: um gate que cobre metade de uma
/// família afirma sobre metade dela.
#[test]
fn the_two_axes_get_their_own_lane() {
    let ids: Vec<u32> = (0..N as u32).collect();
    let out = cook_via_registry(&stream_with_ids(&ids), CH_SIZE, 0.5);
    let Some(Column::Vec2(s)) = out.get("size") else {
        panic!("size")
    };
    let off = s.iter().filter(|q| (q[0] - q[1]).abs() > 1e-6).count();
    assert!(
        off > N * 3 / 4,
        "so' {off} de {N} deixaram de ser quadrados -- os eixos partilham a pista"
    );
    // E a POSIÇÃO: com uma pista só, todo deslocamento cai sobre a diagonal `x = y`.
    let out = cook_via_registry(&stream_with_ids(&ids), CH_POSITION, 0.5);
    let Some(Column::Vec2(p)) = out.get("P") else {
        panic!("P")
    };
    let diag = p.iter().filter(|q| (q[0] - q[1]).abs() > 1e-6).count();
    assert!(
        diag > N * 3 / 4,
        "so' {diag} de {N} sairam da diagonal -- os eixos da POSICAO partilham a pista"
    );
}

/// ⭐⭐ **O KNOB É UTILIZÁVEL: a excursão é PROPORCIONAL ao `amount`.**
///
/// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE.** Trocar a volta do matiz de `1` por
/// `360` passava em tudo o que havia — porque o gate anterior só perguntava *«a roda foi
/// varrida?»*, e varrê-la **dezoito vezes** também a varre. A pergunta que separa as duas é
/// outra: *com um centésimo do knob, o matiz anda um centésimo da roda?* Um nó cuja menor
/// dose já randomiza tudo tem um knob com uma posição útil, e é indistinguível de ruído.
#[test]
fn a_small_amount_buys_a_small_excursion() {
    let ids: Vec<u32> = (0..N as u32).collect();
    let input = stream_with_ids(&ids);
    let span = |ch: i32, amount: f32| -> f32 {
        let out = cook_via_registry(&input, ch, amount);
        match ch {
            CH_HUE => {
                let Some(Column::Vec4(t)) = out.get("tint") else {
                    panic!("tint")
                };
                let h: Vec<f32> = t.iter().map(|c| ph2d_color::rgb_to_hsv(*c).0).collect();
                let (lo, hi) = h
                    .iter()
                    .fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(*x), b.max(*x)));
                hi - lo
            }
            _ => {
                let Some(Column::Scalar(r)) = out.get("rot") else {
                    panic!("rot")
                };
                let (lo, hi) = r
                    .iter()
                    .fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(*x), b.max(*x)));
                (hi - lo) / TURN
            }
        }
    };
    // As duas famílias angulares, na FRACÇÃO da volta que cada uma varre.
    for ch in [CH_HUE, CH_ROTATION] {
        let small = span(ch, 0.05);
        let full = span(ch, 1.0);
        assert!(
            small < 0.25,
            "canal {ch}: com 5% do knob a excursao foi {small:.3} de volta -- a unidade da              volta esta' errada, e o knob inteiro e' ruido"
        );
        assert!(
            full > small * 4.0,
            "canal {ch}: a excursao tinha de CRESCER com o knob ({small:.3} -> {full:.3})"
        );
    }
}

/// O nó regista-se, e a escada de rótulos alcança o slider.
#[test]
fn the_node_registers_and_the_slider_reaches_every_channel() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).expect("regista");
    assert!(reg.resolve(MANIFEST.id).is_some());
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == "channel")
        .expect("hint do canal");
    assert_eq!(hint.max, (CHANNEL_LABELS.len() - 1) as f32);
    assert_eq!(MANIFEST.param_default("amount"), Some(0.0));
}

// ── o arnês ──────────────────────────────────────────────────────────────────────

/// Coze pelo caminho REAL (o `eval` do `NodeOp`), com um grafo mínimo.
fn cook_via_registry(input: &Stream, channel: i32, amount: f32) -> Stream {
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::{Edge, Graph};
    // A fonte que devolve o stream de teste — registada ao lado do nó em prova.
    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.randomize.test.src"),
        name: "motion.randomize.test.src",
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
    struct Src(Stream);
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(self.0.clone());
        }
    }
    struct Reg(Src, MotionRandomize);
    impl OpResolver for Reg {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            if ty == SRC.id {
                Some(&self.0 as &dyn NodeOp)
            } else if ty == MANIFEST.id {
                Some(&self.1 as &dyn NodeOp)
            } else {
                None
            }
        }
    }
    let reg = Reg(Src(input.clone()), MotionRandomize);
    let mut g = Graph::new();
    let s = g.add_node(SRC.name);
    let r = g.add_node(MANIFEST.name);
    g.set_param(r, "channel", channel as f32);
    g.set_param(r, "amount", amount);
    g.set_param(r, "seed", SEED as f32);
    g.connect(Edge {
        from: (s, 0),
        to: (r, 0),
        delayed: false,
    })
    .expect("liga");
    let mut cook = Cook::new();
    let out = cook.cook(&g, &reg, r, 0.0).expect("coze");
    out[0].as_stream().clone()
}

/// Quantos valores distintos o canal `ch` produziu.
fn distinct_of(s: &Stream, ch: i32) -> usize {
    let mut v: Vec<u32> = match ch {
        CH_ROTATION => match s.get("rot") {
            Some(Column::Scalar(r)) => r.iter().map(|x| x.to_bits()).collect(),
            _ => Vec::new(),
        },
        CH_POSITION => match s.get("P") {
            Some(Column::Vec2(p)) => p.iter().map(|q| q[0].to_bits()).collect(),
            _ => Vec::new(),
        },
        CH_SIZE => match s.get("size") {
            Some(Column::Vec2(z)) => z.iter().map(|q| q[0].to_bits()).collect(),
            _ => Vec::new(),
        },
        _ => match s.get("tint") {
            Some(Column::Vec4(t)) => t
                .iter()
                .map(|c| {
                    let (h, sa, va) = ph2d_color::rgb_to_hsv(*c);
                    match ch {
                        CH_OPACITY => c[3].to_bits(),
                        CH_HUE => h.to_bits(),
                        CH_SATURATION => sa.to_bits(),
                        _ => va.to_bits(),
                    }
                })
                .collect(),
            _ => Vec::new(),
        },
    };
    v.sort_unstable();
    v.dedup();
    v.len()
}
