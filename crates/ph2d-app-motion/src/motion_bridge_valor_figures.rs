//! **AS FIGURAS do tutorial do ciclo 6** (o valor e o pulso — doc 110, passo 6).
//!
//! ⚠️ **Elas não são desenhos:** cada ponto é o `size` que o motor deu a uma peça de verdade ao
//! correr a cena `=117` do produto, num instante de verdade. *Uma curva desenhada à mão é a
//! ilustração de uma teoria, não a saída dela.*
//!
//! ⚠️⚠️ **Estas duas são uma FORMA DE ONDA e não uma nuvem** — o eixo horizontal é o TEMPO. É a
//! primeira vez que um tutorial deste módulo desenha isso, e a razão é o assunto: um número que
//! manda em tudo só se vê **ao longo do tempo**, e uma fotografia de um instante mostraria quatro
//! panos iguais. ⭐ O vocabulário de desenho é o mesmo dos outros ciclos
//! ([`tutorial_draw`](super::tutorial_draw)) — *duas paletas seriam dois tutoriais com duas caras*.
//!
//! ⛔⛔ **A ESCRITA VEM DEPOIS DAS ASSERÇÕES** — a lei que o ciclo 4 pagou: uma corrida VERMELHA
//! que escreve dentro do laço deixa duas figuras idênticas no disco, e o tutorial passa a mostrar
//! a mesma imagem debaixo de duas legendas diferentes.

use super::tutorial_draw as draw;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Quantos instantes cada forma de onda tem. ⚠️ **Denso de propósito**: um degrau só se lê como
/// degrau se houver amostras suficientes DENTRO dele — com poucas, uma escada e uma onda dão a
/// mesma linha quebrada.
const AMOSTRAS: usize = 96;
/// O tempo ganha esta escala antes de virar o `x` da figura, para a onda ficar LARGA e baixa em
/// vez de um risco vertical. ⛔ Não é um número de gosto: a moldura é a forma do que se mostra.
const ESCALA_X: f32 = 2.2;

/// Corre a cena `=117` e colhe o `size.x` da **primeira peça** de cada sink, ao longo de `ate`
/// segundos, como pontos `(t·ESCALA_X, size)`.
///
/// ⚠️ **A cena vem da porta `monta`**, não de um documento montado à mão — uma fixtura que
/// reconstrói a cena mede outro programa que o que o dono abre.
///
/// ⚠️ **O `advance_tick` está aqui e é load-bearing:** as duas metades de baixo têm `pre` (o
/// metrónomo e o contador guardam o ciclo anterior), e sem ele elas ficam presas no primeiro tique.
fn colher(ate: f64) -> Vec<Vec<[f32; 2]>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let (sinks, _) = crate::motion_demo_legend::monta("117", &mut doc, &reg);
    let mut cook = Cook::new();
    let mut fora: Vec<Vec<[f32; 2]>> = vec![Vec::new(); sinks.len()];
    for k in 0..=AMOSTRAS {
        let t = k as f64 * ate / AMOSTRAS as f64;
        for (i, s) in sinks.iter().enumerate() {
            let saida = cook.cook(&doc.graph, &reg, *s, t).expect("coze");
            if let Some(Column::Vec2(v)) = saida[0].as_stream().get("size")
                && let Some(p) = v.first()
            {
                fora[i].push([t as f32 * ESCALA_X, p[0]]);
            }
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("tique");
    }
    fora
}

/// Quantos valores DISTINTOS a forma de onda toma (a `1e-3` de resolução) — a régua que separa
/// uma onda de uma escada.
fn distintos(onda: &[[f32; 2]]) -> usize {
    let mut v: Vec<i64> = onda
        .iter()
        .map(|p| (p[1] * 1000.0).round() as i64)
        .collect();
    v.sort_unstable();
    v.dedup();
    v.len()
}

/// Quantas vezes a forma de onda SOBE de repente — os clarões.
fn acendeu(onda: &[[f32; 2]]) -> usize {
    onda.windows(2).filter(|w| w[1][1] > w[0][1] + 0.05).count()
}

/// ⭐⭐ **Escreve as duas figuras do tutorial 6, com as asserções que as impedem de contar a
/// mesma história.**
///
/// ```text
/// cargo test -p ph2d-app-motion --lib write_the_value_figures -- --ignored --nocapture
/// ```
#[test]
#[ignore = "gerador de figuras — corra à mão"]
fn write_the_value_figures() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Motion Nodes/tutoriais/fig");
    std::fs::create_dir_all(&dir).expect("a pasta");

    // Uma volta inteira do oscilador para o par de cima; quatro batidas e pico para o de baixo.
    let numero = colher(2.4);
    let instante = colher(2.7);
    assert_eq!(numero.len(), 4, "a cena tem quatro sinks");

    let (liso, degraus) = (distintos(&numero[0]), distintos(&numero[1]));
    let (cada, quatro) = (acendeu(&instante[2]), acendeu(&instante[3]));
    eprintln!(
        "
  valores distintos │ liso {liso}  ·  degraus {degraus}
  clarões           │ a cada batida {cada}  ·  a cada quatro {quatro}"
    );

    assert!(
        liso > degraus * 3,
        "o liso toma {liso} valores e a escada {degraus} -- se forem parecidos as duas figuras \
         de cima contam a MESMA historia"
    );
    assert!(
        degraus >= 2,
        "a escada toma {degraus} valor(es) -- ela tem de respirar, so' que aos saltos"
    );
    assert!(
        cada > quatro && quatro >= 1,
        "clarões: {cada} contra {quatro} -- as duas figuras de baixo tem de ter ritmos \
         diferentes, e as DUAS tem de acender"
    );

    // A escrita, só agora — ⛔ ver o cabeçalho do módulo.
    let quadro_cima = draw::moldura(&[&numero[0], &numero[1]]);
    let quadro_baixo = draw::moldura(&[&instante[2], &instante[3]]);
    for (nome, fortes, fantasma, quadro) in [
        ("valor_liso", &numero[0], &numero[0], quadro_cima),
        ("valor_degraus", &numero[1], &numero[0], quadro_cima),
        (
            "valor_cada_batida",
            &instante[2],
            &instante[2],
            quadro_baixo,
        ),
        (
            "valor_a_cada_quatro",
            &instante[3],
            &instante[2],
            quadro_baixo,
        ),
    ] {
        // ⚠️ **O FANTASMA da 2.ª de cada par é a 1.ª** — é assim que a figura mostra que os dois
        // lados são o MESMO número e o MESMO ritmo, com um nó pelo meio. Na 1.ª ele é ela própria
        // (nada por baixo), e o `campo: false` mantém as duas sem linhas de ligação.
        let svg = draw::svg(fortes, fantasma, quadro, false);
        std::fs::write(dir.join(format!("{nome}.svg")), svg).expect("escreve");
    }
    eprintln!("  ✓ 4 figuras em {}", dir.display());
}
