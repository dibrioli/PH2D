//! Os gates e as sondas da **régua da dobra** — irmão do [`super::fold`] pela lei da casa.

use super::fold;
use super::{Skin, SkinBone, Xform};

fn translate(tx: f64, ty: f64) -> Xform {
    Xform([1.0, 0.0, 0.0, 1.0, tx, ty])
}

fn rotate(a: f64) -> Xform {
    let (s, c) = a.sin_cos();
    Xform([c, s, -s, c, 0.0, 0.0])
}

/// ⭐ **Uma corrente de `n` ossos com comprimento de arco TOTAL fixo**, dobrada `total` radianos do
/// primeiro osso ao último, com raio de influência `raio`.
///
/// ⚠️ **O arco total é fixo de propósito:** é o que torna `n` uma variável independente. Com o
/// comprimento por osso fixo, subir `n` faria um braço mais comprido, e a comparação mediria duas
/// coisas ao mesmo tempo.
pub(crate) fn corrente(n: usize, arco: f64, total: f64, raio: f64) -> Skin {
    let l = arco / n as f64;
    let passo = if n > 1 { total / (n - 1) as f64 } else { 0.0 };
    let mut bones = Vec::with_capacity(n);
    let (mut ox, mut oy, mut fi) = (0.0f64, 0.0f64, 0.0f64);
    for i in 0..n {
        if i > 0 {
            fi += passo;
        }
        let pose = translate(-(i as f64) * l, 0.0)
            .then(&rotate(fi))
            .then(&translate(ox, oy));
        bones.push(SkinBone {
            rest_a: [i as f64 * l, 0.0],
            rest_b: [(i + 1) as f64 * l, 0.0],
            radius: raio,
            pose,
        });
        let (s, c) = fi.sin_cos();
        ox += l * c;
        oy += l * s;
    }
    Skin::new(bones).expect("a corrente tem ossos")
}

/// A caixa de arte à volta da corrente, em repouso — `alt` é a meia-altura em fracção do arco.
///
/// ⚠️ **A altura é uma VARIÁVEL, e descobri-lo custou uma medição inteira:** com arte fina
/// (`0,15`) a corrente de dois ossos a `150°` não inverte um único ponto, e a sonda leu o produto
/// como são. A dobra vive **longe do eixo**, onde o gradiente dos pesos é grande — uma caixa
/// estreita não tem lá pontos. *Uma fixtura que não produz o fenómeno mede outro programa.*
fn caixa(arco: f64, alt: f64) -> [f64; 4] {
    [-arco * 0.1, -arco * alt, arco * 1.1, arco * alt]
}

