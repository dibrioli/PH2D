//! ⭐⭐⭐ **O CARET pergunta ao PINTOR onde o texto começa — três braços ainda copiavam a conta.**
//!
//! O mapeador de clique→byte (`text_ops::byte_offset_from_click_xy`) tem um braço por classe de
//! campo de texto, e cada um precisa de saber **onde o pintor pôs a primeira letra**. Três deles
//! respondiam com os números escritos à mão:
//!
//! | braço | o pintor desenha em | o caret procurava em |
//! |---|---|---|
//! | `TextInput` de uma linha | `rect.x + Spacing::Lg.px()` | `rect.x + 12.0` |
//! | `NumberInput` | `rect.x + Spacing::Lg.px()` | `rect.x + 12.0` |
//! | `Combobox` | `+ Lg + ícone + Md` | `+ 12.0 + ícone + 8.0` |
//! | campo `Hex` | `rect.x + Spacing::Md.px() + 36` | `rect.x + 8.0 + 36.0` |
//!
//! ⛔⛔ **Os números da direita são os valores de FÁBRICA dos da esquerda**, e é isso que torna o
//! defeito invisível: enquanto o artista não autora a escala, as duas contas dão o mesmo. Desde
//! que a escala numérica virou autorável, o pintor lê o valor **vivo** e a cópia não — logo mexer
//! no `spacing.lg` faz o utilizador clicar numa letra e escrever noutra.
//!
//! ⚠️⚠️ **A família já tinha sido diagnosticada e curada pela METADE.** O `TextArea` ganhou a porta
//! dele (`text_area_metrics`) com este mecanismo escrito ao lado, em prosa, e o gate que o prova
//! está a três ficheiros daqui — e os outros **três braços do mesmo `match`** ficaram a copiar.
//! *Curar um braço de uma família deixa os outros com o defeito E com a aparência de resolvidos.*
//!
//! # A forma do gate, herdada do braço curado
//!
//! Cada teste corre **duas vezes**: com a escala de **fábrica** (o CONTROLE — é o mundo em que a
//! cópia acerta por coincidência, e sozinho ele é verde sobre o produto quebrado) e com a escala
//! **autorada** (o discriminador). Juntos dizem a propriedade que importa: *a resposta não depende
//! de a escala ser a de fábrica.*
//!
//! ⚠️ E a posição do clique sai da **porta do widget**, nunca de uma terceira régua escrita aqui —
//! pedir a conta a uma cópia dentro do gate que mede cópias seria repetir o defeito no medidor.

use super::*;
use ph2d_tokens::num::NumToken;
use ph2d_tokens::num_overrides::{NumValue, clear_num_overrides, set_num_override};
use ph2d_tokens::{Spacing, Theme, num_runtime};

/// Corre `f` com a escala de fábrica e depois com `token` autorado em `valor`.
fn com_as_duas_escalas<T>(token: NumToken, valor: f32, f: impl Fn() -> T) -> (T, T) {
    clear_num_overrides();
    num_runtime::publish(Theme::Forge);
    let fábrica = f();

    set_num_override(Theme::Forge, token, Some(NumValue::Literal(valor)))
        .expect("o override da escala foi recusado");
    num_runtime::publish(Theme::Forge);
    let autorada = f();

    clear_num_overrides();
    num_runtime::publish(Theme::Forge);
    (fábrica, autorada)
}

fn caret_de(store: &WidgetStore, id: NodeId) -> usize {
    match store.get(id) {
        Some(InteractiveState::TextInput { caret, .. }) => *caret,
        Some(InteractiveState::Combobox { caret, .. }) => *caret,
        Some(InteractiveState::NumberInput { caret, .. }) => *caret,
        _ => usize::MAX,
    }
}

