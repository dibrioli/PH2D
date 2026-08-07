//! **O interop DTCG do painel de Tokens** (plano UI/UX W9) — irmão do [`super::tokens_bridge`], e
//! o corte é por assunto: aqui mora *como a tabela sai para um arquivo e volta dele*.
//!
//! # A metade que DECIDE e a metade que faz I/O são funções diferentes
//!
//! Um diálogo nativo não corre num teste de unidade, então tudo o que é **política** — que modo
//! recebe, o que se mantém dos outros, por que porta se escreve, o que o artista lê no fim — vive
//! na [`install`], que um gate dirige sem tocar num arquivo. O que sobra nas duas funções públicas
//! é escolher o caminho e ler/escrever bytes, e isso tem um arch-gate.
//!
//! ⚠️ Sem esse corte a wave inteira seria *"provada"* por um gate que só consegue afirmar que a
//! crate do codec funciona — e a costura, que é onde as waves desta linha falham, ficaria de fora.

use std::path::Path;

use ph2d_editor::{Toast, ToastQueue};
use ph2d_tokens::Theme;
use ph2d_tokens::num_overrides::{num_overrides, set_num_overrides};
use ph2d_tokens::overrides::{color_overrides, set_color_overrides};
use ph2d_tokens_dtcg::Imported;

/// A extensão que o diálogo oferece. `.json` porque é o que o ecossistema escreve; o `.tokens.json`
/// é convenção de NOME, não de extensão.
const FILTER: (&str, &[&str]) = ("DTCG tokens", &["json"]);

/// **A tabela deste modo SAI.** Não muda o documento, então não devolve nada.
pub(crate) fn export(theme: Theme, toasts: &mut ToastQueue) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter(FILTER.0, FILTER.1)
        .set_file_name(default_name(theme))
        .save_file()
    else {
        return; // O artista desistiu — desistir de um gesto não é um erro a anunciar.
    };
    let body = ph2d_tokens_dtcg::export(theme);
    match std::fs::write(&path, &body) {
        Ok(()) => toasts.push(Toast::success(format!(
            "DTCG exported: {} tokens to {}",
            ph2d_tokens::ColorToken::ALL.len() + ph2d_tokens::NumToken::ALL.len(),
            file_name(&path)
        ))),
        // ⚠️ A frase do SO é repassada inteira: *"não deu"* sem o porquê manda o artista adivinhar
        // se o disco está cheio, se a pasta é só de leitura, ou se o app está partido.
        Err(e) => toasts.push(Toast::warning(format!("DTCG export failed: {e}"))),
    };
}

/// **Um `.tokens.json` re-veste o modo vigente.** Devolve `true` se a camada mudou.
pub(crate) fn import(theme: Theme, toasts: &mut ToastQueue) -> bool {
    let Some(path) = rfd::FileDialog::new()
        .add_filter(FILTER.0, FILTER.1)
        .pick_file()
    else {
        return false;
    };
    let body = match std::fs::read_to_string(&path) {
        Ok(b) => b,
        Err(e) => {
            toasts.push(Toast::warning(format!("DTCG import failed: {e}")));
            return false;
        }
    };
    match ph2d_tokens_dtcg::import(&body, theme) {
        Ok(imported) => {
            let changed = install(theme, imported.clone());
            toasts.push(Toast::success(report(&imported)));
            changed
        }
        // ⚠️ O erro do codec traz linha e coluna — repassá-lo inteiro é o que torna um arquivo
        // meio-escrito consertável em vez de misterioso.
        Err(e) => {
            toasts.push(Toast::warning(format!("DTCG import failed: {e}")));
            false
        }
    }
}

/// **A POLÍTICA: o modo vigente é substituído, os outros três ficam.**
///
/// ⚠️ **Substituído, não somado.** Um import é *"a tabela deste modo passa a ser esta"* — somar
/// deixaria de pé um token que o arquivo não menciona, e o artista veria uma re-vestida meio-nova
/// e meio-velha sem nada dizer de onde a metade antiga veio. É a mesma lei do load de projeto: o
/// que instala a lista inteira é o que faz o app ESQUECER.
///
/// ⚠️ **E só o vigente**, pelo motivo do *Reset This Mode*: apagar trabalho num modo que o artista
/// não está a olhar é a forma mais barata de perder uma re-vestida.
///
/// ⚠️ As duas portas correm **sempre**, mesmo com a lista vazia — pular a família que este arquivo
/// não usa deixaria a escala do arquivo ANTERIOR de pé sob as cores do novo.
pub(crate) fn install(theme: Theme, imported: Imported) -> bool {
    let others: Vec<_> = color_overrides()
        .into_iter()
        .filter(|e| e.theme != theme)
        .collect();
    let mine_before: Vec<_> = color_overrides()
        .into_iter()
        .filter(|e| e.theme == theme)
        .collect();
    let mut colours = others;
    colours.extend(imported.colours.iter().cloned());
    set_color_overrides(colours);

    let others: Vec<_> = num_overrides()
        .into_iter()
        .filter(|e| e.theme != theme)
        .collect();
    let nums_before: Vec<_> = num_overrides()
        .into_iter()
        .filter(|e| e.theme == theme)
        .collect();
    let mut nums = others;
    nums.extend(imported.nums.iter().cloned());
    set_num_overrides(nums);

    // ⚠️ A comparação é contra o que ESTE MODO tinha, e sai das PORTAS (que devolvem a lista em
    // ordem canônica) — não da lista que chegou do arquivo. Uma entrada que a porta de escrita
    // recusou (um laço) não mudou nada, e contá-la marcaria o projeto sujo por um import que não
    // pegou.
    let mine_after: Vec<_> = color_overrides()
        .into_iter()
        .filter(|e| e.theme == theme)
        .collect();
    let nums_after: Vec<_> = num_overrides()
        .into_iter()
        .filter(|e| e.theme == theme)
        .collect();
    mine_before != mine_after || nums_before != nums_after
}

/// **O que o artista lê.** As três contagens são três fatos diferentes, e só aparecem quando são
/// diferentes de zero: uma linha que diz sempre *"0 desconhecidos"* é uma linha que se aprende a
/// não ler, e no dia em que ela tiver conteúdo ninguém olha para lá.
fn report(r: &Imported) -> String {
    let mut s = format!("DTCG imported: {} token(s) authored", r.authored());
    if r.at_factory > 0 {
        s.push_str(&format!(", {} already at factory", r.at_factory));
    }
    if r.unknown > 0 {
        s.push_str(&format!(", {} unknown", r.unknown));
    }
    if r.dropped > 0 {
        s.push_str(&format!(", {} unusable", r.dropped));
    }
    s
}

/// O nome que o diálogo propõe — com o MODO dentro, porque um arquivo é de um modo.
fn default_name(theme: Theme) -> String {
    let mode = match theme {
        Theme::Forge => "forge",
        Theme::Workshop => "workshop",
        Theme::Sunstone => "sunstone",
        Theme::Blueprint => "blueprint",
    };
    format!("ph2d-{mode}.tokens.json")
}

/// Só o nome do arquivo — um caminho inteiro num toast quebra a coluna e some do lado direito.
fn file_name(p: &Path) -> String {
    p.file_name()
        .map_or_else(|| p.display().to_string(), |n| n.to_string_lossy().into())
}

#[cfg(test)]
#[path = "tokens_bridge_dtcg_tests.rs"]
mod tests;
