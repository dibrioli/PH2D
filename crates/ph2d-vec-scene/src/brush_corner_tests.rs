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
        let quad = brush_along_path(&quadrado(7.0), std::slice::from_ref(&art), &s);
        let circ = brush_along_path(
            &crate::ellipse([0.0, 0.0], raio, raio),
            std::slice::from_ref(&art),
            &s,
        );
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
        let copias = brush_along_path(&quad, &[art], &s);
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
        // ⭐ **A contagem esperada é por TRECHO** (W5): o avanço encaixa na PEÇA, e uma peça é um
        // trecho entre duas quinas. ⚠️ Até 30/08 esta linha era `round(total / 1,3)` — a lei
        // GLOBAL —, e ela deixou de descrever o produto no dia em que a quina passou a cortar.
        let mut cortes = guia.corner_arcs(1.0_f64.to_radians());
        assert_eq!(cortes.len(), 4, "um quadrado tem quatro quinas");
        cortes.push(cortes[0] + guia.total());
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let esperadas: usize = cortes
            .windows(2)
            .map(|w| (w[1] - w[0]) / 1.3)
            .map(|n| n.round().max(1.0) as usize)
            .sum();
        let emitidas = brush_along_path(&forma, std::slice::from_ref(&art), &s).len();
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

// ─────────────────────────────────────────────────────────────────────────────────────────────
// W5 metade B — **a quina VERDADEIRA**. A régua primeiro: as duas anteriores nasceram tortas.
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// A distância de `p` ao segmento `[a, b]`.
fn dist_ao_segmento(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let n2 = dx.mul_add(dx, dy * dy);
    let t = if n2 <= 0.0 {
        0.0
    } else {
        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / n2).clamp(0.0, 1.0)
    };
    (a[0] + t * dx - p[0]).hypot(a[1] + t * dy - p[1])
}

/// ⭐⭐⭐ **O DESVIO DE COBERTURA de uma fatia** — o quanto a guia que a cópia recebeu se afasta da
/// **espinha RÍGIDA** que a cópia deita sobre ela.
///
/// Uma cópia ocupa a fatia de arco `[s0, s1]` e é colocada por **UM** referencial, o do centro.
/// A espinha dela é o segmento `p ± t·L/2` — reta, por construção. A pergunta é: *a guia daquela
/// fatia cabe nessa reta?*
///
/// | regime | o que esta régua dá |
/// |---|---|
/// | trecho reto | `0` |
/// | curva suave | a flecha da corda, `≈ L²/8R` — pequena e conhecida |
/// | **quina** | `≈ (L/2)·sin(viragem/2)` — a profundidade do canto |
///
/// ⚠️⚠️ **É a TERCEIRA régua desta wave, e as duas anteriores nasceram tortas.** A primeira media
/// as cópias EMITIDAS e o defeito era uma cópia que **não é emitida**. A segunda (`desvio`) mede a
/// distância ao ponto **mais próximo da guia INTEIRA** — e numa quina o lado perpendicular está
/// logo ali, então uma cópia que saltou para o outro lado do canto lê `1,00×`. *A pergunta não é
/// «a arte está perto do desenho?», é «a arte cobre O PEDAÇO que lhe deram?»*
fn desvio_de_cobertura(guia: &crate::arc_path::ArcPath, s0: f64, s1: f64) -> f64 {
    let sc = 0.5 * (s0 + s1);
    let meio = (s1 - s0) * 0.5;
    let (p, t) = guia.frame_at(sc);
    if t[0] == 0.0 && t[1] == 0.0 {
        return f64::INFINITY;
    }
    let a = [t[0].mul_add(-meio, p[0]), t[1].mul_add(-meio, p[1])];
    let b = [t[0].mul_add(meio, p[0]), t[1].mul_add(meio, p[1])];
    let n = 64;
    (0..=n)
        .map(|k| {
            let s = s0 + (s1 - s0) * f64::from(k) / f64::from(n);
            let (q, _) = guia.frame_at(s);
            dist_ao_segmento(q, a, b)
        })
        .fold(0.0, f64::max)
}

