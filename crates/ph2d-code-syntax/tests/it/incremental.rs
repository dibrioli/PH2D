//! Reparse incremental = parse do zero, depois de CADA edição de um traço aleatório sobre `bob.luau`.

use crate::BOB;
use ph2d_code_buffer::{Buffer, Change, ChangeSet, EditKind};
use ph2d_code_syntax::Syntax;
use tree_sitter::Node;

/// Pedaços de Luau, inteiros e partidos: o texto passa por estados meio escritos o tempo todo.
const PEDACOS: &[&str] = &[
    "a",
    " ",
    "\n",
    "\t",
    "é",
    "😀",
    "end",
    "\nend",
    "function f()",
    "function",
    "if x then",
    "then",
    "else",
    "elseif",
    "do",
    "repeat",
    "until",
    "{",
    "}",
    "(",
    ")",
    "[",
    "]",
    "\"",
    "'",
    "--",
    "--[[",
    "]]",
    "=",
    "==",
    "local ",
    "self.t",
    ":",
    ",",
    "1.5",
    "nil",
    "true",
    "return",
    "`a{b}`",
];

fn boundary(s: &str, rng: &mut fastrand::Rng) -> usize {
    let mut b = rng.usize(0..=s.len());
    while !s.is_char_boundary(b) {
        b -= 1;
    }
    b
}

fn random_changes(s: &str, rng: &mut fastrand::Rng) -> ChangeSet {
    let n = rng.usize(1..=3);
    let mut cuts: Vec<usize> = (0..n * 2).map(|_| boundary(s, rng)).collect();
    cuts.sort_unstable();
    let changes = cuts
        .chunks(2)
        .map(|p| {
            // Cortes curtos (como a escrita), com um corte longo de vez em quando (colar/apagar bloco).
            let end = if rng.u8(0..10) == 0 {
                p[1]
            } else {
                p[0] + (p[1] - p[0]).min(rng.usize(0..4))
            };
            let mut end = end;
            while !s.is_char_boundary(end) {
                end += 1;
            }
            Change {
                range: p[0]..end,
                insert: (0..rng.usize(0..3))
                    .map(|_| PEDACOS[rng.usize(0..PEDACOS.len())])
                    .collect(),
            }
        })
        .collect();
    ChangeSet::new(changes).unwrap_or_default()
}

/// A árvore inteira com faixas em bytes e linhas — o `to_sexp` não as mostra.
fn dump(n: Node, out: &mut String) {
    out.push_str(&format!(
        "{}{}{}[{:?} {:?}-{:?}] ",
        n.kind(),
        if n.is_missing() { "!" } else { "" },
        if n.is_error() { "?" } else { "" },
        n.byte_range(),
        n.start_position(),
        n.end_position()
    ));
    let mut c = n.walk();
    for k in n.children(&mut c) {
        dump(k, out);
    }
}

fn full(s: &Syntax) -> String {
    let mut out = String::new();
    dump(s.tree().root_node(), &mut out);
    out
}

#[derive(Default, Debug)]
struct Conta {
    validos: usize,
    quebrados: usize,
    /// Passos com texto quebrado em que a recuperação de erro do incremental difere da do zero.
    divergem: usize,
}

/// Lei medida (ver o doc do módulo): sem erro, a árvore incremental é IGUAL à do zero (forma e
/// faixas); com erro, as duas concordam que há erro, cobrem o mesmo texto, e a recuperação pode
/// diferir — e nunca sobrevive à volta a um texto válido (o passo válido seguinte exige igualdade).
fn confere(syn: &Syntax, b: &Buffer, passo: usize, conta: &mut Conta) {
    let zero = Syntax::new(b.text());
    let (ri, rz) = (syn.tree().root_node(), zero.tree().root_node());
    assert_eq!(
        ri.has_error(),
        rz.has_error(),
        "passo {passo}: só um dos dois vê erro\ntexto:\n{}",
        b.text()
    );
    assert_eq!(
        ri.byte_range(),
        rz.byte_range(),
        "passo {passo}: raízes cobrem textos diferentes"
    );
    if rz.has_error() {
        conta.quebrados += 1;
        conta.divergem += usize::from(full(syn) != full(&zero));
        return;
    }
    conta.validos += 1;
    assert_eq!(
        syn.sexp(),
        zero.sexp(),
        "passo {passo}: sexp diverge\ntexto:\n{}",
        b.text()
    );
    assert_eq!(
        full(syn),
        full(&zero),
        "passo {passo}: faixas divergem\ntexto:\n{}",
        b.text()
    );
}

#[derive(Clone, Copy)]
enum Op {
    Troca,
    Desfaz,
    Refaz,
}

/// Traço aleatório em episódios: 1–4 trocas aleatórias (o texto parte-se), depois desfazer tudo
/// (volta ao `bob.luau` válido, vindo de uma história quebrada diferente a cada vez) e às vezes
/// refazer. Cada passo é conferido.
#[test]
fn reparse_incremental_igual_ao_parse_do_zero_em_600_edicoes() {
    let mut rng = fastrand::Rng::with_seed(0x05EE_D1A0);
    let mut b = Buffer::new(BOB);
    let mut syn = Syntax::new(b.text());
    let mut conta = Conta::default();
    let mut passo = 0;
    while passo < 600 {
        let k = rng.usize(1..=4);
        let mut ops = vec![Op::Troca; k];
        ops.extend(vec![Op::Desfaz; k]);
        if rng.bool() {
            ops.extend(vec![Op::Refaz; k]);
            ops.extend(vec![Op::Desfaz; k]);
        }
        for op in ops {
            let edits = match op {
                Op::Troca => {
                    let cs = random_changes(&b.text().to_string(), &mut rng);
                    b.apply(cs, None, EditKind::Other)
                }
                Op::Desfaz => b.undo(),
                Op::Refaz => b.redo(),
            };
            syn.edit(&edits, b.text());
            confere(&syn, &b, passo, &mut conta);
            passo += 1;
        }
        assert_eq!(
            b.text().to_string(),
            BOB,
            "o episódio não voltou ao texto de partida"
        );
    }
    println!("traço aleatório: {conta:?}");
    // O traço tem de exercitar os DOIS regimes, ou a lei acima não foi posta à prova.
    assert!(conta.validos >= 100 && conta.quebrados >= 100, "{conta:?}");
}

/// Traço realista: escrever `bob.luau` inteiro tecla a tecla e apagá-lo com Backspace.
#[test]
fn escrever_bob_tecla_a_tecla_e_apagar_segue_o_parse_do_zero() {
    let mut b = Buffer::new("");
    let mut syn = Syntax::new(b.text());
    let mut conta = Conta::default();
    let mut passo = 0;
    for g in BOB.chars() {
        let edits = b.insert(&g.to_string());
        syn.edit(&edits, b.text());
        confere(&syn, &b, passo, &mut conta);
        passo += 1;
    }
    assert_eq!(b.text().to_string(), BOB);
    while !b.text().is_empty() {
        let edits = b.delete_backward();
        syn.edit(&edits, b.text());
        confere(&syn, &b, passo, &mut conta);
        passo += 1;
    }
    println!("escrever e apagar bob: {conta:?}");
    assert!(conta.validos >= 100, "{conta:?}");
}
