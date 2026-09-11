//! Architecture gate — **o DRENO DE UM BRAÇO SÓ**: um valor que um controlo
//! escreve entra numa fila de intents, é drenado, e **nenhum consumidor cobre a
//! variante dele**. O gesto acontece, o intent nasce, e morre no fim do quadro.
//!
//! ## Porque este ficheiro existe (a metade que faltava)
//!
//! A caça de 2026-08-30 seguiu ~504 controlos até ao EFEITO e achou 34 mortos.
//! Nenhum instrumento do repo os via:
//!
//! - `architecture_panel_wiring_parity` mede **focalizabilidade** (pintado sem
//!   `InteractiveState` ⇒ morto sob o dedo). O cabeçalho dele **declara** que a
//!   outra metade — *"o evento é largado no `_ => false`"* — fica por fazer.
//! - Os `seam_*.rs` provam que um clique **chega à ferramenta**; nunca que a
//!   escrita da ferramenta chega a um consumidor.
//!
//! Este gate faz a terceira pergunta: **o valor que o controlo escreve chega a
//! quem o lê?** — na forma em que ela é decidível por leitura de fonte.
//!
//! ## O que ele mede, exactamente
//!
//! Uma **fila de intents** é `pub fn drain_*() -> Vec<T>`; `T` é o vocabulário
//! que um painel publica e outro código interpreta. Para cada `T`:
//!
//! - **PRODUZIDA** = a variante aparece como VALOR (`T::V { .. }` construído).
//! - **CONSUMIDA** = a variante aparece como PADRÃO (braço de `match`, `if let`,
//!   `while let`, `matches!`) — resolvendo aliases (`use ... as I;`) e `Self::`
//!   dentro do `impl T`.
//!
//! Uma variante **produzida e nunca consumida** é um controlo morto: o gesto
//! escreve, a fila enche, o dreno esvazia, e nada acontece. É EXACTAMENTE a
//! forma do defeito nomeado no `render_loop`
//! (`if let AuthoredIntent::Fired { key } = intent`), que mata **seis famílias
//! de widget** de uma vez — Tabs, SegmentedAdaptive, RadioGroup, Dropdown,
//! TextInput, NumberInput — sem que um único gate de registo o note.
//!
//! ⚠️ **A régua é a mesma nas duas direcções.** O gate nasce com CONTROLO
//! POSITIVO (a catraca abaixo tem de continuar a ser DETECTADA) e com controlo
//! NEGATIVO (as filas limpas nomeadas em `CLEAN_QUEUES` têm de continuar
//! limpas). Um instrumento cujo vermelho é garantido pela forma dos dados não
//! mede nada.
//!
//! ## ⛔ O que este gate NÃO vê (lista honesta)
//!
//! - **Enums que não são filas de `drain_*`** — um `PanelEvent`/`EditorAction`
//!   não entra aqui. A porta é o nome `drain_*`, medido: 9 filas, 183 variantes.
//! - **A variante consumida por um braço que não faz nada** (`V => {}`) lê como
//!   viva. Isto mede alcance, não efeito.
//! - **Consumo por reflexão / `serde` / macro que gera o `match`** — nenhuma
//!   fila do repo o faz hoje; se alguma passar a fazer, ela entra em
//!   `CLEAN_QUEUES` com o motivo.
//! - **Uma variante nunca produzida** não é reportada: um vocabulário maior que
//!   o gesto que o preenche é dívida de outra espécie (a irmã `*_reaches_a_consumer`).
//!
//! Dep-free (std only), como os outros gates de arquitectura.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// **Catraca de dívida conhecida — ela só ENCOLHE.** Cada linha é
/// `(tipo, variante, motivo)`. Uma entrada que já não é detectada é uma dívida
/// paga que ninguém apagou, e o gate cobra-a (§ *stale* no fim).
const UNCONSUMED_PENDING: &[(&str, &str, &str)] = &[
    // ═══ DÍVIDA PAGA em 2026-08-30 — as QUATRO entradas `AuthoredIntent` saíram ═══
    //
    // O dreno era um `if let AuthoredIntent::Fired { key } = intent` em
    // `shells/desktop/src/render_loop/mod.rs`, e matava **seis famílias de widget** de uma vez:
    // `Tabs` · `SegmentedAdaptive` · `RadioGroup` · `Dropdown` publicam `Choice`;
    // `TextInput`/`NumberInput` publicam `Text`. O chip acendia e nada mudava.
    //
    // Hoje é um `match` com **braço NOMEADO para cada variante**, em
    // `render_loop/authored_intents.rs` — e o `Choice` publica um sinal cujo NOME compõe a fileira
    // com a opção (`blend/multiply`). ⭐ A alternativa (dar carga ao `SignalOrigin::Control`) foi
    // medida e recusada por DUAS razões: o enum é `Copy` e uma `String` mata-o, e o contrato
    // declarado da crate diz *«o CONTRATO é o nome — um consumidor casa numa string e nunca
    // precisa perguntar a origem»*. `Contact` e `Animation` já recusam carga pela mesma lei.
    //
    // ⚠️ `Value`, `Flag` e `Text` ficam com braços **nomeados e vazios**, com o motivo escrito em
    // cada um: os dois primeiros têm outro fio declarado (o `WidgetStore`, que dirige a arte por
    // `vec_widget_drive`), e o `Text` cala-se porque um nome que o artista digita é um espaço de
    // nomes ilimitado — o valor já chega ao store; o que falta é CONSUMIDOR, não produtor.
    // ⇒ um braço nomeado-e-vazio lê como consumido por esta régua, que é um limite declarado no
    // cabeçalho deste ficheiro. O que impede a próxima variante de cair em silêncio é o `match`
    // exaustivo mais o gate `the_drain_names_every_authored_intent_variant`, no shell.
    // ⛔ A fila de SPAWN do `ph2d-script`: as quatro sao produzidas pelas ligacoes
    // Luau (`host.rs`) e `drain_spawns()` nao tem chamador nenhum fora dos testes
    // da propria crate. E' a mesma ausencia que o CLAUDE.md ja' nomeia — o
    // `ScriptHost` do desktop corre um script placeholder e NUNCA recebe
    // `provide_read`. Nao e' um controlo de UI: e' a superficie de um subsistema
    // que ainda nao tem consumidor. Sai da catraca quando o R1/`shells/game` abrir.
    (
        "SpawnCommand",
        "SpawnEmpty",
        "ph2d-script: drain_spawns() sem chamador (ScriptHost e' placeholder)",
    ),
    (
        "SpawnCommand",
        "SpawnNamed",
        "ph2d-script: drain_spawns() sem chamador (ScriptHost e' placeholder)",
    ),
    (
        "SpawnCommand",
        "Despawn",
        "ph2d-script: drain_spawns() sem chamador (ScriptHost e' placeholder)",
    ),
    (
        "SpawnCommand",
        "AttachScript",
        "ph2d-script: drain_spawns() sem chamador (ScriptHost e' placeholder)",
    ),
];

