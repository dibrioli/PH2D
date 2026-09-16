//! ⭐⭐⭐ **OS GATES DA LEI ADAPTATIVA** — irmão do [`super`] pelo tecto de LOC, cortado por
//! RESPONSABILIDADE: *o `k` global honra o orçamento* é uma pergunta, *a bissecção da aresta mais
//! longa refina só onde a dobra pede* é outra.
//!
//! ⚠️ Filho do arnês do irmão (`#[path]`) para herdar as fixturas dele — uma cópia divergiria no
//! primeiro ajuste, e a `malha()` prega a densidade de propósito (ver o doc dela).

use super::*;
use crate::{AttrLaw, RefineLaw, refine_posed_adaptive};

/// As opções da lei adaptativa com a tolerância e o orçamento que o caso pede.
fn adapt(tolerance_px: f64, max_pieces: usize) -> RefineOptions {
    RefineOptions {
        tolerance_px,
        max_pieces,
        adaptativo: true,
    }
}

/// ⭐⭐⭐ **UMA JUNTA — a metade esquerda parada, a direita rodada, e a curvatura TODA numa banda.**
///
/// ⛔⛔ **Ela existe porque a primeira medição desta wave mediu a pergunta errada.** O
/// [`campo_dobrado`] do irmão roda cada ponto por um ângulo **proporcional a `x`**, logo a arte
/// inteira curva — e sobre um campo que curva por igual em todo o lado a lei uniforme está quase
/// óptima: a adaptativa compra só `1,9×`. *A vantagem dela não é «partir menos»; é partir
/// **onde**.*
///
/// ⇒ o produto é o outro caso, e é este: numa pele de esqueleto os pesos são ~constantes ao longo
/// de um osso e viram **todos** na articulação. Fora da banda o campo é um movimento RÍGIDO, que um
/// afim reproduz **exactamente** — desvio zero, e refinar ali é dinheiro deitado fora.
fn campo_articulado(t: f64) -> impl FnMut([f64; 2]) -> [f64; 2] {
    // A largura da junta, em pixels da arte — `320×96`, logo a banda é `~8 %` da peça.
    const BANDA: f64 = 12.0;
    move |p: [f64; 2]| {
        let u = ((p[0] - 160.0) / BANDA).clamp(-1.0, 1.0);
        let w = f64::midpoint(u, 1.0);
        // Um `smoothstep`: a derivada anula-se nas pontas, então fora da banda o campo é RÍGIDO de
        // verdade — e não «quase».
        let s = w * w * (3.0 - 2.0 * w) * t;
        let (sin, cos) = (s.sin(), s.cos());
        let (x, y) = (p[0] - 160.0, p[1] - 48.0);
        [
            (x * cos - y * sin) + 160.0,
            (x * sin).mul_add(1.0, y * cos) + 48.0,
        ]
    }
}

