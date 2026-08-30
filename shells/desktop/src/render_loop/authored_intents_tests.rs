//! Gates do dreno do painel autorado — **o braço que falta é o controlo que morre.**
//!
//! Três lentes, e nenhuma delas vê o que a outra vê:
//!
//! 1. [`the_drain_names_every_authored_intent_variant`] — **estrutural**. Lê o `enum` na crate que
//!    o declara e o `match` neste dreno, e reprova se uma variante não tiver braço NOMEADO ou se
//!    alguém escrever um curinga. *É o gate que morre quando alguém acrescenta uma variante nova
//!    ao enum e não a drena* — a forma exacta do defeito de 2026-08-30.
//! 2. [`every_control_kind_reaches_a_signal_or_is_named_silent`] — **o censo das FAMÍLIAS DE
//!    WIDGET**. ⚠️ *Um `match` exaustivo NÃO guarda a lista que um laço itera*: o `match` da
//!    lente 1 cobre as 5 variantes do intent, e **nada nele** diz que os 13 tipos de controlo do
//!    catálogo continuam a chegar lá. Esta lente é o censo próprio dessa lista, e ela morre no dia
//!    em que um 14.º tipo de controlo nascer.
//! 3. [`a_choice_grits_the_option_not_just_the_row`] e vizinhas — **comportamentais**. O nome que
//!    sai, para cada variante construída à mão.

use super::{choice_name, signal_name};
use ph2d_editor::widget::WidgetKind;
use ph2d_panel_authored::AuthoredIntent;
use ph2d_panel_authored::rows::{Row, set_live_rows};

// ---------------------------------------------------------------- lente 1 --

/// O caminho, a partir da raiz do repo, dos dois lados desta costura.
const ENUM_DECL: &str = "crates/ph2d-panel-authored/src/state.rs";
const DRAIN: &str = "shells/desktop/src/render_loop/authored_intents.rs";

/// A metade JUSTA: sem ela, um parser que devolve zero variantes passa provando nada.
const MIN_VARIANTS: usize = 5;

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

/// O índice do `}` que fecha o `{` em `open`, ignorando o que estiver dentro de strings.
fn close_of(t: &[u8], open: usize) -> usize {
    let (o, c) = match t[open] {
        b'(' => (b'(', b')'),
        _ => (b'{', b'}'),
    };
    let mut depth = 0i32;
    let mut i = open;
    while i < t.len() {
        match t[i] {
            b'"' => {
                i += 1;
                while i < t.len() && t[i] != b'"' {
                    i += if t[i] == b'\\' { 2 } else { 1 };
                }
            }
            ch if ch == o => depth += 1,
            ch if ch == c => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    t.len()
}

/// As variantes de topo de `pub enum <name>` em `src`.
fn variants_of(src: &str, name: &str) -> Vec<String> {
    let head = format!("pub enum {name}");
    let at = src.find(&head).unwrap_or_else(|| {
        panic!("`pub enum {name}` não foi encontrado em {ENUM_DECL} — a sonda ficou cega")
    });
    let open = at + src[at..].find('{').expect("corpo do enum");
    let end = close_of(src.as_bytes(), open);
    src[open + 1..end]
        .lines()
        .filter_map(|l| {
            let l = l.trim_start();
            let head: String = l
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let after = l[head.len()..].trim_start();
            (head.starts_with(|c: char| c.is_ascii_uppercase())
                && (after.starts_with('{') || after.starts_with('(') || after.starts_with(',')))
            .then_some(head)
        })
        .collect()
}

/// O corpo de `fn <name>` em `src` — onde o `match` do dreno vive.
fn fn_body<'a>(src: &'a str, name: &str) -> &'a str {
    let at = src
        .find(&format!("fn {name}"))
        .unwrap_or_else(|| panic!("`fn {name}` sumiu de {DRAIN} — a sonda ficou cega"));
    let open = at + src[at..].find('{').expect("corpo da função");
    &src[open + 1..close_of(src.as_bytes(), open)]
}

