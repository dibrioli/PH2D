//! Os gates do `motion.drive` — extraídos do `lib.rs` pelo teto de 700 LOC (a wave dos
//! canais de COR o levou a 922). Segue FILHO por `#[path]`, então `use super::*` continua
//! alcançando os privados (`channel::*`, as variantes de GPU) sem abrir visibilidade.

use super::*;
use ph2d_nodegraph::attr::Stream;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

// A source: 2 instances at the origin, plus a value node emitting one value.
static GRID_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.drive.test.grid"),
    name: "motion.drive.test.grid",
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
struct Grid;
impl NodeOp for Grid {
    fn manifest(&self) -> &'static NodeManifest {
        &GRID_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]])));
    }
}
static VAL_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.drive.test.val"),
    name: "motion.drive.test.val",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct Val;
impl NodeOp for Val {
    fn manifest(&self) -> &'static NodeManifest {
        &VAL_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(1).with(VALUE_COL, Column::Scalar(vec![3.0])));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == GRID_MAN.id => Some(&Grid),
            t if t == VAL_MAN.id => Some(&Val),
            t if t == MANIFEST.id => Some(&MotionDrive),
            _ => None,
        }
    }
}

fn drive_graph(setup: impl FnOnce(&mut Graph, NodeId)) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let grid = g.add_node("motion.drive.test.grid");
    let val = g.add_node("motion.drive.test.val");
    let drive = g.add_node("motion.drive");
    g.connect(Edge {
        from: (grid, 0),
        to: (drive, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (val, 0),
        to: (drive, 1),
        delayed: false,
    })
    .unwrap();
    setup(&mut g, drive);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, drive, 0.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("P"),
    }
}

/// The end-to-end value path through the cook: a length-1 value from a
/// separate node drives a channel of the instance stream, broadcast to all
/// instances. This is what proves the value domain is wired (value produced
/// by one node, consumed by another, made visible).
#[test]
fn a_value_node_drives_the_grid_channel_through_the_cook() {
    // scale 0.5, add → each instance X += 3 · 0.5 = 1.5.
    let p = drive_graph(|g, d| {
        g.set_param(d, "channel", 0.0); // X
        g.set_param(d, "scale", 0.5);
    });
    assert_eq!(
        p,
        vec![[1.5, 0.0], [1.5, 0.0]],
        "value broadcast to both, scaled"
    );
}

/// The value-domain WIN a bundled node can't do: ONE value node fans out to
/// TWO drives — X and Rotation — off the same value. `motion.step` (reduce +
/// apply in one node) can only touch one channel; the split lets a single
/// count animate several. Proves the value is a first-class thing that flows,
/// not a private computation.
#[test]
fn one_value_fans_out_to_two_channels() {
    let mut g = Graph::new();
    let grid = g.add_node("motion.drive.test.grid");
    let val = g.add_node("motion.drive.test.val"); // emits 3.0
    let drive_x = g.add_node("motion.drive");
    let drive_r = g.add_node("motion.drive");
    // grid → drive_x.in → drive_r.in ; val → both drives' value port.
    for (from, to) in [((grid, 0), (drive_x, 0)), ((drive_x, 0), (drive_r, 0))] {
        g.connect(Edge {
            from,
            to,
            delayed: false,
        })
        .unwrap();
    }
    for d in [drive_x, drive_r] {
        g.connect(Edge {
            from: (val, 0),
            to: (d, 1),
            delayed: false,
        })
        .unwrap();
    }
    g.set_param(drive_x, "channel", 0.0); // X += 3
    g.set_param(drive_r, "channel", 2.0); // Rotation += 3
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, drive_r, 0.0).unwrap();
    let s = out[0].as_stream();
    match s.get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v[0], [3.0, 0.0], "X driven by the value"),
        _ => panic!("P"),
    }
    match s.get("rot").unwrap() {
        Column::Scalar(v) => assert_eq!(v[0], 3.0, "Rotation driven by the SAME value"),
        _ => panic!("rot"),
    }
}

