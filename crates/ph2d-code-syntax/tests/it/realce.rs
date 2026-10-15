//! Realce de `bob.luau`: fixtura legível (linha, colunas, kind, texto), faixa ⊂ todo, sem sobreposição.

use crate::{BOB, line_of};
use ph2d_code_buffer::Text;
use ph2d_code_syntax::{HighlightKind as K, HighlightSpan, Syntax};

const FIXTURA: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/it/fixtures/bob.highlights.txt"
);

fn render(text: &Text, spans: &[HighlightSpan]) -> String {
    let mut out = String::from(
        "# realce de tests/it/fixtures/bob.luau — linha:col..col kind «texto» (col em bytes, base 0)\n",
    );
    out.push_str("# regravar: PH2D_GRAVA_FIXTURA=1 (e rever o diff à mão)\n");
    for s in spans {
        let a = text.point_of_byte(s.range.start);
        let b = text.point_of_byte(s.range.end);
        let end = if b.row == a.row {
            format!("{}", b.col)
        } else {
            format!("{}:{}", b.row + 1, b.col)
        };
        out.push_str(&format!(
            "{}:{}..{} {:?} {:?}\n",
            a.row + 1,
            a.col,
            end,
            s.kind,
            text.slice(s.range.clone())
        ));
    }
    out
}

#[test]
fn realce_de_bob_igual_a_fixtura() {
    let text = Text::new(BOB);
    let got = render(&text, &Syntax::new(&text).highlights(&text, 0..text.len()));
    if std::env::var_os("PH2D_GRAVA_FIXTURA").is_some() {
        std::fs::write(FIXTURA, &got).expect("gravar a fixtura");
    }
    let want =
        std::fs::read_to_string(FIXTURA).expect("fixtura em falta: corra com PH2D_GRAVA_FIXTURA=1");
    assert_eq!(
        got, want,
        "o realce de bob.luau mudou — reveja o diff e regrave se for o certo"
    );
}

#[test]
fn spans_ordenados_sem_sobreposicao_e_a_faixa_e_o_todo_recortado() {
    let text = Text::new(BOB);
    let syn = Syntax::new(&text);
    let all = syn.highlights(&text, 0..text.len());
    for w in all.windows(2) {
        assert!(
            w[0].range.end <= w[1].range.start,
            "sobreposição: {:?} {:?}",
            w[0],
            w[1]
        );
    }
    // A vista pede só as linhas visíveis: cada janela de 3 linhas = o todo recortado nela.
    for first in 0..text.line_count() {
        let a = text.byte_of_line(first);
        let b = text.byte_of_line((first + 3).min(text.line_count()));
        let clip: Vec<HighlightSpan> = all
            .iter()
            .filter(|s| s.range.end > a && s.range.start < b)
            .map(|s| HighlightSpan {
                range: s.range.start.max(a)..s.range.end.min(b),
                kind: s.kind,
            })
            .collect();
        assert_eq!(
            syn.highlights(&text, a..b),
            clip,
            "janela das linhas {first}..{}",
            first + 3
        );
    }
}

fn kind_at(text: &Text, syn: &Syntax, line: usize, needle: &str) -> Option<K> {
    let at = text.byte_of_line(line) + text.line_string(line).find(needle)?;
    syn.highlights(&text, at..at + needle.len())
        .iter()
        .find(|s| s.range.start == at)
        .map(|s| s.kind)
}

/// O que cada classe deve ser no `bob.luau` (leitura de editor, não da fixtura).
#[test]
fn as_classes_que_o_dono_ve() {
    let text = Text::new(BOB);
    let syn = Syntax::new(&text);
    let init = line_of(BOB, "function init(self)");
    let casos = [
        (init, "function", K::Keyword),
        (init, "init", K::Function),
        (init + 1, "self", K::Builtin),
        (init + 1, "base_x", K::Property),
        (init + 1, "ph2d", K::Variable),
        (init + 1, "get", K::FunctionCall),
        (init + 1, "\"x\"", K::String),
        (init + 3, "0", K::Number),
        (init + 4, "false", K::Boolean),
        (line_of(BOB, "ph2d.property(\"active\""), "true", K::Boolean),
        (line_of(BOB, "math.sin(self.t"), "math", K::Builtin),
        (line_of(BOB, "if not self.active"), "not", K::Keyword),
        (line_of(BOB, "if not self.active"), "return", K::Keyword),
        (line_of(BOB, "local up = s > 0.98"), ">", K::Operator),
        (
            line_of(BOB, "-- bob.luau — sobe e desce."),
            "--",
            K::Comment,
        ),
        (
            line_of(BOB, "ph2d.property(\"amplitude\""),
            "{",
            K::Punctuation,
        ),
        (
            line_of(BOB, "ph2d.property(\"amplitude\""),
            "min",
            K::Property,
        ),
    ];
    for (line, needle, want) in casos {
        assert_eq!(
            kind_at(&text, &syn, line, needle),
            Some(want),
            "linha {} `{needle}`",
            line + 1
        );
    }
}

/// As trocas da cópia FILTRAM: só o nome em maiúsculas (ou em posição de tipo) é Type/Constant.
#[test]
fn match_traduzido_filtra() {
    let text = Text::new("local a = Foo\nlocal b = MAX_N\nlocal c = foo\n");
    let syn = Syntax::new(&text);
    assert_eq!(kind_at(&text, &syn, 0, "Foo"), Some(K::Type));
    assert_eq!(kind_at(&text, &syn, 1, "MAX_N"), Some(K::Constant));
    assert_eq!(kind_at(&text, &syn, 2, "foo"), Some(K::Variable));
    // Em posição de TIPO, o nome em minúsculas é Type; o valor ao lado continua Variable.
    let text = Text::new("local d: point = p\n");
    let syn = Syntax::new(&text);
    assert_eq!(kind_at(&text, &syn, 0, "point"), Some(K::Type));
    let p = text.len() - 2;
    assert_eq!(
        syn.highlights(&text, p..p + 1).first().map(|s| s.kind),
        Some(K::Variable)
    );
}

/// A captura MENOR vence a maior que a contém, mesmo vindo antes na query: a expressão dentro de
/// uma string interpolada é código (`@variable` está antes de `(string) @string` na query).
#[test]
fn a_captura_menor_vence_a_que_a_contem() {
    let text = Text::new("local s = `a{b}c`\n");
    let syn = Syntax::new(&text);
    assert_eq!(kind_at(&text, &syn, 0, "b"), Some(K::Variable));
    assert_eq!(kind_at(&text, &syn, 0, "`a"), Some(K::String));
}
