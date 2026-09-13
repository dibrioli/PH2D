//! ⭐⭐ **Os módulos de topo da `ph2d-editor-core` formam um DAG** — auditoria de arquitectura
//! 2026-09-12, A10.
//!
//! # O defeito, medido
//!
//! A fundação é o maior leque do repo: 43 crates dependem dela, e cada edição a `src/` recompila-as.
//! Parti-la em crates é a cura óbvia — e é **inexprimível** enquanto os módulos dependerem uns dos
//! outros nos DOIS sentidos: um módulo que usa outro e é usado por ele não vira crate própria sem
//! um ciclo de dependências. A auditoria mediu três pares (`widget ↔ interaction` 73/165,
//! `screens ↔ interaction` 162/22, `interaction ↔ ids` 90/10); medido pela ÁRVORE de módulos, depois
//! da descida dos ids (A5b), era **UM ciclo de 23 módulos com 1 191 referências** — e o
//! `interaction ↔ ids` já tinha morrido com ela.
//!
//! As curas por ASSUNTO desta auditoria (um TIPO desce para o módulo dono do conceito, uma LEI desce
//! para quem a corre, uma TABELA é injectada por quem chama; ⛔ nunca um re-export) partiram as
//! arestas pequenas: os três ficheiros do `action_bus` passaram a filhos dele, os gates régua×porta
//! voltaram para a régua, o pintor da fila de avisos mudou-se para o `toast`, e os pintores de texto
//! cortado para o `paint`. Medido depois delas: **UM ciclo de 14 módulos com 835 referências**, que
//! as sete arestas da catraca fecham — cada uma é necessária sozinha. O que sobra está lá, com o
//! número e a cura escrita.
//!
//! # O que conta como aresta
//!
//! - **O módulo de topo de um ficheiro** lê-se da ÁRVORE, a partir do `lib.rs` e seguindo `#[path]` —
//!   nunca do nome do ficheiro: o `motion_tests.rs` é do `motion`, o `action_bus_queue.rs` do
//!   `action_bus`.
//! - **Uma referência** é `crate::X`, cada `X` de `crate::{X, …}`, e uma cadeia `super::…::X` que
//!   sobe até à raiz — só em CÓDIGO (comentários e literais de string em branco).
//! - **Os testes contam.** Um `#[cfg(test)]` que chama o módulo de cima também impede o corte: numa
//!   crate partida, esse teste teria de ver a crate de cima, e a de cima dependeria dele.
//! - **Uma re-exportação do `lib.rs` vê-se através:** `crate::Toast` é aresta para o `toast`. ⚠️ Sem
//!   isto, esconder uma aresta atrás de um `pub use` na raiz passava — que é exactamente a cura que
//!   esta auditoria proíbe.
//!
//! # A catraca
//!
//! `(de, para, tecto de referências, porquê)`. Só ENCOLHE: acima do tecto reprova (a aresta cresceu);
//! abaixo dele reprova também (desça o número — senão a folga é licença para voltar a crescer); e uma
//! entrada que já não fecha ciclo nenhum reprova (apague a linha). ⭐ **Alvo: catraca VAZIA.**

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// ⛔ **A catraca — só encolhe.** Tectos medidos em 2026-09-12, com este leitor.
const ARESTAS_TOLERADAS: &[(&str, &str, usize, &str)] = &[
    (
        "widget",
        "interaction",
        49,
        "o SUBSTRATO que os pintores recebem — `HitIndex`, `WidgetStore`, `InteractiveState`, \
         `WidgetEvent`. Cura: o estado de widget (os `*State` + `InteractiveState` + `HitIndex` + \
         `WidgetStore`) desce para um módulo ABAIXO dos pintores, e o `dispatch` fica em cima.",
    ),
    (
        "interaction",
        "screens",
        18,
        "os tipos de DOCA que o estado guarda (`DockSide`, `TaskLayout`, `Slot`, `ChromeBands`, \
         `TIMELINE_DOCK_H`) e três tabelas do hero que o despacho consulta (`panel_for_tab`, \
         `MENUS`, `LEGACY_PILL_MENUS`). Cura: os tipos descem para um módulo de doca abaixo do \
         `interaction`; as tabelas são injectadas pelo hero. ⚠️ `Slot`/`SlotSet` têm 144 leitores \
         em 86 ficheiros fora da fundação.",
    ),
    (
        "action_bus",
        "screens",
        24,
        "os PAYLOADS do Inspector (`*FieldEdit`, `*Info`, `HierReparentIntent`) moram em \
         `screens::hero::inspector_model*`. Cura: descem para um módulo de vocabulário abaixo do \
         `action_bus`. ⛔ Nesta rodada a cerca da `line/render-loop` nomeia quatro deles pelo \
         caminho `screens::hero::` — descem depois das duas fusões.",
    ),
    (
        "panel",
        "screens",
        7,
        "o contrato do painel nomeia a doca (`slot::{Slot, SlotSet}`) e o hero (`HeroSelection`, \
         `HeroLayout`). Cura: os tipos de doca da entrada `interaction → screens`; a selecção e o \
         layout passam ao contrato como os tipos de baixo que eles são.",
    ),
    (
        "motion",
        "interaction",
        4,
        "só em teste: `motion_surface_tests.rs` e `motion_grid_tests.rs` montam um `WidgetStore`. \
         Cura: os testes sobem para o `interaction` — o `motion_grid_tests` lê privados do `motion` \
         (`Track`, `value`), que precisam de porta antes.",
    ),
    (
        "motion",
        "screens",
        1,
        "só em teste: `motion_grid_tests.rs` monta o `HeroScreen`. A mesma cura da entrada acima.",
    ),
    (
        "motion",
        "widget",
        1,
        "só em teste: um teste do `motion_tests.rs` pinta um `Button`. Cura: ele sobe para o `widget`.",
    ),
];

