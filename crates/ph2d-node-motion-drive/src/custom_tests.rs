//! Os gates do **canal CUSTOM** — o escritor de coluna nomeada (grupo P).

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Uma fonte de 4 instâncias com `P` e uma coluna escalar `heat` já lá.
static SRC_C: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.drive.test.custom"),
    name: "motion.drive.test.custom",
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
struct SrcC;
impl NodeOp for SrcC {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_C
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(
            Stream::new(4)
                .with("P", Column::Vec2(vec![[0.0, 0.0]; 4]))
                .with("heat", Column::Scalar(vec![10.0, 20.0, 30.0, 40.0]))
                // ⚠️ A MÁSCARA vive na fixture desde o começo: sem ela
                // `falloff_at` devolve `1.0` e `blend(x, driven, 1)` É `driven`,
                // então uma mutação que apagasse a mistura **sobreviveria** —
                // foi o que aconteceu com a primeira versão destes gates.
                .with("falloff", Column::Scalar(vec![1.0, 0.5, 0.0, 1.0])),
        );
    }
}

/// Uma fonte de VALOR: um número por elemento.
static SRC_V: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.drive.test.vals"),
    name: "motion.drive.test.vals",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame),
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct SrcV;
impl NodeOp for SrcV {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_V
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(4).with("v", Column::Scalar(vec![1.0, 2.0, 3.0, 4.0])));
    }
}

struct OpsC;
impl OpResolver for OpsC {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_C.id => Some(&SrcC),
            t if t == SRC_V.id => Some(&SrcV),
            t if t == MANIFEST.id => Some(&MotionDrive),
            _ => None,
        }
    }
}

fn rig() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let src = g.add_node("motion.drive.test.custom");
    let vals = g.add_node("motion.drive.test.vals");
    let d = g.add_node("motion.drive");
    g.connect(Edge {
        from: (src, 0),
        to: (d, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (vals, 0),
        to: (d, 1),
        delayed: false,
    })
    .unwrap();
    (g, d)
}

fn cook_out(g: &Graph, d: NodeId) -> Stream {
    let mut c = Cook::new();
    c.cook(g, &OpsC, d, 0.0).unwrap()[0].as_stream().clone()
}

fn scalar(s: &Stream, name: &str) -> Option<Vec<f32>> {
    match s.get(name) {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    }
}

/// **ELE ESCREVE A COLUNA QUE O ARTISTA NOMEOU — e uma que não existia.**
#[test]
fn it_writes_the_column_the_artist_named_even_a_brand_new_one() {
    let (mut g, d) = rig();
    g.set_param(d, "channel", CH_CUSTOM as f32);
    g.set_param(d, "mode", 1.0); // Set
    g.set_text_param(d, DRIVE_COL_KEY, "charge");
    let out = cook_out(&g, d);
    // A coluna nasce em ZERO e é misturada pela máscara: `0 + (v − 0)·f`.
    assert_eq!(
        scalar(&out, "charge"),
        Some(vec![1.0, 1.0, 0.0, 4.0]),
        "a coluna nasce com o campo de valor, MASCARADO"
    );
    // ⚠️ O resto do stream ATRAVESSA — um escritor que engolisse `P` seria um
    // nó que apaga o desenho para escrever um número.
    assert!(out.get("P").is_some(), "o `P` atravessa");
    assert_eq!(scalar(&out, "heat"), Some(vec![10.0, 20.0, 30.0, 40.0]));
}

/// **ELE COMBINA COM O QUE JÁ ESTÁ LÁ, PELA MESMA LEI DOS OUTROS CANAIS.**
#[test]
fn it_combines_with_the_existing_column() {
    let (mut g, d) = rig();
    g.set_param(d, "channel", CH_CUSTOM as f32);
    g.set_param(d, "mode", 0.0); // Add
    g.set_text_param(d, DRIVE_COL_KEY, "heat");
    let out = cook_out(&g, d);
    assert_eq!(
        scalar(&out, "heat"),
        Some(vec![11.0, 21.0, 30.0, 44.0]),
        "Add soma ao que já estava lá, pesado pela máscara"
    );
}

