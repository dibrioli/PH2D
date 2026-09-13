//! **O dreno do barramento partido em sub-drenos guarda as garantias que o `match` único dava de graça**
//! (`line/render-bodies`, 2026-09-13).
//!
//! Enquanto o dreno era UM `match` dentro de `for action in hero.bus.drain()`, três coisas eram impossíveis por
//! construção, e hoje só não acontecem (auditoria do fecho da linha):
//! 1. **escrever na fila a meio do dreno** — o `Drain` emprestava-a inteira; hoje ela sai por `mem::take` e volta no fim,
//!    e um `hero.bus.push(…)` num sub-dreno compilaria e SUMIRIA na devolução;
//! 2. **um braço que nunca casa** — o `rustc` avisava (`unreachable_patterns`); hoje os braços vivem em onze funções
//!    encadeadas, e um segundo braço para o mesmo pedido num sub-dreno ANTERIOR roubaria o pedido sem aviso;
//! 3. **um `?` no corpo de um braço** — o laço não tinha para onde o devolver; hoje o `None` de um sub-dreno lê-se
//!    «pedido tomado», e um `?` que falhasse calaria o pedido em vez de acabar o quadro.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::rust_src::{fn_body, fn_names};

fn render_loop() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop")
}

/// O CÓDIGO, sem comentários e sem o conteúdo das strings — um `{x:?}` numa mensagem não é um `?`, e uma nota que cita
/// `hero.bus` não escreve na fila.
fn code_only(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for line in s.lines() {
        let mut in_str = false;
        let mut prev = '\0';
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if !in_str && c == '/' && chars.peek() == Some(&'/') {
                break;
            }
            if c == '"' && prev != '\\' {
                in_str = !in_str;
                out.push('"');
            } else if !in_str {
                out.push(c);
            }
            prev = if prev == '\\' && c == '\\' { '\0' } else { c };
        }
        out.push('\n');
    }
    out
}

/// Os ficheiros dos sub-drenos (`fase_bus_*.rs`, sem o do próprio dreno).
fn sub_drain_files() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = fs::read_dir(render_loop())
        .expect("o render_loop lê-se")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("fase_bus_") && n.ends_with(".rs") && n != "fase_bus_drain.rs")
        .map(|n| {
            let src = fs::read_to_string(render_loop().join(&n)).expect("o sub-dreno lê-se");
            (n, src)
        })
        .collect();
    out.sort();
    assert!(
        out.len() >= 5,
        "a varredura achou {} ficheiros de sub-drenos — abaixo do piso, e um censo sobre menos ficheiros fica verde \
         sobre nada",
        out.len()
    );
    out
}

/// O corpo da `fase_bus_drain`, o dono da fila.
fn drain_body() -> String {
    let src = fs::read_to_string(render_loop().join("fase_bus_drain.rs")).expect("o dreno lê-se");
    code_only(fn_body(&src, "fase_bus_drain").expect("a `fase_bus_drain` existe"))
}

/// **A fila só se mexe no dreno: sai uma vez, volta uma vez, e volta DEPOIS do laço.**
#[test]
fn no_sub_drain_writes_the_bus_it_borrowed() {
    let offenders: Vec<String> = sub_drain_files()
        .into_iter()
        .filter(|(_, src)| code_only(src).contains(".bus"))
        .map(|(n, _)| n)
        .collect();
    assert!(
        offenders.is_empty(),
        "um sub-dreno lê ou escreve a fila do barramento: {offenders:?} — durante o dreno ela está FORA do \
         `HeroScreen` (`mem::take`), e o que um braço lhe empurrar some quando o dreno a devolve"
    );
    let body = drain_body();
    assert_eq!(body.matches("std::mem::take(&mut hero.bus)").count(), 1, "a fila sai do `HeroScreen` uma vez");
    let laco = body.find("for action in bus.drain()").expect("o laço drena a fila tomada");
    let volta = body.find(".bus = bus;").expect("a fila volta ao `HeroScreen`");
    assert_eq!(body.matches(".bus = bus;").count(), 1, "a fila volta uma vez");
    assert!(volta > laco, "a fila volta ANTES do laço que a drena");
}

/// Os sub-drenos pela ORDEM em que o dreno os chama (a cadeia `and_then`).
fn chain() -> Vec<String> {
    let body = drain_body();
    let names: Vec<String> = body
        .match_indices(".fase_bus_")
        .map(|(i, _)| {
            let rest = &body[i + 1..];
            rest[..rest.find('(').expect("uma chamada")].to_string()
        })
        .collect();
    assert!(names.len() >= 11, "a cadeia do dreno tem {} sub-drenos — abaixo do piso", names.len());
    names
}