/// A metade justa — medido 2026-09-12: 32 módulos de topo, 448 ficheiros na árvore, 1 615 referências.
const PISO_MODULOS: usize = 30;
const PISO_FICHEIROS: usize = 420;
const PISO_REFERENCIAS: usize = 1_500;
/// ⚠️ **A metade justa da leitura ATRAVÉS da raiz** — medido 2026-09-12: 4 referências só viram aresta
/// por um `pub use` do `lib.rs` (`crate::LengthDisplay`, no `gizmo/readout_tests.rs`). Nenhuma delas
/// muda o veredito de hoje, e foi por isso que a mutação que desligava a leitura através SOBREVIVEU
/// à primeira prova: sem este piso, o leitor podia deixar de ver através de uma fachada na raiz e
/// continuar verde.
const PISO_VIA_REEXPORT: usize = 1;

fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Comentários (e, com `strings = true`, literais de string e de carácter) em branco, com os
/// offsets e as quebras de linha preservados.
fn blank(src: &str, strings: bool) -> String {
    let b = src.as_bytes();
    let n = b.len();
    let mut out = b.to_vec();
    let apaga = |out: &mut Vec<u8>, a: usize, e: usize| {
        for x in out.iter_mut().take(e.min(n)).skip(a) {
            if *x != b'\n' {
                *x = b' ';
            }
        }
    };
    let mut i = 0;
    while i < n {
        let c = b[i];
        if c == b'/' && i + 1 < n && b[i + 1] == b'/' {
            let e = src[i..].find('\n').map_or(n, |x| i + x);
            apaga(&mut out, i, e);
            i = e;
        } else if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            let (mut depth, mut j) = (1, i + 2);
            while j < n && depth > 0 {
                if b[j] == b'/' && j + 1 < n && b[j + 1] == b'*' {
                    depth += 1;
                    j += 2;
                } else if b[j] == b'*' && j + 1 < n && b[j + 1] == b'/' {
                    depth -= 1;
                    j += 2;
                } else {
                    j += 1;
                }
            }
            apaga(&mut out, i, j);
            i = j;
        } else if c == b'r'
            && i + 1 < n
            && (b[i + 1] == b'"' || b[i + 1] == b'#')
            && (i == 0 || !is_ident(b[i - 1]))
        {
            let mut j = i + 1;
            while j < n && b[j] == b'#' {
                j += 1;
            }
            if j < n && b[j] == b'"' {
                let fecho = format!("\"{}", "#".repeat(j - i - 1));
                let e = src[j + 1..]
                    .find(&fecho)
                    .map_or(n, |x| j + 1 + x + fecho.len());
                if strings {
                    apaga(&mut out, i, e);
                }
                i = e;
            } else {
                i += 1;
            }
        } else if c == b'"' {
            let mut j = i + 1;
            while j < n && b[j] != b'"' {
                if b[j] == b'\\' {
                    j += 1;
                }
                j += 1;
            }
            let e = (j + 1).min(n);
            if strings {
                apaga(&mut out, i, e);
            }
            i = e;
        } else if c == b'\'' {
            // Um literal de carácter (`'x'`, `'\n'`, `'é'`) — ou um tempo de vida (`'a`), que não fecha.
            let e = if i + 1 < n && b[i + 1] == b'\\' {
                src[i + 2..].find('\'').map(|x| i + 2 + x + 1)
            } else {
                src[i + 1..].chars().next().and_then(|ch| {
                    let fim = i + 1 + ch.len_utf8();
                    (fim < n && b[fim] == b'\'').then_some(fim + 1)
                })
            };
            match e {
                Some(e) => {
                    if strings {
                        apaga(&mut out, i, e);
                    }
                    i = e;
                }
                None => i += 1,
            }
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).expect("só bytes ASCII foram escritos")
}

