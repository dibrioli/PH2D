//! **QUAL DOS DOIS ADVECTS ESTÁ CERTO** — a pergunta que a medição de
//! divergência não responde (doc 28 §5.45).
//!
//! O [`advect`](ph2d_wet_paint::solver::advect) serial é um **Gauss-Seidel**:
//! ele varre em ordem de raster e SUBTRAI nos cantos-fonte, então a célula
//! `x+1` encontra o canto que a célula `x` já drenou. Isso não é uma escolha
//! de física — é uma consequência de **em que ordem o laço anda**, e uma
//! consequência de ordem tem assinatura observável: **ela quebra a simetria
//! que a cena tem.**
//!
//! A fixture é simétrica em `x` por construção — massa espelhada, fluxo
//! **antissimétrico** (`u(espelho) = −u(x)`), gravidade fora do caminho. A
//! física dessa cena é simétrica, logo o resultado tem de ser. O que a
//! varredura faz é ler ORIGINAIS de um lado e valores JÁ ESCRITOS do outro.
//!
//! ⚠️ **Isto não afirma que o gather é "melhor" por gosto** — afirma que ele é
//! **invariante à ordem**, e que o serial não é. O quanto isso importa numa
//! poça real é decisão de produto (o smoke); o que este arquivo pina é que a
//! diferença tem uma DIREÇÃO, e ela aponta para o gather.

use ph2d_wet_paint::drying::{drying_pass, drying_pass_jacobi};
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::sim::{Params, Sim};
use ph2d_wet_paint::solver::{advect, advect_jacobi, rebuild_active_region};
use ph2d_wet_paint::tuning::Tuning;

const W: usize = 96;
const H: usize = 24;

/// Uma faixa de tinta simétrica em `x`, sob um fluxo **antissimétrico**
/// (divergente a partir do centro) e sem componente vertical.
///
/// Sem `y` no fluxo o problema é 1-D por linha, e é exatamente aí que a
/// varredura esquerda→direita fica visível: à esquerda do centro o destino
/// puxa da DIREITA (célula ainda não visitada, valor original), à direita ele
/// puxa da ESQUERDA (célula já visitada, valor reescrito).
fn symmetric_sheet() -> Grid {
    let mut g = Grid::new(W, H);
    let s = g.s;
    let cx = (W as f64 + 1.0) / 2.0; // o eixo do espelho, entre as colunas
    for y in 1..=H {
        for x in 1..=W {
            let i = x + y * s;
            let d = (x as f64 - cx).abs();
            // Um perfil com ESTRUTURA (um platô com dentes): um campo chato
            // seria simétrico sob qualquer coisa, e o gate ficaria verde por
            // vácuo.
            let bump = if d < 24.0 { 1.0 - d / 24.0 } else { 0.0 };
            // ⚠️ Os dentes medem a DISTÂNCIA AO EIXO, nunca `x`: `x % 7` não é
            // simétrico sob `x → W+1−x`, e o gate de controle acima pegou
            // exatamente isso na primeira corrida (erro de espelho 484,7 numa
            // fixture que eu tinha declarado simétrica).
            let teeth = if (d.floor() as i32) % 7 < 3 {
                1.35
            } else {
                0.8
            };
            let m = 900.0 * bump * teeth;
            g.film[i] = (0.9 * bump * teeth) as f32;
            g.susp[i] = m as f32;
            g.susp_rgb[i] = [200.0, 40.0, 90.0];
            // Fluxo DIVERGENTE: negativo à esquerda, positivo à direita,
            // exatamente espelhado. `flow_y = 0` mantém o problema 1-D.
            g.flow_x[i] = (0.55 * (x as f64 - cx) / 24.0).clamp(-0.55, 0.55) as f32;
            g.flow_y[i] = 0.0;
        }
    }
    g.expand_bbox(1, 1, W as i32, H as i32);
    rebuild_active_region(&mut g);
    assert!(g.has_fluid, "a fixture tem de ter agua");
    g
}

fn params() -> Params {
    Sim::default().gather_params(&Tuning::default())
}

/// O pior desvio de espelho do plano, em unidades de massa.
fn mirror_error(g: &Grid) -> f64 {
    let s = g.s;
    let mut worst = 0.0f64;
    for y in 1..=H {
        for x in 1..=W / 2 {
            let a = f64::from(g.susp[x + y * s]);
            let b = f64::from(g.susp[(W + 1 - x) + y * s]);
            let d = (a - b).abs();
            if d > worst {
                worst = d;
            }
        }
    }
    worst
}

/// A fixture É simétrica antes de qualquer passe — o **controle**, sem o qual
/// o gate mede a minha aritmética de fixture em vez do solver.
#[test]
fn the_fixture_starts_perfectly_mirrored() {
    let g = symmetric_sheet();
    assert_eq!(mirror_error(&g), 0.0, "a fixture nasceu torta");
}

