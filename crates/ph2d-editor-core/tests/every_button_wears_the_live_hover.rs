//! **Um `Button` ou um `Slider` pintado veste o `t` VIVO — a estrada do estado duro está fechada.**
//!
//! A wave da UI viva deu ao substrato um `t` de hover por id e publicou-o no store
//! (`WidgetStore::button_visual`/`slider_visual`, os gêmeos exactos do `panel_scroll_live`). O
//! consumo tem **uma porta** por família: `Button::visual((state, t))`,
//! `Slider::visual((state, t))`. Este gate afirma que o sítio N+1 não pode nascer na estrada
//! antiga.
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
//! ⚠️ **O `IconButton` fechou a mesma estrada por TIPO, e por isso não está aqui.** A porta dele é
//! uma função, não um builder, então o par pôde entrar na própria assinatura
//! (`paint_icon_button(.., visual: (ButtonState, f32), ..)`): um `ButtonState` solto **não
//! compila**, e um estado forçado tem de se declarar. *Quando o compilador consegue ser o gate, o
//! gate é ele* — este ficheiro existe só onde a API não pode recusar (um builder cujo `.state()`
//! continua legítimo para os estados duros).
//!
//! ⚠️ **Fora de escopo, MEDIDO, e não é o que esta nota dizia.** Ela afirmava que `Checkbox` e
//! `Toggle` *"ainda não passaram pela porta"* — eles passaram (14 e 4 sítios chamam
//! `checkbox_visual`/`toggle_visual`), e a nota sobreviveu ao facto. O que há hoje é pior e mais
//! interessante: **nove** desses sítios escrevem `.visual(store.…_visual(id)).state(x)` na MESMA
//! cadeia, e como `visual(v)` É `state(v.0).hover_t(v.1)`, o `.state(x)` que vem a seguir
//! **sobrescreve** a metade do estado — o pior deles crava `.state(ToggleState::Normal)`, um
//! toggle que não pode acender. Admiti-los nesta varredura seria **verde sobre defeito**: o
//! `chain_verb` devolve o verbo que aparece PRIMEIRO, e nesses sítios é o `.visual(`.
//!
//! A pergunta que os apanha é outra — *«tomou a estrada e depois saiu dela?»* — e a cura é nos
//! nove sítios, não no scanner. Fica **nomeada** em vez de contrabandeada nesta wave; a família
//! `Button`/`Slider` foi medida e **não tem nenhum** sítio dessa forma.

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

/// Varre a árvore de produção por cadeias `<ctor>` e separa-as em `(ofensores, convertidos)`.
///
/// ⚠️ **Parametrizada pelo construtor de propósito: uma lei, uma varredura.** O `Slider` chegou
/// à mesma porta (`.visual((SliderState, f32))`) pelo mesmo motivo, e dar-lhe um ficheiro próprio
/// seria a segunda cópia de um scanner cujo modo de falha — *ver zero e chamar-lhe verde* — é
/// precisamente o que o controlo positivo abaixo existe para apanhar.
fn scan_widget(ctor: &str) -> (Vec<String>, Vec<String>) {
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
        while let Some(hit) = src[from..].find(ctor) {
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
            from = at + ctor.len();
        }
    };

    visit(&crates_root(), &mut scan);
    visit(&shells_root(), &mut scan);
    (offenders, converted)
}

#[test]
fn every_button_wears_the_live_hover() {
    let (offenders, converted) = scan_widget("Button::new(");

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

/// O mesmo, para o `Slider` — a superfície que se ARRASTA.
///
/// ⚠️ **A lei é a mesma e o defeito era um andar mais fundo.** No `Button` o estado DURO chegava
/// ao pintor e ele o honrava; no `Slider` o despachante escrevia `Hovered`/`Dragging` no store, a
/// struct os carregava, e o **pintor os DEITAVA FORA** — `paint_slider(Hovered)` era byte-idêntico
/// a `paint_slider(Normal)`. Por isso nenhum gate que olhasse o store podia vê-lo, e por isso este
/// gate não basta sozinho: ele prova que a informação CHEGA, e os gates de tinta
/// (`the_track_reacts_to_the_pointer` e irmãos) provam que ela é USADA.
///
/// ⚠️ **O controlo positivo é MENOR que o do botão, e é um facto e não folga:** só ~8 dos 33
/// sítios de `Slider::new` leem o store — o resto passa o neutro declarado (uma pista inerte de
/// waveform, a rota legada do picker), e a alavanca de verdade é o `paint_slider_with_chip`, que
/// serve ~67 linhas de painel por DENTRO e não aparece nesta varredura.
#[test]
fn every_slider_wears_the_live_hover() {
    let (offenders, converted) = scan_widget("Slider::new(");

    assert!(
        converted.len() >= 5,
        "o scanner viu apenas {} cadeias `Slider::new(...).visual(...)` — ele está partido, e um \
         gate que não vê nada não pode acusar nada. Esperado: os sítios que leem o store \
         (fill_modal, onion_modal, painter-layers).",
        converted.len()
    );

    assert!(
        offenders.is_empty(),
        "estes `Slider` recebem o estado DURO e nunca acendem sob o ponteiro:\n  {}\n\
         conserto: `.visual(store.slider_visual(ID))` — UMA pergunta, o estado e o `t` juntos. \
         Uma pista sem `NodeId` a que perguntar declara o neutro: \
         `(SliderState::Normal, ph2d_editor_core::motion::SETTLED)`.",
        offenders.join("\n  ")
    );
}