/// O corpo de cada `fn fase_bus_*` dos ficheiros dos sub-drenos.
fn sub_drain_bodies() -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (_, src) in sub_drain_files() {
        for name in fn_names(&src).into_iter().filter(|n| n.starts_with("fase_bus_")) {
            let body = fn_body(&src, &name).expect("o corpo abre e fecha").to_string();
            out.insert(name, code_only(&body));
        }
    }
    out
}

/// O PEDIDO que um braço toma: os caminhos `Tipo::Variante` do padrão, antes da guarda e da seta
/// (`EditorAction::Hierarchy/HierRequest::Delete`), e se ele tem guarda.
fn arm_key(pattern: &str) -> (String, bool) {
    let (pat, guarded) = match pattern.find(" if ") {
        Some(i) => (&pattern[..i], true),
        None => (pattern, false),
    };
    let bytes = pat.as_bytes();
    let mut parts = Vec::new();
    let mut i = 0;
    while let Some(k) = pat[i..].find("::") {
        let at = i + k;
        let start = pat[..at].rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).map_or(0, |p| p + 1);
        let end = pat[at + 2..]
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .map_or(pat.len(), |p| at + 2 + p);
        if bytes.get(start).is_some_and(u8::is_ascii_uppercase) && bytes.get(at + 2).is_some_and(u8::is_ascii_uppercase) {
            parts.push(pat[start..end].to_string());
        }
        i = at + 2;
    }
    (parts.join("/"), guarded)
}

/// Os padrões dos braços de `match action` de um corpo: da linha que começa por `EditorAction::` (ou `nome @
/// EditorAction::`) até à seta.
fn arms(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut from = 0;
    for line in body.split_inclusive('\n') {
        let t = line.trim_start();
        let starts = t.starts_with("EditorAction::")
            || t.split_once(" @ ").is_some_and(|(a, b)| {
                a.chars().all(|c| c.is_ascii_lowercase() || c == '_') && b.starts_with("EditorAction::")
            });
        if starts {
            let rest = &body[from..];
            let seta = rest.find("=>").expect("um braço tem seta");
            out.push(rest[..seta].split_whitespace().collect::<Vec<_>>().join(" "));
        }
        from += line.len();
    }
    out
}

/// **Cada pedido tem UM braço na cadeia inteira** — o aviso `unreachable_patterns` que as onze funções perderam.
#[test]
fn every_request_has_one_arm_across_the_chain() {
    let bodies = sub_drain_bodies();
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    let mut repetidos = Vec::new();
    let mut total = 0;
    for name in chain() {
        let body = bodies
            .get(&name)
            .unwrap_or_else(|| panic!("a cadeia do dreno chama `{name}`, que não está nos ficheiros dos sub-drenos"));
        for arm in arms(body) {
            let (key, guarded) = arm_key(&arm);
            assert!(key.starts_with("EditorAction::"), "um braço sem pedido legível: {arm}");
            total += 1;
            if let Some(antes) = seen.insert(key.clone(), name.clone()) {
                repetidos.push(format!("{key}{}: `{antes}` e `{name}`", if guarded { " (com guarda)" } else { "" }));
            }
        }
    }
    assert!(
        total >= 87,
        "a varredura achou {total} braços — medidos 87 em 2026-09-13 (mais o `_` da barra do topo, que não conta); \
         abaixo disso ela partiu-se"
    );
    assert!(
        repetidos.is_empty(),
        "um pedido tem mais de um braço na cadeia — o do sub-dreno ANTERIOR rouba-o, e o `rustc` já não avisa: \
         {repetidos:?}. Se a guarda é deliberada, os dois braços têm de viver no MESMO `match`."
    );
}

/// **Nenhum corpo de braço acaba o dreno cedo** — o `?` só mora nas re-derivações do `gfx` e nas chamadas aos pedaços.
#[test]
fn no_arm_body_ends_the_drain_early() {
    let mut achados = Vec::new();
    for (name, body) in sub_drain_bodies() {
        for line in body.lines() {
            let porta = line.contains("self.gfx.as_mut()?")
                || line.contains("hero_screen.as_mut()?")
                || line.contains("self.fase_bus_");
            if line.contains('?') && !porta {
                achados.push(format!("{name}: {}", line.trim()));
            }
        }
    }
    assert!(
        achados.is_empty(),
        "um `?` num corpo de braço: o `None` de um sub-dreno lê-se «pedido tomado», logo ele calaria o pedido em vez \
         de acabar o quadro — {achados:?}"
    );
}
