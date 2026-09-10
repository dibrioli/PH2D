//! ⭐⭐⭐ **A FITA PONTO A PONTO DEVOLVE OS MESMOS BITS** — a única coisa que autoriza a troca.
//!
//! # Por que este gate é o gate
//!
//! O [`ph2d_field_eval::Field`] é a régua de **52** sítios deste módulo: ele mede `‖∇f‖` por
//! diferença central, decide se uma forma ainda é uma distância, se um filete cabe, se um sulco
//! perfurou a chapa. Trocar o motor dele por um mais rápido é barato **se e só se** a resposta não
//! se mexer — senão a wave deixa de ser «acelerar uma sonda» e passa a ser «renegociar a barra de
//! todos os gates do módulo», que é trabalho de outra ordem.
//!
//! ⭐ A troca feita foi escolhida **para ter essa propriedade por construção** (ver
//! `point_tape.rs`): o mesmo grafo do `Context`, os mesmos `BinaryOpcode::eval` / `UnaryOpcode::eval`
//! da `fidget`, e só a **ordem de visita** diferente. Este gate mede-a mesmo assim — *uma
//! propriedade que se deduz e não se mede é uma promessa*.
//!
//! ⛔ **É `assert_eq!` sobre `f64`, e é deliberado.** Uma tolerância aqui não mediria nada: a
//! afirmação não é *«fica perto»*, é *«é o mesmo número»*. Se um dia um `NaN` aparecer, a
//! comparação por bits (`to_bits`) é que decide — `NaN != NaN` deixaria o gate passar em silêncio
//! sobre um campo que se partiu.

use fidget::context::Context;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::Field;

/// O caminho ANTIGO — `Context::eval_xyz`, tal e qual o `Field::at` o chamava.
struct Interpretado {
    ctx: Context,
    root: fidget::context::Node,
}

impl Interpretado {
    fn new(doc: &FieldDoc) -> Self {
        let tree = ph2d_field_eval::compile(doc);
        let mut ctx = Context::new();
        let root = ctx.import(&tree);
        Self { ctx, root }
    }
    fn at(&self, x: f64, y: f64, z: f64) -> f64 {
        self.ctx.eval_xyz(self.root, x, y, z).unwrap_or(f64::NAN)
    }
    fn nos(&self) -> usize {
        self.ctx.len()
    }
}

/// ⚠️ **Um documento com `n` formas, e não `n` documentos de uma forma.**
///
/// O preço que esta wave tirou é `O(tamanho do CONTEXTO)` **por amostra** — logo um corpus de peças
/// de uma primitiva mediria o caso em que o defeito quase não aparece. As junções entram com raio
/// (`Blend`) de propósito: elas é que fazem a árvore crescer para lá da soma das folhas.
fn peca_de(n: usize) -> FieldDoc {
    let mut nos: Vec<Node> = Vec::new();
    for i in 0..n {
        let t = i as f32 * 0.21;
        let p = match i % 4 {
            0 => Primitive::Sphere { radius: 0.3 },
            1 => Primitive::Box {
                half: [0.25, 0.2, 0.18],
                round: 0.05,
                chamfer: 0.0,
            },
            2 => Primitive::Torus {
                major: 0.3,
                minor: 0.1,
            },
            _ => Primitive::Capsule {
                half_height: 0.2,
                radius: 0.12,
            },
        };
        nos.push(Node::new(
            Xform {
                translation: [t.sin() * 0.5, t.cos() * 0.5, t * 0.1],
                ..Xform::default()
            },
            NodeKind::Leaf(p),
        ));
    }
    let filhos: Vec<NodeId> = (0..n as u32).map(NodeId).collect();
    nos.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(Blend::Organic { radius: 0.08 }),
            children: filhos,
        },
    ));
    FieldDoc::new(nos, NodeId(u32::try_from(n).expect("cabe"))).expect("a peça do corpus")
}

/// Uma grelha que cobre dentro, fora e a superfície — mais os eixos, onde os `abs`/`min` dobram.
fn pontos() -> Vec<(f64, f64, f64)> {
    let mut v = Vec::new();
    for i in 0..11 {
        for j in 0..11 {
            for k in 0..11 {
                let f = |a: usize| -1.0 + 0.2 * a as f64;
                v.push((f(i), f(j), f(k)));
            }
        }
    }
    // Os eixos exactos — é onde um `abs` ou um `min` escolhe entre dois braços iguais.
    for a in [-0.5, 0.0, 0.5] {
        v.push((a, 0.0, 0.0));
        v.push((0.0, a, 0.0));
        v.push((0.0, 0.0, a));
    }
    v
}

/// ⭐⭐⭐ **O gate.** Os dois motores, o mesmo corpus, os mesmos bits.
#[test]
fn the_tape_answers_exactly_what_the_interpreter_answered() {
    let mut amostras = 0usize;
    for n in [1usize, 2, 5, 17, 40] {
        let doc = peca_de(n);
        let velho = Interpretado::new(&doc);
        let novo = Field::new(&doc);
        for (x, y, z) in pontos() {
            let a = velho.at(x, y, z);
            let b = novo.at(x, y, z);
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "a fita discorda do interpretador em ({x}, {y}, {z}) na peça de {n} formas: \
                 {a:?} contra {b:?} — a troca deixou de ser gratuita e passa a mover as reguas \
                 de todo o modulo"
            );
            amostras += 1;
        }
    }
    // ⚠️ Controle de VÁCUO: um corpus que encolhesse para zero deixaria este gate verde a dizer
    // nada. Ver a família «censo de ausência» na memória do repo.
    assert!(
        amostras > 6_000,
        "o corpus encolheu para {amostras} amostras — o gate passou a nao medir nada"
    );
}