/// `AuthoredIntent::<v>` aparece em `body` como **braço de match** (padrão seguido de `=>`)?
fn has_named_arm(body: &str, v: &str) -> bool {
    let needle = format!("AuthoredIntent::{v}");
    let mut from = 0usize;
    while let Some(rel) = body[from..].find(&needle) {
        let at = from + rel;
        from = at + needle.len();
        // Fronteira de palavra à direita: `Text` não pode casar dentro de `TextThing`.
        let mut j = at + needle.len();
        if body
            .as_bytes()
            .get(j)
            .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_')
        {
            continue;
        }
        while body.as_bytes().get(j).is_some_and(u8::is_ascii_whitespace) {
            j += 1;
        }
        if matches!(body.as_bytes().get(j), Some(b'{' | b'(')) {
            j = close_of(body.as_bytes(), j) + 1;
        }
        while body.as_bytes().get(j).is_some_and(u8::is_ascii_whitespace) {
            j += 1;
        }
        if body[j..].starts_with("=>") {
            return true;
        }
    }
    false
}

/// **Toda variante do `AuthoredIntent` tem braço NOMEADO no dreno, e não há curinga.**
///
/// ⛔ A cura NÃO é acrescentar um `_ => {}`: ele reproduz o defeito na primeira variante nova, em
/// silêncio e sem aviso do compilador. Um braço nomeado — mesmo vazio, com o motivo ao lado — faz
/// o compilador cobrar a próxima.
#[test]
fn the_drain_names_every_authored_intent_variant() {
    let root = repo_root();
    let decl = std::fs::read_to_string(root.join(ENUM_DECL)).expect("ler o enum");
    let drain = std::fs::read_to_string(root.join(DRAIN)).expect("ler o dreno");

    let variants = variants_of(&decl, "AuthoredIntent");
    assert!(
        variants.len() >= MIN_VARIANTS,
        "a sonda só viu {} variante(s) de `AuthoredIntent` (baseline 2026-08-30: {MIN_VARIANTS}). \
         Ou o enum encolheu, ou a leitura do corpo dele partiu — e num enum vazio este gate seria \
         verde para sempre.",
        variants.len()
    );

    let body = fn_body(&drain, "signal_name");
    let missing: Vec<&String> = variants
        .iter()
        .filter(|v| !has_named_arm(body, v))
        .collect();
    assert!(
        missing.is_empty(),
        "estas variantes de `AuthoredIntent` NÃO têm braço no dreno ({DRAIN}): {missing:?}.\n\n\
         Elas nascem de um gesto do artista, entram na fila, são drenadas e morrem no fim do \
         quadro. O controlo acende sob o dedo e nada acontece — e nenhum gate de REGISTO o nota, \
         porque o widget está registado, pintado e focável.\n\
         Cura: um braço NOMEADO (mesmo vazio, com o motivo escrito ao lado)."
    );

    let wildcard = body
        .lines()
        .map(str::trim_start)
        .find(|l| l.starts_with("_ =>") || l.starts_with("_ if"));
    assert!(
        wildcard.is_none(),
        "o `match` do dreno ganhou um curinga (`{}`). Ele apaga exactamente a protecção deste \
         gate: a próxima variante do enum compila, cai no curinga e morre em silêncio — que é o \
         defeito de 2026-08-30 com outra sintaxe.",
        wildcard.unwrap_or_default()
    );
}

// ---------------------------------------------------------------- lente 2 --

/// **O CENSO das famílias de widget** — que variante cada tipo de controlo publica.
///
/// A tabela é derivada de `ph2d_panel_authored::event::apply_event`, que escolhe o braço pelo que
/// o `WidgetStore` de facto guarda: slider ⇒ `Value`, toggle/checkbox ⇒ `Flag`, texto (o `text` de
/// um `TextInput` **e o `buffer` de um `NumberInput`**) ⇒ `Text`, seleção ⇒ `Choice`, resto ⇒
/// `Fired`.
const FAMILY: &[(WidgetKind, &str)] = &[
    (WidgetKind::Button, "Fired"),
    (WidgetKind::Tag, "Fired"),
    (WidgetKind::ListItem, "Fired"),
    (WidgetKind::IconButton, "Fired"),
    (WidgetKind::Slider, "Value"),
    (WidgetKind::Toggle, "Flag"),
    (WidgetKind::Checkbox, "Flag"),
    (WidgetKind::TextInput, "Text"),
    (WidgetKind::NumberInput, "Text"),
    (WidgetKind::Tabs, "Choice"),
    (WidgetKind::RadioGroup, "Choice"),
    (WidgetKind::SegmentedAdaptive, "Choice"),
    (WidgetKind::Dropdown, "Choice"),
];

