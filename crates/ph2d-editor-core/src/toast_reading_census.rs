//! ⭐⭐ **O censo dos avisos, por CONSUMIDOR** (doc 132 §5): toda frase da tabela inglesa que chega
//! a um `Toast::{info,success,warning,error,new}` — lida do FONTE dos chamadores, com as
//! interpolações cheias pelo argumento mais longo plausível — medida com a fonte real ao orçamento
//! do balão, em todo estilo de texto.
//!
//! Correr o relatório: `cargo test -p ph2d-editor-core --lib toast_reading_census -- --nocapture`.

use super::*;
use ph2d_i18n::{Idioma, tr_em};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Um sítio que empurra um aviso, e as chaves que o alimentam.
struct Sitio {
    familia: &'static str,
    onde: String,
    chaves: Vec<String>,
    /// As chaves do corpo da função do sítio — de onde sai o texto de um marcador que guarda
    /// uma ORAÇÃO (a `{cura}` do Motion).
    escopo: Vec<String>,
    /// Os literais de prosa do argumento que NÃO são chaves (as cenas de smoke falam inglês cru).
    crus: Vec<String>,
    /// As chaves vieram do corpo da função, não do argumento (o aviso é montado antes).
    indirecto: bool,
}

const AGULHAS: [&str; 5] = [
    "Toast::info(",
    "Toast::success(",
    "Toast::warning(",
    "Toast::error(",
    "Toast::new(",
];

fn raiz() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn familia(rel: &str) -> &'static str {
    for (pre, f) in [
        ("shells/", "shell"),
        ("crates/ph2d-app-motion/", "Motion"),
        ("crates/ph2d-app-painter/", "Painter"),
        ("crates/ph2d-app-components/", "Components"),
        ("crates/ph2d-app-vec/", "Vector"),
        ("crates/ph2d-app-board/", "Board"),
        ("crates/ph2d-app-flip/", "Flip"),
        ("crates/ph2d-app-physics/", "Fisica"),
        ("crates/ph2d-panel-timeline/", "Timeline"),
        ("crates/ph2d-editor-core/", "editor-core"),
    ] {
        if rel.starts_with(pre) {
            return f;
        }
    }
    "outro"
}

fn fontes(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        panic!("o censo não leu {}", dir.display());
    };
    for e in rd.flatten() {
        let p = e.path();
        let nome = e.file_name().to_string_lossy().to_string();
        if p.is_dir() {
            if !["target", "tests", "benches", "examples", "node_modules"].contains(&nome.as_str())
            {
                fontes(&p, out);
            }
        } else if nome.ends_with(".rs")
            && !["test", "probe", "diag", "sonda", "census"]
                .iter()
                .any(|t| nome.contains(t))
        {
            out.push(p);
        }
    }
}

/// O ficheiro sem os módulos de teste em linha.
fn sem_testes(s: &str) -> &str {
    let mut corte = s.len();
    let mut de = 0;
    while let Some(i) = s[de..].find("#[cfg(test)]") {
        let i = de + i;
        let resto = s[i + 12..].trim_start();
        let linha = resto.lines().next().unwrap_or("");
        if (linha.starts_with("mod ") || linha.contains(" mod ")) && linha.contains('{') {
            corte = i;
            break;
        }
        de = i + 12;
    }
    &s[..corte]
}

