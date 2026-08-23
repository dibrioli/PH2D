//! ⭐⭐⭐ **OS DOIS CONTROLOS DA HOLONOMIA** — e o gate que prova que a régua
//! anterior não podia dar a resposta que lhe foi atribuída.
//!
//! A fixtura é um **leque plano**: um vértice no centro, `N` no anel, `N` triângulos,
//! todos no plano `z = 0`. ⭐ *Plana de propósito* — numa malha curva o defeito
//! angular de Gauss soma-se ao do campo, e um controlo que mistura as duas causas não
//! é controlo de nenhuma. Aqui a normal é a mesma em toda a parte, então **tudo o que
//! a sonda ler vem do campo**.
//!
//! ⚠️ As faces incidentes num vértice interior formam **um ciclo no grafo dual**, e é
//! exactamente esse ciclo que a holonomia percorre. É por isso que um leque — a malha
//! mais pequena com um vértice interior — chega para medir uma singularidade.

use ph2d_mesh::{Face, Mesh};

use super::{Holonomy, holonomy};

/// Quantos triângulos tem o leque das fixturas.
const N: usize = 8;

/// O leque plano: centro em `0`, anel em `1..=N`, `N` triângulos CCW vistos de `+z`.
fn fan() -> Mesh {
    let mut pos = vec![[0.0f32, 0.0, 0.0]];
    for j in 0..N {
        #[allow(clippy::cast_precision_loss)]
        let a = core::f32::consts::TAU * j as f32 / N as f32;
        pos.push([a.cos(), a.sin(), 0.0]);
    }
    let faces: Vec<Face> = (0..N)
        .map(|j| {
            let (b, c) = (j + 1, (j + 1) % N + 1);
            #[allow(clippy::cast_possible_truncation)]
            Face::tri(0, b as u32, c as u32)
        })
        .collect();
    Mesh::from_parts(pos, faces).expect("o leque plano e' bem formado")
}

/// Um campo que roda `quarters` quartos de volta ao longo da volta completa.
///
/// ⭐ **`quarters = 0`** dá um campo constante (nenhuma singularidade no centro);
/// **`quarters = 1`** põe uma singularidade de índice `+¼` exactamente no vértice
/// interior. *Não há nada entre os dois: o índice é topológico.*
fn field(quarters: i32) -> Vec<[f32; 3]> {
    (0..N)
        .map(|j| {
            #[allow(clippy::cast_precision_loss)]
            let a = core::f32::consts::FRAC_PI_2 * quarters as f32 * j as f32 / N as f32;
            [a.cos(), a.sin(), 0.0]
        })
        .collect()
}

fn measure(quarters: i32) -> Holonomy {
    let mesh = fan();
    #[allow(clippy::cast_possible_truncation)]
    let faces: Vec<u32> = (0..N as u32).collect();
    holonomy(&mesh, &faces, &field(quarters)).expect("o leque plano e' penteavel")
}

/// ⭐ **O CONTROLO NEGATIVO.** Campo constante, nenhuma singularidade: o leque fecha
/// sem resíduo nenhum, e a rugosidade é zero exacto.
///
/// ⚠️ Repare no `cycles`: ele tem de ser `> 0`. *Uma sonda que não testasse ciclo
/// nenhum também diria `0 defeitos`, e passaria este gate por não fazer nada.*
#[test]
fn a_constant_field_closes_every_cycle_with_no_defect() {
    let h = measure(0);
    assert!(
        h.cycles > 0,
        "o leque tem de fechar pelo menos um ciclo, senao este gate passa por vacuidade: {h:?}"
    );
    assert_eq!(h.defects, 0, "campo constante nao tem singularidade: {h:?}");
    assert_eq!(h.turn_max, 0, "campo constante nao roda nada: {h:?}");
    assert!(h.rough_max < 1.0e-3, "campo constante e' liso: {h:?}");
}

