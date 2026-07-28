//! Gates do [`super::TileJournal`] — filho por `#[path]`, então ele enxerga os privados.

use super::{TILE, TileJournal};

/// Um plano cujo valor em cada elemento é o próprio índice (mod 251, primo, para o padrão não repetir
/// dentro de um tile) — assim qualquer byte lido de volta identifica de onde veio.
fn plane(stride: usize, rows: usize) -> Vec<u8> {
    salted(stride, rows, 0)
}

/// O mesmo, deslocado por `salt` — para os casos em que DOIS planos diferentes precisam ter bytes
/// diferentes no mesmo índice.
///
/// ⚠️ Sem isto o gate do reshape passa por acidente: `i % 251` dá o MESMO byte no índice 1 para
/// qualquer geometria, então "o journal devolveu o plano velho" e "devolveu o novo" são
/// indistinguíveis. O fixture tem de conter o fenômeno.
fn salted(stride: usize, rows: usize, salt: u8) -> Vec<u8> {
    (0..stride * rows)
        .map(|i| ((i % 251) as u8).wrapping_add(salt))
        .collect()
}

/// **O que foi capturado volta EXATO, e o que não foi não volta.**
#[test]
fn a_captured_tile_gives_back_the_bytes_that_were_there() {
    let stride = 3 * TILE + 17; // última coluna CURTA de propósito
    let rows = 2 * TILE + 5; // …e a última linha também
    let buf = plane(stride, rows);
    let mut j = TileJournal::default();
    j.capture(&buf, stride, Some((TILE, TILE, TILE + 4, TILE + 4)));

    // Todo elemento do tile (1,1) volta exato — inclusive fora do retângulo pedido, porque a unidade
    // de captura é o TILE.
    for y in TILE..2 * TILE {
        for x in TILE..2 * TILE {
            let i = y * stride + x;
            assert_eq!(j.get(i), Some(buf[i]), "({x},{y})");
        }
    }
    // E um tile que ninguém tocou não tem resposta — o commit tem de saber que não sabe.
    assert_eq!(j.get(0), None, "o tile (0,0) nao foi capturado");
    assert_eq!(j.get(rows * stride - 1), None, "nem o ultimo");
}

/// **A PRIMEIRA captura é a que vale** — o invariante inteiro do journal.
///
/// Um passo escreve várias vezes no mesmo tile. Se a segunda captura sobrescrevesse, o journal
/// guardaria bytes **já modificados** e o undo devolveria um estado que nunca existiu.
///
/// ⚠️ Mutação: trocar o `continue` do `capture` por uma sobrescrita faz este gate ficar VERMELHO
/// com o valor escrito no lugar do original.
#[test]
fn the_first_capture_is_the_one_that_counts() {
    let (stride, rows) = (2 * TILE, 2 * TILE);
    let mut buf = plane(stride, rows);
    let mut j = TileJournal::default();
    j.capture(&buf, stride, Some((0, 0, 4, 4)));

    let i = 2 * stride + 2;
    let original = buf[i];
    buf[i] = original.wrapping_add(90); // a escrita que o dono do passo faz
    j.capture(&buf, stride, Some((0, 0, 8, 8))); // e uma segunda escrita no MESMO tile

    assert_eq!(
        j.get(i),
        Some(original),
        "a 2a captura sobrescreveu o original com um byte JA MODIFICADO"
    );
}

/// **Crescer é capturar o que ainda não foi capturado, e nada mais.**
///
/// O tile novo é capturado no estado em que está — o que é correto justamente porque ele **não** foi
/// escrito ainda (se tivesse sido, o dono da escrita teria declarado antes). É esta propriedade que
/// torna a grade de tiles imune à subtração de retângulos que um journal por retângulo exigiria.
#[test]
fn growing_captures_only_the_tiles_that_were_not_captured_yet() {
    let (stride, rows) = (3 * TILE, TILE);
    let mut buf = plane(stride, rows);
    let mut j = TileJournal::default();
    j.capture(&buf, stride, Some((0, 0, 4, 4))); // tile 0

    let (a, b) = (5usize, TILE + 5);
    let (orig_a, orig_b) = (buf[a], buf[b]);
    buf[a] = orig_a.wrapping_add(70); // escreve no tile JÁ capturado
    j.capture(&buf, stride, Some((TILE, 0, TILE + 4, 4))); // agora o tile 1

    assert_eq!(
        j.get(a),
        Some(orig_a),
        "o tile velho tem de guardar o velho"
    );
    assert_eq!(j.get(b), Some(orig_b), "o tile novo, o que estava nele");
    assert_eq!(j.get(2 * TILE + 5), None, "o tile 2 segue intocado");
}

