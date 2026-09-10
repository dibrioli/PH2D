//! Os gates do refinamento — a alternativa `Smooth` do report de 2026-09-10.

use crate::{Mesh2d, RefineOptions, deviation, grid_mesh_of, refine_posed, splits_for};
use std::collections::BTreeMap;

/// Uma cápsula de `320×96`, a arte do braço pintado do smoke.
fn capsula() -> (Vec<u8>, u32, u32) {
    let (w, h) = (320usize, 96usize);
    let (raio, ax, bx) = (
        h as f64 / 2.0 - 3.0,
        h as f64 / 2.0,
        w as f64 - h as f64 / 2.0,
    );
    let mut a = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let px = x as f64 + 0.5;
            let d = (px - px.clamp(ax, bx)).hypot(y as f64 + 0.5 - h as f64 / 2.0);
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let v = ((raio + 1.0 - d).clamp(0.0, 1.0) * 255.0) as u8;
            a[y * w + x] = v;
        }
    }
    #[expect(clippy::cast_possible_truncation)]
    (a, w as u32, h as u32)
}

fn malha() -> Mesh2d {
    let (a, w, h) = capsula();
    let focos: Vec<[f64; 2]> = (0..=3)
        .map(|k| [f64::from(k) * 320.0 / 3.0, 48.0])
        .collect();
    grid_mesh_of(&a, w, h, &focos, crate::GridOptions::default()).expect("tinta")
}

/// ⭐⭐ **A MESMA malha com os vértices por OUTRA ORDEM.**
///
/// ⚠️⚠️ **Ela existe porque uma mutação SOBREVIVEU:** o `grid_mesh_of` numera os vértices por
/// ORDEM DE PRIMEIRO TOQUE, que é exactamente a ordem em que o refinamento os recriaria — então
/// sobre a malha da grelha o caminho longo devolve por acaso a mesma numeração, e o gate do
/// caminho de omissão ficava verde com a guarda apagada. *Uma fixtura que só usa a malha que o
/// produto fabrica não distingue «devolveu a de entrada» de «reconstruiu uma igual».*
fn malha_permutada() -> Mesh2d {
    let m = malha();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a malha da cápsula tem centenas de nós"
    )]
    let ultimo = (m.rest.len() - 1) as u32;
    Mesh2d {
        rest: m.rest.iter().rev().copied().collect(),
        tris: m
            .tris
            .iter()
            .map(|t| [ultimo - t[0], ultimo - t[1], ultimo - t[2]])
            .collect(),
        size: m.size,
    }
}

/// ⭐ **Um campo CURVO de verdade** — dobra o plano em torno de `x = 160`, como uma junta faz.
fn campo_dobrado(t: f64) -> impl FnMut([f64; 2]) -> [f64; 2] {
    move |p: [f64; 2]| {
        let s = ((p[0] - 160.0) / 160.0).clamp(-1.0, 1.0) * t;
        let (sin, cos) = (s.sin(), s.cos());
        let (x, y) = (p[0] - 160.0, p[1] - 48.0);
        [
            (x * cos - y * sin) + 160.0,
            (x * sin).mul_add(1.0, y * cos) + 48.0,
        ]
    }
}

/// ⭐⭐⭐ **SEM REFINAMENTO A SAÍDA É BYTE-IDÊNTICA** — o caminho de omissão do desenho.
///
/// ⛔⛔ **É a metade que impede esta wave de mexer no que já shipa.** O `Fast` do painel tem de
/// desenhar exactamente o que desenhava antes de a alternativa existir; qualquer deriva aqui seria
/// uma regressão silenciosa numa feature que o dono já aprovou.
///
/// (Mutação: fazer o `k <= 1` cair no caminho longo ⇒ RED, porque o percurso reconstrói os índices.)
#[test]
fn without_refining_the_output_is_byte_identical() {
    // ⚠️ **As DUAS malhas**: a da grelha (o caminho real) e uma com os vértices por outra ORDEM —
    // e é a segunda que tem poder de matar, pela razão escrita na [`malha_permutada`].
    for m in [malha(), malha_permutada()] {
        let mut campo = campo_dobrado(0.6);
        let esperado: Vec<[f64; 2]> = m.rest.iter().map(|&p| campo(p)).collect();
        let (saida, posed, k) = refine_posed(
            &m,
            &mut campo,
            RefineOptions {
                tolerance_px: f64::INFINITY,
                max_split: 6,
            },
        );
        assert_eq!(k, 1, "com tolerancia infinita nao ha' nada a refinar");
        assert_eq!(saida, m, "a malha tem de sair a MESMA, ao bit");
        assert_eq!(posed, esperado, "as posicoes tem de sair as MESMAS, ao bit");
    }
}

