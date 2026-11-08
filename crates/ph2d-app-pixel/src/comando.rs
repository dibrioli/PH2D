//! **P12 R — Run Command**: as linhas do Pixel na paleta `Ctrl+K` do app (`P12_plano.md` §9.2,
//! oráculo O10). Nome = o da acção, tecla = a do mapa VIVO, correr = a tecla
//! ([`crate::input_teclas::acao`]); sem sprite em Draw ou fora do contexto = cinzenta e inerte.

use ph2d_editor_core::NodeId;
use ph2d_editor_core::widget::command_palette::{PaletteGroup, PaletteItem, PaletteSub};
use ph2d_pixel_keys::{Context, PixelAction as A};
use ph2d_tokens::ColorToken;

use crate::PixelApp;
use crate::input::KeyAnswer;

const BASE: u64 = 0x9d1c_3a7e_5b00_0000;

/// O id da linha de `a` na paleta.
#[must_use]
pub fn id(a: A) -> NodeId {
    NodeId(BASE + A::ALL.iter().position(|&x| x == a).unwrap_or(0) as u64 + 1)
}

/// A acção da linha `id`, se é do Pixel.
#[must_use]
pub fn action_of(id: NodeId) -> Option<A> {
    let i = id.0.checked_sub(BASE + 1)?;
    A::ALL.get(usize::try_from(i).ok()?).copied()
}

/// O rótulo: o nome da acção e as teclas dela no mapa vivo («New Layer · Shift+N»).
#[must_use]
pub fn label(app: &PixelApp, a: A) -> String {
    let keys: Vec<String> = app
        .keys
        .keys_of(a)
        .iter()
        .map(|(c, _)| c.to_string())
        .collect();
    let sep = if keys.is_empty() { "" } else { " \u{00b7} " };
    format!("{}{sep}{}", a.name(), keys.join(", "))
}

/// Corre agora? Há sprite em Draw e uma tecla dela vale no contexto (a mesma lei do `resolve`).
#[must_use]
pub fn available(app: &PixelApp, a: A) -> bool {
    let Some(key) = app.chave else { return false };
    let ctx = crate::input_teclas::contexto(app, key);
    let keys = app.keys.keys_of(a);
    let normal = ctx != Context::Stroking || a.while_stroking();
    keys.is_empty()
        || keys
            .iter()
            .any(|&(_, c)| c == ctx || (c == Context::Normal && normal))
}

/// O grupo do Pixel na paleta (`None` sem nenhum Pixel Sprite no projecto).
pub fn group(app: &mut PixelApp) -> Option<PaletteGroup> {
    (!app.docs.is_empty()).then_some(())?;
    let (on, off): (Vec<A>, Vec<A>) = A::ALL.iter().copied().partition(|&a| available(app, a));
    let sub = |title: Option<String>, v: Vec<A>| {
        let items = v.into_iter().map(|a| PaletteItem {
            label: label(app, a),
            id: id(a),
            keywords: "",
        });
        PaletteSub {
            title,
            items: items.collect(),
        }
    };
    let subs = [
        sub(None, on),
        sub(Some(ph2d_i18n::tr("pixel.command.unavailable").into()), off),
    ];
    Some(PaletteGroup {
        title: ph2d_i18n::tr("chrome.menu.pixel").into(),
        color: ColorToken::NodeCatSource,
        subs: subs.into_iter().filter(|s| !s.items.is_empty()).collect(),
    })
}

/// Corre a linha `id` como a tecla dela; `None` = não é do Pixel ou não está disponível.
pub fn run(app: &mut PixelApp, id: NodeId) -> Option<KeyAnswer> {
    let a = action_of(id).filter(|&a| available(app, a))?;
    crate::input_teclas::acao(app, app.chave?, a)
}