/// O censo de arestas de uma malha: `aresta → quantos triângulos a possuem`.
fn arestas(m: &Mesh2d) -> BTreeMap<(u32, u32), usize> {
    let mut out: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for t in &m.tris {
        for i in 0..3 {
            let (a, b) = (t[i], t[(i + 1) % 3]);
            *out.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    out
}

/// ⭐⭐⭐ **A MALHA ADAPTATIVA NÃO TEM NÓ PENDURADO — é a afirmação inteira desta lei.**
///
/// ⛔⛔ **É a metade que o cabeçalho da [`crate::refine`] dizia ser impossível** (*«um `k` por
/// triângulo abriria nós pendurados»*), e a razão de ela ser verdade é a operação: parte-se uma
/// **ARESTA**, e os dois donos dela partem-se no mesmo acto. ⇒ a malha é conforme **depois de cada
/// passo**, não só no fim.
///
/// ⚠️ **Um nó pendurado apresenta-se como uma aresta de BORDO no meio da peça** — é por isso que a
/// prova não é «toda aresta tem ≤ 2 donos» (isso um erro de índice também satisfaz) e sim a
/// contagem do bordo contra o perímetro que a malha de entrada já tinha.
///
/// (Mutação: em `parte_aresta`, partir só o PRIMEIRO dono ⇒ RED, com o bordo a explodir.)
#[test]
fn the_adaptive_mesh_has_no_hanging_nodes() {
    let m = malha();
    let bordo0 = arestas(&m).values().filter(|n| **n == 1).count();
    let mut campo = campo_dobrado(1.8);
    let (r, posed, rel) = refine_posed(&m, &mut campo, adapt(0.5, m.tris.len() * 8));

    assert!(
        matches!(rel.lei, RefineLaw::Adaptive { rondas } if rondas > 0),
        "a fixtura tem de pedir refinamento: {:?}",
        rel.lei
    );
    assert_eq!(r.rest.len(), posed.len());

    // (a) Nenhuma posição de repouso nasce duas vezes — um ponto de aresta é UM vértice, e não um
    // por dono.
    let mut vistos: BTreeMap<(u64, u64), usize> = BTreeMap::new();
    for p in &r.rest {
        *vistos.entry((p[0].to_bits(), p[1].to_bits())).or_default() += 1;
    }
    let duplicados = vistos.values().filter(|n| **n > 1).count();
    assert_eq!(duplicados, 0, "{duplicados} posicoes nasceram DUAS vezes");

    // (b) Nenhuma aresta com três donos, e o bordo NÃO cresce: uma bissecção parte uma aresta de
    // bordo em duas, logo o perímetro conta no máximo o dobro por nível — nunca uma ordem de
    // grandeza, que é o que um lado não-partido produziria.
    let censo = arestas(&r);
    assert!(
        censo.values().all(|n| *n <= 2),
        "ha' aresta com mais de dois triangulos"
    );
    let bordo = censo.values().filter(|n| **n == 1).count();
    let interior = censo.values().filter(|n| **n == 2).count();
    assert!(
        bordo <= bordo0 * 4,
        "o bordo foi de {bordo0} para {bordo} — ha' arestas partidas de um lado so'"
    );
    assert!(
        interior > bordo * 2,
        "bordo {bordo} contra interior {interior} — a malha desfez-se"
    );
}

/// ⭐⭐⭐⭐ **O DEFEITO QUE ESTA LEI CURA: onde a uniforme é INERTE, esta entrega a tolerância.**
///
/// > *«o botão Smooth não faz nada»* — auditoria de 2026-09-16.
///
/// ⚠️⚠️ **A fixtura é o caso REAL, e não um caso patológico:** uma malha de `216` peças com um
/// orçamento de `400` dá `max_split = ⌊√(400/216)⌋ = 1`, logo o `Smooth` uniforme devolve o `Fast`
/// **ao bit** com a tolerância a não decidir nada. É a mesma aritmética que na cena do braço
/// (`780` peças, orçamento `1 543`) e na malha de bind (`2 430` peças, orçamento `3 787`).
///
/// (Mutação: o despachante ignorar `opts.adaptativo` e chamar sempre a uniforme ⇒ RED.)
#[test]
fn where_the_uniform_law_is_inert_the_adaptive_one_delivers() {
    let m = malha();
    let orcamento = 400;
    assert_eq!(
        crate::max_split(m.tris.len(), adapt(0.5, orcamento)),
        1,
        "a fixtura tem de reproduzir a ARITMETICA do defeito, senao mede outra coisa"
    );

    let mut c0 = campo_articulado(1.8);
    let posed0: Vec<[f64; 2]> = m.rest.iter().map(|&p| c0(p)).collect();
    let cru = deviation(&m, &posed0, &mut c0);

    // A lei uniforme, no mesmo orçamento: byte-idêntica à entrada.
    let mut cu = campo_articulado(1.8);
    let (mu, pu, ru) = refine_posed(
        &m,
        &mut cu,
        RefineOptions {
            adaptativo: false,
            ..adapt(0.5, orcamento)
        },
    );
    assert_eq!(mu, m, "a premissa: aqui a lei uniforme NAO refina");
    let du = deviation(&mu, &pu, &mut cu);
    assert!(
        (du - cru).abs() < 1e-12,
        "a uniforme mexeu no desvio ({cru:.3} -> {du:.3}) — a premissa do gate caiu"
    );
    assert!(
        ru.travado_pelo_orcamento,
        "e ela tem de DIZER que foi o orcamento"
    );

    // A adaptativa, no MESMO orçamento.
    let mut ca = campo_articulado(1.8);
    let (ma, pa, ra) = refine_posed(&m, &mut ca, adapt(0.5, orcamento));
    let da = deviation(&ma, &pa, &mut ca);
    assert!(
        ma.tris.len() <= orcamento,
        "a adaptativa furou o orcamento: {} pecas",
        ma.tris.len()
    );
    // ⚠️⚠️ **A barra é uma RAZÃO e não «dentro da tolerância»**: com `400` peças de tecto ela pode
    // não chegar a `0,5 px`, e exigir isso mediria o ORÇAMENTO em vez da lei. O que se afirma é o
    // que o dono vê — *a mesma conta de peças compra uma dobra visivelmente mais lisa*.
    //
    // ⛔ **`4×` sai do vale MEDIDO**: nesta fixtura a uniforme lê `29,860 → 29,860 px` (`1,00×`,
    // inerte por construção) e a adaptativa `29,860 → 2,519 px` em `399` das `400` peças —
    // **`11,9×`**. A barra fica longe dos dois lados, e não é um número congelado de uma corrida.
    assert!(
        da * 4.0 < cru,
        "no mesmo orcamento a adaptativa so' levou o desvio de {cru:.3} para {da:.3} px ({:?})",
        ra.lei
    );
    println!(
        "cru {cru:.3} px | uniforme {du:.3} px em {} pecas | adaptativa {da:.3} px em {} pecas",
        mu.tris.len(),
        ma.tris.len()
    );
}

/// ⭐⭐⭐ **A MESMA TOLERÂNCIA CUSTA UMA FRACÇÃO DAS PEÇAS** — é o número que justifica a wave.
///
/// ⚠️ **A régua é o PREÇO da mesma resposta**, e não a qualidade a preço igual: dá-se às duas leis
/// um orçamento folgado e a mesma tolerância, e compara-se quantas peças cada uma gastou para lá
/// chegar. *Comparar qualidade a orçamento fixo mediria o orçamento.*
///
/// ⚠️⚠️ **E O GANHO É FUNÇÃO DA ARTE, não uma constante — foi a primeira medição desta wave que o
/// disse, contra a minha premissa.** Sobre um campo que curva **por igual em todo o lado** a lei
/// uniforme está quase óptima, e a adaptativa compra pouco. O ganho grande está no caso do
/// PRODUTO, em que a curvatura vive nas articulações ([`campo_articulado`]).
///
/// Medido (cápsula de `216` peças, orçamento folgado de `13 824`):
///
/// | fixtura | tol | uniforme | adaptativa | razão |
/// |---|---:|---:|---:|---:|
/// | dobra global | `1,00 px` | `864` (`0,980`) | `444` (`0,996`) | `1,9×` |
/// | dobra global | `0,50 px` | `1 944` (`0,439`) | `1 025` (`0,499`) | `1,9×` |
/// | dobra global | `0,25 px` | `3 456` (`0,248`) | `1 700` (`0,250`) | `2,0×` |
/// | **junta** | `1,00 px` | `13 824` (**`1,121`**) | `596` (`0,999`) | **`23,2×`** |
/// | **junta** | `0,50 px` | `13 824` (**`1,121`**) | `1 220` (`0,499`) | **`11,3×`** |
/// | **junta** | `0,25 px` | `13 824` (**`1,121`**) | `1 936` (`0,249`) | **`7,1×`** |
///
/// ⛔⛔ **E a coluna do desvio diz uma coisa mais forte que a das peças: na junta a uniforme NUNCA
/// CHEGA.** Ela satura o orçamento (`k = 8`, `216 × 8² = 13 824`) e fica em `1,121 px` nas três
/// tolerâncias — logo as razões acima são um **piso**: o custo verdadeiro dela é `+∞` dentro deste
/// orçamento.
///
/// ⛔ Por isso o gate exige **`≥ 1,5×`** na dobra global (o **piso** honesto, onde a lei nova não
/// pode ser pior) e **`≥ 4×`** na junta (o caso que o produto tem).
#[test]
fn the_adaptive_law_buys_the_same_tolerance_for_a_fraction_of_the_pieces() {
    let m = malha();
    let folgado = m.tris.len() * 64;
    // `(nome, razão mínima exigida)` — a dobra global é o PISO, a junta é o caso do produto.
    for (nome, minimo) in [("dobra global", 1.5_f64), ("junta", 4.0)] {
        let junta = nome == "junta";
        let faz = |t: f64| -> Box<dyn FnMut([f64; 2]) -> [f64; 2]> {
            if junta {
                Box::new(campo_articulado(t))
            } else {
                Box::new(campo_dobrado(t))
            }
        };
        for tol in [1.0, 0.5, 0.25] {
            let mut cu = faz(1.8);
            let (mu, pu, _) = refine_posed(
                &m,
                &mut cu,
                RefineOptions {
                    adaptativo: false,
                    ..adapt(tol, folgado)
                },
            );
            let du = deviation(&mu, &pu, &mut cu);

            let mut ca = faz(1.8);
            let (ma, pa, _) = refine_posed(&m, &mut ca, adapt(tol, folgado));
            let da = deviation(&ma, &pa, &mut ca);

            #[expect(clippy::cast_precision_loss, reason = "contagens de peças")]
            let razao = mu.tris.len() as f64 / ma.tris.len() as f64;
            println!(
                "{nome} tol {tol:.2} px | uniforme {} ({du:.3}) | adaptativa {} ({da:.3}) | {razao:.1}x",
                mu.tris.len(),
                ma.tris.len()
            );
            assert!(
                da <= tol,
                "{nome}: a adaptativa nao honrou a tolerancia {tol:.2} ({da:.3} px)"
            );
            assert!(
                razao >= minimo,
                "{nome} tol {tol:.2}: {} pecas contra {} da uniforme ({razao:.1}x) — abaixo de \
                 {minimo:.1}x o ganho sumiu",
                ma.tris.len(),
                mu.tris.len()
            );
        }
    }
}

/// ⭐⭐⭐ **ELA PARTE ONDE A DOBRA ESTÁ, E DEIXA O RESTO EM PAZ** — é isto que a torna barata.
///
/// ⚠️ **A régua é a DENSIDADE (peças por unidade de largura), e não a contagem crua** — a banda da
/// junta é `~15 %` da peça, então uma contagem crua acusaria a lei certa de não escolher nada.
///
/// ⚠️⚠️ **E a fixtura tem de ser a JUNTA.** Com o [`campo_dobrado`] — que curva a arte inteira —
/// esta régua lê `611` contra `1 089` e a lei está **certa**: ali não há sítio liso nenhum. *Uma
/// régua de localidade sobre um campo sem localidade mede a fixtura.*
///
/// Medido na junta: `1 722` peças em `48 px` de banda (`35,88`/px) contra `214` nos `272 px`
/// restantes (`0,79`/px) — **`45,6×` mais densa**. A barra é `4×`, longe do valor medido.
///
/// (Mutação: a fila entregar sempre o PRIMEIRO triângulo em vez do de pior desvio ⇒ RED.)
#[test]
fn the_adaptive_law_spends_where_the_bend_is() {
    let m = malha();
    let mut campo = campo_articulado(1.8);
    let (r, _p, _rel) = refine_posed(&m, &mut campo, adapt(0.25, m.tris.len() * 64));

    // O centroide de cada peça decide se ela caiu na banda da junta.
    let centro_x = |t: &[u32; 3]| -> f64 {
        (r.rest[t[0] as usize][0] + r.rest[t[1] as usize][0] + r.rest[t[2] as usize][0]) / 3.0
    };
    // ⚠️ O meio-larguras saem da PRÓPRIA fixtura (junta em `x = 160`, banda de `12 px`, arte de
    // `320`) — nenhum número escrito à mão aqui.
    const MEIA_BANDA: f64 = 24.0;
    let na_banda = r
        .tris
        .iter()
        .filter(|t| (centro_x(t) - 160.0).abs() < MEIA_BANDA)
        .count();
    let fora = r.tris.len() - na_banda;
    #[expect(clippy::cast_precision_loss, reason = "contagens de peças")]
    let (d_banda, d_fora) = (
        na_banda as f64 / (2.0 * MEIA_BANDA),
        fora as f64 / (320.0 - 2.0 * MEIA_BANDA),
    );
    println!(
        "junta {na_banda} pecas ({d_banda:.2}/px) | resto {fora} pecas ({d_fora:.2}/px) | \
         {:.1}x mais densa",
        d_banda / d_fora.max(f64::MIN_POSITIVE)
    );
    assert!(
        d_banda > d_fora * 4.0,
        "densidade na junta {d_banda:.2}/px contra {d_fora:.2}/px fora — a lei nao esta' a escolher"
    );
}

/// ⛔⛔⛔ **O ORÇAMENTO É HONRADO, E UM ORÇAMENTO IMPOSSÍVEL NÃO PARTE NADA NEM ESTOURA.**
///
/// ⚠️ **E o relatório tem de DIZER que foi o orçamento** — é esta linha que separa *«a dobra não
/// pediu nada»* de *«o quadro não paga»*, e a falta dela foi o que deixou o `Smooth` morto durante
/// um bloco inteiro sem ninguém ver.
#[test]
fn the_adaptive_law_honours_the_piece_budget_and_says_when_it_stopped() {
    let m = malha();
    for orcamento in [m.tris.len(), 300, 500, 2000] {
        let mut campo = campo_dobrado(1.8);
        let (r, _p, rel) = refine_posed(&m, &mut campo, adapt(0.001, orcamento));
        assert!(
            r.tris.len() <= orcamento,
            "com orcamento {orcamento} saiu com {} pecas",
            r.tris.len()
        );
        assert_eq!(r.tris.len(), rel.pecas, "o relatorio conta outra malha");
        assert!(
            rel.travado_pelo_orcamento,
            "com tolerancia 0,001 px o orcamento {orcamento} TEM de travar"
        );
    }
    // ⛔ Um orçamento menor que a própria malha devolve a malha, sem partir e sem entrar em pânico.
    let mut campo = campo_dobrado(1.8);
    let (r, _p, rel) = refine_posed(&m, &mut campo, adapt(0.001, 10));
    assert_eq!(r.tris, m.tris, "sem orcamento nao se parte nada");
    assert!(rel.travado_pelo_orcamento);
}

/// ⭐⭐ **CADA ARESTA É PERGUNTADA AO CAMPO UMA VEZ SÓ** — medir o desvio e partir são a MESMA conta.
///
/// ⚠️⚠️ **A razão é `perguntas / posições distintas`, e não a contagem crua.** Sem a partilha do
/// meio por aresta, os dois donos perguntariam o mesmo ponto — e as posições distintas **não**
/// subiriam, logo a razão é a régua que vê o defeito.
///
/// ⛔ **A barra `1,05` sai do vale medido:** com a partilha lê-se `1,00` (cada ponto é perguntado
/// exactamente uma vez); apagando o memo dos meios lê-se `~1,9`.
///
/// (Mutação: `garante_meio` deixar de consultar o mapa antes de medir ⇒ RED.)
#[test]
fn the_adaptive_law_asks_the_field_once_per_point() {
    let m = malha();
    let mut vistos: BTreeMap<(u64, u64), usize> = BTreeMap::new();
    let mut base = campo_dobrado(1.8);
    let mut campo = |p: [f64; 2]| {
        *vistos.entry((p[0].to_bits(), p[1].to_bits())).or_default() += 1;
        base(p)
    };
    let (r, _p, rel) = refine_posed(&m, &mut campo, adapt(0.5, m.tris.len() * 16));
    assert!(matches!(rel.lei, RefineLaw::Adaptive { rondas } if rondas > 0));
    let perguntas: usize = vistos.values().sum();
    #[expect(
        clippy::cast_precision_loss,
        reason = "contagens de uma malha de imagem"
    )]
    let razao = perguntas as f64 / vistos.len() as f64;
    println!(
        "{perguntas} perguntas para {} posicoes distintas ({razao:.2}x); malha {} pecas",
        vistos.len(),
        r.tris.len()
    );
    assert!(
        razao < 1.05,
        "{perguntas} perguntas para {} posicoes ({razao:.2}x) — o memo dos meios nao segura",
        vistos.len()
    );
}