/// **O SERIAL QUEBRA A SIMETRIA DA CENA; O GATHER NÃO.**
///
/// Mutação: apontar as duas rotas para o mesmo `advect` faz os dois números
/// coincidirem e o gate morre — que é o ponto, porque é a DIFERENÇA entre
/// eles que carrega a afirmação.
#[test]
fn the_gauss_seidel_advect_leans_left_to_right_and_the_gather_does_not() {
    let p = params();
    let (mut a, mut b) = (symmetric_sheet(), symmetric_sheet());
    for _ in 0..12 {
        advect(&mut a, &p, 0.0, 0.0);
        advect_jacobi(&mut b, &p, 0.0, 0.0);
    }
    let (serial, gather) = (mirror_error(&a), mirror_error(&b));
    println!("  espelho: serial {serial:.4}  gather {gather:.6}");
    // O gather é invariante à ordem por CONSTRUÇÃO — todo mundo lê o estado do
    // início do passe —, então o desvio dele é ruído de `f32`, não viés.
    assert!(
        gather < 1e-2,
        "o gather deveria preservar a simetria da cena, e desviou {gather}"
    );
    // E o serial tem de estar ORDENS de grandeza acima, senão a fixture não
    // contém o fenômeno.
    assert!(
        serial > gather * 100.0,
        "a fixture nao expoe o vies da varredura: serial {serial}, gather {gather}"
    );
}

/// **A massa é conservada nos dois** — o gather não compra a simetria
/// inventando ou perdendo tinta.
///
/// ⚠️ Este é o gate que torna o de cima honesto: um advect que simplesmente
/// não move nada também preservaria a simetria.
#[test]
fn the_gather_conserves_mass_and_actually_moves_it() {
    let p = params();
    let before = symmetric_sheet();
    let m0: f64 = before.susp.iter().map(|&v| f64::from(v)).sum();
    let mut g = symmetric_sheet();
    for _ in 0..12 {
        advect_jacobi(&mut g, &p, 0.0, 0.0);
    }
    let m1: f64 = g.susp.iter().map(|&v| f64::from(v)).sum();
    let moved: f64 = g
        .susp
        .iter()
        .zip(before.susp.iter())
        .map(|(&a, &b)| f64::from(a - b).abs())
        .sum();
    println!("  massa {m0:.3} -> {m1:.3}   |movida| {moved:.1}");
    assert!(
        (m1 - m0).abs() / m0 < 1e-4,
        "o gather nao conservou a massa: {m0} -> {m1}"
    );
    assert!(
        moved > m0 * 0.01,
        "o advect nao moveu nada — o gate da simetria seria verde por vacuo"
    );
}

/// **A SECAGEM TEM A MESMA DOENÇA, E A MESMA CURA.**
///
/// O fator de borda conta quantos dos nove vizinhos carregam pigmento, e o
/// passe **escreve `susp`**: varrendo da esquerda para a direita, a coluna
/// `x−1` já foi secada quando `x` a conta, e a coluna `x+1` ainda não. Numa
/// folha espelhada isso não pode acontecer — os dois lados são a mesma cena.
///
/// ⚠️ Aqui o fluxo é irrelevante (a secagem não o lê); o que a fixture precisa
/// é de um **GRADIENTE de pigmento**, porque é sobre a fronteira do banho que o
/// fator de borda decide alguma coisa. Sem ele, todo mundo conta 9 e o gate
/// fica verde por vácuo — daí o gate de controle abaixo.
#[test]
fn the_gauss_seidel_drying_leans_left_to_right_and_the_jacobi_does_not() {
    let p = params();
    let (mut a, mut b) = (symmetric_sheet(), symmetric_sheet());
    for _ in 0..8 {
        drying_pass(&mut a, &p, 0.02, 0.0002, false);
        drying_pass_jacobi(&mut b, &p, 0.02, 0.0002, false);
    }
    let (serial, jacobi) = (mirror_error(&a), mirror_error(&b));
    println!("  espelho (secagem): serial {serial:.4}  jacobi {jacobi:.6}");
    assert!(
        jacobi < 1e-2,
        "a secagem de Jacobi deveria preservar a simetria da cena, e desviou {jacobi}"
    );
    assert!(
        serial > jacobi * 100.0,
        "a fixture nao expoe o vies da varredura na secagem: serial {serial}, jacobi {jacobi}"
    );
}

/// O CONTROLE do gate acima: a fixture de fato tem células cujo fator de borda
/// é **menor que 9** — senão o `e = 1` uniforme tornaria as duas rotas iguais
/// por construção e o gate mediria nada.
#[test]
fn the_fixture_has_cells_on_the_rim() {
    let g = symmetric_sheet();
    let s = g.s;
    let (mut rim, mut ink) = (0u32, 0u32);
    for y in 1..=H {
        for x in 1..=W {
            let i = x + y * s;
            if g.film[i] <= 0.0 && g.susp[i] <= 0.0 {
                continue;
            }
            ink += 1;
            let n: u32 = [
                i - s - 1,
                i - s,
                i - s + 1,
                i - 1,
                i,
                i + 1,
                i + s - 1,
                i + s,
                i + s + 1,
            ]
            .iter()
            .map(|&j| u32::from(g.susp[j] > 10.0))
            .sum();
            if n < 9 {
                rim += 1;
            }
        }
    }
    println!("  {rim} de {ink} celulas com tinta estao na borda (fator < 9)");
    assert!(
        rim > ink / 20,
        "a fixture quase nao tem borda ({rim}/{ink}) — o fator de borda seria 1 em toda parte"
    );
}
