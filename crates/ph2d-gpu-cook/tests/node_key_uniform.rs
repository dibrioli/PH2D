//! **O uniforme da IDENTIDADE do nó** — o campo que o `EvalCtx::node_key`
//! espelha no device.
//!
//! Ele é **DERIVADO** do kernel (`codegen::declares_node_key`) em vez de
//! declarado num canal do resolver, pela mesma disciplina de `declares_window` e
//! `broadcasts_anything`: a pergunta é feita a partir do que o kernel É, e as
//! duas metades — a que DECLARA o campo e a que o PREENCHE — chamam a MESMA
//! função. Estes gates pinam as duas propriedades que isso compra: o campo
//! aparece exatamente onde é pedido, e **em lugar nenhum onde não é**.

use ph2d_gpu_cook::codegen;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, KernelResolver};
use ph2d_nodegraph::port::Dim;

fn kernel(body: &'static str, lib: &'static str) -> GpuKernel {
    GpuKernel {
        wgsl: body,
        wgsl_lib: lib,
        bindings: &[ColumnBinding {
            column: "v",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        }],
        params: &["seed"],
        count_law: None,
        variant_by_param: None,
        applicable: None,
    }
}

/// **Pede ⇒ tem; não pede ⇒ não tem.** As duas metades, porque um gate que só
/// afirmasse a primeira passaria num mundo em que TODO kernel carrega o campo —
/// e aí o custo (quatro bytes por dispatch) seria pago por 130 nós que nunca o
/// leem.
#[test]
fn the_node_key_uniform_appears_exactly_when_the_kernel_asks() {
    let asks = kernel("write_v(i, f32(params.node_key));\n", "");
    let quiet = kernel("write_v(i, params.seed);\n", "");
    assert!(codegen::declares_node_key(&asks));
    assert!(!codegen::declares_node_key(&quiet));

    let src_asks = codegen::kernel_module(&asks, asks.bindings, &[], None, &[], &[], |_| false);
    let src_quiet = codegen::kernel_module(&quiet, quiet.bindings, &[], None, &[], &[], |_| false);
    assert!(
        src_asks.contains("node_key: u32,"),
        "declarado onde e pedido"
    );
    assert!(
        !src_quiet.contains("node_key"),
        "e ausente onde nao e — 130 kernels nao pagam por ele"
    );
}

/// **A biblioteca do kernel conta como pedido** — um helper em `wgsl_lib` que
/// leia a identidade é tão consumidor quanto o corpo, e um predicado que só
/// olhasse o corpo compilaria contra um campo inexistente.
#[test]
fn a_library_helper_that_reads_it_counts_as_asking() {
    let via_lib = kernel(
        "write_v(i, k());\n",
        "fn k() -> f32 { return f32(params.node_key); }\n",
    );
    assert!(codegen::declares_node_key(&via_lib));
    let src = codegen::kernel_module(&via_lib, via_lib.bindings, &[], None, &[], &[], |_| false);
    assert!(src.contains("node_key: u32,"));
}

/// **O campo é o ÚLTIMO do layout** — é isso que garante que nenhum
/// deslocamento acima dele se move, e que o sequenciador (que soma os offsets na
/// mesma ordem) continua a acertar em todo kernel que já shipava.
///
/// ⚠️ **A fixture declara uma GRADE de propósito.** A primeira versão deste gate
/// passava `None` ali, e sem grade **não existe campo que pudesse vir depois** —
/// *"é o último"* era verdade por vácuo, e a mutação que move o `node_key` para
/// antes da grade **sobreviveu**. Os dois campos da grade são hoje os únicos que
/// disputam a última posição, então é com eles que a pergunta se faz.
#[test]
fn the_node_key_is_last_in_the_layout() {
    static GRID: ph2d_nodegraph::gpu::GridSpec = ph2d_nodegraph::gpu::GridSpec {
        column: "P",
        port: 0,
        cell_param: "seed",
        sweeps_param: None,
    };
    let k = kernel("write_v(i, f32(params.node_key) + params.seed);\n", "");
    let src = codegen::kernel_module(&k, k.bindings, &[], Some(&GRID), &[], &[], |_| false);
    let head = src
        .split_once("struct KernelParams {")
        .expect("o struct")
        .1
        .split_once('}')
        .expect("fecha")
        .0;
    let fields: Vec<&str> = head
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(
        fields.last().copied(),
        Some("node_key: u32,"),
        "layout: {fields:?}"
    );
    // E o que veio antes continua onde estava.
    assert_eq!(fields.first().copied(), Some("count: u32,"));
    assert!(fields.contains(&"seed: f32,"));
    assert!(
        fields.contains(&"grid_num_buckets: u32,"),
        "a fixture CONTEM o fenomeno: ha um campo que poderia vir depois"
    );
}

/// **O nó que de facto o usa PEDE o campo** — o gate que liga o substrato ao
/// consumidor real, para o par não poder ser desligado por um lado só.
#[test]
fn the_instance_field_kernel_asks_for_it() {
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_value_instance_field::register(&mut reg).unwrap();
    let k = reg
        .gpu_kernel(ph2d_node_value_instance_field::MANIFEST.id)
        .expect("o kernel");
    assert!(
        codegen::declares_node_key(k),
        "o `value.instance_field` le a identidade"
    );
}