/// ⭐⭐⭐ **UM VÉRTICE INVENTADO HERDA O ATRIBUTO CERTO, também nesta lei** — o padrão-ouro dos pesos
/// viaja pela subdivisão.
///
/// ⚠️ **O oráculo é o mesmo do irmão** ([`baricentrico_na_malha`]): ele LOCALIZA o ponto por força
/// bruta na malha original, que é o caminho que a lei não usa. *A régua e a lei chegam à mesma
/// resposta por caminhos diferentes.*
///
/// ⛔ Aqui a prova é mais forte que na uniforme: um ponto de bissecção está **sempre** numa aresta,
/// e a média das pontas é o baricêntrico exacto dela — logo qualquer troca de vértice na
/// interpolação sai do valor certo.
///
/// (Mutação: `attrs_do_meio` usar só a ponta `e.0` ⇒ RED.)
#[test]
fn an_invented_vertex_inherits_the_attribute_under_the_adaptive_law() {
    let m = malha();
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
        adapt(0.12, m.tris.len() * 36),
    );
    assert!(matches!(rel.lei, RefineLaw::Adaptive { rondas } if rondas > 0));
    assert_eq!(saida.len(), r.rest.len(), "um atributo por vertice");

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
        "um vertice inventado recebeu o atributo errado (pior {pior:.3e})"
    );
    let maior = saida.iter().copied().fold(0.0_f64, f64::max);
    assert!(
        maior > 0.99,
        "o chapeu nao sobreviveu a subdivisao ({maior})"
    );
}