/// **Controlo NEGATIVO** — filas verificadas limpas em 2026-08-30. Se uma delas
/// aparecer com variante morta, o gate reprova nomeando-a; se uma sumir da
/// varredura, a sonda deixou de a ver e isso também reprova.
const CLEAN_QUEUES: &[&str] = &[
    "FlipStripIntent",
    "ModelIntent",
    "GraphIntent",
    "MotionParamIntent",
    "PhysicsIntent",
    "Sculpt3dIntent",
    "TimelineIntent",
    "TokensIntent",
];

/// A **metade justa**: a sonda tem de VER o que já existe. Baseline medido em
/// 2026-08-30 (9 filas, 183 variantes). Números conservadores: eles descem
/// só se alguém apagar uma fila, e aí o gate quer ser lido.
const MIN_QUEUES: usize = 9;
const MIN_VARIANTS: usize = 150;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn is_test_path(p: &Path) -> bool {
    let s = p.to_string_lossy().replace('\\', "/");
    let f = p
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    s.contains("/tests/")
        || s.contains("/benches/")
        || s.contains("/examples/")
        || f == "tests.rs"
        || f.ends_with("_tests.rs")
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if p.is_dir() {
            if name != "target" && name != ".git" {
                collect_rs(&p, out);
            }
        } else if name.ends_with(".rs") && !is_test_path(&p) {
            out.push(p);
        }
    }
}