/// **O `scale` E O `falloff` VALEM AQUI COMO EM TODO CANAL.**
///
/// ⚠️ É este gate que prova que a lei é UMA: se o escritor nomeado tivesse a sua
/// própria aritmética, ela divergiria no primeiro ajuste da mistura.
#[test]
fn the_scale_and_the_falloff_apply_here_too() {
    let (mut g, d) = rig();
    g.set_param(d, "channel", CH_CUSTOM as f32);
    g.set_param(d, "mode", 1.0); // Set
    g.set_param(d, "scale", 10.0);
    g.set_text_param(d, DRIVE_COL_KEY, "heat");
    let out = cook_out(&g, d);
    assert_eq!(
        scalar(&out, "heat"),
        Some(vec![10.0, 20.0, 30.0, 40.0]),
        "Set com scale 10 põe `v·10` — e aqui coincide com o `heat` de entrada, \
         então a máscara é invisível: é por isso que o gate SEGUINTE existe"
    );
    g.set_param(d, "scale", 1.0);
    let out = cook_out(&g, d);
    // Agora a máscara MORDE: `heat + (v − heat)·f`, com f = 1 · 0,5 · 0 · 1.
    assert_eq!(scalar(&out, "heat"), Some(vec![1.0, 11.0, 30.0, 4.0]));
}

/// **A MÁSCARA ZERA O ELEMENTO, E O ELEMENTO VIZINHO FICA INTEIRO.**
///
/// ⚠️ O gate acima podia passar com a mistura apagada, porque a fixture dele cai
/// num ponto em que os dois lados coincidem. Este pergunta pela diferença.
#[test]
fn a_falloff_of_zero_leaves_the_named_column_untouched() {
    let (mut g, d) = rig();
    g.set_param(d, "channel", CH_CUSTOM as f32);
    g.set_param(d, "mode", 1.0); // Set
    g.set_text_param(d, DRIVE_COL_KEY, "heat");
    let out = scalar(&cook_out(&g, d), "heat").unwrap();
    assert_eq!(out[2], 30.0, "falloff 0 deixa o `heat` de entrada intacto");
    assert_eq!(out[3], 4.0, "e o vizinho de falloff 1 recebe o valor cheio");
    assert_eq!(out[1], 11.0, "e o de meia máscara fica no meio do caminho");
}

/// **UM NOME VAZIO NÃO CUNHA COLUNA NENHUMA** — a entrada atravessa verbatim.
#[test]
fn an_empty_name_writes_nothing() {
    let (mut g, d) = rig();
    g.set_param(d, "channel", CH_CUSTOM as f32);
    let out = cook_out(&g, d);
    assert!(
        out.get("").is_none(),
        "uma coluna de nome vazio é uma que ninguém consegue nomear de volta"
    );
    assert_eq!(scalar(&out, "heat"), Some(vec![10.0, 20.0, 30.0, 40.0]));
}

/// **ELE RECUSA MUDAR O TIPO DE UMA COLUNA QUE JÁ EXISTE.**
///
/// ⚠️ A pergunta é feita ao STREAM, não a uma lista de nomes proibidos: `P` é
/// `Vec2`, e escrever um `Scalar` por cima faria todo leitor a jusante ler outra
/// coisa. Uma lista apodreceria na décima convenção de stream — esta casa já tem
/// `texture_id`, `geometry_id`, `uv_rect` e `nrm`.
#[test]
fn it_refuses_to_change_the_type_of_an_existing_column() {
    let (mut g, d) = rig();
    g.set_param(d, "channel", CH_CUSTOM as f32);
    g.set_param(d, "mode", 1.0);
    g.set_text_param(d, DRIVE_COL_KEY, "P");
    let out = cook_out(&g, d);
    assert!(
        matches!(out.get("P"), Some(Column::Vec2(_))),
        "o `P` continua a ser `Vec2` — a recusa devolve a entrada verbatim"
    );
}

/// **OS NOVE CANAIS DO ENUM NÃO SE MEXEM** — o novo é o ÚLTIMO índice, e o
/// `channel` é um param que o documento GUARDA.
#[test]
fn the_custom_channel_is_the_last_index_and_the_others_did_not_move() {
    assert_eq!(
        CH_CUSTOM, 9,
        "renumerar re-aponta todo grafo salvo, em silêncio"
    );
    let (mut g, d) = rig();
    for (ch, col) in [(0.0, "P"), (2.0, "rot"), (3.0, "size")] {
        g.set_param(d, "channel", ch);
        let out = cook_out(&g, d);
        assert!(out.get(col).is_some(), "o canal {ch} ainda escreve `{col}`");
    }
}