/// O argumento entre os parênteses que abrem em `abre` (o índice do `(`).
fn argumento(s: &str, abre: usize) -> &str {
    let b = s.as_bytes();
    let (mut fundo, mut i) = (0i32, abre);
    while i < b.len() {
        match b[i] {
            b'"' => {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'(' => fundo += 1,
            b')' => {
                fundo -= 1;
                if fundo == 0 {
                    return &s[abre + 1..i];
                }
            }
            _ => {}
        }
        i += 1;
    }
    &s[abre + 1..]
}

/// Os literais de texto de um pedaço de fonte.
fn literais(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'"' {
            let mut lit: Vec<u8> = Vec::new();
            i += 1;
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\\' && i + 1 < b.len() {
                    i += 1;
                    match b[i] {
                        // A continuação de linha come a quebra e o recuo seguinte.
                        b'\n' => {
                            while i + 1 < b.len() && b[i + 1].is_ascii_whitespace() {
                                i += 1;
                            }
                        }
                        b'n' => lit.push(b'\n'),
                        b'u' => {
                            let fim = s[i..].find('}').map_or(i, |f| i + f);
                            let hex = s.get(i + 2..fim).unwrap_or("");
                            if let Some(c) =
                                u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
                            {
                                lit.extend(c.to_string().bytes());
                            }
                            i = fim;
                        }
                        c => lit.push(c),
                    }
                } else {
                    lit.push(b[i]);
                }
                i += 1;
            }
            out.push(String::from_utf8_lossy(&lit).into_owned());
        }
        i += 1;
    }
    out
}

fn e_chave(l: &str) -> bool {
    !l.is_empty() && !l.contains(' ') && tr_em(Idioma::Ingles, l) != l
}

/// Os nomes chamados num pedaço de fonte (`nome(` ou `.nome(`), sem os de sempre.
fn chamadas(s: &str) -> Vec<String> {
    const COMUNS: &[&str] = &[
        "tr_with",
        "to_string",
        "format",
        "push",
        "warning",
        "success",
        "error",
        "clone",
        "into",
        "as_str",
        "unwrap_or",
        "unwrap_or_default",
        "unwrap_or_else",
        "map_or",
        "map_or_else",
        "is_some",
        "is_none",
        "as_mut",
        "as_ref",
        "iter",
        "collect",
        "Toast",
        "Some",
    ];
    let b = s.as_bytes();
    let mut out = Vec::new();
    for (i, _) in s.match_indices('(') {
        let mut j = i;
        while j > 0 && (b[j - 1].is_ascii_alphanumeric() || b[j - 1] == b'_') {
            j -= 1;
        }
        let nome = &s[j..i];
        if nome.len() >= 5 && !COMUNS.contains(&nome) && !out.iter().any(|o| o == nome) {
            out.push(nome.to_string());
        }
    }
    out
}

/// O nome da função que começa em `s` (`fn nome…`).
fn nome_da_fn(s: &str) -> Option<&str> {
    let r = s.strip_prefix("fn ")?;
    let fim = r.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))?;
    (fim > 0).then(|| &r[..fim])
}

/// Os identificadores em MAIÚSCULAS (constantes) de um pedaço de fonte.
fn maiusculas(s: &str) -> Vec<String> {
    s.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| {
            w.len() >= 3
                && w.chars()
                    .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
                && w.chars().any(|c| c.is_ascii_uppercase())
        })
        .map(str::to_string)
        .collect()
}

/// A definição (até ao `;`) de cada `const NOME`, por nome.
fn definicoes_de_consts(textos: &[(PathBuf, String)]) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (_, todo) in textos {
        let s = sem_testes(todo);
        for (i, _) in s.match_indices("const ") {
            let r = &s[i + 6..];
            let fim = r
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(r.len());
            let Some(pv) = r.find(';') else { continue };
            out.entry(r[..fim].to_string())
                .or_default()
                .push(r[..pv].to_string());
        }
    }
    out
}

/// O corpo (com chavetas equilibradas) de cada `fn nome` dos fontes, por nome.
fn corpos_das_funcoes(textos: &[(PathBuf, String)]) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (_, todo) in textos {
        let s = sem_testes(todo);
        for (i, _) in s.match_indices("fn ") {
            let resto = &s[i + 3..];
            let fim = resto
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(resto.len());
            let nome = &resto[..fim];
            let Some(a) = resto.find('{') else { continue };
            if resto[..a].contains(';') {
                continue;
            }
            let corpo = chavetas(&resto[a..]);
            out.entry(nome.to_string())
                .or_default()
                .push(corpo.to_string());
        }
    }
    out
}

