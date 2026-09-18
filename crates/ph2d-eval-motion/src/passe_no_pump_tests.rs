//! ⭐⭐⭐ **O FIO, e não a porta** — os gates do passe automático de separação **dentro do pump**
//! (doc 115 W5).
//!
//! ⚠️⚠️ **Porque eles existem além dos da `ph2d-contact`:** aquela crate tem dez gates sobre a LEI,
//! e todos chamam a porta directamente. *Um gate que chama a função em vez de percorrer a rota
//! afirma que a lei existe, nunca que o produto a usa* — foi uma mutação sobrevivente que ensinou
//! isto a esta casa, e a cerca da W1 já leva o irmão deste. Aqui a rota REAL é percorrível sem
//! device nenhum: o pump da CPU é headless, e o que se mede são as **instâncias que o renderer
//! recebe**.

use super::*;
use ph2d_nodegraph::attr::COLLIDER_BOX_COLUMN;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Duas peças de lado `1` com metade de cada uma dentro da outra, cada uma a DECLARAR a caixa dela
/// pela convenção do objecto (geometria mais o `size` que a converte — doc 115 §12.1).
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.test.duas_sobrepostas"),
    name: "motion.test.duas_sobrepostas",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // Os dois params do sink que esta wave acrescenta — declarados aqui para o nó de teste poder
    // ser ele próprio o sink, que é como o pump os lê.
    params: &[
        ParamSpec {
            name: SINK_COLLIDE_PARAM,
            default: 0.0,
        },
        ParamSpec {
            name: SINK_COLLIDE_ITERATIONS_PARAM,
            default: SINK_COLLIDE_ITERATIONS_DEFAULT,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(
            Stream::new(2)
                .with("P", Column::Vec2(vec![[0.0, 0.0], [0.5, 0.0]]))
                .with("size", Column::Vec2(vec![[1.0, 1.0]; 2]))
                .with(COLLIDER_BOX_COLUMN, Column::Vec2(vec![[0.5, 0.5]; 2])),
        );
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == SRC_MAN.id).then_some(&Src as &dyn NodeOp)
    }
}

/// Corre um quadro e devolve as posições que o renderer vai desenhar.
fn desenhado(armado: Option<f32>) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let sink = g.add_node(SRC_MAN.name);
    if let Some(varreduras) = armado {
        g.set_param(sink, SINK_COLLIDE_PARAM, 1.0);
        g.set_param(sink, SINK_COLLIDE_ITERATIONS_PARAM, varreduras);
    }
    let mut pump = MotionCookPump::new();
    assert!(pump.pump(&g, &Ops, &[sink], 0, 0.0, [0.0, 0.0, 1.0, 1.0], [1.0, 1.0]));
    pump.instances.iter().map(|i| i.world_pos).collect()
}

/// ⛔⛔ **DESARMADO, o quadro é o de sempre — AO BIT.** É esta metade que torna honesta a frase
/// *«esta wave não muda um pixel de nenhuma cena que já existe»*: a corrente que o cozimento
/// devolveu chega ao lowering sem passar por lado nenhum.
#[test]
fn desarmado_o_pump_desenha_o_que_o_cozimento_deu() {
    assert_eq!(desenhado(None), vec![[0.0, 0.0], [0.5, 0.0]]);
}

/// ⭐⭐⭐ **ARMADO, as peças chegam ao renderer SEPARADAS** — a rota inteira: o cozimento, o passe, o
/// lowering, e as instâncias que o desenho recebe.
#[test]
fn armado_o_pump_desenha_as_pecas_separadas() {
    let p = desenhado(Some(32.0));
    let vao = (p[1][0] - p[0][0]).abs();
    assert!(
        (vao - 1.0).abs() < 1e-3,
        "duas caixas de lado 1 tem de chegar ao renderer a 1 de distancia: {p:?}"
    );
}

