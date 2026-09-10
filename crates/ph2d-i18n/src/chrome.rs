//! **AS STRINGS DA MOLDURA DO APP** — o irmão de tabela do [`super`], para o que **não** é de um
//! painel: diálogos, modais, a paleta de comandos, a barra do topo, o selector de cor.
//!
//! ⚠️ **Um corte por ASSUNTO, como o `sculpt3d.rs` e o `model3d.rs`** — e pela mesma razão de
//! isolamento (`CLAUDE.md` §0.2): enquanto todas as chaves moram num `match` só, duas linhas
//! paralelas que acrescentem uma chave cada colidem no mesmo punhado de linhas.
//!
//! # ⭐⭐ Porque esta tabela não é só canon: ela COLAPSOU rótulos duplicados
//!
//! O censo de 2026-09-10 (`scripts/censo-texto-pintado.py`) mostrou o mesmo rótulo escrito em dois
//! pintores diferentes — e **quatro** vezes:
//!
//! | rótulo | escrito em |
//! |---|---|
//! | *No matches* | `context_menu_overlay.rs` **e** `widget/command_palette.rs` |
//! | *Shape options* | `screens/hero/left_rail.rs` **e** `screens/hero/tool_bar.rs` |
//! | *Mask options* | os mesmos dois |
//! | *Hex* | `blender_color_picker/hex_field.rs` **e** `blender_color_picker/paint.rs` |
//!
//! ⇒ *uma palavra escrita em dois sítios ainda não é uma palavra do app — só uma PORTA é.* Mudar
//! *No matches* para outra frase tocava um dos dois e deixava o outro a dizer a antiga, e nenhum
//! gate o via. **8 sítios passaram a 4 chaves.**
//!
//! ⚠️ **`Option` e não `&str`:** devolver a chave crua aqui seria uma SEGUNDA resposta a *«o que
//! fazer com uma chave desconhecida?»* — o `leak_key` do pai responde isso, uma vez.

/// A tradução de uma chave `chrome.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ⭐ **As quatro que existiam em DUPLICADO** — a razão de esta tabela existir.
        "chrome.no_matches" => "No matches",
        "chrome.flyout.shape_options" => "Shape options",
        "chrome.flyout.mask_options" => "Mask options",
        "chrome.color.hex" => "Hex",

        // Os dois verbos que fecham um modo — a barra do prefab, e quem vier a seguir.
        "chrome.cancel" => "Cancel",
        "chrome.done" => "Done",

        // Os diálogos de criação (`context_menu_dialogs.rs`).
        "chrome.dialog.new_image" => "New Image",
        "chrome.dialog.new_sheet" => "New Sprite Sheet",
        "chrome.dialog.size" => "Size",
        "chrome.dialog.background" => "Background",
        "chrome.dialog.resolution" => "Resolution",

        "chrome.fill.title" => "Fill",
        // ⚠️ **Abreviado de propósito**, como os rótulos das ferramentas de imagem: ele vive num
        // chip da barra do topo, e o nome por extenso não cabe na coluna.
        "chrome.topbar.image_chip" => "IMG",
        "chrome.color.read_only" => "Read-only",
        // As duas sondas do inspector da grelha.
        "chrome.grid_snap.probe_a" => "Probe A",
        "chrome.grid_snap.probe_b" => "Probe B",
        _ => return None,
    })
}