/// ⭐ **Um campo de texto de uma linha.**
///
/// Clicar **antes** da primeira letra tem de pôr o caret no byte `0` — seja qual for o recuo que a
/// escala autorada dá ao campo. Com a cópia de fábrica e `spacing.lg = 40`, o pintor empurra o
/// texto para a direita e o despacho continua a medir a partir de `+12`: o clique cai já **dentro**
/// das letras e o caret salta para o meio da palavra.
#[test]
fn a_single_line_field_puts_the_caret_where_the_painter_drew_even_with_an_authored_scale() {
    let clique_antes_da_primeira_letra = || {
        let (mut store, hits, rect) = textarea_setup("abcdefgh");
        let arena = Bump::new();
        let x = crate::widget::text_input_text_origin_x(rect);
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, x + 1.0, rect.y + rect.h * 0.5),
            &arena,
        );
        caret_de(&store, NodeId(42))
    };

    let (fábrica, autorada) = com_as_duas_escalas(
        NumToken::Spacing(Spacing::Lg),
        40.0,
        clique_antes_da_primeira_letra,
    );
    assert_eq!(
        fábrica, 0,
        "controle: com a escala de fabrica o clique na 1.a letra ja' devia dar caret 0"
    );
    assert_eq!(
        autorada, 0,
        "com `spacing.lg` autorado o caret caiu no byte {autorada} — o despacho procura o texto \
         onde o pintor ja' nao o desenha, e o artista clica numa letra e escreve noutra"
    );
}

/// ⭐ **O combobox** — e aqui a cópia tinha DUAS metades erradas (o recuo e o vão do ícone), mais
/// uma terceira cópia da fórmula do tamanho do ícone.
#[test]
fn the_combobox_caret_follows_the_search_icon_even_with_an_authored_scale() {
    let clique_antes_da_primeira_letra = || {
        let (mut store, hits, rect) = combobox_setup("abcdefgh");
        let arena = Bump::new();
        let x = crate::widget::combobox_text_origin_x(rect);
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, x + 1.0, rect.y + rect.h * 0.5),
            &arena,
        );
        caret_de(&store, NodeId(55))
    };

    let (fábrica, autorada) = com_as_duas_escalas(
        NumToken::Spacing(Spacing::Md),
        24.0,
        clique_antes_da_primeira_letra,
    );
    assert_eq!(fábrica, 0, "controle: escala de fabrica devia dar caret 0");
    assert_eq!(
        autorada, 0,
        "com `spacing.md` autorado o caret do combobox caiu no byte {autorada}: o vao entre o \
         icone de busca e o texto mudou no pintor e nao no despacho"
    );
}

/// ⛔⛔ **A porta do combobox é UMA, e o botão de limpar lê a mesma.**
///
/// A fórmula do tamanho do ícone vivia em **três** cópias — o pintor, o `clear_button_rect` e o
/// mapeador de clique, este último com os limites em literais. *Três cópias de uma conta são três
/// leis que hoje concordam.*
#[test]
fn the_combobox_icon_size_has_one_owner() {
    let host = Rect::new(0.0, 0.0, 240.0, 32.0);
    let cb = crate::widget::Combobox::new(NodeId(1), "x", Vec::new()).query("q");
    let clear = cb
        .clear_button_rect(host)
        .expect("com query nao vazia o botao de limpar existe");
    assert_eq!(
        clear.w,
        crate::widget::combobox_icon_size(host),
        "o botao de limpar deixou de usar a porta do tamanho do icone"
    );
    // E o texto começa depois de um ícone DESSE tamanho, contado do recuo do campo.
    assert_eq!(
        crate::widget::combobox_text_origin_x(host),
        host.x
            + crate::widget::field_pad_x()
            + crate::widget::combobox_icon_size(host)
            + ph2d_tokens::icon_label_gap_px()
    );
}

