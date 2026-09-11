//! ⭐⭐⭐ **A NOTA DIZIA QUE A CURA PUBLICADA «MOVE OS NÚMEROS DE TODOS OS GATES». ISTO MEDE-O.**
//!
//! # O item aberto, tal como estava escrito
//!
//! > *`Field::at` é 143× mais lento que o caminho em lote — a cura publicada é o avaliador de
//! > gradiente ANALÍTICO da `fidget` (já usado no `hybrid.rs`); é uma wave com espec própria porque
//! > **move os números de todos os gates do módulo**.*
//!
//! ⚠️ **A segunda metade nunca foi medida** — e §0.0 diz que um limite que não nomeia o recurso é um
//! palpite à espera de um smoke. Esta sonda dá-lhe o número.
//!
//! # As DUAS coisas que se confundem numa só
//!
//! | | de onde vem | cura-se com precisão? |
//! |---|---|---|
//! | **(a)** `f32` contra `f64` | representação | sim, em princípio |
//! | **(b)** gradiente **analítico** contra **diferença central** | são grandezas DIFERENTES | ⛔ **não** |
//!
//! ⭐⭐⭐ **A (b) é a que decide, e é estrutural.** Num vinco (`min`/`max`) a derivada **não existe**:
//! a diferença central com `eps = 1e-4` põe um pé de cada lado e devolve a **média** dos dois
//! gradientes laterais; o analítico escolhe **um ramo** e devolve `1`. As réguas deste módulo
//! existem precisamente para medir o que acontece **em cima das junções** — ou seja, exactamente
//! onde as duas respostas divergem por construção.
//!
//! ⇒ esta sonda parte o corpus em **liso** e **vinco** e mede as duas populações em separado. Se a
//! discordância viver toda no vinco, a nota está certa **e agora tem mecanismo**; se for ruído em
//! todo o lado, o bloqueio dissolve-se e há uma wave de velocidade à espera.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::Field;
use ph2d_field_eval::hybrid::{Hybrid, Registry};

/// Duas caixas que se cruzam: garante vincos a sério (o `min` de dois braços) **e** faces lisas.
fn peca() -> FieldDoc {
    let nos = vec![
        Node::new(
            Xform {
                translation: [-0.15, 0.0, 0.0],
                ..Xform::default()
            },
            NodeKind::Leaf(Primitive::Box {
                half: [0.3, 0.25, 0.2],
                round: 0.0,
                chamfer: 0.0,
            }),
        ),
        Node::new(
            Xform {
                translation: [0.18, 0.05, 0.0],
                ..Xform::default()
            },
            NodeKind::Leaf(Primitive::Sphere { radius: 0.28 }),
        ),
        Node::new(
            Xform::IDENTITY,
            NodeKind::Combine {
                op: Op::Union(Blend::Sharp),
                children: vec![NodeId(0), NodeId(1)],
            },
        ),
    ];
    FieldDoc::new(nos, NodeId(2)).expect("a peça")
}

/// ⚠️ **Como se sabe que um ponto está sobre um vinco, sem perguntar ao suspeito.**
///
/// Um vinco do `min` é onde os dois braços empatam. Em vez de os inspeccionar, mede-se a
/// **assimetria** da diferença central: num ponto liso a diferença para a frente e para trás
/// concordam; num vinco elas discordam, e a discordância é da ordem do próprio gradiente.
fn dobra(f: &Field, x: f64, y: f64, z: f64, eps: f64) -> f64 {
    let c = f.at(x, y, z);
    let mut pior: f64 = 0.0;
    for (dx, dy, dz) in [(eps, 0.0, 0.0), (0.0, eps, 0.0), (0.0, 0.0, eps)] {
        let mais = f.at(x + dx, y + dy, z + dz);
        let menos = f.at(x - dx, y - dy, z - dz);
        // |f(+) + f(−) − 2f(0)| — a segunda diferença. Zero numa função linear, grande num vinco.
        pior = pior.max((mais + menos - 2.0 * c).abs() / eps);
    }
    pior
}

