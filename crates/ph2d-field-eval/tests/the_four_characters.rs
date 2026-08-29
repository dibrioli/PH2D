//! ⭐⭐⭐ **A SONDA QUE ESCREVE DOIS NÚMEROS** (W99) — o `‖∇f‖` do chanfro e a calibração do orgânico.
//!
//! ⚠️ **Ela é o oráculo de duas constantes do produto**, e por isso vive fora dos testes de unidade:
//! o `march_depth` precisa de saber em que balde o chanfro cai (um erro ali **fura** a peça), e a
//! fileira de caracteres precisa que os quatro meçam a **mesma** coisa (senão trocar de carácter
//! muda o tamanho da peça sem ninguém ter mexido num raio).
//!
//! Corre com `cargo test -p ph2d-field-eval --test the_four_characters -- --nocapture`.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::Field;

/// Duas caixas grandes que se cruzam em ângulo **reto** — o canto de 90° em que a régua do filete e
/// a do chanfro são definidas.
///
/// ⚠️ **Meias-extensões grandes de propósito**: a régua olha o canto na origem, e uma caixa curta
/// poria a outra face dela dentro da janela de medida.
fn corner(blend: Blend) -> FieldDoc {
    let plate = |half: [f32; 3], at: [f32; 3]| {
        Node::new(
            Xform::at(at[0], at[1], at[2]),
            NodeKind::Leaf(Primitive::Box { half, round: 0.0 }),
        )
    };
    FieldDoc::new(
        vec![
            // Ocupa `x ≤ 0`: a face dela é o plano `x = 0`, logo `a = x`.
            plate([2.0, 2.0, 2.0], [-2.0, 0.0, 0.0]),
            // Ocupa `y ≤ 0`: `b = y`.
            plate([2.0, 2.0, 2.0], [0.0, -2.0, 0.0]),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(blend),
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ],
        NodeId(2),
    )
    .expect("o canto")
}

/// **Onde a superfície cruza a diagonal do canto**, medida da quina para fora.
///
/// ⚠️ **É a única régua que os quatro caracteres partilham.** O filete exacto de raio `r` cruza a
/// diagonal a `r·(√2 − 1)` da quina — geometria, não medição —, então converter a travessia de volta
/// por esse factor dá o **raio equivalente** de qualquer carácter. Sem uma régua comum, «3/4» e
/// «85 %» são as duas verdadeiras sobre grandezas diferentes.
fn diagonal_crossing(doc: &FieldDoc) -> f64 {
    let f = Field::new(doc);
    // A quina viva está na origem; caminha-se pela diagonal `x = y = t`, `t > 0` (fora do sólido).
    let (mut lo, mut hi) = (0.0f64, 2.0f64);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if f.at(mid, mid, 0.0) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    // A distância da quina é `t·√2`.
    0.5 * (lo + hi) * std::f64::consts::SQRT_2
}

fn worst_gradient(doc: &FieldDoc, e: f64, steps: usize) -> f64 {
    let f = Field::new(doc);
    let mut worst = 0.0f64;
    for i in 0..steps {
        for j in 0..steps {
            for k in 0..steps {
                let p = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / steps as f64;
                let g = f.gradient_norm(p(i), p(j), p(k), 1e-4);
                if g.is_finite() {
                    worst = worst.max(g);
                }
            }
        }
    }
    worst
}

