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

/// ⛔⛔ **A fixtura PREGA a densidade (`target_tris: 0`), e não é conforto — é o que a mantém a ser
/// a mesma pergunta.** Estes gates medem a LEI DO REFINAMENTO (`k`, conformidade, orçamento de
/// peças) sobre uma malha de `216` triângulos, e todas as barras deles saíram dessa contagem. Desde
/// que a `GridOptions` passou a ser um ORÇAMENTO (2026-09-15) o `default()` entrega `~3 000`
/// triângulos, e cinco destes gates reprovaram de uma vez — não por a lei ter mudado, mas por o
/// **sujeito** ter mudado por baixo deles. *Um gate cujo sujeito muda não afirma nada.*
fn malha() -> Mesh2d {
    let (a, w, h) = capsula();
    let focos: Vec<[f64; 2]> = (0..=3)
        .map(|k| [f64::from(k) * 320.0 / 3.0, 48.0])
        .collect();
    grid_mesh_of(
        &a,
        w,
        h,
        &focos,
        crate::GridOptions {
            target_tris: 0,
            ..crate::GridOptions::default()
        },
    )
    .expect("tinta")
}

/// ⭐⭐⭐ **UM CAMPO QUE ENGANA O ESTIMADOR** — uma onda cujo período é da ordem da célula.
///
/// ⚠️⚠️ **Ela existe porque uma mutação SOBREVIVEU:** o tecto do orçamento **dentro da correcção**
/// nunca era alcançado por nenhuma fixtura. Com o campo suave, a lei `O(h²)` acerta, a correcção
/// pede `k + 1` e o `clamp` de cima nunca morde — *uma guarda que só o caso patológico alcança
/// precisa do caso patológico escrito.*
///
/// ⭐ Aqui o estimador **ALIASA**: os meios das arestas da malha grossa caem perto dos zeros da
/// onda, o primeiro `k` sai `5`, e a conferência — que já vê a onda — pede **`19`**. Medido.
fn campo_traicoeiro() -> impl FnMut([f64; 2]) -> [f64; 2] {
    move |p: [f64; 2]| {
        [
            p[0],
            5.0_f64.mul_add((p[0] * std::f64::consts::TAU / 7.0).sin(), p[1]),
        ]
    }
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
        // ⭐⭐ **AS DUAS LEIS**, porque a promessa é do PRODUTO e não de uma delas: o `Fast` do
        // painel tem de desenhar o mesmo, esteja o `Smooth` a usar a lei que estiver.
        for adaptativo in [false, true] {
            let (saida, posed, r) = refine_posed(
                &m,
                &mut campo,
                RefineOptions {
                    tolerance_px: f64::INFINITY,
                    max_pieces: 216 * 36,
                    adaptativo,
                },
            );
            assert_eq!(
                r.pecas,
                m.tris.len(),
                "com tolerancia infinita nada se parte"
            );
            assert_eq!(saida, m, "a malha tem de sair a MESMA, ao bit");
            assert_eq!(posed, esperado, "as posicoes tem de sair as MESMAS, ao bit");
        }
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
    let (r, posed, rel) = refine_posed(
        &m,
        &mut campo,
        RefineOptions {
            tolerance_px: 0.5,
            max_pieces: 216 * 36,
            adaptativo: false,
        },
    );
    let k = rel.k().expect("a lei uniforme declara o k");
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
        let (r, posed, _rel) = refine_posed(
            &m,
            &mut campo,
            RefineOptions {
                tolerance_px: tol,
                max_pieces: 216 * 64,
                adaptativo: false,
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
    // Uma malha de UMA peça com orçamento `64²`: o tecto não morde, e o que se lê é a lei.
    let o = RefineOptions {
        tolerance_px: 1.0,
        max_pieces: 64 * 64,
        adaptativo: false,
    };
    assert_eq!(splits_for(1.0, 1, o), 1, "no ponto certo nao se parte nada");
    assert_eq!(splits_for(0.2, 1, o), 1, "abaixo da barra tambem nao");
    assert_eq!(splits_for(4.0, 1, o), 2, "4x o desvio pede 2 partes, nao 4");
    assert_eq!(splits_for(9.0, 1, o), 3);
    assert_eq!(splits_for(100.0, 1, o), 10);
    // ⛔ Um desvio que não é um número não pode virar um `k` gigante.
    assert_eq!(splits_for(f64::NAN, 1, o), 1);
    assert_eq!(splits_for(f64::INFINITY, 1, o), 1);
}

/// ⛔⛔⛔ **O ORÇAMENTO DE PEÇAS É HONRADO, E ELE É O RECURSO QUE O RENDERER PAGA.**
///
/// > *«Smooth bugado quebrando a forma»* — report do dono, 2026-09-10, com foto.
///
/// ⚠️⚠️ **O tecto estava na grandeza ERRADA.** Ele limitava o `k`, e o que o renderer paga é a
/// **contagem de recortes**: cada triângulo é um `push_clip` do Vello, que dimensiona os buffers
/// dele por heurística e **degrada em silêncio** quando estouram. Um tecto no `k` é quadrático na
/// contagem, e a malha de partida pode ter qualquer tamanho ⇒ *o mesmo `k = 6` custa `7 776` peças
/// numa malha de 216 e `36` numa de 1.*
///
/// ⭐ A experiência que o report deu, sem querer: com o braço quase **recto** (`2°`) o desvio já é
/// `0,499 px`, logo o `k` saturava e a malha ia a `7 776` peças **para desenhar a mesma coisa que
/// o `Fast` desenha em 216** — e partia. *A malha estava provadamente correcta* (área conservada
/// ao cêntimo, zero triângulos saltados, zero arestas com mais de dois donos), o que é exactamente
/// o que aponta o dedo ao consumidor.
///
/// (Mutação: o `clamp` do orçamento desaparecer ⇒ RED.)
#[test]
fn the_piece_budget_is_honoured_because_it_is_what_the_renderer_pays() {
    let m = malha();
    for orcamento in [400usize, 1024, 4000] {
        let opts = RefineOptions {
            tolerance_px: 0.001,
            max_pieces: orcamento,
            adaptativo: false,
        };
        let mut campo = campo_dobrado(1.8);
        let (r, _p, rel) = refine_posed(&m, &mut campo, opts);
        let k = rel.k().expect("a lei uniforme declara o k");
        assert!(
            r.tris.len() <= orcamento,
            "com orcamento {orcamento} a malha saiu com {} pecas (k={k})",
            r.tris.len()
        );
        // ⚠️ E a metade que impede o gate de ficar verde sobre um produto que nunca refina: com um
        // orçamento folgado ele TEM de gastar o que pode.
        if orcamento >= m.tris.len() * 4 {
            assert!(k >= 2, "com orcamento {orcamento} o produto ficou em k={k}");
        }
    }
    // ⭐⭐⭐ **O caso que a CORRECÇÃO alcança**, e que nenhuma fixtura suave produz: com o campo
    // que aliasa, o primeiro `k` sai `5` e a conferência pede **`19`** — o tecto tem de o segurar.
    // *Sem isto, um campo patológico furava o orçamento pelo caminho de dentro.*
    for orcamento in [216 * 36, 216 * 9] {
        let opts = RefineOptions {
            tolerance_px: 0.5,
            max_pieces: orcamento,
            adaptativo: false,
        };
        let mut campo = campo_traicoeiro();
        let (r, _p, rel) = refine_posed(&m, &mut campo, opts);
        assert!(
            r.tris.len() <= orcamento,
            "o campo traicoeiro furou o orcamento {orcamento}: {} pecas ({:?})",
            r.tris.len(),
            rel.lei
        );
    }
    // ⛔ Um orçamento menor que a própria malha não pode partir nada — nem entrar em pânico.
    let apertado = RefineOptions {
        tolerance_px: 0.001,
        max_pieces: 10,
        adaptativo: false,
    };
    let mut campo = campo_dobrado(1.8);
    let (r, _p, rel) = refine_posed(&m, &mut campo, apertado);
    assert_eq!(rel.k(), Some(1), "sem orcamento para uma peca a mais, k=1");
    assert_eq!(r.tris.len(), m.tris.len());
    assert!(
        rel.travado_pelo_orcamento,
        "o relatorio tem de DIZER que foi o orcamento — e' esta a linha que separa «nao precisou» \
         de «nao pode»"
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
    let (r, _posed, rel) = refine_posed(
        &m,
        &mut campo,
        RefineOptions {
            tolerance_px: 0.5,
            max_pieces: 216 * 36,
            adaptativo: false,
        },
    );
    assert!(
        rel.k().is_some_and(|k| k > 1),
        "a fixtura tem de pedir refinamento"
    );
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

// ─────────────────────────────────────────────────────────────────────────────────────────────
// Os ATRIBUTOS que viajam na subdivisão — o que o padrão-ouro dos pesos de pele exige.
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// O valor baricêntrico EXACTO de `attrs` no ponto `p`, calculado na malha ORIGINAL.
///
/// ⚠️ **Ele localiza o ponto por força bruta**, que é precisamente o que o produto NÃO faz (`O(n)`
/// por ponto) — e é por isso que ele serve de oráculo: *a régua e a lei chegam à mesma resposta por
/// caminhos diferentes*. A lei usa a proveniência que a subdivisão já conhece; a régua procura.
fn baricentrico_na_malha(m: &Mesh2d, attrs: &[f64], p: [f64; 2]) -> Option<f64> {
    let mut melhor: Option<(f64, f64)> = None;
    for t in &m.tris {
        let (a, b, c) = (
            m.rest[t[0] as usize],
            m.rest[t[1] as usize],
            m.rest[t[2] as usize],
        );
        let den = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
        if den.abs() < 1e-12 {
            continue;
        }
        let u = ((p[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (p[1] - a[1])) / den;
        let v = ((b[0] - a[0]) * (p[1] - a[1]) - (p[0] - a[0]) * (b[1] - a[1])) / den;
        let fora = (-u).max(-v).max(u + v - 1.0).max(0.0);
        let val = (1.0 - u - v) * attrs[t[0] as usize]
            + u * attrs[t[1] as usize]
            + v * attrs[t[2] as usize];
        if melhor.is_none_or(|(f, _)| fora < f) {
            melhor = Some((fora, val));
        }
    }
    melhor.filter(|(f, _)| *f < 1e-9).map(|(_, v)| v)
}

/// ⭐⭐⭐ **UM VÉRTICE INVENTADO HERDA O ATRIBUTO DO TRIÂNGULO QUE O GEROU** — a lei inteira do
/// [`crate::refine_posed_attrs`].
///
/// ⚠️⚠️ **O atributo desta fixtura NÃO é linear na posição, e é de propósito:** uma função linear é
/// interpolada igual em QUALQUER triângulo, então ela não distingue *«usou a proveniência certa»*
/// de *«usou o triângulo ao lado»*. O chapéu de um vértice só é reproduzido por quem interpola nos
/// vértices certos.
///
/// ⛔⛔ **E a tolerância tem de puxar o `k` a `4` ou mais — MEDIDO, por uma mutação que SOBREVIVEU.**
/// A 1.ª redacção corria a `0,5 px` e o estimador parava em `k = 3`, onde o **único** nó de miolo de
/// cada triângulo é o `(1,1)` ⇒ `u = v = ⅓`. Trocar `t[1]` por `t[2]` na interpolação baricêntrica
/// é, ali, a **identidade algébrica** — a mutação passava com o gate verde. *Uma grelha de `k = 3`
/// não tem um único ponto interior onde as duas coordenadas baricêntricas difiram, logo nenhuma
/// fixtura nesse `k` pode distinguir os dois vértices.* A `0,12 px` o `k` sai `6`, e os nós `(1,2)`
/// e `(2,1)` matam-na.
///
/// (Mutação: no `No::Miolo` trocar `t[1]` por `t[2]` ⇒ RED; devolver o atributo do canto `t[0]` em
/// vez de interpolar ⇒ RED.)
#[test]
fn an_invented_vertex_inherits_the_attribute_of_the_triangle_that_made_it() {
    let m = malha();
    // O chapéu de um vértice do MIOLO da cápsula: `1` nele, `0` em todo o resto.
    let alvo = m
        .rest
        .iter()
        .enumerate()
        .min_by(|a, b| {
            let d = |p: &[f64; 2]| (p[0] - 160.0).hypot(p[1] - 48.0);
            d(a.1).total_cmp(&d(b.1))
        })
        .map(|(i, _)| i)
        .expect("a malha tem vertices");
    let attrs: Vec<f64> = (0..m.rest.len())
        .map(|v| f64::from(u8::from(v == alvo)))
        .collect();

    let mut base = campo_dobrado(1.8);
    let (r, _p, saida, rel) = crate::refine_posed_attrs(
        &m,
        &attrs,
        1,
        &mut |q, _| base(q),
        RefineOptions {
            tolerance_px: 0.12,
            max_pieces: m.tris.len() * 36,
            adaptativo: false,
        },
    );
    let k = rel.k().expect("a lei uniforme declara o k");
    // ⛔ `4`, e não `1`: ver o cabeçalho — a `k = 3` a mutação do miolo é a identidade.
    assert!(
        k >= 4,
        "a fixtura tem de chegar a k>=4 para ter um no' de miolo com u != v (k={k})"
    );
    assert_eq!(
        saida.len(),
        r.rest.len(),
        "um atributo por vertice refinado"
    );

    let mut pior = 0.0_f64;
    let mut medidos = 0usize;
    for (v, &q) in r.rest.iter().enumerate() {
        let Some(esperado) = baricentrico_na_malha(&m, &attrs, q) else {
            continue;
        };
        medidos += 1;
        pior = pior.max((saida[v] - esperado).abs());
    }
    assert!(
        medidos * 2 > r.rest.len(),
        "o oraculo so' localizou {medidos} de {} vertices",
        r.rest.len()
    );
    assert!(
        pior < 1e-12,
        "um vertice inventado recebeu um atributo que nao e' o do triangulo dele (pior {pior:.3e})"
    );
    // ⚠️ E o chapéu tem de CHEGAR a algum lado: com tudo a zero o gate acima passa por vacuidade.
    let maior = saida.iter().copied().fold(0.0_f64, f64::max);
    assert!(
        maior > 0.99,
        "o chapeu nao sobreviveu a subdivisao ({maior})"
    );
}

/// ⭐⭐⭐ **LEVAR ATRIBUTOS NÃO MEXE NA GEOMETRIA** — a metade que impede esta wave de tocar no que
/// já shipa.
///
/// A malha, as posições e o `k` de [`crate::refine_posed_attrs`] têm de ser os de
/// [`crate::refine_posed`] **ao bit**, com atributos ou sem eles. *Um atributo é carga, nunca uma
/// voz na decisão.*
#[test]
fn carrying_attributes_does_not_move_a_single_vertex() {
    let m = malha();
    // ⭐ **As DUAS leis**: a promessa é da porta, e ela despacha para as duas.
    for adaptativo in [false, true] {
        let opts = RefineOptions {
            tolerance_px: 0.5,
            max_pieces: m.tris.len() * 36,
            adaptativo,
        };
        let mut a = campo_dobrado(1.8);
        let (m0, p0, r0) = refine_posed(&m, &mut a, opts);

        let attrs: Vec<f64> = (0..m.rest.len() * 3)
            .map(|i| (i % 7) as f64 / 7.0)
            .collect();
        let mut b = campo_dobrado(1.8);
        let (m1, p1, saida, r1) = crate::refine_posed_attrs(&m, &attrs, 3, &mut |q, _| b(q), opts);

        assert_eq!(r0.lei, r1.lei, "a lei mudou por levar carga");
        assert_eq!(
            r0.pecas, r1.pecas,
            "a contagem de pecas mudou por levar carga"
        );
        assert_eq!(m0.rest, m1.rest, "as posicoes de repouso mudaram");
        assert_eq!(m0.tris, m1.tris, "os triangulos mudaram");
        assert_eq!(p0, p1, "as posicoes posadas mudaram");
        assert_eq!(saida.len(), m1.rest.len() * 3, "stride errado na saida");
    }
}

/// ⛔ **SEM ATRIBUTOS, A PORTA NOVA É A ANTIGA** — `stride = 0` não inventa carga nenhuma.
#[test]
fn with_no_attributes_the_new_door_carries_nothing() {
    let m = malha();
    let mut campo = campo_dobrado(1.8);
    let (_, _, saida, _) = crate::refine_posed_attrs(
        &m,
        &[],
        0,
        &mut |q, a| {
            assert!(a.is_empty(), "o campo recebeu atributos que ninguem deu");
            campo(q)
        },
        RefineOptions {
            tolerance_px: 0.5,
            max_pieces: m.tris.len() * 36,
            ..RefineOptions::default()
        },
    );
    assert!(saida.is_empty(), "a saida inventou atributos");
}

#[path = "refine_adaptive_tests.rs"]
mod adaptativo;

#[path = "refine_rest_tests.rs"]
mod refine_rest_tests;

#[path = "attr_law_tests.rs"]
mod lei_dos_atributos;