/// FALSIFICATION: with the value input UNCONNECTED (empty value field), the
/// drive is a no-op — the channel passes through untouched. A drive that
/// invented a value would move the grid off an empty input.
#[test]
fn an_unconnected_value_leaves_the_channel_untouched() {
    let mut g = Graph::new();
    let grid = g.add_node("motion.drive.test.grid");
    let drive = g.add_node("motion.drive");
    g.connect(Edge {
        from: (grid, 0),
        to: (drive, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, drive, 0.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 0.0], [0.0, 0.0]], "no value → no move"),
        _ => panic!("P"),
    }
}

/// **A CPU e o device escrevem a MESMA coluna, canal por canal.**
///
/// O doc do `variant_by_param` diz, com todas as letras, que ele espelha o `channel_column`
/// *"including its `_ => size` catch-all"* — ou seja: duas tabelas, uma pergunta. Acrescentar
/// um canal a UMA delas não quebra a compilação, não quebra nenhum gate de comportamento, e
/// faz a CPU escrever `falloff` enquanto o device escreve `size`.
///
/// ⚠️ O modo de falha é mudo por construção: as duas rotas produzem um stream bem-formado,
/// de contagem certa, com uma coluna escrita — só que não a mesma. Uma cena que cozinha na
/// GPU e outra que cai na CPU desenhariam coisas diferentes, e nenhuma delas erraria.
///
/// A varredura passa do intervalo válido de propósito: o catch-all é parte da resposta, e um
/// gate que só olhasse os canais nomeados deixaria justamente ele livre para divergir.
#[test]
fn the_cpu_and_the_device_write_the_same_column_for_every_channel() {
    let pick = GPU_KERNEL
        .variant_by_param
        .expect("o drive escolhe variante por param — se deixou de escolher, este gate mente");
    for ch in -2..=8 {
        let cpu = channel::channel_column(ch);
        let gpu = pick(&|_| ch as f32).bindings[0].column;
        assert_eq!(
            cpu, gpu,
            "canal {ch}: a CPU escreve `{cpu}` e o device escreve `{gpu}` — as duas tabelas \
             deixaram de responder a mesma pergunta"
        );
    }
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}
/// **Opacity is a channel** (doc 51): the drive writes the ALPHA of the tint, so a particle
/// can FADE — which is what "fades away" means, and what the library could not do at all.
///
/// An uncoloured stream starts from opaque white, so driving the opacity of a stream nobody
/// tinted does exactly what it says instead of silently doing nothing.
#[test]
fn the_opacity_channel_fades_the_tint_and_starts_from_opaque_white() {
    let plain = Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0]; 2]));
    let out = channel::drive_channel(
        &plain,
        channel::CH_OPACITY,
        &[0.25, 0.75],
        1.0,
        Combine::Set,
    );
    match out.get("tint") {
        Some(Column::Vec4(v)) => {
            assert_eq!(v[0], [1.0, 1.0, 1.0, 0.25], "white, a quarter opaque");
            assert_eq!(v[1][3], 0.75);
        }
        _ => panic!("the opacity drive minted a tint"),
    }

    // Multiply against an existing colour: the hue survives, the alpha bleeds.
    let red = Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("tint", Column::Vec4(vec![[1.0, 0.0, 0.0, 1.0]]));
    let faded = channel::drive_channel(&red, channel::CH_OPACITY, &[0.5], 1.0, Combine::Multiply);
    match faded.get("tint") {
        Some(Column::Vec4(v)) => assert_eq!(v[0], [1.0, 0.0, 0.0, 0.5]),
        _ => panic!("tint"),
    }

    // An alpha the renderer cannot use is not a brighter particle — it is a bug. Clamped.
    let over = channel::drive_channel(&plain, channel::CH_OPACITY, &[4.0, -2.0], 1.0, Combine::Set);
    match over.get("tint") {
        Some(Column::Vec4(v)) => assert_eq!((v[0][3], v[1][3]), (1.0, 0.0)),
        _ => panic!("tint"),
    }
}