/// ⭐⭐ **E o GRADIENTE também**, porque é ele que 52 sítios de facto chamam.
///
/// ⚠️ A diferença central **amplifica**: ela divide por `2e-4`, logo um bit de discordância no
/// valor viraria `~1e-12` no gradiente. Medir só o `at` deixaria essa amplificação por gatear.
#[test]
fn the_gradient_ruler_did_not_move_a_single_bit() {
    let doc = peca_de(9);
    let velho = Interpretado::new(&doc);
    let novo = Field::new(&doc);
    let eps = 1.0e-4;
    let g_velho = |x: f64, y: f64, z: f64| {
        let gx = (velho.at(x + eps, y, z) - velho.at(x - eps, y, z)) / (2.0 * eps);
        let gy = (velho.at(x, y + eps, z) - velho.at(x, y - eps, z)) / (2.0 * eps);
        let gz = (velho.at(x, y, z + eps) - velho.at(x, y, z - eps)) / (2.0 * eps);
        (gx * gx + gy * gy + gz * gz).sqrt()
    };
    let mut n = 0usize;
    for i in 0..15 {
        for j in 0..15 {
            let (x, y) = (-0.7 + 0.1 * f64::from(i), -0.7 + 0.1 * f64::from(j));
            for z in [-0.3, 0.0, 0.35] {
                assert_eq!(
                    g_velho(x, y, z).to_bits(),
                    novo.gradient_norm(x, y, z, eps).to_bits(),
                    "o gradiente mudou em ({x}, {y}, {z}) — {} contra {}",
                    g_velho(x, y, z),
                    novo.gradient_norm(x, y, z, eps)
                );
                n += 1;
            }
        }
    }
    assert!(n > 600, "o corpus do gradiente encolheu para {n}");
}

/// ⭐⭐ **A TABELA do preço** — `#[ignore]`, porque um relógio não é um gate (§5.0).
///
/// Corra com:
/// ```text
/// cargo test -p ph2d-field-eval --test the_point_probe_is_the_same_answer -- --ignored --nocapture
/// ```
#[test]
#[ignore = "imprime a tabela do preço; um relógio não é um gate"]
fn the_table_of_what_a_sample_costs() {
    println!(
        "\nloadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!(
        "\n{:>7} {:>9} {:>13} {:>13} {:>9} {:>13} {:>13} {:>9}",
        "formas",
        "nos_ctx",
        "velho_ns/pt",
        "novo_ns/pt",
        "ganho",
        "velho_ns/grad",
        "novo_ns/grad",
        "ganho_g"
    );
    for n in [1usize, 4, 16, 64] {
        let doc = peca_de(n);
        let velho = Interpretado::new(&doc);
        let novo = Field::new(&doc);
        let ps = pontos();
        // Aquece as duas rotas — a fita cresce o scratch da thread na primeira passagem.
        let mut sink = 0.0f64;
        for (x, y, z) in ps.iter().take(50) {
            sink += velho.at(*x, *y, *z) + novo.at(*x, *y, *z);
        }
        let t0 = std::time::Instant::now();
        for (x, y, z) in &ps {
            sink += velho.at(*x, *y, *z);
        }
        let dv = t0.elapsed().as_secs_f64() / ps.len() as f64 * 1.0e9;
        let t1 = std::time::Instant::now();
        for (x, y, z) in &ps {
            sink += novo.at(*x, *y, *z);
        }
        let dn = t1.elapsed().as_secs_f64() / ps.len() as f64 * 1.0e9;
        // ⭐ O que os 52 sítios de facto chamam: seis amostras que viram um `‖∇f‖`.
        let eps = 1.0e-4;
        let g_velho = |x: f64, y: f64, z: f64| {
            let gx = (velho.at(x + eps, y, z) - velho.at(x - eps, y, z)) / (2.0 * eps);
            let gy = (velho.at(x, y + eps, z) - velho.at(x, y - eps, z)) / (2.0 * eps);
            let gz = (velho.at(x, y, z + eps) - velho.at(x, y, z - eps)) / (2.0 * eps);
            (gx * gx + gy * gy + gz * gz).sqrt()
        };
        for (x, y, z) in ps.iter().take(20) {
            sink += g_velho(*x, *y, *z) + novo.gradient_norm(*x, *y, *z, eps);
        }
        let t2 = std::time::Instant::now();
        for (x, y, z) in &ps {
            sink += g_velho(*x, *y, *z);
        }
        let gv = t2.elapsed().as_secs_f64() / ps.len() as f64 * 1.0e9;
        let t3 = std::time::Instant::now();
        for (x, y, z) in &ps {
            sink += novo.gradient_norm(*x, *y, *z, eps);
        }
        let gn = t3.elapsed().as_secs_f64() / ps.len() as f64 * 1.0e9;
        println!(
            "{n:>7} {:>9} {dv:>13.1} {dn:>13.1} {:>8.1}x {gv:>13.1} {gn:>13.1} {:>8.1}x",
            velho.nos(),
            dv / dn,
            gv / gn
        );
        assert!(sink.is_finite() || sink.is_nan(), "o sink existe");
    }
}
