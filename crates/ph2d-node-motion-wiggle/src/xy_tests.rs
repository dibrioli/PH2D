//! Os gates do canal **`Position XY`** — o `Separate Channels` da folha 06.
//!
//! ## A régua é a CORRELAÇÃO, e não "houve deslocamento"
//!
//! O modo de falha desta feature não é ela não mexer — é ela mexer **igual nos dois
//! eixos**. O mesmo campo em X e em Y põe cada peça a oscilar sobre uma **diagonal a
//! 45°**: um segmento, não um vaguear. Isso passa em qualquer gate que meça excursão
//! ou contagem de linhas. ⇒ o oráculo é o coeficiente de correlação entre os dois
//! eixos, com o **controle** no deslocamento zero (que tem de dar `1` exacto).
//!
//! ⚠️ **E este arquivo NÃO é a cópia do irmão `motion.noise`**, apesar da lei ser a
//! mesma: aquele é o nó de CAMPO e amostra `P`, este é o de ÍNDICE e amostra a linha
//! `i + seed` (a cerca 5 da folha). A fixture certa aqui é uma FILEIRA na origem — um
//! bloco espacial não mediria nada que este nó leia —, e o deslocamento é de LINHA e
//! **fracionário**, não de seed inteiro. *Copiar o irmão daria um gate verde sobre a
//! fixture errada.*

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId as GNodeId};

/// A barra da decorrelação. ⚠️ Ela fica longe de **1** (o valor do defeito) e não
/// colada em **0**: o resíduo é erro de amostragem, não acoplamento — o irmão
/// `motion.noise` mede-o a cair de `0,120` para `0,009` só ao afinar o campo. Aqui,
/// num nó de ÍNDICE (sem coerência espacial), ele mede `0,079`.
const MAX_R: f32 = 0.5;

/// Quantas peças — grande o bastante para uma correlação querer dizer alguma coisa.
const COUNT: usize = 512;

/// Uma fileira na ORIGEM: este nó lê o índice, não a posição, então o que a fixture
/// tem de trazer é CONTAGEM, e o deslocamento é a saída inteira.
static ROW: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.wiggle.test.xyrow"),
    name: "motion.wiggle.test.xyrow",
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
struct Row;
impl NodeOp for Row {
    fn manifest(&self) -> &'static NodeManifest {
        &ROW
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(COUNT).with("P", Column::Vec2(vec![[0.0, 0.0]; COUNT])));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == ROW.id => Some(&Row),
            t if t == MANIFEST.id => {
                static W: MotionWiggle = MotionWiggle;
                Some(&W)
            }
            _ => None,
        }
    }
}