/// ⭐⭐⭐ **QUANTO A QUINA TIRA DA COBERTURA** — a medição que abre a metade B.
#[test]
#[ignore = "medicao: imprime a tabela da W5 metade B"]
fn measure_what_a_corner_costs_in_coverage() {
    let b = pincel();
    let s = traco(&b, 1.0, None);
    let art = arte(1.3, 1.0);
    let meia_altura = crate::brush_height(&b, 1.0) * 0.5;
    println!("\n  [plano 36 W5-B] meia-altura da arte = {meia_altura:.4}");
    println!("  desvio de COBERTURA por fatia, em multiplos da meia-altura\n");

    for (nome, forma) in [
        ("quadrado 7x7", quadrado(7.0)),
        (
            "circulo (controlo)",
            crate::ellipse(
                [0.0, 0.0],
                28.0 / std::f64::consts::TAU,
                28.0 / std::f64::consts::TAU,
            ),
        ),
    ] {
        let guia = guia_de(&forma);
        let copias = brush_along_path(&forma, std::slice::from_ref(&art), &s);
        #[allow(clippy::cast_precision_loss)]
        let avanco = guia.total() / copias.len() as f64;
        let mut piores: Vec<f64> = (0..copias.len())
            .map(|k| {
                #[allow(clippy::cast_precision_loss)]
                let s0 = avanco * k as f64;
                desvio_de_cobertura(&guia, s0, s0 + avanco) / meia_altura
            })
            .collect();
        let pior = piores.iter().copied().fold(0.0, f64::max);
        piores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mediana = piores[piores.len() / 2];
        let feias = piores.iter().filter(|d| **d > 0.25).count();
        println!(
            "  {nome:>20}  fatias {:>3}  avanco {avanco:.3}  cobertura: mediana {mediana:.4}  \
             PIOR {pior:.4}  ·  {feias} fatia(s) acima de 0,25",
            copias.len()
        );
    }
}

/// As posições de arco onde o contorno **VIRA**, e quanto — `(arco, graus)`.
///
/// ⚠️ Sonda de MEDIÇÃO: a viragem é lida um `δ` para cada lado da âncora. No produto ela sairá
/// das tangentes exactas dos dois segmentos, sem `δ` nenhum — mas para escolher a lei o que
/// interessa é o número, e ele não se move com o `δ`.
fn quinas(guia: &crate::arc_path::ArcPath, limiar_graus: f64) -> Vec<(f64, f64)> {
    let total = guia.total();
    let d = total * 1e-4;
    let ang = |s: f64| {
        let (_, t) = guia.frame_at(s.rem_euclid(total));
        t[1].atan2(t[0])
    };
    guia.anchor_arcs()
        .iter()
        .filter_map(|&a| {
            let mut g = (ang(a + d) - ang(a - d)).abs().to_degrees();
            if g > 180.0 {
                g = 360.0 - g;
            }
            (g >= limiar_graus).then_some((a, g))
        })
        .collect()
}

/// As fatias da política **A — HOJE**: um avanço para a volta inteira.
fn fatias_a(total: f64, avanco: f64) -> Vec<(f64, f64)> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = (total / avanco).round().max(1.0) as usize;
    #[allow(clippy::cast_precision_loss)]
    let a = total / n as f64;
    #[allow(clippy::cast_precision_loss)]
    (0..n).map(|k| (a * k as f64, a * (k + 1) as f64)).collect()
}

/// As fatias da política **B — QUEBRAR NAS QUINAS**: cada trecho entre duas quinas encaixa o
/// próprio número inteiro de cópias. É o *Auto-Between* do Illustrator: as cópias entram até à
/// quina, uma de cada lado, e a emenda vê-se.
fn fatias_b(guia: &crate::arc_path::ArcPath, avanco_nominal: f64) -> Vec<(f64, f64)> {
    let total = guia.total();
    let mut cortes: Vec<f64> = quinas(guia, 5.0).into_iter().map(|(a, _)| a).collect();
    if cortes.is_empty() {
        return fatias_a(total, avanco_nominal);
    }
    cortes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    cortes.push(cortes[0] + total);
    let mut out = Vec::new();
    for w in cortes.windows(2) {
        let (c0, c1) = (w[0], w[1]);
        let len = c1 - c0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = (len / avanco_nominal).round().max(1.0) as usize;
        #[allow(clippy::cast_precision_loss)]
        let a = len / n as f64;
        for k in 0..n {
            #[allow(clippy::cast_precision_loss)]
            out.push((c0 + a * k as f64, c0 + a * (k + 1) as f64));
        }
    }
    out
}

