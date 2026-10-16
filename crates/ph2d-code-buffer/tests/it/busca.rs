//! E9 — buscar/substituir e ir para linha.

use ph2d_code_buffer::{Buffer, FindOptions, find_all, goto_line, replace_all, replace_one};

const OPTS: FindOptions = FindOptions {
    case_sensitive: true,
    whole_word: false,
};

#[test]
fn substituir_tudo_e_um_passo_de_undo() {
    let src = "vida = 3\nif vida > 0 then\n  vida = vida - 1\nend\n";
    let mut b = Buffer::new(src);
    let m = find_all(b.text(), "vida", OPTS);
    assert_eq!(m.len(), 4, "a prévia da contagem");
    let n = b.history().undo_len();
    replace_all(&mut b, &m, "hp");
    assert_eq!(
        b.to_string(),
        "hp = 3\nif hp > 0 then\n  hp = hp - 1\nend\n"
    );
    assert_eq!(b.history().undo_len(), n + 1);
    b.undo();
    assert_eq!(b.to_string(), src, "um Ctrl+Z desfaz as quatro trocas");
}

#[test]
fn busca_sem_resultado_devolve_vazio() {
    let b = Buffer::new("local x = 1\n");
    assert!(find_all(b.text(), "nada", OPTS).is_empty());
    assert!(
        find_all(b.text(), "", OPTS).is_empty(),
        "agulha vazia não casa em todo o lado"
    );
    let mut b = b;
    assert!(replace_all(&mut b, &[], "y").is_empty());
    assert!(
        !b.history().can_undo(),
        "substituir zero ocorrências não deixa passo de undo"
    );
}

#[test]
fn palavra_inteira_nao_casa_dentro_de_vida_max() {
    let b = Buffer::new("vida_max = 10\nvida = vida_max\nself.vida = 1");
    let whole = FindOptions {
        case_sensitive: true,
        whole_word: true,
    };
    let got: Vec<_> = find_all(b.text(), "vida", whole)
        .iter()
        .map(|r| b.text().slice(r.clone()))
        .collect();
    assert_eq!(got.len(), 2, "só `vida` e `self.vida`");
    let at = find_all(b.text(), "vida", whole);
    assert_eq!(b.text().slice(at[0].start - 1..at[0].end + 2), "\nvida =");
    assert_eq!(
        find_all(b.text(), "vida", OPTS).len(),
        4,
        "sem palavra inteira casa as quatro"
    );
}

#[test]
fn maiusculas_e_utf8_sem_distinguir_caixa() {
    let b = Buffer::new("Água e ÁGUA e água; Agua");
    let insens = FindOptions::default();
    let m = find_all(b.text(), "água", insens);
    let got: Vec<_> = m.iter().map(|r| b.text().slice(r.clone())).collect();
    assert_eq!(got, ["Água", "ÁGUA", "água"]);
    assert_eq!(find_all(b.text(), "água", OPTS).len(), 1);
}

#[test]
fn substituir_uma_poe_o_cursor_no_fim_do_texto_novo() {
    let mut b = Buffer::new("a + a");
    let m = find_all(b.text(), "a", OPTS);
    replace_one(&mut b, m[1].clone(), "bb");
    assert_eq!(b.to_string(), "a + bb");
    assert_eq!(b.selection().primary().head, 6);
}

#[test]
fn goto_line_40() {
    let src: String = (1..=60)
        .map(|i| format!("{}x{i}\n", "  ".repeat(i % 3)))
        .collect();
    let mut b = Buffer::new(&src);
    let at = goto_line(&mut b, 40);
    let t = b.text();
    assert_eq!(t.line_of_byte(at), 39, "linha 40 em base 1");
    assert_eq!(t.slice(at..t.line_end(39)), "x40", "no 1.º não-espaço");
    assert_eq!(b.selection().primary().head, at);
    let first = goto_line(&mut b, 0);
    assert_eq!(b.text().line_of_byte(first), 0, "0 encosta na primeira");
    let last = b.text().line_count() - 1;
    let end = goto_line(&mut b, 10_000);
    assert_eq!(
        b.text().line_of_byte(end),
        last,
        "além do fim encosta na última"
    );
}

/// ⭐ **A busca que segue as edições = a busca do zero** (item 10 da auditoria): um traço aleatório
/// de trocas multicursor sobre texto periódico (`aaaa`, onde o casamento guloso muda de fase),
/// acentos e maiúsculas, com as quatro combinações de opções.
#[test]
fn seguir_as_edicoes_da_o_mesmo_que_buscar_do_zero() {
    use ph2d_code_buffer::{Change, ChangeSet, EditKind, find_update};
    let mut rng = fastrand::Rng::with_seed(0xF1ED);
    let pedacos = ["a", "aa", "A", "b", " ", "\n", "é", "É", "vida", "Vida_", "_", "ç", "😀"];
    let agulhas = ["a", "aa", "aaa", "vida", "é", "ab", "a a"];
    for ronda in 0..300 {
        let texto: String = (0..rng.usize(0..40)).map(|_| pedacos[rng.usize(..pedacos.len())]).collect();
        let mut b = Buffer::new(&texto);
        let needle = agulhas[rng.usize(..agulhas.len())];
        let opts = FindOptions { case_sensitive: rng.bool(), whole_word: rng.bool() };
        let mut m = find_all(b.text(), needle, opts);
        for passo in 0..15 {
            let t = b.text().to_string();
            let fr: Vec<usize> = t.char_indices().map(|(i, _)| i).chain([t.len()]).collect();
            let mut changes = Vec::new();
            for _ in 0..rng.usize(1..4) {
                let (x, y) = (fr[rng.usize(..fr.len())], fr[rng.usize(..fr.len())]);
                let insert: String = (0..rng.usize(0..3)).map(|_| pedacos[rng.usize(..pedacos.len())]).collect();
                changes.push(Change { range: x.min(y)..x.max(y), insert });
            }
            let Some(cs) = ChangeSet::new(changes) else { continue };
            let edits = b.apply(cs, None, EditKind::Other);
            find_update(b.text(), needle, opts, &mut m, &edits);
            assert_eq!(
                m,
                find_all(b.text(), needle, opts),
                "ronda {ronda} passo {passo}: {needle:?} {opts:?} em {:?}",
                b.text().to_string()
            );
        }
    }
}