/// **`None` = o plano inteiro** — a resposta de quem não sabe onde vai escrever, e ela é sempre
/// correta (só custa o que o fork custa hoje).
#[test]
fn a_site_that_does_not_know_where_it_writes_captures_everything() {
    let (stride, rows) = (TILE + 9, TILE + 3);
    let buf = plane(stride, rows);
    let mut j = TileJournal::default();
    j.capture(&buf, stride, None);
    for (i, &want) in buf.iter().enumerate() {
        assert_eq!(j.get(i), Some(want), "elemento {i}");
    }
}

/// **A geometria da borda tem de fechar** — a última coluna e a última linha são CURTAS, e um `get`
/// que assumisse tiles quadrados leria a linha errada de dentro do tile.
///
/// ⚠️ Mutação: fixar `tile_w` em `TILE` deixa este gate VERMELHO (e só ele — os outros usam planos
/// cuja última coluna é cheia ou não leem a borda).
#[test]
fn a_short_edge_tile_round_trips() {
    let stride = TILE + 3; // 3 elementos na última coluna
    let rows = TILE + 2;
    let buf = plane(stride, rows);
    let mut j = TileJournal::default();
    j.capture(&buf, stride, None);
    for y in [0, 1, TILE - 1, TILE, rows - 1] {
        for x in [TILE, TILE + 1, TILE + 2] {
            let i = y * stride + x;
            assert_eq!(j.get(i), Some(buf[i]), "borda ({x},{y})");
        }
    }
}

/// **Um plano que o stride não mede não é capturado** — e o journal DIZ que não sabe, em vez de
/// devolver um byte de um lugar que ele inventou.
#[test]
fn a_stride_that_does_not_measure_the_plane_captures_nothing() {
    let buf = plane(10, 10);
    let mut j = TileJournal::default();
    j.capture(&buf, 3, None); // 100 não é múltiplo de 3
    assert!(j.is_empty());
    assert_eq!(j.get(0), None);
}

/// **Trocar a forma do plano descarta o journal** — meia grade velha misturaria duas geometrias, e
/// nesse caso o `get` responderia com confiança um byte do plano ERRADO.
#[test]
fn a_reshaped_plane_drops_what_was_captured_for_the_old_shape() {
    let a = salted(2 * TILE, TILE, 0);
    let mut j = TileJournal::default();
    j.capture(&a, 2 * TILE, None);
    assert!(!j.is_empty());

    // ⚠️ `salt` diferente: os dois planos TÊM de diferir byte a byte, senão "devolveu o velho" e
    // "devolveu o novo" dão o mesmo número e o gate passa sobre a grade velha.
    let b = salted(3 * TILE, TILE, 111);
    j.capture(&b, 3 * TILE, Some((0, 0, 4, 4)));
    assert_eq!(
        j.get(2 * TILE + 1),
        None,
        "o tile 2 nao foi capturado agora"
    );
    assert_eq!(j.get(1), Some(b[1]), "e o tile 0 descreve o plano NOVO");
}

/// `reset` esquece tudo — o passo fechou.
#[test]
fn a_reset_forgets_the_step() {
    let buf = plane(TILE, TILE);
    let mut j = TileJournal::default();
    j.capture(&buf, TILE, None);
    assert!(j.heap_bytes() > 0);
    j.reset();
    assert!(j.is_empty());
    assert_eq!(j.heap_bytes(), 0);
    assert_eq!(j.get(0), None);
}
