//! ⭐⭐⭐ **AS FORMAS NO DISPOSITIVO** (doc 121 do Motion, W3) — o cozimento escreve as linhas de
//! FORMA como cópias do passe de formas, num buffer próprio, e cala-as no buffer das sprites.
//!
//! Três metades, porque são três defeitos diferentes:
//! 1. **O layout** — as `16` palavras que o baixamento escreve são as do `#[repr(C)]` do
//!    `ph2d_shape_gpu::ShapeInstance` (puro, corre sempre). Sem ele, uma reordenação de campos
//!    naquela crate desenharia a cor na posição, com todos os outros gates verdes.
//! 2. **O valor** — cada cópia é a que a CPU baixa (`lower_to_vector_instances_onto`) sobre as
//!    MESMAS colunas que o dispositivo cozeu, lidas de volta: posição, tamanho, base, âncora
//!    (com pivô), handle e tinta; e só as linhas de forma, pela ordem das linhas.
//! 3. **A mistura por linha RECUSA** ([`GpuCookError::FormaComMistura`]) — a camada que ela pede
//!    não existe no passe, e a CPU desenha o quadro.
//!
//! ⚠️ A fronteira é uma `source.object` alimentada à mão (o molde do `gpu_texture_id`): uma fonte de
//! forma não corre sem a membrana da shell, e o que se mede aqui é o que o DISPOSITIVO faz com as
//! colunas que recebe.
//!
//! `#[ignore]` nos dois de adaptador. Na pista da GPU:
//!   `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-gpu-cook --test it \
//!        as_formas_no_dispositivo --release -- --ignored --nocapture`

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::lower_forma::{FORMA_WORDS, SEM_GEOMETRIA};
use ph2d_gpu_cook::{CookClock, GpuCook, GpuCookError, plan, read_formas, read_instances};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;
use ph2d_shape_gpu::ShapeInstance;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// ⭐ **As palavras do baixamento são as do `ShapeInstance`** — uma cópia com um valor diferente
/// em cada campo, vista como palavras, põe cada campo na palavra que o WGSL escreve.
#[test]
fn as_palavras_da_forma_sao_as_do_shape_instance() {
    assert_eq!(
        FORMA_WORDS as usize * 4,
        std::mem::size_of::<ShapeInstance>(),
        "o baixamento escreve 16 palavras por cópia"
    );
    let c = ShapeInstance {
        pos: [1.0, 2.0],
        size: [3.0, 4.0],
        basis: [5.0, 6.0, 7.0, 8.0],
        anchor: [9.0, 10.0],
        geometry: 77,
        _pad: 0,
        tint: [11.0, 12.0, 13.0, 14.0],
    };
    let w: &[u32] = bytemuck::cast_slice(std::slice::from_ref(&c));
    let f = |i: usize| f32::from_bits(w[i]);
    assert_eq!([f(0), f(1)], c.pos, "pos: palavras 0-1");
    assert_eq!([f(2), f(3)], c.size, "size: palavras 2-3");
    assert_eq!([f(4), f(5), f(6), f(7)], c.basis, "basis: palavras 4-7");
    assert_eq!([f(8), f(9)], c.anchor, "anchor: palavras 8-9");
    assert_eq!(w[10], 77, "geometry: palavra 10");
    assert_eq!([f(12), f(13), f(14), f(15)], c.tint, "tint: palavras 12-15");
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_source_object::register(&mut reg).unwrap();
    ph2d_node_motion_bend::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    reg
}

