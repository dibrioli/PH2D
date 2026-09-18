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
        // ⚠️⚠️ **A PREMISSA DESTA CÉLULA MORREU, e a morte fica visível no diff** (doc 115 §18):
        // ela era `(1000.0, MAX)` — com o tecto herdado em `64`, mil estava ACIMA dele. O tecto
        // subiu por MEDIÇÃO para `4096`, logo `1000` passou a ser um pedido LEGÍTIMO, e a célula
        // deixou de medir a coacção para medir a passagem. *Um gate cuja premissa expira e que
        // ninguém reescreve passa a afirmar o contrário do que o nome dele diz.*
        (1000.0, 1000),
        (
            SINK_COLLIDE_ITERATIONS_MAX + 1.0,
            SINK_COLLIDE_ITERATIONS_MAX as usize,
        ),
        (1e9, SINK_COLLIDE_ITERATIONS_MAX as usize),
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

// ─────────────────────────────────────────────────────────────────────────────
// doc 115 §16 — A TOMADA VÊ O QUE SE DESENHA (report do dono, foto de 2026-09-18)
// ─────────────────────────────────────────────────────────────────────────────

/// Corre um quadro com uma TOMADA no sink e devolve `(o que se desenha, o que a tomada viu)`.
fn desenhado_e_tomada(armado: Option<f32>) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let mut g = Graph::new();
    let sink = g.add_node(SRC_MAN.name);
    if let Some(varreduras) = armado {
        g.set_param(sink, SINK_COLLIDE_PARAM, 1.0);
        g.set_param(sink, SINK_COLLIDE_ITERATIONS_PARAM, varreduras);
    }
    let mut pump = MotionCookPump::new();
    pump.set_taps(&[sink]);
    assert!(pump.pump(&g, &Ops, &[sink], 0, 0.0, [0.0, 0.0, 1.0, 1.0], [1.0, 1.0]));
    let desenho = pump.instances.iter().map(|i| i.world_pos).collect();
    let tomada = pump
        .tap_streams()
        .iter()
        .find(|(n, _)| *n == sink)
        .map(|(_, s)| match s.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!("a tomada tem de trazer P"),
        })
        .expect("a tomada do sink existe");
    (desenho, tomada)
}

/// ⭐⭐⭐ **A TOMADA NUM SINK VÊ O QUE O SINK DESENHA — o report do dono, com foto.**
///
/// Ele escreveu: *«A colisão está correta e não se observa interpenetração entre as formas. Mas o
/// gizmo do collider não está correto e se separa de sua shape e interpenetra.»*
///
/// ⛔⛔ **E era exactamente isso:** o passe reescreve o `P` no fim do cozimento, e a tomada — de que
/// o gizmo do colisor vive — cozinhava por conta própria e guardava a corrente **CRUA**. As formas
/// saíam nas posições de DEPOIS e o contorno azul nas de ANTES, sobrepostas umas às outras.
///
/// ⚠️ **A régua é a IGUALDADE entre as duas**, e não *«a tomada está separada»*: o que o artista vê
/// é a DISCORDÂNCIA, e um gate que medisse só uma delas passaria no dia em que as duas ficassem
/// erradas juntas.
#[test]
fn a_tomada_num_sink_ve_o_que_o_sink_desenha() {
    let (desenho, tomada) = desenhado_e_tomada(Some(32.0));
    assert_eq!(
        tomada, desenho,
        "o gizmo le' a TOMADA e o renderer le' o DESENHO — se elas discordarem, o contorno do \
         colisor aparece separado da forma (o report do dono de 18/09)"
    );
    // ⭐ CONTROLO: a fixtura tem de conter o fenómeno. Sem isto, duas listas iguais e AMBAS por
    // separar passariam — que é precisamente o estado de antes desta cura.
    let vao = (desenho[1][0] - desenho[0][0]).abs();
    assert!(
        (vao - 1.0).abs() < 1e-3,
        "controlo: o quadro medido tem de estar SEPARADO, senao a igualdade nao diz nada: {desenho:?}"
    );
}

/// ⚠️ **E DESARMADO as duas continuam a concordar, no estado NÃO separado** — a metade que impede
/// a cura de virar *«a tomada separa sempre»*.
#[test]
fn desarmado_a_tomada_e_o_desenho_concordam_no_estado_cru() {
    let (desenho, tomada) = desenhado_e_tomada(None);
    assert_eq!(tomada, desenho);
    assert_eq!(
        desenho,
        vec![[0.0, 0.0], [0.5, 0.0]],
        "desarmado, as duas leem a corrente do cozimento AO BIT"
    );
}

/// ⛔⛔⛔ **UMA TOMADA QUE NÃO É SINK NUNCA É SEPARADA — e a armadilha tem nome.**
///
/// O interruptor do sink chama-se `"collide"`… e a **`source.shape` tem um param com o MESMO
/// nome** (`ph2d_node_motion_shape::param::COLLIDE`, o botão que faz a forma declarar a caixa
/// dela). O gizmo do colisor toma os DOIS nós — a forma e o sink —, logo uma cura que perguntasse
/// o param à cega separaria também a corrente da **geometria da própria forma**.
///
/// ⇒ o discriminador é *«este nó é um dos SINKS deste quadro?»*, e não o param.
///
/// ⚠️⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE:** apagar a cerca dos sinks (separar toda
/// tomada) passava a suíte inteira, porque a única tomada da fixtura ao lado **é** o sink. *Uma
/// cerca que a fixtura não exercita é uma cerca por afirmar.*
#[test]
fn uma_tomada_que_nao_e_sink_nunca_e_separada() {
    let mut g = Graph::new();
    // O nó TAPADO: armado como se fosse a `source.shape` com o `Collide` dela ligado.
    let forma = g.add_node(SRC_MAN.name);
    g.set_param(forma, SINK_COLLIDE_PARAM, 1.0);
    g.set_param(forma, SINK_COLLIDE_ITERATIONS_PARAM, 32.0);
    // E o SINK, que é outro nó.
    let sink = g.add_node(SRC_MAN.name);

    let mut pump = MotionCookPump::new();
    pump.set_taps(&[forma]);
    assert!(pump.pump(&g, &Ops, &[sink], 0, 0.0, [0.0, 0.0, 1.0, 1.0], [1.0, 1.0]));

    let p = pump
        .tap_streams()
        .iter()
        .find(|(n, _)| *n == forma)
        .map(|(_, s)| match s.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!("a tomada tem de trazer P"),
        })
        .expect("a tomada da forma existe");
    assert_eq!(
        p,
        vec![[0.0, 0.0], [0.5, 0.0]],
        "a tomada de um no' que NAO e' sink tem de trazer a corrente crua, mesmo com um param \
         `collide` armado — senao a geometria da propria forma sai separada"
    );
}