/// **Catraca das famílias deliberadamente CALADAS — ela só ENCOLHE.**
///
/// Cada linha é `(variante, motivo)`. As duas primeiras têm o outro fio a existir de facto
/// (`crate::vec_widget_drive`); a terceira **não tem consumidor nenhum**, e é a dívida que este
/// gate mantém à vista em vez de a deixar disfarçada de decisão.
const SILENT: &[(&str, &str)] = &[
    (
        "Value",
        "o valor vive no WidgetStore e a arte le'-o por vec_widget_drive::Drive::Opacity",
    ),
    (
        "Flag",
        "idem Value: vec_widget_drive::Drive::Visible le' o estado do store",
    ),
    (
        "Text",
        "DIVIDA: o conteudo chega ao WidgetStore e vec_widget_drive::bindable ainda nao aceita \
         TextInput/NumberInput — compor o texto no NOME esta' refutado (contrato digitavel)",
    ),
];

fn row_of(kind: WidgetKind) -> Row {
    Row {
        kind,
        label: "X".to_owned(),
        key: "x".to_owned(),
        id: ph2d_editor::ids::authored_row_id("x"),
        rgba: None,
        icon: None,
        icon_id: None,
        options: Vec::new(),
    }
}

/// Um intent da família `family`, para perguntar ao dreno o que ele faz com ela.
fn sample(family: &str) -> AuthoredIntent {
    let key = "blend".to_owned();
    match family {
        "Fired" => AuthoredIntent::Fired { key },
        "Choice" => AuthoredIntent::Choice { key, index: 1 },
        "Value" => AuthoredIntent::Value { key, value: 0.5 },
        "Flag" => AuthoredIntent::Flag { key, on: true },
        "Text" => AuthoredIntent::Text {
            key,
            text: "abc".to_owned(),
        },
        other => panic!("familia desconhecida no censo: {other}"),
    }
}

/// **Todo tipo que RESPONDE a um gesto chega a um sinal, ou está nomeado como calado.**
///
/// ⚠️ Esta lente vê o que a estrutural não vê: um `match` completo sobre as 5 variantes continua
/// completo depois de um 14.º tipo de controlo nascer — e o tipo novo pode publicar uma família
/// que ninguém publica. O censo morre nesse dia, que é o único em que alguém tem de pensar.
#[test]
fn every_control_kind_reaches_a_signal_or_is_named_silent() {
    // Metade JUSTA nº 1: a tabela cobre EXACTAMENTE os tipos que respondem.
    let controls: Vec<WidgetKind> = WidgetKind::ALL
        .into_iter()
        .filter(|k| row_of(*k).is_control())
        .collect();
    let censused: Vec<WidgetKind> = FAMILY.iter().map(|(k, _)| *k).collect();
    let uncensused: Vec<&WidgetKind> = controls.iter().filter(|k| !censused.contains(k)).collect();
    assert!(
        uncensused.is_empty(),
        "tipos de controlo fora do censo: {uncensused:?}. Um tipo novo publica UMA das famílias \
         do `AuthoredIntent`, e ninguém aqui sabe qual — acrescente-o ao `FAMILY` depois de ler \
         `ph2d_panel_authored::event::apply_event`."
    );
    let ghosts: Vec<&WidgetKind> = censused.iter().filter(|k| !controls.contains(k)).collect();
    assert!(
        ghosts.is_empty(),
        "o censo lista tipos que `Row::is_control()` já não aceita: {ghosts:?}. Eles não geram \
         gesto nenhum — a linha tem de sair, senão este gate mede uma fixtura sua."
    );

    // Metade JUSTA nº 2: pelo menos uma família tem de chegar a um sinal, senão um dreno
    // inteiramente calado passaria com todas as linhas na catraca.
    let loud: Vec<&str> = FAMILY
        .iter()
        .map(|(_, f)| *f)
        .filter(|f| signal_name(&sample(f)).is_some())
        .collect();
    assert!(
        !loud.is_empty(),
        "NENHUMA família chega a um sinal — o painel autorado inteiro está mudo, e este gate \
         estaria a medir a própria catraca."
    );

    let dead: Vec<String> = FAMILY
        .iter()
        .filter(|(_, f)| signal_name(&sample(f)).is_none() && !SILENT.iter().any(|(s, _)| s == f))
        .map(|(k, f)| format!("{k:?} (publica AuthoredIntent::{f})"))
        .collect();
    assert!(
        dead.is_empty(),
        "estas famílias de widget são MORTAS — o gesto escreve, a fila enche, o dreno esvazia e \
         nada acontece:\n  {}\n\n\
         O sintoma que o artista reporta é *«o chip acende e nada muda»*.",
        dead.join("\n  ")
    );

    // **Controlo POSITIVO / a catraca só encolhe:** uma linha do `SILENT` que já não descreve
    // nada é uma dívida paga que ninguém apagou — ou uma sonda que ficou cega.
    let stale: Vec<String> = SILENT
        .iter()
        .filter(|(f, _)| signal_name(&sample(f)).is_some())
        .map(|(f, why)| format!("AuthoredIntent::{f} ({why})"))
        .collect();
    assert!(
        stale.is_empty(),
        "estas linhas do `SILENT` já não são caladas — a dívida foi paga e a catraca tem de \
         DESCER:\n  {}",
        stale.join("\n  ")
    );
}