/// ⭐⭐⭐ **AS POLÍTICAS DE QUINA, MEDIDAS LADO A LADO** — a tabela que o plano 36 §11.3 prometeu.
#[test]
#[ignore = "medicao: a tabela das politicas de quina"]
fn measure_the_corner_policies_side_by_side() {
    let b = pincel();
    let meia_altura = crate::brush_height(&b, 1.0) * 0.5;
    let nominal = 1.3;
    println!("\n  [plano 36 W5-B] desvio de COBERTURA (multiplos da meia-altura {meia_altura:.2})");
    println!("  A = hoje (um avanco para a volta)   ·   B = quebrar nas quinas (Auto-Between)\n");

    let estrela = crate::star([0.0, 0.0], 6.0, 6.0, 5, 0.45);
    for (nome, forma) in [
        ("quadrado 7x7", quadrado(7.0)),
        ("retangulo 12x4", crate::rectangle([0.0, 0.0], [12.0, 4.0])),
        ("estrela 5 pontas", estrela),
    ] {
        let guia = guia_de(&forma);
        let qs = quinas(&guia, 5.0);
        // ⭐ **B'** — quebrar nas quinas MAS manter o avanço GLOBAL: cada trecho leva
        // `floor(len/a)` cópias inteiras e sobra um resto NA QUINA. É a variante que preserva o
        // ritmo único do W3-bis, e o que ela custa é o VÃO — medido na coluna `resto`.
        let global = {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let n = (guia.total() / nominal).round().max(1.0) as usize;
            #[allow(clippy::cast_precision_loss)]
            {
                guia.total() / n as f64
            }
        };
        let mut b_linha = Vec::new();
        let mut resto_pior = 0.0_f64;
        {
            let mut cortes: Vec<f64> = qs.iter().map(|&(a, _)| a).collect();
            cortes.sort_by(|a, b| a.partial_cmp(b).unwrap());
            if !cortes.is_empty() {
                cortes.push(cortes[0] + guia.total());
                for w in cortes.windows(2) {
                    let len = w[1] - w[0];
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let n = (len / global).floor().max(0.0) as usize;
                    #[allow(clippy::cast_precision_loss)]
                    let resto = len - global * n as f64;
                    resto_pior = resto_pior.max(resto / meia_altura);
                    for k in 0..n {
                        #[allow(clippy::cast_precision_loss)]
                        b_linha.push((w[0] + global * k as f64, w[0] + global * (k + 1) as f64));
                    }
                }
            }
        }
        for (rotulo, fatias) in [
            ("A hoje ", fatias_a(guia.total(), nominal)),
            ("B quina", fatias_b(&guia, nominal)),
            ("B' glob", b_linha),
        ] {
            let mut d: Vec<f64> = fatias
                .iter()
                .map(|&(s0, s1)| desvio_de_cobertura(&guia, s0, s1) / meia_altura)
                .collect();
            let pior = d.iter().copied().fold(0.0, f64::max);
            d.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let feias = d.iter().filter(|x| **x > 0.25).count();
            println!(
                "  {nome:>17} {rotulo}  fatias {:>3}  mediana {:.4}  PIOR {pior:>7.4}  ·  \
                 {feias} acima de 0,25   ·  vao pior na quina {:.3}   (quinas: {})",
                d.len(),
                d[d.len() / 2],
                if rotulo == "B\' glob" {
                    resto_pior
                } else {
                    0.0
                },
                qs.len()
            );
        }
        println!();
    }
}

/// ⚠️ **DE QUE TAMANHO É UMA VIRAGEM QUE NÃO É QUINA** — o número que escolhe o limiar.
#[test]
#[ignore = "medicao: o limiar de quina"]
fn measure_the_turn_at_a_smooth_anchor_against_a_real_corner() {
    let estrela = crate::star([0.0, 0.0], 6.0, 6.0, 5, 0.45);
    for (nome, forma, fechado) in [
        (
            "circulo (suave)",
            crate::ellipse([0.0, 0.0], 4.0, 4.0),
            true,
        ),
        (
            "elipse 3:1 (suave)",
            crate::ellipse([0.0, 0.0], 9.0, 3.0),
            true,
        ),
        ("quadrado", quadrado(7.0), true),
        ("estrela", estrela, true),
    ] {
        let g = crate::arc_path::ArcPath::from_contour(&forma.verts, fechado).expect("guia");
        // O limiar a ZERO devolve toda âncora cuja viragem se meça; o de 1 grau devolve as quinas.
        let todas = g.corner_arcs(0.0).len();
        let acima_de_1 = g.corner_arcs(1.0_f64.to_radians()).len();
        let acima_de_10 = g.corner_arcs(10.0_f64.to_radians()).len();
        // O menor limiar (em graus) que ainda apanha alguma âncora — a viragem máxima da forma.
        let mut lo = 1e-12_f64;
        for _ in 0..80 {
            let m = (lo * 1.5).min(180.0);
            if g.corner_arcs(m.to_radians()).is_empty() {
                break;
            }
            lo = m;
        }
        println!(
            "  {nome:>20}  ancoras com viragem >= 0: {todas:>2}  ·  >= 1 grau: {acima_de_1:>2}  \
             ·  >= 10 graus: {acima_de_10:>2}  ·  viragem MAXIMA ~ {lo:.3e} graus"
        );
    }
}

