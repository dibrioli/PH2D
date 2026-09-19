//! ⭐⭐⭐⭐ **O VALOR DE FÁBRICA DO PENTE** — a sonda que a decisão do dono pediu
//! (2026-09-19: *«ligar o pente de topologia»*).
//!
//! # ⚠️ Porque isto é uma DIVERGÊNCIA DECLARADA e não uma afinação
//!
//! O [`ph2d_sculpt3d::Brush::pente`] nasce em `0,0` **porque esse é o valor de
//! fábrica MEDIDO DO ALVO** (espec §5). Mudá-lo é divergir dele de propósito,
//! logo precisa das duas colunas na mesma tabela e do número escrito ao lado —
//! *o §0.0 proíbe escrever um limite que a medição não defende.*
//!
//! # A coluna que decide é o RELÓGIO, e a razão é o «por omissão»
//!
//! ⛔⛔ Com o pente a `0` ele **não corre**. Ligado por omissão, **todo carimbo
//! de todo artista passa a pagá-lo** — e o report de 21/09 foi exactamente
//! *«algoritmo mais lento que o modo padrão»*. ⇒ a pergunta não é *«qual valor
//! dá a melhor grade?»* (é o topo, e isso já está medido), é **«qual valor
//! compra fileira sem pôr o dab no tecto do orçamento?»**
//!
//! ⚠️ **A régua da qualidade é a FILEIRA e não a grade** — a lição de 20/09
//! ([`ph2d_sculpt3d::medida_da_fileira`]): a grade satura a `66,7 %` por
//! construção (um terço das arestas de uma grade triangulada são diagonais), e
//! o que o artista lê é o COMPRIMENTO das linhas contínuas.

use super::*;
use ph2d_sculpt3d::medida_do_pente::{grade_da_faixa, lascas, pior_angulo, vinco_da_faixa};

/// Os degraus que a decisão varre. ⚠️ `0,0` é o CONTROLO (o que shipa hoje) e
/// `1,0` é o tecto — sem os dois extremos a tabela não diz o que se ganha nem o
/// que se paga.
const DEGRAUS: [f32; 6] = [0.0, 0.15, 0.25, 0.40, 0.65, 1.0];

/// ⭐⭐⭐⭐ **A TABELA DA DECISÃO** — qualidade e relógio, nos quatro rumos.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-app-sculpt3d --release --lib \
///   diag_o_valor_de_fabrica_do_pente -- --ignored --nocapture --test-threads=1
/// ```
///
/// ⚠️ **Corre em `--release`**: o debug lê ~20× mais lento e daria um tecto
/// cinco vezes menor — a mesma cerca que o §24 desta linha já escreveu.
#[test]
#[ignore = "sonda: imprime a tabela da decisao, nao afirma nada"]
fn diag_o_valor_de_fabrica_do_pente() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    println!("\n== O VALOR DE FABRICA DO PENTE ==");
    println!("   peca da cena =49 · raio do app {raio:.4} · alvo do refino {alvo:.4}");
    println!("   media dos QUATRO rumos · tecto do dab = 8 ms\n");
    println!(
        "{:>6} | {:>7} {:>7} {:>7} | {:>7} {:>7} {:>6} | {:>9} {:>7}",
        "pente", "grade%", "fil p50", "fil p90", "vinco", "pior", "lascas", "ms/dab", "% orc"
    );
    println!("{}", "-".repeat(80));

    for pente in DEGRAUS {
        let (mut grade, mut f50, mut f90, mut vinco, mut pior, mut lasca) =
            (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64, 0usize);
        let mut relogio = 0.0f64;
        for (_, e) in RUMOS {
            let t0 = std::time::Instant::now();
            let (m, c) = traco_com(pente, e, raio, alvo);
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            // ⚠️ A grade vem em TRÊS baldes de 15°; a fracção é o balde ZERO.
            let (baldes, n_arestas) = grade_da_faixa(&m, &c, raio);
            let g = if n_arestas == 0 {
                0.0
            } else {
                baldes[0] as f64 / n_arestas as f64
            };
            let (_, v, _, _) = vinco_da_faixa(&m, &c, raio);
            let (fa, fb) = fileira_p50_p90(&m, &c, raio);
            grade += g;
            f50 += fa;
            f90 += fb;
            vinco += f64::from(v);
            pior += pior_angulo(&m, &c, raio).0;
            lasca += lascas(&m, &c, raio, LIMIAR_DA_LASCA).0;
            relogio += dt;
        }
        let n = RUMOS.len() as f64;
        // ⚠️ **O `traco_com` faz `24` dabs** (contado no corpo dele), e o que
        // decide é o custo POR DAB — é ele que enfrenta o tecto de `8 ms`.
        let por_dab = relogio / n / 24.0;
        println!(
            "{pente:>6.2} | {:>6.2}% {:>7.1} {:>7.1} | {:>7.3} {:>7.1} {:>6} | {por_dab:>9.3} {:>6.1}%",
            grade / n * 100.0,
            f50 / n,
            f90 / n,
            vinco / n,
            pior / n,
            lasca,
            por_dab / 8.0 * 100.0
        );
    }
    println!(
        "\n   ⇒ o valor de fabrica vive onde a FILEIRA ja' subiu e o ms/dab ainda\n   \
         nao encosta no tecto. A grade satura em 66,7% por construcao."
    );
}

