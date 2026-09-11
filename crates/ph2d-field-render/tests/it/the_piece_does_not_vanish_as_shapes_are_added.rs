//! ⛔⛔⛔ **A PEÇA DESAPARECIA À MEDIDA QUE FORMAS ERAM ACRESCENTADAS** — o report do Enio, 2026-08-30.
//!
//! # O report
//!
//! *«quanto mais objetos colocamos na tela, mais artefatos e mais largos os vãos»*, com uma foto de
//! um modelo de tubos filetados e riscos de fundo a atravessar as juntas.
//!
//! # A causa
//!
//! O passo da marcha era `2^(−profundidade/2)` — **exponencial no número de formas do grupo**, porque
//! um nó de `n` filhos é uma corrente de `n − 1` misturas. Com o tecto de passos da marcha
//! (`MAX_STEPS`), a partir de certo ponto o raio **acaba os passos antes de chegar à superfície** e é
//! largado em silêncio, o que se lê como fundo. Medido: `12` formas deixavam `688` pixels de
//! `34 737`, e `13` deixavam **zero**.
//!
//! A lei certa soma os QUADRADOS (ver [`ph2d_field_eval::gradient_bound`]) e é `√n`.
//!
//! # ⭐⭐ A régua não precisa de oráculo, e é isso que a torna forte
//!
//! Uma união **filetada** é um SUPERCONJUNTO da união **viva** das mesmas formas — o campo
//! arredondado nunca é maior que o `min` (`r − ‖(u,v)‖ ≤ min(a,b)` sempre que `min < r`). E a união
//! viva marcha a `1,0`, que nenhum tecto de passos aperta.
//!
//! ⇒ **`acertos(filetada) ≥ acertos(viva)`** e **`acertos(n) ≥ acertos(n−1)`**, as duas por
//! geometria. *Nenhuma das duas precisa de saber qual é a imagem certa — só que ela não pode
//! encolher.*

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::{hybrid::Registry, leaf, safe_march_step};
use ph2d_field_render::{EXHAUSTED, MARCH_RAYS, Orbit, STEP_SAMPLES, trace_stepped_for_test};
use std::sync::atomic::Ordering;

/// `n` esferas encavalitadas ao longo de uma hélice apertada, num nó só — a forma da cena do report.
///
/// ⚠️ **Ela é um BLOCO MACIÇO de propósito:** consecutivas sobrepõem-se com folga, então a união não
/// tem vão interior nenhum e **todo** pixel de fundo cercado por peça é um defeito da marcha. *Uma
/// fixtura com vãos verdadeiros faria a régua de furos medir a fixtura.*
fn espiral(n: usize, blend: Blend) -> FieldDoc {
    let mut nodes: Vec<Node> = Vec::new();
    for i in 0..n {
        #[allow(clippy::cast_precision_loss)]
        let t = i as f32;
        #[allow(clippy::cast_precision_loss)]
        let x = (t - (n - 1) as f32 * 0.5) * 0.16;
        nodes.push(leaf(
            Primitive::Sphere { radius: 0.28 },
            Xform::at(x, 0.10 * t.sin(), 0.10 * t.cos()),
        ));
    }
    let ids: Vec<NodeId> = (0..n)
        .map(|i| NodeId(u32::try_from(i).expect("poucas")))
        .collect();
    nodes.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(blend),
            children: ids,
        },
    ));
    FieldDoc::new(nodes, NodeId(u32::try_from(n).expect("poucas"))).expect("cena")
}

/// Um pixel de fundo com peça dos DOIS lados na linha **e** na coluna está dentro da silhueta.
fn furos(g: &ph2d_field_render::Gbuffer) -> usize {
    let (w, h) = (g.width as usize, g.height as usize);
    let mut n = 0;
    for y in 0..h {
        for x in 0..w {
            if g.hit[y * w + x] {
                continue;
            }
            let linha = (0..x).any(|i| g.hit[y * w + i]) && (x + 1..w).any(|i| g.hit[y * w + i]);
            let coluna = (0..y).any(|j| g.hit[j * w + x]) && (y + 1..h).any(|j| g.hit[j * w + x]);
            if linha && coluna {
                n += 1;
            }
        }
    }
    n
}

