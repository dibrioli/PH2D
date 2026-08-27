//! Os gates do **`Seed Per Element`** — o *Use Layer as Seed* da folha 06 (célula 24).
//!
//! ⚠️ Arquivo próprio por TETO DE LOC (HR-18), como o `xy_tests.rs` ao lado.
//!
//! ## A fixtura tem de sair do PONTO DE REDE
//!
//! O defeito que este modo cura é *duas peças no mesmo sítio lêem o mesmo número*, então
//! a fixtura empilha as peças. ⚠️ **Empilhá-las na ORIGEM não serve, e a 1.ª versão da
//! sonda caiu nisso:** a origem é um ponto de rede do ruído de gradiente, onde ele vale
//! **zero para todo seed, por construção** — as duas metades saíam `0,000000` e a
//! leitura *"elas partilham o campo"* ficava certa pelo motivo errado. A pilha vive num
//! ponto qualquer fora da rede.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId as GNodeId};

/// Quantas peças empilhar.
const N: usize = 8;
/// O sítio da pilha — **fora da rede** de propósito (ver o doc do módulo).
const SPOT: [f32; 2] = [0.37, 0.21];

/// Uma fonte que põe as `N` peças no MESMO ponto, com um `id` opcional.
static STACK: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.noise.test.stack"),
    name: "motion.noise.test.stack",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // `0` sem coluna `id` · `1` com `id` TODOS IGUAIS · `2` com `id` distintos.
    params: &[ParamSpec {
        name: "ids",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

struct Stack;
impl NodeOp for Stack {
    fn manifest(&self) -> &'static NodeManifest {
        &STACK
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mut s = Stream::new(N).with("P", Column::Vec2(vec![SPOT; N]));
        #[expect(clippy::cast_precision_loss, reason = "N e' 8")]
        match ctx.param("ids").round() as i32 {
            1 => s = s.with("id", Column::Scalar(vec![7.0; N])),
            2 => s = s.with("id", Column::Scalar((0..N).map(|i| i as f32).collect())),
            _ => {}
        }
        ctx.emit(s);
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == STACK.id => Some(&Stack),
            t if t == MANIFEST.id => {
                static N: MotionNoise = MotionNoise;
                Some(&N)
            }
            _ => None,
        }
    }
}