fn ident_at(s: &str, i: usize) -> Option<&str> {
    let b = s.as_bytes();
    let e = (i..b.len()).find(|&k| !is_ident(b[k])).unwrap_or(b.len());
    (e > i && !b[i].is_ascii_digit()).then(|| &s[i..e])
}

/// Os `{ … }` de cada `mod NOME {` inline — o que está dentro deles é um nível mais fundo.
fn inline_mods(cod: &str) -> Vec<(usize, usize)> {
    let b = cod.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(p) = cod[i..].find("mod ") {
        let at = i + p;
        i = at + 4;
        if at > 0 && is_ident(b[at - 1]) {
            continue;
        }
        let Some(nome) = ident_at(cod, at + 4) else {
            continue;
        };
        let mut j = at + 4 + nome.len();
        while j < b.len() && b[j].is_ascii_whitespace() {
            j += 1;
        }
        if j < b.len() && b[j] == b'{' {
            let mut depth = 0usize;
            for (k, &ch) in b.iter().enumerate().skip(j) {
                if ch == b'{' {
                    depth += 1;
                } else if ch == b'}' {
                    depth -= 1;
                    if depth == 0 {
                        out.push((j, k));
                        break;
                    }
                }
            }
        }
    }
    out
}

/// `(ficheiro, caminho de módulo)` de todo ficheiro que a árvore alcança a partir do `lib.rs`.
fn arvore(src: &Path) -> Vec<(PathBuf, Vec<String>)> {
    let lib = src.join("lib.rs");
    let mut out = vec![(lib.clone(), Vec::new())];
    let mut fila = vec![(lib, Vec::<String>::new())];
    let mut vistos = BTreeSet::new();
    while let Some((f, caminho)) = fila.pop() {
        if !vistos.insert(f.clone()) {
            continue;
        }
        let texto = std::fs::read_to_string(&f).unwrap_or_default();
        let sem_coment = blank(&texto, false);
        let cod = blank(&texto, true);
        let dentro = inline_mods(&cod);
        let dir = f.parent().expect("dir").to_path_buf();
        let nome_f = f.file_name().and_then(|x| x.to_str()).unwrap_or("");
        let sob = if nome_f == "lib.rs" || nome_f == "mod.rs" {
            dir.clone()
        } else {
            dir.join(nome_f.trim_end_matches(".rs"))
        };
        let b = cod.as_bytes();
        let mut i = 0;
        while let Some(p) = cod[i..].find("mod ") {
            let at = i + p;
            i = at + 4;
            if (at > 0 && is_ident(b[at - 1])) || dentro.iter().any(|&(a, e)| at > a && at < e) {
                continue;
            }
            let Some(nome) = ident_at(&cod, at + 4) else {
                continue;
            };
            let mut j = at + 4 + nome.len();
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j >= b.len() || b[j] != b';' {
                continue;
            }
            // Os atributos das linhas logo acima (e da mesma linha): o `#[path = "…"]`.
            let ini_linha = cod[..at].rfind('\n').map_or(0, |x| x + 1);
            let mut attrs_ini = ini_linha;
            while attrs_ini > 0 {
                let prev = sem_coment[..attrs_ini - 1].rfind('\n').map_or(0, |x| x + 1);
                if sem_coment[prev..attrs_ini - 1]
                    .trim_start()
                    .starts_with("#[")
                {
                    attrs_ini = prev;
                } else {
                    break;
                }
            }
            let attrs = &sem_coment[attrs_ini..at];
            let alvo = attrs.find("#[path").and_then(|k| {
                let r = &attrs[k..];
                let a = r.find('"')? + 1;
                let e = r[a..].find('"')? + a;
                Some(dir.join(&r[a..e]))
            });
            let cands = match alvo {
                Some(p) => vec![p],
                None => vec![
                    sob.join(format!("{nome}.rs")),
                    sob.join(nome).join("mod.rs"),
                ],
            };
            if let Some(c) = cands.into_iter().find(|c| c.is_file()) {
                let mut filho = caminho.clone();
                filho.push(nome.to_owned());
                out.push((c.clone(), filho.clone()));
                fila.push((c, filho));
            }
        }
    }
    out
}

