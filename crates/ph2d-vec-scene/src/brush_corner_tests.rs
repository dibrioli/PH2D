//! **Os gates das QUINAS** do pincel de contorno (plano 36, W5) — irmão do ficheiro de gates do
//! motor, separado quando ele passou o teto de LOC.

use super::fixtures::*;
use super::*;

// ─────────────────────────────────────────────────────────────────────────────────────────────
// W5 — **AS QUINAS**: a MEDIÇÃO do defeito, antes de qualquer desenho de cura.
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐ **O DESVIO de uma cópia** — a maior distância de um vértice dela à GUIA que ela devia
/// percorrer, em unidades de mundo.
///
/// ⚠️ **Régua LOCAL, por cópia**, e é de propósito: a lição que o quad-remesh desta casa pagou três
/// vezes é que *uma régua que resume o conjunto é cega ao caso que interessa* — aqui o defeito são
/// duas ou três cópias numa volta de dezenas, e uma mediana não se mexe.
fn desvio(copia: &VecPath, guia: &crate::arc_path::ArcPath) -> f64 {
    copia
        .verts
        .iter()
        .map(|v| {
            let s = guia.closest_arc(v.anchor);
            let (p, _) = guia.frame_at(s);
            (p[0] - v.anchor[0]).hypot(p[1] - v.anchor[1])
        })
        .fold(0.0, f64::max)
}

fn guia_de(p: &VecPath) -> crate::arc_path::ArcPath {
    crate::arc_path::ArcPath::from_contour(&p.verts, p.closed).expect("guia com comprimento")
}

/// ⭐⭐⭐ **QUANTO É QUE A QUINA PARTE O PINCEL** — a medição que abre a W5.
///
/// ⚠️⚠️ **A 1.ª fixtura desta sonda NÃO continha o fenómeno e leu o defeito como AUSENTE.** Com uma
/// arte de largura `1` num quadrado de lado `7`, o avanço encaixa em `1,0` e os centros caem em
/// `0,5 · 1,5 · …` — as quinas (`7 · 14 · 21 · 28`) caem **exactamente ENTRE** duas cópias, e
/// nenhuma atravessa uma. *A fixtura mais azarada possível é a que diz que está tudo bem.*
///
/// ⚠️⚠️ **E a 1.ª RÉGUA era cega ao defeito principal:** ela media o desvio das cópias **EMITIDAS**,
/// e o que uma quina faz hoje é **não emitir**. O `ArcPath` devolve tangente NULA numa cúspide, o
/// `GlyphFrame::on_path` devolve `None`, e o `pattern_along` **pula a cópia** — fica um BURACO.
/// *Uma régua que percorre o que existe não vê o que faltou.*
///
/// O CONTROLO é um círculo do MESMO perímetro com a MESMA arte: ele dá a contagem que a quina
/// deveria ter dado.
#[test]
#[ignore = "medicao: imprime a tabela da W5"]
fn measure_how_far_a_corner_throws_the_copies_off_the_guide() {
    let largura = 1.0;
    let b = pincel();
    let s = traco(&b, largura, None);
    let meia_altura = crate::brush_height(&b, largura) * 0.5;
    let raio = 28.0 / std::f64::consts::TAU;
    println!("\n  [plano 36 W5] meia-altura da arte = {meia_altura:.4} (o desvio 'de graca')");
    println!("  perimetro 28 nas duas formas; o CIRCULO da a contagem que a quina devia dar\n");

    for w in [1.0, 1.3, 1.7, 2.0] {
        let art = arte(w, 1.0);
        let quad = brush_along_path(&quadrado(7.0), &art, &s);
        let circ = brush_along_path(&crate::ellipse([0.0, 0.0], raio, raio), &art, &s);
        let guia = guia_de(&quadrado(7.0));
        let pior = quad
            .iter()
            .map(|c| desvio(c, &guia))
            .fold(0.0_f64, f64::max);
        let buracos = circ.len() as i64 - quad.len() as i64;
        println!(
            "  arte {w:>4.1}  ·  circulo {:>3} copias  ·  quadrado {:>3}  ⇒  BURACOS {buracos:>2}  \
             ·  desvio pior no quadrado {pior:.4} ({:.2}x a meia-altura)",
            circ.len(),
            quad.len(),
            pior / meia_altura
        );
    }

    // ⭐⭐ **O REGIME EM QUE DÓI** — a queixa nº 1 dos fóruns do Illustrator é *"apliquei o pincel a
    // um rectângulo pequeno e os lados sobrepõem-se nas quinas"*. A grandeza que manda não é o
    // tamanho da forma nem o da arte: é a RAZÃO entre eles.
    println!("\n  lado do quadrado / largura da arte  ->  o desvio, em multiplos da meia-altura");
    for lado in [2.0_f64, 3.0, 5.0, 7.0, 12.0, 20.0] {
        let art = arte(1.3, 1.0);
        let quad = quadrado(lado);
        let guia = guia_de(&quad);
        let copias = brush_along_path(&quad, &art, &s);
        if copias.is_empty() {
            println!("  lado {lado:>5.1}  ·  SEM COPIAS");
            continue;
        }
        // ⚠️ **O CÍRCULO NÃO É UM CONTROLO EXACTO** — o `total` dele é a medida Gauss-Legendre de
        // quatro cúbicas, não `2πr`, e um `total` ligeiramente diferente muda o `round` do encaixe
        // em `1`. A contagem esperada sai da LEI: `round(total / avanço nominal)`, que é o que o
        // `dash_fit` calcula.
        let esperadas = (guia.total() / 1.3).round().max(1.0) as usize;
        let pior = copias
            .iter()
            .map(|c| desvio(c, &guia))
            .fold(0.0_f64, f64::max);
        let acima: usize = copias
            .iter()
            .filter(|c| desvio(c, &guia) > meia_altura * 1.2)
            .count();
        println!(
            "  lado {lado:>5.1}  (lado/arte = {:>5.1})  esperadas {esperadas:>3}  emitidas {:>3}  \
             ⇒ BURACOS {:>2}  ·  desvio pior {:>5.2}x  ·  {acima} copia(s) acima de 1,2x",
            lado / 1.3,
            copias.len(),
            esperadas as i64 - copias.len() as i64,
            pior / meia_altura
        );
    }

    // ⚠️ **E o RESÍDUO: a tangente EXACTAMENTE em cima de cada quina.** Se ela existir, o buraco
    // que sobra não é cúspide — é outra coisa.
    println!("\n  tangente EXACTAMENTE na quina (o `t` que o `inv_arclen` devolve importa):");
    for lado in [2.0_f64, 12.0, 7.0] {
        let g = guia_de(&quadrado(lado));
        let mut nulas = 0;
        for q in 1..=4 {
            let arco = f64::from(q) * lado;
            let sarco = arco.rem_euclid(g.total());
            let (_, t) = g.frame_at(sarco);
            if t[0] == 0.0 && t[1] == 0.0 {
                nulas += 1;
                println!(
                    "      NULA em s={sarco:.17} (total={:.17}, starts={:?})",
                    g.total(),
                    g.anchor_arcs()
                );
            }
        }
        println!("  lado {lado:>5.1}  ·  quinas com tangente NULA: {nulas} de 4");
    }

    // ⚠️ **A CÚSPIDE é o mecanismo, e mede-se directamente**: quantas posições de arco de uma volta
    // do quadrado devolvem tangente NULA? Um contorno de 4 quinas tem 4.
    let guia = guia_de(&quadrado(7.0));
    let mut cuspides = 0;
    for k in 0..=4000 {
        let arco = f64::from(k) / 4000.0 * guia.total();
        let (_, t) = guia.frame_at(arco);
        if t[0] == 0.0 && t[1] == 0.0 {
            cuspides += 1;
        }
    }
    println!("\n  posicoes de arco com tangente NULA numa volta (4001 amostras): {cuspides}");
}