/// As posições cozidas. A fonte está na origem, então elas JÁ SÃO o deslocamento.
fn cooked(setup: impl FnOnce(&mut Graph, GNodeId)) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let src = g.add_node("motion.wiggle.test.xyrow");
    let wg = g.add_node("motion.wiggle");
    g.connect(Edge {
        from: (src, 0),
        to: (wg, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(wg, "amplitude", 1.0);
    setup(&mut g, wg);
    let mut cook = Cook::new();
    match cook.cook(&g, &Ops, wg, 0.5).unwrap()[0]
        .as_stream()
        .get("P")
        .unwrap()
    {
        Column::Vec2(v) => v.clone(),
        _ => panic!("P"),
    }
}

fn axis_deltas(setup: impl FnOnce(&mut Graph, GNodeId)) -> (Vec<f32>, Vec<f32>) {
    let p = cooked(setup);
    (
        p.iter().map(|q| q[0]).collect(),
        p.iter().map(|q| q[1]).collect(),
    )
}

/// O coeficiente de correlação de Pearson. `1` = os dois eixos são o mesmo campo.
fn pearson(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len() as f32;
    let ma = a.iter().sum::<f32>() / n;
    let mb = b.iter().sum::<f32>() / n;
    let mut cov = 0.0;
    let mut va = 0.0;
    let mut vb = 0.0;
    for (x, y) in a.iter().zip(b) {
        cov += (x - ma) * (y - mb);
        va += (x - ma) * (x - ma);
        vb += (y - mb) * (y - mb);
    }
    if va <= 0.0 || vb <= 0.0 {
        return 0.0;
    }
    cov / (va * vb).sqrt()
}

fn swing(v: &[f32]) -> f32 {
    v.iter().fold(f32::NEG_INFINITY, |m, x| m.max(*x))
        - v.iter().fold(f32::INFINITY, |m, x| m.min(*x))
}

/// **OS DOIS EIXOS ANDAM, E SÃO CAMPOS DIFERENTES.**
///
/// ⚠️ As duas metades são precisas. Sem a primeira, um canal que escrevesse zero num
/// dos eixos passaria na correlação (um campo constante correlaciona-se com nada).
/// Sem a segunda, o mesmo campo nos dois eixos passaria em tudo — e é o defeito real.
#[test]
fn the_xy_channel_writes_both_axes_and_they_are_decorrelated() {
    let (dx, dy) = axis_deltas(|g, wg| g.set_param(wg, "channel", CH_XY as f32));
    assert!(swing(&dx) > 0.5, "o eixo X anda: {}", swing(&dx));
    assert!(swing(&dy) > 0.5, "o eixo Y anda: {}", swing(&dy));
    let r = pearson(&dx, &dy);
    assert!(
        r.abs() < MAX_R,
        "os dois eixos tem de ser campos DIFERENTES, correlacao {r}"
    );
}

/// **O CONTROLE: com deslocamento ZERO os dois eixos seriam o MESMO campo.**
///
/// É esta metade que impede o gate acima de passar por vácuo — ela mede o que o
/// produto seria **sem** o [`AXIS_ROW_OFFSET`], lendo o mesmo campo pelos dois canais
/// de um eixo só.
#[test]
fn without_the_offset_the_two_axes_would_be_one_field_at_45_degrees() {
    let (dx, _) = axis_deltas(|g, wg| g.set_param(wg, "channel", 0.0)); // X
    let (_, dy) = axis_deltas(|g, wg| g.set_param(wg, "channel", 1.0)); // Y
    let r = pearson(&dx, &dy);
    assert!(
        (r - 1.0).abs() < 1e-4,
        "a mesma linha nos dois eixos e' UM campo (r = {r}) — e' isso que o deslocamento evita"
    );
}

/// **O DESLOCAMENTO É FRACIONÁRIO, E ISSO É UMA AFIRMAÇÃO SOBRE A CONTAGEM.**
///
/// ⚠️ Aqui a linha é uma coordenada CONTÍNUA (`i + seed`), então um deslocamento
/// INTEIRO `K` faria o eixo Y da peça `i` ser exactamente o eixo X da peça `i + K` —
/// invisível numa fileira curta e real assim que ela passasse de `K` elementos. Com
/// `,5` a igualdade `i + K = j` não tem solução inteira **para contagem nenhuma**.
///
/// O gate mede a propriedade, não o número: qualquer valor que alguém escolha tem de
/// continuar a não ser inteiro.
#[test]
fn the_row_offset_can_never_coincide_with_another_elements_row() {
    let frac = AXIS_ROW_OFFSET - AXIS_ROW_OFFSET.floor();
    assert!(
        frac > 1e-3 && frac < 1.0 - 1e-3,
        "o deslocamento de linha tem de ser FRACIONARIO, deu {AXIS_ROW_OFFSET}"
    );
    // E ele é grande: mesmo a parte inteira fica muito além de qualquer fileira que
    // um artista monte, o que mantém as duas leituras longe uma da outra no campo.
    const {
        assert!(AXIS_ROW_OFFSET > 1000.0);
    }
}

/// **OS QUATRO CANAIS DE SEMPRE FICAM BYTE-IDÊNTICOS.**
#[test]
fn the_four_original_channels_are_untouched_by_the_new_one() {
    for ch in 0..=3 {
        let p = cooked(|g, wg| g.set_param(wg, "channel", ch as f32));
        let moved_x = p.iter().any(|q| q[0] != 0.0);
        let moved_y = p.iter().any(|q| q[1] != 0.0);
        assert_eq!(moved_x, ch == 0, "canal {ch}: X");
        assert_eq!(moved_y, ch == 1, "canal {ch}: Y");
    }
}

/// **O WGSL CARREGA O MESMO DESLOCAMENTO QUE O RUST** — o gémeo literal.
///
/// ⚠️ O corpo do kernel é uma **string** e não vê a const do Rust: dois sítios a dizer
/// o mesmo número, e nada a obrigá-los senão este gate, que deriva a agulha da const.
#[test]
fn the_wgsl_carries_the_same_axis_offset_as_the_rust() {
    let needle = format!(", {AXIS_ROW_OFFSET})");
    assert!(
        gpu::pxy_wgsl().contains(&needle),
        "o WGSL do variant XY tem de deslocar por {AXIS_ROW_OFFSET}: {}",
        gpu::pxy_wgsl()
    );
    // O controle: a agulha é específica.
    assert!(!gpu::pxy_wgsl().contains(", 7919.4)"));
}

/// **O PAINEL OFERECE O CANAL NOVO** — a costura entre a lei e o dropdown.
#[test]
fn the_channel_dropdown_offers_the_two_axis_entry() {
    let row = PARAM_HINTS
        .iter()
        .find(|h| h.param == "channel")
        .expect("a linha do canal");
    let ParamWidget::Enum { labels } = row.widget else {
        panic!("o canal é um seletor nomeado")
    };
    assert_eq!(labels.len() as i32 - 1, CH_XY, "a ultima etiqueta é o XY");
    assert_eq!(labels[CH_XY as usize], "Position XY");
    assert!((row.max - CH_XY as f32).abs() < f32::EPSILON);
}
