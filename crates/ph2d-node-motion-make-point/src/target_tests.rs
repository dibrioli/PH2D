//! **A COLUNA QUE O PAR CONSTRÓI** (doc 89 folha 08 — o P0 do §0, a quarta
//! aparição do escritor ausente).
//!
//! A célula: *"construir uma coluna **Vec2** que NÃO seja `P` (ex. `vel`)"*, com
//! o veredito **NÃO — o §0, o mesmo escritor ausente**.
//!
//! ⚠️ **A refutação irmã (a do `motion.cull`) NÃO fecha esta**, e é por isso que a
//! folha as separa: aquela usa o canal `Falloff` do `motion.drive`, que escreve um
//! **escalar**. Medido, o `drive` escreve `P` · `rot` · `size` · `tint` ·
//! `falloff` — nenhuma rota dele alcança `vel` ou `accel`.
//!
//! ⚠️ **E a lista de alvos é MEDIDA, não escolhida.** As colunas `Vec2` que o repo
//! escreve são sete: `P` (296 escritas) · `size` (40) · `accel` (22) · `vel` (20)
//! · e `sim_d`/`sb_vel`/`rope_prev` (4/2/2). As três últimas são estado PRIVADO de
//! um nó cada; o `size` já tem escritor (`drive`) e dono prescrito noutra folha.
//! Sobram as duas inalcançáveis — que é exactamente o que faz do P0 um P0.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Uma fonte de duas instâncias com `P` já posto — o que um layout a montante dá.
static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.make_point.target.src"),
    name: "motion.make_point.target.src",
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
        ctx.emit(
            Stream::new(2)
                .with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]))
                .with("size", Column::Vec2(vec![[0.5, 0.5]; 2])),
        );
    }
}

/// Um campo de valor constante, para as portas `x`/`y`.
static VAL: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.make_point.target.val"),
    name: "motion.make_point.target.val",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "k",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};
struct Val;
impl NodeOp for Val {
    fn manifest(&self) -> &'static NodeManifest {
        &VAL
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let k = ctx.param("k");
        ctx.emit(Stream::new(2).with(VALUE_COL, Column::Scalar(vec![k; 2])));
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC.id => Some(&Src),
            t if t == VAL.id => Some(&Val),
            t if t == MANIFEST.id => Some(&MotionMakePoint),
            _ => None,
        }
    }
}

/// Constrói `src → make_point(target)` com `x = 7, y = 9` e devolve o stream.
fn built(target: f32) -> Stream {
    let mut g = Graph::new();
    let src = g.add_node("motion.make_point.target.src");
    let xv = g.add_node("motion.make_point.target.val");
    g.set_param(xv, "k", 7.0);
    let yv = g.add_node("motion.make_point.target.val");
    g.set_param(yv, "k", 9.0);
    let mp = g.add_node("motion.make_point");
    g.set_param(mp, "target", target);
    for (from, port) in [(src, 0), (xv, 1), (yv, 2)] {
        g.connect(Edge {
            from: (from, 0),
            to: (mp, port),
            delayed: false,
        })
        .expect("in");
    }
    let mut cook = Cook::new();
    cook.cook(&g, &Ops, mp, 0.0).expect("cook")[0]
        .as_stream()
        .clone()
}

fn vec2(s: &Stream, name: &str) -> Option<Vec<[f32; 2]>> {
    match s.get(name) {
        Some(Column::Vec2(v)) => Some(v.clone()),
        _ => None,
    }
}

/// **O default é o nó que shipava** — `target = 0` escreve `P` e o resto atravessa.
#[test]
fn the_default_target_is_the_position_the_node_always_built() {
    let s = built(0.0);
    assert_eq!(vec2(&s, "P"), Some(vec![[7.0, 9.0], [7.0, 9.0]]));
    assert_eq!(
        vec2(&s, "size"),
        Some(vec![[0.5, 0.5]; 2]),
        "as outras colunas atravessam"
    );
    assert!(
        vec2(&s, "vel").is_none() && vec2(&s, "accel").is_none(),
        "e nenhuma coluna e inventada"
    );
}

