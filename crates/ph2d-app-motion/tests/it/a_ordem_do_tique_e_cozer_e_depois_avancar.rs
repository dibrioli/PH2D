//! ⛔ **NENHUM ARNÊS AVANÇA O TIQUE ANTES DE COZER O QUADRO** (doc 123 §4).
//!
//! A lei é do `Cook::advance_tick`: *chamar uma vez por quadro, DEPOIS do(s) `cook` do quadro, com o
//! mesmo playhead* — e é a ordem do produto (`ph2d-eval-motion`: `cook_target_into` → `avanca_o_pre`;
//! o `scrub` idem). Um laço que faz `advance_tick(t)` e depois `cook(t)` coze cada nó sequencial
//! (fonte de `pre`) DUAS vezes por tique e mede outro programa: o `motion.delay` da cena do teto
//! leu `0,1678` contra `0,0800` e o gate ficou `#[ignore]` como «defeito ABERTO» do kernel.
//!
//! Censo de TEXTO sobre `src/` desta crate e `ph2d-node-registry-init/tests/it`: dentro de cada
//! corpo de `for`/`while`/`loop` e de cada closure com chaves, recusa um `advance_tick*` seguido
//! (no corpo MAIS INTERNO que o contém) por um `cook`/`cook_scoped`/`cook_scoped_fanned` com o
//! MESMO argumento de playhead. Corpos de `fn` não entram: o tique único com `t` constante é outro
//! caso (doc 123 §4).

use std::path::{Path, PathBuf};

/// Onde o censo lê, relativo ao `CARGO_MANIFEST_DIR`.
const RAIZES: &[&str] = &["src", "../ph2d-node-registry-init/tests/it"];

/// Isenções `ficheiro:fn`, cada uma com o porquê. Uma isenção que não casa nada REPROVA.
const ISENTOS: &[(&str, &str)] = &[(
    "lei_da_aparencia_tests.rs:uma_forma_solta_no_grafo",
    "um Cook NOVO por nível e um tique só, a t = 0,0: o laço percorre CENAS, não tiques, e só conta \
     instâncias de uma grelha",
)];

/// Piso da população: laços/closures que contêm um avanço E um cozimento. Medido 08/10: 206 (de 3 772 corpos).
const PISO_DE_CIRCUITOS: usize = 20;

