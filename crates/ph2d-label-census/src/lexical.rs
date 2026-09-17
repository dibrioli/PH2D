//! **A RÉGUA LEXICAL** — todo literal com cara de língua que o produto compila.

use std::fs;
use std::path::{Path, PathBuf};

use crate::cfg_test::is_declared_under_cfg_test;
use crate::source::{self, Ctx};

/// Um literal com cara de língua, onde está e o que diz.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Literal {
    /// O caminho relativo à raiz varrida, com `/`.
    pub rel: String,
    /// A linha, a contar de 1.
    pub line: usize,
    /// O conteúdo, sem aspas nem prefixo.
    pub text: String,
    /// O literal INTEIRO (aspas e prefixo incluídos) em índices de CARÁCTER do ficheiro — a porta
    /// para quem o substitui sem uma segunda leitura do fonte.
    pub chars: std::ops::Range<usize>,
    /// De onde ele entra: `fn:<nome>` / `macro:<nome>` do parêntese que o envolve, ou vazio.
    pub via: String,
}

/// As macros cujo texto nunca chega a um ecrã: falhas, logs, asserções e o que o compilador lê.
///
/// ⚠️ `format!`, `vec!` e `write!` **não** estão aqui: montam o texto que um painel pinta.
const MACROS_QUE_NAO_PINTAM: &[&str] = &[
    "panic",
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "unreachable",
    "todo",
    "unimplemented",
    "eprintln",
    "println",
    "eprint",
    "print",
    "env",
    "option_env",
    "include_str",
    "include_bytes",
    "concat",
    "stringify",
    "matches",
    "compile_error",
    "trace",
    "debug",
    "info",
    "warn",
    "error",
];

/// As chamadas cujo argumento de texto é uma CHAVE, uma mensagem de falha, um nome de variável de
/// ambiente ou um padrão de busca — nunca um rótulo.
const CHAMADAS_QUE_NAO_PINTAM: &[&str] = &[
    "expect",
    "hash_node_id",
    "hash_node_id_runtime",
    "tr",
    "tr_with",
    "var",
    "var_os",
    "set_var",
    "remove_var",
    "starts_with",
    "ends_with",
    "contains",
    "strip_prefix",
    "strip_suffix",
    "find",
    "rfind",
    "split",
    "rsplit",
    "split_once",
    "rsplit_once",
    "splitn",
    "rsplitn",
    "replace",
    "replacen",
    "trim_start_matches",
    "trim_end_matches",
    "eq_ignore_ascii_case",
    // atributos com argumentos: `#[cfg(feature = "…")]` é lido como a chamada `cfg(`
    "cfg",
    "cfg_attr",
    "doc",
    "deprecated",
    "must_use",
];

/// A memória da [`language_literals`]: uma entrada por RAIZ varrida.
///
/// ⚠️ Ela é um alias por exigência do `clippy::type_complexity`, que o `ship.sh` corre com
/// `-D warnings` — e ⛔ **só o `ship.sh` o corre**, logo isto chegou ao `main` verde e foi
/// apanhado pela integração seguinte. É a mesma forma dos censos e dos tectos de LOC: um portão
/// que a LINHA nunca vê.
type Memo = Vec<(PathBuf, Vec<Literal>)>;

/// Todo literal com cara de língua nos `.rs` debaixo de `src_root` que o produto compila.
///
/// ⚠️ **Um ficheiro de teste é perguntado ao PAI** ([`is_declared_under_cfg_test`]), nunca ao nome.
pub fn language_literals(src_root: &Path) -> Vec<Literal> {
    // ⭐⭐ **MEMOIZADO por raiz, e a razão é MEDIDA** (2026-09-16): o gate de obsolescência da shell
    //    chama três portas desta crate (`excecoes_mortas` · `isentos_mortos` · `literais_de_cena`) e
    //    cada uma varria a árvore INTEIRA — `233 s` numa shell de 195 k linhas, acima do tecto de
    //    morte de `180 s` do executor. ⚠️ A cura NÃO é subir o tecto: era a mesma resposta calculada
    //    três vezes. O fonte não muda enquanto um binário de teste corre, logo a cache é correcta
    //    por construção; ela vive no processo e morre com ele.
    static CACHE: std::sync::OnceLock<std::sync::Mutex<Memo>> = std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| std::sync::Mutex::new(Vec::new()));
    if let Ok(c) = cache.lock()
        && let Some((_, v)) = c.iter().find(|(k, _)| k == src_root)
    {
        return v.clone();
    }
    let out = language_literals_uncached(src_root);
    if let Ok(mut c) = cache.lock() {
        c.push((src_root.to_path_buf(), out.clone()));
    }
    out
}

/// A varredura de facto — [`language_literals`] é a porta, e ela memoiza.
fn language_literals_uncached(src_root: &Path) -> Vec<Literal> {
    let mut files = Vec::new();
    walk(src_root, &mut files);
    files.sort();
    let mut out = Vec::new();
    for p in files {
        if is_declared_under_cfg_test(&p) {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&p) else {
            continue;
        };
        let rel = p
            .strip_prefix(src_root)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        out.extend(language_literals_in(&rel, &raw));
    }
    out
}

