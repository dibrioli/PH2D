//! ⭐⭐⭐ **A COSTURA QUE NÃO MUDOU NÃO VOLTA A ATRAVESSAR** (ciclo 8, W1 —
//! [doc 113](../../../../docs/Motion%20Nodes/113_ciclo_8_fontes_e_dados.md) §6).
//!
//! ⚠️ **Ficheiro próprio, e com o registo INTEIRO:** o sujeito é uma FONTE de dados (a `source.table`
//! é a que pesa), e os registos à mão dos ficheiros de paridade vizinhos não a têm — um gate
//! montado sobre um tipo que o registry não conhece não mede o produto, mede um grafo inválido.
//!
//! `#[ignore]`: precisa de adaptador.
//! ```text
//! cargo test -p ph2d-gpu-cook --test it -- --ignored --nocapture an_unchanged_boundary
//! ```

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó regista");
    reg
}

fn connect(g: &mut Graph, a: NodeId, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    })
    .unwrap();
}

/// ⭐⭐⭐ **A COSTURA PARADA NÃO VOLTA A ATRAVESSAR — e o quadro sai IGUAL** (ciclo 8, W1 — doc 113 §6).
///
/// ⚠️ **As duas metades, e nenhuma basta:**
/// 1. o CONTADOR — o segundo quadro reutiliza o envio (sem ele, a cura é invisível e evapora);
/// 2. os BITS — o que a placa desenha no segundo quadro é **byte a byte** o do primeiro. É esta
///    metade que prova a premissa da cura: *nenhum estágio escreve no buffer que recebe*. Se algum
///    escrevesse, o quadro dois leria o buffer já mexido e a imagem derivaria — em silêncio.
///
/// ⚠️ E a TERCEIRA: uma costura que MUDA volta a ser enviada (senão a cura seria «nunca enviar»).
#[test]
#[ignore = "precisa de adapter de GPU"]
fn an_unchanged_boundary_is_uploaded_once_and_draws_the_same_bits() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // `source.table` é a fonte cuja costura pesa; aqui basta um stream publicado à mão pelo
    // chamador, que é exactamente o que a membrana faz.
    let mut g = Graph::new();
    let fonte = g.add_node("source.table");
    let escala = g.add_node("motion.scale");
    g.set_param(escala, "amount", 0.5);
    let out = g.add_node("motion.output");
    connect(&mut g, fonte, escala);
    connect(&mut g, escala, out);
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert_eq!(
        plan.boundaries.len(),
        1,
        "a fonte e' a costura: {:?}",
        plan.boundaries
    );

    let tabela = Stream::new(3)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]; 3]));
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    let mut cozer = |gc: &mut ph2d_gpu_cook::GpuCook, s: &Stream| {
        gc.cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &plan,
            &[(fonte, s)],
            CookClock::at(0.0),
            DEFAULT_UV,
            DEFAULT_SIZE,
            SinkStyle::PLAIN,
        )
        .expect("gpu cook");
        // Os BYTES, e não a struct: é a comparação que a promessa faz («byte a byte»), e a
        // `RenderInstance` não é `PartialEq`.
        let inst = ph2d_gpu_cook::read_instances(&gpu, gc.instances().expect("cozido"));
        bytemuck::cast_slice::<_, u8>(&inst).to_vec()
    };
    let q1 = cozer(&mut gc, &tabela);
    let (env1, reuso1) = gc.boundary_upload_counts();
    assert_eq!((env1, reuso1), (1, 0), "o primeiro quadro envia");
    // O que a membrana faz no quadro seguinte com uma tabela parada: o MESMO stream (clone =
    // refcount sobre as mesmas colunas).
    let q2 = cozer(&mut gc, &tabela.clone());
    let (env2, reuso2) = gc.boundary_upload_counts();
    assert_eq!((env2, reuso2), (1, 1), "o segundo quadro reutiliza");
    assert_eq!(q1, q2, "o quadro reutilizado tem de sair byte a byte igual");
    // A metade que falsifica: outro conteúdo volta a atravessar.
    let outra = Stream::new(3)
        .with("P", Column::Vec2(vec![[0.0, 5.0], [1.0, 5.0], [2.0, 5.0]]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]; 3]));
    let q3 = cozer(&mut gc, &outra);
    let (env3, _) = gc.boundary_upload_counts();
    assert_eq!(env3, 2, "uma costura diferente e' enviada");
    assert_ne!(q1, q3, "e o quadro muda");
}