#[test]
fn the_piece_does_not_vanish_as_shapes_are_added() {
    let reg = Registry::default();
    let cam = Orbit {
        half_extent: 1.4,
        ..Orbit::default()
    };
    const LADO: u32 = 320;
    for n in 2..=16usize {
        let filetada = espiral(n, Blend::Exact { radius: 0.12 });
        let viva = espiral(n, Blend::Sharp);
        let passo = safe_march_step(&filetada);
        let passo_vivo = safe_march_step(&viva);
        EXHAUSTED.store(0, Ordering::Relaxed);
        let com = trace_stepped_for_test(&filetada, &reg, &cam, LADO, LADO, passo);
        // ⭐ **O raio que acaba os passos é largado em silêncio** — contá-lo é o que torna o furo
        // acima diagnosticável em vez de misterioso. Ver [`ph2d_field_render::EXHAUSTED`].
        let esgotados = EXHAUSTED.load(Ordering::Relaxed);
        let sem = trace_stepped_for_test(&viva, &reg, &cam, LADO, LADO, passo_vivo);
        assert!(
            (passo_vivo - 1.0).abs() < 1e-6,
            "a união viva tem de marchar a 1,0 — senão o controle desta régua também está apertado \
             (leu {passo_vivo})"
        );
        // ⭐ **O sintoma do report, medido directamente:** fundo no meio da peça.
        let buracos = furos(&com);
        assert_eq!(
            buracos, 0,
            "{n} formas: {buracos} pixels de FUNDO dentro da silhueta, com o passo {passo:.4} — a \
             marcha está a desistir antes de chegar à superfície"
        );
        assert_eq!(
            esgotados, 0,
            "{n} formas: {esgotados} raios acabaram o orçamento de passos com o passo {passo:.4} — \
             o orçamento sai do passo e devia bastar"
        );
        assert!(
            com.hits() >= sem.hits(),
            "{n} formas: a peça FILETADA acertou {} pixels e a VIVA {} — o filete só acrescenta \
             matéria, então isto é a marcha a desistir (passo {passo:.4})",
            com.hits(),
            sem.hits()
        );
    }

    // ⛔ **O CONTROLE**: sem ele o gate passaria numa cena que a marcha nunca desafia — uma peça
    // ausente também não tem furos. À lei antiga (`2^(−(n−1)/2)`) a peça de `13` formas media
    // **zero** pixels.
    let cheia = espiral(16, Blend::Exact { radius: 0.12 });
    let passo = safe_march_step(&cheia);
    let com = trace_stepped_for_test(&cheia, &reg, &cam, LADO, LADO, passo).hits();
    assert!(
        com > 20_000,
        "a peça de 16 formas devia encher boa parte de um quadro de {LADO}² e acertou {com} — se \
         este número desabou, a régua acima está a olhar para uma imagem vazia"
    );
}

/// ⭐⭐⭐ **E O PREÇO NÃO EXPLODE COM O NÚMERO DE FORMAS** — o gate da LEI, e ele existe porque o de
/// cima **não** a prova.
///
/// ⛔⛔ **Medido: a mutação que repõe a lei exponencial de ontem SOBREVIVE ao gate da imagem.** Com o
/// orçamento derivado do passo, um passo curto de mais deixa de furar a peça — passa só a pagar por
/// ela. *Duas curas independentes fazem um gate só medir a combinação delas.*
///
/// A régua é o número de **amostras de campo por raio**, que é uma contagem determinista (⛔ não um
/// relógio: um gate de razão de tempos entra na família de flakes de carga do `CLAUDE.md` §5.0).
///
/// | formas | amostras/raio, lei da SOMA | lei EXPONENCIAL |
/// |---:|---:|---:|
/// | 4 | `14,0` | `20,1` |
/// | 8 | `16,5` | `66,3` |
/// | 12 | `21,8` | `287,4` |
/// | 16 | **`31,9`** | **`1 464,9`** |
///
/// ⭐ `46×` de diferença na mesma imagem, e o quadro de 16 formas foi de `221 ms` para o
/// orçamento inteiro de um quadro. *A lei não é uma optimização: é o que separa «desenha» de
/// «desenhou».*
#[test]
fn the_price_of_a_shape_is_not_exponential() {
    let reg = Registry::default();
    let cam = Orbit {
        half_extent: 1.4,
        ..Orbit::default()
    };
    let amostras_por_raio = |n: usize| {
        let doc = espiral(n, Blend::Exact { radius: 0.12 });
        STEP_SAMPLES.store(0, Ordering::Relaxed);
        MARCH_RAYS.store(0, Ordering::Relaxed);
        let g = trace_stepped_for_test(&doc, &reg, &cam, 320, 320, safe_march_step(&doc));
        let raios = MARCH_RAYS.load(Ordering::Relaxed);
        assert!(
            raios > 10_000,
            "{n} formas: só {raios} raios entraram na caixa"
        );
        assert!(g.hits() > 5_000, "{n} formas: a peça mal aparece");
        #[allow(clippy::cast_precision_loss)]
        let por_raio = STEP_SAMPLES.load(Ordering::Relaxed) as f64 / raios as f64;
        por_raio
    };

    let base = amostras_por_raio(4);
    let cheia = amostras_por_raio(16);
    // A barra é `4×` porque a medição diz `2,28×` e a lei exponencial diz `72,9×` — ela fica no meio
    // do vazio entre as duas, e não colada em nenhuma. ⚠️ Subir de 4 para 16 formas **tem** de custar
    // alguma coisa (o tecto de `‖∇f‖` é `√n`, logo o passo encolhe `2×`): a cerca é contra a
    // EXPLOSÃO, não contra o crescimento.
    assert!(
        cheia <= base * 4.0,
        "16 formas custam {cheia:.1} amostras por raio contra {base:.1} de 4 formas — o preço de \
         uma forma voltou a ser exponencial, e o quadro deixou de fechar"
    );
}