/// `nome re-exportado na raiz → módulo de topo` (os `pub use módulo::…` do `lib.rs`).
fn reexportacoes(lib: &str, tops: &BTreeSet<String>) -> BTreeMap<String, String> {
    let cod = blank(lib, true);
    let mut out = BTreeMap::new();
    for stmt in cod.split(';') {
        let t = stmt.trim();
        let Some(corpo) = t.strip_prefix("pub use ") else {
            continue;
        };
        let corpo: String = corpo.split_whitespace().collect::<Vec<_>>().join(" ");
        let topo = corpo.split("::").next().unwrap_or("").trim();
        if !tops.contains(topo) {
            continue;
        }
        let resto = corpo[topo.len()..].trim_start_matches("::");
        let itens: Vec<String> = if let Some(g) = resto.rfind('{') {
            resto[g + 1..]
                .trim_end_matches('}')
                .split(',')
                .map(str::to_owned)
                .collect()
        } else {
            vec![resto.to_owned()]
        };
        for it in itens {
            let it = it.trim();
            let nome = it
                .rsplit(" as ")
                .next()
                .unwrap_or(it)
                .rsplit("::")
                .next()
                .unwrap_or(it)
                .trim();
            if !nome.is_empty() && nome != "*" && nome != "self" {
                out.insert(nome.to_owned(), topo.to_owned());
            }
        }
    }
    out
}

/// `(de, para) → (referências, sítios de exemplo)`.
type Arestas = BTreeMap<(String, String), (usize, Vec<String>)>;

fn referencias(texto: &str, caminho: &[String]) -> Vec<(usize, String)> {
    let cod = blank(texto, true);
    let b = cod.as_bytes();
    let dentro = inline_mods(&cod);
    let mut out = Vec::new();
    let precede = |at: usize| at > 0 && (is_ident(b[at - 1]) || b[at - 1] == b':');
    let mut i = 0;
    while let Some(p) = cod[i..].find("crate::") {
        let at = i + p;
        i = at + 7;
        if precede(at) {
            continue;
        }
        if b.get(at + 7) == Some(&b'{') {
            let mut depth = 0usize;
            let mut k = at + 7;
            let mut fim = b.len();
            while k < b.len() {
                if b[k] == b'{' {
                    depth += 1;
                } else if b[k] == b'}' {
                    depth -= 1;
                    if depth == 0 {
                        fim = k;
                        break;
                    }
                }
                k += 1;
            }
            let mut nivel = 0usize;
            let mut item_ini = at + 8;
            for (q, &ch) in b.iter().enumerate().take(fim + 1).skip(at + 8) {
                match ch {
                    b'{' => nivel += 1,
                    b'}' if nivel > 0 => nivel -= 1,
                    b',' | b'}' if nivel == 0 => {
                        let bruto = &cod[item_ini..q];
                        let off = item_ini + (bruto.len() - bruto.trim_start().len());
                        if let Some(nome) = ident_at(&cod, off) {
                            out.push((at, nome.to_owned()));
                        }
                        item_ini = q + 1;
                    }
                    _ => {}
                }
            }
        } else if let Some(nome) = ident_at(&cod, at + 7) {
            out.push((at, nome.to_owned()));
        }
    }
    let mut i = 0;
    while let Some(p) = cod[i..].find("super::") {
        let at = i + p;
        i = at + 7;
        if precede(at) {
            continue;
        }
        let mut n = 0;
        let mut k = at;
        while cod[k..].starts_with("super::") {
            n += 1;
            k += 7;
        }
        i = k;
        let profundidade =
            caminho.len() + dentro.iter().filter(|&&(a, e)| at > a && at < e).count();
        if n == profundidade {
            if let Some(nome) = ident_at(&cod, k) {
                out.push((at, nome.to_owned()));
            }
        }
    }
    out
}