/// ⭐⭐⭐ **A SONDA DA DOBRA** — reproduz o defeito e mede as duas curas candidatas na mesma corrida.
#[test]
#[ignore]
fn probe_fold() {
    const ALT: f64 = ARTE;
    println!("== ONDE A DOENÇA VIVE: a ALTURA da arte (2 ossos, raio = comprimento do osso)");
    for alt in [0.15_f64, 0.25, 0.4, 0.6, 0.8] {
        for graus in [60.0_f64, 150.0] {
            let s = corrente(2, ARCO, graus.to_radians(), ARCO / 2.0);
            let r = fold::measure(&s, caixa(ARCO, alt), 160);
            println!(
                "  meia-altura {alt:4.2}× arco  dobra {graus:5.0}°  det_min {:9.4}  invertido {:6.2}%  orfas {:5.1}%",
                r.det_min,
                r.inverted * 100.0,
                r.orphan * 100.0
            );
        }
    }
    println!("\n== CURA A: o RAIO de influência (2 ossos, dobra 150°)");
    for m in [0.25_f64, 0.5, 1.0, 2.0, 4.0] {
        let s = corrente(2, ARCO, 150.0_f64.to_radians(), ARCO / 2.0 * m);
        let r = fold::measure(&s, caixa(ARCO, ALT), 160);
        println!(
            "  raio {m:4.2}× do osso  det_min {:8.4}  invertido {:6.2}%  orfas {:5.1}%",
            r.det_min,
            r.inverted * 100.0,
            r.orphan * 100.0
        );
    }
    println!(
        "\n== O RAIO CONTRA A ESPESSURA DA ARTE (2 ossos; meia-altura da arte = {:.2})",
        ARCO * ALT
    );
    for m in [1.0_f64, 1.5, 2.0, 2.5, 3.0, 4.0] {
        let raio = ARCO / 2.0 * m;
        let mut linha = format!(
            "  raio {:4.1} = {:4.2}× a meia-altura da arte |",
            raio,
            raio / (ARCO * ALT)
        );
        for graus in [60.0_f64, 90.0, 120.0, 150.0] {
            let s = corrente(2, ARCO, graus.to_radians(), raio);
            let r = fold::measure(&s, caixa(ARCO, ALT), 160);
            linha.push_str(&format!(
                " {graus:3.0}°: {:5.2}%/{:4.1}%orf |",
                r.inverted * 100.0,
                r.orphan * 100.0
            ));
        }
        println!("{linha}");
    }
    println!("\n== SUBDIVIDIR COM O RAIO JÁ CERTO (2,08× a meia-altura da arte), dobra 150°");
    for n in [2usize, 3, 4, 6, 8, 12, 16, 24] {
        let s = corrente(n, ARCO, 150.0_f64.to_radians(), 5.0);
        let r = fold::measure(&s, caixa(ARCO, ALT), 160);
        println!(
            "  n {n:3}  det_min {:8.4}  invertido {:6.2}%  orfas {:5.1}%",
            r.det_min,
            r.inverted * 100.0,
            r.orphan * 100.0
        );
    }
    println!("\n== CURA B: SUBDIVIDIR (o mecanismo do B-Bone) — mesmo arco, mesma dobra total");
    for n in [2usize, 3, 4, 6, 8, 12, 16, 24] {
        let l = ARCO / n as f64;
        for (nome, raio) in [("raio = sub-osso", l), ("raio = osso inteiro", ARCO / 2.0)] {
            let s = corrente(n, ARCO, 150.0_f64.to_radians(), raio);
            let r = fold::measure(&s, caixa(ARCO, ALT), 160);
            println!(
                "  n {n:3}  {nome:20}  det_min {:8.4}  invertido {:6.2}%  orfas {:5.1}%",
                r.det_min,
                r.inverted * 100.0,
                r.orphan * 100.0
            );
        }
    }
}

/// A meia-altura da arte das fixturas, em unidades do arco.
const ARTE: f64 = 0.6;
/// O arco das fixturas.
const ARCO: f64 = 4.0;

/// ⭐⭐⭐ **A RÉGUA VÊ A DOBRA, e a doença é o RAIO contra a ESPESSURA DA ARTE** — não a dobra
/// sozinha.
///
/// ⛔⛔ **Descobrir isto custou uma fixtura inteira:** com arte FINA (meia-altura `0,15 × arco`) a
/// corrente de dois ossos a `150°` não inverte **um único ponto**, e a sonda leu o produto como são.
/// A dobra vive **longe do eixo**, onde o gradiente dos pesos é grande — uma caixa estreita não tem
/// lá pontos. *Uma fixtura que não produz o fenómeno mede outro programa.*
///
/// ⚠️ A metade ANTI-VÁCUO é a 1.ª asserção: com pouca dobra a MESMA arte sai limpa, logo o que a
/// régua acusa é a dobra e não a fixtura.
#[test]
fn the_fold_ruler_sees_the_fold() {
    let raio = ARCO / 2.0; // o que o produto usa hoje: o raio é o comprimento do osso
    let manso = fold::measure(
        &corrente(2, ARCO, 20.0_f64.to_radians(), raio),
        caixa(ARCO, ARTE),
        160,
    );
    assert_eq!(
        manso.inverted, 0.0,
        "a fixtura inverte-se com a corrente quase recta: ela não mede a dobra, mede-se a si mesma"
    );
    let bravo = fold::measure(
        &corrente(2, ARCO, 150.0_f64.to_radians(), raio),
        caixa(ARCO, ARTE),
        160,
    );
    assert!(
        bravo.inverted > 0.01,
        "a `150°` a régua lê {:.4} de arte invertida — ela deixou de ver a dobra que o produto tem",
        bravo.inverted
    );
    assert!(
        bravo.det_min < 0.0,
        "o pior ponto tem determinante {:.4}: sem ele negativo não há arte do avesso",
        bravo.det_min
    );
}

