//! ⭐ **Uma linha de um menu da BARRA, na cara da maquete C** (onda 8; `.c15it`: grelha
//! `16px | rótulo | atalho | 10px`, vão 7, `min-height:23px`, `padding:0 6px`, raio 5).
//!
//! Irmão do [`super::context_menu_overlay`] (que arruma o cartão e as linhas de todos os menus): este
//! responde *«como se pinta UMA linha de um menu da barra»* — ícone nítido à esquerda, rótulo, atalho
//! numa coluna à direita, «›» na última coluna, e a linha acesa no acento (o caminho da cascata e a
//! linha sob o rato). Os tons saem de [`super::menu_tons`], com contraste nos 12 temas.

use super::menu_tons::{TonsDoMenu, atalho_aceso};
use crate::icons::IconId;
use crate::interaction::{ContextMenuKind, WidgetStore};
use crate::paint::{fill_rounded_rect, folga_do_texto, paint_icon_nitido, paint_text_in_weight};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{BAR_ICON_PX, MENU_W_PX, Spacing, Theme, TypeToken};
use ph2d_vector::{Color, VectorScene};

/// **Este menu veste a cara da barra?** Os menus da barra, a seta do Play, e todo submenu aberto a
/// partir deles (Edit ▸ Preferences ▸ …, View ▸ Theme): a cascata inteira com uma cara só.
#[must_use]
pub fn veste(store: &WidgetStore, kind: ContextMenuKind) -> bool {
    super::menu_bar::is_bar_menu(kind)
        || kind == ContextMenuKind::MenuBarPlay
        || store.menu_chain().first().is_some_and(|r| {
            super::menu_bar::is_bar_menu(r.kind) || r.kind == ContextMenuKind::MenuBarPlay
        })
}

/// A largura do menu `kind`: a da maquete (`chrome.menu-w`) nos da barra, a de sempre nos outros.
#[must_use]
pub fn largura(store: &WidgetStore, kind: ContextMenuKind) -> f32 {
    if veste(store, kind) {
        MENU_W_PX
    } else {
        super::context_menu_overlay::MENU_W
    }
}

/// O glifo da linha `id`: a tabela da barra, o que a família publicou, ou o do módulo que a soma.
#[must_use]
pub fn icone_da_linha(
    store: &WidgetStore,
    id: NodeId,
    contrib: &[crate::widget::ToolRailEntry],
) -> Option<IconId> {
    super::menu_icones::icone(id)
        .or_else(|| store.doc_menu_icon(id))
        .or_else(|| {
            contrib.iter().find_map(|e| match e {
                crate::widget::ToolRailEntry::Icon { id: eid, icon, .. } if *eid == id => {
                    Some(*icon)
                }
                _ => None,
            })
        })
}

/// O peso do texto dos menus da barra, EXATO (sem o acréscimo da preferência do app). Dono 13/11: «mais leve,
/// sem perder definição em 1920×1080». Medido (`ferramentas/regua_texto.py`, traço que cai cheio num pixel): antes
/// 25–33 %, 400 → 6–7 % (cinza, fraco), 500 → 13–15 %, 550 → 16–17 %, 600 → 13–16 %; VS Code 20 %, Godot 14 %.
pub const PESO: FontWeight = FontWeight::new(550.0); // LITERAL-PX-OK: eixo `wght` da fonte, medido (acima)

/// O que vai na coluna do glifo.
#[derive(Clone, Copy, Debug)]
pub enum Glifo {
    Icone(IconId),
    /// A escolha corrente de uma lista de VALORES sem ícone (Pixels per Meter ▸ 32): o visto no acento.
    Escolha,
    /// A amostra de um tema (pintada por quem a conhece).
    Tema(Theme),
    Amostra([u8; 4]),
    Nada,
}

/// Uma linha a pintar.
#[derive(Clone, Copy, Debug)]
pub struct Linha<'a> {
    pub r: Rect,
    pub rotulo: &'a str,
    pub atalho: Option<&'a str>,
    pub glifo: Glifo,
    pub abre_submenu: bool,
    /// Ligada (um painel à vista, o modo do Play): o visto na coluna da direita, o ícone fica.
    pub ligada: bool,
    /// O caminho da cascata ou a linha sob o rato.
    pub acesa: bool,
    pub cinzenta: bool,
}