// ---------------------------------------------------------------- lente 3 --

/// Publica uma tabela viva com uma row de lista `blend` de três opções.
fn publish_blend_row() {
    set_live_rows(Some(vec![Row {
        kind: WidgetKind::Tabs,
        label: "Blend".to_owned(),
        key: "blend".to_owned(),
        id: ph2d_editor::ids::authored_row_id("blend"),
        rgba: None,
        icon: None,
        icon_id: None,
        options: vec![
            "Normal".to_owned(),
            "Multiply".to_owned(),
            "Screen".to_owned(),
        ],
    }]));
}

/// **Uma escolha grita QUAL opção, e nunca o nome que a row gritaria sozinha.**
///
/// Este é o par mínimo do defeito: com o `if let Fired` de antes, `Multiply` e `Screen` não
/// gritavam nada; com um `Choice` que recuasse para a chave da row, os dois gritariam `blend` — e
/// um consumidor não teria como os distinguir, que é a mesma morte com outro nome.
#[test]
fn a_choice_grits_the_option_not_just_the_row() {
    publish_blend_row();
    let multiply = signal_name(&AuthoredIntent::Choice {
        key: "blend".to_owned(),
        index: 1,
    });
    let screen = signal_name(&AuthoredIntent::Choice {
        key: "blend".to_owned(),
        index: 2,
    });
    let fired = signal_name(&AuthoredIntent::Fired {
        key: "blend".to_owned(),
    });
    set_live_rows(None);

    assert_eq!(multiply.as_deref(), Some("blend/multiply"));
    assert_eq!(screen.as_deref(), Some("blend/screen"));
    assert_eq!(fired.as_deref(), Some("blend"));
    assert_ne!(
        multiply, screen,
        "duas opções diferentes gritam o mesmo nome — o consumidor não as distingue"
    );
    assert_ne!(
        multiply, fired,
        "uma escolha grita o mesmo nome que um botão com a chave da row — ela dispararia, em \
         silêncio, o contrato de outro controle"
    );
}

/// **O recuo é o ÍNDICE, e continua distinto do nome da row.**
///
/// Uma opção que a tabela viva não tem (índice fora de faixa) não pode colapsar em `blend`.
#[test]
fn an_option_the_live_table_does_not_have_still_gets_its_own_name() {
    publish_blend_row();
    let out = choice_name("blend", 9);
    let unknown_row = choice_name("nao_existe", 0);
    set_live_rows(None);
    assert_eq!(out, "blend/9");
    assert_eq!(unknown_row, "nao_existe/0");
}

/// **O separador não pode ser produzido por um rótulo do artista.**
///
/// `key_of` troca todo carácter não-alfanumérico por `_`, então uma row chamada *"Blend/Mode"* e
/// uma opção qualquer nunca compõem o mesmo nome que outra row + outra opção.
#[test]
fn no_authored_label_can_forge_the_separator() {
    let k = crate::ui_panel_spec::key_of("Blend/Mode");
    assert!(
        !k.contains('/'),
        "a chave de uma row passou a poder conter o separador ({k}) — duas composições diferentes \
         podem colidir num nome só"
    );
}

/// **As três caladas continuam caladas** — o controlo negativo do braço vazio.
#[test]
fn the_three_deliberately_silent_arms_publish_nothing() {
    for f in ["Value", "Flag", "Text"] {
        assert!(
            signal_name(&sample(f)).is_none(),
            "AuthoredIntent::{f} passou a publicar um sinal. Se é de propósito, a linha \
             correspondente do `SILENT` tem de sair."
        );
    }
}
