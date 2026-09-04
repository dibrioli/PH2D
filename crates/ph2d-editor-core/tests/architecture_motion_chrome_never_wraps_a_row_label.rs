//! ⭐⭐⭐ **O CHROME DO MOTION DESENHA CADA RÓTULO NUMA LINHA SÓ** — o gate do report do
//! Enio de 2026-08-30 (*"dropdown de L-System: Connect Inside Group como Tropism repetido e
//! labels emboladas"*).
//!
//! # O defeito
//!
//! Nada estava repetido. As duas etiquetas mais longas (`L-System: Tropism Direction`, a
//! `178,03 px`, e `L-System: Trunk Segments`, a `169,81`) **quebravam em duas linhas** dentro
//! de uma linha de `22 px` com `159 px` de orçamento, e a segunda metade caía por cima da
//! entrada seguinte — daí `Tropism` a ler-se duas vezes e `Seed` ilegível.
//!
//! A causa é uma porta com dois sentidos: o `max_width` de
//! [`ph2d_editor_core::paint::paint_text`] é um **orçamento de QUEBRA**, e o de
//! [`ph2d_editor_core::text_elide::paint_text_elided`] é um **limite de CORTE**.
//!
//! ⚠️ **A lei já existia e ninguém a chamava.** O `text_elide.rs` diz no cabeçalho *"a label
//! one pixel too wide for its column silently becomes two lines and spills into the row
//! below"* — e medido em 2026-08-30, das **279** chamadas com orçamento de quebra na árvore,
//! **13** usavam a porta segura, e as treze eram da timeline.
//!
//! # O que este gate varre, e por que a fronteira é ESTA
//!
//! As duas crates de painel do Motion **mais** o [`slider_with_chip`], que é o pintor de
//! `ph2d-editor-core` por onde passam **265 dos ~400** controlos do catálogo — e onde uma
//! auditoria mediu **70 de 258** rótulos escalares a quebrar (`Sides`/`Points`/`Teeth` em
//! TRÊS linhas) depois de a 1.ª varredura os ter deixado de fora.
//!
//! ⛔ **E fica FORA, de propósito e com medição:**
//! - O resto de `widget/` (`checkbox.rs`, `section_header`, `panel_chrome`) — orçamentos
//!   largos (`≈242 px`), e **nenhum rótulo do catálogo os estoura hoje**.
//! - A `paint_text_centered`, que é uma SEGUNDA porta que quebra (73 chamadores). Medido: 14
//!   de 30 legendas multi-palavra do catálogo dão duas linhas num botão de `28 px`. ⚠️ Mas ela
//!   **centra** pela altura já quebrada, então o defeito é «apertado», não «por cima da linha
//!   seguinte» — e cortá-la apagaria a segunda palavra de um botão. *É decisão de produto, e
//!   um gate defende o que foi decidido, não o que ninguém mediu.*
//! - Os outros ~253 sítios da árvore, pela mesma razão.
//!
//! # ⛔ O que uma varredura TEXTUAL não vence, declarado
//!
//! `use ph2d_editor_core::paint::paint_text as wrap_label;` seguido de `wrap_label(…)` **passa**
//! — nenhum gate sobre texto resolve um `use … as`. Medido em 2026-08-30 e deixado assim de
//! propósito: a alternativa é um lint estrutural sobre a AST, que esta árvore não tem, e um
//! gate que finge alcançar o que não alcança é pior que um que diz onde acaba.
//!
//! O resto das nove derrotas que uma auditoria adversarial lhe deu está curado: o caminho
//! qualificado, o `paint_text_block`, o comentário entre o nome e o `(`, o literal com
//! parêntese desequilibrado, o `if false { f32::INFINITY } else { … }` e o `const` cujo NOME
//! contém `INFINITY` — mais os **cinco falsos positivos** em que citar a chamada antiga num
//! comentário reprovava o portão.
//!
//! Dep-free (std only), como os outros gates de arquitectura.

use std::fs;
use std::path::{Path, PathBuf};

/// As crates cujo chrome é varrido por inteiro.
const SCANNED_CRATES: &[&str] = &["ph2d-panel-motion-graph", "ph2d-panel-motion-params"];

/// Ficheiros avulsos de `ph2d-editor-core` que desenham rótulos de linha do Motion.
// ⚠️⚠️ **O ficheiro virou PASTA na integracao de 2026-09-04** (`line/UIUX`, tecto de LOC), e este
// gate afirma sobre um CAMINHO. ⭐ Ele falhou ALTO -- *«nao existe: o ficheiro mudou de nome»* --
// porque quem o escreveu po^s a asserção de existencia ANTES da varredura; sem ela, um caminho
// morto varre zero ficheiros e o gate fica **verde a medir nada**.
// ⇒ varre-se a pasta INTEIRA: o `mod.rs` **e** o `number_chip.rs`, que e' onde o valor da caixa
// se pinta agora e onde a mesma lei tem de valer.
const SCANNED_FILES: &[&str] = &[
    "ph2d-editor-core/src/widget/slider_with_chip/mod.rs",
    "ph2d-editor-core/src/widget/slider_with_chip/number_chip.rs",
];

