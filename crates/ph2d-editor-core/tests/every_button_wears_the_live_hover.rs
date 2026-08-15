//! **Um `Button` pintado veste o `t` VIVO — a estrada do estado duro está fechada.**
//!
//! A wave da UI viva deu ao substrato um `t` de hover por id e publicou-o no store
//! (`WidgetStore::button_visual`, o gêmeo exacto do `panel_scroll_live`). O consumo tem **uma
//! porta**: `Button::visual((state, t))`. Este gate afirma que o sítio N+1 não pode nascer na
//! estrada antiga.
//!
//! ⚠️ **Porquê um gate e não «a revisão apanha»:** `.state(x)` continua a compilar, a pintar e a
//! responder ao rato — o que falta é a única coisa que nenhum teste de unidade olha, a
//! *transição*. Um botão esquecido fica **silenciosamente discreto** no meio de vizinhos que
//! deslizam, e a suíte inteira fica verde sobre isso. É a razão de `visual()` receber o PAR em vez
//! de haver um `.hover_t()` ao lado: com duas chamadas, esquecer a segunda é o estado natural.
//!
//! ⚠️ **O controlo positivo é metade do gate.** Um scanner com a raiz errada, a extensão errada ou
//! um regex que deixou de casar reporta **zero ofensores** — exactamente o que um produto correcto
//! reporta. Por isso ele também conta os sítios JÁ convertidos e exige que os veja, em vários
//! crates: uma varredura vazia é uma falha alta, nunca um verde.
//!
//! **Fora de escopo, e nomeado:** `IconButton`, `Checkbox` e `Toggle` têm estados próprios e ainda
//! não passaram pela porta. Quando passarem, é aqui que a família entra — não num segundo gate.

use std::fs;
use std::path::{Path, PathBuf};

/// Raiz de `crates/` (o `CARGO_MANIFEST_DIR` é `crates/ph2d-editor-core`).
fn crates_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ dir")
        .to_path_buf()
}

/// Raiz de `shells/` — os painéis não são os únicos que pintam botões.
fn shells_root() -> PathBuf {
    crates_root()
        .parent()
        .expect("workspace root")
        .join("shells")
}

/// Fonte de PRODUÇÃO: fixtures de smoke declaram estados à mão de propósito.
fn is_production(rel: &str) -> bool {
    !(rel.contains("/tests/") || rel.ends_with("_tests.rs") || rel.ends_with("/tests.rs"))
}

fn visit(dir: &Path, cb: &mut dyn FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(&path, cb);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            cb(&path);
        }
    }
}

/// Percorre uma cadeia `Button::new(` a partir de `at` e devolve o método que ela usa para
/// receber o estado visual: `Some(".state(")` (a estrada antiga) ou `Some(".visual(")` (a porta).
///
/// ⚠️ **Para no `;`** — uma cadeia de builder é UMA expressão, e atravessar o ponto e vírgula
/// leria o `.state(` de um `NumberInput` que calhasse vir a seguir (é literalmente o erro que a
/// conversão desta wave cometeu num ficheiro: um nome repetido apanhado por uma substituição de
/// ficheiro inteiro).
fn chain_verb(src: &str, at: usize) -> Option<&'static str> {
    let tail = &src[at..];
    let end = tail.find(';').unwrap_or(tail.len());
    let chain = &tail[..end];
    let s = chain.find(".state(");
    let v = chain.find(".visual(");
    match (s, v) {
        (Some(a), Some(b)) => Some(if a < b { ".state(" } else { ".visual(" }),
        (Some(_), None) => Some(".state("),
        (None, Some(_)) => Some(".visual("),
        (None, None) => None,
    }
}

#[test]
fn every_button_wears_the_live_hover() {
    let mut offenders: Vec<String> = Vec::new();
    let mut converted: Vec<String> = Vec::new();

    let mut scan = |path: &Path| {
        let rel = path.to_string_lossy().replace('\\', "/");
        if !is_production(&rel) {
            return;
        }
        let Ok(src) = fs::read_to_string(path) else {
            return;
        };
        let mut from = 0usize;
        while let Some(hit) = src[from..].find("Button::new(") {
            let at = from + hit;
            // `IconButton::new(` / `ToggleButton::new(` não são este widget.
            let is_plain = src[..at]
                .chars()
                .next_back()
                .is_none_or(|c| !c.is_alphanumeric() && c != '_');
            if is_plain {
                let line = src[..at].matches('\n').count() + 1;
                match chain_verb(&src, at) {
                    Some(".state(") => offenders.push(format!("{rel}:{line}")),
                    Some(".visual(") => converted.push(format!("{rel}:{line}")),
                    _ => {}
                }
            }
            from = at + "Button::new(".len();
        }
    };

    visit(&crates_root(), &mut scan);
    visit(&shells_root(), &mut scan);

    // O CONTROLO POSITIVO, primeiro: sem ele um scanner partido passa em silêncio.
    assert!(
        converted.len() >= 30,
        "o scanner viu apenas {} cadeias `Button::new(...).visual(...)` — ele está partido \
         (raiz errada / regex que deixou de casar), e um gate que não vê nada não pode acusar \
         nada. Esperado: dezenas, espalhadas pelos painéis.",
        converted.len()
    );
    let crates_seen = converted
        .iter()
        .filter_map(|s| s.split("/crates/").nth(1))
        .filter_map(|s| s.split('/').next())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        crates_seen.len() >= 5,
        "o scanner só alcançou {} crate(s) ({crates_seen:?}) — a varredura não está a descer a \
         árvore inteira.",
        crates_seen.len()
    );

    assert!(
        offenders.is_empty(),
        "estes `Button` recebem o estado DURO e ficam silenciosamente discretos:\n  {}\n\
         conserto: `let v = store.button_visual(ID);` + `.visual(v)`. Um estado FORÇADO \
         (um toggle armado, um botão desligado) emparelha com o neutro: \
         `(ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)`.",
        offenders.join("\n  ")
    );
}