/// As duas medianas da fileira — a porta devolve `(p50, p90, máximo, cadeias)`.
fn fileira_p50_p90(m: &ph2d_mesh::Mesh, c: &[[f32; 3]], raio: f32) -> (f64, f64) {
    let (p50, p90, _, _) = ph2d_sculpt3d::medida_da_fileira::fileira_da_faixa(m, c, raio);
    (p50, p90)
}

/// ⛔⛔⛔ **A COLUNA QUE DECIDE SE «LIGADO POR OMISSÃO» É SEGURO: a DENSIDADE.**
///
/// A tabela irmã mede uma peça só. Mas o custo do pente é **por vértice da
/// PEGADA** (§82: `~4,5 µs/vértice`), e a pegada cresce com a densidade da
/// malha — logo *o número de fábrica que é confortável nesta peça pode estourar
/// o tecto na peça do artista*.
///
/// ⚠️ **É a mesma armadilha que a `=43`/`=44` pagaram em 14/09** (*«meio
/// travado»*): a cena mandava construir uma peça de `1,5 M` e ali **toda**
/// ferramenta estourava. *Um valor de fábrica escolhido numa peça só é um valor
/// escolhido para essa peça.*
#[test]
#[ignore = "sonda: imprime a escada da densidade, nao afirma nada"]
fn diag_o_pente_contra_a_densidade_da_peca() {
    let raio = raio_do_app();
    println!("\n== O PENTE CONTRA A DENSIDADE (o tecto do dab e' 8 ms) ==");
    println!("   pente = 0,65 contra o CONTROLO (0,0) · media dos quatro rumos\n");
    println!(
        "{:>10} {:>9} | {:>11} {:>11} | {:>8} {:>8}",
        "alvo", "vertices", "ms/dab OFF", "ms/dab ON", "% orc", "razao"
    );
    println!("{}", "-".repeat(66));

    // ⚠️ O `alvo` é a aresta que o refino persegue: mais FINO ⇒ mais vértices na
    // pegada. O de fábrica da cena é o do meio.
    for alvo in [0.0340f32, 0.0170, 0.0120, 0.0085] {
        let mut linha = [0.0f64; 2];
        let mut verts = 0usize;
        for (i, pente) in [0.0f32, 0.65].into_iter().enumerate() {
            let mut soma = 0.0f64;
            for (_, e) in RUMOS {
                let t0 = std::time::Instant::now();
                let (m, _) = traco_com(pente, e, raio, alvo);
                soma += t0.elapsed().as_secs_f64() * 1000.0;
                verts = m.positions().len();
            }
            linha[i] = soma / RUMOS.len() as f64 / 24.0;
        }
        println!(
            "{alvo:>10.4} {verts:>9} | {:>11.3} {:>11.3} | {:>7.1}% {:>8.2}x",
            linha[0],
            linha[1],
            linha[1] / 8.0 * 100.0,
            linha[1] / linha[0].max(1e-9)
        );
    }
    println!(
        "\n   ⇒ «ligado por omissao» so' e' seguro se a coluna `% orc` ficar\n   \
         abaixo de 100 na densidade que o artista de facto usa."
    );
}