/// Um stream de UMA instância com a cor `c`, o mínimo que os gates de cor precisam.
fn tinted(c: [f32; 4]) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("tint", Column::Vec4(vec![c]))
}
fn tint_of(s: &Stream) -> [f32; 4] {
    match s.get("tint") {
        Some(Column::Vec4(v)) => v[0],
        _ => panic!("o drive de cor tem de emitir um tint"),
    }
}

/// **A METADE DE ESCRITA DO LAÇO DE COR** — ver [`channel::CH_HUE`].
///
/// A §0 do doc 89 fam. 9 mediu que *nada no catálogo escrevia R/G/B a partir de um valor*:
/// o único canal de cor era o Opacity, que escreve o ALFA. Estes três dirigem a cor que já
/// está lá, e os modos que a referência usa já existiam — o *Master Hue* do AE é um
/// deslocamento (`Add`), *Saturation*/*Lightness* são escalas (`Multiply`).
///
/// ⚠️ O oráculo é a COR resultante, não o HSV intermediário: um gate que comparasse a
/// tripla que o próprio kernel acabou de computar estaria a espelhar a implementação.
#[test]
fn the_colour_channels_shift_the_colour_that_is_already_there() {
    let red = tinted([1.0, 0.0, 0.0, 0.7]);
    // Matiz + 1/3 de volta: vermelho → verde (a definição de HSV, não o nosso código).
    let hue = tint_of(&channel::drive_channel(
        &red,
        channel::CH_HUE,
        &[1.0 / 3.0],
        1.0,
        Combine::Add,
    ));
    assert!(
        hue[1] > 0.99 && hue[0] < 0.01 && hue[2] < 0.01,
        "matiz +1/3 leva vermelho a verde: {hue:?}"
    );
    assert_eq!(hue[3], 0.7, "o alfa atravessa — quem o dirige é o Opacity");

    // Saturação × 0.5: o vermelho desbota para o meio do caminho até o branco.
    let sat = tint_of(&channel::drive_channel(
        &red,
        channel::CH_SAT,
        &[0.5],
        1.0,
        Combine::Multiply,
    ));
    assert!(
        (sat[0] - 1.0).abs() < 1e-6 && (sat[1] - 0.5).abs() < 1e-6 && (sat[2] - 0.5).abs() < 1e-6,
        "saturação ×0.5 desbota sem mudar o matiz nem o brilho: {sat:?}"
    );

    // Valor × 0.5: o mesmo vermelho, metade do brilho.
    let val = tint_of(&channel::drive_channel(
        &red,
        channel::CH_VAL,
        &[0.5],
        1.0,
        Combine::Multiply,
    ));
    assert!(
        (val[0] - 0.5).abs() < 1e-6 && val[1] < 1e-6 && val[2] < 1e-6,
        "valor ×0.5 escurece sem dessaturar: {val:?}"
    );
}