/// **O CUSTOM RECUSA O DEVICE, E OS OUTROS NÃO.**
///
/// As duas metades: sem a segunda, *"ele recusa"* seria satisfeito por um nó que
/// desistisse do device inteiro.
#[test]
fn the_custom_channel_refuses_the_device_and_the_others_keep_it() {
    let f = GPU_KERNEL.applicable.expect("o nó declara a recusa");
    let ch = |v: f32| move |name: &str| if name == "channel" { v } else { 0.0 };
    assert!(!f(&ch(CH_CUSTOM as f32)), "o Custom recua para a CPU");
    for other in [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0] {
        assert!(f(&ch(other)), "o canal {other} continua no device");
    }
}

/// Uma fonte que CARREGA as colunas de escrituracao — sem elas a recusa seria
/// verde por vacuo (nao ha' o que preservar).
fn source_with_bookkeeping() -> Stream {
    Stream::new(4)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 4]))
        .with("heat", Column::Scalar(vec![10.0, 20.0, 30.0, 40.0]))
        .with("id", Column::Scalar(vec![1.0, 2.0, 3.0, 4.0]))
        .with("sim_t", Column::Scalar(vec![0.5; 4]))
        .with("sim_d", Column::Scalar(vec![0.25; 4]))
        .with("dl_1", Column::Scalar(vec![7.0; 4]))
}

/// **O CANAL NOMEADO RECUSA AS COLUNAS DE ESCRITURACAO.**
///
/// A recusa por TIPO nao as cobre, e e' esse o ponto: `id` e `sim_t` sao
/// `Column::Scalar`, exactamente a forma que o canal aceita — o que os separa e'
/// o PAPEL. Um `sim_t` sobrescrito faz `dt = (playhead - sim_t).clamp(0, MAX)`
/// devolver zero e a simulacao CONGELA, sem erro e sem badge; um `id`
/// sobrescrito faz o pareamento por identidade entregar a linha de historia ao
/// elemento errado. Os dois modos de falha sao MUDOS.
///
/// E o picker do painel OFERECE `id` ao artista (ele e' um nome legitimo de
/// LEITURA), entao o artista o aprende ali e depois o digita aqui: o campo do
/// nome e' texto livre, sem lista.
#[test]
fn the_named_channel_refuses_the_bookkeeping_columns() {
    for name in ["id", "sim_t", "sim_d", "dl_1"] {
        let src = source_with_bookkeeping();
        let Some(Column::Scalar(before)) = src.get(name).cloned() else {
            panic!("a fixture tem de CONTER a coluna, senao a recusa e' verde por vacuo");
        };
        let out = drive_named(&src, name, &[9.0, 9.0, 9.0, 9.0], 1.0, Combine::Set);
        let Some(Column::Scalar(after)) = out.get(name).cloned() else {
            panic!("a recusa devolve a entrada verbatim, com a coluna intacta");
        };
        for (i, (a, b)) in before.iter().zip(after.iter()).enumerate() {
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "`{name}` e' escrituracao e nao pode ser escrita (elemento {i})"
            );
        }
    }
}

/// **E O CONTROLE: um nome ORDINARIO da MESMA forma continua a ser escrito.**
///
/// Sem esta metade, *"a recusa funciona"* seria satisfeito por o canal ter
/// parado de escrever seja onde for.
#[test]
fn an_ordinary_scalar_of_the_same_shape_is_still_written() {
    let src = source_with_bookkeeping();
    let out = drive_named(&src, "heat", &[9.0, 9.0, 9.0, 9.0], 1.0, Combine::Set);
    let Some(Column::Scalar(v)) = out.get("heat").cloned() else {
        panic!("o canal escreve a coluna nomeada");
    };
    assert!(
        v.iter().all(|x| (*x - 9.0).abs() < 1e-6),
        "um nome ordinario continua escrevivel: {v:?}"
    );
}