/// As posições da pilha depois do ruído.
fn stacked(ids: f32, setup: impl FnOnce(&mut Graph, GNodeId)) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let src = g.add_node("motion.noise.test.stack");
    g.set_param(src, "ids", ids);
    let ns = g.add_node("motion.noise");
    g.connect(Edge {
        from: (src, 0),
        to: (ns, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(ns, "channel", CH_XY as f32);
    g.set_param(ns, "amplitude", 1.0);
    g.set_param(ns, "scale", 1.0);
    setup(&mut g, ns);
    let mut cook = Cook::new();
    match cook.cook(&g, &Ops, ns, 0.5).unwrap()[0]
        .as_stream()
        .get("P")
        .unwrap()
    {
        Column::Vec2(v) => v.clone(),
        _ => panic!("P"),
    }
}

/// A maior distância entre duas peças — zero é «moveram-se como uma».
fn spread(p: &[[f32; 2]]) -> f32 {
    let mut m = 0.0f32;
    for a in p {
        for b in p {
            m = m.max(((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt());
        }
    }
    m
}

/// **O CONTROLE — o defeito, exacto.** Sem o modo, oito peças no mesmo ponto recebem o
/// MESMO deslocamento: elas movem-se como uma só, e nenhum knob antigo as separava (o
/// `seed` é param do nó, não coluna — dois nós dão dois campos, e as oito partilham
/// cada um deles).
#[test]
fn stacked_pieces_move_as_one_without_the_per_element_seed() {
    let p = stacked(0.0, |_, _| {});
    assert_eq!(spread(&p), 0.0, "o defeito e' EXACTO: {p:?}");
    // E a fixtura CONTÉM o fenómeno — a pilha saiu da origem, senão o campo valeria
    // zero para todo seed e este gate passaria sobre nada.
    assert!(
        (p[0][0] - SPOT[0]).abs() > 1e-6 || (p[0][1] - SPOT[1]).abs() > 1e-6,
        "o campo tem de mover a pilha: {:?} vs {SPOT:?}",
        p[0]
    );
}

/// ⭐ **A ENTREGA.** Com o modo ligado as mesmas oito peças separam-se.
#[test]
fn the_per_element_seed_separates_pieces_at_the_same_spot() {
    let p = stacked(0.0, |g, ns| g.set_param(ns, "own_field", 1.0));
    assert!(
        spread(&p) > 0.1,
        "as pecas tinham de se separar: envergadura {}",
        spread(&p)
    );
    // E TODAS distintas, não só duas — um par a divergir passaria a barra acima.
    for i in 0..N {
        for j in (i + 1)..N {
            assert!(
                p[i] != p[j],
                "as pecas {i} e {j} continuam coladas: {:?}",
                p[i]
            );
        }
    }
}

/// **DESLIGADO É O NÓ DE SEMPRE, AO BIT.** Escrever `0` no param tem de dar exactamente
/// o mesmo campo que não o escrever — o deslocamento é zero e somar zero a um `i32` é a
/// identidade.
#[test]
fn the_mode_off_is_byte_identical_to_the_node_that_shipped() {
    assert_eq!(
        stacked(0.0, |_, _| {}),
        stacked(0.0, |g, ns| g.set_param(ns, "own_field", 0.0)),
        "o param desligado nao pode mover um bit"
    );
}

/// ⭐⭐ **A CHAVE É LIDA DE FACTO, e é a `id` quando ela existe.** O par é o que prova:
/// com `id` TODOS IGUAIS as peças voltam a colar (elas *são* a mesma identidade), e com
/// `id` distintos separam-se. Sem esta metade, «lê o `id`» seria indistinguível de
/// «ignora o `id` e usa sempre o índice».
#[test]
fn the_identity_column_is_what_the_seed_is_keyed_on() {
    let same = stacked(1.0, |g, ns| g.set_param(ns, "own_field", 1.0));
    assert_eq!(
        spread(&same),
        0.0,
        "oito pecas com a MESMA identidade partilham o campo: {same:?}"
    );
    let distinct = stacked(2.0, |g, ns| g.set_param(ns, "own_field", 1.0));
    assert!(
        spread(&distinct) > 0.1,
        "identidades distintas separam: {}",
        spread(&distinct)
    );
}

/// **A COLUNA AUSENTE CAI NA POSIÇÃO, NUNCA EM ZERO.** Um zero materializado daria a
/// toda a fila o mesmo campo — o próprio defeito, de volta e em silêncio. A prova é que
/// a queda coincide com a identidade `0..N`, que é o que o índice vale.
#[test]
fn an_absent_identity_falls_on_the_position_not_on_zero() {
    assert_eq!(
        stacked(0.0, |g, ns| g.set_param(ns, "own_field", 1.0)),
        stacked(2.0, |g, ns| g.set_param(ns, "own_field", 1.0)),
        "sem `id`, a chave tem de ser a POSICAO -- e `id = 0..N` e' a posicao"
    );
}

/// ⚠️ **O WGSL é uma string e não vê consts do Rust** — o mesmo motivo do gate irmão do
/// `AXIS_SEED_OFFSET`. Quem os mantém iguais é isto, mais a paridade CPU×GPU no device.
#[test]
fn the_wgsl_carries_the_same_element_stride_as_the_rust() {
    let lib = kernel::NS_LIB;
    let needle = format!("key * {ELEMENT_SEED_STRIDE}");
    assert!(
        lib.contains(&needle),
        "o WGSL tem de multiplicar a chave por {ELEMENT_SEED_STRIDE}"
    );
    // O controle: a agulha é específica.
    assert!(!lib.contains("key * 15012"));
    // E o kernel declara o param — sem esta linha ele lê um `params.layer_seed`
    // inexistente, ou calcula o ruído ANTIGO em silêncio enquanto a CPU calcula o novo.
    assert!(
        kernel::NS_PARAMS.contains(&"own_field"),
        "o param tem de estar na lista do kernel"
    );
}

/// **O PASSO NÃO SE TOCA COM A PILHA DE UM ELEMENTO.** Um elemento inteiro ocupa
/// `oitavas (7091) + o segundo eixo (7919) = 15010`, e é isso que obriga o passo a ser
/// maior — senão dois elementos partilhariam uma oitava, em silêncio.
#[test]
fn the_element_stride_clears_a_whole_element() {
    let span = AXIS_SEED_OFFSET + 1013 * (MAX_OCTAVES as i32 - 1);
    assert!(
        ELEMENT_SEED_STRIDE > span,
        "o passo {ELEMENT_SEED_STRIDE} tem de passar o vao de um elemento ({span})"
    );
    // E é ÍMPAR: a multiplicação por um ímpar é bijecção módulo 2³², então chaves
    // distintas dão produtos distintos mesmo com o wrap.
    assert_eq!(ELEMENT_SEED_STRIDE % 2, 1, "o passo tem de ser impar");
}

/// **O PAINEL OFERECE O MODO** — a costura entre a lei e o interruptor, e ele mora
/// colado ao `Seed` porque a pergunta que responde é sobre ele.
#[test]
fn the_panel_offers_the_per_element_seed_next_to_the_seed() {
    let row = PARAM_HINTS
        .iter()
        .find(|h| h.param == "own_field")
        .expect("a linha do modo");
    assert!(matches!(
        row.widget,
        ph2d_node_registry::ParamWidget::Toggle
    ));
    let idx = |p: &str| PARAM_HINTS.iter().position(|h| h.param == p).unwrap();
    assert_eq!(
        idx("own_field"),
        idx("seed") + 1,
        "o interruptor mora colado ao numero de que fala"
    );
}