#[derive(Debug, PartialEq, Eq)]
struct Achado {
    linha: usize,
    func: String,
    playhead: String,
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Apaga comentários e conteúdo de literais (mantém os comprimentos e as quebras de linha), para
/// que chaves e nomes dentro deles não contem.
fn sanitize(s: &str) -> Vec<u8> {
    let src = s.as_bytes();
    let mut out = src.to_vec();
    let n = src.len();
    let blank = |out: &mut Vec<u8>, a: usize, b: usize| {
        for c in out.iter_mut().take(b.min(n)).skip(a) {
            if *c != b'\n' {
                *c = b' ';
            }
        }
    };
    let mut i = 0;
    while i < n {
        let c = src[i];
        if src[i..].starts_with(b"//") {
            let j = src[i..]
                .iter()
                .position(|&b| b == b'\n')
                .map_or(n, |p| i + p);
            blank(&mut out, i, j);
            i = j;
        } else if src[i..].starts_with(b"/*") {
            let (mut d, mut j) = (1, i + 2);
            while j < n && d > 0 {
                if src[j..].starts_with(b"/*") {
                    d += 1;
                    j += 2;
                } else if src[j..].starts_with(b"*/") {
                    d -= 1;
                    j += 2;
                } else {
                    j += 1;
                }
            }
            blank(&mut out, i, j);
            i = j;
        } else if c == b'r'
            && (i == 0 || !is_ident(src[i - 1]))
            && raw_open(&src[i + 1..]).is_some()
        {
            let hashes = raw_open(&src[i + 1..]).unwrap_or(0);
            let body = i + 2 + hashes;
            let mut close = vec![b'"'];
            close.extend(std::iter::repeat_n(b'#', hashes));
            let j = src[body..]
                .windows(close.len())
                .position(|w| w == close.as_slice())
                .map_or(n, |p| body + p + close.len());
            blank(&mut out, i + 1, j);
            i = j;
        } else if c == b'"' {
            let mut j = i + 1;
            while j < n && src[j] != b'"' {
                j += if src[j] == b'\\' { 2 } else { 1 };
            }
            blank(&mut out, i + 1, j);
            i = j + 1;
        } else if c == b'\'' {
            i = char_literal_end(s, i).map_or(i + 1, |j| {
                blank(&mut out, i + 1, j);
                j + 1
            });
        } else {
            i += 1;
        }
    }
    out
}

/// `#…#"` depois de um `r`: quantos `#`.
fn raw_open(rest: &[u8]) -> Option<usize> {
    let h = rest.iter().take_while(|&&b| b == b'#').count();
    (rest.get(h) == Some(&b'"')).then_some(h)
}

/// O índice do `'` que fecha um literal de carácter aberto em `i` — `None` para um lifetime.
fn char_literal_end(s: &str, i: usize) -> Option<usize> {
    let b = s.as_bytes();
    if b.get(i + 1) == Some(&b'\\') {
        return b
            .get(i + 3..)?
            .iter()
            .position(|&c| c == b'\'')
            .map(|p| i + 3 + p);
    }
    let ch = s.get(i + 1..)?.chars().next()?;
    let j = i + 1 + ch.len_utf8();
    (b.get(j) == Some(&b'\'')).then_some(j)
}

fn match_close(t: &[u8], open: usize, o: u8, c: u8) -> Option<usize> {
    let mut d = 0i32;
    for (j, &b) in t.iter().enumerate().skip(open) {
        if b == o {
            d += 1;
        } else if b == c {
            d -= 1;
            if d == 0 {
                return Some(j);
            }
        }
    }
    None
}

/// O primeiro `{` fora de `()`/`[]` a partir de `from`; `None` se um `;` vier antes.
fn header_brace(t: &[u8], from: usize) -> Option<usize> {
    let mut d = 0i32;
    for (j, &b) in t.iter().enumerate().skip(from) {
        match b {
            b'(' | b'[' => d += 1,
            b')' | b']' => d -= 1,
            b'{' if d == 0 => return Some(j),
            b';' if d == 0 => return None,
            _ => {}
        }
    }
    None
}

fn word_at(t: &[u8], i: usize, w: &[u8]) -> bool {
    t[i..].starts_with(w)
        && (i == 0 || !is_ident(t[i - 1]))
        && t.get(i + w.len()).is_none_or(|&b| !is_ident(b))
}

fn has_word(t: &[u8], w: &[u8]) -> bool {
    (0..t.len()).any(|i| word_at(t, i, w))
}

/// Os corpos `(abre, fecha)` dos laços e das closures com chaves.
fn corpos(t: &[u8]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for i in 0..t.len() {
        for kw in [&b"for"[..], &b"while"[..], &b"loop"[..]] {
            if !word_at(t, i, kw) {
                continue;
            }
            let from = i + kw.len();
            let Some(b) = header_brace(t, from) else {
                continue;
            };
            let head = &t[from..b];
            let ok = match kw {
                b"for" => has_word(head, b"in"),
                b"loop" => head.iter().all(u8::is_ascii_whitespace),
                _ => true,
            };
            if let (true, Some(e)) = (ok, match_close(t, b, b'{', b'}')) {
                out.push((b, e));
            }
        }
        if t[i] == b'|'
            && let Some(b) = closure_brace(t, i)
            && let Some(e) = match_close(t, b, b'{', b'}')
        {
            out.push((b, e));
        }
    }
    out
}

/// Se um `|` em `i` abre os parâmetros de uma closure cujo corpo é um bloco, o `{` desse bloco.
fn closure_brace(t: &[u8], i: usize) -> Option<usize> {
    let before = t[..i].iter().rposition(|b| !b.is_ascii_whitespace())?;
    let after_move = before >= 3 && word_at(t, before - 3, b"move");
    if !matches!(t[before], b'(' | b',' | b'=') && !after_move {
        return None;
    }
    if t[before] == b'=' && before > 0 && matches!(t[before - 1], b'=' | b'!' | b'<' | b'>') {
        return None;
    }
    let close = if t.get(i + 1) == Some(&b'|') {
        i + 1
    } else {
        i + 1 + t[i + 1..].iter().position(|&b| b == b'|')?
    };
    let mut j = close + 1;
    while t.get(j).is_some_and(u8::is_ascii_whitespace) {
        j += 1;
    }
    match t.get(j)? {
        b'{' => Some(j),
        b'-' if t.get(j + 1) == Some(&b'>') => {
            let k = j + 2 + t[j + 2..].iter().position(|&b| b == b'{' || b == b';')?;
            (t[k] == b'{').then_some(k)
        }
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tipo {
    Avanca,
    Coze,
}

/// `(posição, tipo, argumento de playhead sem espaços)` de cada chamada.
fn chamadas(t: &[u8]) -> Vec<(usize, Tipo, String)> {
    const AVANCA: &[&str] = &[
        "advance_tick",
        "advance_tick_fanned",
        "advance_tick_scoped",
        "advance_tick_fanned_within",
    ];
    const COZE: &[&str] = &["cook", "cook_scoped", "cook_scoped_fanned"];
    let mut out = Vec::new();
    for i in 0..t.len() {
        if t[i] != b'.' {
            continue;
        }
        let end = i + 1 + t[i + 1..].iter().take_while(|&&b| is_ident(b)).count();
        let name = std::str::from_utf8(&t[i + 1..end]).unwrap_or("");
        let (tipo, idx) = if AVANCA.contains(&name) {
            (Tipo::Avanca, 2)
        } else if COZE.contains(&name) {
            (Tipo::Coze, 3)
        } else {
            continue;
        };
        let mut p = end;
        while t.get(p).is_some_and(u8::is_ascii_whitespace) {
            p += 1;
        }
        if t.get(p) != Some(&b'(') {
            continue;
        }
        let Some(q) = match_close(t, p, b'(', b')') else {
            continue;
        };
        let mut args = vec![String::new()];
        let mut d = 0i32;
        for &b in &t[p + 1..q] {
            match b {
                b'(' | b'[' | b'{' => d += 1,
                b')' | b']' | b'}' => d -= 1,
                _ => {}
            }
            if b == b',' && d == 0 {
                args.push(String::new());
            } else if !b.is_ascii_whitespace()
                && let Some(a) = args.last_mut()
            {
                a.push(char::from(b));
            }
        }
        if let Some(arg) = args.get(idx).filter(|a| !a.is_empty()) {
            out.push((i, tipo, arg.clone()));
        }
    }
    out
}

fn fn_antes(t: &[u8], pos: usize) -> String {
    (0..pos)
        .rev()
        .find(|&i| word_at(t, i, b"fn"))
        .map(|i| {
            let s = &t[i + 2..];
            let s = &s[s.iter().take_while(|b| b.is_ascii_whitespace()).count()..];
            let n = s.iter().take_while(|&&b| is_ident(b)).count();
            String::from_utf8_lossy(&s[..n]).into_owned()
        })
        .unwrap_or_default()
}

/// `(corpos examinados, corpos com avanço E cozimento, achados)`.
fn censo(src: &str) -> (usize, usize, Vec<Achado>) {
    let t = sanitize(src);
    let bs = corpos(&t);
    let cs = chamadas(&t);
    let circuitos = bs
        .iter()
        .filter(|&&(b, e)| {
            let dentro = |k| cs.iter().any(|(p, tk, _)| *tk == k && b < *p && *p < e);
            dentro(Tipo::Avanca) && dentro(Tipo::Coze)
        })
        .count();
    let mut achados = Vec::new();
    for (pa, _, arg) in cs.iter().filter(|c| c.1 == Tipo::Avanca) {
        let Some(&(_, e)) = bs
            .iter()
            .filter(|(b, e)| b < pa && pa < e)
            .min_by_key(|(b, e)| e - b)
        else {
            continue;
        };
        if cs
            .iter()
            .any(|(pc, tc, ac)| *tc == Tipo::Coze && pa < pc && *pc < e && ac == arg)
        {
            achados.push(Achado {
                linha: t[..*pa].iter().filter(|&&b| b == b'\n').count() + 1,
                func: fn_antes(&t, *pa),
                playhead: arg.clone(),
            });
        }
    }
    (bs.len(), circuitos, achados)
}

fn ficheiros(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("ler {}: {e}", dir.display()));
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            ficheiros(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// ⭐ O CONTROLO: a régua vê o laço e a closure invertidos, e cala-se na ordem do produto — com
/// comentário, literal com chave e dois laços irmãos sob um laço de fora (que não é inversão).
#[test]
fn o_censo_ve_a_inversao_e_cala_se_na_ordem_do_produto() {
    let invertido = r#"
fn marcha() {
    for k in 0..9 {
        let t = k as f64;
        cook.advance_tick(&g, &reg, t).expect("avanca");
        for s in &sinks {
            let o = cook.cook(&g, &reg, *s, t).expect("coze {");
        }
    }
    let ps = |t: f64, cook: &mut Cook| {
        cook.advance_tick(g, &reg, t).expect("avanca");
        cook.cook(g, &reg, fk, t).expect("coze")
    };
}
"#;
    let (_, circ, achados) = censo(invertido);
    assert_eq!(circ, 2, "os dois corpos são circuitos");
    assert_eq!(
        achados,
        vec![
            Achado {
                linha: 5,
                func: "marcha".into(),
                playhead: "t".into()
            },
            Achado {
                linha: 11,
                func: "marcha".into(),
                playhead: "t".into()
            },
        ],
        "a régua tem de acusar o laço E a closure"
    );

    let produto = r#"
fn marcha() {
    for k in 0..9 {
        let t = k as f64;
        // cook.advance_tick(&g, &reg, t) num comentário não conta
        let _ = "cook.advance_tick(&g, &reg, t) { num literal também não";
        for s in &sinks {
            let o = cook.cook(&g, &reg, *s, t).expect("coze }");
        }
        cook.advance_tick(&g, &reg, t).expect("avanca");
    }
    for caso in casos {
        for k in 0..9 {
            let t = k as f64;
            cook.cook(&g, &reg, a, t).expect("coze");
            cook.advance_tick(&g, &reg, t).expect("avanca");
        }
        for k in 0..9 {
            let t = k as f64;
            cook.cook(&g, &reg, a, t).expect("coze");
            cook.advance_tick(&g, &reg, t).expect("avanca");
        }
    }
}
"#;
    let (_, circ, achados) = censo(produto);
    assert_eq!(
        circ, 4,
        "quatro corpos têm avanço e cozimento (o de fora inclusive)"
    );
    assert!(
        achados.is_empty(),
        "a ordem do produto não é inversão: {achados:?}"
    );
}

#[test]
fn nenhum_laco_avanca_o_tique_antes_de_cozer() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (mut corpos_n, mut circuitos, mut usados) = (0, 0, vec![false; ISENTOS.len()]);
    let mut recusas = Vec::new();
    for raiz in RAIZES {
        let mut fs = Vec::new();
        ficheiros(&base.join(raiz), &mut fs);
        assert!(
            !fs.is_empty(),
            "a raiz `{raiz}` não tem ficheiros — o censo leria nada"
        );
        fs.sort();
        for f in fs {
            let src = std::fs::read_to_string(&f).expect("ler o fonte");
            let (n, c, achados) = censo(&src);
            corpos_n += n;
            circuitos += c;
            let nome = f.file_name().and_then(|x| x.to_str()).unwrap_or_default();
            for a in achados {
                let chave = format!("{nome}:{}", a.func);
                match ISENTOS.iter().position(|(k, _)| *k == chave) {
                    Some(k) => usados[k] = true,
                    None => recusas.push(format!(
                        "{}:{} (fn {}) — `advance_tick(…, {})` antes do `cook(…, {})`",
                        f.strip_prefix(base).unwrap_or(&f).display(),
                        a.linha,
                        a.func,
                        a.playhead,
                        a.playhead
                    )),
                }
            }
        }
    }
    eprintln!("ordem do tique: {corpos_n} corpos lidos, {circuitos} com avanço e cozimento");
    assert!(
        circuitos >= PISO_DE_CIRCUITOS,
        "o censo só viu {circuitos} circuitos (piso {PISO_DE_CIRCUITOS}) — está a ler o sítio errado"
    );
    let mortos: Vec<_> = ISENTOS
        .iter()
        .zip(&usados)
        .filter(|(_, u)| !**u)
        .map(|((k, _), _)| *k)
        .collect();
    assert!(
        mortos.is_empty(),
        "isenções que já não casam nada (apague-as): {mortos:?}"
    );
    assert!(
        recusas.is_empty(),
        "laços que avançam o tique ANTES de cozer o quadro — cozem cada nó sequencial duas vezes \
         por tique e medem outro programa (doc 123 §4). Cozer, e DEPOIS `advance_tick` com o \
         mesmo playhead:\n  {}",
        recusas.join("\n  ")
    );
}