/// **O RECUO ao longo da face** — até onde o carácter sobe a parede, medido da quina.
///
/// ⚠️ **É a régua da CONVENÇÃO**, e a outra ([`diagonal_crossing`]) é a régua da MORDIDA. As duas
/// são verdadeiras sobre grandezas diferentes, e foi por confundi-las que a nota antiga desta crate
/// disse «3/4»: um filete e um chanfro do mesmo número arrancam no **mesmo** sítio da parede e comem
/// o canto de maneiras diferentes — é isso que os torna dois caracteres.
///
/// As duas caixas ocupam `x ≤ 0` e `y ≤ 0`, então o canto na origem é **côncavo** e a mistura
/// **acrescenta** material no quadrante vazio. Para cada `y`, a superfície está em `x = 0` enquanto
/// o carácter não a alcançar, e em `x > 0` a partir daí.
fn face_setback(doc: &FieldDoc) -> f64 {
    let f = Field::new(doc);
    let surface_x = |y: f64| {
        let (mut lo, mut hi) = (-1.0f64, 2.0f64);
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if f.at(mid, y, 0.0) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    };
    // Desce pela parede até a superfície sair do plano `x = 0`.
    let (mut lo, mut hi) = (0.0f64, 3.0f64);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if surface_x(mid) > 1.0e-4 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// ⭐⭐⭐ **O QUE OS TRÊS CARACTERES PARTILHAM, E O QUE CADA UM NÃO PARTILHA** — medido, com a
/// divergência de cada um declarada.
///
/// # ⚠️ São DUAS réguas, e nenhum carácter bate as duas
///
/// | régua | o que mede | quem concorda com o filete |
/// |---|---|---|
/// | **recuo** ([`face_setback`]) | até onde sobe a parede | o **chanfro** (`1,00×`) · o orgânico **não** (`1,16×`) |
/// | **mordida** ([`diagonal_crossing`]) | onde fica a silhueta do canto | o **orgânico** (`1,00×`) · o chanfro **não** (`1,71×`) |
///
/// ⛔ **Foi confundir as duas que produziu a nota antiga desta crate** (*«o orgânico entrega 3/4»*):
/// ela media uma e falava da outra.
///
/// # ⭐ A escolha, e o motivo
///
/// A calibração do orgânico ([`ph2d_field::Blend::ORGANIC_REACH`]) é feita pela **MORDIDA**, porque
/// é a silhueta que o artista vê: trocar `Fillet` ↔ `Organic` com o mesmo número deixa o canto onde
/// está e muda só a **forma** da transição. ⇒ o recuo dele fica em `1,16×`, e isso é **divergência
/// declarada**, não defeito — um borrão derretido não tem linha de tangência nítida para alinhar.
///
/// ⛔ **E o chanfro NÃO se calibra:** um corte reto e um arco de mesmo recuo arrancam material
/// diferente no meio, e é exactamente essa diferença que o artista escolhe. Calibrá-lo daria quatro
/// chips com três formas.
#[test]
fn the_four_characters_measure_the_same_radius() {
    const R: f32 = 0.5;
    let base = corner(Blend::Exact { radius: R });
    let (exacto_recuo, exacto_mordida) = (face_setback(&base), diagonal_crossing(&base));
    println!("  carácter |   recuo | recuo/exacto |  mordida | mordida/exacto");
    let mut lido: Vec<(&str, f64, f64)> = Vec::new();
    for (nome, b) in [
        ("Exact", Blend::Exact { radius: R }),
        ("Chamfer", Blend::Chamfer { radius: R }),
        ("Organic", Blend::Organic { radius: R }),
    ] {
        let (recuo, mordida) = (face_setback(&corner(b)), diagonal_crossing(&corner(b)));
        println!(
            "{nome:>10} | {recuo:7.5} | {:12.4} | {mordida:8.5} | {:14.4}",
            recuo / exacto_recuo,
            mordida / exacto_mordida
        );
        lido.push((nome, recuo / exacto_recuo, mordida / exacto_mordida));
    }
    let ler = |nome: &str| -> (f64, f64) {
        let (_, r, m) = *lido.iter().find(|(n, ..)| *n == nome).expect("medido");
        (r, m)
    };

    // ── O CHANFRO partilha a CONVENÇÃO: arranca no mesmo sítio da parede ──
    let (chanfro_recuo, chanfro_mordida) = ler("Chamfer");
    assert!(
        (chanfro_recuo - 1.0).abs() < 0.03,
        "o chanfro recua {chanfro_recuo:.4}x o filete — o numero deixou de querer dizer a mesma \
         coisa nos dois chips, e trocar de caracter passa a mudar o tamanho da peca"
    );
    // ⭐ **E o CONTROLO que impede uma «calibração» de o apagar:** ele morde MAIS, por construção.
    assert!(
        chanfro_mordida > 1.3,
        "o chanfro morde {chanfro_mordida:.4}x o filete — abaixo disto ele deixou de ser um corte \
         reto e virou um segundo filete"
    );

    // ── O ORGÂNICO partilha a MORDIDA: a silhueta do canto fica no mesmo sítio ──
    let (organico_recuo, organico_mordida) = ler("Organic");
    assert!(
        (organico_mordida - 1.0).abs() < 0.02,
        "a calibracao do organico esta a {:.2} % — corrija `Blend::ORGANIC_REACH`",
        (organico_mordida - 1.0) * 100.0
    );
    // ⚠️ **A divergência DECLARADA**, com barra dos dois lados: ela não pode encolher (seria o
    // orgânico a virar filete) nem crescer (seria a calibração a escorregar).
    assert!(
        (1.10..=1.25).contains(&organico_recuo),
        "o recuo do organico leu {organico_recuo:.4}x — a divergencia declarada e' 1,16x, e ela e' \
         a consequencia de calibrar pela MORDIDA. Fora desta faixa, ou a constante mudou ou a \
         forma do operador mudou"
    );
}

/// ⭐⭐⭐ **EM QUE BALDE O CHANFRO CAI NA MARCHA** — e o erro aqui não fica lento, **fura**.
///
/// ⚠️ O `march_depth` conta **arredondamentos exactos** porque `‖∇f‖` chega a `√2` neles. Esta sonda
/// mede o mesmo número para o chanfro, e é ela que autoriza o balde escolhido.
#[test]
fn the_chamfer_is_measured_against_the_march() {
    println!("  carácter | ‖∇f‖ pior");
    let mut chanfro = 0.0;
    for (nome, b) in [
        ("Sharp", Blend::Sharp),
        ("Exact", Blend::Exact { radius: 0.5 }),
        ("Chamfer", Blend::Chamfer { radius: 0.5 }),
        ("Organic", Blend::Organic { radius: 0.5 }),
    ] {
        let g = worst_gradient(&corner(b), 1.0, 40);
        println!("{nome:>10} | {g:.4}");
        if nome == "Chamfer" {
            chanfro = g;
        }
    }
    // ⚠️ **A afirmação é sobre o BALDE, não sobre o número.** O `safe_march_step` trata o chanfro
    // como um arredondamento exacto; isso só é honesto enquanto ele não subir acima do `√2` que
    // aquele balde já paga.
    assert!(
        chanfro <= std::f64::consts::SQRT_2 + 0.05,
        "o chanfro lê ‖∇f‖ = {chanfro:.4}, acima do √2 do balde em que o `march_depth` o pôs — a \
         marcha passa a atravessar a superfície"
    );
}

/// ⭐⭐⭐ **O CHANFRO NUNCA MENTE PARA CIMA** — ele é sempre um minorante, e é disso que a marcha
/// depende.
///
/// # ⛔ Este gate nasceu de uma MUTAÇÃO QUE SOBREVIVEU
///
/// Apagar o `min` com a quina viva de `union_chamfer` **não mudou nenhum dos dois números** das
/// réguas acima. A causa é que as duas medem **no canto**, e ali o termo do corte já é o mínimo — o
/// `min` só protege a região **longe** do canto, onde o plano continua a descer e a superfície de
/// verdade é a face plana.
///
/// ⚠️ *Uma régua que só olha onde o fenómeno é forte não vê a guarda que o segura noutro sítio.* E
/// a propriedade estava **escrita no doc-comment do operador** — sem gate nenhum.
///
/// A afirmação é a que a marcha consome: `f ≤ min(a, b)` em toda parte, e `min(a, b)` é o campo da
/// união dura, que já é um minorante da distância. Um passo do tamanho do valor só é seguro por
/// causa disto.
#[test]
fn the_chamfer_never_overstates_the_distance() {
    const R: f32 = 0.5;
    let vivo = Field::new(&corner(Blend::Sharp));
    let chanfrado = Field::new(&corner(Blend::Chamfer { radius: R }));
    let mut pior = 0.0f64;
    let mut onde = [0.0f64; 3];
    // ⚠️ A janela vai muito além do alcance do chanfro **de propósito**: é longe do canto que o
    // `min` é a única coisa a segurar o plano.
    let n = 26;
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let p = |t: usize| -3.0 + 6.0 * (t as f64 + 0.5) / n as f64;
                let (x, y, z) = (p(i), p(j), p(k));
                let excesso = chanfrado.at(x, y, z) - vivo.at(x, y, z);
                if excesso.is_finite() && excesso > pior {
                    pior = excesso;
                    onde = [x, y, z];
                }
            }
        }
    }
    println!("  pior excesso sobre a união dura: {pior:.6} em {onde:?}");
    assert!(
        pior < 1.0e-4,
        "o chanfro devolveu {pior:.6} A MAIS que a união dura em {onde:?} — ele deixou de ser um \
         minorante, e a marcha passa a atravessar a superfície"
    );
}