/// Índice do fecho do delimitador aberto em `open` (que tem de ser `(`/`[`/`{`).
fn close_of(t: &[u8], open: usize) -> usize {
    let (o, c) = match t[open] {
        b'(' => (b'(', b')'),
        b'[' => (b'[', b']'),
        _ => (b'{', b'}'),
    };
    let mut depth = 0i32;
    let mut i = open;
    while i < t.len() {
        let ch = t[i];
        if ch == b'"' {
            i += 1;
            while i < t.len() {
                if t[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if t[i] == b'"' {
                    break;
                }
                i += 1;
            }
        } else if ch == o {
            depth += 1;
        } else if ch == c {
            depth -= 1;
            if depth == 0 {
                return i;
            }
        }
        i += 1;
    }
    t.len()
}

/// As variantes de topo de um `pub enum NAME { .. }`.
fn enum_variants(src: &str) -> BTreeMap<String, Vec<String>> {
    let b = src.as_bytes();
    let mut out = BTreeMap::new();
    let mut from = 0usize;
    while let Some(rel) = src[from..].find("pub enum ") {
        let at = from + rel;
        from = at + 9;
        let rest = &src[from..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }
        let Some(brace_rel) = rest.find('{') else {
            continue;
        };
        // Só um `{` colado ao nome (com genéricos/where pelo meio) conta.
        if rest[..brace_rel].contains(';') || rest[..brace_rel].contains("pub enum") {
            continue;
        }
        let open = from + brace_rel;
        let end = close_of(b, open);
        let body = &src[open + 1..end.min(src.len())];
        let mut vs = Vec::new();
        for line in body.lines() {
            let l = line.trim_start();
            let head: String = l
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if head.is_empty() || !head.starts_with(|c: char| c.is_ascii_uppercase()) {
                continue;
            }
            let after = l[head.len()..].trim_start();
            if after.starts_with('{') || after.starts_with('(') || after.starts_with(',') {
                vs.push(head);
            }
        }
        out.insert(name, vs);
    }
    out
}

/// Todos os prefixos por que `ty` pode ser nomeado dentro de `src`.
fn prefixes_for(src: &str, ty: &str) -> Vec<String> {
    let mut v = vec![ty.to_string()];
    let needle = format!("{ty} as ");
    let mut from = 0usize;
    while let Some(rel) = src[from..].find(&needle) {
        let at = from + rel + needle.len();
        from = at;
        let alias: String = src[at..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        // Só conta se a linha for um `use` (um `as` de cast não renomeia nada).
        let line_start = src[..at].rfind('\n').map_or(0, |i| i + 1);
        if !alias.is_empty() && src[line_start..at].contains("use ") {
            v.push(alias);
        }
    }
    if src.contains(&format!("impl {ty} ")) || src.contains(&format!("impl {ty}\n")) {
        v.push("Self".to_string());
    }
    v
}

/// Recua `i` até uma fronteira de carácter (o fonte tem UTF-8 nos comentários).
fn floor_boundary(src: &str, mut i: usize) -> usize {
    while i > 0 && !src.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// `true` se a ocorrência de variante em `[start, end)` for um PADRÃO (consumo)
/// e não um VALOR (produção).
fn is_pattern(src: &str, start: usize, end: usize) -> bool {
    let b = src.as_bytes();
    // Depois do nome: salta a carga (`{..}` / `(..)`) e olha o que vem a seguir.
    let mut j = end;
    while j < b.len() && (b[j] as char).is_whitespace() {
        j += 1;
    }
    if j < b.len() && (b[j] == b'{' || b[j] == b'(') {
        j = close_of(b, j) + 1;
    }
    while j < b.len() && (b[j] as char).is_whitespace() {
        j += 1;
    }
    let j = j.min(src.len());
    let tail_end = floor_boundary(src, (j + 4).min(src.len()));
    let tail = src.get(j..tail_end).unwrap_or("");
    if tail.starts_with("=>") || tail.starts_with('|') || tail.starts_with("if ") {
        return true;
    }
    // Antes do nome: `if let` / `while let` / `matches!(`.
    let from = floor_boundary(src, start.saturating_sub(80));
    let before = src.get(from..start).unwrap_or("");
    let trimmed = before.trim_end();
    trimmed.ends_with("if let")
        || trimmed.ends_with("while let")
        || trimmed.ends_with("matches!(")
        || trimmed.ends_with("Some(")
        || before.contains("matches!(")
}

struct Queue {
    func: String,
    ty: String,
    decl: String,
}

fn find_queues(files: &[(PathBuf, String)], root: &Path) -> Vec<Queue> {
    let mut out = Vec::new();
    for (p, src) in files {
        let mut from = 0usize;
        while let Some(rel) = src[from..].find("pub fn drain_") {
            let at = from + rel + "pub fn ".len();
            from = at;
            let func: String = src[at..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let rest = &src[at + func.len()..];
            let Some(sig) = rest.get(..80) else { continue };
            let Some(arrow) = sig.find("-> Vec<") else {
                continue;
            };
            if sig[..arrow].contains('{') {
                continue;
            }
            let ty: String = sig[arrow + "-> Vec<".len()..]
                .trim_start()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if ty.is_empty() {
                continue;
            }
            let rel_path = p.strip_prefix(root).unwrap_or(p).to_string_lossy();
            out.push(Queue {
                func,
                ty,
                decl: rel_path.to_string(),
            });
        }
    }
    out
}

#[test]
fn the_intent_drain_reaches_every_variant() {
    let root = repo_root();
    let mut paths = Vec::new();
    collect_rs(&root.join("crates"), &mut paths);
    collect_rs(&root.join("shells"), &mut paths);
    paths.sort();
    let files: Vec<(PathBuf, String)> = paths
        .into_iter()
        .filter_map(|p| std::fs::read_to_string(&p).ok().map(|s| (p, s)))
        .collect();

    let mut all_variants: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (_, src) in &files {
        for (k, v) in enum_variants(src) {
            all_variants.entry(k).or_insert(v);
        }
    }

    let queues = find_queues(&files, &root);
    let mut seen_types: BTreeSet<String> = BTreeSet::new();
    let mut variant_total = 0usize;
    let mut offenders: Vec<String> = Vec::new();
    let mut detected: BTreeSet<(String, String)> = BTreeSet::new();

    for q in &queues {
        // Uma fila cujo payload é STRUCT (e não enum) não tem variantes para
        // cobrir — fica fora, sem ruído.
        let Some(vs) = all_variants.get(&q.ty) else {
            continue;
        };
        if !seen_types.insert(q.ty.clone()) {
            continue;
        }
        variant_total += vs.len();
        let known: BTreeSet<&str> = vs.iter().map(String::as_str).collect();
        let mut produced: BTreeMap<&str, String> = BTreeMap::new();
        let mut consumed: BTreeSet<&str> = BTreeSet::new();

        for (p, src) in &files {
            for pfx in prefixes_for(src, &q.ty) {
                let needle = format!("{pfx}::");
                let mut from = 0usize;
                while let Some(rel) = src[from..].find(&needle) {
                    let at = from + rel;
                    from = at + needle.len();
                    // Fronteira de palavra: o prefixo pode vir qualificado
                    // (`ph2d_panel_x::AuthoredIntent::Fired`), então recua-se
                    // sobre o caminho inteiro antes de decidir.
                    let mut path_start = at;
                    while path_start > 0 {
                        let prev = src.as_bytes()[path_start - 1];
                        if prev.is_ascii_alphanumeric() || prev == b'_' {
                            path_start -= 1;
                        } else if prev == b':'
                            && path_start >= 2
                            && src.as_bytes()[path_start - 2] == b':'
                        {
                            path_start -= 2;
                        } else {
                            break;
                        }
                    }
                    if path_start > 0 {
                        let prev = src.as_bytes()[path_start - 1];
                        if prev.is_ascii_alphanumeric() || prev == b'_' {
                            continue;
                        }
                    }
                    let vstart = at + needle.len();
                    let v: String = src[vstart..]
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    let Some(&name) = known.get(v.as_str()) else {
                        continue;
                    };
                    if is_pattern(src, path_start, vstart + v.len()) {
                        consumed.insert(name);
                    } else {
                        produced.entry(name).or_insert_with(|| {
                            let line = src[..at].matches('\n').count() + 1;
                            format!(
                                "{}:{line}",
                                p.strip_prefix(&root).unwrap_or(p).to_string_lossy()
                            )
                        });
                    }
                }
            }
        }

        for v in vs {
            if produced.contains_key(v.as_str()) && !consumed.contains(v.as_str()) {
                detected.insert((q.ty.clone(), v.clone()));
                if UNCONSUMED_PENDING
                    .iter()
                    .any(|(t, var, _)| *t == q.ty && var == v)
                {
                    continue;
                }
                let at = produced.get(v.as_str()).map_or("?", String::as_str);
                offenders.push(format!(
                    "{}::{v} — produzida em {at}, drenada por {}() em {}, e NENHUM braco a cobre",
                    q.ty, q.func, q.decl
                ));
            }
        }
    }

    // A metade justa: sem ela um corpus vazio passa sempre.
    assert!(
        seen_types.len() >= MIN_QUEUES,
        "a sonda so' viu {} fila(s) de intents (baseline 2026-08-30: {MIN_QUEUES}). \
         Ou uma fila foi apagada, ou a deteccao de `pub fn drain_*() -> Vec<T>` partiu — \
         num corpus vazio este gate seria verde para sempre.",
        seen_types.len()
    );
    assert!(
        variant_total >= MIN_VARIANTS,
        "a sonda so' viu {variant_total} variante(s) (baseline 2026-08-30: {MIN_VARIANTS}). \
         A leitura do corpo do `enum` partiu."
    );

    offenders.sort();
    assert!(
        offenders.is_empty(),
        "variantes de intent PRODUZIDAS por um gesto e consumidas por NINGUEM — o controlo \
         escreve, a fila enche, o dreno esvazia, e nada acontece:\n  {}\n\n\
         cura: um braco no consumidor (o dreno), ou tirar a variante do vocabulario. \
         Um `if let T::X = intent` num enum de N variantes mata as outras N-1 em silencio.",
        offenders.join("\n  ")
    );

    // **Controlo POSITIVO / catraca que só encolhe.** Cada linha da catraca tem
    // de continuar a ser DETECTADA: se deixou de ser, ou a dívida foi paga (e a
    // linha tem de descer) ou a sonda ficou cega (e o verde é falso).
    let stale: Vec<String> = UNCONSUMED_PENDING
        .iter()
        .filter(|(t, v, _)| !detected.contains(&((*t).to_string(), (*v).to_string())))
        .map(|(t, v, why)| format!("{t}::{v} ({why})"))
        .collect();
    assert!(
        stale.is_empty(),
        "estas linhas do UNCONSUMED_PENDING ja' nao sao detectadas. Ou a divida foi paga — e a \
         catraca tem de DESCER, apagando a linha — ou a sonda deixou de ver a familia inteira, \
         e entao todo o verde deste gate e' falso:\n  {}",
        stale.join("\n  ")
    );

    // **Controlo NEGATIVO** — as filas verificadas limpas continuam limpas E
    // continuam a ser vistas.
    let missing: Vec<&str> = CLEAN_QUEUES
        .iter()
        .filter(|t| !seen_types.contains(**t))
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "filas que a sonda via em 2026-08-30 e ja' nao ve — ela pode ter ficado cega para elas \
         sem que nada reprove:\n  {}",
        missing.join("\n  ")
    );
    let regressed: Vec<String> = detected
        .iter()
        .filter(|(t, _)| CLEAN_QUEUES.contains(&t.as_str()))
        .map(|(t, v)| format!("{t}::{v}"))
        .collect();
    assert!(
        regressed.is_empty(),
        "uma fila que estava LIMPA passou a ter variante morta:\n  {}",
        regressed.join("\n  ")
    );
}