/// `fronteira → bend → saída`, com a fronteira por preencher.
fn grafo() -> (Graph, NodeId, NodeId, NodeId) {
    let mut g = Graph::new();
    let obj = g.add_node("source.object");
    let bend = g.add_node("motion.bend");
    let out = g.add_node("motion.output");
    g.set_param(bend, "angle", 30.0);
    for (a, b) in [(obj, bend), (bend, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    (g, obj, bend, out)
}

/// Cinco linhas: quatro formas (duas com o mesmo handle) e uma IMAGEM (`geometry_id = 0`) no meio.
fn fronteira() -> Stream {
    let n = 5;
    let mut s = Stream::new(n);
    s.set(
        "P",
        Column::Vec2((0..n).map(|i| [i as f32 * 0.5 - 1.0, 0.1]).collect()),
    );
    s.set(
        "size",
        Column::Vec2(vec![
            [1.0, 1.0],
            [2.0, 0.5],
            [0.3, 0.3],
            [1.5, 1.5],
            [0.7, 1.1],
        ]),
    );
    s.set("rot", Column::Scalar(vec![0.0, 30.0, -45.0, 90.0, 12.5]));
    s.set(
        "tint",
        Column::Vec4(vec![
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 0.5],
            [0.0, 0.0, 1.0, 1.0],
            [0.2, 0.4, 0.6, 0.8],
            [1.0, 1.0, 1.0, 1.0],
        ]),
    );
    s.set(
        "geometry_id",
        Column::Scalar(vec![11.0, 12.0, 0.0, 11.0, 13.0]),
    );
    s
}

const PIVO: [f32; 2] = [0.5, -0.25];

fn estilo() -> SinkStyle {
    SinkStyle {
        pivot: PIVO,
        ..SinkStyle::PLAIN
    }
}

/// ⭐⭐⭐ **Cada cópia do dispositivo é a da CPU sobre as mesmas colunas**, e a sprite cala as
/// linhas de forma.
#[test]
#[ignore = "precisa de adaptador"]
fn o_dispositivo_baixa_as_formas_como_a_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — as_formas_no_dispositivo saltado");
        return;
    };
    let reg = registry();
    let (g, obj, bend, out) = grafo();
    let p = plan(&g, &reg, &reg, out);
    assert_eq!(p.boundaries, vec![(obj, 0)], "a fonte é a fronteira");
    let b = fronteira();
    let mut gc = GpuCook::new();
    gc.retain_streams_for_debug(true);
    gc.cook(
        &gpu,
        &g,
        &reg,
        &reg,
        &p,
        &[(obj, &b)],
        CookClock {
            playhead: 0.0,
            tick: None,
        },
        [0.0, 0.0, 1.0, 1.0],
        [0.4, 0.4],
        estilo(),
    )
    .expect("o cozimento das formas");

    // As colunas que o dispositivo cozeu, lidas de volta — a entrada da CPU.
    let mut cozida = Stream::new(5);
    cozida.set(
        "P",
        Column::Vec2(gc.read_column_vec2(&gpu, bend, "P").expect("P")),
    );
    cozida.set(
        "size",
        Column::Vec2(gc.read_column_vec2(&gpu, bend, "size").expect("size")),
    );
    cozida.set(
        "rot",
        Column::Scalar(gc.read_column(&gpu, bend, "rot").expect("rot")),
    );
    cozida.set(
        "tint",
        Column::Vec4(gc.read_column_vec4(&gpu, bend, "tint").expect("tint")),
    );
    cozida.set(
        "geometry_id",
        Column::Scalar(
            gc.read_column(&gpu, bend, "geometry_id")
                .expect("geometry_id"),
        ),
    );
    let mut cpu = Vec::new();
    ph2d_eval_motion::lower_to_vector_instances_onto(&cozida, estilo(), &mut cpu);
    assert_eq!(cpu.len(), 4, "a CPU baixa as quatro formas e não a imagem");

    let palavras = read_formas(&gpu, gc.formas().expect("as formas foram escritas"));
    let dev: &[ShapeInstance] = bytemuck::cast_slice(&palavras);
    assert_eq!(dev.len(), 5, "uma cópia por linha");
    let formas: Vec<&ShapeInstance> = dev.iter().filter(|c| c.geometry != SEM_GEOMETRIA).collect();
    assert_eq!(
        dev[2].geometry, SEM_GEOMETRIA,
        "a linha de imagem leva a geometria que o passe não acha"
    );
    assert_eq!(formas.len(), cpu.len());
    let perto = |a: f32, b: f32| (a - b).abs() <= 1e-5 * (1.0 + b.abs());
    for (k, (d, c)) in formas.iter().zip(&cpu).enumerate() {
        assert_eq!(d.geometry, c.geometry_id, "cópia {k}: o handle");
        for (x, y, campo) in [
            (d.pos[0], c.world_pos[0], "pos.x"),
            (d.pos[1], c.world_pos[1], "pos.y"),
            (d.size[0], c.size[0], "size.x"),
            (d.size[1], c.size[1], "size.y"),
            (d.basis[0], c.basis[0], "basis.0"),
            (d.basis[1], c.basis[1], "basis.1"),
            (d.basis[2], c.basis[2], "basis.2"),
            (d.basis[3], c.basis[3], "basis.3"),
            (d.anchor[0], c.anchor[0], "anchor.x"),
            (d.anchor[1], c.anchor[1], "anchor.y"),
            (d.tint[0], c.tint[0], "tint.r"),
            (d.tint[1], c.tint[1], "tint.g"),
            (d.tint[2], c.tint[2], "tint.b"),
            (d.tint[3], c.tint[3], "tint.a"),
        ] {
            assert!(
                perto(x, y),
                "cópia {k}: {campo} dispositivo {x} contra CPU {y}"
            );
        }
    }
    // ⚠️ O pivô chegou: sem ele a âncora seria zero e a paridade passaria por coincidência.
    assert!(formas[1].anchor[0].abs() > 0.1, "o pivô entra na âncora");

    // A sprite CALA as linhas de forma e desenha a de imagem.
    let sprites = read_instances(&gpu, gc.instances().expect("as sprites"));
    assert_eq!(sprites.len(), 5);
    for (i, s) in sprites.iter().enumerate() {
        let forma = i != 2;
        assert_eq!(
            s.size == [0.0, 0.0] && s.opacity == 0.0,
            forma,
            "linha {i}: uma forma é um quad degenerado, a imagem não"
        );
    }
    eprintln!(
        "as_formas_no_dispositivo: {} formas iguais às da CPU",
        formas.len()
    );
}

/// ⛔ **Uma saída com formas e com a coluna `blend` recusa o quadro** — a mistura por linha pede
/// uma camada que o passe de formas não tem. ⚠️ O CONTROLO é a mesma fronteira sem a coluna.
#[test]
#[ignore = "precisa de adaptador"]
fn a_mistura_por_linha_de_uma_forma_recusa_o_quadro() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — saltado");
        return;
    };
    let reg = registry();
    let (g, obj, _, out) = grafo();
    let p = plan(&g, &reg, &reg, out);
    let clock = CookClock {
        playhead: 0.0,
        tick: None,
    };
    let mut gc = GpuCook::new();
    let sem = fronteira();
    let controlo = gc.cook(
        &gpu,
        &g,
        &reg,
        &reg,
        &p,
        &[(obj, &sem)],
        clock,
        [0.0, 0.0, 1.0, 1.0],
        [0.4, 0.4],
        SinkStyle::PLAIN,
    );
    assert!(
        controlo.is_ok(),
        "sem mistura, o dispositivo desenha as formas"
    );
    let mut com = fronteira();
    com.set("blend", Column::Scalar(vec![0.0, 2.0, 0.0, 0.0, 0.0]));
    assert_eq!(
        gc.cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &p,
            &[(obj, &com)],
            clock,
            [0.0, 0.0, 1.0, 1.0],
            [0.4, 0.4],
            SinkStyle::PLAIN,
        ),
        Err(GpuCookError::FormaComMistura)
    );
}
