//! Gates do `motion.sub_uv` (doc 89, folha 17).

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.sub_uv.test.src"),
    name: "motion.sub_uv.test.src",
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
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(4).with(
            "P",
            Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]),
        ));
    }
}

static VAL_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.sub_uv.test.val"),
    name: "motion.sub_uv.test.val",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "n",
        default: 1.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};
struct Val;
impl NodeOp for Val {
    fn manifest(&self) -> &'static NodeManifest {
        &VAL_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // `n` linhas: `1` prova o BROADCAST, `4` prova o por-elemento.
        let n = ctx.param("n").round().max(1.0) as usize;
        let v: Vec<f32> = (0..n).map(|i| (i * 2) as f32).collect();
        ctx.emit(Stream::new(n).with("v", Column::Scalar(v)));
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Src),
            t if t == VAL_MAN.id => Some(&Val),
            t if t == MANIFEST.id => Some(&MotionSubUv),
            _ => None,
        }
    }
}

/// Coze o nó com os params dados; devolve a coluna `uv_cell`.
fn cook_cells(params: &[(&str, f32)], port: Option<f32>, playhead: f64) -> Vec<[f32; 4]> {
    let mut g = Graph::new();
    let src = g.add_node("motion.sub_uv.test.src");
    let node = g.add_node("motion.sub_uv");
    for (k, v) in params {
        g.set_param(node, *k, *v);
    }
    g.connect(Edge {
        from: (src, 0),
        to: (node, 0),
        delayed: false,
    })
    .unwrap();
    if let Some(n) = port {
        let val = g.add_node("motion.sub_uv.test.val");
        g.set_param(val, "n", n);
        g.connect(Edge {
            from: (val, 0),
            to: (node, 1),
            delayed: false,
        })
        .unwrap();
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, node, playhead).unwrap();
    match out[0].as_stream().get(CELL_COLUMN) {
        Some(Column::Vec4(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// ⭐ **UMA GRELHA 1×1 É A IDENTIDADE** — o default, e portanto o mundo de antes deste
/// nó existir.
///
/// ⚠️ Não é uma trivialidade: a identidade do `uv_xform` é `[1,1,0,0]` e não `[0,0,0,0]`,
/// e uma lei que devolvesse zeros faria toda sprite deste grafo amostrar um ponto só.
#[test]
fn the_default_grid_is_the_identity_uv_transform() {
    for cells in cook_cells(&[], None, 0.0) {
        assert_eq!(cells, [1.0, 1.0, 0.0, 0.0]);
    }
}

/// ⭐⭐ **A ORDEM DAS CÉLULAS É A DA CASA — LINHA-MAIOR, a linha 0 no topo.**
///
/// O oráculo é a conta do `sprite_sheet_subrect` da shell (`col = k % cols`,
/// `row = k / cols`), que é a mesma que o Inspector e o importador de Aseprite usam.
/// ⚠️ Uma ordem por COLUNAS dá uma folha bonita e **todas as animações trocadas** — e
/// nada nesta casa daria erro.
#[test]
fn the_cells_are_numbered_row_major_like_every_other_sheet_in_the_app() {
    // 4×2: a célula 5 é coluna 1, linha 1.
    let got = cell_xform(5.0, 4, 2);
    assert_eq!(got, [0.25, 0.5, 0.25, 0.5]);
    // A célula 0 é o canto de cima à esquerda, sem deslocamento.
    assert_eq!(cell_xform(0.0, 4, 2), [0.25, 0.5, 0.0, 0.0]);
    // A última célula da 1.ª linha é a 3.
    assert_eq!(cell_xform(3.0, 4, 2), [0.25, 0.5, 0.75, 0.0]);
}

/// ⭐ **UM ÍNDICE NEGATIVO CONTA DO FIM, e um não-finito cai na célula 0.**
///
/// ⚠️ O `%` de Rust devolveria um resto NEGATIVO ⇒ um deslocamento negativo ⇒ o shader a
/// amostrar o ladrilho vizinho no atlas partilhado. É por isso que a lei é `rem_euclid`,
/// e é isto que o mede.
#[test]
fn a_negative_index_counts_from_the_end_and_a_broken_one_lands_on_cell_zero() {
    assert_eq!(cell_xform(-1.0, 4, 2), cell_xform(7.0, 4, 2));
    assert_eq!(cell_xform(-9.0, 4, 2), cell_xform(-1.0, 4, 2));
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(cell_xform(bad, 4, 2), cell_xform(0.0, 4, 2));
    }
    // E o embrulho é o da GRELHA INTEIRA, não o de um eixo.
    assert_eq!(cell_xform(8.0, 4, 2), cell_xform(0.0, 4, 2));
}

/// ⭐⭐ **A TAXA ANDA COM O RELÓGIO, e o ESCALONAMENTO separa os elementos.**
///
/// ⚠️ **O controle é a 1.ª asserção**: a `0` de taxa o relógio não pode mover nada, senão
/// uma cena estática cintilaria sozinha. E as duas somas são independentes — medi-las na
/// mesma corrida provaria só que ALGUMA delas mexeu.
#[test]
fn the_rate_walks_with_the_clock_and_the_stagger_separates_the_elements() {
    let grid = [("cols", 4.0), ("rows", 2.0)];
    let at = |extra: &[(&str, f32)], t: f64| {
        let mut p = grid.to_vec();
        p.extend_from_slice(extra);
        cook_cells(&p, None, t)
    };
    // CONTROLE: sem taxa nem escalonamento, o tempo nao move nada.
    assert_eq!(at(&[], 0.0), at(&[], 3.0));
    // A taxa: 2 celulas/s, 1,5 s ⇒ celula 3.
    assert_eq!(at(&[("speed", 2.0)], 1.5)[0], cell_xform(3.0, 4, 2));
    // O escalonamento: uma celula por elemento, e o tempo parado.
    let s = at(&[("stagger", 1.0)], 0.0);
    for (i, c) in s.iter().enumerate() {
        assert_eq!(*c, cell_xform(i as f32, 4, 2), "elemento {i}");
    }
}

/// ⭐⭐⭐ **A PORTA É A ESCADA DE SEMPRE**: vazia ⇒ o param · 1 ⇒ broadcast · n ⇒ por
/// elemento.
///
/// ⚠️ **A metade que quase ficou de fora é o BROADCAST**: um `value.*` desligado emite UMA
/// linha, e uma implementação que lesse `port[i]` daria a célula 0 aos elementos 1..n e
/// pareceria funcionar no elemento que o artista olha primeiro.
#[test]
fn the_cell_port_broadcasts_one_row_and_reads_n() {
    let grid = [("cols", 4.0), ("rows", 2.0), ("cell", 7.0)];
    // Desligada: o param.
    for c in cook_cells(&grid, None, 0.0) {
        assert_eq!(c, cell_xform(7.0, 4, 2));
    }
    // UMA linha: vale para todos, e o param perde.
    for c in cook_cells(&grid, Some(1.0), 0.0) {
        assert_eq!(c, cell_xform(0.0, 4, 2));
    }
    // `n` linhas: um índice por elemento (a fonte emite `2·i`).
    let per = cook_cells(&grid, Some(4.0), 0.0);
    for (i, c) in per.iter().enumerate() {
        assert_eq!(*c, cell_xform((i * 2) as f32, 4, 2), "elemento {i}");
    }
}

/// **A grelha é COAGIDA na porta.** Um `0` (ou um param dirigido negativo) dá a folha
/// inteira, que é o mundo sem este nó — nunca uma divisão por zero.
#[test]
fn a_degenerate_grid_falls_back_to_the_whole_sheet() {
    for bad in [0.0, -3.0] {
        for c in cook_cells(&[("cols", bad), ("rows", bad)], None, 0.0) {
            assert_eq!(c, [1.0, 1.0, 0.0, 0.0]);
        }
    }
    // ⚠️ **E o TETO é da porta, não da lei** — o `cell_xform` recebe uma grelha já
    // coagida, então é pelo COOK que ele se mede. Uma asserção sobre o `cell_xform`
    // directamente mediria uma coisa que aquela função nunca prometeu.
    let over = cook_cells(&[("cols", 9999.0), ("rows", 1.0)], None, 0.0);
    assert_eq!(over[0][0], 1.0 / MAX_CELLS_PER_AXIS);
}

/// **O nó PASSA O RESTO DO STREAM adiante** — ele acrescenta uma coluna, não substitui a
/// instância. Um `motion.sub_uv` que emitisse só a `uv_cell` apagaria as posições.
#[test]
fn the_node_adds_a_column_and_keeps_everything_else() {
    let mut g = Graph::new();
    let src = g.add_node("motion.sub_uv.test.src");
    let node = g.add_node("motion.sub_uv");
    g.connect(Edge {
        from: (src, 0),
        to: (node, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, node, 0.0).unwrap();
    let st = out[0].as_stream();
    assert_eq!(st.count(), 4);
    match st.get("P") {
        Some(Column::Vec2(v)) => assert_eq!(v[3], [3.0, 0.0]),
        _ => panic!("o `P` do stream de entrada tem de sobreviver"),
    }
}

/// **O WGSL ESPELHA A LEI** — a régua é textual e corre sem GPU nenhuma.
///
/// ⚠️ **É a metade que nenhuma varredura de naga vê:** um kernel que usasse o `%` do WGSL
/// (resto com o sinal do dividendo) em vez do `floor`-mod emite WGSL perfeitamente
/// válido, e divergiria da CPU **só** em índices negativos — que é onde o `stagger` para
/// trás vive.
#[test]
fn the_gpu_kernel_mirrors_the_euclidean_wrap_and_the_axis_ceiling() {
    let lib = GPU_KERNEL.wgsl_lib;
    assert!(
        lib.contains("return x - n * floor(x / n);"),
        "o kernel deixou de fazer o `rem_euclid` — um indice negativo sai da folha"
    );
    // ⚠️ **A régua tem de ignorar COMENTÁRIOS**, e ela não ignorava: a 1.ª corrida
    // reprovou por causa de um `%` numa linha de comentário que EXPLICA por que ele não
    // pode estar no código. *Uma régua que lê o texto inteiro mede a prosa também.*
    let code = |src: &str| {
        src.lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<String>()
    };
    assert!(
        !code(GPU_KERNEL.wgsl).contains('%') && !code(lib).contains('%'),
        "um `%` no WGSL e' resto com o sinal do dividendo, nao o `rem_euclid`"
    );
    assert!(
        lib.contains(&format!("clamp(r, 1.0, {MAX_CELLS_PER_AXIS:?});")),
        "o teto de eixo do device tem de ser o MAX_CELLS_PER_AXIS declarado"
    );
    // E a porta de índice é lida pelo acessor QUALIFICADO (2 portas ⇒ o nome da porta
    // entra no acessor). Um `read_v` aqui compilaria em Rust e não no naga.
    assert!(GPU_KERNEL.wgsl.contains("read_cell_v(i)"));
    assert!(GPU_KERNEL.wgsl.contains("HAS_cell_v"));
}