fn grafo() -> (BTreeSet<String>, usize, Arestas, usize) {
    let src = src_root();
    let arv = arvore(&src);
    let tops: BTreeSet<String> = arv.iter().filter_map(|(_, c)| c.first().cloned()).collect();
    let lib = std::fs::read_to_string(src.join("lib.rs")).expect("lib.rs");
    let reexp = reexportacoes(&lib, &tops);
    let mut arestas = Arestas::new();
    let mut via_reexport = 0;
    for (f, caminho) in &arv {
        let Some(de) = caminho.first() else { continue };
        let texto = std::fs::read_to_string(f).unwrap_or_default();
        for (at, nome) in referencias(&texto, caminho) {
            // Um nome da raiz que também é módulo continua a ser o módulo.
            let (para, atraves) = match reexp.get(&nome) {
                Some(m) if !tops.contains(&nome) => (m.clone(), true),
                _ => (nome, false),
            };
            if tops.contains(&para) && &para != de {
                via_reexport += usize::from(atraves);
                let e = arestas.entry((de.clone(), para)).or_default();
                e.0 += 1;
                if e.1.len() < 3 {
                    let rel = f.strip_prefix(&src).unwrap_or(f).display().to_string();
                    e.1.push(format!("{rel}:{}", texto[..at].matches('\n').count() + 1));
                }
            }
        }
    }
    (tops, arv.len(), arestas, via_reexport)
}

/// Um ciclo no grafo sem as arestas de `fora`, se houver.
fn um_ciclo(
    tops: &BTreeSet<String>,
    arestas: &Arestas,
    fora: &BTreeSet<(String, String)>,
) -> Option<Vec<String>> {
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (a, b) in arestas.keys() {
        if !fora.contains(&(a.clone(), b.clone())) {
            adj.entry(a).or_default().push(b);
        }
    }
    fn dfs<'a>(
        v: &'a str,
        adj: &BTreeMap<&'a str, Vec<&'a str>>,
        cor: &mut BTreeMap<&'a str, u8>,
        pilha: &mut Vec<&'a str>,
    ) -> Option<Vec<String>> {
        cor.insert(v, 1);
        pilha.push(v);
        for &w in adj.get(v).map_or(&[][..], Vec::as_slice) {
            match cor.get(w) {
                Some(1) => {
                    let k = pilha.iter().position(|x| *x == w).unwrap_or(0);
                    let mut c: Vec<String> = pilha[k..].iter().map(|s| (*s).to_owned()).collect();
                    c.push(w.to_owned());
                    return Some(c);
                }
                None => {
                    if let Some(c) = dfs(w, adj, cor, pilha) {
                        return Some(c);
                    }
                }
                _ => {}
            }
        }
        pilha.pop();
        cor.insert(v, 2);
        None
    }
    let mut cor = BTreeMap::new();
    for v in tops {
        if !cor.contains_key(v.as_str()) {
            if let Some(c) = dfs(v, &adj, &mut cor, &mut Vec::new()) {
                return Some(c);
            }
        }
    }
    None
}

