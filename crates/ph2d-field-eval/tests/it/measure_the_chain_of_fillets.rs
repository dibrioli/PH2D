//! **SONDA** — a lei da SOMA DOS QUADRADOS contra o gradiente medido, em quatro formas de árvore.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::{Field, gradient_bound, leaf, safe_march_step};

fn grupo(children: Vec<NodeId>, op: Op) -> Node {
    Node {
        xform: Xform::IDENTITY,
        kind: NodeKind::Combine { op, children },
        mods: Vec::new(),
        verb: None,
    }
}

fn esfera(x: f32, y: f32) -> Node {
    leaf(Primitive::Sphere { radius: 0.3 }, Xform::at(x, y, 0.0))
}

const R: f32 = 0.2;
fn ex() -> Op {
    Op::Union(Blend::Exact { radius: R })
}

fn pior_gradiente(doc: &FieldDoc, meia: f64, passos: usize) -> f64 {
    let f = Field::new(doc);
    let mut pior = 0.0f64;
    for i in 0..passos {
        for j in 0..passos {
            for k in 0..passos {
                let c = |t: usize| -meia + 2.0 * meia * (t as f64 + 0.5) / passos as f64;
                let g = f.gradient_norm(c(i), c(j), c(k), 1e-4);
                if g.is_finite() {
                    pior = pior.max(g);
                }
            }
        }
    }
    pior
}

#[test]
fn measure_the_chain_of_fillets() {
    // (1) PLANA: n esferas empilhadas num nó só.
    let plana = |n: usize| {
        let mut v: Vec<Node> = (0..n)
            .map(|i| esfera(0.03 * i as f32, 0.02 * i as f32))
            .collect();
        v.push(grupo((0..n).map(|i| NodeId(i as u32)).collect(), ex()));
        FieldDoc::new(v, NodeId(n as u32)).expect("cena")
    };
    // (2) EQUILIBRADA: round(round(A,B), round(C,D)) — 4 folhas, profundidade 2.
    let equilibrada = || {
        let v = vec![
            esfera(-0.04, -0.03),
            esfera(0.04, -0.03),
            grupo(vec![NodeId(0), NodeId(1)], ex()),
            esfera(-0.04, 0.03),
            esfera(0.04, 0.03),
            grupo(vec![NodeId(3), NodeId(4)], ex()),
            grupo(vec![NodeId(2), NodeId(5)], ex()),
        ];
        FieldDoc::new(v, NodeId(6)).expect("cena")
    };
    // (3) IRMÃS: os mesmos dois pares, juntos por junta VIVA e afastados.
    let irmas = || {
        let v = vec![
            esfera(-1.0, -0.03),
            esfera(-0.9, -0.03),
            grupo(vec![NodeId(0), NodeId(1)], ex()),
            esfera(0.9, 0.03),
            esfera(1.0, 0.03),
            grupo(vec![NodeId(3), NodeId(4)], ex()),
            grupo(vec![NodeId(2), NodeId(5)], Op::Union(Blend::Sharp)),
        ];
        FieldDoc::new(v, NodeId(6)).expect("cena")
    };

    let mostra = |nome: &str, doc: &FieldDoc, l2_previsto: f64, meia: f64| {
        let g = pior_gradiente(doc, meia, 44);
        let hoje = f64::from(safe_march_step(doc));
        let novo = 1.0 / l2_previsto.sqrt();
        println!(
            "{nome:<18} tecto {:>6.4}  |grad| {g:>7.4}  esperado {:>7.4}  hoje*g {:>6.3}  \
             novo*g {:>6.3}",
            gradient_bound(doc),
            l2_previsto.sqrt(),
            hoje * g,
            novo * g
        );
        assert!(
            g <= l2_previsto.sqrt() * 1.02,
            "{nome}: |grad| {g:.4} passou o tecto {:.4}",
            l2_previsto.sqrt()
        );
    };

    println!();
    for n in 2..=8usize {
        mostra(&format!("plana n={n}"), &plana(n), n as f64, 1.0);
    }
    mostra("equilibrada 4", &equilibrada(), 4.0, 1.0);
    mostra("irmas (viva)", &irmas(), 2.0, 1.6);
}