/// ⛔⛔⛔ **O MAPEADOR DE CARET não escreve NÚMEROS de geometria — ele pergunta.**
///
/// Esta é a lei que os quatro braços violavam, e é a única régua que a apanharia **antes** de
/// alguém autorar a escala: todo número de pixel ali era, por construção, uma cópia do valor de
/// **fábrica** de um token que o pintor lê **vivo**.
///
/// ⚠️ A régua tolera exactamente duas coisas, e as duas são declaradas: o `0.0` de um `max`/`min`
/// (não é geometria, é o piso de uma subtracção) e a razão de avanço aproximado, que tem nome
/// (`APPROX_ADVANCE_RATIO`) e doc próprio. *Uma isenção sem nome vira a porta pela qual o defeito
/// regressa.*
#[test]
fn the_caret_mapper_writes_no_pixel_of_its_own() {
    use std::fs;
    use std::path::PathBuf;

    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/interaction/dispatch/text_ops.rs");
    let src = fs::read_to_string(&path).expect("text_ops.rs");

    let mut strays = Vec::new();
    for (n, line) in src.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with("//") || t.contains("APPROX_ADVANCE_RATIO") {
            continue;
        }
        let bytes = line.as_bytes();
        for (i, w) in bytes.windows(2).enumerate() {
            // um literal de vírgula flutuante: dígito, ponto, dígito
            if w[0] == b'.' && w[1].is_ascii_digit() && i > 0 && bytes[i - 1].is_ascii_digit() {
                // reconstrói o número para deixar passar o `0.0`
                let start = line[..i]
                    .rfind(|c: char| !c.is_ascii_digit())
                    .map_or(0, |k| k + 1);
                let lit: String = line[start..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                if lit != "0.0" {
                    strays.push(format!("{}: {lit} em `{}`", n + 1, t));
                }
            }
        }
    }
    assert!(
        strays.is_empty(),
        "o mapeador de clique->caret voltou a escrever geometria a` mao — e todo numero ali e' o \
         valor de FABRICA de um token que o pintor le^ VIVO:\n  {}",
        strays.join("\n  ")
    );
}

/// ⛔⛔ **A FÓRMULA do tamanho do ícone existe UMA vez no repo.**
///
/// ⚠️ **Este teste nasceu de uma MUTAÇÃO QUE SOBREVIVEU**: repor a cópia dentro do
/// `clear_button_rect` devolve exactamente o mesmo valor hoje, logo nenhum teste de
/// **comportamento** a distingue da porta. É a mesma espécie do piso do recuo na wave 23 — *uma
/// cópia que hoje concorda só se lê no texto*, e a única régua honesta é contar donos.
#[test]
fn the_icon_size_formula_has_exactly_one_owner_in_the_tree() {
    use std::fs;
    use std::path::{Path, PathBuf};

    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().and_then(|n| n.to_str()) != Some("target") {
                    walk(&p, out);
                }
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(p);
            }
        }
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let mut files = Vec::new();
    walk(&root.join("crates"), &mut files);
    walk(&root.join("shells"), &mut files);

    const NEEDLE: &str = "clamp(14.0, 18.0)";
    let mut owners = Vec::new();
    for p in files {
        // ⚠️ **O censo tem de se EXCLUIR.** A agulha aparece literalmente neste ficheiro, e a 1.ª
        // corrida acusou o próprio gate como o segundo dono da fórmula. *Uma varredura de fonte
        // que não se exclui mede-se a si própria.*
        if p.file_name().and_then(|n| n.to_str()) == Some("caret_doors.rs") {
            continue;
        }
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        for (n, line) in src.lines().enumerate() {
            if line.contains(NEEDLE) && !line.trim_start().starts_with("//") {
                owners.push(format!(
                    "{}:{}",
                    p.strip_prefix(&root)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .replace('\\', "/"),
                    n + 1
                ));
            }
        }
    }
    assert_eq!(
        owners.len(),
        1,
        "a formula do tamanho do icone tem {} donos, e a porta e' `combobox::inline_icon_size`:\n  {}",
        owners.len(),
        owners.join("\n  ")
    );
    assert!(
        owners[0].contains("widget/combobox.rs"),
        "o unico dono da formula mudou de sitio: {}",
        owners[0]
    );
}

/// ⚠️ **A porta do campo é a MESMA para o campo de texto e para o numérico** — os dois pintores
/// escreviam `Spacing::Lg.px()` cada um, e o caret escrevia `12.0` para ambos.
#[test]
fn the_text_field_and_the_number_field_share_one_padding_door() {
    let rect = Rect::new(7.0, 0.0, 100.0, 22.0);
    assert_eq!(
        crate::widget::text_input_text_origin_x(rect),
        rect.x + crate::widget::field_pad_x()
    );
    assert_eq!(crate::widget::field_pad_x(), Spacing::Lg.px());
}