/// ⭐⭐⭐ **O ALCANCE CURA A DOBRA, e o número é uma razão à ESPESSURA DA ARTE.**
///
/// Medido: com o raio a `2,08 ×` a meia-altura da arte, **zero** pontos invertidos em todas as
/// dobras até `150°`; a `0,83 ×` (o que o produto entrega hoje num membro típico) são `0,17 %` já a
/// `60°` e `2,61 %` a `150°`.
///
/// ⚠️ **E o sentido é o CONTRÁRIO da intuição:** apertar os pesos PIORA (`0,25 ×` do osso dá
/// `0,64 %` de inversão), porque o que dobra a arte é o **gradiente** dos pesos, e apertá-los
/// torna-o mais íngreme.
#[test]
fn a_wider_reach_cures_the_fold() {
    let meia_arte = ARCO * ARTE;
    for graus in [60.0_f64, 90.0, 120.0, 150.0] {
        let largo = fold::measure(
            &corrente(2, ARCO, graus.to_radians(), meia_arte * 2.08),
            caixa(ARCO, ARTE),
            160,
        );
        assert_eq!(
            largo.inverted, 0.0,
            "com o alcance a 2,08× a arte e a dobra a {graus}° ainda há {:.4} de arte do avesso",
            largo.inverted
        );
        assert_eq!(
            largo.orphan, 0.0,
            "com esse alcance nenhum ponto da arte podia ficar órfão"
        );
    }
    // E o estreito continua doente — senão a asserção de cima passaria sem o alcance fazer nada.
    let estreito = fold::measure(
        &corrente(2, ARCO, 150.0_f64.to_radians(), meia_arte * 0.83),
        caixa(ARCO, ARTE),
        160,
    );
    assert!(
        estreito.inverted > 0.01,
        "o alcance estreito deixou de produzir a dobra: este gate passaria sem medir nada"
    );
}

/// ⭐⭐⭐ **A COLUNA DE ÓRFÃS É A ANTI-VACUIDADE DA RÉGUA — e ela já apanhou uma cura falsa.**
///
/// ⛔⛔ Subdividir o osso encolhendo o raio com ele lê **`0,00 %` de arte invertida** a `24`
/// sub-ossos — e **`94,5 %` das amostras ficaram de fora por serem órfãs**. Não é uma cura: é a
/// régua a não medir quase nada. *Uma régua que exclui amostras tem de dizer quantas.*
#[test]
fn the_orphan_column_catches_a_ruler_measuring_nothing() {
    let n = 24;
    let falsa = fold::measure(
        &corrente(n, ARCO, 150.0_f64.to_radians(), ARCO / n as f64),
        caixa(ARCO, ARTE),
        160,
    );
    assert_eq!(
        falsa.inverted, 0.0,
        "a fixtura da cura FALSA mudou: ela existe porque lê zero inversão"
    );
    assert!(
        falsa.orphan > 0.9,
        "ela lia `0 %` de inversão com {:.1} % de órfãs — sem essa fracção alta este gate não \
         demonstra o perigo",
        falsa.orphan * 100.0
    );
}

/// ⭐⭐⭐ **A MESMA PELE DEZ VEZES MAIOR DÁ O MESMO RELATÓRIO** — a régua mede a FORMA da dobra, não
/// o tamanho do desenho.
///
/// ⚠️ **É isto que obriga o passo da diferença central a ser uma fracção da CÉLULA**, e não um
/// epsilon em unidades do documento. Com um epsilon absoluto a mesma lei lê-se diferente num rig
/// desenhado noutra escala — e nenhum dos outros gates o vê, porque todos correm numa escala só.
/// ⛔ Uma mutação que o troque por `1e-9` sobrevive a eles; é aqui que ela morre.
///
/// ⚠️ A fracção invertida é **exactamente** igual (a grelha amostra os mesmos pontos relativos); o
/// determinante mínimo escala com o quadrado do tamanho, então compara-se **normalizado**.
#[test]
fn the_fold_ruler_reads_the_same_at_any_scale() {
    let escala = 10.0;
    let a = fold::measure(
        &corrente(2, ARCO, 150.0_f64.to_radians(), ARCO / 2.0),
        caixa(ARCO, ARTE),
        160,
    );
    let b = fold::measure(
        &corrente(
            2,
            ARCO * escala,
            150.0_f64.to_radians(),
            ARCO * escala / 2.0,
        ),
        caixa(ARCO * escala, ARTE),
        160,
    );
    assert_eq!(
        a.inverted, b.inverted,
        "a mesma pele {escala}× maior inverte outra fracção da arte: a régua mede o TAMANHO"
    );
    assert_eq!(a.orphan, b.orphan, "e a fracção órfã também mudou");
    let normalizado = b.det_min;
    assert!(
        (normalizado - a.det_min).abs() < 1e-6 * a.det_min.abs().max(1.0),
        "o determinante mínimo mudou com a escala ({} contra {}) — ele é adimensional por \
         construção (é uma razão de áreas)",
        a.det_min,
        normalizado
    );
}