/// ⛔⛔ **A ORDEM DAS ARESTAS É TOTAL, E É ISSO QUE FAZ A CADEIA TERMINAR.**
///
/// ⚠️⚠️ **Numa malha de GRELHA todos os empates existem por construção** — toda célula tem a mesma
/// diagonal, então a cadeia de vizinhos pode voltar a um triângulo já visitado e girar para
/// sempre. O desempate pela chave canónica é o que o impede, e o laço traz um `debug_assert` que
/// **dispara** se a cadeia exceder a malha viva.
///
/// ⭐ Este gate corre em DEBUG (é onde os testes correm), logo a mutação que apaga o desempate é
/// apanhada pelo assert e não por um relógio.
///
/// (Mutação: em `maior`, devolver só `a.0 > b.0` ⇒ o `debug_assert` dispara ⇒ RED.)
#[test]
fn the_edge_order_is_total_so_the_chain_terminates_on_a_grid() {
    // As duas malhas, e a permutada de propósito: ela troca a numeração dos vértices, que é
    // exactamente o que o desempate lê.
    for m in [malha(), malha_permutada()] {
        let mut campo = campo_dobrado(1.8);
        let (r, _p, rel) = refine_posed(&m, &mut campo, adapt(0.2, m.tris.len() * 32));
        assert!(matches!(rel.lei, RefineLaw::Adaptive { rondas } if rondas > 0));
        assert!(r.tris.len() > m.tris.len());
    }
}