/// **O NEUTRO É A IDENTIDADE — DENTRO DA IDA-E-VOLTA DO HSV, e o número é MEDIDO.**
///
/// ⚠️ O doc 89 pedia `hue = 0 · sat = 1 · val = 1` como *default que reduz*, e a versão
/// honesta disso **não é byte-identidade**: a conversão RGB→HSV→RGB passa por divisões, e
/// só é exata por sorte aritmética. Este gate mede o pior erro em vez de afirmar exatidão
/// — e nenhuma arte regride por causa disso, porque os canais 6/7/8 não existiam (o picker
/// oferecia 0..5 e `channel_column` mandava tudo acima de 5 para `size`).
#[test]
fn the_neutral_colour_drive_is_the_identity_within_the_hsv_round_trip() {
    let colours = [
        [1.0, 0.0, 0.0, 1.0],
        [0.2, 0.5, 0.8, 0.4],
        [0.0, 0.0, 0.0, 1.0], // preto: saturação indefinida, o degenerado
        [1.0, 1.0, 1.0, 1.0], // branco: idem
        [0.13, 0.87, 0.31, 0.6],
    ];
    let mut worst = 0.0f32;
    for c in colours {
        for (ch, v, mode) in [
            (channel::CH_HUE, 0.0, Combine::Add),
            (channel::CH_SAT, 1.0, Combine::Multiply),
            (channel::CH_VAL, 1.0, Combine::Multiply),
        ] {
            let out = tint_of(&channel::drive_channel(&tinted(c), ch, &[v], 1.0, mode));
            for k in 0..4 {
                worst = worst.max((out[k] - c[k]).abs());
            }
        }
    }
    assert!(
        worst < 1e-6,
        "o drive neutro tem de devolver a cor; pior desvio medido {worst}"
    );
}

/// **O CLAMP DA SATURAÇÃO É ESTRUTURAL, NÃO GOSTO** — e o valor NÃO tem teto, de propósito.
///
/// Com `s > 1` o `hsv_to_rgba` calcula `p = v·(1−s)` e devolve canais **negativos**; um
/// tint negativo não é uma cor mais viva, é lixo que atravessa o compositor. Já um teto no
/// VALOR seria uma regra que os outros produtores desta coluna não obedecem — a
/// interpolação `Cardinal` da rampa passa de 1 por construção, e o doc dela diz isso.
#[test]
fn the_saturation_is_clamped_because_it_must_be_and_the_value_is_not() {
    let half = tinted([1.0, 0.5, 0.5, 1.0]); // saturação 0.5
    let over = tint_of(&channel::drive_channel(
        &half,
        channel::CH_SAT,
        &[3.0],
        1.0,
        Combine::Multiply,
    ));
    assert!(
        over.iter().all(|k| *k >= 0.0),
        "saturação acima de 1 mintaria canais negativos: {over:?}"
    );
    let bright = tint_of(&channel::drive_channel(
        &half,
        channel::CH_VAL,
        &[2.0],
        1.0,
        Combine::Multiply,
    ));
    assert!(
        bright[0] > 1.5,
        "o valor não é capado — a rampa Cardinal já passa de 1: {bright:?}"
    );
    let dark = tint_of(&channel::drive_channel(
        &half,
        channel::CH_VAL,
        &[-1.0],
        1.0,
        Combine::Multiply,
    ));
    assert!(
        dark.iter().all(|k| *k >= 0.0),
        "mas o PISO existe: valor negativo também mintaria canais negativos: {dark:?}"
    );
}

/// **A MÁSCARA ALCANÇA A COR** — o `falloff` pesa o drive de cor como pesa todos os outros,
/// e com `Add` (o default) ele escala o DESLOCAMENTO, que é o que torna o matiz sem
/// ambiguidade de arco.
#[test]
fn the_falloff_masks_the_colour_drive() {
    let masked = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 2]))
        .with("tint", Column::Vec4(vec![[1.0, 0.0, 0.0, 1.0]; 2]))
        .with("falloff", Column::Scalar(vec![0.0, 1.0]));
    let out = channel::drive_channel(&masked, channel::CH_HUE, &[1.0 / 3.0], 1.0, Combine::Add);
    match out.get("tint") {
        Some(Column::Vec4(v)) => {
            assert!(
                v[0][0] > 0.99 && v[0][1] < 1e-6,
                "máscara 0 deixa a cor onde estava: {:?}",
                v[0]
            );
            assert!(
                v[1][1] > 0.99 && v[1][0] < 0.01,
                "máscara 1 leva o deslocamento inteiro: {:?}",
                v[1]
            );
        }
        _ => panic!("tint"),
    }
}