/// ⭐⭐⭐ **A ARTE NÃO CORTA A QUINA** — nenhuma cópia recebe um pedaço de caminho que a espinha
/// rígida dela não cobre.
///
/// A barra é `0,25` da meia-altura, e ela sai da medição, não de gosto: uma curva **suave** custa
/// `0,091` (a flecha da corda, o preço inevitável de uma cópia rígida), e a quina custava
/// `0,92`–`1,28`. *Vinte e cinco centésimos ficam três vezes acima do inevitável e quatro vezes
/// abaixo do defeito.*
#[test]
fn the_art_never_cuts_a_corner() {
    let b = pincel();
    let s = traco(&b, 1.0, None);
    let art = arte(1.3, 1.0);
    let meia_altura = crate::brush_height(&b, 1.0) * 0.5;
    let estrela = crate::star([0.0, 0.0], 6.0, 6.0, 5, 0.45);
    let mut vistas = 0;
    for (nome, forma) in [
        ("quadrado", quadrado(7.0)),
        ("retangulo", crate::rectangle([0.0, 0.0], [12.0, 4.0])),
        ("estrela", estrela),
    ] {
        let guia = guia_de(&forma);
        let copias = brush_along_path(&forma, std::slice::from_ref(&art), &s);
        assert!(copias.len() > 8, "{nome}: so' {} copias", copias.len());
        // As fatias que o produto de facto usou: as peças entre quinas, cada uma com o avanço
        // dela. Reconstruí-las aqui é a única forma de medir a MESMA divisão que o motor fez.
        let mut cortes = guia.corner_arcs(1.0_f64.to_radians());
        assert!(cortes.len() >= 4, "{nome}: {} quinas", cortes.len());
        cortes.push(cortes[0] + guia.total());
        for w in cortes.windows(2) {
            let len = w[1] - w[0];
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let n = (len / 1.3).round().max(1.0) as usize;
            #[allow(clippy::cast_precision_loss)]
            let a = len / n as f64;
            for k in 0..n {
                #[allow(clippy::cast_precision_loss)]
                let s0 = w[0] + a * k as f64;
                let d = desvio_de_cobertura(&guia, s0, s0 + a) / meia_altura;
                assert!(
                    d <= 0.25,
                    "{nome}: a fatia [{s0:.3}, {:.3}] tem desvio de cobertura {d:.4} - a arte \
                     corta a quina",
                    s0 + a
                );
                vistas += 1;
            }
        }
    }
    assert!(vistas >= 60, "a varredura encolheu para {vistas} fatias");
    // ⚠️⚠️ **A metade que prova que a fixtura CONTÉM o fenómeno**: com um avanço ÚNICO para a volta
    // inteira — a lei de antes da W5 — as mesmas formas reprovam. Sem esta metade o gate ficaria
    // verde sobre um caminho que nunca teve quina.
    let guia = guia_de(&quadrado(7.0));
    let antes = fatias_a(guia.total(), 1.3)
        .into_iter()
        .map(|(s0, s1)| desvio_de_cobertura(&guia, s0, s1) / meia_altura)
        .fold(0.0_f64, f64::max);
    assert!(
        antes > 1.0,
        "com o avanco GLOBAL o quadrado tinha de cortar a quina, e mediu {antes:.4} - a fixtura \
         deixou de conter o fenomeno"
    );
}