fn toleradas() -> BTreeSet<(String, String)> {
    ARESTAS_TOLERADAS
        .iter()
        .map(|(a, b, _, _)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
}

#[test]
fn the_foundation_modules_form_a_dag() {
    let (tops, ficheiros, arestas, via_reexport) = grafo();
    let total: usize = arestas.values().map(|(n, _)| n).sum();
    assert!(
        tops.len() >= PISO_MODULOS,
        "li {} módulos de topo (piso {PISO_MODULOS})",
        tops.len()
    );
    assert!(
        ficheiros >= PISO_FICHEIROS,
        "a árvore alcançou {ficheiros} ficheiros (piso {PISO_FICHEIROS})"
    );
    assert!(
        total >= PISO_REFERENCIAS,
        "li {total} referências entre módulos (piso {PISO_REFERENCIAS})"
    );
    assert!(
        via_reexport >= PISO_VIA_REEXPORT,
        "li {via_reexport} referências através de um `pub use` do lib.rs (piso {PISO_VIA_REEXPORT}) — \
         o leitor deixou de ver através da raiz, e uma fachada lá passaria a esconder uma aresta"
    );
    if let Some(ciclo) = um_ciclo(&tops, &arestas, &toleradas()) {
        let passos: Vec<String> = ciclo
            .windows(2)
            .map(|p| {
                let (n, ex) = &arestas[&(p[0].clone(), p[1].clone())];
                format!("{} → {} ({n}: {})", p[0], p[1], ex.join(", "))
            })
            .collect();
        panic!(
            "os módulos de topo da fundação voltaram a formar um CICLO:\n  {}\n\n\
             cura (a da auditoria A10): leve o que se usa do outro lado para o módulo do ASSUNTO — um \
             tipo para o módulo dono do conceito, uma lei para quem a corre, uma tabela injectada por \
             quem chama. ⛔ Nunca um re-export: uma fachada é a mesma aresta com outro nome.",
            passos.join("\n  ")
        );
    }
    // A tabela medida — a `--success-output` do nextest mostra-a, e é ela que o handoff cola.
    println!(
        "módulos {} · ficheiros {ficheiros} · referências {total} · através de re-exportação {via_reexport}",
        tops.len()
    );
    for ((a, b), (n, _)) in &arestas {
        let volta = arestas.get(&(b.clone(), a.clone())).map_or(0, |x| x.0);
        if volta > 0 {
            println!("  {a} → {b}: {n} (volta {volta})");
        }
    }
    let cresceram: Vec<String> = ARESTAS_TOLERADAS
        .iter()
        .filter_map(|(a, b, tecto, _)| {
            let n = arestas
                .get(&((*a).to_owned(), (*b).to_owned()))
                .map_or(0, |x| x.0);
            (n > *tecto).then(|| format!("{a} → {b}: {n} referências, tecto {tecto}"))
        })
        .collect();
    assert!(
        cresceram.is_empty(),
        "arestas TOLERADAS que CRESCERAM — a catraca só encolhe:\n  {}",
        cresceram.join("\n  ")
    );
}

#[test]
fn the_ratchet_only_describes_what_is_still_true() {
    let (tops, _, arestas, _) = grafo();
    let todas = toleradas();
    for (a, b, tecto, porque) in ARESTAS_TOLERADAS {
        assert!(!porque.trim().is_empty(), "`{a} → {b}` tolerada sem motivo");
        let n = arestas
            .get(&((*a).to_owned(), (*b).to_owned()))
            .map_or(0, |x| x.0);
        assert!(
            n > 0,
            "`{a} → {b}` está na catraca e já não é aresta — apague a linha"
        );
        assert!(
            n >= *tecto,
            "`{a} → {b}` desceu para {n} referências e a catraca diz {tecto} — desça o número"
        );
        let mut com_ela = todas.clone();
        com_ela.retain(|e| e != &((*a).to_owned(), (*b).to_owned()));
        assert!(
            um_ciclo(&tops, &arestas, &com_ela).is_some(),
            "`{a} → {b}` já não fecha ciclo nenhum — ela deixou de ser dívida: apague a linha"
        );
    }
}

/// ⚠️ **E o leitor sabe dizer «não» — e «sim».**
#[test]
fn the_reader_sees_what_it_claims_to_see() {
    // Uma aresta que desce, e que tem de estar lá: sem ela o leitor partiu-se e mede nada.
    let (_, _, arestas, _) = grafo();
    for (a, b) in [
        ("widget", "paint"),
        ("screens", "interaction"),
        ("toast", "progress"),
    ] {
        assert!(
            arestas.contains_key(&(a.to_owned(), b.to_owned())),
            "sentinela: o leitor não viu `{a} → {b}`"
        );
    }
    // Comentários e strings não contam; grupos e cadeias de `super` contam.
    let src = "// crate::screens::x\nlet s = \"crate::widget\";\nuse crate::{paint::A, zones::{B, C}};\nfn f() { super::super::motion::g(); }\n";
    let nomes: Vec<String> = referencias(src, &["a".to_owned(), "b".to_owned()])
        .into_iter()
        .map(|(_, n)| n)
        .collect();
    assert_eq!(
        nomes,
        vec!["paint".to_owned(), "zones".to_owned(), "motion".to_owned()]
    );
    // Dentro de um `mod tests { … }` inline, a raiz fica um `super` mais longe.
    let src = "mod tests {\n    use super::super::widget::W;\n    use super::paint::P;\n}\n";
    let nomes: Vec<String> = referencias(src, &["a".to_owned()])
        .into_iter()
        .map(|(_, n)| n)
        .collect();
    assert_eq!(nomes, vec!["widget".to_owned()]);
}
