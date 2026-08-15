//! Gates da SEMENTE POR NÓ — o `unique_per_node`, e o que ele custa quando
//! está desligado (nada, ao bit).

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// A grade de fixture: `n` instâncias, para o campo ler uma contagem.
static SEED_GRID: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.instance_field.seed.grid"),
    name: "value.instance_field.seed.grid",
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
struct SeedGrid(usize);
impl NodeOp for SeedGrid {
    fn manifest(&self) -> &'static NodeManifest {
        &SEED_GRID
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(self.0).with("P", Column::Vec2(vec![[0.0, 0.0]; self.0])));
    }
}
struct SeedOps(usize);
impl OpResolver for SeedOps {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SEED_GRID.id => Some(Box::leak(Box::new(SeedGrid(self.0))) as &dyn NodeOp),
            t if t == MANIFEST.id => Some(&ValueInstanceField),
            _ => None,
        }
    }
}

/// **DOIS** `value.instance_field` na MESMA grade, com a MESMA semente — cozidos
/// no mesmo grafo, que é o que faz deles irmãos de verdade.
fn twin_fields(n: usize, use_node: f32) -> (Vec<f32>, Vec<f32>) {
    let ops = SeedOps(n);
    let mut g = Graph::new();
    let grid = g.add_node("value.instance_field.seed.grid");
    let mut cook = Cook::new();
    let read = |g: &mut Graph, cook: &mut Cook| {
        let f = g.add_node("value.instance_field");
        g.set_param(f, "mode", 2.0); // Random
        g.set_param(f, "seed", 7.0);
        g.set_param(f, "unique_per_node", use_node);
        g.connect(Edge {
            from: (grid, 0),
            to: (f, 0),
            delayed: false,
        })
        .unwrap();
        let out = cook.cook(g, &ops, f, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => v.clone(),
            _ => panic!("v"),
        }
    };
    let a = read(&mut g, &mut cook);
    let b = read(&mut g, &mut cook);
    (a, b)
}

/// **A wave inteira num gate:** com o toggle desligado, dois nós de mesma
/// semente são GÊMEOS — campos idênticos, elemento a elemento; ligado, deixam
/// de ser.
///
/// ⚠️ **A metade dos gêmeos é o CONTROLE, e sem ela o gate não diz nada:** um
/// teste que só afirmasse *"ligado eles diferem"* passaria também num mundo em
/// que eles nunca foram iguais, ou seja num mundo sem o defeito que a wave
/// existe para curar.
#[test]
fn two_nodes_with_the_same_seed_are_twins_and_the_toggle_ends_it() {
    let (a, b) = twin_fields(64, 0.0);
    assert_eq!(a, b, "sem o toggle os dois nos sao GEMEOS — o defeito");

    let (a, b) = twin_fields(64, 1.0);
    assert_ne!(a, b, "com o toggle deixam de ser");
    // Não basta diferirem em algum lugar: eles têm de ser campos DIFERENTES.
    let same = a.iter().zip(&b).filter(|(x, y)| x == y).count();
    assert!(
        same < 4,
        "quase todo elemento difere (iguais: {same} de {})",
        a.len()
    );
}

/// **Desligado é o nó que shipava, AO BIT** — e o `0` do default é o que garante
/// que todo grafo já salvo lê o mesmo campo de sempre.
#[test]
fn off_is_the_field_that_shipped_bit_for_bit() {
    for (seed, key) in [(0u32, 0u32), (7, 3), (9999, 4_000_000_000)] {
        assert_eq!(
            decorrelate(seed, key, false),
            seed,
            "off nao move a semente (key {key})"
        );
    }
    // E o default do manifesto é o `off`.
    assert_eq!(MANIFEST.param_default("unique_per_node"), Some(0.0));
}

/// **A decorrelação não colide sob semente e id TROCADOS** — o motivo de o
/// `node_key` ser multiplicado antes de somar.
///
/// ⚠️ Somar o id cru faria `(semente 5, id 3)` e `(semente 3, id 5)` caírem na
/// MESMA semente efetiva, e dois nós que o artista afinou de propósito voltariam
/// a ser gêmeos.
#[test]
fn swapping_the_seed_and_the_id_does_not_collide() {
    assert_ne!(decorrelate(5, 3, true), decorrelate(3, 5, true));
    // E a mutação que este gate mata: a soma crua colidiria.
    assert_eq!(5u32.wrapping_add(3), 3u32.wrapping_add(5), "a premissa");
}

/// **Ids VIZINHOS dão campos independentes** — o `NodeId` é monotônico, então os
/// nós de um documento diferem de um em um, e é justamente esse par que tem de
/// se separar.
#[test]
fn neighbouring_node_ids_give_independent_fields() {
    let n = 512;
    for id in 0u32..6 {
        let a: Vec<f32> = (0..n)
            .map(|i| rand01(decorrelate(7, id, true), i))
            .collect();
        let b: Vec<f32> = (0..n)
            .map(|i| rand01(decorrelate(7, id + 1, true), i))
            .collect();
        let same = a.iter().zip(&b).filter(|(x, y)| x == y).count();
        assert!(same < 3, "ids {id}/{} colidem em {same} elementos", id + 1);
    }
}

/// **O toggle só é oferecido no modo que o LÊ** — Index e Ramp são funções puras
/// do ordinal, e um controle que não faz nada não é pintado (a mesma lei que já
/// esconde o `Seed`).
#[test]
fn the_toggle_is_offered_only_where_the_seed_is_read() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    let gates = reg.param_gates(MANIFEST.id).expect("gates");
    let g = gates
        .iter()
        .find(|g| g.param == "unique_per_node")
        .expect("o toggle e gateado");
    assert_eq!(g.when, "mode");
    assert_eq!(g.values, &[2], "so no Random");
}

/// **O kernel PEDE a identidade pelo nome que o gerador reconhece** — sem isto o
/// device compilaria `params.node_key` contra um campo que ninguém declarou.
#[test]
fn the_kernel_asks_for_the_node_key_by_the_name_the_codegen_knows() {
    assert!(
        GPU_KERNEL.wgsl.contains("params.node_key"),
        "o corpo le a identidade"
    );
    // E a rota do device espelha a lei da CPU: multiplica antes de somar.
    assert!(GPU_KERNEL.wgsl.contains("params.node_key * 0x9e3779b9u"));
    assert!(
        GPU_KERNEL.params.contains(&"unique_per_node"),
        "o param novo chega ao uniforme"
    );
}