/// A régua sobre UM texto de fonte — a porta que os testes de controlo usam.
pub fn language_literals_in(rel: &str, src: &str) -> Vec<Literal> {
    let mut code = source::strip_comments(src);
    source::blank_cfg_test_items(&mut code);
    let mask = source::string_mask(&code);
    let mut out = Vec::new();
    for lit in source::literals(&code) {
        if lit.bytes {
            continue;
        }
        let text: String = code[lit.content.clone()].iter().collect();
        if !is_language(&text) {
            continue;
        }
        let via = match source::enclosing(&code, &mask, lit.start) {
            Ctx::Macro(m) if MACROS_QUE_NAO_PINTAM.contains(&m.as_str()) => continue,
            Ctx::Call(c) if CHAMADAS_QUE_NAO_PINTAM.contains(&c.as_str()) => continue,
            Ctx::Attr => continue,
            Ctx::Macro(m) => format!("macro:{m}"),
            Ctx::Call(c) => format!("fn:{c}"),
            Ctx::Other => String::new(),
        };
        if source::is_pattern_or_comparison(&code, lit.start, lit.end) {
            continue;
        }
        let line = code[..lit.start].iter().filter(|&&c| c == '\n').count() + 1;
        out.push(Literal {
            rel: rel.to_string(),
            line,
            text,
            chars: lit.start..lit.end,
            via,
        });
    }
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// **Este texto tem cara de LÍNGUA?** — o critério, com cada exclusão a dizer o que exclui.
///
/// - os marcadores de `format!` (`{n}`, `{:.2}`) não são palavras;
/// - sem duas letras seguidas não há palavra (`X`, `%`, `1:2`);
/// - uma CHAVE (`chrome.fill.title`, `area.menu.{slot}`) não é língua;
/// - um TOKEN sem espaços que não começa por maiúscula-minúscula é um identificador (`snake_case`,
///   `SCREAMING`, `kebab-id`, `wgsl`) — ⚠️ excepto quando o texto tinha um marcador e sobra uma
///   palavra de 3+ letras: `"{n} entities"` é uma frase;
/// - um caminho de ficheiro não é língua.
pub fn is_language(text: &str) -> bool {
    // ⛔⛔ **Os escapes do FONTE saem antes de tudo.** O texto de um literal chega tal como está
    //    escrito (`\u{00b7}`, `\n`, `\"`), e a 1.ª redacção desta função mandava para fora todo texto
    //    com `\` por «parecer um caminho» — logo TODO rótulo com um escape sumia em silêncio. Quem o
    //    apanhou foi a contagem da Hierarquia: `"{entities} entities \u{00b7} {components}
    //    components"` não aparecia. E o `\u{…}` tem de sair ANTES dos marcadores, senão o `{00b7}` é
    //    lido como um marcador de `format!`.
    let mut unescaped = String::with_capacity(text.len());
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        if c != '\\' {
            unescaped.push(c);
            continue;
        }
        match it.next() {
            Some('u') if it.peek() == Some(&'{') => {
                for d in it.by_ref() {
                    if d == '}' {
                        break;
                    }
                }
                unescaped.push('·');
            }
            Some('"') => unescaped.push('"'),
            Some(_) | None => unescaped.push(' '),
        }
    }
    let mut had_placeholder = false;
    let mut depth = 0usize;
    let mut kept = String::with_capacity(unescaped.len());
    for c in unescaped.chars() {
        match c {
            '{' => {
                depth += 1;
                had_placeholder = true;
            }
            '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 => kept.push(c),
            _ => {}
        }
    }
    let u = kept.trim();
    // ⛔ **Uma ABREVIATURA ligada por `&` é língua** (medido 2026-09-16): `"B&W"` (preto e branco,
    //    `P&B` em português) é um botão da pilha de ajustes e não tinha duas letras SEGUIDAS — saía
    //    como símbolo. Só a forma `letra & letra`: `&&` e `&` sozinho continuam de fora.
    let bytes = u.as_bytes();
    let has_word = bytes
        .windows(2)
        .any(|w| w[0].is_ascii_alphabetic() && w[1].is_ascii_alphabetic())
        || bytes
            .windows(3)
            .any(|w| w[0].is_ascii_alphabetic() && w[1] == b'&' && w[2].is_ascii_alphabetic());
    if !has_word {
        return false;
    }
    let key_like = u.contains('.')
        && u.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.');
    if key_like {
        return false;
    }
    let token = u
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':' || c == '-');
    let mut chars = u.chars();
    let capitalized = matches!(
        (chars.next(), chars.next()),
        (Some(a), Some(b)) if a.is_ascii_uppercase() && b.is_ascii_lowercase()
    );
    let letters = u.chars().filter(char::is_ascii_alphabetic).count();
    // ⛔⛔ **Uma palavra GRITADA é língua** (medido 2026-09-13, na migração do Inspector): os títulos
    //    dos cards da §14 são `"LEG"`, `"WALK"`, `"FORGIVENESS"`, pintados no ecrã, e a 1.ª redacção
    //    só aceitava um token Capitalizado — todo título em maiúsculas saía em silêncio. ⚠️ Só
    //    LETRAS e pelo menos três: `SCREAMING_CASE` (identificador), `UV` (sigla curta) e `RGBA16`
    //    (formato) continuam de fora.
    let shouted = letters >= 3 && u.chars().all(|c| c.is_ascii_uppercase());
    if token && !capitalized && !shouted && !(had_placeholder && letters >= 3) {
        return false;
    }
    // ⛔⛔ **Uma BARRA numa FRASE não é um caminho** (medido 2026-09-13, na migração do Inspector):
    //    `"Speed (m/s)"` e `"Corners F fixed (on/off)…"` saíam como caminho, e treze rótulos da §14
    //    ficaram fora do censo — quem os achou foi o COMPILADOR, com a tabela deles meio `TextKey`. Um
    //    caminho não tem espaço; uma unidade sozinha (`m/s`) continua de fora pela mesma pergunta.
    if (u.contains('/') || u.contains('\\')) && !u.contains(char::is_whitespace) {
        return false;
    }
    ![
        ".rs", ".png", ".svg", ".json", ".toml", ".wgsl", ".txt", ".obj", ".ph2d",
    ]
    .iter()
    .any(|ext| u.contains(ext))
}