/// ⛔ **A MESMA ENTRADA DÁ A MESMA SAÍDA, AO BIT** — a fila tem desempate por índice e os mapas são
/// ordenados, logo não há sorteio nenhum aqui.
///
/// ⚠️ Sem isto o desenho de um quadro podia mudar entre corridas sem nada ter mudado, que é a
/// espécie de defeito que se lê como «o app pisca».
#[test]
fn the_adaptive_law_is_deterministic() {
    let m = malha();
    let opts = adapt(0.3, m.tris.len() * 16);
    let mut a = campo_dobrado(1.8);
    let (m0, p0, r0) = refine_posed(&m, &mut a, opts);
    let mut b = campo_dobrado(1.8);
    let (m1, p1, r1) = refine_posed(&m, &mut b, opts);
    assert_eq!(m0, m1, "duas corridas deram malhas diferentes");
    assert_eq!(p0, p1, "duas corridas deram posicoes diferentes");
    assert_eq!(r0, r1, "duas corridas deram relatorios diferentes");
}

/// ⛔ **A PORTA DIRECTA E A DESPACHADA SÃO A MESMA COISA** — o `opts.adaptativo` não é decoração.
///
/// ⚠️ Sem este gate, o campo das opções podia deixar de ser lido e nada acusaria: o produto
/// continuaria a chamar a porta e a receber *uma* das leis. *É exactamente a forma do controlo
/// morto que o `CLAUDE.md` §5.0 chama de «consumidor que projecta o valor fora».*
#[test]
fn the_options_flag_is_what_chooses_the_law() {
    let m = malha();
    let opts = adapt(0.3, m.tris.len() * 16);
    let mut a = campo_dobrado(1.8);
    let (m0, _, r0) = refine_posed(&m, &mut a, opts);
    let mut b = campo_dobrado(1.8);
    let (m1, _, _, r1) = refine_posed_adaptive(&m, &[], 0, AttrLaw::Linear, &mut |q, _| b(q), opts);
    assert_eq!(m0, m1, "a porta despachou para outra lei");
    assert_eq!(r0, r1);

    let mut c = campo_dobrado(1.8);
    let (m2, _, r2) = refine_posed(
        &m,
        &mut c,
        RefineOptions {
            adaptativo: false,
            ..opts
        },
    );
    assert!(
        matches!(r2.lei, RefineLaw::Uniform { .. }),
        "com a bandeira em falso tem de correr a lei uniforme"
    );
    assert_ne!(
        m0.tris.len(),
        m2.tris.len(),
        "as duas leis deram a MESMA contagem — a bandeira nao esta' a escolher nada"
    );
}