/// ⭐⭐⭐ **UM QUADRADO RECEBE TODAS AS CÓPIAS QUE O ENCAIXE PEDE** — zero buracos nas quinas.
///
/// ⚠️ **Era falso até 2026-08-30, e em TODO tamanho:** o `frame_at` devolvia tangente nula na quina
/// (`B'` anula-se nas duas pontas de um segmento reto autorado), o `GlyphFrame::on_path` devolvia
/// `None` e o `pattern_along` fazia `continue`. Medido: `1`–`2` cópias em falta por quadrado, de
/// lado `2` a `20` ([plano 36 §11](../../../docs/Vector%20Module/36_plano_pincel_de_contorno.md)).
///
/// ⚠️ **A contagem esperada sai da LEI, não de um círculo de igual perímetro.** O `total` de um
/// círculo é a quadratura de quatro cúbicas e não `2πr`; um `total` diferente por `1e-4` muda o
/// `round` do encaixe em `1`, e o «controlo» acusaria um buraco que não existe.
#[test]
fn a_square_gets_every_copy_the_fit_asks_for() {
    let b = pincel();
    let s = traco(&b, 1.0, None);
    // ⚠️ Uma arte cuja largura NÃO divide o perímetro: com `1,0` num lado de `7` as quinas caem
    // exactamente ENTRE duas cópias e a fixtura deixa de conter o fenómeno.
    let art = arte(1.3, 1.0);
    let mut medidos = 0;
    for lado in [2.0_f64, 3.0, 5.0, 7.0, 12.0, 20.0] {
        let forma = quadrado(lado);
        let guia = guia_de(&forma);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let esperadas = (guia.total() / 1.3).round().max(1.0) as usize;
        let emitidas = brush_along_path(&forma, &art, &s).len();
        assert_eq!(
            emitidas, esperadas,
            "o quadrado de lado {lado} recebeu {emitidas} copias e o encaixe pediu {esperadas} - \
             ha' buraco(s) na(s) quina(s)"
        );
        medidos += 1;
    }
    assert_eq!(medidos, 6, "a varredura encolheu");
    // ⚠️⚠️ **A metade que prova que a fixtura contém o fenómeno**: a quina de um contorno autorado
    // com vértices de quina é onde `B'` se anula. Se um dia ela deixar de se anular, este gate
    // continua verde **sem sujeito**, e é isso que esta linha impede.
    let v = quadrado(7.0).verts;
    assert!(
        v.iter()
            .all(|x| x.in_handle == x.anchor && x.out_handle == x.anchor),
        "o quadrado da fixtura deixou de ser autorado com VERTICES DE QUINA (alcas em cima da \
         ancora) - e' isso que faz `B'` anular-se nas pontas de cada segmento, e sem isso este \
         gate fica verde sem sujeito"
    );
}