#[test]
#[ignore = "sonda de diagnóstico: imprime a tabela que decide o item aberto"]
fn does_the_analytic_gradient_actually_move_the_rulers() {
    let doc = peca();
    let f = Field::new(&doc);
    let mut h = Hybrid::new(&doc, &Registry::default());
    let eps = 1.0e-4;

    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    let mut alvo = Vec::new();
    for i in 0..40 {
        for j in 0..40 {
            for k in 0..40 {
                let g = |a: i32| -0.8 + 0.041 * f64::from(a);
                let (x, y, z) = (g(i), g(j), g(k));
                // Só junto da superfície — é lá que toda régua deste módulo mede.
                if f.at(x, y, z).abs() > 0.05 {
                    continue;
                }
                xs.push(x as f32);
                ys.push(y as f32);
                zs.push(z as f32);
                alvo.push((x, y, z));
            }
        }
    }
    let mut out = Vec::new();
    h.gradients(&xs, &ys, &zs, eps as f32, &mut out)
        .expect("o gradiente analítico");
    assert_eq!(out.len(), alvo.len(), "uma resposta por ponto");

    // Duas populações: LISO e VINCO, partidas pela segunda diferença.
    let mut liso: Vec<f64> = Vec::new();
    let mut vinco: Vec<f64> = Vec::new();
    for (i, (x, y, z)) in alvo.iter().enumerate() {
        let central = f.gradient_norm(*x, *y, *z, eps);
        let g = out[i];
        let analitico = f64::from((g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt());
        let d = (central - analitico).abs();
        if dobra(&f, *x, *y, *z, eps) > 1.0e-3 {
            vinco.push(d);
        } else {
            liso.push(d);
        }
    }
    let resumo = |v: &mut Vec<f64>| -> (usize, f64, f64) {
        if v.is_empty() {
            return (0, 0.0, 0.0);
        }
        v.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
        (v.len(), v[v.len() / 2], v[v.len() - 1])
    };
    let (nl, ml, xl) = resumo(&mut liso);
    let (nv, mv, xv) = resumo(&mut vinco);
    println!(
        "\n{:>16} {:>8} {:>12} {:>12}",
        "regiao", "pontos", "mediana", "pior"
    );
    println!("{:>16} {nl:>8} {ml:>12.3e} {xl:>12.3e}", "liso");
    println!("{:>16} {nv:>8} {mv:>12.3e} {xv:>12.3e}", "vinco (grelha)");

    // ⛔⛔ **A grelha NÃO produz o fenómeno, e uma sonda que pára aqui mente.** A hipótese de que
    // vinco e liso divergem vive num conjunto de MEDIDA NULA: a chance de um ponto de uma grelha
    // de passo `0,041` cair a menos de `eps = 1e-4` de uma aresta é ~`0,5 %`. ⇒ os pontos do vinco
    // POEM-SE, não se procuram.
    //
    // A peça abaixo é escolhida para ter uma aresta cuja equação se escreve: a esfera atravessa a
    // face de cima da caixa, e o vinco é a CIRCUNFERÊNCIA dessa intersecção.
    let nos = vec![
        Node::new(
            Xform::IDENTITY,
            NodeKind::Leaf(Primitive::Box {
                half: [0.3, 0.3, 0.2],
                round: 0.0,
                chamfer: 0.0,
            }),
        ),
        Node::new(
            Xform {
                translation: [0.0, 0.0, 0.12],
                ..Xform::default()
            },
            NodeKind::Leaf(Primitive::Sphere { radius: 0.25 }),
        ),
        Node::new(
            Xform::IDENTITY,
            NodeKind::Combine {
                op: Op::Union(Blend::Sharp),
                children: vec![NodeId(0), NodeId(1)],
            },
        ),
    ];
    let doc2 = FieldDoc::new(nos, NodeId(2)).expect("a peça da aresta");
    let f2 = Field::new(&doc2);
    let mut h2 = Hybrid::new(&doc2, &Registry::default());
    // A face de cima está em `z = 0,2`; a esfera (centro `z = 0,12`, raio `0,25`) corta-a num
    // círculo de raio `√(0,25² − 0,08²)`.
    let r = (0.25f64 * 0.25 - 0.08 * 0.08).sqrt();
    let (mut ax, mut ay, mut az, mut alvo2) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for i in 0..360 {
        let t = f64::from(i) * std::f64::consts::TAU / 360.0;
        let (x, y, z) = (r * t.cos(), r * t.sin(), 0.2);
        if x.abs() > 0.3 || y.abs() > 0.3 {
            continue;
        }
        ax.push(x as f32);
        ay.push(y as f32);
        az.push(z as f32);
        alvo2.push((x, y, z));
    }
    let mut out2 = Vec::new();
    h2.gradients(&ax, &ay, &az, eps as f32, &mut out2)
        .expect("gradiente na aresta");
    let mut sobre: Vec<f64> = Vec::new();
    for (i, (x, y, z)) in alvo2.iter().enumerate() {
        let central = f2.gradient_norm(*x, *y, *z, eps);
        let g = out2[i];
        let analitico = f64::from((g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt());
        sobre.push((central - analitico).abs());
    }
    let (ns, ms, xs_) = resumo(&mut sobre);
    println!("{:>16} {ns:>8} {ms:>12.3e} {xs_:>12.3e}", "vinco (POSTO)");
    println!("\nA barra tipica deste modulo e SLACK = 1,02 sobre um |grad| de 1,0 ⇒ folga 2,0e-2.");
}