fn chavetas(s: &str) -> &str {
    let b = s.as_bytes();
    let (mut fundo, mut i) = (0i32, 0);
    while i < b.len() {
        match b[i] {
            b'"' => {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'{' => fundo += 1,
            b'}' => {
                fundo -= 1;
                if fundo == 0 {
                    return &s[..=i];
                }
            }
            _ => {}
        }
        i += 1;
    }
    s
}

fn sitios() -> Vec<Sitio> {
    let raiz = raiz();
    let mut fs = Vec::new();
    fontes(&raiz.join("crates"), &mut fs);
    fontes(&raiz.join("shells"), &mut fs);
    let textos: Vec<(PathBuf, String)> = fs
        .into_iter()
        .filter_map(|p| Some((p.clone(), std::fs::read_to_string(&p).ok()?)))
        .collect();
    let corpos = corpos_das_funcoes(&textos);
    let consts = definicoes_de_consts(&textos);
    let mut out = Vec::new();
    for (p, todo) in &textos {
        let s = sem_testes(todo);
        let rel = p
            .strip_prefix(&raiz)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        for ag in AGULHAS {
            let mut de = 0;
            while let Some(i) = s[de..].find(ag) {
                let i = de + i;
                de = i + ag.len();
                let arg = argumento(s, i + ag.len() - 1);
                let mut chaves: Vec<String> =
                    literais(arg).into_iter().filter(|l| e_chave(l)).collect();
                let mut indirecto = false;
                let inicio = s[..i].rfind("fn ").unwrap_or(0);
                let corpo = &s[inicio..i + ag.len() + arg.len()];
                let escopo: Vec<String> =
                    literais(corpo).into_iter().filter(|l| e_chave(l)).collect();
                if chaves.is_empty() {
                    chaves.clone_from(&escopo);
                    if chaves.is_empty() {
                        // O texto vem de uma função: as chamadas do argumento e as que um
                        // `tr(..)` do corpo faz (`tr(r.chave())`).
                        let mut nomes = chamadas(arg);
                        for t in ["tr(", "tr_with("] {
                            for (j, _) in corpo.match_indices(t) {
                                nomes.extend(chamadas(argumento(corpo, j + t.len() - 1)));
                            }
                        }
                        let seguidos: Vec<&String> = nomes
                            .iter()
                            .flat_map(|f| corpos.get(f.as_str()).into_iter().flatten())
                            .collect();
                        chaves = seguidos
                            .iter()
                            .flat_map(|c| literais(c))
                            .filter(|l| e_chave(l))
                            .collect();
                        // As constantes `TextKey` nomeadas no argumento, no corpo e nas
                        // funções seguidas (`AVISO.tr()`, `REFUSAL.key()`).
                        let mut fontes_de_consts = vec![arg, corpo];
                        fontes_de_consts.extend(seguidos.iter().map(|c| c.as_str()));
                        for f in fontes_de_consts {
                            for c in maiusculas(f) {
                                for def in consts.get(&c).into_iter().flatten() {
                                    chaves.extend(literais(def).into_iter().filter(|l| e_chave(l)));
                                }
                            }
                        }
                        // Uma função-porta (`fn failed(key, e) -> Toast`): o texto vem de
                        // quem a chama.
                        if chaves.is_empty()
                            && let Some(nome) = nome_da_fn(&s[inicio..])
                        {
                            for (_, outro) in &textos {
                                let o = sem_testes(outro);
                                for (j, _) in o.match_indices(&format!("{nome}(")) {
                                    let antes = o[..j].chars().next_back();
                                    if antes.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
                                        || o[..j].ends_with("fn ")
                                    {
                                        continue;
                                    }
                                    let a = argumento(o, j + nome.len());
                                    chaves.extend(literais(a).into_iter().filter(|l| e_chave(l)));
                                }
                            }
                        }
                    }
                    indirecto = true;
                }
                let linha = s[..i].lines().count();
                out.push(Sitio {
                    familia: familia(&rel),
                    onde: format!("{rel}:{linha}"),
                    crus: literais(arg)
                        .into_iter()
                        .filter(|l| l.contains(' ') && !e_chave(l))
                        .collect(),
                    chaves,
                    escopo,
                    indirecto,
                });
            }
        }
    }
    out
}

/// O valor mais longo plausível de um marcador `{nome}`, pela CLASSE do nome (contagem,
/// caminho, erro, lista; o resto é um nome de objecto).
fn exemplo(nome: &str) -> String {
    const NUMERO: &[&str] = &[
        "n",
        "v",
        "r",
        "a",
        "b",
        "s",
        "k",
        "t",
        "l",
        "d",
        "f",
        "c",
        "i",
        "x",
        "y",
        "z",
        "w",
        "h",
        "count",
        "max",
        "min",
        "size",
        "max_dim",
        "now",
        "dist",
        "ms",
        "px",
        "broken",
        "lado",
        "skipped",
        "made",
        "made2",
        "overrides_kept",
        "dropped",
        "frames",
        "out_w",
        "out_h",
        "inside",
        "id",
        "ram",
        "pct",
        "weight",
        "total",
        "done",
        "rate",
        "entities",
        "limite",
        "mb",
        "nodes",
        "kept",
        "quantos",
        "bodies",
        "tracks",
        "pieces",
        "pieces_back",
        "changed",
        "islands",
        "spawned",
        "applied",
        "downgraded",
        "n_sources",
        "regions",
        "size_px",
        "objects",
        "copies",
        "ab",
        "bb",
        "force_0",
        "force",
        "beyond",
        "db",
        "lufs",
        "lo",
        "hi",
        "col",
        "row",
        "cells",
        "neighbors",
        "fps",
        "stall",
        "stamps",
        "colors",
        "cell",
        "ow",
        "oh",
        "iw",
        "ih",
        "ox",
        "oy",
        "span",
        "join_count",
        "rig_parts",
        "pw",
        "ph",
        "repeat",
        "achadas",
        "have",
        "radius",
        "ratio",
        "gw",
        "gh",
        "fw",
        "fh",
        "cost",
        "ambiguous",
        "stable_id",
        "groups",
        "joints",
        "refused",
        "px_per_m_0",
        "size_3",
        "size2_3",
        "sheet_w",
        "sheet_h",
        "per_frame_ms",
        "spawn_failed",
        "size2",
        "max_texture_dimension_2d",
        "shapes",
        "animations",
        "layers",
        "ver",
        "bytes",
        "moved",
        "image_size",
        "image_size2",
        "formas",
        "extra",
        "aproximadas",
        "start",
        "end",
        "ev",
        "width",
        "height",
        "bounds",
        "anchors_max",
        "anchor_name_max_bytes",
        "anim_tags_max",
        "anim_name_max_bytes",
        "texture_id",
        "nivel",
        "level",
        "value",
        "members",
        "membros",
        "components",
        "tags",
        "left_out",
        "targets",
    ];
    const ERRO: &[&str] = &[
        "e", "e_", "err", "error", "msg", "because", "raw", "status", "say",
    ];
    const CAMINHO: &[&str] = &[
        "path",
        "file",
        "stem",
        "png",
        "json",
        "disk",
        "dir",
        "ext",
        "extension",
        "image_filename",
    ];
    const LISTA: &[&str] = &["names", "list", "labels", "soltas", "queixas", "follows"];
    let n = nome.to_ascii_lowercase();
    if ERRO.contains(&n.as_str()) {
        "No such file or directory (os error 2)".into()
    } else if CAMINHO.contains(&n.as_str()) {
        "~/Projects/my_game/assets/hero_walk_cycle.png".into()
    } else if LISTA.contains(&n.as_str()) {
        "Hero, Enemy 02, Coin Pickup".into()
    } else if NUMERO.contains(&n.as_str()) {
        "4096".into()
    } else {
        "Character Controller 02".into()
    }
}

/// A frase de uma chave em inglês, cada marcador cheio. Um `{cura}` (a cura do Motion) leva a
/// frase mais longa das OUTRAS chaves do mesmo sítio — é dali que o chamador a monta.
fn frase(chave: &str, irmas: &[String]) -> String {
    let t = tr_em(Idioma::Ingles, chave);
    let mut out = String::new();
    let mut resto = t;
    while let Some(a) = resto.find('{') {
        out.push_str(&resto[..a]);
        let depois = &resto[a + 1..];
        let Some(f) = depois.find('}') else {
            out.push_str(&resto[a..]);
            resto = "";
            break;
        };
        let nome = &depois[..f];
        if nome == "cura" {
            let maior = irmas
                .iter()
                .filter(|k| k.as_str() != chave)
                .map(|k| frase(k, &[]))
                .filter(|f| f.starts_with(' '))
                .max_by_key(String::len)
                .unwrap_or_default();
            out.push_str(&maior);
        } else {
            out.push_str(&exemplo(nome));
        }
        resto = &depois[f + 1..];
    }
    out.push_str(resto);
    out
}

/// Uma frase do censo: a família, a chave, a frase cheia e o sítio que a empurra.
type Mensagem = (&'static str, String, String, String);

/// Toda frase de aviso da tabela, sem repetir a chave.
fn mensagens() -> (Vec<Mensagem>, Vec<String>, usize) {
    let todos = sitios();
    let n_sitios = todos.len();
    let mut vistas = BTreeMap::new();
    let mut sem_chave = Vec::new();
    for s in &todos {
        if s.chaves.is_empty() {
            if s.crus.is_empty() {
                sem_chave.push(s.onde.clone());
            }
            for (i, c) in s.crus.iter().enumerate() {
                vistas.entry(format!("(cru #{i}) {}", s.onde)).or_insert((
                    s.familia,
                    c.replace("{}", "4096"),
                    s.onde.clone(),
                ));
            }
            continue;
        }
        for k in &s.chaves {
            let f = frase(k, &s.escopo);
            // Um sítio indirecto pode ver chaves que não são do aviso: só as que têm prosa.
            if s.indirecto && !f.contains(' ') {
                continue;
            }
            vistas
                .entry(k.clone())
                .or_insert((s.familia, f, s.onde.clone()));
        }
    }
    let msgs = vistas
        .into_iter()
        .map(|(k, (fam, f, onde))| (fam, k, f, onde))
        .collect();
    (msgs, sem_chave, n_sitios)
}

/// Por frase, o pior sobre todo estilo de texto: as linhas que pede, e se o balão a CORTA (o
/// que o pintor de facto pinta, [`shown`], não é a frase, ou uma linha passa da largura).
fn pior_estilo(msgs: &[&str]) -> Vec<(usize, bool)> {
    let mut pior = vec![(0, false); msgs.len()];
    let _ = crate::text_elide::em_todo_estilo(|ts| {
        for (p, m) in pior.iter_mut().zip(msgs) {
            let (n, larga) = lines_needed(ts, m);
            let cortada = larga || shown(ts, m).text != *m;
            *p = (p.0.max(n), p.1 || cortada);
        }
        Vec::new()
    });
    pior
}

/// ⭐ **O relatório** (o antes e o depois, por família) e o gate (i): toda mensagem cabe inteira.
#[test]
fn toda_mensagem_de_aviso_da_tabela_cabe_inteira() {
    let (msgs, sem_chave, n_sitios) = mensagens();
    let mut ts = TextSystem::new();
    let mut sem_sistema = TextSystem::without_system_fonts();
    let antes = crate::paint::label_budget(text_budget_px());
    let mut por_familia: BTreeMap<&str, (usize, usize, usize, usize, [usize; 8])> = BTreeMap::new();
    let mut linhas = Vec::new();
    let mut maior_delta = 0.0f32;
    let mut falhas = Vec::new();
    let piores = pior_estilo(&msgs.iter().map(|m| m.2.as_str()).collect::<Vec<_>>());
    for ((fam, chave, msg, onde), &(n, cortada)) in msgs.iter().zip(&piores) {
        let w = ts.prefix_width_weighted(msg, font_px(), FontWeight::MEDIUM);
        let w2 = sem_sistema.prefix_width_weighted(msg, font_px(), FontWeight::MEDIUM);
        maior_delta = maior_delta.max((w - w2).abs());
        let cortava = w > antes;
        let (n_hoje, _) = lines_needed(&mut ts, msg);
        let e = por_familia.entry(fam).or_insert((0, 0, 0, 0, [0; 8]));
        e.0 += 1;
        e.1 += usize::from(cortava);
        e.2 += usize::from(cortada);
        e.3 = e.3.max(n);
        e.4[n.min(7)] += 1;
        linhas.push((w, *fam, chave.clone(), msg.clone(), n_hoje, n, onde.clone()));
        if cortada {
            falhas.push(format!(
                "{fam} `{chave}` pede {n} linhas e o balão corta: {msg:?}"
            ));
        }
    }
    eprintln!(
        "\n=== CENSO DOS AVISOS · {} sítios, {} frases (tabela + cruas de smoke), {} sítios sem chave legível ===",
        n_sitios,
        msgs.len(),
        sem_chave.len()
    );
    eprintln!(
        "orçamento: hoje {antes:.1} px (porta {:.1} - respiro), depois {:.1} px · letra {:.1} px · \
         TextSystem::new vs without_system_fonts: Δ máx {maior_delta:.3} px",
        text_budget_px(),
        text_budget_px(),
        font_px()
    );
    eprintln!(
        "família     | frases | cortavam (1 linha elidida) | cortam (quebra, máx {MAX_LINES}) | linhas máx (todo estilo) | 1/2/3/4/5/6/7+"
    );
    for (f, (n, c, c2, m, d)) in &por_familia {
        eprintln!(
            "{f:<11} | {n:>6} | {c:>26} | {c2:>24} | {m:>24} | {}/{}/{}/{}/{}/{}/{}",
            d[1], d[2], d[3], d[4], d[5], d[6], d[7]
        );
    }
    linhas.sort_by(|a, b| b.0.total_cmp(&a.0));
    eprintln!(
        "--- as 15 mais longas (largura numa linha · linhas no estilo de fábrica · pior estilo) ---"
    );
    for (w, f, k, m, n0, n, onde) in linhas.iter().take(15) {
        eprintln!("{w:>7.1} px · {n0} · {n} · {f} `{k}` ({onde}): {m:?}");
    }
    eprintln!(
        "--- sítios sem chave legível (texto montado fora da função; {} frases cruas de smoke medidas) ---",
        msgs.iter().filter(|m| m.1.starts_with("(cru")).count()
    );
    for s in &sem_chave {
        eprintln!("  {s}");
    }
    // Pisos de população: uma varredura que não acha nada passa por vácuo.
    assert!(n_sitios >= 300, "o censo achou só {n_sitios} sítios");
    assert!(msgs.len() >= 150, "o censo achou só {} frases", msgs.len());
    assert!(
        falhas.is_empty(),
        "avisos que não se leem inteiros:\n{}",
        falhas.join("\n")
    );
}

/// Controlo do gate: uma frase de `MAX_LINES + 1` linhas É acusada.
#[test]
fn o_censo_acusa_uma_frase_longa_demais() {
    let mut ts = TextSystem::without_system_fonts();
    let frase = "word ".repeat(400);
    let (n, _) = lines_needed(&mut ts, &frase);
    assert!(n > MAX_LINES, "a régua não viu {n} linhas");
}