/// ⭐⭐⭐ **A CONFORMIDADE É EXACTA: nenhum ponto nasce duas vezes, e toda aresta interior é
/// partilhada por exactamente DOIS triângulos.**
///
/// ⛔⛔ **Uma fenda aqui é pior que numa malha normal:** cada triângulo é um RECORTE independente,
/// então um nó pendurado não é um artefacto de sombreamento — é um **fio de fundo a atravessar a
/// arte**. É a mesma razão pela qual a [`crate::grid`] recusa a *quadtree* por escrito.
///
/// ⚠️ A prova é dupla de propósito: **posições distintas** (nenhum ponto duplicado, que é o que um
/// `k` por triângulo produziria) **e** a contagem de arestas (que é o que um erro de índice daria).
///
/// (Mutação: tirar a canonicalização `if u < v` da chave ⇒ RED por pontos duplicados.)
#[test]
fn the_refined_mesh_has_no_hanging_nodes_and_no_duplicated_points() {
    let m = malha();
    let mut campo = campo_dobrado(1.8);
    let (r, posed, k) = refine_posed(
        &m,
        &mut campo,
        RefineOptions {
            tolerance_px: 0.5,
            max_split: 6,
        },
    );
    assert!(k >= 3, "uma dobra a serio tem de pedir refinamento: k={k}");
    assert_eq!(r.rest.len(), posed.len());

    // (a) Nenhuma posição de repouso aparece duas vezes.
    let mut vistos: BTreeMap<(u64, u64), usize> = BTreeMap::new();
    for p in &r.rest {
        *vistos.entry((p[0].to_bits(), p[1].to_bits())).or_default() += 1;
    }
    let duplicados = vistos.values().filter(|n| **n > 1).count();
    assert_eq!(
        duplicados, 0,
        "{duplicados} posicoes nasceram DUAS vezes — a chave canonica nao esta' a unir os lados"
    );

    // (b) Toda aresta é partilhada por 1 (bordo) ou 2 (interior) triângulos. Três é impossível
    // numa malha sã, e **um nó pendurado aparece como uma aresta de bordo no meio da peça**.
    let mut arestas: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for t in &r.tris {
        for i in 0..3 {
            let (a, b) = (t[i], t[(i + 1) % 3]);
            *arestas.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    assert!(
        arestas.values().all(|n| *n <= 2),
        "ha' aresta com mais de dois triangulos — os indices estao trocados"
    );
    // O bordo de um rectângulo de células vivas: as arestas de bordo têm de ser MENOS que as
    // interiores por uma margem larga, senão a malha desfez-se em ilhas.
    let bordo = arestas.values().filter(|n| **n == 1).count();
    let interior = arestas.values().filter(|n| **n == 2).count();
    assert!(
        interior > bordo * 2,
        "bordo {bordo} contra interior {interior} — a malha refinada desfez-se"
    );
}

/// ⭐⭐⭐ **O REFINAMENTO APROXIMA O CAMPO, e a barra é a TOLERÂNCIA PEDIDA.**
///
/// ⚠️ **A régua é o desvio que o próprio módulo mede** ([`deviation`]), corrido sobre a malha já
/// refinada: com `k` partes, os meios das arestas novas têm de estar dentro da tolerância. ⛔ Medir
/// a malha ORIGINAL depois de refinar mediria outra coisa.
///
/// (Mutação: `splits_for` devolver sempre `1` ⇒ RED.)
#[test]
fn refining_brings_the_drawn_map_within_the_tolerance() {
    let m = malha();
    // ⚠️ **A escada começa ABAIXO do desvio que a malha guardada já tem** — senão os primeiros
    // degraus não mordem, o `k` fica em `1` e o gate acusa o produto de não melhorar quando o que
    // aconteceu foi não lhe ter sido pedido nada. *Uma escada que começa no ponto neutro do knob
    // não testa o knob.*
    let mut campo0 = campo_dobrado(1.8);
    let posed0: Vec<[f64; 2]> = m.rest.iter().map(|&p| campo0(p)).collect();
    let cru = deviation(&m, &posed0, &mut campo0);
    assert!(
        cru > 2.0,
        "a fixtura tem de produzir o fenomeno: desvio cru {cru:.2} px"
    );
    let mut anterior = cru;
    // ⭐ A escada sai do PRÓPRIO desvio cru, e não de números escritos à mão: assim cada degrau
    // morde por construção, e a fixtura pode mudar sem o gate passar a medir o ponto neutro.
    for tol in [cru / 2.0, cru / 4.0, cru / 8.0, cru / 16.0] {
        let mut campo = campo_dobrado(1.8);
        let (r, posed, _k) = refine_posed(
            &m,
            &mut campo,
            RefineOptions {
                tolerance_px: tol,
                max_split: 8,
            },
        );
        let d = deviation(&r, &posed, &mut campo);
        assert!(
            d <= tol,
            "com tolerancia {tol:.3} o desvio ficou em {d:.3} px"
        );
        // ⚠️ **NÃO-CRESCENTE, e não estritamente melhor** — e a diferença é o mecanismo: a correcção
        // de um passo pode **passar** da tolerância pedida (ela parte de `k·√(d/tol)` e arredonda
        // para cima), e aí o degrau seguinte já está satisfeito pelo mesmo `k`. *Exigir melhoria
        // estrita seria exigir que o estimador nunca acertasse com folga.*
        assert!(
            d <= anterior + 1e-12,
            "apertar a tolerancia de {anterior:.3} para {tol:.3} PIOROU ({d:.3})"
        );
        anterior = d;
    }
    // ⭐ E a ponta da escada tem de ser MUITO melhor que o cru — senão o laço acima passaria com
    // um produto que nunca refina nada.
    assert!(
        anterior * 4.0 < cru,
        "do cru {cru:.3} ate' ao fim da escada {anterior:.3} nao houve ganho a serio"
    );
}

/// ⭐⭐ **A LEI É `O(h²)`, E É POR ISSO QUE O `k` SAI DE UMA RAIZ QUADRADA.**
///
/// ⚠️⚠️ **Medido, não escolhido** (a tabela no cabeçalho do [`crate::refine`]): partir ao meio
/// divide o desvio por `~3,6`, não por `2`. Um `k` **linear** no desvio pediria `4×` os triângulos
/// para a mesma resposta — e este gate é o que impede alguém de o «simplificar» para uma divisão.
#[test]
fn the_split_count_follows_the_square_root_law() {
    let o = RefineOptions {
        tolerance_px: 1.0,
        max_split: 64,
    };
    assert_eq!(splits_for(1.0, o), 1, "no ponto certo nao se parte nada");
    assert_eq!(splits_for(0.2, o), 1, "abaixo da barra tambem nao");
    assert_eq!(splits_for(4.0, o), 2, "4x o desvio pede 2 partes, nao 4");
    assert_eq!(splits_for(9.0, o), 3);
    assert_eq!(splits_for(100.0, o), 10);
    // ⛔ Um desvio que não é um número não pode virar um `k` gigante.
    assert_eq!(splits_for(f64::NAN, o), 1);
    assert_eq!(splits_for(f64::INFINITY, o), 1);
}

/// ⛔ **O TECTO MORDE, e ele é de um RECURSO** — o custo de encodar o quadro cresce com `k²`.
///
/// ⚠️ Sem ele uma dobra extrema pediria um `k` de duas casas e o quadro passaria a valer segundos.
/// A barra do `max_split` é medida no cabeçalho do módulo; aqui prova-se que ela é **honrada**.
#[test]
fn the_ceiling_on_the_split_is_honoured() {
    let apertado = RefineOptions {
        tolerance_px: 0.001,
        max_split: 3,
    };
    assert_eq!(splits_for(1000.0, apertado), 3);
    let m = malha();
    let mut campo = campo_dobrado(1.2);
    let (r, _p, k) = refine_posed(&m, &mut campo, apertado);
    assert_eq!(k, 3, "o tecto nao foi honrado no caminho do produto");
    assert!(
        r.tris.len() <= m.tris.len() * 9,
        "com k=3 cada triangulo da' 9, e {} > {}",
        r.tris.len(),
        m.tris.len() * 9
    );
}

/// ⛔ **DENTRO DE UMA CONSTRUÇÃO, CADA PONTO É PERGUNTADO UMA VEZ SÓ** — é o que a chave canónica
/// compra além da conformidade.
///
/// ⚠️ **A régua é a razão `perguntas / vértices`, e não a contagem crua**, porque o `refine_posed`
/// tem mais do que uma construção por desenho: ele **confere** o que entregou (um passe pelos meios
/// das arestas) e pode **reconstruir uma vez**. Sem a chave, cada ponto de aresta seria deformado
/// **duas vezes por construção** — uma por cada triângulo vizinho —, e numa malha de grelha as
/// arestas são a maioria dos pontos.
///
/// (Mutação: trocar a chave `No::Aresta` por `No::Miolo` ⇒ a razão salta e o gate fica RED.)
#[test]
fn within_one_build_each_point_is_asked_once() {
    let m = malha();
    let mut vistos: BTreeMap<(u64, u64), usize> = BTreeMap::new();
    let mut base = campo_dobrado(1.8);
    let mut campo = |p: [f64; 2]| {
        *vistos.entry((p[0].to_bits(), p[1].to_bits())).or_default() += 1;
        base(p)
    };
    let (r, _posed, k) = refine_posed(
        &m,
        &mut campo,
        RefineOptions {
            tolerance_px: 0.5,
            max_split: 6,
        },
    );
    assert!(k > 1, "a fixtura tem de pedir refinamento");
    let perguntas: usize = vistos.values().sum();
    // ⚠️⚠️ **O denominador é o número de posições DISTINTAS, e não o de vértices** — e a diferença
    // foi medida por uma mutação que SOBREVIVEU: sem a chave canónica os pontos de aresta nascem
    // **duas** vezes, então o número de vértices sobe junto com o de perguntas e a razão **não se
    // mexe**. *Uma razão cujo denominador cresce com o defeito não vê o defeito.*
    let razao = perguntas as f64 / vistos.len() as f64;
    // ⚠️⚠️ **A BARRA SAI DO VALE MEDIDO ENTRE OS DOIS LADOS, e as duas leituras estão aqui:**
    //
    // | árvore | razão |
    // |---|---:|
    // | com o memo | **`1,89`** |
    // | sem o memo (cada triângulo recria a grelha dele) | **`2,76`** |
    //
    // ⛔⛔ **E este gate NÃO guarda a chave canónica, apesar de ela viver ao lado** — mediu-se:
    // apagá-la move a razão de `1,89` para `1,92`, porque as perguntas e as posições distintas
    // sobem **juntas**. *Uma razão cujo denominador cresce com o defeito não vê o defeito.* Quem a
    // guarda é o [`the_refined_mesh_has_no_hanging_nodes_and_no_duplicated_points`], por mutação.
    assert!(
        razao < 2.4,
        "{perguntas} perguntas para {} posicoes distintas ({razao:.2}x) — o memo do indice nao esta' a segurar",
        vistos.len()
    );
    assert!(
        vistos.len() >= r.rest.len(),
        "ha' vertices na malha refinada que nunca foram perguntados ao campo"
    );
}