/// ⚠️ **O interruptor manda, e o número é só o quanto** — com o `collide` desligado, pôr as
/// varreduras no máximo continua a não separar nada.
///
/// ⛔ Sem esta metade, um `sink_collide_sweeps` que devolvesse as varreduras sem olhar ao
/// interruptor passaria nos dois gates acima.
#[test]
fn o_numero_de_varreduras_nao_arma_nada_sozinho() {
    let mut g = Graph::new();
    let sink = g.add_node(SRC_MAN.name);
    g.set_param(sink, SINK_COLLIDE_ITERATIONS_PARAM, 64.0);
    assert_eq!(
        sink_collide_sweeps(&g, sink),
        0,
        "as varreduras sem o interruptor sao zero"
    );
}

/// ⭐⭐⭐ **O DEFAULT não vem do manifesto, e este gate é quem o afirma** (ver
/// [`sink_collide_sweeps`]).
///
/// O leitor de params deste módulo devolve `0.0` quando não há **override** — ele nunca consulta o
/// `ParamSpec::default`. Os quatro params antigos do sink têm todos default `0`, logo ninguém tinha
/// reparado; este é o primeiro com default `≠ 0`, e lido pela porta de sempre ele valeria `0` num
/// documento acabado de criar ⇒ *o artista ligava o interruptor e não acontecia nada*.
#[test]
fn armar_sem_tocar_no_numero_corre_as_varreduras_de_fabrica() {
    let mut g = Graph::new();
    let sink = g.add_node(SRC_MAN.name);
    g.set_param(sink, SINK_COLLIDE_PARAM, 1.0);
    assert_eq!(
        sink_collide_sweeps(&g, sink),
        SINK_COLLIDE_ITERATIONS_DEFAULT as usize,
        "sem override, as varreduras sao as de FABRICA e nunca zero"
    );
    // E a rota inteira: armar sem tocar no número separa de facto.
    let mut pump = MotionCookPump::new();
    assert!(pump.pump(&g, &Ops, &[sink], 0, 0.0, [0.0, 0.0, 1.0, 1.0], [1.0, 1.0]));
    let p: Vec<[f32; 2]> = pump.instances.iter().map(|i| i.world_pos).collect();
    assert!(
        (p[1][0] - p[0][0]).abs() > 0.9,
        "armar e nao tocar no numero tem de separar: {p:?}"
    );
}

/// ⚠️ **O número é COAGIDO na porta, nos dois extremos** — um knob arrastado além do tecto não pode
/// pôr o passe a varrer mil vezes num quadro (o recurso é o relógio: doc 115 §9.4).
///
/// ⛔⛔ **O caso `NaN` NÃO está aqui, e a razão é do substrato:** o `Graph::set_param` tem um
/// `debug_assert!(value.is_finite())`, logo um teste — que corre em debug — **rebenta na porta**
/// antes de chegar à leitura. A guarda do [`sink_collide_sweeps`] continua a valer e não é morta:
/// em `--release` aquele `assert` desaparece, e o doc do próprio `set_param` diz que ele *«guarda o
/// caminho em código»* enquanto o parser textual guarda o do ficheiro. ⇒ ela espelha o que o
/// [`param`] deste mesmo módulo já fazia pelos outros quatro params do sink, e o que a torna
/// inexercitável por um gate é a porta a montante ser estrita **em debug**, não a guarda ser inútil.
#[test]
fn as_varreduras_sao_coagidas_na_porta() {
    let mut g = Graph::new();
    let sink = g.add_node(SRC_MAN.name);
    g.set_param(sink, SINK_COLLIDE_PARAM, 1.0);
    for (pedido, esperado) in [
        (1000.0, SINK_COLLIDE_ITERATIONS_MAX as usize),
        (0.0, 1),
        (-5.0, 1),
        (16.0, 16),
    ] {
        g.set_param(sink, SINK_COLLIDE_ITERATIONS_PARAM, pedido);
        assert_eq!(
            sink_collide_sweeps(&g, sink),
            esperado,
            "pedido {pedido} tem de ler {esperado}"
        );
    }
}
