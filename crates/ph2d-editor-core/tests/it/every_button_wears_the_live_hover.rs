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
//! ⚠️ **E a estrada tem DUAS saídas, não uma — a segunda foi curada em 2026-08-15.** Uma wave
//! anterior deixou aqui, nomeado com o número, que **nove** sítios escreviam
//! `.visual(store.…_visual(id)).state(x)` na MESMA cadeia: como `visual(v)` É
//! `state(v.0).hover_t(v.1)`, o `.state(x)` que vem a seguir **sobrescreve** a metade do estado.
//! Sete deles re-passavam o estado GUARDADO (redundante enquanto o widget está registado, e a
//! perder a alternativa hot/active quando não está) e **o nono cravava
//! `.state(ToggleState::Normal)`** — os interruptores das camadas de ajuste do Painter, mortos ao
//! ponteiro apesar de a wave lhes ter dado o par.
//!
//! Admiti-los na varredura de cima seria **verde sobre defeito**: o `chain_verb` devolve o verbo
//! que aparece PRIMEIRO, e nesses sítios é o `.visual(`. A pergunta que os apanha é outra —
//! *«tomou a estrada e depois SAIU dela?»* — e é o
//! [`a_visual_pair_is_not_undone_by_a_later_state`] abaixo que a faz, para as quatro famílias de
//! uma vez. ⚠️ A cura foi nos nove sítios, **não** no scanner: um scanner tolerante teria
//! transformado a saída da estrada num idioma.

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

/// Dentro da cadeia que começa em `at`: ela chama `.visual(` e **depois** `.state(`?
///
/// ⚠️ **É uma pergunta diferente da do [`chain_verb`], e não uma versão melhor dela.** Aquele
/// responde *«por que porta este sítio recebe o estado?»* e devolve o verbo que aparece PRIMEIRO
/// — a resposta certa para *«tomou a estrada nova?»*. Este responde *«e depois desfez?»*. Um
/// único scanner a tentar as duas escolheria uma delas para ficar cega.
fn chain_leaves_the_road(src: &str, at: usize) -> bool {
    let tail = &src[at..];
    let end = tail.find(';').unwrap_or(tail.len());
    let chain = &tail[..end];
    match (chain.find(".visual("), chain.rfind(".state(")) {
        (Some(v), Some(s)) => s > v,
        _ => false,
    }
}

/// Varre a produção por cadeias `<ctor>` que tomam a porta e **saem dela** a seguir.
fn scan_leavers(ctor: &str) -> Vec<String> {
    let mut offenders: Vec<String> = Vec::new();
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
            let is_plain = src[..at]
                .chars()
                .next_back()
                .is_none_or(|c| !c.is_alphanumeric() && c != '_');
            if is_plain && chain_leaves_the_road(&src, at) {
                offenders.push(format!("{rel}:{}", src[..at].matches('\n').count() + 1));
            }
            from = at + ctor.len();
        }
    };
    visit(&crates_root(), &mut scan);
    visit(&shells_root(), &mut scan);
    offenders
}

/// **Quem toma a porta não sai dela três linhas abaixo.**
///
/// **Mutação que deve sangrar:** repor `.state(ToggleState::Normal)` no
/// `paint_adjust.rs` (ou qualquer `.state(x)` depois de um `.visual(..)`).
#[test]
fn a_visual_pair_is_not_undone_by_a_later_state() {
    // O CONTROLO POSITIVO é o mesmo dos gates acima, e por isso é reusado em vez de re-escrito:
    // se a varredura não vê as cadeias convertidas, também não veria uma que saísse da estrada.
    let seen: usize = [
        "Button::new(",
        "Slider::new(",
        "Checkbox::new(",
        "Toggle::new(",
    ]
    .iter()
    .map(|c| scan_widget(c).1.len())
    .sum();
    assert!(
        seen >= 40,
        "o scanner viu apenas {seen} cadeias `.visual(..)` nas quatro famílias — ele está \
         partido, e um gate que não vê nada não pode acusar nada."
    );

    let offenders: Vec<String> = [
        "Button::new(",
        "Slider::new(",
        "Checkbox::new(",
        "Toggle::new(",
    ]
    .iter()
    .flat_map(|c| scan_leavers(c))
    .collect();
    assert!(
        offenders.is_empty(),
        "estas cadeias tomam a porta e depois a DESFAZEM — `visual(v)` É \
         `state(v.0).hover_t(v.1)`, então o `.state(x)` a seguir sobrescreve metade do par:\n  \
         {}\n\
         conserto: apagar o `.state(..)`. Se o estado é mesmo FORÇADO (um botão desligado), \
         então é o `.visual(..)` que sai, emparelhado com o neutro declarado.",
        offenders.join("\n  ")
    );
}