/// As portas que tratam o 7.º argumento (`max_width`) como orçamento de QUEBRA.
const WRAPPING_DOORS: &[&str] = &["paint_text", "paint_text_title", "paint_text_block"];

/// Quantos rótulos usam hoje a porta que CORTA, nos ficheiros varridos. ⚠️ **Contagem EXACTA
/// com censo**: uma folga («pelo menos N») deixaria apagar rótulos em silêncio, que é
/// literalmente o que o controlo existe para impedir.
///
/// **Histórico do número** — cada subida nomeia o rótulo que a causou, senão isto é a catraca
/// sem censo que o CLAUDE.md §5.0 proíbe:
/// - `26` (2026-08-30) — a redacção original, o dia em que a lei passou a valer.
/// - `27` (2026-08-31) — a linha de QUEIXA de uma regra malformada do `source.lsystem`
///   (`rows_paint::paint_one_row`, braço `ParamRow::Text(text) if text.problem.is_some()`):
///   texto livre do artista, comprimento sem tecto, e por isso obrigada a CORTAR.
/// - `26` (2026-09-04, integracao) — e desta vez o numero **DESCEU**, que e' a metade que o
///   proprio doc acima manda explicar. A `line/UIUX` fundiu as TRES colunas de uma linha de
///   propriedade (rotulo | trilho | caixa) numa **caixa unica**: o `slider_with_chip` tinha DOIS
///   rotulos a cortar e passou a ter **um**. ⛔ Nao e' «alguem trocou um rotulo de volta» — a lei
///   continua a valer em todos os que restaram, e o gate confirma-o ao varrer agora a pasta
///   inteira (`mod.rs` **e** `number_chip.rs`).
///
/// ⚠️ E este numero literal e' exactamente a armadilha que a memoria da casa nomeia — *uma
/// contagem literal num gate faz cada feature nova editar o teste de outra pessoa*. Ele so' se
/// sustenta porque tem o censo dos dois lados; deriva-lo continua por fazer.
const ELIDED_TODAY: usize = 26;

fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ dir")
        .to_path_buf()
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// ⭐⭐ **O código, com os COMENTÁRIOS e os LITERAIS apagados** (cada byte trocado por um
/// espaço, para as linhas e as posições não se moverem).
///
/// ⚠️⚠️ Sem isto o gate errava nos DOIS sentidos, e a auditoria de 2026-08-30 mediu os dois:
/// - **falso positivo** — documentar a própria cura (*"antes era `paint_text(ts, sc, label,
///   x, y, f, rect.w, cor)` e quebrava"*, a frase natural do próximo autor) **reprovava o
///   portão**, acusando uma linha de comentário de ser um rótulo;
/// - **falso negativo** — um literal com um parêntese desequilibrado (`"Tropism :("`) fazia a
///   contagem de argumentos desalinhar, e a chamada a seguir escapava.
fn code_only(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = vec![b' '; b.len()];
    let (mut i, mut n) = (0usize, b.len());
    while i < n {
        // `//` até ao fim da linha (doc-comments incluídos).
        if b[i] == b'/' && i + 1 < n && b[i + 1] == b'/' {
            while i < n && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // `/* … */`, aninhados como o Rust os aceita.
        if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
            let mut depth = 1;
            i += 2;
            while i < n && depth > 0 {
                if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
                    depth += 1;
                    i += 2;
                } else if b[i] == b'*' && i + 1 < n && b[i + 1] == b'/' {
                    depth -= 1;
                    i += 2;
                } else {
                    if b[i] == b'\n' {
                        out[i] = b'\n';
                    }
                    i += 1;
                }
            }
            continue;
        }
        // `'a'` e `'\n'` — um lifetime (`'a` sem fecho) não entra aqui porque exige o `'`.
        if b[i] == b'\'' {
            let close = if i + 2 < n && b[i + 1] == b'\\' { 3 } else { 2 };
            if i + close < n && b[i + close] == b'\'' {
                i += close + 1;
                continue;
            }
        }
        // `"…"` e `r"…"` / `r#"…"#`.
        if b[i] == b'"' {
            i += 1;
            while i < n {
                if b[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if b[i] == b'"' {
                    i += 1;
                    break;
                }
                if b[i] == b'\n' {
                    out[i] = b'\n';
                }
                i += 1;
            }
            continue;
        }
        out[i] = b[i];
        i += 1;
    }
    // `n` só existe para o `while`; devolve-se o texto saneado.
    n = out.len();
    debug_assert_eq!(n, b.len());
    String::from_utf8_lossy(&out).into_owned()
}

/// Os argumentos de topo de uma chamada que abre em `open`.
fn top_level_args(s: &str, open: usize) -> Option<Vec<String>> {
    let b = s.as_bytes();
    let (mut depth, mut start, mut out) = (0i32, open + 1, Vec::new());
    for (j, c) in b.iter().enumerate().skip(open) {
        match c {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    out.push(s[start..j].to_string());
                    return Some(out);
                }
            }
            b',' if depth == 1 => {
                out.push(s[start..j].to_string());
                start = j + 1;
            }
            _ => {}
        }
    }
    None
}

