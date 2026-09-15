//! Os gates da [memória do contacto](super) — a fatia 1 da obra do doc 111.
//!
//! ⚠️ **O que eles defendem é uma coisa só:** *um `λ` guardado para um contacto tem de voltar para
//! ESSE contacto, e para mais nenhum.* O doc 111 §5 W2 escreve porquê: **um `λ` que aquece o
//! contacto errado é PIOR que não aquecer**, e nenhum gate da lei consumidora o apanharia — ela
//! veria um número plausível.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};

/// ⭐ **O que se guarda volta, e só para o parceiro certo.**
#[test]
fn a_remembered_lambda_comes_back_for_that_partner_and_no_other() {
    let mut m = Memoria::default();
    assert_eq!(m.lambda(7), 0.0, "uma memoria vazia arranca FRIA");
    m.guardar(7, 1.5);
    m.guardar(9, 0.25);
    assert_eq!(m.lambda(7), 1.5);
    assert_eq!(m.lambda(9), 0.25);
    assert_eq!(m.lambda(8), 0.0, "um parceiro que nunca empurrou le' zero");
    // ⚠️ E o índice `0` é uma peça legítima, não o «vazio» — a armadilha que o `VAZIA` nomeia.
    m.guardar(0, 3.0);
    assert_eq!(m.lambda(0), 3.0, "a peca 0 e' uma peca como as outras");
}

/// ⭐ **Guardar outra vez ACTUALIZA a mesma ranhura** — senão seis tiques enchiam a memória com o
/// mesmo parceiro e expulsavam os cinco vizinhos.
#[test]
fn storing_the_same_partner_twice_updates_instead_of_filling() {
    let mut m = Memoria::default();
    for l in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0] {
        m.guardar(4, l);
    }
    assert_eq!(
        m.ocupadas(),
        1,
        "oito escritas do mesmo parceiro = UMA ranhura"
    );
    assert_eq!(m.lambda(4), 8.0);
}

/// ⭐⭐ **O TRANSBORDO esquece o apoio MAIS FRACO** (ver [`Memoria::guardar`]).
#[test]
fn the_overflow_forgets_the_weakest_support() {
    let mut m = Memoria::default();
    #[expect(clippy::cast_precision_loss, reason = "um indice de ranhura pequeno")]
    for k in 0..K {
        m.guardar(k as u32, (k + 1) as f32);
    }
    assert_eq!(m.ocupadas(), K, "as seis ranhuras cheias");
    // O mais fraco e' o parceiro 0 (λ = 1). Um apoio mais forte expulsa-o…
    m.guardar(99, 10.0);
    assert_eq!(m.lambda(0), 0.0, "o mais fraco saiu");
    assert_eq!(m.lambda(99), 10.0);
    assert_eq!(m.lambda(1), 2.0, "e os outros cinco ficaram");
    // …e um mais fraco que TODOS não entra, em vez de expulsar um apoio real.
    m.guardar(123, 0.5);
    assert_eq!(
        m.lambda(123),
        0.0,
        "um apoio mais fraco que todos nao entra"
    );
    assert_eq!(m.ocupadas(), K);
}

/// ⭐ **Um `λ` a ZERO (ou não-finito) APAGA a ranhura** — um contacto que deixou de empurrar não
/// tem o que lembrar, e guardá-lo gastaria uma das seis.
#[test]
fn a_contact_that_stopped_pushing_frees_its_slot() {
    let mut m = Memoria::default();
    m.guardar(3, 2.0);
    assert_eq!(m.ocupadas(), 1);
    m.guardar(3, 0.0);
    assert_eq!(m.ocupadas(), 0, "o zero APAGA");
    m.guardar(3, 2.0);
    m.guardar(3, f32::NAN);
    assert_eq!(m.ocupadas(), 0, "e o nao-finito tambem");
}