/// ⭐⭐⭐ **O CONTROLO POSITIVO — e o gate que enterra a régua anterior.**
///
/// O campo dá **um quarto de volta** ao longo do anel: é uma singularidade de índice
/// `+¼` no vértice interior, por construção e não por acidente numérico.
///
/// | | o que a régua diz |
/// |---|---|
/// | ⭐ `defects` | `≥ 1` — **a singularidade é vista** |
/// | ⭐ `turn_max` | `1` — um quarto de volta, o valor exacto |
/// | ⛔ `rough_max` | `≈ 11,25°` — *indistinguível de campo rugoso* |
///
/// ⛔⛔ **A terceira linha é a prova, e ela é ANTI-CORRELACIONADA.** `11,25°` está
/// **abaixo** dos `29°`–`44°` que a sonda antiga leu nos patches reais e chamou de
/// singularidade — ou seja, a régua antiga dava a uma singularidade **de verdade** um
/// número *menor* do que dava a campo limpo mas irregular. *Ela não estava a medir mal
/// a coisa certa; estava a medir outra coisa, e no sentido oposto.*
///
/// ⭐ **Provado por mutação** (2026-08-23): repor o `raw[u]` da versão antiga no lugar
/// do braço penteado devolve, para esta mesma fixtura,
/// `rough_max: 11,25° · defects: 0` — e o gate fica vermelho. *A régua nova apanha
/// exactamente o defeito que a antiga tinha, nem mais nem menos.*
#[test]
fn a_quarter_turn_around_the_ring_is_seen_only_by_the_integer_ruler() {
    let h = measure(1);
    assert!(
        h.defects >= 1,
        "uma volta de 90 graus e' uma singularidade e tem de aparecer: {h:?}"
    );
    assert_eq!(
        h.turn_max, 1,
        "e' um quarto de volta, nem mais nem menos: {h:?}"
    );
    // ⭐ `90° / N` é o passo entre faces vizinhas, e é isso que sobra ao arredondar.
    #[allow(clippy::cast_precision_loss)]
    let step = 90.0 / N as f32;
    assert!(
        (h.rough_max - step).abs() < 0.5,
        "a rugosidade de uma singularidade real e' o passo entre faces ({step:.2} graus), \
         e e' por isso que ela nao a sabe nomear: {h:?}"
    );
}

/// ⛔ **O TECTO, executável.** A rugosidade é o resto depois de virar para o quarto
/// de volta mais próximo, logo ela **nunca** pode passar de `45°`.
///
/// ⚠️ Este gate não é decorativo: é a frase que teria poupado meia jornada. Enquanto
/// ninguém a escreveu, `29°` e `44°` leram-se como *«grande»* em vez de *«encostado
/// ao máximo que este número consegue imprimir»*.
///
/// ⭐ Mede as três voltas possíveis — `0`, `1` e `2` quartos — porque um tecto que só
/// se confirma no caso fácil não é um tecto.
#[test]
fn the_roughness_can_never_reach_the_quarter_turn_it_was_read_as() {
    for q in [0, 1, 2] {
        let h = measure(q);
        assert!(
            h.rough_max <= 45.0 + 1.0e-3,
            "a rugosidade e' limitada a meio quarto de volta por construcao (voltas={q}): {h:?}"
        );
    }
}

/// ⭐ **DUAS voltas leem-se como duas**, e não saturam em uma.
///
/// ⚠️ Um detector que dissesse só «sujo/limpo» empataria uma singularidade de índice
/// `+¼` com uma de `+½`, e é a segunda que nenhuma grade de quads contorna.
#[test]
fn a_half_turn_reads_as_two_quarters() {
    let h = measure(2);
    assert!(h.defects >= 1, "meia volta tambem e' singularidade: {h:?}");
    assert_eq!(h.turn_max, 2, "meia volta sao dois quartos: {h:?}");
}

/// ⭐⭐ **CADA ARESTA DUAL CONTA UMA VEZ.**
///
/// ⚠️ A versão antiga media dentro da travessia e passava por cada aresta **duas**
/// vezes, uma por sentido. Os percentis sobreviviam a isso; a coluna `edges` não, e
/// ela é o denominador de qualquer fracção que alguém venha a escrever daqui.
///
/// O leque de `N` triângulos tem exactamente `N` arestas duais — o anel fechado.
#[test]
fn every_dual_edge_is_counted_once() {
    let h = measure(0);
    assert_eq!(
        h.edges, N,
        "o leque de {N} triangulos tem {N} arestas duais, cada uma contada uma vez: {h:?}"
    );
    // ⭐ A árvore de uma travessia sobre `N` faces usa `N − 1` arestas; sobra **uma**,
    // e é ela que fecha o anel. *Se este número mudar, a árvore deixou de ser árvore.*
    assert_eq!(h.cycles, 1, "um anel fecha exactamente um ciclo: {h:?}");
}