/// ⭐⭐ **UM VÉRTICE SUAVE NÃO É UMA QUINA** — e um de polígono é.
///
/// A separação é de **treze ordens de grandeza** (medida em
/// `measure_the_turn_at_a_smooth_anchor_against_a_real_corner`): num círculo e numa elipse `3:1` a
/// viragem máxima de uma âncora é `≤ 1e-12°`, e num quadrado é `~90°`. O limiar de `1°` fica no
/// meio dessa vala.
#[test]
fn a_smooth_vertex_is_not_a_corner() {
    let limiar = 1.0_f64.to_radians();
    let estrela = crate::star([0.0, 0.0], 6.0, 6.0, 5, 0.45);
    for (nome, forma, esperadas) in [
        ("circulo", crate::ellipse([0.0, 0.0], 4.0, 4.0), 0),
        ("elipse 3:1", crate::ellipse([0.0, 0.0], 9.0, 3.0), 0),
        ("quadrado", quadrado(7.0), 4),
        ("estrela 5 pontas", estrela, 10),
    ] {
        let g = guia_de(&forma);
        assert_eq!(
            g.corner_arcs(limiar).len(),
            esperadas,
            "{nome}: contagem de quinas errada"
        );
        // ⚠️ **A metade que dá sujeito**: a forma TEM âncoras: se a resposta fosse zero por não
        // haver nada que medir, o caso suave passaria por vacuidade.
        assert!(
            g.anchor_arcs().len() >= 4,
            "{nome}: a fixtura nao tem ancoras que se contem"
        );
    }
}

/// ⭐⭐⭐ **A GUIA FICA COBERTA PELAS CÓPIAS QUE O PRODUTO EMITIU** — a régua sobre a SAÍDA, e não
/// sobre uma divisão que o teste refaz.
///
/// ⚠️⚠️ **A 1.ª redacção deste gate era CEGA à mutação que importa.** Ela reconstruía as fatias a
/// partir do `corner_arcs` e media o desvio delas — e a mutação que **desliga o corte no produto**
/// **SOBREVIVEU**, porque o teste continuava a medir a sua própria divisão. *Um gate que compara
/// duas construções é cego à mutação partilhada; a lei pede a SAÍDA.*
///
/// A régua: para cada ponto da guia, a distância à **espinha** da cópia mais próxima — a espinha é
/// o eixo longo da arte, os dois vértices extremos dela, que é exactamente o que a colocação
/// rígida deita sobre o caminho.
#[test]
fn the_guide_is_covered_by_the_copies_the_product_emitted() {
    let b = pincel();
    let s = traco(&b, 1.0, None);
    let art = arte(1.3, 1.0);
    let meia_altura = crate::brush_height(&b, 1.0) * 0.5;
    let estrela = crate::star([0.0, 0.0], 6.0, 6.0, 5, 0.45);
    for (nome, forma, barra) in [
        ("quadrado", quadrado(7.0), 0.25),
        ("retangulo", crate::rectangle([0.0, 0.0], [12.0, 4.0]), 0.25),
        ("estrela", estrela, 0.25),
        // ⚠️ O CONTROLO suave: uma curva paga a flecha da corda, e é ela que fixa o piso.
        ("circulo", crate::ellipse([0.0, 0.0], 4.456, 4.456), 0.25),
    ] {
        let guia = guia_de(&forma);
        let copias = brush_along_path(&forma, std::slice::from_ref(&art), &s);
        assert!(copias.len() > 8, "{nome}: so' {} copias", copias.len());
        // A espinha de cada cópia: os dois vértices extremos do losango (índices 0 e 2 do
        // `arte`), que a colocação rígida leva ao mundo.
        let espinhas: Vec<([f64; 2], [f64; 2])> = copias
            .iter()
            .map(|c| (c.verts[0].anchor, c.verts[2].anchor))
            .collect();
        let n = 900;
        let pior = (0..=n)
            .map(|k| {
                let (q, _) = guia.frame_at(f64::from(k) / f64::from(n) * guia.total());
                espinhas
                    .iter()
                    .map(|&(a, b)| dist_ao_segmento(q, a, b))
                    .fold(f64::MAX, f64::min)
            })
            .fold(0.0_f64, f64::max)
            / meia_altura;
        assert!(
            pior <= barra,
            "{nome}: o ponto pior da guia esta' a {pior:.4} da espinha mais proxima (barra \
             {barra}) - a arte deixou um pedaco de caminho descoberto"
        );
    }
}
