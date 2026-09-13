//! ⛔ **Só a porta lê o conjunto DIRECTO de tags** — gate 12 do plano de Tags
//! (`docs/Components/08_plano_tags.md` §5.2).
//!
//! # O defeito que isto cerca
//!
//! *«Este objecto pertence a `Enemy`?»* tem uma resposta com HIERARQUIA: um objecto marcado
//! `Enemy/Flying` pertence (o `all_objects` do Blender, medido). O conjunto directo diz que não. Um
//! consumidor que respondesse lendo [`ph2d_ecs::Tags::direct_ids`] compilaria, passaria nos gates dele
//! com uma fixture de um nível só, e erraria no dia em que o artista criasse a primeira tag-filha.
//!
//! ⇒ `direct_ids` é para quem MOSTRA as tags (os chips); quem PERGUNTA passa por `belongs`/`tagged`.
//! O campo do `Tags` é privado, então o compilador já fecha o acesso ao `BTreeSet`; este censo fecha
//! a porta que sobra.
//!
//! # ⚠️ As duas metades obrigatórias (HOWTO §2.7 e CLAUDE.md §5.0)
//!
//! - **Piso de população**: um censo que passa a varrer zero ficheiros fica verde a medir nada. Ele
//!   exige ler a porta e encontrar nela, depois de tirar comentários e textos, as duas funções.
//! - **Obsolescência**: cada leitor autorizado tem de AINDA ler — senão a lista vira licença.

use std::path::{Path, PathBuf};

/// O ficheiro da porta, e o dos gates dela.
const DOOR: &[&str] = &[
    "crates/ph2d-ecs/src/tags.rs",
    "crates/ph2d-ecs/src/tags_tests.rs",
];

/// Quem pode ler o conjunto directo fora da porta, e PORQUÊ. ⚠️ Entrada nova = uma razão de
/// MOSTRAR, nunca de decidir.
const READERS: &[(&str, &str)] = &[];

/// O que se procura, já sem comentários nem textos.
const NEEDLE: &str = "direct_ids";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("a raiz da workspace")
        .to_path_buf()
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name();
        let name = name.to_string_lossy();
        if p.is_dir() {
            if name == "target" || name.starts_with('.') {
                continue;
            }
            rust_files(&p, out);
        } else if name.ends_with(".rs") {
            out.push(p);
        }
    }
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// O texto sem comentários (`//`, `/* */` aninhados) nem literais (`"…"`, `r#"…"#`, `'x'`).
///
/// ⚠️ **Um lifetime (`'a`) não é um literal**, e comê-lo engoliria código até à próxima aspa.
fn strip(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c == b'/' && b.get(i + 1) == Some(&b'*') {
            let mut depth = 1;
            i += 2;
            while i < b.len() && depth > 0 {
                if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
                    depth += 1;
                    i += 2;
                } else if b[i] == b'*' && b.get(i + 1) == Some(&b'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        if c == b'r' && (i == 0 || !is_ident(b[i - 1])) {
            let mut j = i + 1;
            let mut hashes = 0;
            while b.get(j) == Some(&b'#') {
                hashes += 1;
                j += 1;
            }
            if b.get(j) == Some(&b'"') && (hashes > 0 || j == i + 1) {
                j += 1;
                while j < b.len() {
                    if b[j] == b'"'
                        && b.len() >= j + 1 + hashes
                        && b[j + 1..j + 1 + hashes].iter().all(|&h| h == b'#')
                    {
                        j += 1 + hashes;
                        break;
                    }
                    j += 1;
                }
                i = j;
                out.extend_from_slice(b"\"\"");
                continue;
            }
        }
        if c == b'"' {
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if b[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            out.extend_from_slice(b"\"\"");
            continue;
        }
        if c == b'\'' {
            if b.get(i + 1) == Some(&b'\\') {
                let mut j = i + 2;
                while j < b.len() && b[j] != b'\'' {
                    j += 1;
                }
                i = j + 1;
                continue;
            }
            // Um carácter (ASCII ou até 4 bytes) fechado por aspa é literal; o resto é lifetime.
            if let Some(k) = (2..=5).find(|&k| b.get(i + k) == Some(&b'\''))
                && std::str::from_utf8(&b[i + 1..i + k]).is_ok_and(|s| s.chars().count() == 1)
            {
                i += k + 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn relative(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .expect("dentro da raiz")
        .to_string_lossy()
        .replace('\\', "/")
}

#[test]
fn only_the_door_reads_tags() {
    let root = workspace_root();
    let mut files = Vec::new();
    for top in ["crates", "shells", "tools"] {
        rust_files(&root.join(top), &mut files);
    }

    // ⚠️ Piso: a workspace tem milhares de ficheiros; uma varredura partida lê dezenas.
    assert!(
        files.len() > 1_000,
        "o censo so' leu {} ficheiros .rs -- a varredura partiu",
        files.len()
    );
    let door = strip(
        &std::fs::read_to_string(root.join(DOOR[0]))
            .expect("a porta existe onde o censo a procura"),
    );
    for code in ["fn belongs", "fn tagged", "fn direct_ids"] {
        assert!(
            door.contains(code),
            "a porta, sem comentarios, nao contem `{code}` -- o `strip` comeu codigo ou a porta mudou de sitio"
        );
    }

    let mut outside = Vec::new();
    let mut still_reading = vec![false; READERS.len()];
    for f in &files {
        let rel = relative(&root, f);
        if DOOR.contains(&rel.as_str()) {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        if !src.contains(NEEDLE) {
            continue;
        }
        if !strip(&src).contains(NEEDLE) {
            continue;
        }
        match READERS.iter().position(|(p, _)| *p == rel) {
            Some(k) => still_reading[k] = true,
            None => outside.push(rel),
        }
    }
    assert!(
        outside.is_empty(),
        "lido o conjunto DIRECTO de tags fora da porta: {outside:?}\n\
         Para responder «pertence?» use `ph2d_ecs::tags::belongs`/`tagged` (alcancam a subarvore); \
         para MOSTRAR os chips, acrescente o ficheiro a `READERS` com a razao."
    );
    let stale: Vec<&str> = READERS
        .iter()
        .zip(&still_reading)
        .filter(|(_, lê)| !**lê)
        .map(|((p, _), _)| *p)
        .collect();
    assert!(
        stale.is_empty(),
        "leitores autorizados que ja' nao leem -- apague-os da lista: {stale:?}"
    );
}

/// ⚠️ **O `strip` é o instrumento, e um instrumento tem controlo** — senão um `strip` que apagasse
/// tudo deixava o censo verde (o piso da porta apanha-o, e este apanha-o com o nome certo).
#[test]
fn the_stripper_keeps_code_and_drops_comments_strings_and_chars() {
    let src = "fn a<'x>(s: &'x str) -> char { let _ = \"direct_ids\"; // direct_ids\n\
               /* direct_ids /* aninhado */ */ let r = r#\"direct_ids\"#; let c = 'é'; t.direct_ids() }";
    let s = strip(src);
    assert_eq!(s.matches(NEEDLE).count(), 1, "{s}");
    assert!(s.contains("fn a<'x>(s: &'x str)"), "o lifetime ficou: {s}");
    assert!(!s.contains('é'), "o literal de caracter saiu: {s}");
}