/// `true` quando `name` em `at` é uma chamada e não parte de um identificador maior.
fn is_call(s: &str, name: &str, at: usize) -> bool {
    if s[..at]
        .chars()
        .next_back()
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
    {
        return false;
    }
    let rest = &s[at + name.len()..];
    // O nome tem de acabar aqui: `paint_text_elided` não é `paint_text`.
    if rest
        .chars()
        .next()
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
    {
        return false;
    }
    rest.trim_start().starts_with('(') && !s[..at].trim_end().ends_with("fn")
}

fn scan(path: &Path, label: &str, offenders: &mut Vec<String>, elided: &mut usize) {
    let raw = fs::read_to_string(path).expect("ler o ficheiro");
    let src = code_only(&raw);
    *elided += src.matches("_elided(").count();
    for door in WRAPPING_DOORS {
        let mut from = 0usize;
        while let Some(rel) = src[from..].find(door) {
            let at = from + rel;
            from = at + door.len();
            if !is_call(&src, door, at) {
                continue;
            }
            let Some(open) = src[at..].find('(').map(|o| at + o) else {
                continue;
            };
            let Some(args) = top_level_args(&src, open) else {
                continue;
            };
            if args.len() < 7 {
                continue;
            }
            let w = args[6].split_whitespace().collect::<Vec<_>>().join(" ");
            // ⚠️ Igualdade EXACTA, não `contains`: a auditoria mostrou que
            // `const INFINITY_BUDGET_PX: f32 = 120.0;` e
            // `if false { f32::INFINITY } else { rect.w }` passavam os dois.
            if w == "f32::INFINITY" {
                continue;
            }
            let line = src[..at].matches('\n').count() + 1;
            offenders.push(format!("{label}:{line}  {door}(..., max_width = {w}, ...)"));
        }
    }
}

#[test]
fn the_motion_chrome_never_gives_a_row_label_a_wrap_budget() {
    let root = crates_dir();
    let mut offenders: Vec<String> = Vec::new();
    let mut elided = 0usize;
    for c in SCANNED_CRATES {
        let mut files = Vec::new();
        rs_files(&root.join(c).join("src"), &mut files);
        assert!(
            !files.is_empty(),
            "{c}/src vazio — o gate varre o sitio errado"
        );
        for f in files {
            let name = f
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            scan(&f, &format!("{c}/{name}"), &mut offenders, &mut elided);
        }
    }
    for rel in SCANNED_FILES {
        let f = root.join(rel);
        assert!(f.is_file(), "{rel} nao existe — o ficheiro mudou de nome");
        scan(&f, rel, &mut offenders, &mut elided);
    }

    // ⚠️⚠️ **O CONTROLE das PORTAS conta a ÁRVORE INTEIRA, e não os ficheiros varridos.**
    // A 1.ª redacção contava-os nos varridos e pendia de UMA chamada — a excepção declarada do
    // separador `/`. Isso punha-o a ficar VERMELHO exactamente quando a lei vencesse por
    // completo, com a mensagem errada (*"foram renomeadas"*). *Um controlo que se apaga com o
    // sucesso da lei mede a lei, não a ferramenta.*
    let mut tree = Vec::new();
    rs_files(&root, &mut tree);
    let doors_in_tree: usize = tree
        .iter()
        .filter(|p| !p.to_string_lossy().contains("/tests/"))
        .map(|p| {
            let raw = fs::read_to_string(p).unwrap_or_default();
            let src = code_only(&raw);
            WRAPPING_DOORS
                .iter()
                .map(|d| {
                    let mut k = 0usize;
                    let mut from = 0usize;
                    while let Some(rel) = src[from..].find(d) {
                        let at = from + rel;
                        from = at + d.len();
                        if is_call(&src, d, at) {
                            k += 1;
                        }
                    }
                    k
                })
                .sum::<usize>()
        })
        .sum();
    assert!(
        doors_in_tree > 100,
        "so' {doors_in_tree} chamadas as portas que quebram em toda a arvore — elas foram \
         renomeadas e este gate esta' a medir o vazio"
    );

    // ⚠️ **Contagem EXACTA com as DUAS metades** — uma folga deixaria apagar rótulos, e um
    // número que só sobe seria uma catraca sem censo (CLAUDE.md §5.0).
    assert_eq!(
        elided, ELIDED_TODAY,
        "os rotulos que CORTAM eram {ELIDED_TODAY} e agora sao {elided}: se subiu, actualize o \
         `ELIDED_TODAY`; se desceu, alguem trocou um rotulo de volta ou apagou-o"
    );

    assert!(
        offenders.is_empty(),
        "rotulo de linha com orcamento de QUEBRA — ele parte em duas linhas e escreve por cima \
         da entrada seguinte (o report do Enio de 2026-08-30). Use \
         `ph2d_editor_core::text_elide::paint_text_elided` (ou `paint_text_title_elided` para \
         SemiBold), que corta com reticencias:\n  {}",
        offenders.join("\n  ")
    );
}