/// A caixa do glifo de uma linha (16 px ao meio da altura, depois do respiro).
#[must_use]
pub fn caixa_do_glifo(r: Rect) -> Rect {
    Rect::new(
        r.x + Spacing::Sm.px(),
        r.y + (r.h - BAR_ICON_PX) * 0.5,
        BAR_ICON_PX,
        BAR_ICON_PX,
    )
}

/// Onde começa o rótulo (a coluna do glifo existe em TODA linha, haja glifo ou não: os rótulos
/// alinham, como na maquete).
#[must_use]
pub fn x_do_rotulo(r: Rect) -> f32 {
    caixa_do_glifo(r).x + BAR_ICON_PX + Spacing::Md.px()
}

/// A coluna do «›» (a última da grelha), reservada em toda linha: os atalhos acabam todos no mesmo x.
fn coluna_do_chevron(r: Rect) -> Rect {
    let lado = Spacing::Lg.px();
    Rect::new(
        r.x + r.w - Spacing::Sm.px() - lado,
        r.y + (r.h - lado) * 0.5,
        lado,
        lado,
    )
}

/// ⭐ Pinta a linha. `amostra` pinta a coluna do glifo quando ela é um tema ou uma cor (o pintor dos
/// menus sabe desenhá-las; aqui só se reserva o lugar).
pub fn pinta(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    t: &TonsDoMenu,
    l: Linha<'_>,
    amostra: impl FnOnce(&mut VectorScene, Rect),
) {
    let acesa = l.acesa && !l.cinzenta;
    if acesa {
        fill_rounded_rect(scene, l.r, t.raio, t.aceso);
    }
    let (rotulo, atalho, icone) = match (l.cinzenta, acesa) {
        (true, _) => (t.cinza, t.cinza, t.cinza),
        (false, true) => (t.aceso_texto, atalho_aceso(t), t.aceso_texto),
        (false, false) => (t.rotulo, t.atalho, t.icone),
    };
    let caixa = caixa_do_glifo(l.r);
    match l.glifo {
        Glifo::Icone(i) => paint_icon_nitido(scene, i, caixa, icone, false),
        Glifo::Escolha => {
            let visto: Color = if acesa { t.aceso_texto } else { t.aceso };
            paint_icon_nitido(scene, IconId::Check, caixa, visto, false);
        }
        Glifo::Tema(_) | Glifo::Amostra(_) => amostra(scene, caixa),
        Glifo::Nada => {}
    }
    let font = TypeToken::Sm.px();
    let y = l.r.y + folga_do_texto(l.r.h, font);
    let chev = coluna_do_chevron(l.r);
    let fim = chev.x - Spacing::Md.px();
    let fim_do_rotulo = match l.atalho {
        Some(k) => {
            let kw = ph2d_text::com_peso_exato(|| text_system.prefix_width_weighted(k, font, PESO));
            paint_text_in_weight(
                text_system,
                scene,
                k,
                [fim - kw, y],
                font,
                kw + 1.0,
                atalho,
                PESO,
            );
            fim - kw - Spacing::Lg.px()
        }
        None => fim,
    };
    let x = x_do_rotulo(l.r);
    // Peso EXATO (sem o acréscimo da preferência do app) e uma linha com «…» (`ElideToOne`).
    paint_text_in_weight(
        text_system,
        scene,
        l.rotulo,
        [x, y],
        font,
        (fim_do_rotulo - x).max(0.0),
        rotulo,
        PESO,
    );
    if l.abre_submenu {
        paint_icon_nitido(scene, IconId::ChevronRight, chev, icone, false);
    } else if l.ligada {
        let visto: Color = if acesa { t.aceso_texto } else { t.aceso };
        let lado = BAR_ICON_PX;
        let caixa = Rect::new(
            chev.x + chev.w - lado,
            l.r.y + (l.r.h - lado) * 0.5,
            lado,
            lado,
        );
        paint_icon_nitido(scene, IconId::Check, caixa, visto, false);
    }
}

#[cfg(test)]
#[path = "menu_barra_linha_tests.rs"]
mod tests;