/// ⭐⭐ **A ida e volta pela corrente é a IDENTIDADE** — é isto que torna a memória uma coluna, e
/// com ela o dispositivo deixa de ser uma wave (doc 111 §5.4).
#[test]
fn the_round_trip_through_the_stream_is_the_identity() {
    let n = 5;
    let mut ms = vec![Memoria::default(); n];
    for (i, m) in ms.iter_mut().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "indices pequenos")]
        for k in 0..=(i % K) {
            m.guardar((i * 10 + k) as u32, (k + 1) as f32 * 0.5);
        }
    }
    // ⭐⭐ **E uma identidade GRANDE, que é a cerca do TIPO que o cabeçalho declara.** Sem ela um
    // empacotamento em `u16` passaria este gate e truncaria toda cena com mais de 65 536 peças,
    // em silêncio — *uma ida e volta só prova o que a fixtura contém*.
    ms[0].guardar(1 << 20, 7.25);
    let mut out = Stream::new(n);
    escrever(&mut out, &ms);
    let volta = ler(&out, n);
    assert_eq!(volta, ms, "escrever → ler tem de ser a identidade");
    assert_eq!(volta[0].lambda(1 << 20), 7.25, "um `id` grande sobrevive");
}

/// ⛔⛔ **Colunas AUSENTES ⇒ tudo FRIO**, que é a lei de hoje ao bit — é o que permite a memória
/// nascer sem migração nenhuma, e o que garante que uma cena que nunca a escreveu não muda um bit.
#[test]
fn a_stream_without_the_columns_reads_all_cold() {
    let s = Stream::new(4);
    let m = ler(&s, 4);
    assert_eq!(m.len(), 4);
    assert!(m.iter().all(|x| x.ocupadas() == 0 && x.lambda(1) == 0.0));
}

/// ⚠️⚠️ **Uma coluna de comprimento errado nem CHEGA a existir** — o `Stream::set` recusa-a na
/// porta, e é por isso que a guarda do [`ler`] é um cinto e não a lei.
///
/// ⭐ O gate mede a RECUSA e não a guarda: *se um dia o `Stream` deixar de a impor, é aqui que se
/// fica a saber* — e aí a guarda passa a ser o único muro, em vez de ser redundante.
#[test]
#[should_panic(expected = "column length must equal stream element count")]
fn a_column_of_the_wrong_length_is_refused_at_the_door() {
    let mut s = Stream::new(4);
    s.set(
        COLUNAS[0].to_string(),
        Column::Vec4(vec![[1.0, 2.0, 3.0, 4.0]]),
    );
}

/// ⭐⭐⭐ **A CHAVE: o `id` quando ele existe, o ÍNDICE quando não** — e a cerca que isso traz está
/// no cabeçalho do módulo.
#[test]
fn the_key_is_the_id_when_there_is_one_and_the_index_when_there_is_not() {
    let sem = Stream::new(3);
    assert_eq!(chaves(&sem, 3), vec![0, 1, 2], "sem `id`, o indice");

    let com = Stream::new(3).with("id", Column::Scalar(vec![70.0, 71.0, 72.0]));
    assert_eq!(chaves(&com, 3), vec![70, 71, 72], "com `id`, o id");

    // ⚠️ Um `id` negativo ou não-finito cai em `0` — a mesma lei que o `sim.collide` já aplica.
    let torto = Stream::new(2).with("id", Column::Scalar(vec![-5.0, f32::NAN]));
    assert_eq!(chaves(&torto, 2), vec![0, 0]);
}

/// ⭐ **`K` cobre o pior caso MEDIDO** (doc 111 §5.4: `5` encostos na `=114`), com uma ranhura de
/// folga — e a folga é UMA, porque `K` multiplica a memória de toda peça de toda cena.
#[test]
fn k_covers_the_measured_worst_case_with_one_slot_to_spare() {
    /// O pior caso medido em `probe_quantos_encostos_por_peca`.
    const PIOR_MEDIDO: usize = 5;
    assert_eq!(
        K,
        PIOR_MEDIDO + 1,
        "uma ranhura de folga, nem mais nem menos"
    );
    assert_eq!(COLUNAS.len(), K / 2, "duas ranhuras por coluna `Vec4`");
    // ⚠️ `K` PAR e' lei de COMPILACAO, nao de corrida: um `K` impar deixaria a ultima ranhura sem
    // metade de coluna, e o `escrever` indexaria fora. O `const` faz disso um erro de compilar.
    const _: () = assert!(K.is_multiple_of(2), "o empacotamento em `Vec4` pede K par");
}