/// **E a metade de TINTA: construído só com o par, o widget segue o store.**
///
/// O gate de cima é um scanner — ele prova que a informação não é deitada fora na cadeia. Este
/// prova que ela CHEGA à tinta, que é a única coisa que o artista vê. Sem ele, apagar os nove
/// `.state(..)` estaria apoiado numa leitura minha de `checkbox_visual`, não numa medição.
///
/// ⚠️ **O CONTROLO é a segunda metade:** um `id` que o store nunca viu tem de pintar o mundo de
/// antes. Sem ela o gate seria satisfeito por qualquer lei que tingisse tudo no arranque.
///
/// ⚠️ **A caixa é `Unchecked` de propósito, e a primeira versão deste gate com `Checked` nasceu
/// VERMELHA sobre um produto correcto.** O `paint_checkbox` diz porquê na linha ao lado do
/// código: uma caixa MARCADA é `Accent` em **qualquer** estado, então ali não existe eixo a
/// percorrer — o eixo do hover é o par NÃO-marcado (`Bg1 → Bg2`, `Border → BorderEmph`). Fica
/// **nomeado** e não construído: dar reacção à caixa marcada pede um par de tokens que o design
/// system não tem, e isso é decisão de look. O `Toggle` não partilha o limite (o eixo dele é uma
/// PRESENÇA de traço, independente do `on`), e por isso a fixture dele liga o `on`.
#[test]
fn a_checkbox_and_a_toggle_built_from_the_pair_alone_follow_the_store() {
    use ph2d_a11y::NodeId;
    use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
    use ph2d_editor_core::widget::{
        Checkbox, CheckboxState, CheckboxValue, Toggle, ToggleState, paint_checkbox, paint_toggle,
    };
    use ph2d_editor_core::zones::Rect;
    use ph2d_text::TextSystem;
    use ph2d_tokens::Theme;
    use ph2d_vector::VectorScene;

    const ID: NodeId = NodeId(1);
    let rect = Rect::new(0.0, 0.0, 120.0, 18.0);

    let cb_ink = |armed: Option<CheckboxState>| {
        let mut store = WidgetStore::with_capacity(4);
        if let Some(state) = armed {
            store.register(
                ID,
                InteractiveState::Checkbox {
                    state,
                    value: CheckboxValue::Unchecked,
                },
            );
        }
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let cb = Checkbox::new(ID, "x")
            .visual(store.checkbox_visual(ID))
            .value(CheckboxValue::Unchecked);
        paint_checkbox(&cb, rect, &mut scene, &mut text, Theme::Forge);
        let e = scene.inner().encoding();
        (e.path_data.clone(), e.draw_data.clone())
    };
    let tg_ink = |armed: Option<ToggleState>| {
        let mut store = WidgetStore::with_capacity(4);
        if let Some(state) = armed {
            store.register(ID, InteractiveState::Toggle { state, on: true });
        }
        let mut scene = VectorScene::new();
        let tg = Toggle::new(ID, "").visual(store.toggle_visual(ID)).on(true);
        paint_toggle(&tg, rect, &mut scene, Theme::Forge);
        let e = scene.inner().encoding();
        (e.path_data.clone(), e.draw_data.clone())
    };

    let cb_normal = cb_ink(Some(CheckboxState::Normal));
    assert_ne!(
        cb_ink(Some(CheckboxState::Hovered)),
        cb_normal,
        "um Checkbox construído só com `.visual(store.checkbox_visual(id))` nao segue o store — \
         as caixas do app ficam mudas sob o ponteiro"
    );
    let tg_normal = tg_ink(Some(ToggleState::Normal));
    assert_ne!(
        tg_ink(Some(ToggleState::Hovered)),
        tg_normal,
        "um Toggle construído só com `.visual(store.toggle_visual(id))` nao segue o store"
    );

    // O CONTROLO: id desconhecido cai no repouso, e a tinta tem de ser a de antes.
    assert_eq!(
        cb_ink(None),
        cb_normal,
        "um Checkbox que o store nunca viu deixou de pintar o mundo de antes"
    );
    assert_eq!(
        tg_ink(None),
        tg_normal,
        "um Toggle que o store nunca viu deixou de pintar o mundo de antes"
    );
}
