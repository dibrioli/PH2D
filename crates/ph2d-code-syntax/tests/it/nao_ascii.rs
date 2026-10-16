//! ⛔ **O `AutoEdit` nunca entra em pânico sobre texto NÃO-ASCII** (auditoria do fecho da Onda 1):
//! `context_at` cortava a cabeça de um comentário em `s + 4` BYTES — o 4.º byte de `-- é` cai dentro
//! do `é` e o `crop` entrava em pânico (o app caía ao escrever `(` depois de um comentário acentuado).
//! Os casos da auditoria com a saída esperada, e um traço aleatório com acentos, emoji, combinantes e
//! CJK que escreve cada abridor em posições aleatórias.

use ph2d_code_buffer::{Buffer, Range, Selection};
use ph2d_code_syntax::{AutoEdit, IndentStyle, Syntax};

fn escreve(texto: &str, ch: char) -> String {
    let mut buf = Buffer::new(texto);
    buf.set_selection(Selection::single(Range::cursor(texto.len())));
    let mut syn = Syntax::new(buf.text());
    AutoEdit::new(IndentStyle::Spaces(2)).on_char(&mut buf, &mut syn, ch);
    buf.text().to_string()
}

/// As entradas da auditoria: dentro de um comentário não se abre par nenhum (a lei do Godot).
#[test]
fn os_casos_da_auditoria_nao_derrubam_o_editor() {
    for (texto, ch, esperado) in [
        ("-- é", '(', "-- é("),
        ("--aé", '(', "--aé("),
        ("-- 😀", '"', "-- 😀\""),
        ("--ação", '[', "--ação["),
        ("local x = 1 --aé", '{', "local x = 1 --aé{"),
        // Controlo: fora do comentário o par abre.
        ("local é = ", '(', "local é = ()"),
    ] {
        assert_eq!(escreve(texto, ch), esperado, "{texto:?} + {ch:?}");
    }
}

const ALFABETO: &[&str] = &[
    "a", "x", " ", "  ", "\n", "-", "--", "--[[", "]]", "\"", "'", "(", ")", "{", "}", "[", "]",
    "é", "ç", "ã", "😀", "👍🏽", "e\u{301}", "a\u{308}\u{301}", "日本", "語", "\t", "end",
    "function f()", "local s = \"", "[=[", "]=]", "🇵🇹",
];

/// Os bytes que são fronteira de carácter (onde o editor pode pôr um cursor).
fn fronteiras(s: &str) -> Vec<usize> {
    s.char_indices().map(|(i, _)| i).chain([s.len()]).collect()
}

#[test]
fn um_traco_aleatorio_nao_ascii_nunca_entra_em_panico() {
    let mut rng = fastrand::Rng::with_seed(0x00C0_DE1A);
    let abridores = ['(', '[', '{', '"', '\'', ')', ']', '}', '\n'];
    for ronda in 0..400 {
        let n = rng.usize(1..24);
        let texto: String = (0..n)
            .map(|_| ALFABETO[rng.usize(..ALFABETO.len())])
            .collect();
        let mut buf = Buffer::new(&texto);
        let mut syn = Syntax::new(buf.text());
        let ae = AutoEdit {
            indent: if rng.bool() {
                IndentStyle::Tabs
            } else {
                IndentStyle::Spaces(rng.usize(0..5))
            },
            close_pairs: true,
        };
        for _ in 0..12 {
            let agora = buf.text().to_string();
            let fr = fronteiras(&agora);
            let cursores: Vec<Range> = (0..rng.usize(1..4))
                .map(|_| {
                    let a = fr[rng.usize(..fr.len())];
                    let b = if rng.bool() { a } else { fr[rng.usize(..fr.len())] };
                    Range::new(a, b)
                })
                .collect();
            buf.set_selection(Selection::new(cursores, 0));
            match rng.usize(0..10) {
                0 => {
                    ae.on_backspace(&mut buf, &mut syn);
                }
                _ => {
                    let ch = abridores[rng.usize(..abridores.len())];
                    ae.on_char(&mut buf, &mut syn, ch);
                }
            }
            let t = buf.text().to_string();
            assert!(
                buf.selection()
                    .ranges()
                    .iter()
                    .all(|r| t.is_char_boundary(r.anchor) && t.is_char_boundary(r.head)),
                "ronda {ronda}: um cursor fora de uma fronteira de carácter em {t:?}"
            );
        }
    }
}
