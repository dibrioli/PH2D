//! **O interruptor *Show as Panel* escreve a visibilidade REAL do painel autorado** (plano UI/UX
//! W8b.2) — e a lê de volta da mesma.
//!
//! # Por que um arch-gate, e não um teste de comportamento
//!
//! A decisão mora dentro de `render_loop`, cujo laço exige `AppGfx` — ou seja, uma janela e uma
//! GPU. **Nenhum teste de unidade a alcança**, e é exactamente a classe de costura que este repo
//! já viu shipar quebrada com toda a suíte verde (o painel de física do W2b, que o `W` alternava
//! sem que o painel existisse). O gate lê o FONTE do produto, que é a única vista que pode dizer a
//! verdade aqui.
//!
//! # A propriedade, e não o endereço
//!
//! Ele afirma três coisas — que o clique ESCREVE, que a publicação LÊ, e que **as duas passam pela
//! porta `visibility_key()` em vez de um literal**. A terceira é a que importa: um literal seria
//! uma segunda resposta a *"como este painel se chama no mapa de visibilidade?"*, e ela ficaria
//! para trás em silêncio no dia em que o `Panel::ID` mudasse — com o chip a alternar a
//! visibilidade de um painel que não existe.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// Controle positivo: se a varredura não achar o arquivo do produto, todo gate abaixo seria verde
/// por vácuo.
fn body() -> &'static str {
    assert!(
        SRC.len() > 10_000,
        "o fonte do render_loop nao foi lido — os gates abaixo seriam verdes por vacuo"
    );
    SRC
}

/// **O clique nos chips escreve a visibilidade do painel autorado.**
#[test]
fn the_switch_writes_the_panels_visibility() {
    let s = body();
    let arm = s
        .find("ph2d_editor::ids::VECTOR_FRAME_PANEL_OFF")
        .expect("o braco do chip Show as Panel sumiu do render_loop");
    // A janela é o braço do `else if` — o `insert` tem de estar DENTRO dele, não em qualquer
    // lugar do arquivo.
    let window = &s[arm..(arm + 1200).min(s.len())];
    assert!(
        window.contains("panel_visibility"),
        "o braco do chip nao escreve `panel_visibility` — o interruptor acende e nao abre nada"
    );
    assert!(
        window.contains("ph2d_panel_authored::visibility_key()"),
        "o braco do chip usa uma chave que nao e' a porta `visibility_key()` — um literal aqui \
         fica para tras quando o `Panel::ID` mudar, em silencio"
    );
}

/// **E o chip LÊ de volta a mesma visibilidade** — um fato, um lugar.
///
/// ⚠️ Sem esta metade o chip mostraria uma cópia própria: fechar pelo X do painel deixaria o
/// interruptor aceso, e clicá-lo não faria nada (ele já "estaria" no estado pedido).
#[test]
fn the_switch_reads_back_the_same_visibility() {
    let s = body();
    let call = s
        .find("set_frame_panel_open(")
        .expect("a publicacao do estado do interruptor sumiu do render_loop");
    let window = &s[call..(call + 400).min(s.len())];
    assert!(
        window.contains("is_panel_visible(ph2d_panel_authored::visibility_key())"),
        "o interruptor nao le' a visibilidade REAL do painel — ele passou a mostrar uma copia"
    );
}

/// **A chave nunca é um literal, em lugar nenhum deste arquivo.**
///
/// ⚠️ **A primeira versão deste gate ENUMERAVA formatações** (`panel_visibility.insert("authored"`
/// e `is_panel_visible("authored")`) e por isso não podia falhar pelo motivo que alegava: a
/// mutação que trocou a porta pelo literal foi escrita em várias linhas — o `rustfmt` a quebrou —
/// e o gate ficou VERDE sobre exactamente o defeito que ele nomeia. A pergunta certa não tem
/// forma: *este literal aparece aqui?*
#[test]
fn the_visibility_key_is_never_spelled_out() {
    let s = body();
    assert!(
        !s.contains("\"authored\""),
        "a chave do painel autorado foi escrita a' mao neste arquivo — use `visibility_key()`, \
         senao ela fica para tras no dia em que o `Panel::ID` mudar, em silencio"
    );
}

/// **O pill UI e o chip escrevem o MESMO fato** (W8b.3).
///
/// ⚠️ O handler do pill mora na `ph2d-editor-core`, que **não pode** depender de
/// `ph2d-panel-authored` (os painéis dependem dela, nunca o contrário) — então a chave lá é um
/// literal, obrigatoriamente. Este gate é o que impede o literal de divergir do `Panel::ID`: ele
/// dirige o pill pela porta REAL do chrome e pergunta ao painel qual é a chave dele. Sem ele, um
/// rename do `Panel::ID` deixaria o pill a alternar a visibilidade de um painel que não existe —
/// em silêncio, que é a cicatriz do painel de física do W2b.
#[test]
fn the_ui_pill_writes_the_key_the_panel_answers() {
    use ph2d_editor::interaction::WidgetEvent;
    use ph2d_editor::screens::hero::{HeroScreen, chrome, ids};

    let mut hero = HeroScreen::new(ph2d_editor::NodeId(1));
    let key = ph2d_panel_authored::visibility_key();
    assert!(!hero.is_panel_visible(key), "nasce fechado");

    assert!(
        chrome::dispatch_all(&mut hero, WidgetEvent::Click(ids::TOPBAR_AUTHORED)),
        "o clique no pill tem de ser consumido"
    );
    assert!(
        hero.is_panel_visible(key),
        "o pill escreveu uma chave que NAO e' a do painel — o literal do chrome divergiu do \
         `Panel::ID`, e o pill alterna a visibilidade de um painel que nao existe"
    );
}