/// **A velocidade e a aceleração são construíveis, e o `P` da entrada SOBREVIVE.**
///
/// ⚠️ Esta segunda metade é a que carrega o peso. O laço de pass-through excluía
/// `"P"` literal, então escrever `vel` teria APAGADO a geometria em que a
/// velocidade age — um adaptador que semeia um campo sobre coisa nenhuma. O que é
/// excluído passou a ser a coluna ALVO.
#[test]
fn the_other_vec2_conventions_are_buildable_and_the_position_survives() {
    for (target, col) in [(1.0, "vel"), (2.0, "accel")] {
        let s = built(target);
        assert_eq!(
            vec2(&s, col),
            Some(vec![[7.0, 9.0], [7.0, 9.0]]),
            "o par constroi `{col}`"
        );
        assert_eq!(
            vec2(&s, "P"),
            Some(vec![[1.0, 2.0], [3.0, 4.0]]),
            "e o `P` da entrada atravessa INTOCADO (target = {target})"
        );
    }
}

/// **O índice é arredondado, não truncado** — o param é `f32` e um `0,999999`
/// vindo de um slider não pode virar a coluna anterior.
#[test]
fn the_target_index_rounds_instead_of_truncating() {
    assert_eq!(Target::of(0.999_999), Target::Velocity);
    assert_eq!(Target::of(1.000_001), Target::Velocity);
    assert_eq!(Target::of(1.6), Target::Acceleration);
    // E um índice que não existe cai no nó que shipava, nunca num pânico.
    assert_eq!(Target::of(9.0), Target::Position);
    assert_eq!(Target::of(-3.0), Target::Position);
}

/// **A lista de alvos é a das colunas que o domínio de VALOR não alcança.**
///
/// ⚠️ Este gate não mede aritmética, mede uma DECISÃO — e existe para que
/// acrescentar um alvo custe reconferir o motivo. `size` fica de fora porque o
/// `motion.drive` já o escreve e o por-eixo tem forma prescrita na folha 05; as
/// três colunas de estado privado (`sim_d`/`sb_vel`/`rope_prev`) ficam de fora
/// porque oferecê-las é deixar o artista corromper o buffer de um solver.
#[test]
fn the_target_list_is_the_columns_the_value_domain_cannot_otherwise_write() {
    let cols: Vec<&str> = [Target::Position, Target::Velocity, Target::Acceleration]
        .iter()
        .map(|t| t.column())
        .collect();
    assert_eq!(cols, vec!["P", "vel", "accel"]);
    assert!(
        !cols.contains(&"size"),
        "o `size` tem escritor (drive) e dono prescrito noutra folha"
    );
    for private in ["sim_d", "sb_vel", "rope_prev"] {
        assert!(
            !cols.contains(&private),
            "`{private}` e estado privado de um solver, nao uma convencao"
        );
    }
}

/// **O rótulo do enum e a coluna andam JUNTOS** — o índice é o que o grafo guarda,
/// então um rótulo que descreve outra coluna renomearia o alvo de todo documento
/// já autorado, em silêncio.
#[test]
fn every_label_names_the_column_its_index_builds() {
    let ParamWidget::Enum { labels } = PARAM_HINTS[0].widget else {
        panic!("o alvo e um seletor de opcoes")
    };
    assert_eq!(labels.len(), 3, "tres alvos, tres rotulos");
    #[expect(clippy::cast_precision_loss, reason = "tres opcoes")]
    for (i, label) in labels.iter().enumerate() {
        let built = Target::of(i as f32);
        let expected = match *label {
            "Position" => Target::Position,
            "Velocity" => Target::Velocity,
            "Acceleration" => Target::Acceleration,
            other => panic!("rotulo sem alvo: {other}"),
        };
        assert_eq!(built, expected, "o indice {i} constroi o que `{label}` diz");
    }
}
